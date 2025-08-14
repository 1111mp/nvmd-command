use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use std::path::PathBuf;

static NVMD_HOME: OnceCell<NvmdHome> = OnceCell::new();

pub fn nvmd_home<'a>() -> Result<&'a NvmdHome> {
    NVMD_HOME.get_or_try_init(|| {
        let home_dir = match std::env::var_os("NVMD_HOME") {
            Some(home) => PathBuf::from(home),
            None => default_home_dir()?,
        };

        Ok(NvmdHome::new(home_dir))
    })
}

pub struct NvmdHome(PathBuf);

impl NvmdHome {
    pub fn new(home: PathBuf) -> Self {
        Self(home)
    }

    pub fn home(&self) -> &PathBuf {
        &self.0
    }

    pub fn bin_dir(&self) -> PathBuf {
        self.0.join("bin")
    }

    pub fn versions_dir(&self) -> PathBuf {
        self.0.join("versions")
    }

    pub fn default_path(&self) -> PathBuf {
        self.0.join("default")
    }

    pub fn setting_path(&self) -> PathBuf {
        self.0.join("setting.json")
    }

    pub fn projects_path(&self) -> PathBuf {
        self.0.join("projects.json")
    }

    pub fn groups_path(&self) -> PathBuf {
        self.0.join("groups.json")
    }

    pub fn packages_path(&self) -> PathBuf {
        self.0.join("packages.json")
    }
}

fn default_home_dir() -> Result<PathBuf> {
    let mut home = dirs::home_dir().context("Could not determine home directory")?;
    home.push(".nvmd");
    Ok(home)
}
