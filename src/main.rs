use std::{env, ffi::OsString, process};

use crate::nvmd::{ENV_PATH, NVMD_PATH, VERSION};
use lazy_static::lazy_static;

mod command;
mod nvmd;

lazy_static! {
    static ref INSTALL: OsString = OsString::from("install");
    static ref UNINSTALL: OsString = OsString::from("uninstall");
    static ref GLOBAL: OsString = OsString::from("--global");
    static ref SHORT_GLOBAL: OsString = OsString::from("-g");
}

fn main() {
    let mut native_args = env::args_os();
    let exe = nvmd::get_tool_name(&mut native_args).expect("get tool name error");
    let args: Vec<_> = native_args.collect();

    let lib = match exe.clone().into_string() {
        Ok(s) => s,
        Err(_) => String::from(""),
    };

    if !lib.is_empty() && lib != "node" && lib != "npm" && lib != "npx" && lib != "corepack" {
        // check the third package
        let mut lib_path = NVMD_PATH.clone();
        lib_path.push("versions");
        lib_path.push(VERSION.clone());
        if cfg!(unix) {
            // unix
            lib_path.push("bin");
        }
        lib_path.push(&lib);

        if !lib_path.exists() {
            println!("[nvm-desktop] command not found: {}", &lib);
            process::exit(1);
        }
    }

    // for npm uninstall -g packages
    // collection the bin names of packages
    if lib == "npm"
        && args.contains(&UNINSTALL)
        && (args.contains(&GLOBAL) || args.contains(&SHORT_GLOBAL))
    {
        nvmd::collection_packages_name(&args);
    }

    let mut command = command::create_command(&exe);

    let child = command
        .env("PATH", ENV_PATH.clone())
        .args(&args)
        .spawn()
        .expect("command failed to start");

    let output = child.wait_with_output().expect("failed to wait on child");

    let code = match output.status.success() {
        true => 0,
        false => 1,
    };

    if code == 0 {
        // successed
        if (args.contains(&INSTALL) || args.contains(&UNINSTALL))
            && (args.contains(&SHORT_GLOBAL) || args.contains(&GLOBAL))
        {
            if args.contains(&INSTALL) {
                nvmd::install_packages(&args);
            }

            if args.contains(&UNINSTALL) {
                nvmd::uninstall_packages();
            }
        }
    }

    process::exit(code);
}
