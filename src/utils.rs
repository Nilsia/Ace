use crate::args::Args;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub fn install_unchecked<P: AsRef<Path>>(from: P, to: P, args: &Args) -> Result<()> {
    if args.symbolic {
        std::os::unix::fs::symlink(from, to)?;
    } else {
        fs::copy(from, to)?;
    }
    Ok(())
}

pub fn install<P: AsRef<Path>>(from: P, to: P, args: &Args) -> Result<()> {
    if args.force {
        install_unchecked(from, to, args)
    } else {
        let path = to.as_ref();
        if path.exists() {
            let mut choice = String::new();

            print!("Attention: voulez vous remplacer {}", path.display());
            io::stdin().read_line(&mut choice)?;

            match choice.trim().to_lowercase().as_str() {
                "y" => install_unchecked(from, to, args),
                _ => Ok(()),
            }
        } else {
            install_unchecked(from, to, args)
        }
    }
}

pub fn remove<P: AsRef<Path>>(path: P) -> Result<()> {
    if path.as_ref().exists() {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}

pub fn get_config_path() -> PathBuf {
    dirs::config_dir().unwrap_or(PathBuf::from("~/.config"))
}

pub fn get_bin_path() -> PathBuf {
    dirs::executable_dir().unwrap_or(PathBuf::from("~/.local/bin"))
}
