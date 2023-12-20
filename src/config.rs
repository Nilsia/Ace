use crate::args::Args;
use crate::editor::Editor;
use crate::package::Package;
use crate::tools::Tools;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const DEFAULT_FILENAME: &str = "config.toml";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub editor: Editor,
    pub tools: Option<HashMap<String, Tools>>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(filename: P) -> Result<Config> {
        let config: Config = toml::from_str(&fs::read_to_string(filename)?)?;
        if let Err(e) = config.is_valid() {
            Err(e)
        } else {
            Ok(config)
        }
    }

    pub fn install(&self, args: &Args) -> Result<()> {
        self.editor.install(args)?;
        if let Some(packages) = &self.tools {
            for package in packages.values() {
                package.install(args)?;
            }
        }
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
