use std::path::Path;

use zircon_runtime_interface::project::PROJECT_MANIFEST_FORMAT_VERSION;

use crate::core::resource::io::{atomic_write_with_fault, AtomicWriteFault};

use super::{ProjectManifest, ProjectManifestError};

impl ProjectManifest {
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ProjectManifestError> {
        self.save_with_atomic_fault(path, AtomicWriteFault::None)
    }

    pub(crate) fn save_with_atomic_fault(
        &self,
        path: impl AsRef<Path>,
        fault: AtomicWriteFault,
    ) -> Result<(), ProjectManifestError> {
        self.validate()?;
        let path = path.as_ref();
        let mut current = self.clone();
        current.format_version = PROJECT_MANIFEST_FORMAT_VERSION;
        let document = toml::to_string_pretty(&current)
            .map_err(|source| ProjectManifestError::Encode { source })?;
        atomic_write_with_fault(path, document.as_bytes(), fault)
            .map_err(|source| ProjectManifestError::Write { source })
    }
}
