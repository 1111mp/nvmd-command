use serde::Deserialize;
use std::path::PathBuf;

use super::help::read_json;

#[derive(Debug, Default, Deserialize)]
pub struct Setting {
    /// installation directory
    pub directory: Option<String>,
}

pub fn get_directory(path: &PathBuf) -> Option<PathBuf> {
    match read_json::<Setting>(&path) {
        Ok(setting) => setting.directory.map(|directory| PathBuf::from(directory)),
        Err(_) => None,
    }
}
