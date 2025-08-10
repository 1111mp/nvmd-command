use crate::module::nvmd_home;
use crate::module::Setting;
use anyhow::{bail, Context, Result};
use fs_extra::file::{remove, write_all};
use serde::de::DeserializeOwned;
use std::{fs, path::PathBuf};

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

pub fn node_strict_available(version: &str) -> Result<bool> {
    let mut path = Setting::global()?.get_directory()?.join(version);

    if cfg!(windows) {
        path.push("node.exe");
    }
    if cfg!(unix) {
        path.push("bin");
        path.push("node");
    }
    Ok(path.exists())
}

pub fn node_available(version: &str) -> Result<bool> {
    Setting::global()
        .and_then(|s| s.get_directory())
        .map(|dir| dir.join(version).exists())
}

pub fn sanitize_version(version: &str) -> String {
    version.strip_prefix('v').unwrap_or(version).to_string()
}

#[cfg(unix)]
pub fn link_package(name: &str) -> Result<()> {
    use std::os::unix::fs;

    let home = nvmd_home()?;
    let source = home.bin_dir().join("nvmd");
    let alias = home.bin_dir().join(name);
    fs::symlink(source, alias)?;

    Ok(())
}

#[cfg(windows)]
pub fn link_package(name: &str) -> Result<()> {
    use fs_extra::file::{copy, CopyOptions};

    let home = nvmd_home()?;
    let exe_source = home.bin_dir().join("nvmd.exe");
    let cmd_source = home.bin_dir().join("npm.cmd");
    let exe_alias = home.bin_dir().join(format!("{}.exe", name));
    let cmd_alias = home.bin_dir().join(format!("{}.cmd", name));
    let options = CopyOptions::new().skip_exist(true);

    copy(&exe_source, &exe_alias, &options)?;
    copy(&cmd_source, &cmd_alias, &options)?;

    Ok(())
}

#[cfg(unix)]
pub fn unlink_package(name: &str) -> Result<()> {
    let alias = nvmd_home()?.bin_dir().join(name);
    remove(alias)?;

    Ok(())
}

#[cfg(windows)]
pub fn unlink_package(name: &str) -> Result<()> {
    let home = nvmd_home()?;
    let exe_alias = home.bin_dir().join(format!("{}.exe", name));
    let cmd_alias = home.bin_dir().join(format!("{}.cmd", name));

    remove(exe_alias)?;
    remove(cmd_alias)?;

    Ok(())
}
