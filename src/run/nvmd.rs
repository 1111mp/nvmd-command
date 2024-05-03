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
    eprintln!("v{}", *VERSION);
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
    };

    Ok(())
}

fn command_for_list() -> Result<()> {
    let install_path = INSTALLTION_PATH.clone();

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);

    let ls_result = ls(&install_path, &config)?;
    let mut versions: Vec<String> = vec![];
    for item in ls_result.items {
        match item.get(&DirEntryAttr::Name) {
            Some(DirEntryValue::String(version)) => {
                if is_valid_version(version) {
                    versions.push(version.to_string());
                }
            }
            _ => {}
        }
    }
    versions.sort_by(|a, b| match compare(b, a) {
        Ok(Cmp::Lt) => Ordering::Less,
        Ok(Cmp::Eq) => Ordering::Equal,
        Ok(Cmp::Gt) => Ordering::Greater,
        _ => unreachable!(),
    });

    for version in versions {
        if version == *VERSION {
            eprintln!("v{} (currently)", version);
        } else {
            eprintln!("v{}", version);
        }
    }

    Ok(())
}

fn command_for_use_global(ver: &String) -> Result<()> {
    if is_group_name(ver)? {
        return Err(anyhow!("{} can only be used for projects", ver));
    }

    let mut version = ver.to_owned();
    if version.starts_with("v") {
        version.remove(0);
    }

    if !is_valid_version(&version) {
        eprintln!("nvm-desktop: v{} has not been installed", &version);
        return Ok(());
    }

    let mut default_path = NVMD_PATH.clone();
    default_path.push("default");

    write_all(default_path, &version)?;
    eprintln!("Now using node v{}", &version);

    Ok(())
}

fn command_for_use_project(input_ver: &String) -> Result<()> {
    // input_ver may be a version number or a group name
    let is_group = is_group_name(input_ver)?;
    let mut version = if is_group {
        if let Some(version) = get_version_from_group(input_ver)? {
            version
        } else {
            return Err(anyhow!(
                "the nodejs version of the {} has not been set yet",
                input_ver
            ));
        }
    } else {
        input_ver.to_owned()
    };

    if version.starts_with("v") {
        version.remove(0);
    }

    if !is_valid_version(&version) {
        eprintln!("nvm-desktop: v{} has not been installed", &version);
        return Ok(());
    }

    let mut nvmdrc_path = env::current_dir()?;
    let project_name = nvmdrc_path
        .file_name()
        .map(|name| name.to_str().unwrap())
        .unwrap();

    // update projects.json
    let mut projects_path = NVMD_PATH.clone();
    projects_path.push("projects.json");

    let update_projects = || -> Result<()> {
        let mut project = json!({});
        let now = Local::now().to_string();
        project["name"] = json!(project_name);
        project["path"] = json!(nvmdrc_path);
        project["version"] = if is_group {
            json!(input_ver)
        } else {
            json!(version)
        };
        project["active"] = json!(true);
        project["createAt"] = json!(now);
        project["updateAt"] = json!(now);

        let mut json_obj = json!([]);
        if let Some(projects) = json_obj.as_array_mut() {
            projects.push(project);
        }

        let json_str = json_obj.to_string();
        write_all(&projects_path, &json_str)?;

        Ok(())
    };

    let json_str = read_to_string(&projects_path)?;
    if json_str.is_empty() {
        update_projects()?;
    } else {
        let mut json_obj: Value = from_str(&json_str)?;
        let mut not_exist: bool = true;
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
                let mut project = json!({});
                let now = Local::now().to_string();
                project["name"] = json!(project_name);
                project["path"] = json!(nvmdrc_path);
                project["version"] = if is_group {
                    json!(input_ver)
                } else {
                    json!(version)
                };
                project["active"] = json!(true);
                project["createAt"] = json!(now);
                project["updateAt"] = json!(now);

                projects.insert(0, project);
            }
        }
        write_all(&projects_path, &json_obj.to_string())?;
    }

    // update groups.json
    if is_group {
        update_groups_file(input_ver, &nvmdrc_path)?;
    }

    // update .nvmdrc
    nvmdrc_path.push(".nvmdrc");
    write_all(nvmdrc_path, &version)?;
    if is_group {
        eprintln!("Now using node v{} ({})", &version, input_ver);
    } else {
        eprintln!("Now using node v{}", &version);
    }

    Ok(())
}

fn command_for_which(ver: &String) -> Result<()> {
    let mut version = ver.to_owned();
    if version.starts_with("v") {
        version.remove(0);
    }

    let mut version_path = INSTALLTION_PATH.clone();
    version_path.push(&version);
    if cfg!(unix) {
        version_path.push("bin");
    }

    if version_path.exists() {
        eprintln!("{:?}", version_path);
    } else {
        eprintln!("nvm-desktop: the version cannot be found: v{}", &version);
    }

    Ok(())
}

fn update_groups_file(group_name: &String, project_path: &PathBuf) -> Result<()> {
    let mut groups_json = NVMD_PATH.clone();
    groups_json.push("groups.json");

    // "$HOMEPATH/.nvmd/groups.json" must be exist
    let json_str = read_to_string(&groups_json)?;
    let mut json_obj: Value = from_str(&json_str)?;
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
    write_all(&groups_json, &json_obj.to_string())?;

    Ok(())
}

/// get the version of group
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
    };

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
    };

    Ok(false)
}

/// get groups json data from "$HOMEPATH/.nvmd/groups.json"
fn get_groups() -> Result<Option<Vec<Value>>> {
    let mut groups_json = NVMD_PATH.clone();
    groups_json.push("groups.json");
    if !groups_json.exists() {
        return Ok(None);
    }

    let json_str = read_to_string(&groups_json)?;
    let json: Value = serde_json::from_str(&json_str)?;

    Ok(json.as_array().cloned())
}

fn is_valid_version(version: &String) -> bool {
    let mut version_path = INSTALLTION_PATH.clone();
    version_path.push(&version);
    if cfg!(windows) {
        version_path.push("node.exe");
    }
    if cfg!(unix) {
        version_path.push("bin");
        version_path.push("node");
    }

    version_path.exists()
}
