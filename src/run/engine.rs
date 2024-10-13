use super::{anyhow, Result};
use super::{ExitStatus, OsStr, OsString};

use crate::{command as CommandTool, common::ENV_PATH};

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus> {
    let path = ENV_PATH
        .clone()
        .ok_or_else(|| anyhow!("command not found: {:?}", exe))?;

    let status = CommandTool::create_command(exe)
        .env("PATH", path)
        .args(args)
        .status()?;

    Ok(status)
}
