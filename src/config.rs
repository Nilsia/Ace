use crate::args::Args;
use crate::editor::Editor;
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
    pub tools: HashMap<String, Tools>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(filename: P) -> Result<Config> {
        Ok(toml::from_str(&fs::read_to_string(filename)?)?)
    }

    pub fn install(&mut self, args: &Args) -> Result<()> {
        self.editor.try_install(args)?;
        for package in self.tools.values_mut() {
            package.try_install(args)?;
        }
        Ok(())
    }

    pub fn remove(&mut self, args: &Args) -> Result<()> {
        self.editor.remove(args)?;
        for package in self.tools.values_mut() {
            package.remove(args)?;
        }
        Ok(())
    }
}
