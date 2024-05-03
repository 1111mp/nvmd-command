use super::{anyhow, Result};
use super::{ExitStatus, OsStr, OsString};

use lazy_static::lazy_static;

use crate::{
    command as CommandTool,
    common::{link_package, package_can_be_removed, unlink_package, ENV_PATH},
};

lazy_static! {
    static ref ENABLE: OsString = OsString::from("enable");
    static ref DISABLE: OsString = OsString::from("disable");
    static ref INSTALL_DIRECTORY: OsString = OsString::from("--install-directory");
}

// corepack enable --install-directory /path/to/folder

// When run, this commmand will check whether the shims for the specified package
// managers can be found with the correct values inside the install directory. If
// not, or if they don't exist, they will be created.

// By default it will locate the install directory by running the equivalent of
// `which corepack`, but this can be tweaked by explicitly passing the install
// directory via the `--install-directory` flag.

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus> {
    if ENV_PATH.is_empty() {
        return Err(anyhow!("command not found: {:?}", exe));
    }

    let status = CommandTool::create_command(exe)
        .env("PATH", ENV_PATH.clone())
        .args(args)
        .status()?;

    let install_directory = args.contains(&INSTALL_DIRECTORY);
    // corepack enable ..
    // No special handling is required when using the "--install-directory" option
    if args.contains(&ENABLE) && !install_directory {
        corepack_enable(args);
    }

    // corepack disable ..
    // No special handling is required when using the "--install-directory" option
    if args.contains(&DISABLE) && !install_directory {
        corepack_disable(args);
    }

    Ok(status)
}

fn corepack_enable(args: &[OsString]) {
    let packages = &mut args
        .into_iter()
        .filter(is_positional)
        .map(|name| String::from(name.to_str().unwrap()))
        .collect::<Vec<String>>();

    if packages.is_empty() && !args.contains(&OsString::from("npm")) {
        packages.push(String::from("yarn"));
        packages.push(String::from("pnpm"));
    }

    for package in packages {
        link_package(package);
        if package == "yarn" {
            link_package(&String::from("yarnpkg"));
        }
        if package == "pnpm" {
            link_package(&String::from("pnpx"));
        }
    }
}

fn corepack_disable(args: &[OsString]) {
    let packages = &mut args
        .into_iter()
        .filter(is_positional)
        .map(|name| String::from(name.to_str().unwrap()))
        .collect::<Vec<String>>();

    if packages.is_empty() && !args.contains(&OsString::from("npm")) {
        packages.push(String::from("yarn"));
        packages.push(String::from("pnpm"));
    }

    for package in packages {
        if package_can_be_removed(package) {
            // need to remove executable file
            unlink_package(package);
            if package == "yarn" {
                unlink_package(&String::from("yarnpkg"));
            }
            if package == "pnpm" {
                unlink_package(&String::from("pnpx"));
            }
        }
    }
}

fn is_flag<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    match arg.as_ref().to_str() {
        Some(a) => a == "yarn" || a == "pnpm",
        None => false,
    }
}

fn is_positional<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    is_flag(arg)
}
