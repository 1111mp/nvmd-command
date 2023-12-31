use fs_extra::file::{read_to_string, write_all};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{from_str, json, Value};
use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Stdio,
    sync::Mutex,
};

use super::{ExitStatus, OsStr, OsString};
use crate::{
    command as CommandTool,
    common::{link_package, unlink_package, ENV_PATH, INSTALLTION_PATH, NVMD_PATH, VERSION},
};

lazy_static! {
    static ref INSTALL: OsString = OsString::from("install");
    static ref SHORT_INSTALL: OsString = OsString::from("i");
    static ref UNINSTALL: OsString = OsString::from("uninstall");
    static ref GLOBAL: OsString = OsString::from("--global");
    static ref SHORT_GLOBAL: OsString = OsString::from("-g");
    pub static ref UNINSTALL_PACKAGES_NAME: Mutex<Vec<String>> = {
        let m = Vec::new();
        Mutex::new(m)
    };
}

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus, String> {
    if ENV_PATH.is_empty() {
        return Err(String::from("command not found: ") + exe.to_str().unwrap());
    }

    let mut lib_path = INSTALLTION_PATH.clone();
    lib_path.push(VERSION.clone());
    if cfg!(unix) {
        // unix
        lib_path.push("bin");
    }
    lib_path.push(exe);

    if !lib_path.exists() {
        return Err(String::from("command not found: ") + exe.to_str().unwrap());
    }

    let is_global = args.contains(&SHORT_GLOBAL) || args.contains(&GLOBAL);
    let is_global_uninstall = is_global && args.contains(&UNINSTALL);

    // for npm uninstall -g packages
    // collection the bin names of packages
    if is_global_uninstall {
        collection_packages_name(args);
    }

    let child = CommandTool::create_command(exe)
        .env("PATH", ENV_PATH.clone())
        .args(args)
        .status();

    // npm install/uninstall -g packages

    match child {
        Ok(status) => {
            if status.success() {
                let is_global_install =
                    is_global && (args.contains(&INSTALL) || args.contains(&SHORT_INSTALL));
                // npm install -g packages
                if is_global_install {
                    global_install_packages(args);
                }

                // npm uninstall -g packages
                if is_global_uninstall {
                    global_uninstall_packages();
                }
            }

            Ok(status)
        }
        Err(_) => Err(String::from("failed to execute process")),
    }
}

fn global_install_packages(args: &[OsString]) {
    let re_str = "@[0-9]|@latest|@\"|@npm:";
    let re: Regex = Regex::new(re_str).unwrap();
    let packages = &args
        .into_iter()
        .filter(is_positional)
        .map(|name| {
            let package = name.to_str().unwrap();

            match re.find(&package) {
                None => OsString::from(&package),
                Some(mat) => OsString::from(&package[0..(mat.start())]),
            }
        })
        .collect();

    let npm_perfix = get_npm_perfix();

    let package_bin_names = get_package_bin_names(&npm_perfix, packages);

    if !package_bin_names.is_empty() {
        record_installed_package(&package_bin_names);

        for name in &package_bin_names {
            link_package(name);
        }
    }
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

fn record_installed_package(packages: &Vec<String>) {
    let mut packages_path = NVMD_PATH.clone();
    packages_path.push("packages.json");

    let update_packages = || {
        let mut json_obj = json!({});
        for package in packages {
            json_obj[package] = json!([*VERSION]);
        }
        let json_str = json_obj.to_string();
        write_all(&packages_path, &json_str).unwrap();
    };

    match read_to_string(&packages_path) {
        Err(_) => {
            // not exsit
            update_packages();
        }
        Ok(content) => {
            // exsit
            if content.is_empty() {
                update_packages();
            } else {
                let mut json_obj: Value = from_str(&content).unwrap();
                for package in packages {
                    if json_obj[package].is_null() {
                        json_obj[package] = json!([*VERSION]);
                    } else {
                        let versions = json_obj[package].as_array_mut().unwrap();
                        let version_value = json!(*VERSION);
                        if !versions.contains(&version_value) {
                            versions.push(version_value);
                        }
                    }
                }
                let json_str = json_obj.to_string();
                write_all(&packages_path, &json_str).unwrap();
            }
        }
    };
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

fn collection_packages_name(args: &[OsString]) {
    let re: Regex = Regex::new(r"@[0-9]|@latest").unwrap();
    let packages = &args
        .into_iter()
        .filter(is_positional)
        .map(|name| {
            let package = name.to_str().unwrap();

            match re.find(&package) {
                None => OsString::from(&package),
                Some(mat) => OsString::from(&package[0..(mat.start())]),
            }
        })
        .collect();

    let npm_perfix = get_npm_perfix();

    let package_bin_names = get_package_bin_names(&npm_perfix, packages);

    if !package_bin_names.is_empty() {
        let mut global_names = UNINSTALL_PACKAGES_NAME.lock().unwrap();
        for name in package_bin_names {
            global_names.push(name);
        }
    }
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

fn get_package_bin_names(npm_perfix: &String, packages: &Vec<OsString>) -> Vec<String> {
    let mut package_bin_names: Vec<String> = vec![];

    for package in packages {
        let mut package_json = PathBuf::from(npm_perfix);
        package_json.push(package);
        package_json.push("package.json");

        let json_str = match read_to_string(&package_json) {
            Err(_) => String::from(""),
            Ok(v) => v,
        };

        if !json_str.is_empty() {
            let json: Value = serde_json::from_str(&json_str).unwrap();
            let bin = &json["bin"];

            if bin.is_string() {
                let name = json["name"].as_str().unwrap();
                package_bin_names.push(String::from(name));
            } else {
                let keys = json["bin"].as_object().unwrap();
                for (key, _val) in keys {
                    package_bin_names.push(String::from(key));
                }
            }
        }
    }

    package_bin_names
}

fn is_flag<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    match arg.as_ref().to_str() {
        Some(a) => a.starts_with('-') || a == "install" || a == "i" || a == "uninstall",
        None => false,
    }
}

fn is_positional<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    !is_flag(arg)
}
