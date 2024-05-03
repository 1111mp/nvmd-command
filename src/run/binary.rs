use super::{anyhow, Result};
use super::{ExitStatus, OsStr, OsString};

use crate::{
    command as CommandTool,
    common::{ENV_PATH, INSTALLTION_PATH, VERSION},
};

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus> {
    let mut lib_path = INSTALLTION_PATH.clone();
    lib_path.push(VERSION.clone());
    if cfg!(unix) {
        // unix
        lib_path.push("bin");
    }
    lib_path.push(exe);

    if !lib_path.exists() {
        return Err(anyhow!("command not found: {:?}", exe));
    }

    let status = CommandTool::create_command(exe)
        .env("PATH", ENV_PATH.clone())
        .args(args)
        .status()?;

    Ok(status)
}
