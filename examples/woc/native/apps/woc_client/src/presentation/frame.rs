use woc_runtime::{HudProjection, PresentationBlendMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientFrameAdvance {
    pub committed_ticks: u32,
    pub backlog_ticks: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientPresentedFrame<'a> {
    pub hud: &'a HudProjection,
    pub blend_mode: PresentationBlendMode,
    pub alpha: f32,
}
