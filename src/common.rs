use anyhow::{bail, Context, Result};
#[cfg(windows)]
use fs_extra::file::{copy, CopyOptions};
use fs_extra::file::{read_to_string, remove};
use lazy_static::lazy_static;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{from_str, Value};
use std::{collections::HashMap, env, ffi::OsString, fs, path::PathBuf};

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
        let setting_content = read_to_string(&setting_path)?;
        if setting_content.is_empty() {
            return Ok(DEFAULT_INSTALLATION_PATH.clone());
        }

        let json_obj: Value = from_str(&setting_content)?;
        if let Some(directory) = json_obj["directory"].as_str() {
            Ok(Some(PathBuf::from(directory)))
        } else {
            Ok(DEFAULT_INSTALLATION_PATH.clone())
        }
    } else {
        Ok(None)
    }
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

pub fn package_can_be_removed(name: &String) -> Result<bool> {
    if let Some(mut packages_path) = NVMD_PATH.clone() {
        packages_path.push("packages.json");
        let packages = read_json::<Packages>(&packages_path)?;
        if packages.is_empty() {
            return Ok(true);
        }
        if let Some(package) = packages.get(name) {
            if package.is_empty() {
                return Ok(true);
            }

            if package.len() == 1 && package.contains(&VERSION.clone().unwrap()) {
                return Ok(true);
            }
        }
        return Ok(true);
    }
    Ok(true)
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// is it active
    pub active: bool,

    /// project name
    pub name: String,

    /// project path
    pub path: String,

    /// the node version of project used
    pub version: Option<String>,

    /// create date
    pub create_at: Option<String>,

    /// update date
    pub update_at: Option<String>,
}

pub type Packages = HashMap<String, Vec<String>>;

pub fn read_json<T: DeserializeOwned>(path: &PathBuf) -> Result<T> {
    if !path.exists() {
        bail!("file not found \"{}\"", path.display());
    }

    let json_str = fs::read_to_string(path)
        .with_context(|| format!("failed to read the file \"{}\"", path.display()))?;

    serde_json::from_str::<T>(&json_str).with_context(|| {
        format!(
            "failed to read the file with json format \"{}\"",
            path.display()
        )
    })
}

#[cfg(unix)]
pub fn link_package(name: &str) -> Result<()> {
    use std::os::unix::fs;
    if let Some(path) = NVMD_PATH.clone() {
        let mut source = path.clone();
        source.push("bin");
        source.push("nvmd");
        let mut alias = path.clone();
        alias.push("bin");
        alias.push(name);

        fs::symlink(source, alias)?;
    }
    Ok(())
}

#[cfg(windows)]
pub fn link_package(name: &str) -> Result<()> {
    if let Some(path) = NVMD_PATH.clone() {
        let mut exe_source = path.clone();
        exe_source.push("bin");
        exe_source.push("nvmd.exe");
        let mut cmd_source = path.clone();
        cmd_source.push("bin");
        cmd_source.push("npm.cmd");

        let mut exe_alias = path.clone();
        exe_alias.push("bin");
        exe_alias.push(format!("{}.exe", name));
        let mut cmd_alias = path.clone();
        cmd_alias.push("bin");
        cmd_alias.push(format!("{}.cmd", name));

        let mut options = CopyOptions::new();
        options.skip_exist = true;
        copy(&exe_source, &exe_alias, &options)?;
        copy(&cmd_source, &cmd_alias, &options)?;
    }
    Ok(())
}

#[cfg(unix)]
pub fn unlink_package(name: &str) -> Result<()> {
    if let Some(mut alias) = NVMD_PATH.clone() {
        alias.push("bin");
        alias.push(name);

        remove(alias)?;
    }
    Ok(())
}

#[cfg(windows)]
pub fn unlink_package(name: &str) -> Result<()> {
    if let Some(path) = NVMD_PATH.clone() {
        let mut exe_alias = path.clone();
        exe_alias.push("bin");
        exe_alias.push(format!("{}.exe", name));
        let mut cmd_alias = path.clone();
        cmd_alias.push("bin");
        cmd_alias.push(format!("{}.cmd", name));

        remove(exe_alias)?;
        remove(cmd_alias)?;
    }
    Ok(())
}
