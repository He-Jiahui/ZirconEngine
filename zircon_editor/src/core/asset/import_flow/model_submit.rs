use std::path::PathBuf;

use zircon_runtime::core::CoreError;

use crate::core::jobs::{EditorJobSpec, JobCategory};

use super::diagnostics::EditorModelImportDiagnostics;
use super::job::AssetImportModelJob;
use super::{EditorAssetImportFlow, EditorModelImportTicket};

impl EditorAssetImportFlow {
    /// Submits one Runtime-owned compound model transaction. The editor job observes the Runtime
    /// receipt and does not stage sources, registry writes, or resource mutations itself.
    pub fn submit_model_source(
        &self,
        source_path: PathBuf,
    ) -> Result<EditorModelImportTicket, CoreError> {
        let manager = self.model_manager.as_ref().cloned().ok_or_else(|| {
            CoreError::ServiceUnavailable("EditorAssetImportFlow model backend".to_owned())
        })?;
        let diagnostics = std::sync::Arc::new(EditorModelImportDiagnostics::new(
            source_path.clone(),
            self.diagnostics.clone(),
        ));
        let submission = self.jobs.submit(
            EditorJobSpec::new(
                format!("Import model {}", source_path.display()),
                JobCategory::Import,
            ),
            AssetImportModelJob::new(manager, source_path.clone(), diagnostics.clone()),
        );
        match submission {
            Ok(ticket) => {
                diagnostics.arm();
                Ok(EditorModelImportTicket::new(ticket, source_path))
            }
            Err(error) => {
                let detail = error.to_string();
                diagnostics.reject_submission(&detail);
                Err(CoreError::ServiceUnavailable(detail))
            }
        }
    }
}
