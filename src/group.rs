use std::{collections::HashMap, io::Write};

use crate::{
    args::Args,
    dependencies::Dependencies,
    tool::Tool,
    utils::{CYAN, GREEN, NC, YELLOW},
};
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Group {
    pub name: String,
    pub dependencies: Vec<String>,
}
impl Group {
    pub(crate) fn list(
        &self,
        dependencies: &Dependencies,
        group_key: &String,
        tools: Option<&HashMap<String, Tool>>,
        args: &Args,
    ) -> Result<()> {
        let dep_errors: Vec<(&String, String)> = self
            .dependencies
            .iter()
            .map(|key| {
                (
                    key,
                    if tools.as_ref().is_some_and(|tools| tools.contains_key(key))
                        && !dependencies.unsatisfied_tools.contains_key(key)
                    {
                        String::new()
                    } else {
                        format!("{YELLOW}ERROR{NC}")
                    },
                )
            })
            .collect();
        let group_error = if dep_errors.iter().any(|(_, s)| !s.is_empty()) {
            format!("{CYAN}ERROR{NC}")
        } else {
            String::new()
        };
        if args.verbose {
            print!(
                "Group : {GREEN}{}{NC} {}\n\tDependencies : \n",
                group_key, group_error
            );
            for (key, error) in &dep_errors {
                print!("\t - {key} {error}\n");
            }
        } else {
            print!(" - {GREEN}{group_key}{NC} {group_error}\n");
        }
        std::io::stdout().flush()?;
        Ok(())
    }
}
