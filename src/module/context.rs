use super::nvmd_home;
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
    for path in [find_nvmdrc()?, Some(nvmd_home()?.default_path())]
        .into_iter()
        .flatten()
    {
        let version = read_to_string(&path)?;
        if !version.trim().is_empty() {
            return Ok(Some(version.trim().to_string()));
        }
    }

    Ok(None)
}

fn find_nvmdrc() -> Result<Option<PathBuf>> {
    Ok(std::env::current_dir()?
        .ancestors()
        .map(|dir| dir.join(".nvmdrc"))
        .find(|path| path.is_file()))
}
