use std::{env, process};

mod command;
mod nvmd;

fn main() {
    let mut native_args = env::args_os();
    let exe = nvmd::get_tool_name(&mut native_args).expect("get tool name error");
    let args: Vec<_> = native_args.collect();

    let env_path = nvmd::get_env_path();
    // println!("{:?}", env_path);

    let mut command = command::create_command(exe);

    // .env("PATH", "/Users/zhangyifan/.nvmd/versions/18.17.1/bin:/Users/zhangyifan/.nvmd/bin::/opt/homebrew/bin:/opt/homebrew/sbin:/usr/local/bin:/System/Cryptexes/App/usr/bin:/usr/bin:/bin:/usr/sbin:/sbin:/Library/Apple/usr/bin:/var/run/com.apple.security.cryptexd/codex.system/bootstrap/usr/local/bin:/var/run/com.apple.security.cryptexd/codex.system/bootstrap/usr/bin:/var/run/com.apple.security.cryptexd/codex.system/bootstrap/usr/appleinternal/bin:/Users/zhangyifan/.cargo/bin")
    let child = command
        .env("PATH", env_path)
        .args(args)
        .spawn()
        .expect("command failed to start");

    let output = child.wait_with_output().expect("failed to wait on child");

    let code = match output.status.success() {
        true => 0,
        false => 1,
    };

    process::exit(code);
}
