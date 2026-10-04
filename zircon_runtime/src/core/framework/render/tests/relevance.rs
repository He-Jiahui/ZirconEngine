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
fn primitive_relevance_preserves_layers_above_scene_schema_v1_mask_width() {
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
