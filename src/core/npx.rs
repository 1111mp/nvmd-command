use super::{ExitStatus, OsStr, OsString};
use crate::{module::Context, signal::pass_control_to_shim, utils::command};
use anyhow::Result;

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus> {
    let path = Context::global()?.env_path()?;

    let mut command = command::create_command(exe);
    command.args(args);
    command.env("PATH", path);

    pass_control_to_shim();

    let status = command.status()?;
    Ok(status)
}
