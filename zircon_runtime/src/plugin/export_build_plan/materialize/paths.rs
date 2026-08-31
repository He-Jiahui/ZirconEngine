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

    let mut normalized = String::with_capacity(relative_path.len());
    let mut saw_component = false;
    for component in Path::new(relative_path).components() {
        match component {
            Component::Normal(component) => {
                let Some(component) = component.to_str() else {
                    return Err(invalid_materialized_path(
                        relative_path,
                        "path components must be UTF-8",
                    ));
                };
                if saw_component {
                    normalized.push('/');
                }
                normalized.push_str(component);
                saw_component = true;
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

    Ok(normalized)
}

fn invalid_materialized_path(relative_path: &str, reason: &str) -> std::io::Error {
    Error::new(
        ErrorKind::InvalidInput,
        format!("generated export file path {relative_path:?} is invalid: {reason}"),
    )
}

#[cfg(test)]
mod tests {
    use super::validated_materialized_relative_path;

    #[test]
    fn streaming_normalization_preserves_portable_paths() {
        assert_eq!(
            validated_materialized_relative_path("plugins/rendering/plugin.toml")
                .expect("portable path should remain valid"),
            "plugins/rendering/plugin.toml"
        );
        assert_eq!(
            validated_materialized_relative_path("plugins//rendering///plugin.toml")
                .expect("repeated separators should normalize"),
            "plugins/rendering/plugin.toml"
        );
    }

    #[test]
    fn streaming_normalization_preserves_path_rejections() {
        for path in [
            "",
            "./plugin.toml",
            "../plugin.toml",
            "plugins\\plugin.toml",
            "plugins/",
        ] {
            assert!(
                validated_materialized_relative_path(path).is_err(),
                "path {path:?} should remain invalid"
            );
        }
    }
}
