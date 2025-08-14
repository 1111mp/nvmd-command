use super::nvmd_home;
use crate::utils::help::{read_json, write_json};
use anyhow::Result;
use serde::Deserialize;
use std::{collections::HashMap, env, ffi::OsStr, path::PathBuf};

pub type PackagesData = HashMap<String, Vec<String>>;

pub struct Packages {
    path: PathBuf,
    data: HashMap<String, Vec<String>>,
}

impl Packages {
    pub fn new() -> Result<Self> {
        let path = nvmd_home()?.packages_path();
        let data = read_json::<PackagesData>(&path).unwrap_or_default();
        Ok(Self { path, data })
    }

    pub fn save(&self) -> Result<()> {
        write_json(&self.path, &self.data)
    }

    pub fn record_installed(&mut self, packages: &[String], version: &str) {
        for pkg in packages {
            // Remove the specified version first
            self.data
                .entry(pkg.clone())
                .or_default()
                .retain(|v| v != version);
            // Then add the specified version
            self.data
                .entry(pkg.clone())
                .or_default()
                .push(version.to_string());
        }
    }

    pub fn record_uninstalled(&mut self, packages: &[String], version: &str) -> Vec<String> {
        let mut to_remove = Vec::new();
        for pkg in packages {
            if let Some(versions) = self.data.get_mut(pkg) {
                versions.retain(|v| v != version);
                if versions.is_empty() {
                    to_remove.push(pkg.clone());
                }
            } else {
                to_remove.push(pkg.clone());
            }
        }
        to_remove
    }

    pub fn can_be_removed(&self, name: &str) -> bool {
        self.data.get(name).map(|v| v.is_empty()).unwrap_or(true)
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PackageJson {
    pub name: Option<String>,
    pub bin: Option<Bin>,
}

/// https://docs.npmjs.com/cli/v10/configuring-npm/package-json#bin
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Bin {
    Single(String),
    Multiple(HashMap<String, String>),
}

impl PackageJson {
    pub fn new(prefix: &PathBuf, pkg: impl AsRef<OsStr>) -> Self {
        let path = prefix.join(pkg.as_ref()).join("package.json");
        read_json::<PackageJson>(&path).unwrap_or_default()
    }

    pub fn from_current_dir() -> Result<Self> {
        let path = env::current_dir()?.join("package.json").canonicalize()?;
        read_json::<PackageJson>(&path)
    }

    pub fn from_current_dir_with_pkg(pkg: impl AsRef<OsStr>) -> Result<Self> {
        let path = env::current_dir()?
            .join(pkg.as_ref())
            .join("package.json")
            .canonicalize()?;
        read_json::<PackageJson>(&path)
    }

    pub fn bin_names(&self) -> Vec<String> {
        match &self.bin {
            Some(Bin::Single(_bin)) => vec![self.name.clone().unwrap_or_default()],
            Some(Bin::Multiple(map)) => map.keys().cloned().collect(),
            None => vec![],
        }
    }
}
