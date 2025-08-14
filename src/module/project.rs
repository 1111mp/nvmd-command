use super::nvmd_home;
use crate::utils::help::{read_json, write_json};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use time::OffsetDateTime;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// is it active
    pub active: bool,

    /// project name
    pub name: String,

    /// project path
    pub path: String,

    /// the node version of project used
    pub version: Option<String>,

    /// create date
    pub create_at: Option<String>,

    /// update date
    pub update_at: Option<String>,
}

#[derive(Debug)]
pub struct Projects {
    pub path: PathBuf,
    pub data: Vec<Project>,
}

impl Projects {
    pub fn new() -> Result<Self> {
        let path = nvmd_home()?.projects_path();
        let data = read_json::<Vec<Project>>(&path).unwrap_or_default();
        Ok(Self { path, data })
    }

    pub fn save(&self) -> Result<()> {
        write_json(&self.path, &self.data)
    }

    pub fn patch(&mut self, path: &str, name: &str, version: &str) {
        let now = OffsetDateTime::now_local()
            .unwrap_or_else(|_| OffsetDateTime::now_utc())
            .to_string();

        if let Some(project) = self.data.iter_mut().find(|p| p.path == path) {
            project.version = Some(version.to_string());
            project.update_at = Some(now);
        } else {
            let project = Project {
                name: name.to_string(),
                path: path.to_string(),
                version: Some(version.to_string()),
                active: true,
                create_at: Some(now.clone()),
                update_at: Some(now),
            };
            self.data.insert(0, project);
        }
    }

    /// update projects data
    pub fn update_and_save(path: &str, name: &str, version: &str) -> Result<()> {
        let mut projects = Projects::new()?;
        projects.patch(path, name, version);
        projects.save()
    }
}
