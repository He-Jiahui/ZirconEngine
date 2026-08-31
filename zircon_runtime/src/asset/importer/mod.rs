mod contract;
mod environment_ibl;
mod error;
mod gltf_geometry;
mod gltf_material_extensions;
mod gltf_texture_semantics;
mod gltf_texture_subassets;
mod gltf_texture_transform;
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
    EnvironmentIblSourceStagingError, EnvironmentIblSourceStagingOutput,
    EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingRestore,
    EnvironmentIblSourceStagingStatus, EnvironmentIblSourceStagingTiming,
    environment_ibl_request_for_dimensions, restore_environment_ibl_source_if_current,
    stage_environment_ibl_source, stage_environment_ibl_source_with_parallel_executor,
    stage_environment_ibl_source_with_parallel_executor_and_decoded_image,
    stage_external_source_cubemap_texture, stage_source_cubemap_texture,
};
pub(crate) use environment_ibl::{
    prepare_environment_ibl_source, prepare_environment_ibl_source_with_parallel_executor,
    prepare_external_source_cubemap_texture, prepare_source_cubemap_texture,
};
pub use error::AssetImportError;
pub use gltf_geometry::{
    ensure_gltf_tangent_uv_attribute_present, gltf_normal_texture_tangent_uv_attribute,
    gltf_tangent_uv_attribute, remap_gltf_morph_targets_for_flat_normals,
    resolve_gltf_normal_texture_tangent_uv_attribute,
};
pub use gltf_material_extensions::{
    GltfClearcoatNormalTextureProjection, gltf_clearcoat_normal_texture_projection,
    project_gltf_material_extensions, validate_required_gltf_material_extension_support,
};
pub use gltf_texture_semantics::{
    GltfTextureColorSpace, GltfTextureUsage, GltfTextureVariant, gltf_texture_color_space_usages,
    gltf_texture_label, gltf_texture_variant,
};
pub use gltf_texture_subassets::{
    add_gltf_texture_subassets, validate_gltf_texture_import_support,
};
pub use gltf_texture_transform::{GltfTextureTransformProjection, project_gltf_texture_transform};
pub use image_decode::{
    DecodedTextureImage, DecodedTextureImageRgba32F, decode_texture_source_image,
    decode_texture_source_image_rgba32f,
};
pub(crate) use image_decode::{
    TextureSourceImageMetadata, decode_texture_source_image_metadata,
    texture_source_image_format_identity,
};
pub use ingest::{
    AssetImporter, IndexedMeshMissingNormalPolicy, IndexedMeshSource, backfill_mesh_sdf_for_model,
    backfill_virtual_geometry_for_model, cook_mesh_asset_derived_data,
    project_indexed_mesh_primitive,
};
pub use native::{
    NativeAssetImportCommandHost, NativeAssetImportCommandReport, NativeAssetImportCommandStatus,
    NativeAssetImportEntryMetadata, NativeAssetImportRequestMetadata,
    NativeAssetImportResponseMetadata, NativeAssetImporterHandler,
};
pub use registry::{AssetImporterRegistry, AssetImporterRegistryError};
pub use schema::{AssetSchemaMigrator, StaticAssetSchemaMigrator};

pub(crate) use contract::{normalize_extension, normalize_full_suffix};
