use anyhow::bail;

use super::Result;
use super::{ExitStatus, OsStr, OsString};

use crate::signal::pass_control_to_shim;
use crate::{
    command as CommandTool,
    common::{ENV_PATH, VERSION},
};

pub(super) fn command(exe: &OsStr, args: &[OsString]) -> Result<ExitStatus> {
    let path = match ENV_PATH.as_ref() {
        Some(env_path) => env_path,
        None => {
            if VERSION.is_none() {
                bail!("the default node version is not set, you can set it by executing \"nvmd use {{version}}\"");
            }
            if let Some(version) = VERSION.as_ref() {
                bail!(
                    "version v{} is not installed, please install it before using",
                    version
                );
            }
            bail!("command not found: {:?}", exe);
        }
    };

    let mut command = CommandTool::create_command(exe);
    command.env("PATH", path).args(args);

    pass_control_to_shim();

    let status = command.status()?;
    Ok(status)
}
