use clap::{Parser, ValueEnum};

use crate::config::CONFIG_FILENAME;

#[derive(ValueEnum, Clone)]
pub enum Action {
    Remove,
    Install,
    Update,
}

#[derive(Parser)]
pub struct EditorArg {
    /// the action choices: remove, install, update
    #[arg(value_enum)]
    pub action: Action,

    #[arg(short, long, default_value_t =String::from(CONFIG_FILENAME))]
    pub config: String,

    #[arg(short, long, default_value_t = false)]
    pub symbolic: bool,

    #[arg(short, long)]
    pub languages: Option<Vec<String>>,

    #[arg(short, long, default_value_t = false)]
    pub force: bool,

    /// verbose mode
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
