use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::common::NVMD_PATH;

use super::help::{read_json, write_json};

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

/// update group info by name
pub fn update_group_info_by_name(name: &String, project_path: &str) -> Result<()> {
    if let Some(mut groups_path) = NVMD_PATH.clone() {
        groups_path.push("groups.json");
        let mut groups = read_json::<Vec<Group>>(&groups_path)?;
        for group in &mut groups {
            let project_path = project_path.to_string();
            if name == &group.name && !group.projects.contains(&project_path) {
                group.projects.push(project_path);
            }
        }
        write_json(&groups_path, &groups)?;
    }
    Ok(())
}

/// query group data by group name
pub fn find_group_by_name(name: &String) -> Result<Option<Group>> {
    if let Some(groups) = get_groups()? {
        for group in groups {
            if name == &group.name {
                return Ok(Some(group));
            }
        }
    }
    Ok(None)
}

/// determine whether the input is a group name
pub fn is_group_name(input: &String) -> Result<bool> {
    if let Some(groups) = get_groups()? {
        for group in &groups {
            if input == &group.name {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// read groups data from 'groups.json' file
pub fn get_groups() -> Result<Option<Vec<Group>>> {
    if let Some(mut groups_path) = NVMD_PATH.clone() {
        groups_path.push("groups.json");
        if groups_path.exists() {
            let groups = read_json::<Vec<Group>>(&groups_path)?;
            return Ok(Some(groups));
        }
    }
    Ok(None)
}
