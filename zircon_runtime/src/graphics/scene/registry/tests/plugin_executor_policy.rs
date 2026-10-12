use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::{
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions, RendererFeatureAsset,
};
use crate::render_graph::QueueLane;

use super::super::super::{
    RenderPassExecutionContext, RenderPassExecutorId, RenderPassExecutorRegistration,
};
use super::super::RenderPassExecutorRegistry;
use super::support::{
    explicit_virtual_geometry_executor, plugin_virtual_geometry_descriptor, test_extract,
};

#[test]
fn plugin_render_feature_descriptors_require_explicit_executor_registration() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let descriptor = plugin_virtual_geometry_descriptor();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::plugin(descriptor.clone()));
    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry),
        )
        .unwrap();

    let core_registry = RenderPassExecutorRegistry::with_builtin_noop_executors();
    let error = core_registry
        .validate_compiled_pipeline(&compiled)
        .unwrap_err();
    assert!(
        error.contains("virtual-geometry.prepare"),
        "core registry should reject plugin executor ids before plugin registration: {error}"
    );

    let descriptor_only_registry =
        RenderPassExecutorRegistry::with_builtin_noop_executors_for_render_features([
            descriptor.clone()
        ]);
    let error = descriptor_only_registry
        .validate_compiled_pipeline(&compiled)
        .unwrap_err();
    assert!(
        error.contains("unregistered executor `virtual-geometry.prepare`"),
        "plugin descriptors should not auto-register runtime no-op executors: {error}"
    );

    let plugin_registry = RenderPassExecutorRegistry::with_builtin_noop_executors_for_render_features_and_executor_registrations(
        [descriptor],
        [RenderPassExecutorRegistration::new(
            "virtual-geometry.prepare",
            explicit_virtual_geometry_executor,
        )],
    );
    plugin_registry
        .validate_compiled_pipeline(&compiled)
        .expect("explicit plugin executor registration should satisfy the compiled graph");
}

#[test]
fn plugin_particle_extensions_still_require_explicit_registration_for_custom_executor_ids() {
    let descriptor = RenderFeatureDescriptor::new(
        "particle",
        Vec::new(),
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Transparent3d,
            "particle-plugin-gpu-simulation",
            QueueLane::Graphics,
        )
        .with_executor_id("particle.plugin-gpu-simulation")
        .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
        .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
        .write_texture(PostProcessGraphResourceNames::SCENE_COLOR)],
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([descriptor.clone()])
        .compile(&test_extract())
        .unwrap();
    let registry =
        RenderPassExecutorRegistry::with_builtin_noop_executors_for_render_features([descriptor]);

    let error = registry.validate_compiled_pipeline(&compiled).unwrap_err();

    assert!(
        error.contains("unregistered executor `particle.plugin-gpu-simulation`"),
        "custom particle graph pass should require an explicit plugin executor registration: {error}"
    );
}

#[test]
fn explicit_executor_registration_satisfies_plugin_executor_id() {
    let descriptor = plugin_virtual_geometry_descriptor();
    let registry = RenderPassExecutorRegistry::with_builtin_noop_executors_for_render_features_and_executor_registrations(
        [descriptor],
        [RenderPassExecutorRegistration::new(
            "virtual-geometry.prepare",
            explicit_virtual_geometry_executor,
        )],
    );

    let error = registry
        .execute(&mut RenderPassExecutionContext::new(
            "plugin-virtual-geometry-registry",
            RenderPassExecutorId::new("virtual-geometry.prepare"),
        ))
        .unwrap_err();

    assert_eq!(error, "explicit virtual geometry executor called");
}
