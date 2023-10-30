use std::{env, ffi::OsString, io::ErrorKind, path::PathBuf, process::ExitStatus};

use fs_extra::file::read_to_string;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref NVMD_PATH: PathBuf = get_nvmd_path();
    pub static ref VERSION: String = get_version();
    pub static ref ENV_PATH: OsString = get_env_path();
}

fn get_env_path() -> OsString {
    if VERSION.is_empty() {
        return OsString::from("");
    }

    let bin_path = get_bin_path();

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
    let mut nvmd_path = NVMD_PATH.clone();
    nvmd_path.push("versions");
    nvmd_path.push(VERSION.clone());

    if cfg!(unix) {
        nvmd_path.push("bin");
    }

    nvmd_path.into_os_string()
}

fn get_version() -> String {
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

    match read_to_string(&default_path) {
        Err(_) => String::from(""),
        Ok(v) => v,
    }
}

fn get_nvmd_path() -> PathBuf {
    match default_home_dir() {
        Ok(p) => p,
        Err(_) => PathBuf::from(""),
    }
}

fn default_home_dir() -> Result<PathBuf, ErrorKind> {
    let mut home = dirs::home_dir().ok_or(ErrorKind::NotFound)?;
    home.push(".nvmd");
    Ok(home)
}

pub enum Error {
    Message(String),
    Code(i32),
}

pub trait IntoResult<T> {
    fn into_result(self) -> Result<T, Error>;
}

impl IntoResult<()> for Result<ExitStatus, String> {
    fn into_result(self) -> Result<(), Error> {
        match self {
            Ok(status) => {
                if status.success() {
                    Ok(())
                } else {
                    let code = status.code().unwrap_or(1);
                    Err(Error::Code(code))
                }
            }
            Err(err) => Err(Error::Message(err)),
        }
    }
}
