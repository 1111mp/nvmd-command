use super::help::read_json;
use crate::common::nvmd_home;
use anyhow::{anyhow, Result};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct Setting {
    /// installation directory
    pub directory: Option<PathBuf>,

    /// download base url
    pub mirror: Option<String>,
}

impl Setting {
    pub fn global<'a>() -> Result<&'a Setting> {
        static SETTING: OnceCell<Setting> = OnceCell::new();

        SETTING.get_or_try_init(|| {
            let setting_file = nvmd_home()?.join("setting.json");
            match read_json::<Setting>(&setting_file) {
                Ok(setting) => Ok(setting),
                Err(_) => Ok(Setting::template()),
            }
        })
    }

    pub fn template() -> Self {
        Self {
            directory: None,
            mirror: Some("https://nodejs.org/dist".into()),
        }
    }

    pub fn get_mirror(&self) -> String {
        self.mirror
            .clone()
            .unwrap_or("https://nodejs.org/dist".into())
    }

    pub fn get_directory(&self) -> Result<PathBuf> {
        self.directory
            .clone()
            .ok_or(anyhow!("Could not determine node install directory"))
    }
}

pub fn get_directory(path: &PathBuf) -> Option<PathBuf> {
    match read_json::<Setting>(&path) {
        Ok(setting) => setting.directory.map(|directory| PathBuf::from(directory)),
        Err(_) => None,
    }
}
