use crate::args::Args;
use crate::utils::{clear_line, get_bin_dir, get_config_dir};
use anyhow::{anyhow, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

const RED: &str = "\x1b[0;31m";
const YELLOW: &str = "\x1b[0;33m";
const GREEN: &str = "\x1b[0;32m";
const NC: &str = "\x1b[0m";

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
        self.general_install(self.bin(), &self.get_bin_path(), args)
    }

    fn install_bin_unchecked(&self, args: &Args) -> Result<()> {
        self.general_install_unchecked(self.bin(), &self.get_bin_path(), args)
    }

    fn install_config(&self, args: &Args) -> Result<()> {
        if let (Some(config), Some(path)) = (self.config(), self.get_config_path()) {
            self.general_install(config, &path, args)
        } else {
            Ok(())
        }
    }

    fn install_config_unchecked(&self, args: &Args) -> Result<()> {
        if let (Some(config), Some(path)) = (self.config(), self.get_config_path()) {
            self.general_install_unchecked(config, &path, args)
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
        let path = self.get_bin_path();
        if args.force {
            self.general_remove_unchecked(&path)
        } else {
            self.general_remove(&path)
        }
    }

    fn remove_config(&self, args: &Args) -> Result<()> {
        if let Some(config) = self.get_config_path() {
            if args.force {
                self.general_remove_unchecked(config)
            } else {
                self.general_remove(config)
            }
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
    fn general_install_unchecked<P: AsRef<Path>>(&self, from: P, to: P, args: &Args) -> Result<()> {
        if args.symbolic {
            std::os::unix::fs::symlink(env::current_dir()?.join(&from), &to)?;
            print!(
                "{} is now a {GREEN}link{NC} to {}\n",
                from.as_ref().display(),
                to.as_ref().display()
            );
            std::io::stdout().flush()?;
        } else {
            print!(
                "Installing {} -> {}\n",
                from.as_ref().display(),
                to.as_ref().display()
            );
            io::stdout().flush()?;
            if from.as_ref().is_dir() {
                copy_dir::copy_dir(&from, &to)?;
            } else {
                fs::copy(&from, &to)?;
            }
            clear_line()?;
            print!(
                "{GREEN}Installed{NC} {} -> {}\n",
                from.as_ref().display(),
                to.as_ref().display()
            );
        }
        Ok(())
    }

    fn general_install<P: AsRef<Path>>(&self, from: P, to: P, args: &Args) -> Result<()> {
        if args.force {
            self.general_remove_unchecked(&to)?;
            self.general_install_unchecked(&from, &to, args)?;

            Ok(())
        } else {
            let path = to.as_ref();
            if path.exists() || path.is_symlink() {
                let mut choice = String::new();

                print!(
                    "{YELLOW}Warning{NC}: Do you want to replace '{}' (y/N): ",
                    path.display()
                );
                io::stdout().flush()?;
                io::stdin().read_line(&mut choice)?;

                match choice.trim().to_lowercase().as_str() {
                    "y" => {
                        self.general_remove_unchecked(&to)?;
                        self.general_install_unchecked(from, to, args)
                    }
                    _ => Ok(()),
                }
            } else {
                self.general_install_unchecked(from, to, args)
            }
        }
    }

    fn general_remove<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        if path.as_ref().exists() || path.as_ref().is_symlink() {
            let mut answer = "".to_string();
            let config_str = path.as_ref().display();
            print!("Do you want to remove {} : (Y/n)", config_str);
            io::stdout().flush()?;
            io::stdin().read_line(&mut answer)?;
            match answer.trim().to_lowercase().as_str() {
                "n" => {
                    println!("Deletion of {} canceled", config_str);
                    Ok(())
                }
                _ => self.general_remove_unchecked(path),
            }
        } else {
            Ok(())
        }
    }
    fn general_remove_unchecked<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        if path.as_ref().exists() || path.as_ref().is_symlink() {
            let config_str = path.as_ref().display();
            print!("{RED}Deleting{NC} {}", config_str);
            io::stdout().flush()?;

            if path.as_ref().is_dir() {
                std::fs::remove_dir_all(&path)?;
            } else {
                std::fs::remove_file(&path)?;
            }
            clear_line()?;
            print!("{RED}Deleted{NC} {}\n", config_str);
            io::stdout().flush()?;
        }
        Ok(())
    }
}
