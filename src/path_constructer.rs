use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Result;

pub fn get_config_path() -> Result<PathBuf> {
    match dirs::config_dir() {
        Some(v) => Ok(v),
        None => return Err(anyhow!("Cannot get config directory")),
    }
}
