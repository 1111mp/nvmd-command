use super::{ExitStatus, OsStr, OsString};

use crate::{command as CommandTool, common::ENV_PATH};

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus, String> {
    if ENV_PATH.is_empty() {
        return Err(String::from("command not found: ") + exe.to_str().unwrap());
    }

    let child = CommandTool::create_command(exe)
        .env("PATH", ENV_PATH.clone())
        .args(args)
        .status();

    match child {
        Ok(status) => Ok(status),
        Err(_) => Err(String::from("failed to execute process")),
    }
}
