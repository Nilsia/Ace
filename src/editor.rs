use crate::args::Args;
use crate::utils::{get_bin_path, get_config_path};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Editor {
    pub name: String,
    pub config: PathBuf,
    pub binary: PathBuf,
    pub lib: Option<PathBuf>,
}

// Editor ressemble beaucoup à `Package` au final
impl Editor {
    pub fn try_install(&self, args: &Args) -> Result<()> {
        self.install_bin(args)?;
        self.install_config(args)
    }

    pub fn install_bin_unchecked(&self, args: &Args) -> Result<()> {
        if args.symbolic {
            std::os::unix::fs::symlink(&self.binary, self.get_bin_path())?;
        } else {
            fs::copy(&self.binary, self.get_bin_path())?;
        }
        Ok(())
    }

    pub fn install_config_unchecked(&self, args: &Args) -> Result<()> {
        if args.symbolic {
            std::os::unix::fs::symlink(&self.config, self.get_config_path())?;
        } else {
            fs::copy(&self.config, self.get_config_path())?;
        }
        Ok(())
    }

    // TODO: Les deux méthodes se ressemblent beaucoup, il faudrait généraliser
    pub fn install_bin(&self, args: &Args) -> Result<()> {
        if args.force {
            self.remove_bin(args)?;
            self.install_bin_unchecked(args)
        } else {
            let binary = self.get_config_path();
            let mut answer = String::new();

            if binary.exists() {
                println!(
                    "{} already exists, do you want to overwrite it (y/N) ? ",
                    &self.name
                );
                std::io::stdin().read_line(&mut answer)?;
            }

            match answer.trim().to_lowercase().as_str() {
                "y" => {
                    self.remove_bin(args)?;
                    self.install_bin_unchecked(args)
                }
                _ => Ok(()),
            }
        }
    }

    pub fn install_config(&self, args: &Args) -> Result<()> {
        if args.force {
            self.remove_config(args)?;
            self.install_config_unchecked(args)
        } else {
            let config = self.get_config_path();
            let mut answer = String::new();

            if config.exists() {
                println!(
                    "{} already exists, do you want to overwrite it (y/N) ? ",
                    &self.name
                );
                std::io::stdin().read_line(&mut answer)?;
            }

            match answer.trim().to_lowercase().as_str() {
                "y" => {
                    self.remove_config(args)?;
                    self.install_config_unchecked(args)
                }
                _ => Ok(()),
            }
        }
    }

    pub fn remove(&self, args: &Args) -> Result<()> {
        self.remove_bin(args)?;
        self.remove_config(args)
    }

    pub fn remove_bin(&self, args: &Args) -> Result<()> {
        Ok(fs::remove_file(self.get_bin_path())
            .context(format!("Could not remove {}", self.name))?)
    }

    pub fn remove_config(&self, args: &Args) -> Result<()> {
        Ok(fs::remove_dir_all(self.get_config_path())
            .context(format!("Could not remove {} configuration", self.name))?)
    }

    fn get_bin_name(&self) -> String {
        self.binary
            .to_path_buf()
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&self.name)
            .to_string()
    }

    fn get_bin_path(&self) -> PathBuf {
        get_bin_path().join(self.get_bin_name())
    }

    fn get_config_name(&self) -> String {
        self.config
            .to_path_buf()
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&self.name)
            .to_string()
    }

    fn get_config_path(&self) -> PathBuf {
        get_config_path().join(self.get_config_name())
    }
}
