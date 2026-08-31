use zircon_plugin_rendering_reflection_probes_runtime::EncodedReflectionProbeCaptureSource;
use zircon_runtime::asset::{ProjectAssetManager, ProjectGeneratedSourceReceipt};
use zircon_runtime::core::CoreError;

pub fn publish_reflection_probe_capture_source(
    asset_manager: &ProjectAssetManager,
    source: EncodedReflectionProbeCaptureSource,
) -> Result<ProjectGeneratedSourceReceipt, ReflectionProbeCaptureProjectPublicationError> {
    let (output_uri, bytes) = source.into_parts();
    asset_manager
        .publish_generated_project_source(output_uri, bytes)
        .map_err(ReflectionProbeCaptureProjectPublicationError::Project)
}

#[derive(Debug, thiserror::Error)]
pub enum ReflectionProbeCaptureProjectPublicationError {
    #[error("publish captured reflection-probe source through the project asset transaction: {0}")]
    Project(#[source] CoreError),
}

#[cfg(test)]
mod tests {
    #[test]
    fn project_publication_consumes_encoded_source_through_runtime_transaction_owner() {
        let source = include_str!("publication.rs");

        assert!(source.contains("source.into_parts()"));
        assert!(source.contains("publish_generated_project_source(output_uri, bytes)"));
        assert!(!source.contains("std::fs"));
        assert!(!source.contains("ResourceRecord::new"));
    }
}
