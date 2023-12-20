use crate::args::Args;
use crate::package::Package;
use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Editor {
    pub name: String,
    pub config: PathBuf,
    pub bin: PathBuf,
    pub lib: Option<PathBuf>,
}

impl Package for Editor {
    fn name(&self) -> &String {
        &self.name
    }

    fn bin(&self) -> &PathBuf {
        &self.bin
    }

    fn config(&self) -> Option<&PathBuf> {
        Some(&self.config)
    }

    fn lib(&self) -> Option<&PathBuf> {
        self.lib.as_ref()
    }

    fn install_requirements(&self, args: &Args) -> Result<()> {
        todo!()
    }
}
