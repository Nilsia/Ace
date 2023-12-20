use crate::args::Args;
use crate::utils::{
    find_common_ancestor, find_relative_path, get_bin_dir, get_config_dir, get_data_dir,
    make_absolute,
};
use crate::utils::{BLUE, GREEN, NC, RED, RESTORE, SAVE, YELLOW};
use anyhow::{anyhow, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub trait Package {
    fn name(&self) -> &String;

    fn bin(&self) -> &PathBuf;

    fn config(&self) -> Option<&PathBuf>;

    fn lib(&self) -> Option<&PathBuf>;

    fn parent(&self) -> Option<PathBuf> {
        if let Some(lib) = self.lib() {
            find_common_ancestor(self.bin(), lib).ok()
        } else {
            Some(self.bin().clone())
        }
    }

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

    fn get_lib_name(&self) -> Option<String> {
        self.lib().map(|v| {
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
        self.get_config_name()
            .map(|name| get_config_dir().join(name))
    }

    fn install(&self, args: &Args) -> Result<()> {
        self.install_bin(args)?;
        self.install_config(args)
    }

    fn install_bin(&self, args: &Args) -> Result<()> {
        if args.symbolic || self.lib().is_none() {
            self.install_files(self.bin(), &self.get_bin_path(), args)
        } else {
            let lib = self.lib().unwrap();
            let ancestor = find_common_ancestor(self.bin(), lib)?;
            let path = get_data_dir().join(self.name());
            self.install_files(&ancestor, &path, args)?;
            // Activate symbolic in args to link bin to lib
            let mut sym_args = args.clone();
            sym_args.symbolic = true;
            let relative = find_relative_path(self.bin(), &ancestor)?;
            let bin = path.join(relative);
            self.install_files(&bin, &self.get_bin_path(), &sym_args)
        }
    }

    fn install_config(&self, args: &Args) -> Result<()> {
        if let (Some(config), Some(path)) = (self.config(), &self.get_config_path()) {
            self.install_files(config, path, args)
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
            if self.lib().is_some() {
                self.remove_files_unchecked(get_data_dir().join(self.name()))?;
            }
            self.remove_files_unchecked(&path)
        } else {
            if self.lib().is_some() {
                self.remove_files(get_data_dir().join(self.name()))?;
            }
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
        let f_display = from.as_ref().display();
        let t_display = to.as_ref().display();
        if args.symbolic {
            std::os::unix::fs::symlink(make_absolute(&from)?, &to)?;
            println!("{BLUE}LINKED{NC}: {t_display} {BLUE}->{NC} {f_display}");
        } else {
            print!("{SAVE}INSTALLING: {f_display} -> {t_display}");
            io::stdout().flush()?;

            if from.as_ref().is_dir() {
                copy_dir::copy_dir(&from, &to)?;
            } else {
                fs::copy(&from, &to)?;
            }

            println!("{RESTORE}{GREEN}INSTALLED{NC}: {f_display} -> {t_display}");
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

                let display = path.display();
                print!("{SAVE}{YELLOW}WARNING{NC}: Do you want to overwrite '{display}' (y/N): ");
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
            print!("{SAVE}{YELLOW}WARNING{NC}: Do you want to remove '{display}' (Y/n): ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut choice)?;

            match choice.trim().to_lowercase().as_str() {
                "n" => {
                    println!("{YELLOW}WARNING{NC}: Canceled '{display}' deletion");
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
            print!("{SAVE}{RED}DELETING{NC}: {display}");
            io::stdout().flush()?;

            if path.as_ref().is_dir() {
                std::fs::remove_dir_all(&path)?;
            } else {
                std::fs::remove_file(&path)?;
            }

            println!("{RESTORE}{RED}DELETED{NC}: {display}");
        }
        Ok(())
    }

    fn is_valid(&self) -> Result<bool> {
        // TODO handle requires
        if !self.bin().exists() {
            let display = self.bin().display();
            Err(anyhow!("'{display}' is not present"))
        } else if self.config().is_some_and(|v| !v.exists()) {
            let display = self.config().unwrap().display();
            Err(anyhow!("'{display}' is not present"))
        } else if self.lib().is_some_and(|v| !v.exists()) {
            let display = self.lib().unwrap().display();
            Err(anyhow!("'{display}' is not present"))
        } else {
            Ok(true)
        }
    }
}
