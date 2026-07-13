mod contract;
mod environment_ibl;
mod error;
mod image_decode;
mod ingest;
mod native;
mod registry;
mod schema;

pub use contract::{
    AssetImportContext, AssetImportOutcome, AssetImporterCapabilityReport,
    AssetImporterCapabilityStatus, AssetImporterDescriptor, AssetImporterHandler,
    AssetSchemaMigrationReport, DiagnosticOnlyAssetImporter, FunctionAssetImporter,
    ImportedAssetEntry,
};
pub use environment_ibl::{
    stage_environment_ibl_source, stage_external_source_cubemap_texture,
    EnvironmentIblSourceStagingError, EnvironmentIblSourceStagingReport,
    EnvironmentIblSourceStagingStatus, ENVIRONMENT_IBL_FACE_SIZE_IMPORT_SETTING,
    ENVIRONMENT_IBL_IMPORT_SETTING,
};
pub use error::AssetImportError;
pub use image_decode::{
    decode_texture_source_image, decode_texture_source_image_rgba32f, DecodedTextureImage,
    DecodedTextureImageRgba32F,
};
pub use ingest::AssetImporter;
pub use native::{
    NativeAssetImportCommandHost, NativeAssetImportCommandReport, NativeAssetImportCommandStatus,
    NativeAssetImportEntryMetadata, NativeAssetImportRequestMetadata,
    NativeAssetImportResponseMetadata, NativeAssetImporterHandler,
};
pub use registry::{AssetImporterRegistry, AssetImporterRegistryError};
pub use schema::{AssetSchemaMigrator, StaticAssetSchemaMigrator};

pub(crate) use contract::{normalize_extension, normalize_full_suffix};
