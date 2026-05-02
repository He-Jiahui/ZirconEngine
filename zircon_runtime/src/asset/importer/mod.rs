mod contract;
mod error;
mod ingest;
mod native;
mod registry;
mod schema;

pub use contract::{
    AssetImportContext, AssetImportOutcome, AssetImporterDescriptor, AssetImporterHandler,
    AssetSchemaMigrationReport, DiagnosticOnlyAssetImporter, FunctionAssetImporter,
};
pub use error::AssetImportError;
pub use ingest::AssetImporter;
pub use native::{
    NativeAssetImportRequestMetadata, NativeAssetImportResponseMetadata, NativeAssetImporterHandler,
};
pub use registry::{AssetImporterRegistry, AssetImporterRegistryError};
pub use schema::{AssetSchemaMigrator, StaticAssetSchemaMigrator};

pub(crate) use contract::{normalize_extension, normalize_full_suffix};
