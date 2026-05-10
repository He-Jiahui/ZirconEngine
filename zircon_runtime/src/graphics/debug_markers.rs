use crate::graphics::pipeline::RenderPassStage;

pub(crate) const RENDERDOC_MARKER_FRAME_EXTRACT: &str = "zircon::FrameExtract";
pub(crate) const RENDERDOC_MARKER_CLEAR: &str = "zircon::Clear";
pub(crate) const RENDERDOC_MARKER_PREPASS: &str = "zircon::Prepass";
pub(crate) const RENDERDOC_MARKER_MAIN_SCENE: &str = "zircon::MainScene";
pub(crate) const RENDERDOC_MARKER_LIGHTING: &str = "zircon::Lighting";
pub(crate) const RENDERDOC_MARKER_DEFERRED_LIGHTING: &str = "zircon::DeferredLighting";
pub(crate) const RENDERDOC_MARKER_POST_PROCESS: &str = "zircon::PostProcess";
pub(crate) const RENDERDOC_MARKER_HISTORY_COPY: &str = "zircon::HistoryCopy";
pub(crate) const RENDERDOC_MARKER_OVERLAY: &str = "zircon::Overlay";
pub(crate) const RENDERDOC_MARKER_UI: &str = "zircon::UI";
pub(crate) const RENDERDOC_MARKER_READBACK: &str = "zircon::Readback";

#[cfg(test)]
pub(crate) const REQUIRED_RENDERDOC_STAGE_MARKERS: &[&str] = &[
    RENDERDOC_MARKER_FRAME_EXTRACT,
    RENDERDOC_MARKER_CLEAR,
    RENDERDOC_MARKER_PREPASS,
    RENDERDOC_MARKER_MAIN_SCENE,
    RENDERDOC_MARKER_LIGHTING,
    RENDERDOC_MARKER_DEFERRED_LIGHTING,
    RENDERDOC_MARKER_POST_PROCESS,
    RENDERDOC_MARKER_HISTORY_COPY,
    RENDERDOC_MARKER_OVERLAY,
    RENDERDOC_MARKER_UI,
    RENDERDOC_MARKER_READBACK,
];

pub(crate) fn marker_for_render_pass_stage(stage: RenderPassStage) -> Option<&'static str> {
    match stage {
        RenderPassStage::DepthPrepass | RenderPassStage::Shadow => Some(RENDERDOC_MARKER_PREPASS),
        RenderPassStage::Deferred => Some(RENDERDOC_MARKER_DEFERRED_LIGHTING),
        RenderPassStage::Lighting => Some(RENDERDOC_MARKER_LIGHTING),
        RenderPassStage::Opaque2d
        | RenderPassStage::AlphaMask2d
        | RenderPassStage::Transparent2d
        | RenderPassStage::Opaque3d
        | RenderPassStage::AlphaMask3d
        | RenderPassStage::Transparent3d
        | RenderPassStage::Opaque
        | RenderPassStage::Transparent => Some(RENDERDOC_MARKER_MAIN_SCENE),
        RenderPassStage::PostProcess | RenderPassStage::AmbientOcclusion => {
            Some(RENDERDOC_MARKER_POST_PROCESS)
        }
        RenderPassStage::Ui => Some(RENDERDOC_MARKER_UI),
        RenderPassStage::Overlay | RenderPassStage::Debug => Some(RENDERDOC_MARKER_OVERLAY),
    }
}

pub(crate) fn insert_marker(encoder: &mut wgpu::CommandEncoder, label: &'static str) {
    encoder.insert_debug_marker(label);
}

pub(crate) fn push_group(encoder: &mut wgpu::CommandEncoder, label: &'static str) {
    encoder.push_debug_group(label);
}

pub(crate) fn pop_group(encoder: &mut wgpu::CommandEncoder) {
    encoder.pop_debug_group();
}
