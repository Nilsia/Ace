use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

use crate::args::EditorArg;

#[derive(Deserialize, Debug)]
pub struct Editor {
    pub name: String,
    pub config_path: PathBuf,
    pub binary_path: PathBuf,
    pub files_path: PathBuf,
}
impl Editor {
    pub(crate) fn remove(&self, args: &EditorArg) -> Result<()> {
        Ok(())
    }
}
