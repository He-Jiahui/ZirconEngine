mod history;
mod hybrid_gi;
mod offline_bake;
mod render_framework;
mod virtual_geometry;

pub(crate) use history::ViewportFrameHistory;
#[cfg(test)]
pub(crate) use hybrid_gi::HybridGiProbeResidencyState;
#[cfg(test)]
pub(crate) use hybrid_gi::HybridGiProbeUpdateRequest;
pub(crate) use hybrid_gi::HybridGiRuntimeState;
#[cfg(test)]
pub(crate) use hybrid_gi::{HybridGiInputSet, HybridGiSceneRepresentation};
pub use offline_bake::{offline_bake_frame, OfflineBakeOutput, OfflineBakeSettings};
pub use render_framework::WgpuRenderFramework;
pub(crate) use virtual_geometry::VirtualGeometryRuntimeState;
#[cfg(test)]
pub(crate) use virtual_geometry::{
    build_virtual_geometry_automatic_extract, build_virtual_geometry_automatic_extract_from_meshes,
    resolve_virtual_geometry_extract, VirtualGeometryAutomaticExtractInstance,
    VirtualGeometryCpuReferenceConfig, VirtualGeometryCpuReferenceFrame,
    VirtualGeometryDebugConfig, VirtualGeometryExecutionMode,
};
pub(crate) use virtual_geometry::{
    build_virtual_geometry_automatic_extract_from_meshes_with_debug,
    VirtualGeometryAutomaticExtractOutput,
};
#[cfg(test)]
pub(crate) use virtual_geometry::{VirtualGeometryPageRequest, VirtualGeometryPageResidencyState};
