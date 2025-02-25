use super::{anyhow, Result};
use super::{ExitStatus, OsStr, OsString};

use crate::signal::pass_control_to_shim;
use crate::utils::help::link_package;
use crate::utils::package::{
    collect_package_bin_names, collect_package_bin_names_for_link,
    collect_package_bin_names_from_curdir, record_installed_package_info,
    record_uninstall_package_info,
};
use crate::{
    command as CommandTool,
    common::{ENV_PATH, INSTALLTION_DIRECTORY, VERSION},
};

use anyhow::bail;
use lazy_static::lazy_static;
use regex::Regex;
use std::path::Path;
use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Stdio,
    sync::Mutex,
};

lazy_static! {
    pub static ref UNINSTALL_PACKAGES_NAME: Mutex<Vec<String>> = {
        let m = Vec::new();
        Mutex::new(m)
    };
    static ref SKIP_NEXT: Mutex<bool> = Mutex::new(false);
}

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
    match ENV_PATH.as_ref() {
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
    let lib_path = INSTALLTION_DIRECTORY.clone().and_then(|mut path| {
        VERSION.clone().map(|version| {
            path.push(version);
            if cfg!(unix) {
                path.push("bin");
            }
            path.push(exe);
            path
        })
    });
    // Check if the path exists and return an error if it doesn't
    match lib_path {
        Some(ref path) if path.exists() => path,
        _ => return Err(anyhow!("command not found: {:?}", exe)),
    };

    let mut positionals = args.iter().filter(is_positional).map(|arg| arg.as_os_str());
    let status = match positionals.next() {
        Some(cmd) if NPM_INSTALL_ALIASES.iter().any(|a| a == &cmd) => {
            let tools: Vec<_> = positionals.collect();
            command_install_executor(exe, args, tools)
        }
        Some(cmd) if NPM_UNINSTALL_ALIASES.iter().any(|a| a == &cmd) => {
            let tools: Vec<_> = positionals.collect();
            command_uninstall_executor(exe, args, tools)
        }
        Some(cmd) if cmd == "unlink" => {
            let tools: Vec<_> = positionals.collect();
            command_unlink_executor(exe, args, tools)
        }
        Some(cmd) if NPM_LINK_ALIASES.iter().any(|a| a == &cmd) => {
            let tools: Vec<_> = positionals.collect();
            command_link_executor(exe, args, tools)
        }
        Some(cmd) if NPM_UPDATE_ALIASES.iter().any(|a| a == &cmd) => {
            let tools: Vec<_> = positionals.collect();
            command_update_executor(exe, args, tools)
        }
        _ => executor(exe, args),
    }?;

    Ok(status)
}

/// npm instll command
fn command_install_executor(
    exe: &OsStr,
    args: &[OsString],
    tools: Vec<&OsStr>,
) -> Result<ExitStatus> {
    let status = executor(exe, args)?;
    if status.success() && command_has_global(args) {
        global_install_packages(tools)?;
    }
    Ok(status)
}

/// npm uninstll command
fn command_uninstall_executor(
    exe: &OsStr,
    args: &[OsString],
    tools: Vec<&OsStr>,
) -> Result<ExitStatus> {
    let has_global = command_has_global(args);
    if has_global {
        collect_packages_names_before_uninstall(tools)?;
    }

    let status = executor(exe, args)?;
    if status.success() && has_global {
        global_uninstall_packages()?;
    }
    Ok(status)
}

/// npm unlink command
fn command_unlink_executor(
    exe: &OsStr,
    args: &[OsString],
    tools: Vec<&OsStr>,
) -> Result<ExitStatus> {
    let has_global = command_has_global(args);
    if has_global {
        collection_packages_name_for_unlink(tools)?;
    }

    let status = executor(exe, args)?;

    if status.success() && has_global {
        global_uninstall_packages()?;
    }

    Ok(status)
}

/// npm link command
fn command_link_executor(exe: &OsStr, args: &[OsString], tools: Vec<&OsStr>) -> Result<ExitStatus> {
    let status = executor(exe, args)?;

    if status.success() {
        global_link_packages(tools)?;
    }

    Ok(status)
}

/// npm update command
fn command_update_executor(
    exe: &OsStr,
    args: &[OsString],
    tools: Vec<&OsStr>,
) -> Result<ExitStatus> {
    let status = executor(exe, args)?;

    if status.success() && command_has_global(args) {
        global_install_packages(tools)?;
    }

    Ok(status)
}

