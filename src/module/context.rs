use super::nvmd_home;
use crate::module::Setting;
use anyhow::{anyhow, bail, Context as AnyhowContext, Result};
use fs_extra::file::read_to_string;
use once_cell::sync::OnceCell;
use std::{ffi::OsString, path::PathBuf};

pub struct Context {
    pub version: Option<String>,
}

impl Context {
    pub fn global<'a>() -> Result<&'a Context> {
        static CONTEXT: OnceCell<Context> = OnceCell::new();

        CONTEXT.get_or_try_init(|| {
            let version = get_version()?;
            Ok(Self { version })
        })
    }

    pub fn get_version(&self) -> Option<String> {
        self.version.clone()
    }

    pub fn env_path(&self) -> Result<OsString> {
        let version = self.version
            .clone()
            .ok_or(
             anyhow!("The default Node version is not set, you can set it by executing \"nvmd use {{version}}\"")
            )?;

        let mut path = super::Setting::global()?.get_directory()?.join(&version);
        if cfg!(unix) {
            path.push("bin");
        }

        if !path.exists() {
            bail!(
                "Node@v{} is not installed, please install it before using",
                &version
            );
        }

        let old_env_path = envoy::path().unwrap_or_else(|| envoy::Var::from(""));
        old_env_path
            .split()
            .prefix_entry(&path)
            .join()
            .with_context(|| {
                anyhow!(
                    "Could not create execution environment.\nPlease ensure your PATH is valid."
                )
            })
    }
}

fn get_version() -> Result<Option<String>> {
    // 1. Check the NVMD_NODE_VERSION environment variable first
    if let Ok(env_version) = std::env::var("NVMD_NODE_VERSION") {
        let v = env_version.trim();
        if !v.is_empty() {
            return Ok(Some(v.to_string()));
        }
    }

    // 2. Look for a .nvmdrc file in the current directory or its ancestors
    //    If found, read its content and return the first non-empty value
    if let Some(path) = find_nvmdrc()? {
        let content = read_to_string(&path)?;
        let t = content.trim();
        if !t.is_empty() {
            return Ok(Some(t.to_string()));
        }
    }

    // 3. If .nvmdrc is missing or empty, check the default configuration file path
    //    Only proceed if the file exists and contains a non-empty value
    let default_path = nvmd_home()?.default_path();
    if default_path.is_file() {
        let content = read_to_string(&default_path)?;
        let t = content.trim();
        if !t.is_empty() {
            return Ok(Some(t.to_string()));
        }
    }

    // 4. If no valid version is found, return None
    Ok(None)
}

fn find_nvmdrc() -> Result<Option<PathBuf>> {
    let file_name = Setting::global()?.get_node_version_file();
    Ok(std::env::current_dir()?
        .ancestors()
        .map(|dir| dir.join(&file_name))
        .find(|path| path.is_file()))
}
