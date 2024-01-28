use std::path::Path;

use git2::Repository;
use git2::Status;
use tempfile::tempdir;

const MARK_FILES: [&str; 2] = [
    "updates/findata-funnel-pull-gpayslip-success-mark",
    "updates/findata-funnel-success-mark",
];
const REPO_PATH: &str = "/Users/grzesiek/wallet";

/// Copiues the content of one directory P to another.
///
/// # Arguments
///
/// * `from` - The source directory
/// * `to` - The target directory.
fn copy_content<P, Q>(from: P, to: Q) -> Result<(), fs_extra::error::Error>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let options = fs_extra::dir::CopyOptions::new().content_only(true);
    fs_extra::dir::copy(from, to, &options).map(|_| ())
}

/// Copies a repository from the given path to a temporary directory.
///
/// # Arguments
///
/// * `repo_path` — The original repository path.
///
/// # Returns
///
/// A temporary directory with the copied repository.
fn copy_repository<P>(repo_path: P) -> Result<tempfile::TempDir, String>
where
    P: AsRef<Path>,
{
    let temp_dir: tempfile::TempDir = tempdir().map_err(|io_err| {
        format!(
            "Could not create a temporary directory:\n{}",
            io_err.to_string()
        )
    })?;
    println!("Created a temporary directory at {:?}", temp_dir.path());
    copy_content(repo_path.as_ref(), temp_dir.path()).map_err(|fs_err| {
        format!(
            "Could not copy the repository {} to {}:\n{}",
            repo_path.as_ref().display(),
            temp_dir.path().display(),
            fs_err.to_string()
        )
    })?;
    println!(
        "Copied the repo at {} to the temporary directory.",
        repo_path.as_ref().display()
    );
    return Ok(temp_dir);
}

fn is_index_status(s: &Status) -> bool {
    let index_status: Status = Status::INDEX_NEW
        | Status::INDEX_DELETED
        | Status::INDEX_MODIFIED
        | Status::INDEX_RENAMED
        | Status::INDEX_TYPECHANGE;
    s.intersects(index_status)
}

fn is_index_empty(repo: &Repository) -> Result<bool, String> {
    let statuses = repo
        .statuses(None)
        .map_err(|e| format!("Could not fetch file statuses: {}", e))?;
    for status in statuses.into_iter() {
        if is_index_status(&status.status()) {
            return Ok(false);
        }
    }
    return Ok(true);
}

fn push_wallet_marks<P>(repo_path: P) -> Result<(), String>
where
    P: AsRef<Path>,
{
    let repo = Repository::open(repo_path.as_ref()).map_err(|e| {
        format!(
            "Failed to open a repository, {}: {}",
            repo_path.as_ref().display(),
            e
        )
    })?;

    if !is_index_empty(&repo)? {
        return Err(String::from("The repository’s index is not empty. There’s possibly a manual change ongoing so we’re aborting."));
    }

    let statuses = repo.statuses(None).unwrap();
    // TODO: If there are no changes to the mark files, exit early.
    // TODO: If the changes do something else, then changing content, exit early. Possibly report.

    statuses
        .into_iter()
        .for_each(|s| println!("{:?}, {:?}", s.path(), s.status()));
    println!("Hello, world!");
    return Ok(());
}

fn main() {
    let temp_dir: tempfile::TempDir = copy_repository(REPO_PATH).unwrap();
    push_wallet_marks(temp_dir.path()).unwrap();
}
