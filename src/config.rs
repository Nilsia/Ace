use crate::args::Args;
use crate::editor::Editor;
use crate::group::Group;
use crate::package::Package;
use crate::tool::Tool;
use crate::utils::{create_dirs, export_bin_dir, GREEN, NC};
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const DEFAULT_FILENAME: &str = "config.toml";

#[derive(Debug)]
struct UnSatisfiedTools<'l> {
    required: Tool,
    parent_tool: &'l Tool,
}

#[derive(Debug)]
struct Dependencies<'l> {
    satisfied_keys: Vec<&'l String>,
    satisfied_tools: Vec<&'l Tool>,
    unsatisfied_keys: Vec<&'l String>,
    unsatisfied_tools: Vec<UnSatisfiedTools<'l>>,
}

impl<'l> Dependencies<'l> {
    pub fn new() -> Self {
        Dependencies {
            satisfied_keys: Vec::new(),
            satisfied_tools: Vec::new(),
            unsatisfied_keys: Vec::new(),
            unsatisfied_tools: Vec::new(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub editor: Editor,
    pub tools: Option<HashMap<String, Tool>>,
    pub groups: Option<HashMap<String, Group>>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(filename: P, args: &Args) -> Result<Config> {
        if filename.as_ref().exists() {
            let config: Config = toml::from_str(&std::fs::read_to_string(filename)?)?;
            match config.is_valid(args) {
                Err(e) => Err(e),
                Ok(_) => Ok(config),
            }
        } else {
            Err(anyhow!("No config file"))
        }
    }

    pub fn install(&self, args: &Args) -> Result<()> {
        create_dirs()?;
        export_bin_dir()?;

        if !args.except_editor {
            self.editor.install(args)?;
        }
        if !args.only_editor {
            let dependencies = self.get_dependencies(args)?;
            for tool in &dependencies.satisfied_tools {
                tool.install(args)?;
            }
        }

        println!("{GREEN}SUCCESS{NC}: Refresh your terminal for the changes to take effect");
        Ok(())
    }

    pub fn remove(&self, args: &Args) -> Result<()> {
        if !args.except_editor {
            self.editor.remove(args)?;
        }
        if !args.only_editor {
            let dependencies = self.get_dependencies(args)?;
            for tool in &dependencies.satisfied_tools {
                tool.remove(args)?;
            }
        }
        println!("{GREEN}SUCCESS{NC}");
        Ok(())
    }

    fn is_valid(&self, args: &Args) -> Result<bool> {
        self.editor.is_valid()?;
        if let Some(tools) = self.tools.as_ref() {
            for tool in tools.values() {
                tool.is_valid()?;
            }
        }

        if self.tools.is_none() && args.tools.is_some() {
            return Err(anyhow!(
                "Please provide packages in you configuration before passing them as arguments"
            ));
        }

        let dependencies = self.get_dependencies(args)?;
        if !dependencies.unsatisfied_tools.is_empty() {
            return Err(anyhow!(Self::print_unsatisfied_dependencies(&dependencies)));
        }

        Ok(true)
    }

    fn get_dependencies<'l>(&'l self, args: &'l Args) -> Result<Dependencies> {
        let mut dependencies = Dependencies::new();
        if let Some(tools) = self.tools.as_ref() {
            let available_tool_keys: Vec<&String> = tools.keys().collect();
            let mut required_tools: Vec<(&String, &Tool)> = vec![];
            if let Some(args_tools) = args.tools.as_ref() {
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
            // there are groups in config file
            else if let Some(groups) = self.groups.as_ref() {
                for group_key in &args.groups {
                    // given group in arguments exists in config
                    if let Some(group) = groups.get(group_key) {
                        // iterate through tool to check if they exist
                        for tool_key in &group.dependencies {
                            if let Some(tool) = tools.get(tool_key) {
                                required_tools.push((tool_key, tool));
                            }
                        }
                    }
                }
            }
            if required_tools.is_empty() {
                required_tools = tools.iter().collect();
            }

            for tool in required_tools {
                self.get_dependencies_rec(&available_tool_keys, tool.1, tool.0, &mut dependencies);
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
            && !dependencies.satisfied_keys.contains(&current_tool_key)
        {
            dependencies.satisfied_keys.push(current_tool_key);
            dependencies.satisfied_tools.push(current_tool);
            if let Some(required) = current_tool.requires.as_ref() {
                for tool_key in required {
                    if let Some(required_tool_key) = self.tools.as_ref().unwrap().get(tool_key) {
                        self.get_dependencies_rec(
                            available_tool_keys,
                            required_tool_key,
                            tool_key,
                            dependencies,
                        );
                    } else {
                        dependencies.unsatisfied_keys.push(tool_key);
                        dependencies.unsatisfied_tools.push(UnSatisfiedTools {
                            required: Tool {
                                name: tool_key.to_owned(),
                                bin: PathBuf::new(),
                                config: None,
                                lib: None,
                                requires: None,
                            },
                            parent_tool: current_tool,
                        });
                    }
                }
            }
        }
    }

    fn print_unsatisfied_dependencies(dependencies: &Dependencies) -> String {
        let mut res = String::from("The following dependencies are not satisfied");
        for tool in &dependencies.unsatisfied_tools {
            res.push_str(&format!(
                "\n- '{}' depends on '{}'",
                tool.parent_tool.name, tool.required.name
            ));
        }
        res
    }
}
