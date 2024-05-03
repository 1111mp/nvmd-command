use super::{anyhow, Result};
use super::{ExitStatus, OsStr, OsString};

use fs_extra::file::{read_to_string, write_all};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{from_str, json, Value};
use std::{
    env,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Stdio,
    sync::Mutex,
};

use crate::{
    command as CommandTool,
    common::{link_package, unlink_package, ENV_PATH, INSTALLTION_PATH, NVMD_PATH, VERSION},
};

lazy_static! {
    pub static ref UNINSTALL_PACKAGES_NAME: Mutex<Vec<String>> = {
        let m = Vec::new();
        Mutex::new(m)
    };
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
    if ENV_PATH.is_empty() {
        return Err(anyhow!("command not found: {:?}", exe));
    }

    let mut lib_path = INSTALLTION_PATH.clone();
    lib_path.push(VERSION.clone());
    if cfg!(unix) {
        // unix
        lib_path.push("bin");
    }
    lib_path.push(exe);

    if !lib_path.exists() {
        return Err(anyhow!("command not found: {:?}", exe));
    }

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
        collection_packages_name(tools)?;
    }

    let status = executor(exe, args)?;

    if status.success() && has_global {
        global_uninstall_packages();
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
        global_uninstall_packages();
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

fn executor<A>(exe: &OsStr, args: &[A]) -> Result<ExitStatus>
where
    A: AsRef<OsStr>,
{
    Ok(CommandTool::create_command(exe)
        .env("PATH", ENV_PATH.clone())
        .args(args)
        .status()?)
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

fn global_link_packages(tools: Vec<&OsStr>) -> Result<()> {
    let package_bin_names = get_package_bin_names_for_link(tools)?;

    if !package_bin_names.is_empty() {
        record_installed_package(&package_bin_names)?;

        for name in &package_bin_names {
            link_package(name);
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
    let package_bin_names = get_package_bin_names(&npm_perfix, packages)?;

    if !package_bin_names.is_empty() {
        record_installed_package(&package_bin_names)?;

        for name in &package_bin_names {
            link_package(name);
        }
    }

    Ok(())
}

fn global_uninstall_packages() {
    let names = UNINSTALL_PACKAGES_NAME.lock().unwrap();
    for name in names.iter() {
        if record_uninstall_package(name) {
            // need to remove executable file
            unlink_package(name);
        }
    }
}

fn record_installed_package(packages: &Vec<String>) -> Result<()> {
    let mut packages_path = NVMD_PATH.clone();
    packages_path.push("packages.json");

    let update_packages = || -> Result<()> {
        let mut json_obj = json!({});
        for package in packages {
            json_obj[package] = json!([*VERSION]);
        }
        let json_str = json_obj.to_string();
        write_all(&packages_path, &json_str)?;
        Ok(())
    };

    match read_to_string(&packages_path) {
        Err(_) => {
            // not exsit
            update_packages()?;
        }
        Ok(content) => {
            // exsit
            if content.is_empty() {
                update_packages()?;
            } else {
                let mut json_obj: Value = from_str(&content)?;
                for package in packages {
                    if json_obj[package].is_null() {
                        json_obj[package] = json!([*VERSION]);
                    } else {
                        if let Some(versions) = json_obj[package].as_array_mut() {
                            let version_value = json!(*VERSION);
                            if !versions.contains(&version_value) {
                                versions.push(version_value);
                            }
                        }
                    }
                }
                let json_str = json_obj.to_string();
                write_all(&packages_path, &json_str)?;
            }
        }
    };

    Ok(())
}

fn record_uninstall_package(name: &String) -> bool {
    let mut packages_path = NVMD_PATH.clone();
    packages_path.push("packages.json");

    match read_to_string(&packages_path) {
        Err(_) => true,
        Ok(content) => {
            if content.is_empty() {
                return true;
            }

            let mut json_obj: Value = from_str(&content).unwrap();

            if json_obj.is_null() || !json_obj.is_object() {
                return true;
            }

            if json_obj[name].is_null() || !json_obj[name].is_array() {
                return true;
            }

            let versions = json_obj[name].as_array_mut().unwrap();

            if versions.is_empty() {
                return true;
            }

            let target = json!(*VERSION);

            if !versions.contains(&target) {
                return false;
            }

            versions.retain(|x| x.as_str().unwrap() != target.as_str().unwrap());

            let mut ret: bool = false;
            if versions.is_empty() {
                ret = true;
            }

            let json_str = json_obj.to_string();
            write_all(packages_path, &json_str).unwrap();

            return ret;
        }
    }
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

fn collection_packages_name(tools: Vec<&OsStr>) -> Result<()> {
    let npm_perfix = get_npm_perfix();

    let package_bin_names = get_package_bin_names(&npm_perfix, &tools)?;

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
        .env("PATH", ENV_PATH.clone())
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
        return Ok(get_package_bin_names_from_curdir()?);
    }

    get_package_bin_names(npm_perfix, &tools)
}

/// The link command supports execution from the current package directory without parameters.
/// It also supports execution using the relative directory of the package as a parameter.
/// Finally, the package name is passed in directly, similar to the "install" command.
fn get_package_bin_names_for_link(tools: Vec<&OsStr>) -> Result<Vec<String>> {
    if tools.is_empty() {
        return Ok(get_package_bin_names_from_curdir()?);
    }

    let re_str = "@[0-9]|@latest|@\"|@npm:";
    let re: Regex = Regex::new(re_str).unwrap();
    let npm_perfix = get_npm_perfix();
    let packages = &tools
        .iter()
        .map(|name| {
            if PathBuf::from(name).is_relative() {
                return *name;
            }

            let package = name.to_str().unwrap();
            match re.find(&package) {
                None => OsStr::new(package),
                Some(mat) => OsStr::new(&package[0..(mat.start())]),
            }
        })
        .collect();

    bin_names_for_link(&npm_perfix, &packages)
}

fn bin_names_for_link(npm_perfix: &String, packages: &Vec<&OsStr>) -> Result<Vec<String>> {
    let mut package_bin_names: Vec<String> = vec![];
    for package in packages {
        let mut package_json = if PathBuf::from(package).is_relative() {
            let mut cur_dir = env::current_dir()?;
            cur_dir.push(package);
            cur_dir.canonicalize()?
        } else {
            let mut path = PathBuf::from(npm_perfix);
            path.push(package);
            path
        };
        package_json.push("package.json");

        let json_str = read_to_string(&package_json)?;

        if json_str.is_empty() {
            continue;
        }

        let json: Value = serde_json::from_str(&json_str)?;
        let bin = &json["bin"];

        if bin.is_null() {
            continue;
        }

        if bin.is_string() {
            if let Some(name) = json["name"].as_str() {
                package_bin_names.push(name.to_owned());
            }
        } else {
            if let Some(keys) = json["bin"].as_object() {
                for (key, _val) in keys {
                    package_bin_names.push(key.to_string());
                }
            }
        }
    }

    Ok(package_bin_names)
}

/// find package bin namesc from current dir for npm link/unlink command
fn get_package_bin_names_from_curdir() -> Result<Vec<String>> {
    let mut package_bin_names: Vec<String> = vec![];

    let mut package_json = env::current_dir()?;
    package_json.push("package.json");

    let json_str = read_to_string(&package_json)?;

    if json_str.is_empty() {
        return Ok(package_bin_names);
    }

    let json: Value = serde_json::from_str(&json_str)?;
    let bin = &json["bin"];

    if bin.is_null() {
        return Ok(package_bin_names);
    }

    if bin.is_string() {
        if let Some(name) = json["name"].as_str() {
            package_bin_names.push(name.to_owned());
        }
    } else {
        if let Some(bin) = json["bin"].as_object() {
            for (key, _val) in bin {
                package_bin_names.push(key.to_string());
            }
        }
    }

    Ok(package_bin_names)
}

fn get_package_bin_names(npm_perfix: &String, packages: &Vec<&OsStr>) -> Result<Vec<String>> {
    let mut package_bin_names: Vec<String> = vec![];

    for package in packages {
        let mut package_json = PathBuf::from(npm_perfix);
        package_json.push(package);
        package_json.push("package.json");

        let json_str = read_to_string(&package_json)?;

        if json_str.is_empty() {
            continue;
        }

        let json: Value = serde_json::from_str(&json_str)?;
        let bin = &json["bin"];

        if bin.is_null() {
            continue;
        }

        if bin.is_string() {
            if let Some(name) = json["name"].as_str() {
                package_bin_names.push(name.to_owned());
            }
        } else {
            if let Some(bin) = json["bin"].as_object() {
                for (key, _val) in bin {
                    package_bin_names.push(key.to_string());
                }
            }
        }
    }

    Ok(package_bin_names)
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

fn is_positional<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    !is_flag(arg)
}
