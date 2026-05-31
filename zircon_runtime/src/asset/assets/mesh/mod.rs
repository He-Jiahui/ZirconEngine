mod attribute;
mod constants;
mod indices;
mod mesh_asset;
mod metadata;
mod normals;
mod tangents;
mod usage;
mod validation;
mod zmesh_document;

pub use attribute::{
    MeshAttributeFormat, MeshAttributeSummary, MeshAttributeValues, MeshMorphTargetAttributeSummary,
};
pub use constants::{
    MESH_ATTRIBUTE_COLOR, MESH_ATTRIBUTE_JOINT_INDEX, MESH_ATTRIBUTE_JOINT_WEIGHT,
    MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT, MESH_ATTRIBUTE_UV0,
};
pub use indices::{MeshIndexFormat, MeshIndices};
pub use mesh_asset::{
    MeshAsset, MeshAssetManagementRecord, MeshAssetManagementRecordFailure,
    MeshAssetManagementRecordSet, MeshAssetManagementRecordSetSummary, MeshAssetOverview,
};
pub use metadata::{MeshMorphTargetAsset, MeshSkinAsset};
pub use usage::MeshAssetUsage;
pub use validation::MeshValidationError;
pub use zmesh_document::{ZMeshDocument, ZMESH_DOCUMENT_VERSION};
