use std::{
    env::{self, ArgsOs},
    ffi::{OsStr, OsString},
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

use cfg_if::cfg_if;
use fs_extra::file::read_to_string;

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
    let mut nvmd_path = match default_home_dir() {
        Ok(p) => p,
        Err(_) => PathBuf::from(""),
    };

    let mut nvmdrc = match env::current_dir() {
        Ok(c) => c,
        Err(_) => PathBuf::from(""),
    };
    nvmdrc.push(".nvmdrc");

    let project_version = match read_to_string(&nvmdrc) {
        Err(_) => String::from(""),
        Ok(v) => v,
    };

    if !project_version.is_empty() {
        nvmd_path.push("versions");
        nvmd_path.push(project_version);
        if cfg!(unix) {
            nvmd_path.push("bin");
        }

        let bin_path = nvmd_path.into_os_string();

        return bin_path;
    }

    // let mut default_path = PathBuf::from(nvmd_path);
    nvmd_path.push("default");

    let default_version = match read_to_string(&nvmd_path) {
        Err(_) => String::from(""),
        Ok(v) => v,
    };

    nvmd_path.pop();
    nvmd_path.push("versions");
    nvmd_path.push(default_version);

    if cfg!(unix) {
        nvmd_path.push("bin");
    }

    let bin_path = nvmd_path.into_os_string();

    return bin_path;
}

fn get_version(nvmd_path: &PathBuf) -> Result<String, Error> {
    let mut nvmdrc = env::current_dir()?;
    nvmdrc.push(".nvmdrc");

    let project_version = match read_to_string(&nvmdrc) {
        Err(_) => String::from(""),
        Ok(v) => v,
    };

    if !project_version.is_empty() {
        return Ok(project_version);
    }

    let mut default_path = PathBuf::from(nvmd_path);
    default_path.push("default");

    let default_version = match read_to_string(&default_path) {
        Err(_) => String::from(""),
        Ok(v) => v,
    };

    Ok(default_version)
}

fn default_home_dir() -> Result<PathBuf, ErrorKind> {
    let mut home = dirs::home_dir().ok_or(ErrorKind::NotFound)?;
    home.push(".nvmd");
    Ok(home)
}
