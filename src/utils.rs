use anyhow::Result;
use std::io::Write;
use std::path::PathBuf;

pub const SAVE: &str = "\x1b[s";
pub const RESTORE: &str = "\x1b[u";

pub const RED: &str = "\x1b[0;31m";
pub const GREEN: &str = "\x1b[0;32m";
pub const YELLOW: &str = "\x1b[0;33m";
pub const BLUE: &str = "\x1b[0;34m";
pub const NC: &str = "\x1b[0m";

pub fn get_home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or(PathBuf::from("~"))
}

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or(PathBuf::from("~/.config/"))
}

pub fn get_bin_dir() -> PathBuf {
    dirs::executable_dir().unwrap_or(PathBuf::from("~/.local/bin/"))
}

pub fn create_dirs() -> Result<()> {
    if !get_config_dir().exists() {
        std::fs::create_dir(get_config_dir())?;
    }
    if !get_bin_dir().exists() {
        std::fs::create_dir_all(get_bin_dir())?;
    }
    Ok(())
}

pub fn get_shell_config_path() -> PathBuf {
    if let Ok(shell) = std::env::var("SHELL") {
        if let Some(basename) = PathBuf::from(shell).file_name() {
            return get_home_dir().join(format!(".{}rc", basename.to_string_lossy()));
        }
    }
    get_home_dir().join(".bashrc")
}

pub fn check_path() -> bool {
    if let Some(dir) = get_bin_dir().to_str() {
        std::env::var("PATH").is_ok_and(|path| path.contains(dir))
    } else {
        false
    }
}

pub fn export_bin_dir() -> Result<()> {
    if !check_path() {
        let mut config = std::fs::OpenOptions::new()
            .append(true)
            .open(get_shell_config_path())?;
        let export = format!("export PATH=\"$PATH:{}\"\n", get_bin_dir().display());
        config.write_all(export.as_bytes())?;
    }
    Ok(())
}

pub fn clear_line() -> Result<()> {
    Ok(std::io::stdout().write_all(b"\x1b[u")?)
}
pub fn vec_includes<P, V, U>(owner: V, includer: U) -> bool
where
    P: PartialEq,
    V: IntoIterator<Item = P>,
    U: IntoIterator<Item = P>,
{
    let owner_vec = owner.into_iter().collect::<Vec<P>>();
    includer.into_iter().all(|v| owner_vec.contains(&v))
}
