/// Copyright (c) 2017, The Volta Contributors.
/// Copyright (c) 2017, LinkedIn Corporation.
/// https://github.com/volta-cli/volta
///
use super::{tool_version, Node};
use crate::module::Setting;
use crate::utils::progress::progress_bar;
use anyhow::{anyhow, Context, Result};
use archive::Archive;
use fs_utils::ensure_containing_dir_exists;
use node_semver::Version;
use retry::delay::Fibonacci;
use retry::{retry, OperationResult};
use std::{fs::File, path::Path};
use tempfile::{tempdir_in, NamedTempFile, TempDir};

pub fn fetch(version: &Version) -> Result<()> {
    let install_dir = Setting::global()?.get_directory()?;
    let temp_file = install_dir.join(Node::archive_filename(version));

    let archive = match load_cached_distro(&temp_file) {
        Some(archive) => {
            eprintln!(
                "Loading {} from cached archive at '{}'",
                tool_version("node", version),
                temp_file.display()
            );
            archive
        }
        None => {
            let staging = create_staging_file(&install_dir)?;
            let remote_url = determine_remote_url(version)?;
            let archive = fetch_remote_distro(version, &remote_url, staging.path())?;
            archive
        }
    };

    unpack_archive(archive, version)?;

    Ok(())
}

fn unpack_archive(archive: Box<dyn Archive>, version: &Version) -> Result<()> {
    let temp = create_staging_dir()?;
    eprintln!("Unpacking node into '{}'", temp.path().display());

    let progress = progress_bar(
        archive.origin(),
        &tool_version("node", version),
        archive.compressed_size(),
    );
    let version_string = version.to_string();

    archive
        .unpack(temp.path(), &mut |_, read| {
            progress.inc(read as u64);
        })
        .with_context(|| anyhow!("Could not unpack Node v{}", &version_string))?;

    let dest = Setting::global()?.get_directory()?.join(&version_string);
    ensure_containing_dir_exists(&dest)
        .with_context(|| anyhow!("Could not create the containing directory for {:?}", &dest))?;

    rename(temp.path().join(Node::archive_basename(version)), &dest).with_context(|| {
        anyhow!(
            "Could not create environment for Node v{}
at {:?}",
            &version_string,
            &dest
        )
    })?;

    progress.finish_and_clear();

    eprintln!("Installing node in '{}'", dest.display());

    Ok(())
}

/// Fetch the distro archive from the internet
fn fetch_remote_distro(
    version: &Version,
    url: &str,
    staging_path: &Path,
) -> Result<Box<dyn Archive>> {
    eprintln!("Downloading {} from {}", tool_version("node", version), url);
    archive::fetch_native(url, staging_path)
        .with_context(|| format!("Could not download node@{} from {} \nPlease verify your internet connection and ensure the correct version is specified.", version.to_string(), url))
}

/// Return the archive if it is valid. It may have been corrupted or interrupted in the middle of
/// downloading.
// ISSUE(#134) - verify checksum
fn load_cached_distro(file: &Path) -> Option<Box<dyn Archive>> {
    if file.is_file() {
        let file = File::open(file).ok()?;
        archive::load_native(file).ok()
    } else {
        None
    }
}

fn determine_remote_url(version: &Version) -> Result<String> {
    let distro_file_name = Node::archive_filename(version);

    Ok(format!(
        "{}/v{}/{}",
        Setting::global()?.get_mirror(),
        version,
        distro_file_name
    ))
}

/// Creates a NamedTempFile in the Volta tmp directory
fn create_staging_file(tmp_dir: &Path) -> Result<NamedTempFile> {
    NamedTempFile::new_in(tmp_dir).with_context(|| {
        format!(
            "Could not create temporary file
in {:?}",
            tmp_dir
        )
    })
}

/// Creates a staging directory in the Volta tmp directory
fn create_staging_dir() -> Result<TempDir> {
    let tmp_root = Setting::global()?.get_directory()?;
    tempdir_in(&tmp_root).with_context(|| {
        anyhow!(
            "Could not create temporary directory
in {:?}",
            &tmp_root
        )
    })
}

/// Rename a file or directory to a new name, retrying if the operation fails because of permissions
///
/// Will retry for ~30 seconds with longer and longer delays between each, to allow for virus scan
/// and other automated operations to complete.
fn rename<F, T>(from: F, to: T) -> std::io::Result<()>
where
    F: AsRef<Path>,
    T: AsRef<Path>,
{
    // 21 Fibonacci steps starting at 1 ms is ~28 seconds total
    // See https://github.com/rust-lang/rustup/pull/1873 where this was used by Rustup to work around
    // virus scanning file locks
    let from = from.as_ref();
    let to = to.as_ref();

    retry(
        Fibonacci::from_millis(1).take(21),
        || match std::fs::rename(from, to) {
            Ok(_) => OperationResult::Ok(()),
            Err(e) => match e.kind() {
                std::io::ErrorKind::PermissionDenied => OperationResult::Retry(e),
                _ => OperationResult::Err(e),
            },
        },
    )
    .map_err(|e| e.error)
}
