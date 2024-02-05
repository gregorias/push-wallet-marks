use std::path::Path;
use std::path::PathBuf;

use git2::Index;
use git2::Repository;
use git2::Status;
use git2::StatusEntry;
use git2::Statuses;
use tempfile::tempdir;

const MARK_FILES: [&str; 2] = [
    "updates/findata-funnel-pull-gpayslip-success-mark",
    "updates/findata-funnel-success-mark",
];
const REPO_PATH: &str = "/Users/grzesiek/wallet";

/// A modification of git2::StatusEntry that owns its path.
///
/// Owning the path gives us a saner interface for working with the path without
/// checking the Option every time.
struct StatusEntryBetter {
    pub path: PathBuf,
    pub status: Status,
}

impl StatusEntryBetter {
    fn from_status_entry(status_entry: &StatusEntry) -> Option<Self> {
        let path: &str = status_entry.path()?;
        Some(StatusEntryBetter {
            path: PathBuf::from(path),
            status: status_entry.status(),
        })
    }
}

/// Copies the content of one directory P to another.
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

fn is_index_empty(statuses: &Statuses) -> Result<bool, String> {
    for status in statuses.into_iter() {
        if is_index_status(&status.status()) {
            return Ok(false);
        }
    }
    return Ok(true);
}

fn filter_statuses_by_path<'a>(
    statuses: &'a Statuses<'a>,
    mark_files: &[&str],
) -> Vec<StatusEntry<'a>> {
    statuses
        .into_iter()
        .filter(|status_entry: &StatusEntry| -> bool {
            for mark_file in mark_files {
                if *mark_file == status_entry.path().unwrap_or("") {
                    return true;
                }
            }
            return false;
        })
        .collect()
}

/// Stages and pushes mark files in the wallet repository upstream.
///
/// # Arguments
///
/// * `repo_path` - The wallet repository path.
/// * `mark_files` - The mark files to potentially push.
fn push_wallet_marks<P>(repo_path: P, mark_files: &[&str]) -> Result<(), String>
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

    let statuses: Statuses = repo
        .statuses(None)
        .map_err(|e| format!("Could not fetch file statuses: {}", e))?;

    let mut index: Index = repo
        .index()
        .map_err(|e| format!("Could not fetch the index: {}", e))?;

    if !is_index_empty(&statuses)? {
        println!("The repository’s index is not empty. There’s possibly a manual change ongoing so we’re aborting the push.");
        return Ok(());
    }

    let mark_file_statuses: Vec<StatusEntry> = filter_statuses_by_path(&statuses, mark_files);
    let mark_file_statuses: Vec<StatusEntryBetter> = mark_file_statuses
        .iter()
        .map(StatusEntryBetter::from_status_entry)
        .collect::<Option<Vec<StatusEntryBetter>>>()
        .map_or(Err("Could not convert all mark files to a path."), Ok)?;

    if mark_file_statuses.is_empty() {
        println!("No mark files to push.");
        return Ok(());
    }

    for mark_file_status in &mark_file_statuses {
        if mark_file_status.status == Status::WT_MODIFIED {
            index
                .add_path(mark_file_status.path.as_path())
                .map_err(|e| {
                    format!(
                        "Could not add {} to the index: {}",
                        mark_file_status.path.display(),
                        e
                    )
                })?;
        } else {
            return Err(format!(
                "The mark file {} has an unexpected status: {:?}.",
                mark_file_status.path.display(),
                mark_file_status.status
            ));
        }
    }
    // NOTE: Let’s see.

    // TODO: If we commit & push, what happens to the original repository?
    // Ideally, I shouldn’t have to pull and resolve conflicts manually.

    mark_file_statuses
        .into_iter()
        .for_each(|s| println!("{:?}, {:?}", s.path, s.status));
    println!("Hello, world!");
    return Ok(());
}

fn main() -> Result<(), String> {
    let temp_dir: tempfile::TempDir = copy_repository(REPO_PATH)?;
    push_wallet_marks(temp_dir.path(), &MARK_FILES)?;
    return Ok(());
}
