use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use crate::error::HubError;
use crate::state::HubMessage;

pub(super) fn reject_inside_root(
    protected_root: &Path,
    candidate: &Path,
    message: HubMessage,
) -> Result<(), HubError> {
    if path_is_inside_root(protected_root, candidate)? {
        Err(HubError::status(message, None))
    } else {
        Ok(())
    }
}

pub(super) fn create_owned_dir(
    path: &Path,
    already_exists_message: impl FnOnce() -> HubMessage,
) -> Result<(), HubError> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            Err(HubError::status(already_exists_message(), None))
        }
        Err(error) => Err(error.into()),
    }
}

pub(super) fn cleanup_dir_on_error<T>(
    created_dir: &Path,
    result: Result<T, HubError>,
) -> Result<T, HubError> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => match remove_owned_dir(created_dir) {
            Ok(()) => Err(error),
            Err(cleanup_error) => Err(HubError::message(format!("{error}; {cleanup_error}"))),
        },
    }
}

pub(super) fn remove_owned_dir(path: &Path) -> Result<(), HubError> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(HubError::message(format!(
            "failed to remove owned output `{}`: {error}",
            path.display()
        ))),
    }
}

fn path_is_inside_root(protected_root: &Path, candidate: &Path) -> Result<bool, HubError> {
    let protected_root = protected_root.canonicalize()?;
    for ancestor in candidate.ancestors() {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        if let Ok(resolved_ancestor) = ancestor.canonicalize() {
            if is_same_or_child(&resolved_ancestor, &protected_root) {
                return Ok(true);
            }
        }
    }

    let resolved_candidate = resolve_without_creating(candidate)?;
    Ok(is_same_or_child(&resolved_candidate, &protected_root))
}

fn resolve_without_creating(path: &Path) -> Result<PathBuf, HubError> {
    for ancestor in path.ancestors() {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        if let Ok(resolved_ancestor) = ancestor.canonicalize() {
            let suffix = path
                .strip_prefix(ancestor)
                .unwrap_or_else(|_| Path::new(""));
            return Ok(normalize_lexically(&resolved_ancestor.join(suffix)));
        }
    }

    Ok(normalize_lexically(&std::env::current_dir()?.join(path)))
}

fn is_same_or_child(path: &Path, protected_root: &Path) -> bool {
    path == protected_root || path.starts_with(protected_root)
}

fn normalize_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}

#[cfg(test)]
#[path = "tests/local_paths.rs"]
mod tests;
