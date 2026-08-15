use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::recovery::AutosaveError;

/// A source-document location persisted with an autosave snapshot.
///
/// Recovery records only project-relative paths, so replay cannot be redirected
/// outside the project by an autosave directory entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutosaveSourcePath(PathBuf);

impl AutosaveSourcePath {
    pub fn parse(value: impl Into<PathBuf>) -> Result<Self, AutosaveError> {
        let path = value.into();
        let is_project_relative = !path.as_os_str().is_empty()
            && !path.is_absolute()
            && path.to_str().is_some()
            && path
                .components()
                .all(|component| matches!(component, Component::Normal(_)));
        if !is_project_relative {
            return Err(AutosaveError::InvalidRecoverySourcePath { path });
        }
        let normalized = path
            .components()
            .map(|component| {
                component
                    .as_os_str()
                    .to_str()
                    .expect("validated autosave source components are UTF-8")
            })
            .collect::<Vec<_>>()
            .join("/");
        Ok(Self(PathBuf::from(normalized)))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::AutosaveSourcePath;

    #[cfg(windows)]
    #[test]
    fn source_path_normalizes_windows_separators_to_the_project_form() {
        let source = AutosaveSourcePath::parse(r"assets\ui\panel.zui").unwrap();

        assert_eq!(source.as_path(), Path::new("assets/ui/panel.zui"));
    }
}
