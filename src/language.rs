use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

use crate::args::EditorArg;

#[derive(Deserialize, Debug)]
pub struct Language {
    pub name: String,
    pub path: PathBuf,
    pub require: Option<Vec<String>>,
}
impl Language {
    pub(crate) fn remove(&self, args: &EditorArg) -> Result<()> {
        todo!()
    }
}
