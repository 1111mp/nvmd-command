use super::ExitStatus;
use super::{anyhow, Result};

use chrono::Local;
use clap::{Parser, Subcommand};
use fs_extra::{
    dir::{ls, DirEntryAttr, DirEntryValue},
    file::{read_to_string, write_all},
};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, Value};
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
use std::{cmp::Ordering, collections::HashSet, env, path::PathBuf};
use version_compare::{compare, Cmp};

use crate::common::{read_json, INSTALLTION_PATH, NVMD_PATH, VERSION};

#[derive(Parser)]
#[command(name=env!("CARGO_PKG_NAME"), author=env!("CARGO_PKG_AUTHORS"), version=env!("CARGO_PKG_VERSION"), about="command tools for nvm-desktop", after_help="Please download new version of Node.js in nvm-desktop.", long_about = None)]
#[command(help_template = "\
{before-help}{name} ({version})
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Get the currently used version
    Current {},
    /// List the all installed versions of Node.js
    List {
        /// List tha all groups of the project
        #[arg(short, long)]
        group: bool,
    },
    /// List the all installed versions of Node.js
    Ls {
        /// List tha all groups of the project
        #[arg(short, long)]
        group: bool,
    },
    /// Use the installed version of Node.js (default is global)
    Use {
        /// The version number of Node.js
        version: String,

        /// Use version for project
        #[arg(short, long)]
        project: bool,
    },
    /// Get the path to the executable to where Node.js was installed
    Which {
        /// The version number of Node.js
        version: String,
    },
}

