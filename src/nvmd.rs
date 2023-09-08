use std::{
    env::{self, ArgsOs},
    ffi::{OsStr, OsString},
    io::{BufRead, BufReader, Error, ErrorKind},
    os::unix::fs,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Mutex,
};

use fs_extra::file::{read_to_string, remove, write_all};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{from_str, json, Value};

use crate::command;

lazy_static! {
    pub static ref NVMD_PATH: PathBuf = get_nvmd_path();
    pub static ref VERSION: String = get_version();
    pub static ref ENV_PATH: OsString = get_env_path();
    pub static ref UNINSTALL_PACKAGES_NAME: Mutex<Vec<String>> = {
        let m = Vec::new();
        Mutex::new(m)
    };
}

/// Determine the name of the command to run by inspecting the first argument to the active process
pub fn get_tool_name(args: &mut ArgsOs) -> Result<OsString, Error> {
    args.next()
        .and_then(|arg0| Path::new(&arg0).file_name().map(tool_name_from_file_name))
        .ok_or_else(|| ErrorKind::InvalidInput.into())
}

#[cfg(unix)]
fn tool_name_from_file_name(file_name: &OsStr) -> OsString {
    file_name.to_os_string()
}

#[cfg(windows)]
fn tool_name_from_file_name(file_name: &OsStr) -> OsString {
    // On Windows PowerShell, the file name includes the .exe suffix,
    // and the Windows file system is case-insensitive
    // We need to remove that to get the raw tool name
    match file_name.to_str() {
        Some(file) => OsString::from(file.to_ascii_lowercase().trim_end_matches(".exe")),
        None => OsString::from(file_name),
    }
}

pub fn get_env_path() -> OsString {
    let bin_path = get_bin_path();

    let path = match env::var_os("PATH") {
        Some(path) => {
            let mut paths = env::split_paths(&path).collect::<Vec<_>>();

            paths.insert(0, PathBuf::from(bin_path));

            let env_path = match env::join_paths(paths) {
                Ok(p) => p,
                Err(_) => OsString::from(""),
            };

            return env_path;
        }
        None => bin_path,
    };

    return path;
}

fn get_bin_path() -> OsString {
    let mut nvmd_path = NVMD_PATH.clone();
    nvmd_path.push("versions");
    nvmd_path.push(VERSION.clone());

    if cfg!(unix) {
        nvmd_path.push("bin");
    }

    let bin_path = nvmd_path.into_os_string();

    return bin_path;
}

pub fn get_version() -> String {
    let mut nvmdrc = match env::current_dir() {
        Err(_) => PathBuf::from(""),
        Ok(dir) => dir,
    };
    nvmdrc.push(".nvmdrc");

    let project_version = match read_to_string(&nvmdrc) {
        Err(_) => String::from(""),
        Ok(v) => v,
    };

    if !project_version.is_empty() {
        return project_version;
    }

    let mut default_path = NVMD_PATH.clone();
    default_path.push("default");

    let default_version = match read_to_string(&default_path) {
        Err(_) => String::from(""),
        Ok(v) => v,
    };

    return default_version;
}

pub fn get_nvmd_path() -> PathBuf {
    let nvmd_path = match default_home_dir() {
        Ok(p) => p,
        Err(_) => PathBuf::from(""),
    };

    return nvmd_path;
}

pub fn install_packages(args: &Vec<OsString>) {
    let re: Regex = Regex::new(r"@[0-9]|@latest").unwrap();
    let packages = &args
        .into_iter()
        .filter(is_positional)
        .map(|x| {
            let package = String::from(x.to_str().unwrap());
            // let mat = re.find(&package).unwrap();

            let new_package = match re.find(&package) {
                None => OsString::from(&package),
                Some(mat) => {
                    let str = &package[0..(mat.start())];

                    return OsString::from(str);
                }
            };

            return OsString::from(new_package);
        })
        .collect();

    let npm_perfix = get_npm_perfix();
    println!("{}", npm_perfix);

    for package in packages {
        println!("{:?}", package);
    }

    let package_bin_names = get_package_bin_names(&npm_perfix, packages);

    record_installed_package(&package_bin_names);

    for name in &package_bin_names {
        println!("{}", name);

        if cfg!(unix) {
            let mut source = NVMD_PATH.clone();
            source.push("bin");
            source.push("nvmd");
            let mut alias = NVMD_PATH.clone();
            alias.push("bin");
            alias.push(name);

            fs::symlink(source, alias).unwrap_or_else(|why| {
                println!("! {:?}", why.kind());
            })
        }
    }
}

pub fn uninstall_packages() {
    let names = UNINSTALL_PACKAGES_NAME.lock().unwrap();
    for name in names.iter() {
        println!("{}", name);
        if record_uninstall_package(name) {
            // need to remove executable file
            let mut alias = NVMD_PATH.clone();
            alias.push("bin");
            alias.push(name);

            if cfg!(unix) {
                remove(alias).unwrap_or_else(|why| {
                    println!("! {:?}", why.kind);
                });
            }
        }
    }
}

