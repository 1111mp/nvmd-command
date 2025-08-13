use crate::{
    module::{nvmd_home, Groups, Projects},
    utils::{
        help::{node_strict_available, node_version_parse},
        notice::Notice,
    },
};
use anyhow::{anyhow, bail, Result};
use fs_extra::file::write_all;

#[derive(clap::Args)]
pub struct Use {
    /// The version number of Node.js
    version: String,

    /// Use version for project
    #[arg(short, long)]
    project: bool,
}

impl super::Command for Use {
    fn run(self) -> Result<()> {
        match self.project {
            true => self.use_project(),
            false => self.use_global(),
        }
    }
}

impl Use {
    fn use_global(self) -> Result<()> {
        let groups = Groups::new()?;
        if groups.exists(&self.version) {
            bail!("Group@{} can only be used for projects", &self.version)
        }

        let version = node_version_parse(&self.version)?;
        if !node_strict_available(&version.to_string())? {
            bail!("Node@v{} has not been installed", &version);
        }

        let default_path = nvmd_home()?.default_path();
        write_all(default_path, &version.to_string())?;
        eprintln!("Now using node v{}", &version);

        let _ = Notice::from_current(version.to_string()).send();

        Ok(())
    }

    fn use_project(self) -> Result<()> {
        let mut groups = Groups::new()?;
        let group = groups.find_by_name(&self.version);
        let is_group = group.is_some();
        let version = match group {
            Some(g) => g.version.clone().ok_or_else(|| {
                anyhow!(
                    "The Node.js version for group '{}' has not been set yet",
                    &self.version
                )
            })?,
            None => node_version_parse(&self.version)?.to_string(),
        };

        if !node_strict_available(&version)? {
            bail!("Node@v{} has not been installed", &version);
        }

        let project_path = std::env::current_dir()?;
        let project_name = project_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        Projects::update_and_save(
            &project_path.to_string_lossy(),
            project_name,
            if is_group { &self.version } else { &version },
        )?;

        if is_group {
            groups.update(&self.version, &project_path.to_string_lossy());
            groups.save()?;
        }

        let nvmdrc = project_path.join(".nvmdrc");
        write_all(nvmdrc, &version)?;

        if is_group {
            eprintln!("Now using node v{} ({})", &version, &self.version);
        } else {
            eprintln!("Now using node v{}", &version);
        }

        let _ = Notice::from_project(
            project_name.to_string(),
            if is_group {
                self.version
            } else {
                version.to_string()
            },
        )
        .send();

        Ok(())
    }
}
