use anyhow::Result;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::common::NVMD_PATH;

use super::help::{read_json, write_json};

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

/// update projects data
pub fn update_project_info_by_path(path: &str, name: &str, version: &str) -> Result<()> {
    if let Some(mut projects_path) = NVMD_PATH.clone() {
        projects_path.push("projects.json");
        let mut projects = read_json::<Vec<Project>>(&projects_path).unwrap_or(vec![]);
        let mut not_exist = true;
        for project in &mut projects {
            if path == project.path {
                not_exist = false;
                project.version = Some(version.to_owned());
                project.update_at = Some(
                    OffsetDateTime::now_local()
                        .unwrap_or_else(|_| OffsetDateTime::now_utc())
                        .to_string(),
                );
            }
        }

        if not_exist {
            let now = OffsetDateTime::now_local()
                .unwrap_or_else(|_| OffsetDateTime::now_utc())
                .to_string();
            let project = Project {
                name: name.to_owned(),
                path: path.to_owned(),
                version: Some(version.to_owned()),
                active: true,
                update_at: Some(now.to_owned()),
                create_at: Some(now),
            };
            projects.insert(0, project);
        }

        write_json(&projects_path, &projects)?;
    }

    Ok(())
}

// /// read projects data from 'projects.json'
// /// default data is '[]'
// pub fn read_projects() -> Result<Vec<Project>> {
//     if let Some(mut projects_path) = NVMD_PATH.clone() {
//         projects_path.push("projects.json");
//         let projects = read_json::<Vec<Project>>(&projects_path).unwrap_or(vec![]);
//         return Ok(projects);
//     }

//     Ok(vec![])
// }
