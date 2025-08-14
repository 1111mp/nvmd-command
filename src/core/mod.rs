use anyhow::{anyhow, Result};
use std::{
    env::{self, ArgsOs},
    ffi::{OsStr, OsString},
    path::Path,
    process::ExitStatus,
};

mod binary;
mod corepack;
mod node;
mod npm;
mod npx;
mod nvmd;

pub fn execute() -> Result<ExitStatus> {
    let mut native_args = env::args_os();
    let exe = get_tool_name(&mut native_args)?;
    let args: Vec<_> = native_args.collect();

    match exe.to_str() {
        Some("nvmd") => nvmd::command(),
        Some("node") => node::command(&exe, &args),
        Some("npm") => npm::command(&exe, &args),
        Some("corepack") => corepack::command(&exe, &args),
        Some("npx") => npx::command(&exe, &args),
        _ => binary::command(&exe, &args),
    }
}

fn get_tool_name(args: &mut ArgsOs) -> Result<OsString> {
    args.next()
        .and_then(|arg0| Path::new(&arg0).file_name().map(tool_name_from_file_name))
        .ok_or_else(|| anyhow!("Could not determine tool name"))
}

#[cfg(unix)]
fn tool_name_from_file_name(file_name: &OsStr) -> OsString {
    file_name.to_os_string()
}

#[cfg(windows)]
fn tool_name_from_file_name(file_name: &OsStr) -> OsString {
    // On Windows PowerShell, the file name includes the .exe suffix,
    // and the Windows file system is case-insensitive
    // We need to remove that to get the raw tool name
    match file_name.to_str() {
        Some(file) => OsString::from(file.to_ascii_lowercase().trim_end_matches(".exe")),
        None => OsString::from(file_name),
    }
}
