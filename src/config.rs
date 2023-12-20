use crate::args::Args;
use crate::editor::Editor;
use crate::package::Package;
use crate::tools::Tools;
use crate::utils::{create_dirs, export_bin_dir, GREEN, NC};
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const DEFAULT_FILENAME: &str = "config.toml";

#[derive(Debug)]
struct UnSatisfiedTools<'a> {
    required_tool: Tools,
    parent_tool: &'a Tools,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub editor: Editor,
    pub tools: Option<HashMap<String, Tools>>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(filename: P, args: &Args) -> Result<Config> {
        if filename.as_ref().exists() {
            let config: Config = toml::from_str(&std::fs::read_to_string(filename)?)?;
            match config.is_valid(args) {
                Err(e) => Err(e),
                Ok(v) => {
                    if v {
                        Ok(config)
                    } else {
                        Err(anyhow!(""))
                    }
                }
            }
        } else {
            Err(anyhow!("No config file"))
        }
    }

    pub fn install(&self, args: &Args) -> Result<()> {
        create_dirs()?;
        export_bin_dir()?;

        self.editor.install(args)?;
        if let Some(tools) = &self.tools {
            for tool in tools.values() {
                tool.install(args)?;
            }
        }

        println!("{GREEN}SUCCESS{NC}: Refresh your terminal for the changes to take effect");
        Ok(())
    }

    pub fn remove(&self, args: &Args) -> Result<()> {
        self.editor.remove(args)?;
        if let Some(packages) = &self.tools {
            for package in packages.values() {
                package.remove(args)?;
            }
        }
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
                "Please provide packages in you config file before giving packages as arguments"
            ));
        }

        let (_satis_tools, unsatis_tools) = self.get_unsatis_and_satis_tools(args)?;
        println!("satis_tools = {:#?}", _satis_tools);
        println!("unsatis_tools = {:#?}", unsatis_tools);
        if !unsatis_tools.is_empty() {
            self.print_unsatisfied_tools(&unsatis_tools);
            return Ok(false);
        }

        Ok(true)
    }

    fn get_unsatis_and_satis_tools<'a>(
        &'a self,
        args: &'a Args,
    ) -> Result<(Vec<&Tools>, Vec<UnSatisfiedTools>)> {
        let mut satisfied_tools: Vec<&Tools> = Vec::new();
        let mut unsatisfied_tools: Vec<UnSatisfiedTools> = Vec::new();
        if let Some(tools) = self.tools.as_ref() {
            let mut satisfied_keys: Vec<&String> = Vec::new();
            let mut unsatisfied_keys: Vec<&String> = Vec::new();
            let available_tool_keys: Vec<&String> = tools.keys().collect();
            let mut asked_tools: Vec<(&String, &Tools)> = vec![];
            if let Some(args_tools) = args.packages.as_ref() {
                for tool_key in args_tools {
                    if let Some(tool) = tools.get(tool_key) {
                        asked_tools.push((tool_key, tool));
                    } else {
                        return Err(anyhow!("A provided package from arguments is not present in file configuration : {}", tool_key));
                    }
                }
            } else {
                asked_tools = tools.iter().collect();
            }
            for tool in asked_tools {
                self.get_unsatis_and_satis_tools_recur(
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

    fn get_unsatis_and_satis_tools_recur<'a>(
        &'a self,
        available_tool_keys: &[&String],
        current_tool: &'a Tools,
        current_tool_key: &'a String,
        satisfied_keys: &mut Vec<&'a String>,
        satisfied_tools: &mut Vec<&'a Tools>,
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
                            self.get_unsatis_and_satis_tools_recur(
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
                                required_tool: Tools {
                                    name: tool_key.to_owned(),
                                    bin: PathBuf::from(""),
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

    fn print_unsatisfied_tools(&self, unsatisfied_tools: &[UnSatisfiedTools]) {
        println!("The following dependencies are not satisfied : ");
        for tool in unsatisfied_tools {
            println!(
                "{} depends on {}",
                tool.parent_tool.name, tool.required_tool.name
            )
        }
    }
}
