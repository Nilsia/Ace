use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use crate::{args::EditorArg, path_constructer::get_config_path};

#[derive(Deserialize, Debug)]
pub struct Editor {
    pub name: String,
    pub config_path: PathBuf,
    pub binary_path: PathBuf,
    pub files_path: PathBuf,
}
impl Editor {
    fn build_config_path(&self) -> Result<PathBuf> {
        Ok(get_config_path()?.join(&self.name))
    }

    pub(crate) fn remove(&self, args: &EditorArg) -> Result<()> {
        self.remove_config(args)?;
        // TODO remove the others
        Ok(())
    }

    pub(crate) fn install(&self, args: &EditorArg) -> Result<()> {
        self.try_install_config(args)
    }

    pub(crate) fn remove_config(&self, args: &EditorArg) -> Result<()> {
        fs::remove_dir(&self.build_config_path()?)
            .with_context(|| format!("Could not remove your {} configuration", &self.name))?;
        Ok(())
    }

    pub(crate) fn install_config(&self, args: &EditorArg) -> Result<()> {
        // do sometgin here
        Ok(())
    }

    pub(crate) fn try_install_config(&self, args: &EditorArg) -> Result<()> {
        if args.force {
            self.remove_config(args)?;
        }
        let config_dir = self.build_config_path()?;
        let mut answer: String = "".to_string();
        if config_dir.exists() && !args.force {
            println!(
                "Your {} configuration already exists, do you want to overwrite it (N, y) ?",
                &self.name
            );
            std::io::stdin().read_line(&mut answer)?;
        }
        match answer.trim().to_lowercase().as_str() {
            "y" => {
                self.remove_config(args)?;
                self.install_config(args)
            }
            _ => Ok(()),
        }
    }
}
