use super::{anyhow, Result};
use super::{ExitStatus, OsStr, OsString};

use crate::signal::pass_control_to_shim;
use crate::{
    command as CommandTool,
    common::{ENV_PATH, INSTALLTION_DIRECTORY, VERSION},
};

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus> {
    let lib_path = INSTALLTION_DIRECTORY.clone().and_then(|mut path| {
        VERSION.clone().map(|version| {
            path.push(version);
            if cfg!(unix) {
                path.push("bin");
            }
            path.push(exe);
            path
        })
    });

    // Check if the path exists and return an error if it doesn't
    match lib_path {
        Some(ref path) if path.exists() => path,
        _ => return Err(anyhow!("command not found: {:?}", exe)),
    };

    let mut command = CommandTool::create_command(exe);
    command
        .env("PATH", ENV_PATH.clone().unwrap_or_default())
        .args(args);

    pass_control_to_shim();

    let status = command.status()?;
    Ok(status)
}
