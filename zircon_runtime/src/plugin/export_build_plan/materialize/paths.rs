use std::io::{Error, ErrorKind};
use std::path::{Component, Path, PathBuf};

pub(super) fn resolve_materialized_relative_path(
    root: &Path,
    relative_path: &str,
) -> Result<PathBuf, std::io::Error> {
    let portable_path = validated_materialized_relative_path(relative_path)?;
    Ok(root.join(portable_path))
}

pub(super) fn validated_materialized_relative_path(
    relative_path: &str,
) -> Result<String, std::io::Error> {
    if relative_path.is_empty() {
        return Err(invalid_materialized_path(relative_path, "path is empty"));
    }
    if relative_path.contains('\\') {
        return Err(invalid_materialized_path(
            relative_path,
            "backslash separators are not portable export paths",
        ));
    }
    if relative_path.ends_with('/') {
        return Err(invalid_materialized_path(
            relative_path,
            "path cannot end with a separator",
        ));
    }

    let mut normalized = Vec::new();
    let mut saw_component = false;
    for component in Path::new(relative_path).components() {
        match component {
            Component::Normal(component) => {
                saw_component = true;
                let Some(component) = component.to_str() else {
                    return Err(invalid_materialized_path(
                        relative_path,
                        "path components must be UTF-8",
                    ));
                };
                normalized.push(component);
            }
            Component::CurDir => {
                return Err(invalid_materialized_path(
                    relative_path,
                    "current-directory components are not allowed",
                ));
            }
            Component::ParentDir => {
                return Err(invalid_materialized_path(
                    relative_path,
                    "parent-directory components are not allowed",
                ));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(invalid_materialized_path(
                    relative_path,
                    "absolute paths are not allowed",
                ));
            }
        }
    }

    if !saw_component {
        return Err(invalid_materialized_path(
            relative_path,
            "path has no file components",
        ));
    }

    Ok(normalized.join("/"))
}

fn invalid_materialized_path(relative_path: &str, reason: &str) -> std::io::Error {
    Error::new(
        ErrorKind::InvalidInput,
        format!("generated export file path {relative_path:?} is invalid: {reason}"),
    )
}
