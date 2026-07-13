use std::fs;
use std::path::Path;

use zircon_runtime_interface::project::PROJECT_MANIFEST_FORMAT_VERSION;

use super::{ProjectManifest, ProjectManifestError};

impl ProjectManifest {
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ProjectManifestError> {
        self.validate()?;
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|source| ProjectManifestError::Write { source })?;
            }
        }
        let mut current = self.clone();
        current.format_version = PROJECT_MANIFEST_FORMAT_VERSION;
        let document = toml::to_string_pretty(&current)
            .map_err(|source| ProjectManifestError::Encode { source })?;
        fs::write(path, document).map_err(|source| ProjectManifestError::Write { source })
    }
}
