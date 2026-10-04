//! 所有生成路径入导出根目录前必须是可移植相对路径；落盘和 ZIP 使用同一归一化规则。
use std::io::{Error, ErrorKind};
use std::path::{Component, Path, PathBuf};

pub(super) fn resolve_materialized_relative_path(
    root: &Path,
    relative_path: &str,
) -> Result<PathBuf, std::io::Error> {
    let portable_path = validated_materialized_relative_path(relative_path)?;
    Ok(root.join(portable_path))
}

/// 生成文件与 ZIP 条目的共同边界；调用者不得把外部绝对路径当作导出内路径。
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
#[path = "tests/paths.rs"]
mod tests;
