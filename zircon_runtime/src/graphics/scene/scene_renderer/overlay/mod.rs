//! 汇总视口场景内容与辅助 pass 的准备、录制接口，供直接场景路径与 compiled graph 共用。
mod icon_source;
mod icons;
mod passes;
mod prepared;
mod viewport_overlay_renderer;

pub(crate) use icon_source::EmptyViewportIconSource;
pub(crate) use icon_source::ViewportIconSource;
pub(crate) use icons::ViewportIconAtlas;
pub(crate) use passes::begin_line_pass_for_region;
#[cfg(test)]
pub(crate) use passes::PASS_ORDER;
pub(crate) use passes::{
    BaseScenePass, GridPass, HandlePass, PreviewSkyPass, SceneGizmoPass, SelectionOutlinePass,
    WireframePass,
};
pub(crate) use prepared::{PreparedIconDraw, PreparedOverlayBuffers, PreparedSceneGizmoPass};
pub(crate) use viewport_overlay_renderer::ViewportOverlayRenderer;
