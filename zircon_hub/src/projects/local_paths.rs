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
        Err(error) => {
            let _ = fs::remove_dir_all(created_dir);
            Err(error)
        }
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
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn missing_child_under_existing_root_is_rejected_without_creating() {
        let root = temp_dir("local-path-root");
        let missing_child = root.join("missing").join("child");

        let result = reject_inside_root(&root, &missing_child, HubMessage::legacy("inside"));

        assert!(result.is_err());
        assert!(!missing_child.exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn create_owned_dir_rejects_existing_directory_with_caller_message() {
        let root = temp_dir("local-path-owned-existing");
        let existing = root.join("existing");
        fs::create_dir(&existing).unwrap();

        let error = create_owned_dir(&existing, || HubMessage::legacy("custom already exists"))
            .unwrap_err();

        assert_eq!(error.to_string(), "custom already exists");
        assert!(existing.is_dir());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cleanup_dir_on_error_removes_dir_only_on_error() {
        let root = temp_dir("local-path-cleanup");
        let failed_dir = root.join("failed");
        fs::create_dir(&failed_dir).unwrap();
        fs::write(failed_dir.join("partial.txt"), "partial").unwrap();

        let error = cleanup_dir_on_error::<()>(&failed_dir, Err(HubError::message("copy failed")))
            .unwrap_err();

        assert_eq!(error.to_string(), "copy failed");
        assert!(!failed_dir.exists());

        let successful_dir = root.join("successful");
        fs::create_dir(&successful_dir).unwrap();
        let value = cleanup_dir_on_error(&successful_dir, Ok(42)).unwrap();

        assert_eq!(value, 42);
        assert!(successful_dir.is_dir());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sibling_with_shared_prefix_is_not_inside_root() {
        let parent = temp_dir("local-path-parent");
        let root = parent.join("Game");
        let sibling = parent.join("GameBuild").join("out");
        fs::create_dir_all(&root).unwrap();

        reject_inside_root(&root, &sibling, HubMessage::legacy("inside")).unwrap();

        fs::remove_dir_all(parent).unwrap();
    }

    fn temp_dir(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-{label}-{}",
            crate::projects::now_unix_ms()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }
}
