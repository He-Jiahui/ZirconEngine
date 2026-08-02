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
    ENVIRONMENT_IBL_FACE_SIZE_IMPORT_SETTING, ENVIRONMENT_IBL_IMPORT_SETTING,
    EnvironmentIblSourceStagingError, EnvironmentIblSourceStagingReport,
    EnvironmentIblSourceStagingStatus, stage_environment_ibl_source,
    stage_environment_ibl_source_with_parallel_executor,
    stage_environment_ibl_source_with_parallel_executor_and_decoded_image,
    stage_external_source_cubemap_texture,
};
pub use error::AssetImportError;
pub use image_decode::{
    DecodedTextureImage, DecodedTextureImageRgba32F, decode_texture_source_image,
    decode_texture_source_image_rgba32f,
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
