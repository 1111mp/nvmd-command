use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};

static SHIM_HAS_CONTROL: AtomicBool = AtomicBool::new(false);
const INTERRUPTED_EXIT_CODE: i32 = 130;

pub fn pass_control_to_shim() {
    SHIM_HAS_CONTROL.store(true, Ordering::SeqCst);
}

pub fn setup_signal_handler() {
    let result = ctrlc::set_handler(|| {
        if !SHIM_HAS_CONTROL.load(Ordering::SeqCst) {
            exit(INTERRUPTED_EXIT_CODE);
        }
    });

    if result.is_err() {
        eprintln!(
            "{}",
            console::style("Unable to set Ctrl+C handler, SIGINT will not be handled correctly")
                .red()
        );
    }
}