pub fn collection_packages_name(args: &Vec<OsString>) {
    let re: Regex = Regex::new(r"@[0-9]|@latest").unwrap();
    let packages = &args
        .into_iter()
        .filter(is_positional)
        .map(|x| {
            let package = String::from(x.to_str().unwrap());
            // let mat = re.find(&package).unwrap();

            let new_package = match re.find(&package) {
                None => OsString::from(&package),
                Some(mat) => {
                    let str = &package[0..(mat.start())];

                    return OsString::from(str);
                }
            };

            return OsString::from(new_package);
        })
        .collect();

    let npm_perfix = get_npm_perfix();
    println!("{}", npm_perfix);

    for package in packages {
        println!("{:?}", package);
    }

    let package_bin_names = get_package_bin_names(&npm_perfix, packages);

    let mut global_names = UNINSTALL_PACKAGES_NAME.lock().unwrap();
    for name in package_bin_names {
        global_names.push(name);
    }

    // return package_bin_names;
}

fn record_installed_package(packages: &Vec<String>) {
    println!("record_installed_package");
    let mut packages_path = NVMD_PATH.clone();
    packages_path.push("packages.json");

    match read_to_string(&packages_path) {
        Err(_) => {
            // not exsit
            let mut json_obj = json!({});
            for package in packages {
                json_obj[package] = json!([*VERSION]);
            }
            let json_str = json_obj.to_string();
            println!("{}", json_str);
            write_all(packages_path, &json_str).unwrap();
        }
        Ok(content) => {
            // exsit
            if content.is_empty() {
                let mut json_obj = json!({});
                for package in packages {
                    json_obj[package] = json!([*VERSION]);
                }
                let json_str = json_obj.to_string();
                write_all(packages_path, &json_str).unwrap();
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
                println!("{}", json_str);
                write_all(packages_path, &json_str).unwrap();
            }
        }
    };
}

fn record_uninstall_package(name: &String) -> bool {
    let mut packages_path = NVMD_PATH.clone();
    packages_path.push("packages.json");

    let record = match read_to_string(&packages_path) {
        Err(_) => true,
        Ok(content) => {
            if content.is_empty() {
                return true;
            }

            println!("111");

            let mut json_obj: Value = from_str(&content).unwrap();

            if json_obj.is_null() || !json_obj.is_object() {
                return true;
            }

            if json_obj[name].is_null() || !json_obj[name].is_array() {
                return true;
            }

            let versions = json_obj[name].as_array_mut().unwrap();

            for v in versions.clone() {
                println!("v: {}", v.as_str().unwrap());
            }

            println!("{}", versions.is_empty());

            if versions.is_empty() {
                return true;
            }

            println!("{}", *VERSION);
            println!("{}", *VERSION);

            let target = json!(*VERSION);

            println!("{}", versions.contains(&target));
            if !versions.contains(&target) {
                return false;
            }

            println!("333");

            versions.retain(|x| x.as_str().unwrap() != target.as_str().unwrap());

            for v in versions.clone() {
                println!("v: {}", v.as_str().unwrap());
            }

            println!("444");

            let mut ret: bool = false;
            if versions.is_empty() {
                ret = true;
            }
            println!("555");

            let json_str = json_obj.to_string();
            println!("{}", json_str);
            write_all(packages_path, &json_str).unwrap();

            println!("666");

            return ret;
        }
    };

    println!("{}", record);

    return record;
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
                let name = json["bin"].as_str().unwrap();
                package_bin_names.push(String::from(name));
            } else {
                let keys = json["bin"].as_object().unwrap();
                for (key, _val) in keys {
                    package_bin_names.push(String::from(key));
                }
            }
        }
    }

    return package_bin_names;
}

fn get_npm_perfix() -> String {
    let mut command = command::create_command("npm");

    let mut env_path = NVMD_PATH.clone();
    env_path.push("versions");
    env_path.push(VERSION.clone());
    if cfg!(unix) {
        env_path.push("bin");
    }

    let child = command
        .env("PATH", env_path.into_os_string())
        .args(["root", "-g"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("get npm perfix error");

    let output = child.stdout.unwrap();
    let lines = BufReader::new(output).lines().enumerate().take(1);
    let mut perfix = String::from("");
    for (_counter, line) in lines {
        perfix = line.unwrap();
    }

    return perfix;
}

fn default_home_dir() -> Result<PathBuf, ErrorKind> {
    let mut home = dirs::home_dir().ok_or(ErrorKind::NotFound)?;
    home.push(".nvmd");
    Ok(home)
}

fn is_flag<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    match arg.as_ref().to_str() {
        Some(a) => a.starts_with('-') || a == "install" || a == "uninstall",
        None => false,
    }
}

pub fn is_positional<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    !is_flag(arg)
}
