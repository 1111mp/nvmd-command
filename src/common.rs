use anyhow::{Context, Result};
use fs_extra::file::read_to_string;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use std::{env, ffi::OsString, path::PathBuf};

use crate::utils::setting::get_directory;

lazy_static! {
    pub static ref NVMD_PATH: Option<PathBuf> = get_nvmd_path().ok();
    pub static ref VERSION: Option<String> = get_version().unwrap_or(None);
    pub static ref INSTALLTION_DIRECTORY: Option<PathBuf> =
        get_installation_directory().unwrap_or(None);
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
    INSTALLTION_DIRECTORY.clone().map(|mut nvmd_path| {
        nvmd_path.push(version);
        if cfg!(unix) {
            nvmd_path.push("bin");
        }
        nvmd_path.into_os_string()
    })
}

fn get_installation_directory() -> Result<Option<PathBuf>> {
    if let Some(nvmd_path) = NVMD_PATH.clone() {
        let mut setting_path = nvmd_path.clone();
        setting_path.push("setting.json");
        let directory = get_directory(&setting_path);
        if directory.is_some() {
            return Ok(directory);
        }

        let mut default_directory = nvmd_path;
        default_directory.push("versions");
        return Ok(Some(default_directory));
    }

    Ok(None)
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
    let nvmdrc_file = ".nvmdrc";
    let mut current_dir = env::current_dir()?;

    loop {
        let potential_nvmdrc = current_dir.join(nvmdrc_file);
        if potential_nvmdrc.is_file() {
            return Ok(Some(potential_nvmdrc));
        }

        if !current_dir.pop() {
            break;
        }
    }

    Ok(None)
}

fn get_nvmd_path() -> Result<PathBuf> {
    let mut home = dirs::home_dir().context("home directory not found")?;
    home.push(".nvmd");
    Ok(home)
}

fn default_nvmd_home() -> Result<PathBuf> {
    let mut home = dirs::home_dir().context("Could not determine home directory")?;
    home.push(".nvmd");
    Ok(home)
}

static NVMD_HOME: OnceCell<PathBuf> = OnceCell::new();

pub fn nvmd_home<'a>() -> Result<&'a PathBuf> {
    NVMD_HOME.get_or_try_init(default_nvmd_home)
}
