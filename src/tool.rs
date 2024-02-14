use crate::{
    dependencies::Dependencies,
    package::Package,
    utils::{CYAN, GREEN, NC, RED},
};
use anyhow::Result;
use phf::{phf_map, Map};
use serde::Deserialize;
use std::{collections::HashMap, io::Write, path::PathBuf};

const TOOL_FIELD_STR: Map<&str, &str> =
    phf_map! {"bin" => "Binary Source", "lib" => "Library Source", "config" => "Configuration"};

#[derive(Deserialize, Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub bin: PathBuf,
    pub config: Option<PathBuf>,
    pub lib: Option<PathBuf>,
    pub dependencies: Option<Vec<String>>,
}
impl Tool {
    pub(crate) fn get_invalid_paths(&self) -> Result<HashMap<String, &PathBuf>> {
        let mut paths = HashMap::new();
        if !self.bin.try_exists()? {
            paths.insert(String::from("bin"), &self.bin);
        }
        if let Some(lib) = self.lib.as_ref() {
            if !lib.try_exists()? {
                paths.insert(String::from("lib"), lib);
            }
        }
        if let Some(config) = self.config.as_ref() {
            if !config.try_exists()? {
                paths.insert(String::from("config"), config);
            }
        }
        Ok(paths)
    }

    fn generate_self_map(&self) -> HashMap<&str, Option<&PathBuf>> {
        HashMap::from([
            ("bin", Some(&self.bin)),
            ("lib", self.lib.as_ref()),
            ("config", self.config.as_ref()),
        ])
    }

    pub(crate) fn list(&self, dependencies: &Dependencies, tool_key: &String) -> Result<()> {
        let self_map = self.generate_self_map();
        let error_tool = if dependencies.as_errors(tool_key) {
            format!("{CYAN}ERROR{NC}")
        } else {
            String::new()
        };
        print!(
            "Tool: {GREEN}{}{NC} (lsp: {GREEN}{}{NC}) {RED}{error_tool}{NC}\n",
            tool_key, self.name,
        );

        for (field_key, value) in self_map.iter() {
            if let Some(label) = TOOL_FIELD_STR.get(field_key) {
                print!(
                    "\t{} : {} {RED}{}{NC}\n",
                    label,
                    value.map_or(String::from("not given"), |v| v.display().to_string()),
                    dependencies
                        .get_path_error(tool_key, field_key)
                        .unwrap_or(String::new())
                );
            }
        }

        if let Some(deps) = self.dependencies.as_ref() {
            print!("\tDependencies :\n");
            for dep in deps {
                print!(
                    "\t - {} {RED}{}{NC}\n",
                    dep,
                    dependencies
                        .get_error_dependencies(&String::from(tool_key), dep)
                        .unwrap_or(String::new())
                );
            }
        }
        print!("\n");
        std::io::stdout().flush()?;
        Ok(())
    }
}

impl Package for Tool {
    fn name(&self) -> &String {
        &self.name
    }

    fn bin(&self) -> &PathBuf {
        &self.bin
    }

    fn config(&self) -> Option<&PathBuf> {
        self.config.as_ref()
    }

    fn lib(&self) -> Option<&PathBuf> {
        self.lib.as_ref()
    }
}
