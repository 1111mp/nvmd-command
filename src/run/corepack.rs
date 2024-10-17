use super::Result;
use super::{ExitStatus, OsStr, OsString};

use anyhow::bail;
use lazy_static::lazy_static;

use crate::{
    command as CommandTool,
    common::{link_package, package_can_be_removed, unlink_package, ENV_PATH, VERSION},
};

lazy_static! {
    static ref ENABLE: OsString = OsString::from("enable");
    static ref DISABLE: OsString = OsString::from("disable");
    static ref INSTALL_DIRECTORY: OsString = OsString::from("--install-directory");
}

// corepack enable --install-directory /path/to/folder

// When run, this command will check whether the shims for the specified package
// managers can be found with the correct values inside the install directory. If
// not, or if they don't exist, they will be created.

// By default, it will locate the install directory by running the equivalent of
// `which corepack`, but this can be tweaked by explicitly passing the install
// directory via the `--install-directory` flag.

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus> {
    let env_path = match ENV_PATH.as_ref() {
        Some(env_path) => env_path,
        None => {
            if VERSION.is_none() {
                bail!("the default node version is not set, you can set it by executing \"nvmd use {{version}}\"");
            }
            if let Some(version) = VERSION.as_ref() {
                bail!(
                    "version v{} is not installed, please install it before using",
                    version
                );
            }
            bail!("command not found: {:?}", exe);
        }
    };

    let status = CommandTool::create_command(exe)
        .env("PATH", env_path)
        .args(args)
        .status()?;

    let install_directory = args.contains(&*INSTALL_DIRECTORY);

    if args.contains(&*ENABLE) && !install_directory {
        corepack_manage(args, true)?;
    }

    if args.contains(&*DISABLE) && !install_directory {
        corepack_manage(args, false)?;
    }

    Ok(status)
}

fn corepack_manage(args: &[OsString], enable: bool) -> Result<()> {
    let mut packages: Vec<String> = args
        .iter()
        .filter_map(|name| {
            if is_package_name(name) {
                name.to_str().map(String::from)
            } else {
                None
            }
        })
        .collect();

    if packages.is_empty() && !args.contains(&OsString::from("npm")) {
        packages.push(String::from("yarn"));
        packages.push(String::from("pnpm"));
    }

    for package in packages {
        if enable {
            link_package(&package)?;
            if package == "yarn" {
                link_package("yarnpkg")?;
            }
            if package == "pnpm" {
                link_package("pnpx")?;
            }
        } else if package_can_be_removed(&package)? {
            unlink_package(&package)?;
            if package == "yarn" {
                unlink_package("yarnpkg")?;
            }
            if package == "pnpm" {
                unlink_package("pnpx")?;
            }
        }
    }

    Ok(())
}

fn is_package_name<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    match arg.as_ref().to_str() {
        Some(a) => a == "yarn" || a == "pnpm",
        None => false,
    }
}
