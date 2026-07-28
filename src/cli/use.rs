use crate::{
    module::{Groups, NodeVersionResolver, Projects, Setting, nvmd_home},
    utils::{help::node_strict_available, notice::Notice},
};
use anyhow::{Result, anyhow, bail};
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

        let version = NodeVersionResolver::resolve(&self.version)?;
        if !node_strict_available(&version)? {
            bail!("Node@v{} has not been installed", &version);
        }

        let default_path = nvmd_home()?.default_path();
        write_all(default_path, &version)?;
        eprintln!("Now using node v{}", &version);

        let _ = Notice::from_current(version.clone()).send();

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
            None => NodeVersionResolver::resolve(&self.version)?,
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

        let file_name = Setting::global()?.get_node_version_file();
        let nvmdrc = project_path.join(&file_name);
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
