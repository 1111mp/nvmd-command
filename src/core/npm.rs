use crate::module::Context;
use crate::npm::{CommandArg, InterceptedCommand};
use crate::signal::pass_control_to_shim;
use crate::utils::command;
use anyhow::Result;
use std::ffi::{OsStr, OsString};
use std::process::ExitStatus;

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus> {
    let path = Context::global()?.env_path()?;
    let command_arg = CommandArg::from_npm(args);

    // Before command execution
    match &command_arg {
        CommandArg::Global(cmd) => {
            cmd.before_executor()?;
        }
        CommandArg::Intercepted(InterceptedCommand::Unlink(unlink)) => {
            unlink.before_executor()?;
        }
        _ => {}
    };

    let mut command = command::create_command(exe);
    command.args(args);
    command.env("PATH", path);

    pass_control_to_shim();

    let status = command.status()?;

    // After the command is executed
    if status.success() {
        match &command_arg {
            CommandArg::Global(cmd) => {
                cmd.after_executor()?;
            }
            CommandArg::Intercepted(InterceptedCommand::Link(link)) => {
                link.after_executor()?;
            }
            CommandArg::Intercepted(InterceptedCommand::Unlink(unlink)) => {
                unlink.after_executor()?;
            }
            _ => {}
        };
    }

    Ok(status)
}
