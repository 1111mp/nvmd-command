use crate::{
    module::{Context, Groups, Setting},
    utils::help::node_version_parse,
};
use anyhow::Result;
use fs_extra::dir::{ls, DirEntryAttr, DirEntryValue};
use std::{cmp::Ordering, collections::HashSet};
use version_compare::{compare, Cmp};

#[derive(clap::Args)]
pub struct List {
    /// List tha all groups of the project
    #[arg(short, long)]
    group: bool,
}

impl super::Command for List {
    fn run(self) -> Result<()> {
        match self.group {
            true => self.list_group(),
            false => self.list(),
        }
    }
}

impl List {
    fn list(self) -> Result<()> {
        let path = Setting::global()?.get_directory()?;
        let target_version = Context::global()?.get_version().unwrap_or_default();
        let mut config = HashSet::new();
        config.insert(DirEntryAttr::Name);

        let mut versions: Vec<_> = ls(&path, &config)?
            .items
            .into_iter()
            .filter_map(|item| {
                let version_str = match item.get(&DirEntryAttr::Name)? {
                    DirEntryValue::String(s) => s,
                    _ => return None,
                };
                node_version_parse(version_str)
                    .ok()
                    .map(|_| version_str.to_string())
            })
            .collect();

        versions.sort_by(|a, b| match compare(b, a) {
            Ok(Cmp::Lt) => Ordering::Less,
            Ok(Cmp::Eq) => Ordering::Equal,
            Ok(Cmp::Gt) => Ordering::Greater,
            _ => Ordering::Equal,
        });
        for version in versions {
            if version == target_version {
                eprintln!(
                    "{}",
                    console::style(format!("v{} (currently)", version)).green()
                );
            } else {
                eprintln!("v{}", version);
            }
        }

        Ok(())
    }

    fn list_group(self) -> Result<()> {
        for group in Groups::new()?.data {
            if let Some(version) = &group.version {
                eprintln!("{} v{}", group.name, version);
            } else {
                eprintln!("{}", group.name);
            }
        }
        Ok(())
    }
}
