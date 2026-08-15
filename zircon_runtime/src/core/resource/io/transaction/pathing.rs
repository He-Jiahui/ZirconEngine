use std::cmp::Ordering as CmpOrdering;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TRANSACTION_ID: AtomicU64 = AtomicU64::new(1);

/// A physical operation path with platform-native ordering semantics.
///
/// Durable journals retain this path, not a caller-provided alias. The final directory entry is
/// preserved when it is a symlink so the transaction's existing non-link checks cannot be bypassed.
#[derive(Clone, Debug)]
pub(super) struct PathIdentity {
    operation_path: PathBuf,
    #[cfg(windows)]
    comparison_key: Vec<u16>,
}

impl PathIdentity {
    pub(super) fn resolve(path: &Path) -> io::Result<Self> {
        let operation_path = resolve_operation_path(path)?;
        #[cfg(windows)]
        let comparison_key = {
            use std::os::windows::ffi::OsStrExt;

            let key = operation_path.as_os_str().encode_wide().collect::<Vec<_>>();
            i32::try_from(key.len()).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "transaction path exceeds the Windows comparison limit",
                )
            })?;
            key
        };
        Ok(Self {
            operation_path,
            #[cfg(windows)]
            comparison_key,
        })
    }

    pub(super) fn operation_path(&self) -> &Path {
        &self.operation_path
    }

    pub(super) fn into_operation_path(self) -> PathBuf {
        self.operation_path
    }
}

impl PartialEq for PathIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == CmpOrdering::Equal
    }
}

impl Eq for PathIdentity {}

impl PartialOrd for PathIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathIdentity {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        #[cfg(windows)]
        {
            return compare_windows_paths(&self.comparison_key, &other.comparison_key);
        }
        #[cfg(not(windows))]
        self.operation_path.cmp(&other.operation_path)
    }
}

pub(super) fn next_transaction_id() -> String {
    let id = NEXT_TRANSACTION_ID.fetch_add(1, Ordering::Relaxed);
    format!("{}-{id}", std::process::id())
}

pub(super) fn valid_transaction_id(value: &str) -> bool {
    let mut parts = value.split('-');
    parts.next().is_some_and(|part| part.parse::<u32>().is_ok())
        && parts.next().is_some_and(|part| part.parse::<u64>().is_ok())
        && parts.next().is_none()
}

pub(super) fn valid_tag(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

pub(super) fn transaction_sibling(
    owner: &Path,
    tag: &str,
    role: &str,
    transaction_id: &str,
) -> PathBuf {
    let parent = owner.parent().unwrap_or_else(|| Path::new("."));
    let name = owner
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("zircon.data");
    parent.join(format!(".{name}.zr-{tag}-{role}-{transaction_id}"))
}

pub(super) fn journal_path(
    directory: &Path,
    first_target: &Path,
    tag: &str,
    transaction_id: &str,
) -> PathBuf {
    let name = first_target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("zircon.data");
    directory.join(format!(
        ".{name}.zr-{tag}-journal-{transaction_id}.zrjournal"
    ))
}

fn resolve_operation_path(path: &Path) -> io::Result<PathBuf> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            let parent = path
                .parent()
                .ok_or_else(|| invalid_path("path has no parent"))?;
            let name = path
                .file_name()
                .ok_or_else(|| invalid_path("path has no final directory entry"))?;
            fs::canonicalize(parent).map(|parent| parent.join(name))
        }
        Ok(_) => fs::canonicalize(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            resolve_path_below_existing_ancestor(path)
        }
        Err(error) => Err(error),
    }
}

fn resolve_path_below_existing_ancestor(path: &Path) -> io::Result<PathBuf> {
    let (existing, tail) = split_at_deepest_existing_ancestor(path)?;
    let mut resolved = fs::canonicalize(existing)?;
    for component in tail {
        append_uncreated_component(&mut resolved, component);
    }
    Ok(resolved)
}

fn split_at_deepest_existing_ancestor(path: &Path) -> io::Result<(PathBuf, Vec<OsString>)> {
    let mut candidate = PathBuf::new();
    let mut deepest_existing = None;
    let mut unresolved_tail = Vec::new();

    for component in path.components() {
        candidate.push(component.as_os_str());
        match fs::metadata(&candidate) {
            Ok(_) => {
                deepest_existing = Some(candidate.clone());
                unresolved_tail.clear();
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                unresolved_tail.push(component.as_os_str().to_os_string());
            }
            Err(error) => return Err(error),
        }
    }
    deepest_existing
        .map(|existing| (existing, unresolved_tail))
        .ok_or_else(|| invalid_path("path has no existing physical ancestor"))
}

fn append_uncreated_component(path: &mut PathBuf, component: OsString) {
    if component == "." {
        return;
    }
    if component == ".." {
        let _ = path.pop();
        return;
    }
    path.push(component);
}

fn invalid_path(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

#[cfg(windows)]
fn compare_windows_paths(left: &[u16], right: &[u16]) -> CmpOrdering {
    const CSTR_LESS_THAN: i32 = 1;
    const CSTR_EQUAL: i32 = 2;
    const CSTR_GREATER_THAN: i32 = 3;

    let left_length = i32::try_from(left.len()).expect("validated Windows path length");
    let right_length = i32::try_from(right.len()).expect("validated Windows path length");
    let comparison = unsafe {
        CompareStringOrdinal(left.as_ptr(), left_length, right.as_ptr(), right_length, 1)
    };
    match comparison {
        CSTR_LESS_THAN => CmpOrdering::Less,
        CSTR_EQUAL => CmpOrdering::Equal,
        CSTR_GREATER_THAN => CmpOrdering::Greater,
        _ => left.cmp(right),
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CompareStringOrdinal(
        left: *const u16,
        left_length: i32,
        right: *const u16,
        right_length: i32,
        ignore_case: i32,
    ) -> i32;
}
