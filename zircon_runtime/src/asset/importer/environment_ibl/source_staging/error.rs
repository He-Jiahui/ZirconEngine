use std::path::PathBuf;

use thiserror::Error;

use crate::asset::artifact::{IblBakeArtifactAssetDerivedError, IblSourceCubemapStagingError};
use crate::asset::assets::{ExternalSourceCubemapContainerError, ExternalSourceCubemapDecodeError};
use crate::asset::importer::AssetImportError;

#[derive(Debug, Error)]
pub enum EnvironmentIblSourceStagingError {
    #[error("decode environment source image: {0}")]
    Decode(#[source] AssetImportError),
    #[error("environment IBL import setting `{key}` is invalid: {reason}")]
    InvalidSetting { key: &'static str, reason: String },
    #[error("environment IBL source must be a 2:1 equirectangular image, found {width}x{height}")]
    InvalidEquirectangularDimensions { width: u32, height: u32 },
    #[error("read environment IBL asset-derived artifact: {0}")]
    ReadAssetDerived(#[source] IblBakeArtifactAssetDerivedError),
    #[error("write environment IBL asset-derived artifact: {0}")]
    WriteAssetDerived(#[source] IblBakeArtifactAssetDerivedError),
    #[error("remove invalid environment IBL asset-derived artifact {path}: {source}")]
    RemoveAssetDerived {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("classify external source cubemap: {0}")]
    ExternalContainer(#[source] ExternalSourceCubemapContainerError),
    #[error("decode external source cubemap: {0}")]
    ExternalDecode(#[source] ExternalSourceCubemapDecodeError),
    #[error("stage environment IBL source bundle: {0}")]
    Stage(#[source] IblSourceCubemapStagingError),
    #[error("inspect staged environment IBL output {path}: {source}")]
    OutputMetadata {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
