use crate::core::framework::render::{
    RenderCapabilityKind, RenderCapabilityMismatchDetail, RenderCapabilitySummary,
    RenderFrameExtract, RenderFrameworkError, RenderPipelineHandle, RenderQualityProfile,
    RenderWorldSnapshotHandle,
};
use crate::graphics::{
    BuiltinRenderFeature, RenderFeatureCapabilityRequirement, RenderFeatureDescriptor,
    RenderFeaturePassDescriptor, RenderPassStage, RenderPipelineAsset,
    RenderPipelineCompileOptions, RendererFeatureAsset,
};
use crate::render_graph::{QueueLane, RenderGraphComputeWorkload};
use crate::scene::world::World;

use super::{validate_compiled_pipeline_capabilities, validate_quality_profile_capabilities};

#[test]
fn quality_profile_capability_validation_allows_advanced_features_to_degrade() {
    let profile = RenderQualityProfile::new("degradable-advanced")
        .with_virtual_geometry(true)
        .with_hybrid_global_illumination(true);
    let capabilities = RenderCapabilitySummary {
        backend_name: "capability-test".to_string(),
        supports_offscreen: true,
        supports_fxaa: true,
        ..Default::default()
    };

    validate_quality_profile_capabilities(
        Some(RenderPipelineHandle::new(7)),
        &profile,
        &capabilities,
    )
    .expect("advanced profile features should degrade through the runtime plan");
}

#[test]
fn quality_profile_capability_validation_keeps_non_degradable_requirements_strict() {
    let profile = RenderQualityProfile::new("strict-aa");
    let capabilities = RenderCapabilitySummary {
        backend_name: "capability-test".to_string(),
        ..Default::default()
    };

    let error = validate_quality_profile_capabilities(
        Some(RenderPipelineHandle::new(7)),
        &profile,
        &capabilities,
    )
    .unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::CapabilityMismatch {
            pipeline: 7,
            reason: "quality profile `strict-aa` requires screen_space_anti_alias".to_string(),
            missing: vec![RenderCapabilityMismatchDetail::new(
                RenderCapabilityKind::ScreenSpaceAntiAlias,
            )],
        }
    );
}

#[test]
fn compiled_pipeline_capability_validation_reports_descriptor_requirements() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::plugin(
            RenderFeatureDescriptor::new(
                "plugin.virtual_geometry.capability_validation",
                Vec::new(),
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::DepthPrepass,
                    "plugin-virtual-geometry-capability-validation",
                    QueueLane::Graphics,
                )
                .with_executor_id("plugin.virtual-geometry.capability-validation")
                .with_side_effects()],
            )
            .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry),
        ));
    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_disabled(BuiltinRenderFeature::AntiAlias)
                .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry),
        )
        .unwrap();
    assert_eq!(
        compiled.capability_requirements,
        vec![RenderFeatureCapabilityRequirement::VirtualGeometry]
    );
    let capabilities = RenderCapabilitySummary {
        backend_name: "capability-test".to_string(),
        supports_offscreen: true,
        supports_fxaa: true,
        ..Default::default()
    };

    let error = validate_compiled_pipeline_capabilities(&compiled, &capabilities).unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::CapabilityMismatch {
            pipeline: compiled.handle.raw(),
            reason: format!("pipeline `{}` requires virtual_geometry", compiled.name),
            missing: vec![RenderCapabilityMismatchDetail::new(
                RenderCapabilityKind::VirtualGeometry,
            )],
        }
    );
}

#[test]
fn compiled_pipeline_capability_validation_splits_rt_backend_requirements() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::plugin(
            RenderFeatureDescriptor::new(
                "plugin.rt.capability_validation",
                Vec::new(),
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "plugin-rt-capability-validation",
                    QueueLane::Graphics,
                )
                .with_executor_id("plugin.rt.capability-validation")
                .with_side_effects()],
            )
            .with_capability_requirement(RenderFeatureCapabilityRequirement::AccelerationStructures)
            .with_capability_requirement(RenderFeatureCapabilityRequirement::RayTracingPipeline),
        ));
    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::AccelerationStructures)
                .with_capability_enabled(RenderFeatureCapabilityRequirement::RayTracingPipeline),
        )
        .unwrap();
    let capabilities = RenderCapabilitySummary {
        backend_name: "capability-test".to_string(),
        supports_offscreen: true,
        supports_fxaa: true,
        ..Default::default()
    };

    let error = validate_compiled_pipeline_capabilities(&compiled, &capabilities).unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::CapabilityMismatch {
            pipeline: compiled.handle.raw(),
            reason: format!(
                "pipeline `{}` requires acceleration_structures, ray_tracing_pipeline",
                compiled.name
            ),
            missing: vec![
                RenderCapabilityMismatchDetail::new(RenderCapabilityKind::AccelerationStructures,),
                RenderCapabilityMismatchDetail::new(RenderCapabilityKind::RayTracingPipeline),
            ],
        }
    );
}

#[test]
fn compiled_pipeline_capability_validation_reports_neural_compute_requirement() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::plugin(
            RenderFeatureDescriptor::new(
                "plugin.neural.capability_validation",
                Vec::new(),
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "plugin-neural-capability-validation",
                    QueueLane::AsyncCompute,
                )
                .with_executor_id("plugin.neural.capability-validation")
                .with_compute_workload(RenderGraphComputeWorkload::fixed(
                    "plugin-neural-capability-validation",
                    [1, 1, 1],
                    [1, 1, 1],
                ))
                .with_side_effects()],
            )
            .with_capability_requirement(RenderFeatureCapabilityRequirement::NeuralCompute),
        ));
    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::NeuralCompute),
        )
        .unwrap();
    let capabilities = RenderCapabilitySummary {
        backend_name: "capability-test".to_string(),
        supports_offscreen: true,
        supports_fxaa: true,
        supports_async_compute: true,
        supports_storage_buffers: true,
        ..Default::default()
    };

    let error = validate_compiled_pipeline_capabilities(&compiled, &capabilities).unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::CapabilityMismatch {
            pipeline: compiled.handle.raw(),
            reason: format!("pipeline `{}` requires neural_compute", compiled.name),
            missing: vec![RenderCapabilityMismatchDetail::new(
                RenderCapabilityKind::NeuralCompute,
            )],
        }
    );
}

#[test]
fn compiled_pipeline_capability_validation_reports_sparse_texture_requirement() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::builtin(
            BuiltinRenderFeature::SparseTexture,
        ));
    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::SparseTexture)
                .with_capability_enabled(RenderFeatureCapabilityRequirement::SparseTexture),
        )
        .unwrap();
    let capabilities = RenderCapabilitySummary {
        backend_name: "capability-test".to_string(),
        supports_offscreen: true,
        supports_fxaa: true,
        ..Default::default()
    };

    let error = validate_compiled_pipeline_capabilities(&compiled, &capabilities).unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::CapabilityMismatch {
            pipeline: compiled.handle.raw(),
            reason: format!("pipeline `{}` requires sparse_texture", compiled.name),
            missing: vec![RenderCapabilityMismatchDetail::new(
                RenderCapabilityKind::SparseTexture,
            )],
        }
    );
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}
