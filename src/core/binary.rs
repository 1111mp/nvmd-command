use crate::module::Context;
use crate::signal::pass_control_to_shim;
use crate::utils::command;
use anyhow::Result;
use std::ffi::{OsStr, OsString};
use std::process::ExitStatus;

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus> {
    let global_context = Context::global()?;
    // check if the lib is installed under current Nodejs version, if not, return an error
    global_context.check_lib_path(exe)?;

    let path = global_context.env_path()?;
    let mut command = command::create_command(exe);
    command.args(args);
    command.env("PATH", path);

    pass_control_to_shim();

    let status = command.status()?;
    Ok(status)
}
