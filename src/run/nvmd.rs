use super::ExitStatus;
use chrono::Local;
use clap::{Parser, Subcommand};
use fs_extra::{
    dir::{ls, DirEntryAttr, DirEntryValue},
    error::Error,
    file::{read_to_string, write_all},
};
use serde_json::{from_str, json, Value};
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
use std::{cmp::Ordering, collections::HashSet, env};
use version_compare::{compare, Cmp};

use crate::common::{NVMD_PATH, VERSION};

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
    List {},
    /// List the all installed versions of Node.js
    Ls {},
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

pub(super) fn command() -> Result<ExitStatus, String> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Current {}) => {
            eprintln!("v{}", *VERSION);
        }
        Some(Commands::Ls {}) | Some(Commands::List {}) => {
            match command_for_list() {
                Ok(()) => {}
                Err(err) => {
                    return Err(err.to_string());
                }
            };
        }
        Some(Commands::Use { version, project }) => {
            if *project {
                command_for_use_project(version);
            } else {
                command_for_use_global(version);
            }
        }
        Some(Commands::Which { version }) => {
            command_for_which(version);
        }
        None => {}
    };

    Ok(ExitStatus::from_raw(0))
}

fn command_for_list() -> Result<(), Error> {
    let mut install_path = NVMD_PATH.clone();
    install_path.push("versions");

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);

    let ls_result = ls(install_path, &config)?;
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

fn command_for_use_global(ver: &String) {
    let mut version = ver.to_owned();
    if version.starts_with("v") {
        version.remove(0);
    }

    if !is_valid_version(&version) {
        eprintln!("nvm-desktop: v{} has not been installed", &version);
        return;
    }

    let mut default_path = NVMD_PATH.clone();
    default_path.push("default");

    match write_all(default_path, &version) {
        Ok(()) => {
            eprintln!("Now using node v{}", &version);
        }
        Err(err) => {
            eprintln!("nvm-desktop: {}", err);
        }
    };
}

fn command_for_use_project(ver: &String) {
    let mut version = ver.to_owned();
    if version.starts_with("v") {
        version.remove(0);
    }

    if !is_valid_version(&version) {
        eprintln!("nvm-desktop: v{} has not been installed", &version);
        return;
    }

    let mut nvmdrc_path = env::current_dir().unwrap();
    let project_name = nvmdrc_path
        .file_name()
        .map(|name| name.to_str().unwrap())
        .unwrap();

    // update projects.json
    let mut projects_path = NVMD_PATH.clone();
    projects_path.push("projects.json");

    let update_projects = || {
        let mut pro_obj = json!({});
        let now = Local::now().to_string();
        pro_obj["name"] = json!(project_name);
        pro_obj["path"] = json!(nvmdrc_path);
        pro_obj["version"] = json!(version);
        pro_obj["active"] = json!(true);
        pro_obj["createAt"] = json!(now);
        pro_obj["updateAt"] = json!(now);

        let mut json_obj = json!([]);
        json_obj.as_array_mut().unwrap().push(pro_obj);

        let json_str = json_obj.to_string();
        write_all(&projects_path, &json_str).unwrap();
    };

    match read_to_string(&projects_path) {
        Ok(content) => {
            if content.is_empty() {
                update_projects();
            } else {
                let mut json_obj: Value = from_str(&content).unwrap();
                let projects = json_obj.as_array_mut().unwrap();
                for project in projects {
                    let name = project["name"].as_str().unwrap();
                    if name == project_name {
                        project["version"] = json!(version);
                        project["updateAt"] = json!(Local::now().to_string());
                    }
                }

                let json_str = json_obj.to_string();
                write_all(&projects_path, &json_str).unwrap();
            }
        }
        Err(_) => {
            update_projects();
        }
    };

    // update .nvmdrc
    nvmdrc_path.push(".nvmdrc");

    match write_all(nvmdrc_path, &version) {
        Ok(()) => {
            eprintln!("Now using node v{}", &version);
        }
        Err(err) => {
            eprintln!("nvm-desktop: {}", err);
        }
    };
}

fn command_for_which(ver: &String) {
    let mut version = ver.to_owned();
    if version.starts_with("v") {
        version.remove(0);
    }

    let mut version_path = NVMD_PATH.clone();
    version_path.push("versions");
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

fn is_valid_version(version: &String) -> bool {
    let mut version_path = NVMD_PATH.clone();
    version_path.push("versions");
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
