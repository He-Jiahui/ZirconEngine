use std::collections::HashMap;

use crate::core::framework::render::{
    RenderPipelineHandle, RenderStats, RenderViewportHandle, RenderVirtualGeometryDebugSnapshot,
};

use crate::{RenderPipelineAsset, SceneRenderer, VirtualGeometryRuntimeProviderRegistration};

use super::super::viewport_record::ViewportRecord;

pub(in crate::graphics::runtime::render_framework) struct RenderFrameworkState {
    pub(in crate::graphics::runtime::render_framework) renderer: SceneRenderer,
    pub(in crate::graphics::runtime::render_framework) next_viewport_id: u64,
    pub(in crate::graphics::runtime::render_framework) next_history_id: u64,
    pub(in crate::graphics::runtime::render_framework) pipelines:
        HashMap<RenderPipelineHandle, RenderPipelineAsset>,
    pub(in crate::graphics::runtime::render_framework) virtual_geometry_runtime_provider:
        Option<VirtualGeometryRuntimeProviderRegistration>,
    pub(in crate::graphics::runtime::render_framework) last_virtual_geometry_debug_snapshot:
        Option<RenderVirtualGeometryDebugSnapshot>,
    pub(in crate::graphics::runtime::render_framework) viewports:
        HashMap<RenderViewportHandle, ViewportRecord>,
    pub(in crate::graphics::runtime::render_framework) stats: RenderStats,
}
