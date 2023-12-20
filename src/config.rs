use crate::args::Args;
use crate::editor::Editor;
use crate::package::Package;
use crate::tool::Tool;
use crate::utils::{create_dirs, export_bin_dir, GREEN, NC};
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const DEFAULT_FILENAME: &str = "config.toml";

#[derive(Debug)]
struct UnSatisfiedTools<'a> {
    required: Tool,
    parent_tool: &'a Tool,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub editor: Editor,
    pub tools: Option<HashMap<String, Tool>>,
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

        self.editor.install(args)?;
        let tools = self.get_satisfied_and_unsatisfied_tools(args)?.0;
        for tool in &tools {
            tool.install(args)?;
        }

        println!("{GREEN}SUCCESS{NC}: Refresh your terminal for the changes to take effect");
        Ok(())
    }

    pub fn remove(&self, args: &Args) -> Result<()> {
        self.editor.remove(args)?;

        let tools = self.get_satisfied_and_unsatisfied_tools(args)?.0;
        for tool in &tools {
            tool.remove(args)?;
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

        if self.tools.is_none() && args.packages.is_some() {
            return Err(anyhow!(
                "Please provide packages in you configuration before passing them as arguments"
            ));
        }

        let unsatisfied_tools = self.get_satisfied_and_unsatisfied_tools(args)?.1;
        if !unsatisfied_tools.is_empty() {
            return Err(anyhow!(self.print_unsatisfied_tools(&unsatisfied_tools)));
        }

        Ok(true)
    }

    fn get_satisfied_and_unsatisfied_tools<'a>(
        &'a self,
        args: &'a Args,
    ) -> Result<(Vec<&Tool>, Vec<UnSatisfiedTools>)> {
        let mut satisfied_tools: Vec<&Tool> = Vec::new();
        let mut unsatisfied_tools: Vec<UnSatisfiedTools> = Vec::new();
        if let Some(tools) = self.tools.as_ref() {
            let mut satisfied_keys: Vec<&String> = Vec::new();
            let mut unsatisfied_keys: Vec<&String> = Vec::new();
            let available_tool_keys: Vec<&String> = tools.keys().collect();
            let mut asked_tools: Vec<(&String, &Tool)> = vec![];
            if let Some(args_tools) = args.packages.as_ref() {
                for tool_key in args_tools {
                    if let Some(tool) = tools.get(tool_key) {
                        asked_tools.push((tool_key, tool));
                    } else {
                        return Err(anyhow!(
                            "A package from arguments is not present in configuration: '{tool_key}'"
                        ));
                    }
                }
            } else {
                asked_tools = tools.iter().collect();
            }

            for tool in asked_tools {
                self.get_satisfied_and_unsatisfied_tools_rec(
                    &available_tool_keys,
                    tool.1,
                    tool.0,
                    &mut satisfied_keys,
                    &mut satisfied_tools,
                    &mut unsatisfied_keys,
                    &mut unsatisfied_tools,
                )
            }
        }
        Ok((satisfied_tools, unsatisfied_tools))
    }

    fn get_satisfied_and_unsatisfied_tools_rec<'a>(
        &'a self,
        available_tool_keys: &[&String],
        current_tool: &'a Tool,
        current_tool_key: &'a String,
        satisfied_keys: &mut Vec<&'a String>,
        satisfied_tools: &mut Vec<&'a Tool>,
        unsatisfied_keys: &mut Vec<&'a String>,
        unsatisfied_tools: &mut Vec<UnSatisfiedTools<'a>>,
    ) {
        if available_tool_keys.contains(&current_tool_key) {
            if !satisfied_keys.contains(&current_tool_key) {
                satisfied_keys.push(current_tool_key);
                satisfied_tools.push(current_tool);
                if let Some(required) = current_tool.requires.as_ref() {
                    for tool_key in required {
                        if let Some(required_tool_key) =
                            self.tools.as_ref().unwrap().get(tool_key).as_ref()
                        {
                            self.get_satisfied_and_unsatisfied_tools_rec(
                                available_tool_keys,
                                &required_tool_key,
                                tool_key,
                                satisfied_keys,
                                satisfied_tools,
                                unsatisfied_keys,
                                unsatisfied_tools,
                            );
                        } else {
                            unsatisfied_keys.push(tool_key);
                            unsatisfied_tools.push(UnSatisfiedTools {
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
    }

    fn print_unsatisfied_tools(&self, unsatisfied_tools: &[UnSatisfiedTools]) -> String {
        let mut res = String::from("The following dependencies are not satisfied");
        for tool in unsatisfied_tools {
            res.push_str(&format!(
                "\n- '{}' depends on '{}'",
                tool.parent_tool.name, tool.required.name
            ));
        }
        res
    }
}
