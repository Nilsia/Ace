use crate::package::Package;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Tool {
    pub name: String,
    pub bin: PathBuf,
    pub config: Option<PathBuf>,
    pub lib: Option<PathBuf>,
    pub dependencies: Option<Vec<String>>,
}

impl Package for Tool {
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
}
