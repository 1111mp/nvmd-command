use super::ExitStatus;
use super::{anyhow, Result};

use crate::common::{INSTALLTION_DIRECTORY, NVMD_PATH, VERSION};
use crate::utils::group::{
    find_group_by_name, get_groups, is_group_name, update_group_info_by_name,
};
use crate::utils::help::{is_valid_version, sanitize_version};
use crate::utils::project::update_project_info_by_path;

use clap::{Parser, Subcommand};
use fs_extra::{
    dir::{ls, DirEntryAttr, DirEntryValue},
    file::write_all,
};
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
use std::{cmp::Ordering, collections::HashSet, env};
use version_compare::{compare, Cmp};

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

/// nvmd current
fn command_for_current() -> Result<()> {
    if let Some(version) = VERSION.clone() {
        eprintln!("v{}", version);
    }
    Ok(())
}

/// nvmd ls --group
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

/// nvmd list or nvmd ls
fn command_for_list() -> Result<()> {
    if let Some(install_path) = INSTALLTION_DIRECTORY.clone() {
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

/// nvmd use {version}
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

/// nvmd use {version} --project
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
    let project_path_str = project_path.to_str().unwrap();
    let project_name = project_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap();

    update_project_info_by_path(
        project_path_str,
        project_name,
        if is_group { input } else { &version },
    )?;
    if is_group {
        update_group_info_by_name(input, project_path_str)?;
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

/// nvmd which {version}
fn command_for_which(ver: &String) -> Result<()> {
    let version = sanitize_version(ver);
    if let Some(mut version_path) = INSTALLTION_DIRECTORY.clone() {
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
