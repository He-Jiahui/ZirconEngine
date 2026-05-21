mod contract;
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
pub use error::AssetImportError;
pub use image_decode::{decode_texture_source_image, DecodedTextureImage};
pub use ingest::AssetImporter;
pub use native::{
    NativeAssetImportEntryMetadata, NativeAssetImportRequestMetadata,
    NativeAssetImportResponseMetadata, NativeAssetImporterHandler,
};
pub use registry::{AssetImporterRegistry, AssetImporterRegistryError};
pub use schema::{AssetSchemaMigrator, StaticAssetSchemaMigrator};

pub(crate) use contract::{normalize_extension, normalize_full_suffix};
