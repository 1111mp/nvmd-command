use super::{ExitStatus, OsStr, OsString};
use crate::module::{Context, PackageJson, Packages};
use crate::signal::pass_control_to_shim;
use crate::utils::command;
use crate::utils::help::{link_package, unlink_package};
use anyhow::{anyhow, bail, Context as AnyhowContext, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;
use std::sync::Mutex;
use std::{path::PathBuf, process::Stdio};

static UNINSTALL_PACKAGES_NAME: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
static SKIP_NEXT: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

/// Aliases that npm supports for the 'install' command
const NPM_INSTALL_ALIASES: [&str; 12] = [
    "i", "in", "ins", "inst", "insta", "instal", "install", "isnt", "isnta", "isntal", "isntall",
    "add",
];
/// Aliases that npm supports for the 'uninstall' command
const NPM_UNINSTALL_ALIASES: [&str; 5] = ["un", "uninstall", "remove", "rm", "r"];
/// Aliases that npm supports for the 'link' command
const NPM_LINK_ALIASES: [&str; 2] = ["link", "ln"];
/// Aliases that npm supports for the `update` command
const NPM_UPDATE_ALIASES: [&str; 4] = ["update", "udpate", "upgrade", "up"];

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus> {
    let path = Context::global()?.env_path()?;

    let mut positionals = args.iter().filter(is_positional).map(|arg| arg.as_os_str());
    let status = match positionals.next() {
        Some(cmd) if NPM_INSTALL_ALIASES.iter().any(|a| a == &cmd) => {
            let tools: Vec<_> = positionals.collect();
            command_install_executor(&path, exe, args, tools)
        }
        Some(cmd) if NPM_UNINSTALL_ALIASES.iter().any(|a| a == &cmd) => {
            let tools: Vec<_> = positionals.collect();
            command_uninstall_executor(&path, exe, args, tools)
        }
        Some(cmd) if cmd == "unlink" => {
            let tools: Vec<_> = positionals.collect();
            command_unlink_executor(&path, exe, args, tools)
        }
        Some(cmd) if NPM_LINK_ALIASES.iter().any(|a| a == &cmd) => {
            let tools: Vec<_> = positionals.collect();
            command_link_executor(&path, exe, args, tools)
        }
        Some(cmd) if NPM_UPDATE_ALIASES.iter().any(|a| a == &cmd) => {
            let tools: Vec<_> = positionals.collect();
            command_update_executor(&path, exe, args, tools)
        }
        _ => executor(&path, exe, args),
    }?;

    Ok(status)
}

fn executor<A>(path: &OsString, exe: &OsStr, args: &[A]) -> Result<ExitStatus>
where
    A: AsRef<OsStr>,
{
    let mut command = command::create_command(exe);
    command.args(args);
    command.env("PATH", path);

    pass_control_to_shim();

    let status = command.status()?;
    Ok(status)
}

/// npm install command
fn command_install_executor(
    path: &OsString,
    exe: &OsStr,
    args: &[OsString],
    tools: Vec<&OsStr>,
) -> Result<ExitStatus> {
    let status = executor(path, exe, args)?;
    if status.success() && is_global(args) {
        global_install_packages(path, tools)?;
    }
    Ok(status)
}

/// npm uninstall command
fn command_uninstall_executor(
    path: &OsString,
    exe: &OsStr,
    args: &[OsString],
    tools: Vec<&OsStr>,
) -> Result<ExitStatus> {
    let has_global = is_global(args);
    if has_global {
        collect_packages_before_uninstall(path, tools)?;
    }

    let status = executor(path, exe, args)?;
    if status.success() && has_global {
        global_uninstall_packages()?;
    }
    Ok(status)
}

/// npm unlink command
fn command_unlink_executor(
    path: &OsString,
    exe: &OsStr,
    args: &[OsString],
    tools: Vec<&OsStr>,
) -> Result<ExitStatus> {
    let has_global = is_global(args);
    if has_global {
        collection_packages_name_for_unlink(path, tools)?;
    }

    let status = executor(path, exe, args)?;

    if status.success() && has_global {
        global_uninstall_packages()?;
    }

    Ok(status)
}

/// npm link command
fn command_link_executor(
    path: &OsString,
    exe: &OsStr,
    args: &[OsString],
    tools: Vec<&OsStr>,
) -> Result<ExitStatus> {
    let status = executor(path, exe, args)?;

    if status.success() {
        global_link_packages(tools)?;
    }

    Ok(status)
}

/// npm update command
fn command_update_executor(
    path: &OsString,
    exe: &OsStr,
    args: &[OsString],
    tools: Vec<&OsStr>,
) -> Result<ExitStatus> {
    let status = executor(path, exe, args)?;

    if status.success() && is_global(args) {
        global_install_packages(path, tools)?;
    }

    Ok(status)
}

fn global_link_packages(tools: Vec<&OsStr>) -> Result<()> {
    let names = get_package_bin_names_for_link(tools)?;
    let version = Context::global()?.get_version().unwrap_or_default();
    if !names.is_empty() {
        let mut pkg = Packages::new()?;
        pkg.record_installed(&names, &version);
        pkg.save()?;

        for name in &names {
            link_package(name)?;
        }
    }

    Ok(())
}

fn global_install_packages(path: &OsString, tools: Vec<&OsStr>) -> Result<()> {
    // let re_str = "@[0-9]|@latest|@\"|@npm:";
    let reg: Regex = Regex::new("@[0-9]|@latest|@\"|@npm:").unwrap();
    let npm_prefix = get_npm_prefix(path)?;
    let version = Context::global()?.get_version().unwrap_or_default();
    let mut package_names = vec![];

    tools
        .into_iter()
        .map(|name| {
            let package = name.to_str().unwrap_or_default();

            match reg.find(&package) {
                None => OsStr::new(package),
                Some(mat) => OsStr::new(&package[0..(mat.start())]),
            }
        })
        .for_each(|pkg| {
            let names = PackageJson::new(&npm_prefix, pkg).bin_names();
            if !names.is_empty() {
                package_names.extend(names.iter().cloned());
                for name in names {
                    let _ = link_package(&name);
                }
            }
        });

    let mut pkg = Packages::new()?;
    pkg.record_installed(&package_names, &version);
    pkg.save()?;

    Ok(())
}

fn global_uninstall_packages() -> Result<()> {
    let names = UNINSTALL_PACKAGES_NAME.lock().unwrap();
    let version = Context::global()?.get_version().unwrap_or_default();
    let mut pkg = Packages::new()?;
    let need_remove_packages = pkg.record_uninstalled(&names, &version);
    pkg.save()?;

    for package in need_remove_packages {
        unlink_package(&package)?;
    }

    Ok(())
}

/// For unlink, should to find the current project name when without package argument.
fn collection_packages_name_for_unlink(path: &OsString, tools: Vec<&OsStr>) -> Result<()> {
    let npm_prefix = get_npm_prefix(path)?;
    let package_bin_names = get_package_bin_names_for_unlink(tools, &npm_prefix)?;
    if !package_bin_names.is_empty() {
        let mut global_names = UNINSTALL_PACKAGES_NAME.lock().unwrap();
        for name in package_bin_names {
            global_names.push(name);
        }
    }

    Ok(())
}

/// collect the bin name of the package before uninstalling it globally
fn collect_packages_before_uninstall(path: &OsString, tools: Vec<&OsStr>) -> Result<()> {
    let npm_perfix = get_npm_prefix(path)?;

    let packages = tools
        .into_iter()
        .flat_map(|tool| PackageJson::new(&npm_perfix, tool).bin_names())
        .collect::<Vec<String>>();

    if !packages.is_empty() {
        UNINSTALL_PACKAGES_NAME.lock().unwrap().extend(packages);
    }

    Ok(())
}

fn get_npm_prefix(path: &OsString) -> Result<PathBuf> {
    let mut command = command::create_command("npm");

    let output = command
        .args(["root", "-g"])
        .env("PATH", path)
        .stdout(Stdio::piped())
        .output()
        .with_context(|| anyhow!("No valid npm prefix found"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let pb = PathBuf::from(line.trim());
        if pb.is_dir() {
            return Ok(pb);
        }
    }

    bail!("No valid npm prefix found");
}

fn get_package_bin_names_for_unlink(
    tools: Vec<&OsStr>,
    npm_prefix: &PathBuf,
) -> Result<Vec<String>> {
    if tools.is_empty() {
        return Ok(PackageJson::from_current_dir()?.bin_names());
    }

    Ok(tools
        .into_iter()
        .flat_map(|tool| PackageJson::new(&npm_prefix, tool).bin_names())
        .collect::<Vec<String>>())
}

/// The link command supports execution from the current package directory without parameters.
/// It also supports execution using the relative directory of the package as a parameter.
/// Finally, the package name is passed in directly, similar to the "install" command.
/// https://docs.npmjs.com/cli/v9/commands/npm-link#description
fn get_package_bin_names_for_link(tools: Vec<&OsStr>) -> Result<Vec<String>> {
    // 'npm link'
    if tools.is_empty() {
        return Ok(PackageJson::from_current_dir()?.bin_names());
    }

    // 'npm link package' or 'npm link ../package'
    // We only need to deal with 'npm link ../package'
    let names: Vec<&OsStr> = tools
        .iter()
        .filter(is_relative_path)
        .map(|&path| path)
        .collect();

    if names.is_empty() {
        return Ok(vec![]);
    }

    Ok(names
        .into_iter()
        .map(|name| PackageJson::from_current_dir_with_pkg(name).map(|pkg| pkg.bin_names()))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect())
}

fn is_global<A>(args: &[A]) -> bool
where
    A: AsRef<OsStr>,
{
    args.iter()
        .fold(false, |global, arg| match arg.as_ref().to_str() {
            Some("-g") | Some("--global") => true,
            _ => global,
        })
}

fn is_relative_path<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    if let Some(arg_str) = arg.as_ref().to_str() {
        let path = Path::new(arg_str);
        path.is_relative() && (path.starts_with(".") || path.starts_with(".."))
    } else {
        false
    }
}

fn is_flag<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    match arg.as_ref().to_str() {
        Some(a) => a.starts_with('-'),
        None => false,
    }
}

/// https://docs.npmjs.com/cli/v7/commands/npm-install#workspace
/// https://docs.npmjs.com/cli/v7/commands/npm-link#workspace
/// We should filter out the arguments passed via '--workspace <name>'
/// It is a feature introduced in 'npm v7'
fn is_positional<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    let mut skip_next = SKIP_NEXT.lock().unwrap();

    if *skip_next {
        *skip_next = false;
        return false;
    }

    if let Some(arg_str) = arg.as_ref().to_str() {
        if arg_str == "--workspace" {
            *skip_next = true;
            return false;
        }
    }

    !is_flag(arg)
}
