use std::collections::HashSet;

use crate::core::framework::render::{
    CastShadowsMode, RenderViewportPickPolicy, ShaderQualityTier,
};
use crate::core::resource::ResourceId;
use crate::graphics::scene::resources::{MaterialDisabledPasses, default_pipeline_key};
use crate::graphics::scene::scene_renderer::mesh::MaterialPipelineRequirementSet;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassPipelineKind;
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::PipelineCreationTarget;

use super::{
    MaterialPipelineCandidate, MaterialPipelineCensusOwner, MaterialPipelineDrawContext,
    MaterialPipelineFeatureSet, MaterialPipelineGeometryContext, MaterialPipelineInputs,
    MaterialPipelineObservedContexts, MaterialPipelineShadowContext,
    insert_material_pipeline_requirements,
};

#[test]
fn published_context_identity_separates_every_requirement_dimension() {
    let material_id = ResourceId::from_stable_label("material-context-identity");
    let owner = MaterialPipelineCensusOwner {
        material_id,
        generation: 7,
    };
    let context = MaterialPipelineDrawContext {
        geometry: MaterialPipelineGeometryContext::Static,
        velocity_history_eligible: false,
        shadow: MaterialPipelineShadowContext::Disabled,
    };
    let mut observed = HashSet::new();

    assert!(observed.insert((owner, context)));
    assert!(!observed.insert((owner, context)));
    for distinct in [
        (
            MaterialPipelineCensusOwner {
                generation: 8,
                ..owner
            },
            context,
        ),
        (
            owner,
            MaterialPipelineDrawContext {
                geometry: MaterialPipelineGeometryContext::Skinned,
                ..context
            },
        ),
        (
            owner,
            MaterialPipelineDrawContext {
                velocity_history_eligible: true,
                ..context
            },
        ),
        (
            owner,
            MaterialPipelineDrawContext {
                shadow: MaterialPipelineShadowContext::OneSided,
                ..context
            },
        ),
        (
            owner,
            MaterialPipelineDrawContext {
                shadow: MaterialPipelineShadowContext::ForcedTwoSided,
                ..context
            },
        ),
    ] {
        assert!(observed.insert(distinct));
    }
    assert_eq!(observed.len(), 6);
}

#[test]
fn typed_context_bitset_is_a_bijection_over_the_complete_fixed_domain() {
    let mut observed = MaterialPipelineObservedContexts::default();
    for geometry in [
        MaterialPipelineGeometryContext::Static,
        MaterialPipelineGeometryContext::Skinned,
        MaterialPipelineGeometryContext::Morphed,
        MaterialPipelineGeometryContext::SkinnedMorphed,
    ] {
        for velocity_history_eligible in [false, true] {
            for shadow in [
                MaterialPipelineShadowContext::Disabled,
                MaterialPipelineShadowContext::OneSided,
                MaterialPipelineShadowContext::ForcedTwoSided,
            ] {
                let context = MaterialPipelineDrawContext {
                    geometry,
                    velocity_history_eligible,
                    shadow,
                };
                assert!(observed.insert(context));
                assert!(!observed.insert(context));
            }
        }
    }
    assert_eq!(observed.len(), 24);
}

#[test]
fn renderer_and_material_shadow_modes_resolve_to_three_exact_contexts() {
    for renderer_mode in [
        CastShadowsMode::Off,
        CastShadowsMode::On,
        CastShadowsMode::TwoSided,
        CastShadowsMode::ShadowsOnly,
    ] {
        assert_eq!(
            MaterialPipelineShadowContext::from_modes(renderer_mode, false),
            MaterialPipelineShadowContext::Disabled
        );
    }
    assert_eq!(
        MaterialPipelineShadowContext::from_modes(CastShadowsMode::Off, true),
        MaterialPipelineShadowContext::Disabled
    );
    assert_eq!(
        MaterialPipelineShadowContext::from_modes(CastShadowsMode::On, true),
        MaterialPipelineShadowContext::OneSided
    );
    assert_eq!(
        MaterialPipelineShadowContext::from_modes(CastShadowsMode::ShadowsOnly, true),
        MaterialPipelineShadowContext::OneSided
    );
    assert_eq!(
        MaterialPipelineShadowContext::from_modes(CastShadowsMode::TwoSided, true),
        MaterialPipelineShadowContext::ForcedTwoSided
    );
}

#[test]
fn forced_two_sided_context_only_widens_the_shadow_pipeline_key() {
    let one_sided = default_pipeline_key();
    assert!(!one_sided.double_sided);
    assert!(
        !MaterialPipelineShadowContext::OneSided
            .effective_shadow_pipeline_key(&one_sided)
            .double_sided
    );
    assert!(
        MaterialPipelineShadowContext::ForcedTwoSided
            .effective_shadow_pipeline_key(&one_sided)
            .double_sided
    );

    let mut material_two_sided = one_sided;
    material_two_sided.double_sided = true;
    assert!(
        MaterialPipelineShadowContext::OneSided
            .effective_shadow_pipeline_key(&material_two_sided)
            .double_sided
    );
}

