use crate::args::Args;
use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub binary: PathBuf,
    pub config: Option<PathBuf>,
    pub lib: Option<PathBuf>,
    pub requires: Option<Vec<String>>,
}

impl Package {
    pub fn try_install(&self, args: &Args) -> Result<()> {
        todo!()
    }

    pub fn install_config(&self, args: &Args) -> Result<()> {
        todo!()
    }

    pub fn install_requirements(&self, args: &Args) -> Result<()> {
        // Il faudra sans doute prendre la config en paramètre
        todo!()
    }

    pub fn remove(&self, args: &Args) -> Result<()> {
        todo!()
    }

    pub fn remove_config(&self, args: &Args) -> Result<()> {
        todo!()
    }
}
