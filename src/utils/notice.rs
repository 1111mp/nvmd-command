use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Current,
    Version,
    Project,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Notice {
    source: Source,
    name: Option<String>,
    version: Option<String>,
}

impl Notice {
    pub fn from_current(version: String) -> Self {
        Self {
            source: Source::Current,
            name: None,
            version: Some(version),
        }
    }

    pub fn from_version() -> Self {
        Self {
            source: Source::Version,
            name: None,
            version: None,
        }
    }

    pub fn from_project(name: String, version: String) -> Self {
        Self {
            source: Source::Project,
            name: Some(name),
            version: Some(version),
        }
    }

    pub fn send(self) -> Result<()> {
        attohttpc::post("http://127.0.0.1:53333/notice")
            .json(&self)?
            .send()?;

        Ok(())
    }
}
