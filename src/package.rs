use crate::args::Args;
use crate::utils::{get_bin_dir, get_config_dir, install, install_unchecked, remove};
use anyhow::{anyhow, Result};
use std::path::PathBuf;

pub trait Package {
    fn name(&self) -> &String;

    fn bin(&self) -> &PathBuf;

    fn config(&self) -> Option<&PathBuf>;

    fn lib(&self) -> Option<&PathBuf>;

    fn get_bin_name(&self) -> String {
        self.bin()
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(self.name())
            .to_string()
    }

    fn get_config_name(&self) -> Option<String> {
        self.config().map(|v| {
            v.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(self.name())
                .to_string()
        })
    }

    fn get_bin_path(&self) -> PathBuf {
        get_bin_dir().join(self.get_bin_name())
    }

    fn get_config_path(&self) -> Option<PathBuf> {
        if let Some(name) = self.get_config_name() {
            Some(get_config_dir().join(name))
        } else {
            None
        }
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
        if let (Some(config), Some(path)) = (self.config(), self.get_config_path()) {
            install(config, &path, args)
        } else {
            Ok(())
        }
    }

    fn install_config_unchecked(&self, args: &Args) -> Result<()> {
        if let (Some(config), Some(path)) = (self.config(), self.get_config_path()) {
            install_unchecked(config, &path, args)
        } else {
            Ok(())
        }
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
        if let Some(config) = self.get_config_path() {
            remove(config)
        } else {
            Ok(())
        }
    }

    fn is_valid(&self) -> Result<bool> {
        // TODO handle lib and requires
        if !self.bin().exists() {
            Err(anyhow!("Binary is not present"))
        } else if self.config().is_some_and(|v| !v.exists()) {
            Err(anyhow!("Configuration is not present"))
        } else {
            Ok(true)
        }
    }
}
