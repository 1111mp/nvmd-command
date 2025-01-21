mod command;
mod common;
mod core;
mod signal;
mod utils;

use core::execute;
use signal::setup_signal_handler;
use std::process;

fn main() {
    setup_signal_handler();

    let result = execute();
    match result {
        Ok(exit_status) => {
            // If the code method returns None (meaning the process exited due to receiving a signal)
            // Extract the exit code, using the default value 0 if the process terminated due to a signal
            let code = exit_status.code().unwrap_or(0);
            process::exit(code);
        }
        Err(error) => {
            // Print error messages to standard error output
            eprintln!("nvm-desktop: {}", error);
            process::exit(1);
        }
    }
}
