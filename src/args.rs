use crate::config::DEFAULT_FILENAME;
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

    #[arg(short, long, default_value_t = String::from(DEFAULT_FILENAME))]
    pub config: String,

    /// Modify only those packages
    #[arg(short, long)]
    pub packages: Option<Vec<String>>,

    /// Temporary install with symbolic names
    #[arg(short, long, default_value_t = false)]
    pub symbolic: bool,

    /// Force action
    #[arg(short, long, default_value_t = false)]
    pub force: bool,

    /// Verbose mode
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
