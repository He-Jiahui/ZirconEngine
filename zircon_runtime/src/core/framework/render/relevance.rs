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
mod tests {
    use crate::core::framework::render::{
        CorePipelineKind, PrimitiveRelevance, RenderLayerSet, RenderMaterialAlphaMode, RenderPhase,
    };
    use crate::core::framework::scene::Mobility;

    #[test]
    fn primitive_relevance_tracks_material_layer_and_motion_policy() {
        let camera_layers = RenderLayerSet::layer(2);
        let dynamic_opaque = PrimitiveRelevance::for_mesh_view(
            &camera_layers,
            CorePipelineKind::Core3d,
            &RenderLayerSet::layer(2),
            Mobility::Dynamic,
            RenderMaterialAlphaMode::Opaque,
        );

        assert!(dynamic_opaque.render_layer_visible());
        assert!(dynamic_opaque.main_view());
        assert!(dynamic_opaque.depth_prepass());
        assert!(dynamic_opaque.shadow_caster());
        assert!(dynamic_opaque.deferred_geometry());
        assert!(dynamic_opaque.motion_vector_candidate());
        assert!(dynamic_opaque.is_relevant_to_phase(RenderPhase::Opaque3d));
        assert!(dynamic_opaque.is_relevant_to_phase(RenderPhase::Prepass));
        assert!(dynamic_opaque.is_relevant_to_phase(RenderPhase::Shadow));
        assert!(dynamic_opaque.is_relevant_to_phase(RenderPhase::PostProcess));

        let alpha_mask = PrimitiveRelevance::for_mesh_view(
            &camera_layers,
            CorePipelineKind::Core3d,
            &RenderLayerSet::layer(2),
            Mobility::Static,
            RenderMaterialAlphaMode::Mask { cutoff: 0.5 },
        );
        assert!(alpha_mask.is_alpha_mask());
        assert!(alpha_mask.is_relevant_to_phase(RenderPhase::AlphaMask3d));
        assert!(!alpha_mask.motion_vector_candidate());

        let transparent = PrimitiveRelevance::for_mesh_view(
            &camera_layers,
            CorePipelineKind::Core3d,
            &RenderLayerSet::layer(2),
            Mobility::Dynamic,
            RenderMaterialAlphaMode::Blend,
        );
        assert!(transparent.is_transparent());
        assert!(transparent.is_relevant_to_phase(RenderPhase::Transparent3d));
        assert!(!transparent.depth_prepass());
        assert!(!transparent.shadow_caster());
        assert!(!transparent.motion_vector_candidate());
    }

    #[test]
    fn primitive_relevance_keeps_shadow_eligibility_separate_from_main_view_layers() {
        let camera_layers = RenderLayerSet::layer(0);
        let hidden_alpha_mask = PrimitiveRelevance::for_mesh_view(
            &camera_layers,
            CorePipelineKind::Core3d,
            &RenderLayerSet::layer(4),
            Mobility::Static,
            RenderMaterialAlphaMode::Mask { cutoff: 0.5 },
        );

        assert!(!hidden_alpha_mask.render_layer_visible());
        assert!(!hidden_alpha_mask.main_view());
        assert!(!hidden_alpha_mask.is_relevant_to_phase(RenderPhase::AlphaMask3d));
        assert!(!hidden_alpha_mask.depth_prepass());
        assert!(hidden_alpha_mask.shadow_caster());
        assert!(hidden_alpha_mask.is_relevant_to_phase(RenderPhase::Shadow));
    }

    #[test]
    fn primitive_relevance_preserves_layers_above_legacy_mask_width() {
        let camera_layers = RenderLayerSet::layer(40);
        let render_layers = RenderLayerSet::layer(40);
        let relevance = PrimitiveRelevance::for_mesh_view(
            &camera_layers,
            CorePipelineKind::Core3d,
            &render_layers,
            Mobility::Static,
            RenderMaterialAlphaMode::Opaque,
        );

        assert!(relevance.render_layer_visible());
        assert!(relevance.main_view());
        assert!(relevance.view_visible_for_layers(&camera_layers, &render_layers));
        assert!(!relevance.view_visible_for_layers(&RenderLayerSet::layer(0), &render_layers));
    }
}
