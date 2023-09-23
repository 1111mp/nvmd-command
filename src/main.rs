use std::process;

mod command;
mod common;
mod run;

use common::{Error, IntoResult};
use run::execute;

fn main() {
    let result = execute().into_result();
    match result {
        Ok(()) => {
            process::exit(0);
        }
        Err(Error::Code(code)) => {
            process::exit(code);
        }
        Err(Error::Message(msg)) => {
            eprintln!("nvm-desktop: {}", msg);
            process::exit(1);
        }
    }
}
