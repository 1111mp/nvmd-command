use super::nvmd_home;
use crate::utils::help::read_json;
use anyhow::Result;
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
            let home = nvmd_home()?;
            let path = home.setting_path();
            match read_json::<Setting>(&path) {
                Ok(setting) => Ok(setting),
                Err(_) => Ok(Self {
                    directory: Some(home.versions_dir()),
                    mirror: Some("https://nodejs.org/dist".into()),
                }),
            }
        })
    }

    pub fn get_mirror(&self) -> String {
        self.mirror
            .clone()
            .unwrap_or("https://nodejs.org/dist".into())
    }

    pub fn get_directory(&self) -> Result<PathBuf> {
        Ok(self
            .directory
            .clone()
            .unwrap_or(nvmd_home()?.versions_dir()))
    }
}
