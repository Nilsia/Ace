use anyhow::Result;
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

use crate::{args::EditorArg, editor::Editor, language::Language};
pub const CONFIG_FILENAME: &str = "config.toml";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub editor: Editor,
    // pub languages: Vec<String>,
    pub language_config: HashMap<String, Language>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(filename: P) -> Result<Config> {
        Ok(toml::from_str(&fs::read_to_string(filename)?)?)
    }
    pub fn remove(&mut self, args: &EditorArg) -> Result<()> {
        self.editor.remove(args)?;
        for language in self.language_config.values_mut() {
            language.remove(args)?;
        }
        Ok(())
    }
}