#[test]
fn requirement_census_deduplicates_typed_draw_contexts_before_pipeline_construction() {
    let source = include_str!("../material_pipeline_requirements.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("material pipeline requirement test boundary");

    assert!(source.contains("struct MaterialPipelineDrawContext"));
    assert!(source.contains("struct MaterialPipelineCensusOwner"));
    assert!(source.contains("struct MaterialPipelineObservedContexts"));
    assert!(source.contains("bits: u32"));
    let observation = source
        .split("fn observe_published_draw(")
        .nth(1)
        .expect("published draw context observation");
    let deduplicate = observation
        .find("row.observed_contexts.insert(context)")
        .expect("typed context deduplication");
    let construct = observation
        .find("insert_material_pipeline_requirements(")
        .expect("requirement construction");

    assert!(deduplicate < construct);
    assert!(!observation[..construct].contains("pipeline_key.clone()"));
    assert!(!source.contains("MaterialPipelineInputs::from_pending_draw"));
}

#[test]
fn staged_and_previous_censuses_resolve_each_material_candidate_once() {
    let source = include_str!("../material_pipeline_requirements.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("material pipeline requirement test boundary");
    let collection = source
        .split("fn collect_material_pipeline_requirements_for(")
        .nth(1)
        .expect("shared material requirement collection");

    assert!(collection.contains("candidate_owners.entry(material_id)"));
    assert!(collection.contains("census"));
    assert!(collection.contains(".rows"));
    assert!(collection.contains(".get_mut(&owner)"));
    assert!(collection.contains("row.observed_contexts.insert(context)"));
    assert!(source.contains("material_current_requirement_observed_draw_count"));
    assert!(source.contains("material_current_requirement_unique_context_count"));
    assert!(source.contains("material_staged_requirement_candidate_resolution_count"));
    assert!(source.contains("material_previous_requirement_candidate_resolution_count"));
    assert!(!collection.contains("saturating_add"));
}

#[test]
fn requirement_census_checks_the_active_candidate_index_before_scanning_draws() {
    let source = include_str!("../material_pipeline_requirements.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("material pipeline requirement test boundary");
    let census = source
        .split("fn collect_material_pipeline_requirements(")
        .nth(1)
        .expect("material pipeline requirement census");
    let stable_gate = census
        .find("if !streamer.has_active_staged_material_candidates()")
        .expect("stable-frame candidate gate");
    let draw_scan = census
        .find("for pending_draw in pending_draws")
        .expect("draw scan");

    assert!(stable_gate < draw_scan);
    assert!(!census.contains("main_view_visible"));
    assert!(!census.contains("shadow_view_visible"));
    assert!(!census.contains("has_previous_velocity_for_instance"));
}

#[test]
fn material_requirement_censuses_have_separate_cpu_profile_scopes() {
    let source = include_str!("../material_pipeline_requirements.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("material pipeline requirement test boundary");

    for scope in [
        "staged_requirement_census",
        "previous_requirement_census",
        "error_proxy_requirement_census",
    ] {
        assert!(
            source.contains(&format!("\"material\", \"{scope}\"")),
            "missing material CPU scope {scope}"
        );
    }
    let collection = include_str!("../collect_pending_draws.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("pending draw collection test boundary");
    assert!(collection.contains("\"material\", \"current_requirement_census\""));
}

#[test]
fn executor_feature_set_only_enables_present_pipeline_consumers() {
    let features = MaterialPipelineFeatureSet::from_executor_ids([
        Some("deferred.gbuffer"),
        Some("shadow.atlas"),
        Some("oit.fragment_store"),
    ]);

    assert!(features.deferred_gbuffer);
    assert!(features.shadow);
    assert!(features.oit);
    assert!(!features.base_opaque);
    assert!(!features.depth_prepass);
    assert!(!features.velocity);
    assert!(!features.taa_reactive);
}

#[test]
fn transmission_executor_is_recognized_without_enabling_generic_transparency() {
    let features = MaterialPipelineFeatureSet::from_executor_ids([Some("mesh.transmission.0")]);

    assert!(features.transmission);
    assert!(!features.base_transparent);
}

#[test]
fn environment_capture_profile_is_opaque_only_and_reverses_view_winding() {
    let features = MaterialPipelineFeatureSet::environment_capture();

    assert!(features.base_opaque);
    assert!(features.base_alpha_mask);
    assert!(features.advanced_pbr_opaque);
    assert!(features.reverse_view_raster_winding);
    assert!(!features.base_transparent);
    assert!(!features.transmission);
    assert!(!features.shadow);
    assert!(!features.velocity);
    assert!(!features.taa_reactive);
    assert!(!features.oit);
}

#[test]
fn hit_proxy_profile_builds_only_the_editor_selection_pipeline() {
    let mut requirements = MaterialPipelineRequirementSet::default();
    let inputs = MaterialPipelineInputs {
        pipeline_key: default_pipeline_key(),
        disabled_passes: MaterialDisabledPasses::from_shader_pass_names(&[
            "base".to_string(),
            "depth".to_string(),
            "shadow".to_string(),
        ]),
        taa_reactive_mask_strength: 0.0,
    };

    insert_material_pipeline_requirements(
        &mut requirements,
        MaterialPipelineDrawContext {
            geometry: MaterialPipelineGeometryContext::Static,
            velocity_history_eligible: false,
            shadow: MaterialPipelineShadowContext::Disabled,
        },
        &inputs,
        MaterialPipelineFeatureSet::hit_proxy(RenderViewportPickPolicy::default()),
        ShaderQualityTier::Medium,
        false,
    );

    let requirement = requirements.iter().next().expect("hit-proxy requirement");
    assert_eq!(requirements.len(), 1);
    assert_eq!(
        requirement.target(),
        PipelineCreationTarget::MeshPass(MeshPassPipelineKind::HitProxy)
    );
    assert!(!requirement.pipeline_key().double_sided);
}

#[test]
fn hit_proxy_profile_requires_explicit_translucent_and_backface_policies() {
    let context = MaterialPipelineDrawContext {
        geometry: MaterialPipelineGeometryContext::Static,
        velocity_history_eligible: false,
        shadow: MaterialPipelineShadowContext::Disabled,
    };
    let mut transparent_key = default_pipeline_key();
    transparent_key.alpha_blend = true;
    let inputs = MaterialPipelineInputs {
        pipeline_key: transparent_key,
        disabled_passes: MaterialDisabledPasses::default(),
        taa_reactive_mask_strength: 0.0,
    };
    let mut default_requirements = MaterialPipelineRequirementSet::default();
    insert_material_pipeline_requirements(
        &mut default_requirements,
        context,
        &inputs,
        MaterialPipelineFeatureSet::hit_proxy(RenderViewportPickPolicy::default()),
        ShaderQualityTier::Medium,
        false,
    );
    assert_eq!(default_requirements.len(), 0);

    let policy = RenderViewportPickPolicy::from_bits(
        RenderViewportPickPolicy::INCLUDE_TRANSLUCENT | RenderViewportPickPolicy::INCLUDE_BACKFACES,
    )
    .expect("known hit-proxy policy flags");
    let mut inclusive_requirements = MaterialPipelineRequirementSet::default();
    insert_material_pipeline_requirements(
        &mut inclusive_requirements,
        context,
        &inputs,
        MaterialPipelineFeatureSet::hit_proxy(policy),
        ShaderQualityTier::Medium,
        false,
    );

    let requirement = inclusive_requirements
        .iter()
        .next()
        .expect("inclusive hit-proxy requirement");
    assert_eq!(inclusive_requirements.len(), 1);
    assert!(requirement.pipeline_key().double_sided);
}

#[test]
fn error_proxy_pipeline_inputs_are_opaque_standard_pbr_defaults() {
    let candidate = MaterialPipelineCandidate::error_proxy();
    let inputs = candidate.inputs;

    assert!(!inputs.pipeline_key.is_transparent());
    assert!(!inputs.pipeline_key.is_alpha_mask());
    assert!(!inputs.pipeline_key.requires_forward_path());
    assert!(candidate.cast_shadows);
    assert_eq!(inputs.disabled_passes, Default::default());
    assert_eq!(inputs.taa_reactive_mask_strength, 0.0);
}

#[test]
fn error_proxy_census_handles_implicit_cold_generation_without_stable_draw_scan() {
    let source = include_str!("../material_pipeline_requirements.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("material pipeline requirement test boundary");
    let census = source
        .split("fn collect_error_proxy_context_pipeline_requirements(")
        .nth(1)
        .expect("error proxy requirement census");

    assert!(census.contains("has_active_staged_material_candidates"));
    assert!(!census.contains("has_material_pipeline_admission_work"));
    assert!(!census.contains("material_pipeline_publication_required"));
    assert!(census.contains("selection.proxy"));
    assert!(census.contains(".runtime()"));
    assert!(census.contains(".is_none()"));
    assert!(census.contains("material_uses_error_proxy"));
    assert!(census.contains(".entry(material_id)"));
}
