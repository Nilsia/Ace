use crate::args::Args;
use crate::editor::Editor;
use crate::package::Package;
use crate::tools::Tools;
use crate::utils::{create_dirs, export_bin_dir, vec_includes};
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

pub const DEFAULT_FILENAME: &str = "config.toml";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub editor: Editor,
    pub tools: Option<HashMap<String, Tools>>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(filename: P) -> Result<Config> {
        let config: Config = toml::from_str(&std::fs::read_to_string(filename)?)?;
        if let Err(e) = config.is_valid() {
            Err(e)
        } else {
            Ok(config)
        }
    }

    pub fn install(&self, args: &Args) -> Result<()> {
        create_dirs()?;
        export_bin_dir()?;

        self.editor.install(args)?;
        if let Some(packages) = &self.tools {
            for package in packages.values() {
                package.install(args)?;
            }
        }

        println!("SUCCESS: Refresh your terminal for the changes to take effect");
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

    fn is_valid(&self) -> Result<bool> {
        self.editor.is_valid()?;
        if let Some(tools) = self.tools.as_ref() {
            for tool in tools.values() {
                tool.is_valid()?;
            }
        }

        //check that all required tools are in the configuration
        if self
            .tools
            .as_ref()
            .is_some_and(|v| !vec_includes(v.keys(), self.get_all_required_package()))
        {
            return Err(anyhow!("All required tools are not present"));
        }
        Ok(true)
    }

    fn get_all_required_package(&self) -> Vec<&String> {
        let mut requires: Vec<&String> = Vec::new();
        if self.tools.is_some() {
            for tool in self.tools.as_ref().unwrap().values() {
                if let Some(required) = tool.requires.as_ref() {
                    for tool_req in required {
                        if !requires.contains(&tool_req) {
                            requires.push(tool_req);
                        }
                    }
                }
            }
        }
        requires
    }
}
