use crate::module::{Context, Packages};
use crate::signal::pass_control_to_shim;
use crate::utils::command;
use crate::utils::help::{link_package, unlink_package};
use anyhow::Result;
use std::ffi::{OsStr, OsString};
use std::process::ExitStatus;

const ENABLE: &str = "enable";
const DISABLE: &str = "disable";
const INSTALL_DIRECTORY: &str = "--install-directory";

// corepack enable --install-directory /path/to/folder

// When run, this command will check whether the shims for the specified package
// managers can be found with the correct values inside the install directory. If
// not, or if they don't exist, they will be created.

// By default, it will locate the install directory by running the equivalent of
// `which corepack`, but this can be tweaked by explicitly passing the install
// directory via the `--install-directory` flag.

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus> {
    let path = Context::global()?.env_path()?;

    let mut command = command::create_command(exe);
    command.args(args);
    command.env("PATH", path);

    pass_control_to_shim();

    let status = command.status()?;

    let install_directory = args.iter().any(|a| a == INSTALL_DIRECTORY);
    if !install_directory {
        if args.iter().all(|a| a == ENABLE) {
            corepack_manager(args, true)?;
        }

        if args.iter().all(|a| a == DISABLE) {
            corepack_manager(args, false)?;
        }
    }

    Ok(status)
}

fn corepack_manager(args: &[OsString], enable: bool) -> Result<()> {
    let mut shims: Vec<&str> = args
        .iter()
        .filter_map(|name| {
            if is_supported_package(name) {
                name.to_str()
            } else {
                None
            }
        })
        .collect();

    if shims.is_empty() && !args.iter().any(|a| a == "npm") {
        shims.extend(["yarn", "pnpm"]);
    }

    let packages = Packages::new()?;
    for shim in shims {
        if enable {
            link_package(&shim)?;
            for extra in package_extra_aliases(shim) {
                link_package(extra)?;
            }
        } else if packages.can_be_removed(&shim) {
            unlink_package(&shim)?;
            for extra in package_extra_aliases(shim) {
                unlink_package(extra)?;
            }
        }
    }

    Ok(())
}

fn package_extra_aliases(name: &str) -> &'static [&'static str] {
    match name {
        "yarn" => &["yarnpkg"],
        "pnpm" => &["pnpx"],
        _ => &[],
    }
}

fn is_supported_package<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    match arg.as_ref().to_str() {
        Some(a) => a == "yarn" || a == "pnpm",
        None => false,
    }
}
