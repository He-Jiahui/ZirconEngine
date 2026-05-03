use std::fmt::Debug;

use crate::asset::ModelAsset;
use crate::core::framework::render::{RenderMeshSnapshot, RenderVirtualGeometryDebugState};
use crate::core::resource::ResourceId;

use super::{VirtualGeometryRuntimeExtractOutput, VirtualGeometryRuntimeState};

pub trait VirtualGeometryRuntimeProvider: Debug + Send + Sync {
    fn create_state(&self) -> Box<dyn VirtualGeometryRuntimeState>;

    fn build_extract_from_meshes(
        &self,
        _meshes: &[RenderMeshSnapshot],
        _debug: Option<RenderVirtualGeometryDebugState>,
        _load_model: &mut dyn FnMut(ResourceId) -> Option<ModelAsset>,
    ) -> Option<VirtualGeometryRuntimeExtractOutput> {
        None
    }
}
