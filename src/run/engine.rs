use super::{anyhow, Result};
use super::{ExitStatus, OsStr, OsString};

use crate::{command as CommandTool, common::ENV_PATH};

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus> {
    if ENV_PATH.is_empty() {
        return Err(anyhow!("command not found: {:?}", exe));
    }

    let status = CommandTool::create_command(exe)
        .env("PATH", ENV_PATH.clone())
        .args(args)
        .status()?;

    Ok(status)
}
