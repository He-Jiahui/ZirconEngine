use crate::core::framework::scene::Mobility;

use super::camera::RenderLayerSet;
use super::core_pipeline::{CorePipelineKind, RenderPhase};
use super::material::RenderMaterialAlphaMode;

/// Compact pass-eligibility flags computed after typed render-layer filtering.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PrimitiveRelevance {
    bits: u32,
}

impl PrimitiveRelevance {
    const RENDER_LAYER_VISIBLE: u32 = 1 << 0;
    const MAIN_VIEW: u32 = 1 << 1;
    const OPAQUE: u32 = 1 << 2;
    const ALPHA_MASK: u32 = 1 << 3;
    const TRANSPARENT: u32 = 1 << 4;
    const DEPTH_PREPASS: u32 = 1 << 5;
    const SHADOW_CASTER: u32 = 1 << 6;
    const DEFERRED_GEOMETRY: u32 = 1 << 7;
    const MOTION_VECTOR_CANDIDATE: u32 = 1 << 8;

    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// 为一个视图建立材质和渲染层的阶段资格；阴影投射资格独立于主视图层命中，
    /// 因此调用方还须结合实际阴影视图和渲染器的阴影开关裁剪。
    pub fn for_mesh_view(
        camera_layers: &RenderLayerSet,
        pipeline: CorePipelineKind,
        render_layers: &RenderLayerSet,
        mobility: Mobility,
        material_alpha_mode: RenderMaterialAlphaMode,
    ) -> Self {
        let mut relevance = Self::empty();
        let render_layer_visible = camera_layers.intersects(render_layers);
        if render_layer_visible {
            relevance = relevance.with(Self::RENDER_LAYER_VISIBLE | Self::MAIN_VIEW);
        }

        match material_alpha_mode {
            RenderMaterialAlphaMode::Opaque => {
                relevance = relevance.with(Self::OPAQUE | Self::SHADOW_CASTER);
                if render_layer_visible {
                    relevance = relevance.with(Self::DEPTH_PREPASS);
                    if pipeline == CorePipelineKind::Core3d {
                        relevance = relevance.with(Self::DEFERRED_GEOMETRY);
                    }
                }
            }
            RenderMaterialAlphaMode::Mask { .. } => {
                relevance = relevance.with(Self::ALPHA_MASK | Self::SHADOW_CASTER);
                if render_layer_visible {
                    relevance = relevance.with(Self::DEPTH_PREPASS);
                    if pipeline == CorePipelineKind::Core3d {
                        relevance = relevance.with(Self::DEFERRED_GEOMETRY);
                    }
                }
            }
            RenderMaterialAlphaMode::Blend => {
                relevance = relevance.with(Self::TRANSPARENT);
            }
        }

        if render_layer_visible
            && mobility == Mobility::Dynamic
            && (relevance.is_opaque() || relevance.is_alpha_mask())
        {
            relevance = relevance.with(Self::MOTION_VECTOR_CANDIDATE);
        }

        relevance
    }

    pub const fn bits(self) -> u32 {
        self.bits
    }

    pub const fn render_layer_visible(self) -> bool {
        self.has(Self::RENDER_LAYER_VISIBLE)
    }

    pub const fn main_view(self) -> bool {
        self.has(Self::MAIN_VIEW)
    }

    pub const fn is_opaque(self) -> bool {
        self.has(Self::OPAQUE)
    }

    pub const fn is_alpha_mask(self) -> bool {
        self.has(Self::ALPHA_MASK)
    }

    pub const fn is_transparent(self) -> bool {
        self.has(Self::TRANSPARENT)
    }

    pub const fn depth_prepass(self) -> bool {
        self.has(Self::DEPTH_PREPASS)
    }

    pub const fn shadow_caster(self) -> bool {
        self.has(Self::SHADOW_CASTER)
    }

    pub const fn deferred_geometry(self) -> bool {
        self.has(Self::DEFERRED_GEOMETRY)
    }

    pub const fn motion_vector_candidate(self) -> bool {
        self.has(Self::MOTION_VECTOR_CANDIDATE)
    }

    pub fn view_visible_for_layers(
        self,
        camera_layers: &RenderLayerSet,
        render_layers: &RenderLayerSet,
    ) -> bool {
        if !camera_layers.intersects(render_layers) {
            return false;
        }
        self.is_opaque() || self.is_alpha_mask() || self.is_transparent()
    }

    pub fn is_relevant_to_phase(self, phase: RenderPhase) -> bool {
        match phase {
            RenderPhase::Prepass => self.depth_prepass(),
            RenderPhase::Shadow => self.shadow_caster(),
            RenderPhase::Opaque2d | RenderPhase::Opaque3d => self.main_view() && self.is_opaque(),
            RenderPhase::AlphaMask2d | RenderPhase::AlphaMask3d => {
                self.main_view() && self.is_alpha_mask()
            }
            RenderPhase::Transparent2d | RenderPhase::Transparent3d => {
                self.main_view() && self.is_transparent()
            }
            RenderPhase::Deferred => self.deferred_geometry(),
            RenderPhase::PostProcess => self.motion_vector_candidate(),
            RenderPhase::Ui | RenderPhase::Overlay | RenderPhase::Debug => false,
        }
    }

    const fn with(self, bits: u32) -> Self {
        Self {
            bits: self.bits | bits,
        }
    }

    const fn has(self, bits: u32) -> bool {
        (self.bits & bits) == bits
    }
}

#[cfg(test)]
#[path = "tests/relevance.rs"]
mod tests;
