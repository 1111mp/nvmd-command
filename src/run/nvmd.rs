use super::ExitStatus;
use clap::{Parser, Subcommand};
use fs_extra::{
    dir::{ls, DirEntryAttr, DirEntryValue},
    error::Error,
    file::write_all,
};
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
use std::{cmp::Ordering, collections::HashSet, env};
use version_compare::{compare, Cmp};

use crate::common::{NVMD_PATH, VERSION};

#[derive(Parser)]
#[command(name="nvmd", author="The1111mp@outlook.com", version="2.2.0", about="command tools for nvm-desktop", after_help="Please download new version of Node.js in nvm-desktop.", long_about = None)]
#[command(help_template = "\
{before-help}{name} ({version})
{author-with-newline}{about-with-newline}
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
                versions.push(version.clone());
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

    if !is_uninstall_version(&version) {
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

    if !is_uninstall_version(&version) {
        eprintln!("nvm-desktop: v{} has not been installed", &version);
        return;
    }

    let mut nvmdrc_path = env::current_dir().unwrap();
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

fn is_uninstall_version(version: &String) -> bool {
    let mut version_path = NVMD_PATH.clone();
    version_path.push("versions");
    version_path.push(&version);
    if cfg!(unix) {
        version_path.push("bin");
    }

    version_path.exists()
}
