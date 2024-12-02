use anyhow::{Context, Result};
use fs_extra::{error::Error, file::read_to_string};
use lazy_static::lazy_static;
use serde_json::{from_str, Value};
use std::{env, ffi::OsString, path::PathBuf};

lazy_static! {
    pub static ref NVMD_PATH: Option<PathBuf> = get_nvmd_path().ok();
    pub static ref VERSION: Option<String> = get_version().unwrap_or(None);
    pub static ref DEFAULT_INSTALLATION_PATH: Option<PathBuf> = get_default_installation_path();
    pub static ref INSTALLTION_PATH: Option<PathBuf> = get_installation_path().unwrap_or(None);
    pub static ref ENV_PATH: Option<OsString> = get_env_path();
}

fn get_env_path() -> Option<OsString> {
    VERSION.as_ref().and_then(|version| {
        get_bin_path(version).and_then(|bin_path| {
            if !PathBuf::from(&bin_path).exists() {
                return None;
            }
            env::var_os("PATH").map_or(Some(bin_path.clone()), |path| {
                let mut paths = env::split_paths(&path).collect::<Vec<_>>();
                paths.insert(0, PathBuf::from(bin_path));
                env::join_paths(paths).ok()
            })
        })
    })
}

fn get_bin_path(version: &str) -> Option<OsString> {
    INSTALLTION_PATH.clone().map(|mut nvmd_path| {
        nvmd_path.push(version);
        if cfg!(unix) {
            nvmd_path.push("bin");
        }
        nvmd_path.into_os_string()
    })
}

fn get_installation_path() -> Result<Option<PathBuf>> {
    if let Some(mut setting_path) = NVMD_PATH.clone() {
        setting_path.push("setting.json");

        let setting_content = read_to_string(&setting_path).or::<Error>(Ok("".to_string()))?;
        if setting_content.is_empty() {
            return Ok(DEFAULT_INSTALLATION_PATH.clone());
        }

        let json_obj: Value = from_str(&setting_content)?;
        if let Some(directory) = json_obj["directory"].as_str() {
            return Ok(Some(PathBuf::from(directory)));
        } else {
            return Ok(DEFAULT_INSTALLATION_PATH.clone());
        }
    }

    Ok(None)
}

fn get_default_installation_path() -> Option<PathBuf> {
    NVMD_PATH.clone().map(|mut default_path| {
        default_path.push("versions");
        default_path
    })
}

fn get_version() -> Result<Option<String>> {
    if let Some(nvmdrc) = find_nvmdrc()? {
        let project_version = read_to_string(&nvmdrc)?;
        if !project_version.is_empty() {
            return Ok(Some(project_version));
        }
    }

    if let Some(mut default_path) = NVMD_PATH.clone() {
        default_path.push("default");
        let version = read_to_string(&default_path)?;
        if !version.is_empty() {
            return Ok(Some(version));
        }
    }

    Ok(None)
}

fn find_nvmdrc() -> Result<Option<PathBuf>> {
    let mut current_dir = env::current_dir()?;
    current_dir.push(".nvmdrc");

    while current_dir.pop() {
        let mut nvmdrc = current_dir.clone();
        nvmdrc.push(".nvmdrc");
        if nvmdrc.is_file() {
            return Ok(Some(nvmdrc));
        }
    }

    Ok(None)
}

fn get_nvmd_path() -> Result<PathBuf> {
    let mut home = dirs::home_dir().context("home directory not found")?;
    home.push(".nvmd");
    Ok(home)
}
