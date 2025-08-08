/// Copyright (c) 2017, The Volta Contributors.
/// Copyright (c) 2017, LinkedIn Corporation.
/// https://github.com/volta-cli/volta
///
use archive::Origin;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use terminal_size::{terminal_size, Width};

pub const MAX_WIDTH: usize = 100;
const MAX_PROGRESS_WIDTH: usize = 40;

/// Determines the string to display based on the Origin of the operation.
fn action_str(origin: Origin) -> &'static str {
    match origin {
        Origin::Local => "Unpacking",
        Origin::Remote => "Fetching",
    }
}

/// Get the width of the terminal, limited to a maximum of MAX_WIDTH
pub fn text_width() -> Option<usize> {
    terminal_size().map(|(Width(w), _)| (w as usize).min(MAX_WIDTH))
}

pub fn progress_bar(origin: Origin, details: &str, len: u64) -> ProgressBar {
    let action = action_str(origin);
    let action_width = action.len() + 2; // plus 2 spaces to look nice
    let msg_width = action_width + 1 + details.len();

    //   Fetching node@9.11.2  [############>                          ]  34%
    // |--------| |---------|   |--------------------------------------|  |-|
    //    action    details                      bar                 percentage
    let bar_width = match text_width() {
        Some(width) => MAX_PROGRESS_WIDTH.min(width - 2 - msg_width - 2 - 2 - 1 - 3 - 1),
        None => MAX_PROGRESS_WIDTH,
    };

    let progress = ProgressBar::new(len);

    progress.set_message(format!(
        "{: >width$} {}",
        style(action).green().bold(),
        details,
        width = action_width,
    ));
    progress.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{msg}}  [{{bar:{}.cyan/blue}}] {{percent:>3}}%",
                bar_width
            ))
            .expect("template is valid")
            .progress_chars("#> "),
    );

    progress
}
