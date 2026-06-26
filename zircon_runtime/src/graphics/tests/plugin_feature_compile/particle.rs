use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderFrameExtract, RenderParticleSpriteSnapshot,
};
use crate::core::math::{Vec3, Vec4};
use crate::graphics::tests::plugin_render_feature_fixtures::particle_render_feature_descriptor;
use crate::graphics::{
    BuiltinRenderFeature, CompiledRenderPipeline, RenderPipelineAsset, RenderPipelineCompileOptions,
};
use crate::render_graph::RenderGraphResourceAccessKind;

use super::test_extract;

#[test]
fn particle_plugin_render_feature_adds_transparent_pass_to_default_pipeline() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([particle_render_feature_descriptor()]);
    let compiled = pipeline.compile(&test_extract()).unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(pass_names.contains(&"particle-render"));
    assert!(compiled
        .required_extract_sections
        .contains(&"particles".to_string()));
    let particle_feature = compiled
        .enabled_features
        .iter()
        .find(|feature| feature.feature_name() == "particle")
        .expect("particle plugin feature should remain in compiled pipeline");
    assert!(
        particle_feature.builtin_feature().is_none(),
        "particle plugin feature should not reintroduce built-in feature identity"
    );
    assert_particle_pass_uses_depth_read_color_write(&compiled);
}

#[test]
fn core_scene_particle_extract_adds_billboard_pass_without_plugin_feature_identity() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract_with_particle_sprite())
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(pass_names.contains(&"particle-render"));
    assert!(compiled
        .required_extract_sections
        .contains(&"particles".to_string()));
    assert!(
        !compiled
            .enabled_features
            .iter()
            .any(|feature| feature.feature_name() == "particle"),
        "core scene particles should not masquerade as an external particle plugin feature"
    );
    assert!(
        !compiled
            .enabled_features
            .iter()
            .any(|feature| feature.is_builtin(BuiltinRenderFeature::Particle)),
        "core scene particles should not reintroduce the descriptor-only built-in particle slot"
    );
    assert_particle_pass_uses_depth_read_color_write(&compiled);
}

#[test]
fn compile_options_can_disable_core_scene_particle_billboard_pass() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &test_extract_with_particle_sprite(),
            &RenderPipelineCompileOptions::default()
                .with_feature_disabled(BuiltinRenderFeature::Particle),
        )
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"particle-render"));
    assert!(!compiled
        .required_extract_sections
        .contains(&"particles".to_string()));
}

#[test]
fn compile_options_can_disable_particle_plugin_feature_by_name() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([particle_render_feature_descriptor()]);
    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default().with_plugin_feature_disabled("particle"),
        )
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"particle-render"));
    assert!(!compiled
        .required_extract_sections
        .contains(&"particles".to_string()));
}

fn test_extract_with_particle_sprite() -> RenderFrameExtract {
    let mut extract = test_extract();
    extract
        .particles
        .sprites
        .push(RenderParticleSpriteSnapshot {
            entity: 1,
            stable_sprite_key: 1,
            position: Vec3::ZERO,
            size: 1.0,
            color: Vec4::ONE,
            intensity: 1.0,
            ..RenderParticleSpriteSnapshot::default()
        });
    extract
}

fn assert_particle_pass_uses_depth_read_color_write(compiled: &CompiledRenderPipeline) {
    let particle_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "particle-render")
        .expect("compiled graph should contain particle-render pass");

    assert!(
        particle_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_DEPTH
                && resource.access == RenderGraphResourceAccessKind::Read
        }),
        "particle-render should read scene-depth"
    );
    assert!(
        !particle_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_COLOR
                && resource.access == RenderGraphResourceAccessKind::Read
        }),
        "particle-render must not read scene-color while also writing it"
    );
    assert!(
        particle_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_COLOR
                && resource.access == RenderGraphResourceAccessKind::Write
        }),
        "particle-render should write scene-color"
    );
}
