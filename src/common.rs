#[cfg(windows)]
use fs_extra::file::{copy, CopyOptions};
use fs_extra::file::{read_to_string, remove};
use lazy_static::lazy_static;
use serde_json::{from_str, json, Value};
#[cfg(unix)]
use std::os::unix::fs;
use std::{env, ffi::OsString, io::ErrorKind, path::PathBuf};

lazy_static! {
    pub static ref NVMD_PATH: PathBuf = get_nvmd_path();
    pub static ref VERSION: String = get_version();
    pub static ref DEFAULT_INSTALLTION_PATH: PathBuf = get_default_installtion_path();
    pub static ref INSTALLTION_PATH: PathBuf = get_installtion_path();
    pub static ref ENV_PATH: OsString = get_env_path();
}

fn get_env_path() -> OsString {
    if VERSION.is_empty() {
        return OsString::from("");
    }

    let bin_path = get_bin_path();

    if !PathBuf::from(&bin_path).exists() {
        return OsString::from("");
    }

    match env::var_os("PATH") {
        Some(path) => {
            let mut paths = env::split_paths(&path).collect::<Vec<_>>();
            paths.insert(0, PathBuf::from(bin_path));

            match env::join_paths(paths) {
                Ok(p) => p,
                Err(_) => OsString::from(""),
            }
        }
        None => bin_path,
    }
}

fn get_bin_path() -> OsString {
    let mut nvmd_path = INSTALLTION_PATH.clone();
    nvmd_path.push(VERSION.clone());

    if cfg!(unix) {
        nvmd_path.push("bin");
    }

    nvmd_path.into_os_string()
}

// $HOME/.nvmd/setting.json -> directory
fn get_installtion_path() -> PathBuf {
    let mut setting_path = NVMD_PATH.clone();
    setting_path.push("setting.json");

    let setting_content = match read_to_string(&setting_path) {
        Err(_) => String::from(""),
        Ok(content) => content,
    };

    if setting_content.is_empty() {
        return DEFAULT_INSTALLTION_PATH.clone();
    }

    let json_obj: Value = from_str(&setting_content).unwrap();

    if json_obj.is_null() || !json_obj.is_object() {
        return DEFAULT_INSTALLTION_PATH.clone();
    }

    if json_obj["directory"].is_null() || !json_obj["directory"].is_string() {
        return DEFAULT_INSTALLTION_PATH.clone();
    }

    let directory = json_obj["directory"].as_str().unwrap();

    PathBuf::from(directory)
}

fn get_default_installtion_path() -> PathBuf {
    let mut default_path = NVMD_PATH.clone();
    default_path.push("versions");

    default_path
}

fn get_version() -> String {
    let nvmdrc = find_nvmdrc();
    let project_version = match read_to_string(&nvmdrc) {
        Err(_) => String::from(""),
        Ok(v) => v,
    };

    if !project_version.is_empty() {
        return project_version;
    }

    let mut default_path = NVMD_PATH.clone();
    default_path.push("default");

    match read_to_string(&default_path) {
        Err(_) => String::from(""),
        Ok(v) => v,
    }
}

fn find_nvmdrc() -> PathBuf {
    let mut current_dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(_) => PathBuf::from(""),
    };
    current_dir.push(".nvmdrc");

    while current_dir.pop() {
        let mut nvmdrc = current_dir.clone();
        nvmdrc.push(".nvmdrc");
        if nvmdrc.is_file() {
            return nvmdrc;
        }
    }

    PathBuf::from("")
}

fn get_nvmd_path() -> PathBuf {
    match default_home_dir() {
        Ok(p) => p,
        Err(_) => PathBuf::from(""),
    }
}

pub fn package_can_be_removed(name: &String) -> bool {
    let mut packages_path = NVMD_PATH.clone();
    packages_path.push("packages.json");

    match read_to_string(&packages_path) {
        Err(_) => true,
        Ok(content) => {
            if content.is_empty() {
                return true;
            }

            let json_obj: Value = from_str(&content).unwrap();

            if json_obj.is_null() || !json_obj.is_object() {
                return true;
            }

            if json_obj[name].is_null() || !json_obj[name].is_array() {
                return true;
            }

            let versions = json_obj[name].as_array().unwrap();

            if versions.is_empty() {
                return true;
            }

            let target = json!(*VERSION);

            if versions.len() == 1 && versions.contains(&target) {
                return true;
            }

            false
        }
    }
}

#[cfg(unix)]
pub fn link_package(name: &String) {
    let mut source = NVMD_PATH.clone();
    source.push("bin");
    source.push("nvmd");
    let mut alias = NVMD_PATH.clone();
    alias.push("bin");
    alias.push(name);

    fs::symlink(source, alias).unwrap_or_else(|_why| {})
}

#[cfg(windows)]
pub fn link_package(name: &String) {
    // from
    let mut exe_source = NVMD_PATH.clone();
    exe_source.push("bin");
    exe_source.push("nvmd.exe");
    let mut cmd_source = NVMD_PATH.clone();
    cmd_source.push("bin");
    cmd_source.push("npm.cmd");

    // to
    let mut exe_alias = NVMD_PATH.clone();
    exe_alias.push("bin");
    let exe = name.clone() + ".exe";
    exe_alias.push(exe);
    let mut cmd_alias = NVMD_PATH.clone();
    cmd_alias.push("bin");
    let cmd = name.clone() + ".cmd";
    cmd_alias.push(cmd);

    let mut options = CopyOptions::new(); //Initialize default values for CopyOptions
    options.skip_exist = true; // Skip existing files if true (default: false).
    copy(&exe_source, &exe_alias, &options).unwrap();
    copy(&cmd_source, &cmd_alias, &options).unwrap();
}

#[cfg(unix)]
pub fn unlink_package(name: &String) {
    let mut alias = NVMD_PATH.clone();
    alias.push("bin");
    alias.push(name);

    remove(alias).unwrap_or_else(|_why| {});
}

#[cfg(windows)]
pub fn unlink_package(name: &String) {
    let mut exe_alias = NVMD_PATH.clone();
    exe_alias.push("bin");
    let exe = name.clone() + ".exe";
    exe_alias.push(exe);
    let mut cmd_alias = NVMD_PATH.clone();
    cmd_alias.push("bin");
    let cmd = name.clone() + ".cmd";
    cmd_alias.push(cmd);

    remove(exe_alias).unwrap_or_else(|_why| {});
    remove(cmd_alias).unwrap_or_else(|_why| {});
}

fn default_home_dir() -> Result<PathBuf, ErrorKind> {
    let mut home = dirs::home_dir().ok_or(ErrorKind::NotFound)?;
    home.push(".nvmd");
    Ok(home)
}
