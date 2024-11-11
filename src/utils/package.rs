use super::help::{read_json, unlink_package, write_json};

use crate::common::{NVMD_PATH, VERSION};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, ffi::OsStr, path::PathBuf, sync::MutexGuard};

pub type Packages = HashMap<String, Vec<String>>;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct PackageJson {
    pub name: Option<String>,
    pub bin: Option<Bin>,
}

/// https://docs.npmjs.com/cli/v10/configuring-npm/package-json#bin
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Bin {
    Single(String),
    Multiple(HashMap<String, String>),
}

/// record the version information of the package installed globally by npm
pub fn record_installed_package_info(packages: &Vec<String>) -> Result<()> {
    if let Some(mut path) = NVMD_PATH.clone() {
        path.push("packages.json");
        let mut packages_json = read_json::<Packages>(&path).ok().unwrap_or_default();
        let version = VERSION.clone().unwrap();
        for package_name in packages {
            packages_json
                .entry(package_name.clone())
                .or_insert_with(Vec::new);

            if let Some(package_value) = packages_json.get_mut(package_name) {
                if !package_value.contains(&version) {
                    package_value.push(version.clone());
                }
            }
        }
        write_json(&path, &packages_json)?;
    }
    Ok(())
}

pub fn record_uninstall_package_info(names: &MutexGuard<Vec<String>>) -> Result<()> {
    if let Some(mut path) = NVMD_PATH.clone() {
        path.push("packages.json");
        let mut packages_json = read_json::<Packages>(&path).ok().unwrap_or_default();
        let version = VERSION.clone().unwrap();
        for name in names.iter() {
            let mut should_remove = true;
            if let Some(versions) = packages_json.get_mut(name) {
                if let Some(pos) = versions.iter().position(|x| *x == version) {
                    versions.remove(pos);
                }
                if versions.len() > 0 {
                    should_remove = false;
                }
            }
            if should_remove {
                unlink_package(name)?;
            }
        }

        write_json(&path, &packages_json)?;
    }

    Ok(())
}

/// determine whether a package can be removed
pub fn package_can_be_removed(name: &String) -> Result<bool> {
    if let Some(packages) = read_packages() {
        if let Some(package) = packages.get(name) {
            if package.len() > 0 {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

pub fn read_packages() -> Option<Packages> {
    if let Some(mut packages_path) = NVMD_PATH.clone() {
        packages_path.push("packages.json");
        let packages = read_json::<Packages>(&packages_path).ok();
        return packages;
    }
    None
}

pub fn collect_package_bin_names(
    npm_perfix: &String,
    packages: &Vec<&OsStr>,
) -> Result<Vec<String>> {
    let mut package_bin_names = Vec::new();
    for package in packages {
        if let Some(package_json) = read_package_json(npm_perfix, package)? {
            match package_json.bin {
                Some(Bin::Single(_bin)) => {
                    package_bin_names.push(package_json.name.unwrap_or_default())
                }
                Some(Bin::Multiple(bin)) => package_bin_names.extend(bin.keys().cloned()),
                None => {}
            }
        }
    }

    Ok(package_bin_names)
}

pub fn collect_package_bin_names_for_link(
    npm_perfix: &String,
    packages: &Vec<&OsStr>,
) -> Result<Vec<String>> {
    let mut package_bin_names: Vec<String> = vec![];
    for package in packages {
        let mut package_path = if PathBuf::from(package).is_relative() {
            let mut cur_dir = env::current_dir()?;
            cur_dir.push(package);
            cur_dir.canonicalize()?
        } else {
            let mut path = PathBuf::from(npm_perfix);
            path.push(package);
            path
        };
        package_path.push("package.json");
        if let Some(package_json) = read_json::<PackageJson>(&package_path).ok() {
            match package_json.bin {
                Some(Bin::Single(_bin)) => {
                    package_bin_names.push(package_json.name.unwrap_or_default())
                }
                Some(Bin::Multiple(bin)) => package_bin_names.extend(bin.keys().cloned()),
                None => {}
            }
        }
    }

    Ok(package_bin_names)
}

/// find package bin namesc from current dir for npm link/unlink command
pub fn collect_package_bin_names_from_curdir() -> Result<Vec<String>> {
    let mut package_bin_names = Vec::new();
    let mut package_path = env::current_dir()?;
    package_path.push("package.json");
    if let Some(package_json) = read_json::<PackageJson>(&package_path).ok() {
        match package_json.bin {
            Some(Bin::Single(_bin)) => {
                package_bin_names.push(package_json.name.unwrap_or_default())
            }
            Some(Bin::Multiple(bin)) => package_bin_names.extend(bin.keys().cloned()),
            None => {}
        }
    }

    Ok(package_bin_names)
}

/// read info from package's `package.json` file
fn read_package_json(path: &String, package: &OsStr) -> Result<Option<PackageJson>> {
    let mut package_path = PathBuf::from(path);
    package_path.push(package);
    package_path.push("package.json");
    let package_json = read_json::<PackageJson>(&package_path).ok();
    Ok(package_json)
}
