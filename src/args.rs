use crate::config::DEFAULT_FILENAME;
use anyhow::{anyhow, Result};
use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
pub enum Action {
    Install,
    Remove,
    Update,
}

#[derive(Parser, Clone, Debug)]
pub struct Args {
    #[arg(value_enum)]
    pub action: Action,

    /// Provide config toml file configuration
    #[arg(short, long, default_value_t = String::from(DEFAULT_FILENAME))]
    pub config: String,

    /// Modify only those tools
    #[arg(short, long)]
    pub tools: Option<Vec<String>>,

    /// Temporary install with symbolic names
    #[arg(short, long, default_value_t = false)]
    pub symbolic: bool,

    /// Force action
    #[arg(short, long, default_value_t = false)]
    pub force: bool,

    /// Verbose mode - not used at all
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Only make modifications on the editor works on remove, install an update
    #[arg(long, default_value_t = false)]
    pub only_editor: bool,

    /// except the editor configuration works on remove, install and update
    #[arg(long, default_value_t = false)]
    pub except_editor: bool,

    /// Specify the groups you want to install
    #[arg(long, short)]
    pub groups: Vec<String>,
}

impl Args {
    pub fn is_valid(&self) -> Result<bool> {
        if self.only_editor && self.except_editor {
            return Err(anyhow!("You cannot provide only and except editor at once"));
        }
        Ok(true)
    }
}
