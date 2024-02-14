use crate::{
    args::Args,
    utils::{NC, RED, YELLOW},
};
use anyhow::{anyhow, Result};
use std::{collections::HashMap, fmt::Display, io::Write, path::PathBuf};
use toml::toml;

use crate::{config::Config, group::Group, tool::Tool};
#[derive(Debug, Clone)]
pub enum DependencyErrorType {
    NotFound,
    UnSatisfiedDepencies,
}

impl Display for DependencyErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            match self {
                DependencyErrorType::NotFound => "is not found",
                DependencyErrorType::UnSatisfiedDepencies => "contains invalid dependencies",
            }
        ))
    }
}

impl DependencyErrorType {
    pub fn to_uppercase(&self) -> String {
        match self {
            DependencyErrorType::NotFound => format!("{RED}NOT FOUND{NC}"),
            DependencyErrorType::UnSatisfiedDepencies => format!("{YELLOW}INVALID DEPENDENCY{NC}"),
        }
    }
}

#[derive(Debug)]
pub struct UnSatisfiedTool<'l> {
    pub tool: &'l Tool,
    pub required: Option<HashMap<&'l String, (Tool, DependencyErrorType)>>,
    pub paths: Option<HashMap<String, &'l PathBuf>>,
}

#[derive(Debug)]
pub struct UnSatisfiedGroup<'l> {
    pub group: &'l Group,
    pub unsatisfied_tools: Vec<&'l str>,
}

#[derive(Default, Debug)]
pub struct Dependencies<'l> {
    pub checked_tool_keys: Vec<&'l String>,
    pub satisfied_tools: HashMap<String, &'l Tool>,
    pub satisfied_groups: HashMap<String, &'l Group>,
    pub unsatisfied_tools: HashMap<String, UnSatisfiedTool<'l>>,
    pub unsatisfied_groups: HashMap<String, UnSatisfiedGroup<'l>>,
}

impl<'l> Dependencies<'l> {
    pub fn validate(&self, config: &Config, args: &Args) -> Result<()> {
        if !self.unsatisfied_tools.is_empty() || !self.unsatisfied_groups.is_empty() {
            self.print_unsatisfied_dependencies(config, args)?;
            return Err(anyhow!(" because of previous explanation"));
        }
        Ok(())
    }
    fn print_unsatisfied_dependencies(&self, config: &Config, args: &Args) -> Result<()> {
        print!("\nThe following tools and groups are not valid :\n\n");
        for (tool_key, tool) in &self.unsatisfied_tools {
            tool.tool.list(self, tool_key, args)?;
        }
        for (group_key, group) in &self.unsatisfied_groups {
            group
                .group
                .list(self, &group_key, config.tools.as_ref(), args)?;
        }
        std::io::stdout().flush()?;
        Ok(())
    }

    pub(crate) fn get_error_dependencies(
        &self,
        tool_key: &String,
        dep_key: &String,
    ) -> Option<String> {
        self.unsatisfied_tools.get(tool_key).map(|v| {
            v.required.as_ref().map_or(String::new(), |t| {
                t.get(dep_key).map_or(String::new(), |d| d.1.to_uppercase())
            })
        })
    }

    pub(crate) fn as_errors(&self, tool_key: &String) -> bool {
        self.unsatisfied_tools
            .get(tool_key)
            .is_some_and(|tool| tool.required.is_some() || tool.paths.is_some())
    }

    pub(crate) fn get_path_error(&self, tool_key: &String, field_key: &str) -> Option<String> {
        self.unsatisfied_tools
            .get(tool_key)
            .map(|t| {
                if t.paths
                    .as_ref()
                    .is_some_and(|pathes| pathes.contains_key(field_key))
                {
                    "INVALID PATH"
                } else {
                    ""
                }
            })
            .map(|v| String::from(v))
    }

    fn list_for_action<U>(&self, label: &str, after: &str, keys: U)
    where
        U: IntoIterator<Item = &'l String>,
    {
        let items = keys
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        if items.len() != 0 {
            print!("{label} {}{after}", items.join(", "));
        }
    }

    pub(crate) fn list_tools_for_action(&self, label: &str, after: &str) -> Result<()> {
        self.list_for_action(label, after, self.satisfied_tools.keys());
        Ok(std::io::stdout().flush()?)
    }

    // pub(crate) fn list_groups_for_action(&self, label: &str) -> Result<()> {
    //     self.list_for_action(label, self.satisfied_groups.keys());
    //     Ok(std::io::stdout().flush()?)
    // }

    // pub(crate) fn list_all_for_action(&self, label: &str) -> Result<()> {
    //     self.list_tools_for_action(label)?;
    //     self.list_groups_for_action(label)?;
    //     Ok(())
    // }
}
