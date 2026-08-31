use crate::core::math::UVec2;

use super::super::RenderFrameExtract;
use super::{
    handles::{RenderPipelineHandle, RenderViewportHandle},
    quality::RenderQualityProfile,
};

#[derive(Clone, Debug, PartialEq)]
pub enum RenderCommand {
    SubmitFrameExtract {
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
    },
    ReloadPipeline {
        pipeline: RenderPipelineHandle,
    },
    SetQualityProfile {
        viewport: RenderViewportHandle,
        profile: RenderQualityProfile,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderQuery {
    Stats,
    CaptureFrame(RenderViewportHandle),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderHybridGiPayloadSource {
    #[default]
    None,
    SceneRepresentation,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderVirtualGeometryPayloadSource {
    #[default]
    None,
    Authored,
    AutomaticFallback,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderViewportDescriptor {
    pub size: UVec2,
    pub label: Option<String>,
    pub requires_hit_proxies: bool,
}

impl RenderViewportDescriptor {
    pub fn new(size: UVec2) -> Self {
        Self {
            size,
            label: None,
            requires_hit_proxies: false,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_hit_proxies(mut self) -> Self {
        self.requires_hit_proxies = true;
        self
    }
}
