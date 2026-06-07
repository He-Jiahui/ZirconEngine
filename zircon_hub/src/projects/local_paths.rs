use std::path::{Component, Path, PathBuf};

use crate::error::HubError;

pub(super) fn reject_inside_root(
    protected_root: &Path,
    candidate: &Path,
    message: &'static str,
) -> Result<(), HubError> {
    if path_is_inside_root(protected_root, candidate)? {
        Err(HubError::message(message))
    } else {
        Ok(())
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

        let result = reject_inside_root(&root, &missing_child, "inside");

        assert!(result.is_err());
        assert!(!missing_child.exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sibling_with_shared_prefix_is_not_inside_root() {
        let parent = temp_dir("local-path-parent");
        let root = parent.join("Game");
        let sibling = parent.join("GameBuild").join("out");
        fs::create_dir_all(&root).unwrap();

        reject_inside_root(&root, &sibling, "inside").unwrap();

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
