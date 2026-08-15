use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{
    AlphaMode, Asset, AssetImportContext, AssetImportError, AssetImportOutcome,
    AssetImporterDescriptor, AssetKind, AssetLoadState, AssetLoadStates,
    AssetManagementFamilyIssueBucket, AssetManagementFamilyKind, AssetManagementFamilyStatus,
    AssetManagementRecordSets, AssetManager, AssetMetaDocument, AssetReference, AssetSourceUnit,
    AssetUri, AssetUuid, DependencyLoadState, FunctionAssetImporter, ImportedAsset, MaterialAsset,
    MaterialAssetManagementRecordSet, MaterialTextureSlotValue, MeshAsset,
    MeshAssetManagementRecordSet, MeshAttributeValues, ModelAsset, ModelAssetManagementRecordSet,
    ProjectAssetManager, ProjectManager, ProjectManifest, ProjectPaths,
    RecursiveDependencyLoadState, SceneAssetManagementRecordSet, SceneEntityManagementRecordSet,
    ShaderAsset, ShaderAssetManagementRecordSet, TextureAsset, TextureUploadSupport,
    MESH_ATTRIBUTE_POSITION,
};
use crate::core::framework::render::RenderMaterialManagementRecordSet;
use crate::core::resource::ResourceState;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::ResourceStreamer;

mod assertions;
mod end_to_end;
mod fixtures;
mod importers;