fn global_link_packages(tools: Vec<&OsStr>) -> Result<()> {
    let package_bin_names = get_package_bin_names_for_link(tools)?;
    if !package_bin_names.is_empty() {
        record_installed_package_info(&package_bin_names)?;
        for name in &package_bin_names {
            link_package(name)?;
        }
    }

    Ok(())
}

fn global_install_packages(tools: Vec<&OsStr>) -> Result<()> {
    let re_str = "@[0-9]|@latest|@\"|@npm:";
    let re: Regex = Regex::new(re_str).unwrap();
    let packages = &tools
        .iter()
        .map(|name| {
            let package = name.to_str().unwrap();

            match re.find(&package) {
                None => OsStr::new(package),
                Some(mat) => OsStr::new(&package[0..(mat.start())]),
            }
        })
        .collect();

    let npm_perfix = get_npm_perfix();
    let package_bin_names = collect_package_bin_names(&npm_perfix, packages)?;
    if !package_bin_names.is_empty() {
        record_installed_package_info(&package_bin_names)?;
        for name in &package_bin_names {
            link_package(name)?;
        }
    }

    Ok(())
}

fn global_uninstall_packages() -> Result<()> {
    let names = UNINSTALL_PACKAGES_NAME.lock().unwrap();
    record_uninstall_package_info(&names)?;
    Ok(())
}

/// For unlink, should to find the current project name when without package argument.
fn collection_packages_name_for_unlink(tools: Vec<&OsStr>) -> Result<()> {
    let npm_perfix = get_npm_perfix();
    let package_bin_names = get_package_bin_names_for_unlink(tools, &npm_perfix)?;
    if !package_bin_names.is_empty() {
        let mut global_names = UNINSTALL_PACKAGES_NAME.lock().unwrap();
        for name in package_bin_names {
            global_names.push(name);
        }
    }

    Ok(())
}

/// collect the bin name of the package before uninstalling it globally
fn collect_packages_names_before_uninstall(tools: Vec<&OsStr>) -> Result<()> {
    let npm_perfix = get_npm_perfix();
    let package_bin_names = collect_package_bin_names(&npm_perfix, &tools)?;
    if !package_bin_names.is_empty() {
        let mut global_names = UNINSTALL_PACKAGES_NAME.lock().unwrap();
        for name in package_bin_names {
            global_names.push(name);
        }
    }

    Ok(())
}

fn get_npm_perfix() -> String {
    let mut command = CommandTool::create_command("npm");

    let child = command
        .env("PATH", ENV_PATH.clone().unwrap())
        .args(["root", "-g"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("nvmd-desktop: get npm perfix error");

    let output = child.stdout.unwrap();
    let lines = BufReader::new(output).lines();
    let mut perfix = String::from("");
    for line in lines {
        let cur_line = line.unwrap();
        if PathBuf::from(&cur_line).is_dir() {
            perfix = cur_line;
        }
    }

    perfix
}

fn get_package_bin_names_for_unlink(
    tools: Vec<&OsStr>,
    npm_perfix: &String,
) -> Result<Vec<String>> {
    if tools.is_empty() {
        return collect_package_bin_names_from_curdir();
    }
    collect_package_bin_names(npm_perfix, &tools)
}

/// The link command supports execution from the current package directory without parameters.
/// It also supports execution using the relative directory of the package as a parameter.
/// Finally, the package name is passed in directly, similar to the "install" command.
/// https://docs.npmjs.com/cli/v9/commands/npm-link#description
fn get_package_bin_names_for_link(tools: Vec<&OsStr>) -> Result<Vec<String>> {
    // 'npm link'
    if tools.is_empty() {
        return collect_package_bin_names_from_curdir();
    }

    // 'npm link package' or 'npm link ../package'
    // We only need to deal with 'npm link ../package'
    let packages: Vec<&OsStr> = tools
        .iter()
        .filter(is_relative_path)
        .map(|&path| path)
        .collect();

    if packages.is_empty() {
        return Ok(vec![]);
    }

    collect_package_bin_names_for_link(&packages)
}

fn executor<A>(exe: &OsStr, args: &[A]) -> Result<ExitStatus>
where
    A: AsRef<OsStr>,
{
    let mut command = CommandTool::create_command(exe);
    command.env("PATH", ENV_PATH.clone().unwrap()).args(args);

    pass_control_to_shim();

    let status = command.status()?;
    Ok(status)
}

fn command_has_global<A>(args: &[A]) -> bool
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
