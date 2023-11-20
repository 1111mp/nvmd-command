use std::path::PathBuf;

use super::{ExitStatus, OsStr, OsString};

use crate::{
    command as CommandTool,
    common::{BINARY_ENV_PATH, NPM_PREFIX},
};

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus, String> {
    let mut lib_path = PathBuf::from(NPM_PREFIX.clone());
    if cfg!(unix) {
        // unix
        lib_path.push("bin");
    }
    lib_path.push(exe);

    if !lib_path.exists() {
        return Err(String::from("command not found: ") + exe.to_str().unwrap());
    }

    let child = CommandTool::create_command(exe)
        .env("PATH", BINARY_ENV_PATH.clone())
        .args(args)
        .status();

    match child {
        Ok(status) => Ok(status),
        Err(_) => Err(String::from("failed to execute process")),
    }
}
