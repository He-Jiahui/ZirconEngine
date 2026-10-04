use std::fmt::Debug;

use crate::asset::ModelAsset;
use crate::core::framework::render::{RenderMeshSnapshot, RenderVirtualGeometryDebugState};
use crate::core::resource::ResourceId;

use super::{VirtualGeometryRuntimeExtractOutput, VirtualGeometryRuntimeState};

/// 虚拟几何扩展入口；可在没有 authored extract 时从网格资产构造自动提取结果。
/// `build_extract_from_meshes` 返回 None 表示未提供自动内容；启用该特性时仍需已注册的运行时状态 provider。
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
