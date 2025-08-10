use super::nvmd_home;
use crate::utils::help::{read_json, write_json};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Group {
    /// group name
    pub name: String,

    /// group desc
    pub desc: Option<String>,

    /// the group contains projects
    #[serde(default = "default_projects")]
    pub projects: Vec<String>,

    /// the node version of group used
    pub version: Option<String>,
}

fn default_projects() -> Vec<String> {
    vec![]
}

#[derive(Debug)]
pub struct Groups {
    pub path: PathBuf,
    pub data: Vec<Group>,
}

impl Groups {
    pub fn new() -> Result<Self> {
        let path = nvmd_home()?.groups_path();
        let data = read_json::<Vec<Group>>(&path).unwrap_or_default();
        Ok(Self { path, data })
    }

    pub fn save(&self) -> Result<()> {
        write_json(&self.path, &self.data)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Group> {
        self.data.iter().find(|g| g.name == name)
    }

    pub fn exists(&self, name: &str) -> bool {
        self.data.iter().any(|g| g.name == name)
    }

    pub fn update(&mut self, name: &str, project_path: &str) {
        if let Some(group) = self.data.iter_mut().find(|g| g.name == name) {
            if !group.projects.iter().any(|p| p == project_path) {
                group.projects.push(project_path.to_string());
            }
        }
    }
}
