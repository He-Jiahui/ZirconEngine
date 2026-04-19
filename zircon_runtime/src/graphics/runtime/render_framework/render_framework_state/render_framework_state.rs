use std::collections::HashMap;

use crate::core::framework::render::{RenderPipelineHandle, RenderStats, RenderViewportHandle};

use crate::{RenderPipelineAsset, SceneRenderer};

use super::super::viewport_record::ViewportRecord;

pub(in crate::graphics::runtime::render_framework) struct RenderFrameworkState {
    pub(in crate::graphics::runtime::render_framework) renderer: SceneRenderer,
    pub(in crate::graphics::runtime::render_framework) next_viewport_id: u64,
    pub(in crate::graphics::runtime::render_framework) next_history_id: u64,
    pub(in crate::graphics::runtime::render_framework) pipelines:
        HashMap<RenderPipelineHandle, RenderPipelineAsset>,
    pub(in crate::graphics::runtime::render_framework) viewports:
        HashMap<RenderViewportHandle, ViewportRecord>,
    pub(in crate::graphics::runtime::render_framework) stats: RenderStats,
}
