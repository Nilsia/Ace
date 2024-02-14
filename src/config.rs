use crate::args::{Action, Args};
use crate::dependencies::{Dependencies, DependencyErrorType, UnSatisfiedGroup, UnSatisfiedTool};
use crate::editor::Editor;
use crate::group::Group;
use crate::package::{Package, PackageInfo, PackageResult};
use crate::tool::Tool;
use crate::utils::{
    create_dirs, existence, export_bin_dir, iter_includes, CYAN, GREEN, NC, YELLOW,
};
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const DEFAULT_FILENAME: &str = "config.toml";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub editor: Editor,
    pub tools: Option<HashMap<String, Tool>>,
    pub groups: Option<HashMap<String, Group>>,
    pub default_groups: Option<Vec<String>>,
    pub default_tools: Option<Vec<String>>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P, args: &mut Args) -> Result<Config> {
        // Check if config exists
        if path.as_ref().exists() {
            let config: Config = toml::from_str(&std::fs::read_to_string(path)?)?;
            match args.action {
                Action::List => Ok(config),
                _ => match config.validate(args) {
                    Ok(_) => {
                        let (mut missing_tools, mut missing_groups) = (Vec::new(), Vec::new());
                        if let (Some(d_tool), Some(tools)) =
                            (config.default_tools.as_ref(), config.tools.as_ref())
                        {
                            for tool_key in d_tool {
                                if !tools.contains_key(tool_key) {
                                    missing_tools.push(tool_key);
                                }
                            }
                        }

                        // check if all default grousp are in configuration
                        if let (Some(d_groups), Some(groups)) =
                            (config.default_groups.as_ref(), config.groups.as_ref())
                        {
                            for group_key in d_groups {
                                if !groups.contains_key(group_key) {
                                    missing_groups.push(group_key);
                                }
                            }
                        }

                        // check if all default tools are in donfiguration
                        if !missing_tools.is_empty() || !missing_groups.is_empty() {
                            let mut res = String::from(
                                "Tools or groups in default configuration are not present :\n\n",
                            );
                            if missing_tools.is_empty() {
                                res += "The following tools are not present :\n";
                                for tool in missing_tools {
                                    res += "\t - ";
                                    res += tool;
                                }
                            }
                            res += "\n\n";
                            if missing_groups.is_empty() {
                                res += "The following groups are not present :\n";
                                for group in missing_groups {
                                    res += "\t - ";
                                    res += group;
                                }
                            }
                            return Err(anyhow!(res));
                        }

                        // add defautl tools to args to be installed
                        if let Some(d_tools) = config.default_tools.as_ref() {
                            args.tools
                                .get_or_insert(d_tools.to_owned())
                                .extend_from_slice(d_tools);
                        }
                        // add default groups to args to be installed
                        if let Some(d_groups) = config.default_groups.as_ref() {
                            args.groups
                                .get_or_insert(d_groups.to_owned())
                                .extend_from_slice(d_groups);
                        }
                        Ok(config)
                    }
                    Err(e) => Err(e),
                },
            }
        } else {
            Err(anyhow!("No config file"))
        }
    }

    pub fn install(&self, args: &Args) -> Result<()> {
        // Create config/bin/data dirs and export bin to path
        create_dirs()?;
        let added = export_bin_dir()?;
        let mut installed: HashMap<String, PackageInfo> = HashMap::new();

        // Install editor
        if !args.except_editor {
            self.editor.install(args)?;
        }
        // Install required tools
        if !args.only_editor {
            let dependencies = self.get_dependencies(args)?;
            installed = HashMap::with_capacity(dependencies.satisfied_tools.len());
            dependencies
                .list_tools_for_action("\nThe following tools will be installed : ", "\n\n")?;
            for (tool_key, tool) in &dependencies.satisfied_tools {
                installed.insert(tool_key.to_string(), tool.install(args)?);
            }
        }

        let installed_str = if installed.len() == 0 {
            String::new()
        } else {
            installed
                .iter()
                .flat_map(|(key, (lib, config))| {
                    let data = vec![
                        match lib {
                            PackageResult::Installed => Some("lib: installed"),
                            PackageResult::Linked => Some("lib : linked"),
                            _ => None,
                        },
                        match config {
                            PackageResult::Installed => Some("config: installed"),
                            PackageResult::Linked => Some("config : linked"),
                            _ => None,
                        },
                    ]
                    .iter()
                    .flatten()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                    if data.is_empty() {
                        None
                    } else {
                        Some(format!("{GREEN}{key}{NC} ({})", data.join(", ")))
                    }
                })
                .collect::<Vec<String>>()
                .join(", ")
        };

        if added {
            println!("{GREEN}SUCCESS{NC}: Refresh your terminal for the changes to take effect (tools installed : {installed_str})");
        } else {
            println!("{GREEN}SUCCESS{NC} Your tools ({installed_str}) are righly installed in your system");
        }
        Ok(())
    }

    pub fn remove(&self, args: &Args) -> Result<()> {
        // Remove editor
        if !args.except_editor {
            self.editor.remove(args)?;
        }
        // Remove required tools
        if !args.only_editor {
            let dependencies = self.get_dependencies(args)?;
            dependencies
                .list_tools_for_action("\nThe following tools will be removed :", "\n\n")?;
            let mut removed = HashMap::with_capacity(dependencies.satisfied_tools.len());
            let mut res: PackageInfo;
            // this remove tools of groups also
            for (tool_key, tool) in &dependencies.satisfied_tools {
                res = tool.remove(args)?;
                removed.insert(tool_key.to_owned(), res);
            }
            println!(
                "The following tools have been removed : {}",
                removed
                    .iter()
                    .map(|(key, (lib, config))| {
                        let str = format!("{GREEN}{key}{NC}");
                        match (lib, config) {
                            (PackageResult::Removed, PackageResult::Removed) => {
                                Some(format!("{str} (config, lib)"))
                            }
                            (PackageResult::Removed, _) => Some(format!("{GREEN}{str}{NC} (lib)")),
                            (_, PackageResult::Removed) => Some(format!("{str} (config)")),
                            _ => None,
                        }
                    })
                    .flatten()
                    .collect::<Vec<String>>()
                    .join(", ")
            );
        }

        println!("{GREEN}SUCCESS{NC}");
        Ok(())
    }

    fn validate(&self, args: &Args) -> Result<()> {
        // check if user does not except editor and is valid
        if !args.except_editor {
            self.editor.validate()?;
        }

        // check if user has not excepted all tools
        if !args.only_editor {
            // check if no tools are present in config and tools / groups are provided
            if self.tools.is_none() && (args.tools.is_some() || !args.groups.is_some()) {
                return Err(anyhow!(
                    "Please provide tools in you configuration before passing tools or groups as arguments"
                ));
            }

            if !self.groups.as_ref().is_some_and(|groups| {
                iter_includes(
                    groups.keys().collect::<Vec<&String>>(),
                    args.groups.as_ref().unwrap_or(&Vec::new()),
                )
            }) {
                let groups = self.groups.as_ref().unwrap();
                if let Some(groups_args) = args.groups.as_ref() {
                    for group_key in groups_args {
                        if groups.get(group_key).is_none() {
                            let grp_str = if let Some(groups) = self.groups.as_ref() {
                                "\nHowever, the following groups are in your configuration : \n\t - "
                                .to_string()
                                + &groups
                                    .iter()
                                    .map(|g| g.0.to_owned())
                                    .collect::<Vec<String>>()
                                    .join("\n\t - ")
                            } else {
                                String::new()
                            };
                            return Err(anyhow!(
                            "The tool named '{}' does not exist in your configuration.{}\nSee ./{} list for more information",
                            group_key,
                            grp_str
                            , std::env::current_exe()?.file_name().map_or(Cow::Borrowed("editor"), |v| v.to_string_lossy())
                        ));
                        }
                    }
                }
            }

            // Check if dependencies are satisfied
            let dependencies = self.get_dependencies(args)?;
            dependencies.validate(self, args)?;

            for (_, tool) in dependencies.satisfied_tools {
                tool.validate()?;
            }
        }

        Ok(())
    }

    fn get_dependencies<'l>(&'l self, args: &'l Args) -> Result<Dependencies> {
        // Create dependencies
        let mut dependencies = Dependencies::default();
        if let Some(tools) = self.tools.as_ref() {
            let available_tool_keys: Vec<&String> = tools.keys().collect();
            let mut required_tools: Vec<(&String, &Tool)> = vec![];
            if let Some(args_tools) = args.tools.as_ref() {
                // For each dependencies check if it's available
                for tool_key in args_tools {
                    if let Some(tool) = tools.get(tool_key) {
                        required_tools.push((tool_key, tool));
                    } else {
                        return Err(anyhow!(
                            "A package from arguments is not present in configuration: '{tool_key}'"
                        ));
                    }
                }
            }
            // For each group check if it exists
            if let Some(groups) = self.groups.as_ref() {
                if let Some(groups_args) = args.groups.as_ref() {
                    for group_key in groups_args {
                        if let Some(group) = groups.get(group_key) {
                            // For each dependencies check if it's available
                            for tool_key in &group.dependencies {
                                match tools.get(tool_key) {
                                    Some(tool) => {
                                        required_tools.push((tool_key, tool));
                                    }
                                    None => (),
                                }
                            }
                        }
                    }
                }
            }
            // If nothing is required, install everything
            if required_tools.is_empty() {
                required_tools = tools.iter().collect();
            }
            for (tool_key, tool) in &required_tools {
                // check paths for each Tool
                let invalid_paths = tool.get_invalid_paths()?;
                if invalid_paths.len() >= 1 {
                    if dependencies.satisfied_tools.contains_key(*tool_key) {
                        dependencies.satisfied_tools.remove(*tool_key);
                    }
                    dependencies
                        .unsatisfied_tools
                        .entry(tool_key.to_string())
                        .or_insert(UnSatisfiedTool {
                            tool,
                            required: None,
                            paths: Some(HashMap::new()),
                        })
                        .paths
                        .get_or_insert(HashMap::new())
                        .extend(invalid_paths);
                }
            }
            // Create dependencies recursively
            for (tool_key, tool) in &required_tools {
                // call the recursivity for invalid dependencies
                self.get_dependencies_rec(&available_tool_keys, tool, tool_key, &mut dependencies);
            }

            if let Some(groups) = self.groups.as_ref() {
                if let Some(groups_args) = args.groups.as_ref() {
                    for group_key in groups_args {
                        match groups.get(group_key) {
                            Some(g) => {
                                for tool in &g.dependencies {
                                    // tool that group needs is not satisfied
                                    if dependencies.unsatisfied_tools.contains_key(tool)
                                        || !dependencies.satisfied_tools.contains_key(tool)
                                    {
                                        dependencies
                                            .unsatisfied_groups
                                            .entry(group_key.to_string())
                                            .or_insert(UnSatisfiedGroup {
                                                group: g,
                                                unsatisfied_tools: Vec::new(),
                                            })
                                            .unsatisfied_tools
                                            .push(tool);
                                    }
                                }
                                if !dependencies.unsatisfied_groups.contains_key(group_key) {
                                    dependencies
                                        .satisfied_groups
                                        .entry(group_key.to_string())
                                        .or_insert(g);
                                }
                            }
                            None => (),
                        }
                    }
                }
            }
        }

        Ok(dependencies)
    }

    fn get_dependencies_rec<'l>(
        &'l self,
        available_tool_keys: &[&String],
        current_tool: &'l Tool,
        current_tool_key: &'l String,
        dependencies: &mut Dependencies<'l>,
    ) {
        if available_tool_keys.contains(&current_tool_key)
            && !dependencies.satisfied_tools.contains_key(current_tool_key)
        {
            // avoid endless recursivity
            dependencies.checked_tool_keys.push(current_tool_key);
            let mut valid = true;
            if let Some(required) = current_tool.dependencies.as_ref() {
                for tool_key in required {
                    if let Some(required_tool_key) = self.tools.as_ref().unwrap().get(tool_key) {
                        if !dependencies.checked_tool_keys.contains(&tool_key) {
                            self.get_dependencies_rec(
                                available_tool_keys,
                                required_tool_key,
                                tool_key,
                                dependencies,
                            );
                        }
                        if dependencies.unsatisfied_tools.contains_key(tool_key) {
                            valid = false;
                            let required = (
                                Tool {
                                    name: tool_key.to_owned(),
                                    bin: PathBuf::new(),
                                    config: None,
                                    lib: None,
                                    dependencies: None,
                                },
                                DependencyErrorType::UnSatisfiedDepencies,
                            );
                            dependencies
                                .unsatisfied_tools
                                .entry(current_tool_key.to_string())
                                .or_insert(UnSatisfiedTool {
                                    tool: current_tool,
                                    required: Some(HashMap::new()),
                                    paths: None,
                                })
                                .required
                                .get_or_insert(HashMap::new())
                                .entry(tool_key)
                                .or_insert(required);
                        }
                    } else {
                        valid = false;
                        let required = (
                            Tool {
                                name: tool_key.to_owned(),
                                bin: PathBuf::new(),
                                config: None,
                                lib: None,
                                dependencies: None,
                            },
                            DependencyErrorType::NotFound,
                        );
                        dependencies
                            .unsatisfied_tools
                            .entry(current_tool_key.to_string())
                            .or_insert(UnSatisfiedTool {
                                tool: current_tool,
                                required: Some(HashMap::new()),
                                paths: None,
                            })
                            .required
                            .get_or_insert(HashMap::new())
                            .entry(tool_key)
                            .or_insert(required);
                    }
                }
            }
            if valid {
                dependencies
                    .satisfied_tools
                    .entry(current_tool_key.to_string())
                    .or_insert(current_tool);
            }
        }
    }

    pub fn list(&self, args: &Args) -> Result<()> {
        let args_cloned = args.clone_with_everything(self);
        let dependencies = self.get_dependencies(&args_cloned)?;
        let (found_config, found_bin) = (
            existence(&self.editor.config)?,
            existence(&self.editor.bin)?,
        );
        let error_editor = if !found_config.is_empty() || !found_bin.is_empty() {
            format!("({CYAN}ERROR{NC})")
        } else {
            String::new()
        };
        print!(
            "Editor: {GREEN}{}{NC} {}\n\tConfiguration : {} {}\n\tBinary : {} {}\n\n",
            self.editor.name,
            error_editor,
            self.editor.config.display(),
            found_config,
            self.editor.bin.display(),
            found_bin
        );

        if !args.verbose {
            print!("Tools : \n");
        }
        if let Some(tools) = self.tools.as_ref() {
            for (tool_key, tool) in tools {
                tool.list(&dependencies, tool_key, args)?;
            }
        }

        if !args.verbose {
            print!("\nGroups : \n");
        }
        if let Some(groups) = self.groups.as_ref() {
            for (group_key, group) in groups {
                group.list(&dependencies, group_key, self.tools.as_ref(), args)?;
            }
        }

        if let Some(d_groups) = self.default_groups.as_ref() {
            print!("\nDefault groups : ");
            for group in d_groups {
                print!(
                    "\n - {group} {}",
                    dependencies
                        .unsatisfied_groups
                        .get(group)
                        .map_or(String::new(), |_| format!("{YELLOW}ERROR{NC}, see above"))
                );
            }
        }
        if let Some(d_tools) = self.default_tools.as_ref() {
            print!("\nDefault tools : ");
            for tool in d_tools {
                print!(
                    "\n\t - {tool} {}",
                    dependencies
                        .unsatisfied_tools
                        .get(tool)
                        .map_or(String::new(), |_| format!("{YELLOW}ERROR{NC}, see above"))
                );
            }
        }
        print!("\n");
        if !args.verbose {
            print!("\nSee with -v (verbose mode) for more details\n");
        }
        std::io::stdout().flush()?;
        Ok(())
    }
}
