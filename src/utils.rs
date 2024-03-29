use anyhow::{anyhow, Result};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const SAVE: &str = "\x1b[s";
pub const RESTORE: &str = "\x1b[2K\x1b[u";

pub const RED: &str = "\x1b[0;31m";
pub const CYAN: &str = "\x1b[0;36m";
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

pub fn get_data_dir() -> PathBuf {
    dirs::data_dir().unwrap_or(PathBuf::from("~/.local/share/"))
}

pub fn create_dirs() -> Result<()> {
    if !get_config_dir().exists() {
        std::fs::create_dir_all(get_config_dir())?;
    }
    if !get_bin_dir().exists() {
        std::fs::create_dir_all(get_bin_dir())?;
    }
    if !get_data_dir().exists() {
        std::fs::create_dir_all(get_data_dir())?;
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

// retunr true if added to PATH and false otherwise
pub fn export_bin_dir() -> Result<bool> {
    if !check_path() {
        let mut config = std::fs::OpenOptions::new()
            .append(true)
            .open(get_shell_config_path())?;
        let export = format!("export PATH=\"{}:$PATH\"\n", get_bin_dir().display());
        config.write_all(export.as_bytes())?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn iter_includes<P, V, U>(owner: V, includer: U) -> bool
where
    P: PartialEq,
    V: IntoIterator<Item = P>,
    U: IntoIterator<Item = P>,
{
    let owner_vec = owner.into_iter().collect::<Vec<P>>();
    includer.into_iter().all(|v| owner_vec.contains(&v))
}

pub fn make_absolute<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

pub fn find_common_path<P: AsRef<Path>>(one: P, two: P) -> Result<PathBuf> {
    let mut result = PathBuf::new();
    let one = make_absolute(one)?;
    let two = make_absolute(two)?;
    // Loop over paths components until they differs
    for (o, t) in one.components().zip(two.components()) {
        if o == t {
            result.push(o);
        } else {
            break;
        }
    }

    if result.exists() {
        Ok(result)
    } else {
        Err(anyhow!(
            "No common ancestor for '{}' and '{}'",
            one.display(),
            two.display()
        ))
    }
}

pub fn find_relative_path<P: AsRef<Path>>(from: P, to: P) -> Result<PathBuf> {
    let from = make_absolute(from)?;
    let to = make_absolute(to)?;
    // let a = None;
    // a.map_or_else(, )

    Ok(from.strip_prefix(to)?.to_path_buf())
}

pub fn prompt(message: &str) -> Result<String> {
    print!("{message}");
    let mut choice = String::new();
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut choice)?;
    Ok(choice)
}
pub fn existence(path: &PathBuf) -> Result<String> {
    if path.try_exists()? {
        Ok(String::new())
    } else {
        Ok(format!("({RED}NOT FOUND{NC})"))
    }
}
