use crate::args::Args;
use crate::utils::{get_bin_dir, get_config_dir};
use crate::utils::{BLUE, GREEN, NC, RED, RESTORE, SAVE, YELLOW};
use anyhow::{anyhow, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

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
        self.install_files(self.bin(), &self.get_bin_path(), args)
    }

    fn install_config(&self, args: &Args) -> Result<()> {
        if let (Some(config), Some(path)) = (self.config(), self.get_config_path()) {
            self.install_files(config, &path, args)
        } else {
            Ok(())
        }
    }

    // TODO: We will probably need the config as parameter
    fn install_requirements(&self, args: &Args) -> Result<()>;

    fn remove(&self, args: &Args) -> Result<()> {
        self.remove_bin(args)?;
        self.remove_config(args)
    }

    fn remove_bin(&self, args: &Args) -> Result<()> {
        let path = self.get_bin_path();
        if args.force {
            self.remove_files_unchecked(&path)
        } else {
            self.remove_files(&path)
        }
    }

    fn remove_config(&self, args: &Args) -> Result<()> {
        if let Some(config) = self.get_config_path() {
            if args.force {
                self.remove_files_unchecked(config)
            } else {
                self.remove_files(config)
            }
        } else {
            Ok(())
        }
    }

    fn install_files_unchecked<P: AsRef<Path>>(&self, from: P, to: P, args: &Args) -> Result<()> {
        if args.symbolic {
            std::os::unix::fs::symlink(env::current_dir()?.join(&from), &to)?;
            println!(
                "{BLUE}INSTALLED{NC}: {} {BLUE}->{NC} {}",
                from.as_ref().display(),
                to.as_ref().display()
            );
        } else {
            print!(
                "{SAVE}INSTALLING: {} -> {}",
                from.as_ref().display(),
                to.as_ref().display()
            );
            io::stdout().flush()?;

            if from.as_ref().is_dir() {
                copy_dir::copy_dir(&from, &to)?;
            } else {
                fs::copy(&from, &to)?;
            }

            println!(
                "{RESTORE}{GREEN}INSTALLED{NC}: {} -> {}",
                from.as_ref().display(),
                to.as_ref().display()
            );
        }
        Ok(())
    }

    fn install_files<P: AsRef<Path>>(&self, from: P, to: P, args: &Args) -> Result<()> {
        if args.force {
            self.remove_files_unchecked(&to)?;
            self.install_files_unchecked(&from, &to, args)
        } else {
            let path = to.as_ref();
            if path.exists() || path.is_symlink() {
                let mut choice = String::new();

                print!(
                    "{SAVE}{YELLOW}WARNING{NC}: Do you want to overwrite '{}' (y/N): ",
                    path.display()
                );
                io::stdout().flush()?;
                io::stdin().read_line(&mut choice)?;

                match choice.trim().to_lowercase().as_str() {
                    "y" => {
                        self.remove_files_unchecked(&to)?;
                        self.install_files_unchecked(from, to, args)
                    }
                    _ => Ok(()),
                }
            } else {
                self.install_files_unchecked(from, to, args)
            }
        }
    }

    fn remove_files<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        if path.exists() || path.is_symlink() {
            let mut choice = String::new();

            let display = path.display();
            print!(
                "{SAVE}{YELLOW}WARNING{NC}: Do you want to remove '{}' (Y/n): ",
                display
            );
            io::stdout().flush()?;
            io::stdin().read_line(&mut choice)?;

            match choice.trim().to_lowercase().as_str() {
                "n" => {
                    println!("{YELLOW}WARNING{NC}: Canceled '{}' deletion", display);
                    Ok(())
                }
                _ => self.remove_files_unchecked(path),
            }
        } else {
            Ok(())
        }
    }

    fn remove_files_unchecked<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        if path.as_ref().exists() || path.as_ref().is_symlink() {
            let display = path.as_ref().display();
            print!("{SAVE}{RED}DELETING{NC}: {}", display);
            io::stdout().flush()?;

            if path.as_ref().is_dir() {
                std::fs::remove_dir_all(&path)?;
            } else {
                std::fs::remove_file(&path)?;
            }

            println!("{RESTORE}{RED}DELETED{NC}: {}", display);
        }
        Ok(())
    }

    fn is_valid(&self) -> Result<bool> {
        // TODO handle lib and requires
        if !self.bin().exists() {
            Err(anyhow!(
                "{RED}ERROR{NC}: '{}' is not present",
                self.bin().display()
            ))
        } else if self.config().is_some_and(|v| !v.exists()) {
            Err(anyhow!(
                "{RED}ERROR{NC}: '{}' is not present",
                self.config().unwrap().display()
            ))
        } else {
            Ok(true)
        }
    }
}
