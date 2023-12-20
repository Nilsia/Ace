use crate::args::Args;
use anyhow::Result;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn install_unchecked<P: AsRef<Path>>(from: P, to: P, args: &Args) -> Result<()> {
    if args.symbolic {
        std::os::unix::fs::symlink(from, to)?;
    } else {
        if from.as_ref().is_dir() {
            copy_dir::copy_dir(from, to)?;
        } else {
            fs::copy(from, to)?;
        }
    }
    Ok(())
}

pub fn install<P: AsRef<Path>>(from: P, to: P, args: &Args) -> Result<()> {
    if args.force {
        remove(&to)?;
        install_unchecked(from, to, args)
    } else {
        let path = to.as_ref();
        if path.exists() {
            let mut choice = String::new();

            print!(
                "Warning: Do you want to replace '{}' (y/N): ",
                path.display()
            );
            io::stdout().flush()?;
            io::stdin().read_line(&mut choice)?;

            match choice.trim().to_lowercase().as_str() {
                "y" => {
                    remove(&to)?;
                    install_unchecked(from, to, args)
                }
                _ => Ok(()),
            }
        } else {
            install_unchecked(from, to, args)
        }
    }
}

pub fn remove<P: AsRef<Path>>(path: P) -> Result<()> {
    if path.as_ref().exists() {
        if path.as_ref().is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
    }
    Ok(())
}

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or(PathBuf::from("~/.config/"))
}

pub fn get_bin_dir() -> PathBuf {
    dirs::executable_dir().unwrap_or(PathBuf::from("~/.local/bin/"))
}
