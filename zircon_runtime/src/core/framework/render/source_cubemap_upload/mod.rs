#[path = "artifact.rs"]
mod artifact;
#[path = "build.rs"]
mod build;

#[cfg(test)]
#[path = "../tests/source_cubemap_upload_cases.rs"]
mod tests;

pub use artifact::{SourceCubemapUploadArtifact, SourceCubemapUploadMip};
pub use build::build_source_cubemap_upload_artifact;
