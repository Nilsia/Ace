use crate::{args::Args, package::Package};
use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Tools {
    pub name: String,
    pub bin: PathBuf,
    pub config: Option<PathBuf>,
    pub lib: Option<PathBuf>,
    pub requires: Option<Vec<String>>,
}

impl Package for Tools {
    fn name(&self) -> &String {
        &self.name
    }

    fn bin(&self) -> &PathBuf {
        &self.bin
    }

    fn config(&self) -> Option<&PathBuf> {
        self.config.as_ref()
    }

    fn lib(&self) -> Option<&PathBuf> {
        self.lib.as_ref()
    }

    fn install_requirements(&self, args: &Args) -> Result<()> {
        todo!()
    }
}
