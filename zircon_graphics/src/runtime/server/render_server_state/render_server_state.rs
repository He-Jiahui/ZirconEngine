use std::collections::HashMap;

use zircon_render_server::{RenderPipelineHandle, RenderStats, RenderViewportHandle};

use crate::{RenderPipelineAsset, SceneRenderer};

use super::super::viewport_record::ViewportRecord;

pub(in crate::runtime::server) struct RenderServerState {
    pub(in crate::runtime::server) renderer: SceneRenderer,
    pub(in crate::runtime::server) next_viewport_id: u64,
    pub(in crate::runtime::server) next_history_id: u64,
    pub(in crate::runtime::server) pipelines: HashMap<RenderPipelineHandle, RenderPipelineAsset>,
    pub(in crate::runtime::server) viewports: HashMap<RenderViewportHandle, ViewportRecord>,
    pub(in crate::runtime::server) stats: RenderStats,
}
