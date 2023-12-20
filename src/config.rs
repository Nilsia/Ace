use crate::args::Args;
use crate::editor::Editor;
use crate::package::Package;
use crate::tools::Tools;
use crate::utils::{create_dirs, export_bin_dir, get_shell_config_path};
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

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
        println!("Refresh your terminal for the changes to take effect.");
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
        Ok(true)
    }
}
