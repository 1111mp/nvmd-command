use std::{fs, path::PathBuf};

use crate::common::NVMD_PATH;
use anyhow::{bail, Context, Result};
use fs_extra::file::{remove, write_all};
use serde::de::DeserializeOwned;

use crate::common::INSTALLTION_PATH;

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

pub fn write_json<T: serde::Serialize>(path: &PathBuf, data: &T) -> Result<()> {
    let json_content = serde_json::to_string_pretty(data)?;
    write_all(path, &json_content)?;
    Ok(())
}

pub fn is_valid_version(version: &String) -> bool {
    if let Some(mut version_path) = INSTALLTION_PATH.clone() {
        version_path.push(&version);
        if cfg!(windows) {
            version_path.push("node.exe");
        }
        if cfg!(unix) {
            version_path.push("bin");
            version_path.push("node");
        }
        return version_path.exists();
    }
    false
}

pub fn sanitize_version(version: &String) -> String {
    let mut version = version.clone();
    if version.starts_with("v") {
        version.remove(0);
    }
    version
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
    use fs_extra::file::{copy, CopyOptions};

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
