use std::path::PathBuf;

pub fn get_config_path() -> PathBuf {
    dirs::config_dir().unwrap_or(PathBuf::from("~/.config"))
}

pub fn get_bin_path() -> PathBuf {
    dirs::executable_dir().unwrap_or(PathBuf::from("~/.local/bin"))
}
