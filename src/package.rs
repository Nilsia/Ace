use crate::args::Args;
use crate::utils::{get_bin_path, get_config_path, install, install_unchecked, remove};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub trait Package {
    fn name(&self) -> &String;

    fn bin(&self) -> &PathBuf;

    fn config(&self) -> &PathBuf;

    fn lib(&self) -> &PathBuf;

    fn get_bin_name(&self) -> String {
        self.bin()
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(self.name())
            .to_string()
    }

    fn get_config_name(&self) -> String {
        self.config()
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(self.name())
            .to_string()
    }

    fn get_bin_path(&self) -> PathBuf {
        get_bin_path().join(self.get_bin_name())
    }

    fn get_config_path(&self) -> PathBuf {
        get_config_path().join(self.get_config_name())
    }

    fn install(&self, args: &Args) -> Result<()> {
        self.install_bin(args)?;
        self.install_config(args)
    }

    fn install_bin(&self, args: &Args) -> Result<()> {
        install(self.bin(), &self.get_bin_path(), args)
    }

    fn install_bin_unchecked(&self, args: &Args) -> Result<()> {
        install_unchecked(self.bin(), &self.get_bin_path(), args)
    }

    fn install_config(&self, args: &Args) -> Result<()> {
        install(self.config(), &self.get_config_path(), args)
    }

    fn install_config_unchecked(&self, args: &Args) -> Result<()> {
        install_unchecked(self.config(), &self.get_config_path(), args)
    }

    // Il faudra sans doute prendre la config en paramètre
    fn install_requirements(&self, args: &Args) -> Result<()>;

    fn remove(&self, args: &Args) -> Result<()> {
        self.remove_bin(args)?;
        self.remove_config(args)
    }

    fn remove_bin(&self, args: &Args) -> Result<()> {
        remove(self.get_bin_path())
    }

    fn remove_config(&self, args: &Args) -> Result<()> {
        remove(self.get_config_path())
    }
}
