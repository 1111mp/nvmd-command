use crate::{module::Context, utils::command};
use anyhow::{anyhow, bail, Context as AnyhowContext, Result};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Stdio,
};

pub(super) fn get_npm_prefix() -> Result<PathBuf> {
    let path = Context::global()?.env_path()?;
    let mut command = command::create_command("npm");

    let output = command
        .args(["root", "-g"])
        .env("PATH", path)
        .stdout(Stdio::piped())
        .output()
        .with_context(|| anyhow!("No valid npm prefix found"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let pb = PathBuf::from(line.trim());
        if pb.is_dir() {
            return Ok(pb);
        }
    }

    bail!("No valid npm prefix found");
}

pub(super) fn is_global<A>(args: &[A]) -> bool
where
    A: AsRef<OsStr>,
{
    args.iter()
        .fold(false, |global, arg| match arg.as_ref().to_str() {
            Some("-g") | Some("--global") => true,
            _ => global,
        })
}

pub(super) fn is_relative_path<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    if let Some(arg_str) = arg.as_ref().to_str() {
        let path = Path::new(arg_str);
        path.is_relative() && (path.starts_with(".") || path.starts_with(".."))
    } else {
        false
    }
}
