use super::ExitStatus;
use super::{anyhow, Result};

use chrono::Local;
use clap::{Parser, Subcommand};
use fs_extra::{
    dir::{ls, DirEntryAttr, DirEntryValue},
    file::{read_to_string, write_all},
};
use serde_json::{from_str, json, Value};
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
use std::{cmp::Ordering, collections::HashSet, env, path::PathBuf};
use version_compare::{compare, Cmp};

use crate::common::{INSTALLTION_PATH, NVMD_PATH, VERSION};

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

fn command_for_current() -> Result<()> {
    if let Some(version) = VERSION.clone() {
        eprintln!("v{}", version);
    }
    Ok(())
}

fn command_for_list_group() -> Result<()> {
    if let Some(groups) = get_groups()? {
        for group in &groups {
            if let (Some(name), Some(version)) = (group["name"].as_str(), group["version"].as_str())
            {
                eprintln!("{} v{}", name, version);
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

fn command_for_use_project(input_ver: &String) -> Result<()> {
    let is_group = is_group_name(input_ver)?;
    let version = if is_group {
        get_version_from_group(input_ver)?.ok_or_else(|| {
            anyhow!(
                "the nodejs version of the {} has not been set yet",
                input_ver
            )
        })?
    } else {
        sanitize_version(input_ver)
    };
    if !is_valid_version(&version) {
        eprintln!("nvm-desktop: v{} has not been installed", &version);
        return Ok(());
    }
    let nvmdrc_path = env::current_dir()?;
    let project_name = nvmdrc_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap();

    update_projects_file(project_name, &nvmdrc_path, &version, is_group, input_ver)?;
    if is_group {
        update_groups_file(input_ver, &nvmdrc_path)?;
    }

    let mut nvmdrc_file = nvmdrc_path.clone();
    nvmdrc_file.push(".nvmdrc");
    write_all(nvmdrc_file, &version)?;
    if is_group {
        eprintln!("Now using node v{} ({})", &version, input_ver);
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

fn update_groups_file(group_name: &String, project_path: &PathBuf) -> Result<()> {
    if let Some(mut groups_json) = NVMD_PATH.clone() {
        groups_json.push("groups.json");
        let mut json_obj = read_json(&groups_json)?;
        let project_path_json = json!(project_path);
        if let Some(groups) = json_obj.as_array_mut() {
            for group in groups.iter_mut() {
                if let Some(name) = group["name"].as_str() {
                    if name == group_name {
                        if let Some(projects) = group["projects"].as_array_mut() {
                            if !projects.contains(&project_path_json) {
                                projects.push(project_path_json.clone());
                            }
                        }
                    }
                }
            }
        }
        write_json(&groups_json, &json_obj)?;
    }
    Ok(())
}

fn get_version_from_group(group_name: &String) -> Result<Option<String>> {
    if let Some(groups) = get_groups()? {
        for group in &groups {
            if let Some(name) = group["name"].as_str() {
                if group_name == name {
                    if let Some(version) = group["version"].as_str() {
                        return Ok(Some(version.to_string()));
                    }
                }
            }
        }
    }
    Ok(None)
}

fn is_group_name(version: &String) -> Result<bool> {
    if let Some(groups) = get_groups()? {
        for group in &groups {
            if let Some(name) = group["name"].as_str() {
                if version == name {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn get_groups() -> Result<Option<Vec<Value>>> {
    if let Some(mut groups_json) = NVMD_PATH.clone() {
        groups_json.push("groups.json");
        if !groups_json.exists() {
            return Ok(None);
        }
        let json = read_json(&groups_json)?;
        return Ok(json.as_array().cloned());
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

fn read_json(path: &PathBuf) -> Result<Value> {
    let json_str = read_to_string(path)?;
    let json: Value = from_str(&json_str)?;
    Ok(json)
}

fn write_json(path: &PathBuf, json: &Value) -> Result<()> {
    let json_str = json.to_string();
    write_all(path, &json_str)?;
    Ok(())
}

fn update_projects_file(
    project_name: &str,
    nvmdrc_path: &PathBuf,
    version: &str,
    is_group: bool,
    input_ver: &String,
) -> Result<()> {
    if let Some(mut projects_path) = NVMD_PATH.clone() {
        projects_path.push("projects.json");
        let mut json_obj = read_json(&projects_path)?;
        let mut not_exist = true;
        if let Some(projects) = json_obj.as_array_mut() {
            for project in projects.iter_mut() {
                if let Some(name) = project["name"].as_str() {
                    if name == project_name {
                        not_exist = false;
                        project["version"] = if is_group {
                            json!(input_ver)
                        } else {
                            json!(version)
                        };
                        project["updateAt"] = json!(Local::now().to_string());
                    }
                }
            }
            if not_exist {
                let now = Local::now().to_string();
                let project = json!({
                    "name": project_name,
                    "path": nvmdrc_path,
                    "version": if is_group { json!(input_ver) } else { json!(version) },
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
