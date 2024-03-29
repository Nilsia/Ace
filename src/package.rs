use crate::args::Args;
use crate::utils::{
    find_common_path, find_relative_path, get_bin_dir, get_config_dir, get_data_dir, make_absolute,
    prompt,
};
use crate::utils::{BLUE, GREEN, NC, RED, RESTORE, SAVE, YELLOW};
use anyhow::{anyhow, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

#[derive(Debug)]
pub enum PackageResult {
    Canceled,
    Removed,
    Ignored,
    Installed,
    Linked,
}

/// (bin, config)
pub type PackageInfo = (PackageResult, PackageResult);

pub trait Package {
    fn name(&self) -> &String;

    fn bin(&self) -> &PathBuf;

    fn config(&self) -> Option<&PathBuf>;

    fn lib(&self) -> Option<&PathBuf>;

    fn get_config_name(&self) -> Option<String> {
        self.config().map(|v| {
            v.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(self.name())
                .to_string()
        })
    }

    fn get_bin_path(&self) -> PathBuf {
        get_bin_dir().join(self.name())
    }

    fn get_config_path(&self) -> Option<PathBuf> {
        self.get_config_name()
            .map(|name| get_config_dir().join(name))
    }

    fn install(&self, args: &Args) -> Result<PackageInfo> {
        Ok((self.install_bin(args)?, self.install_config(args)?))
    }

    fn install_bin(&self, args: &Args) -> Result<PackageResult> {
        if args.symbolic || self.lib().is_none() {
            self.install_files(self.bin(), &self.get_bin_path(), args)
        } else {
            let lib = self.lib().unwrap();
            let ancestor = find_common_path(self.bin(), lib)?;
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

    fn install_config(&self, args: &Args) -> Result<PackageResult> {
        if let (Some(config), Some(path)) = (self.config(), &self.get_config_path()) {
            self.install_files(config, path, args)
        } else {
            Ok(PackageResult::Ignored)
        }
    }

    fn remove(&self, args: &Args) -> Result<PackageInfo> {
        Ok((self.remove_bin(args)?, self.remove_config(args)?))
    }

    fn remove_bin(&self, args: &Args) -> Result<PackageResult> {
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

    fn remove_config(&self, args: &Args) -> Result<PackageResult> {
        if let Some(config) = self.get_config_path() {
            if args.force {
                self.remove_files_unchecked(config)
            } else {
                self.remove_files(config)
            }
        } else {
            Ok(PackageResult::Ignored)
        }
    }

    fn install_files_unchecked<P: AsRef<Path>>(
        &self,
        from: P,
        to: P,
        args: &Args,
    ) -> Result<PackageResult> {
        let f_display = from.as_ref().display();
        let t_display = to.as_ref().display();
        if args.symbolic {
            std::os::unix::fs::symlink(make_absolute(&from)?, &to)?;
            println!("{BLUE}LINKED{NC}: {t_display} {BLUE}->{NC} {f_display}");
            Ok(PackageResult::Linked)
        } else {
            print!("{SAVE}INSTALLING: {f_display} -> {t_display}");
            io::stdout().flush()?;

            if from.as_ref().is_dir() {
                copy_dir::copy_dir(&from, &to)?;
            } else {
                fs::copy(&from, &to)?;
            }

            println!("{RESTORE}{GREEN}INSTALLED{NC}: {f_display} -> {t_display}");
            Ok(PackageResult::Installed)
        }
    }

    fn install_files<P: AsRef<Path>>(&self, from: P, to: P, args: &Args) -> Result<PackageResult> {
        if args.force {
            self.remove_files_unchecked(&to)?;
            self.install_files_unchecked(&from, &to, args)
        } else {
            let path = to.as_ref();
            if path.exists() || path.is_symlink() {
                let display = path.display();

                match prompt(&format!(
                    "{SAVE}{YELLOW}WARNING{NC}: Do you want to overwrite '{display}' (y/N): ",
                ))?
                .as_str()
                {
                    "y" => {
                        self.remove_files_unchecked(&to)?;
                        self.install_files_unchecked(from, to, args)
                    }
                    _ => Ok(PackageResult::Canceled),
                }
            } else {
                self.install_files_unchecked(from, to, args)
            }
        }
    }

    fn remove_files<P: AsRef<Path>>(&self, path: P) -> Result<PackageResult> {
        let path = path.as_ref();
        if path.exists() || path.is_symlink() {
            let display = path.display();
            match prompt(&format!(
                "{SAVE}{YELLOW}WARNING{NC}: Do you want to remove '{display}' (Y/n): "
            ))?
            .trim()
            {
                "n" => {
                    println!("{YELLOW}WARNING{NC}: Canceled '{display}' deletion");
                    Ok(PackageResult::Canceled)
                }
                _ => self.remove_files_unchecked(path),
            }
        } else {
            Ok(PackageResult::Ignored)
        }
    }

    fn remove_files_unchecked<P: AsRef<Path>>(&self, path: P) -> Result<PackageResult> {
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
            return Ok(PackageResult::Removed);
        }
        Ok(PackageResult::Ignored)
    }

    fn validate(&self) -> Result<()> {
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
            Ok(())
        }
    }
}