pub(super) fn command() -> Result<ExitStatus> {
    let cli = Cli::parse();

    let ret = match &cli.command {
        Some(Commands::Current {}) => command_for_current(),
        Some(Commands::Ls { group }) | Some(Commands::List { group }) => {
            if *group {
                command_for_list_group()
            } else {
                command_for_list()
            }
        }
        Some(Commands::Use { version, project }) => {
            if *project {
                command_for_use_project(version)
            } else {
                command_for_use_global(version)
            }
        }
        Some(Commands::Which { version }) => command_for_which(version),
        None => Ok(()),
    };

    match ret {
        Ok(_) => Ok(ExitStatus::from_raw(0)),
        Err(err) => Err(err),
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
struct Group {
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

fn command_for_current() -> Result<()> {
    if let Some(version) = VERSION.clone() {
        eprintln!("v{}", version);
    }
    Ok(())
}

fn command_for_list_group() -> Result<()> {
    if let Some(groups) = get_groups()? {
        for group in &groups {
            if let Some(version) = &group.version {
                eprintln!("{} v{}", group.name, version);
            } else {
                eprintln!("{}", group.name);
            }
        }
    }
    Ok(())
}

fn command_for_list() -> Result<()> {
    if let Some(install_path) = INSTALLTION_PATH.clone() {
        let mut config = HashSet::new();
        config.insert(DirEntryAttr::Name);
        let ls_result = ls(&install_path, &config)?;
        let mut versions: Vec<String> = vec![];
        for item in ls_result.items {
            if let Some(DirEntryValue::String(version)) = item.get(&DirEntryAttr::Name) {
                if is_valid_version(version) {
                    versions.push(version.to_string());
                }
            }
        }
        versions.sort_by(|a, b| match compare(b, a) {
            Ok(Cmp::Lt) => Ordering::Less,
            Ok(Cmp::Eq) => Ordering::Equal,
            Ok(Cmp::Gt) => Ordering::Greater,
            _ => unreachable!(),
        });
        let target_version = VERSION.clone().unwrap_or_default();
        for version in versions {
            if version == target_version {
                eprintln!("v{} (currently)", version);
            } else {
                eprintln!("v{}", version);
            }
        }
    }
    Ok(())
}

fn command_for_use_global(ver: &String) -> Result<()> {
    if is_group_name(ver)? {
        return Err(anyhow!("{} can only be used for projects", ver));
    }
    let version = sanitize_version(ver);
    if !is_valid_version(&version) {
        eprintln!("nvm-desktop: v{} has not been installed", &version);
        return Ok(());
    }
    if let Some(mut default_path) = NVMD_PATH.clone() {
        default_path.push("default");
        write_all(default_path, &version)?;
        eprintln!("Now using node v{}", &version);
    }
    Ok(())
}

fn command_for_use_project(input: &String) -> Result<()> {
    let group = find_group_by_name(input)?;
    let is_group = group.is_some();
    let version = if let Some(group) = group {
        group
            .version
            .ok_or_else(|| anyhow!("the nodejs version of the {} has not been set yet", input))?
    } else {
        sanitize_version(input)
    };
    if !is_valid_version(&version) {
        eprintln!("nvm-desktop: v{} has not been installed", &version);
        return Ok(());
    }

    let project_path = env::current_dir()?;
    let project_name = project_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap();

    update_projects_file(
        project_name,
        &project_path,
        if is_group { input } else { &version },
    )?;
    if is_group {
        update_groups_file(input, project_path.to_str().unwrap())?;
    }

    let mut nvmdrc_file = project_path.clone();
    nvmdrc_file.push(".nvmdrc");
    write_all(nvmdrc_file, &version)?;
    if is_group {
        eprintln!("Now using node v{} ({})", &version, input);
    } else {
        eprintln!("Now using node v{}", &version);
    }

    Ok(())
}

fn command_for_which(ver: &String) -> Result<()> {
    let version = sanitize_version(ver);
    if let Some(mut version_path) = INSTALLTION_PATH.clone() {
        version_path.push(&version);
        if cfg!(unix) {
            version_path.push("bin");
        }
        if version_path.exists() {
            eprintln!("{:?}", version_path);
        } else {
            eprintln!("nvm-desktop: the version cannot be found: v{}", &version);
        }
    }
    Ok(())
}

fn update_groups_file(name: &String, project_path: &str) -> Result<()> {
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
fn find_group_by_name(name: &String) -> Result<Option<Group>> {
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
fn is_group_name(input: &String) -> Result<bool> {
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
fn get_groups() -> Result<Option<Vec<Group>>> {
    if let Some(mut groups_path) = NVMD_PATH.clone() {
        groups_path.push("groups.json");
        if groups_path.exists() {
            let groups = read_json::<Vec<Group>>(&groups_path)?;
            return Ok(Some(groups));
        }
    }
    Ok(None)
}

fn is_valid_version(version: &String) -> bool {
    if let Some(mut version_path) = INSTALLTION_PATH.clone() {
        version_path.push(&version);
        if cfg!(windows) {
            version_path.push("node.exe");
        }
        if cfg!(unix) {
            version_path.push("bin");
            version_path.push("node");
        }
        return version_path.exists();
    }
    false
}

fn sanitize_version(version: &String) -> String {
    let mut version = version.clone();
    if version.starts_with("v") {
        version.remove(0);
    }
    version
}

fn write_json<T: serde::Serialize>(path: &PathBuf, data: &T) -> Result<()> {
    let json_content = serde_json::to_string_pretty(data)?;
    write_all(path, &json_content)?;
    Ok(())
}

fn update_projects_file(project_name: &str, nvmdrc_path: &PathBuf, version: &str) -> Result<()> {
    if let Some(mut projects_path) = NVMD_PATH.clone() {
        projects_path.push("projects.json");
        let mut json_obj = read_json(&projects_path).unwrap_or(json!([]));
        let mut not_exist = true;
        if let Some(projects) = json_obj.as_array_mut() {
            for project in projects.iter_mut() {
                if let Some(name) = project["name"].as_str() {
                    if name == project_name {
                        not_exist = false;
                        project["version"] = json!(version);
                        project["updateAt"] = json!(Local::now().to_string());
                    }
                }
            }
            if not_exist {
                let now = Local::now().to_string();
                let project = json!({
                    "name": project_name,
                    "path": nvmdrc_path,
                    "version": json!(version),
                    "active": true,
                    "createAt": now,
                    "updateAt": now
                });
                projects.insert(0, project);
            }
        }
        write_json(&projects_path, &json_obj)?;
    }
    Ok(())
}
