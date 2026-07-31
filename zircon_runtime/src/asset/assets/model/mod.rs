mod model_asset;
mod primitive;
mod virtual_geometry;

pub use model_asset::{
    ModelAsset, ModelAssetManagementRecord, ModelAssetManagementRecordSet,
    ModelAssetManagementRecordSetSummary, ModelAssetOverview, ModelPrimitiveOverview,
};
pub use primitive::ModelPrimitiveAsset;
pub(crate) use primitive::{
    VIRTUAL_GEOMETRY_VERTEX_ORDINAL_HIGH_JOINT_SLOT, VIRTUAL_GEOMETRY_VERTEX_ORDINAL_LOW_JOINT_SLOT,
};
pub use virtual_geometry::{
    VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset, VirtualGeometryClusterPageHeaderAsset,
    VirtualGeometryDebugMetadataAsset, VirtualGeometryHierarchyNodeAsset,
    VirtualGeometryPageDependencyAsset, VirtualGeometryRootClusterRangeAsset,
};
