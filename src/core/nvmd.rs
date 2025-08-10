use super::ExitStatus;
use super::{anyhow, Result};
use crate::module::{nvmd_home, Context, Groups, Projects, Setting};
use crate::node::Node;
use crate::utils::help::{node_strict_available, sanitize_version};
use anyhow::bail;
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
#[command(
    name = env!("CARGO_PKG_NAME"),
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = "command tools for nvm-desktop",
    after_help = "Please download new version of Node.js in nvm-desktop.",
    long_about = None,
    styles = CLAP_STYLING
)]
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

// See also `clap_cargo::style::CLAP_STYLING`
pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);

#[derive(Subcommand)]
enum Commands {
    /// Get the currently used version
    Current {},
    /// Install the specified version of Node.js
    Install {
        /// The version number of Node.js
        version: String,
    },
    /// Uninstall the specified version of Node.js
    Uninstall {
        /// The version number of Node.js
        version: String,
    },
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
        Some(Commands::Install { version }) => Node::install(version),
        Some(Commands::Uninstall { version }) => {
            eprintln!("{}", version);
            Ok(())
        }
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
    if let Some(version) = Context::global()?.get_version() {
        eprintln!("v{}", version);
    }
    Ok(())
}

/// nvmd ls --group
fn command_for_list_group() -> Result<()> {
    for group in &Groups::new()?.data {
        if let Some(version) = &group.version {
            eprintln!("{} v{}", group.name, version);
        } else {
            eprintln!("{}", group.name);
        }
    }
    Ok(())
}

/// nvmd list or nvmd ls
fn command_for_list() -> Result<()> {
    let path = Setting::global()?.get_directory()?;
    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    let ls_result = ls(&path, &config)?;

    let mut versions: Vec<String> = vec![];
    for item in ls_result.items {
        if let Some(DirEntryValue::String(version)) = item.get(&DirEntryAttr::Name) {
            if node_strict_available(version)? {
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
    let target_version = Context::global()?.get_version().unwrap_or_default();
    for version in versions {
        if version == target_version {
            eprintln!("v{} (currently)", version);
        } else {
            eprintln!("v{}", version);
        }
    }

    Ok(())
}

/// nvmd use {version}
fn command_for_use_global(input: &String) -> Result<()> {
    let groups = Groups::new()?;
    if groups.exists(input) {
        bail!("{} can only be used for projects", input)
    }

    let version = sanitize_version(input);
    if !node_strict_available(&version)? {
        bail!("Node@v{} has not been installed", &version);
    }

    let default_path = nvmd_home()?.default_path();
    write_all(default_path, &version)?;
    eprintln!("Now using node v{}", &version);

    Ok(())
}

/// nvmd use {version} --project
fn command_for_use_project(input: &String) -> Result<()> {
    let mut groups = Groups::new()?;
    let group = groups.find_by_name(input);
    let is_group = group.is_some();
    let version = match group {
        Some(g) => g.version.clone().ok_or_else(|| {
            anyhow!(
                "The Node.js version for group '{}' has not been set yet",
                input
            )
        })?,
        None => sanitize_version(input),
    };

    if !node_strict_available(&version)? {
        bail!("Node@v{} has not been installed", &version);
    }

    let project_path = env::current_dir()?;
    let project_path_str = project_path.to_str().unwrap();
    let project_name = project_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap();

    Projects::update_and_save(
        project_path_str,
        project_name,
        if is_group { input } else { &version },
    )?;
    if is_group {
        groups.update(&input, project_path_str);
        groups.save()?;
    }

    let nvmdrc = project_path.join(".nvmdrc");
    write_all(nvmdrc, &version)?;
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
    let mut path = Setting::global()?.get_directory()?.join(&version);
    if cfg!(unix) {
        path.push("bin");
    }
    if path.exists() {
        eprintln!("{:?}", path);
    } else {
        bail!("Node@v{} cannot be found", &version);
    }

    Ok(())
}
