use super::{ExitStatus, OsStr, OsString};

use crate::{
    command as CommandTool,
    common::{ENV_PATH, INSTALLTION_PATH, VERSION},
};

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus, String> {
    let mut lib_path = INSTALLTION_PATH.clone();
    lib_path.push(VERSION.clone());
    if cfg!(unix) {
        // unix
        lib_path.push("bin");
    }
    lib_path.push(exe);

    if !lib_path.exists() {
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
