use super::{ComputePassDescriptor, ComputeShaderSource};
use crate::core::framework::render::{RenderFrameExtract, RenderWorldSnapshotHandle};
use crate::graphics::feature::RenderFeaturePassDescriptor;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::{
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeatureResourceAccess,
    RenderFeatureResourceKind, RenderPipelineAsset, RenderPipelineCompileOptions,
};
use crate::render_graph::{
    BindingSchemaEntry, ComputeBindingKind, PassFlags, QueueLane, RenderGraphComputeDispatchExtent,
    RenderGraphExternalResourceBinding,
};
use crate::scene::world::World;

#[test]
fn compute_pass_lowering_preserves_bindings_dispatch_and_generic_executor() {
    let pass = ComputePassDescriptor::new(
        "plugin-reduce",
        RenderPassStage::PostProcess,
        QueueLane::AsyncCompute,
        ComputeShaderSource::inline_wgsl(
            "plugin-reduce",
            "@compute @workgroup_size(8, 8, 1) fn cs_main() {}",
        ),
        "cs_main",
        [8, 8, 1],
        vec![
            BindingSchemaEntry::new(0, "scene-color", ComputeBindingKind::SampledTexture),
            BindingSchemaEntry::new(1, "plugin-output", ComputeBindingKind::StorageTextureWrite),
        ],
        RenderGraphComputeDispatchExtent::PerPixel {
            target: "plugin-output".to_string(),
            local_size: [8, 8],
        },
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    )
    .into_feature_pass();

    assert_eq!(pass.executor_id.as_str(), "compute.generic");
    assert_eq!(
        pass.compute_workload.as_ref().unwrap().pipeline_label,
        "plugin-reduce"
    );
    assert_eq!(pass.resources.len(), 2);
    let compute_metadata = pass.compute_pass.as_ref().unwrap().graph_metadata();
    assert_eq!(compute_metadata.entry_point, "cs_main");
    assert_eq!(compute_metadata.bindings.len(), 2);
    assert!(matches!(
        compute_metadata.shader,
        crate::render_graph::RenderGraphComputeShaderSource::Wgsl { label, .. }
            if label == "plugin-reduce"
    ));
    assert!(pass.resources.iter().any(|resource| {
        resource.name == "scene-color"
            && resource.kind == RenderFeatureResourceKind::Texture
            && resource.access == RenderFeatureResourceAccess::Read
            && resource.external_binding == RenderGraphExternalResourceBinding::required_texture()
    }));
    assert!(pass.resources.iter().any(|resource| {
        resource.name == "plugin-output"
            && resource.kind == RenderFeatureResourceKind::Texture
            && resource.access == RenderFeatureResourceAccess::Write
            && resource.external_binding == RenderGraphExternalResourceBinding::required_texture()
    }));
}

#[test]
fn compute_binding_can_request_an_owned_full_mip_chain() {
    let binding = BindingSchemaEntry::new(5, "hzb-furthest", ComputeBindingKind::SampledTexture)
        .with_texture_full_mip_chain();

    assert_eq!(binding.texture_mip_level, None);
    assert!(binding.texture_full_mip_chain);
}

#[test]
fn compute_pass_lowering_declares_unbound_per_pixel_dispatch_target() {
    let pass = ComputePassDescriptor::new(
        "plugin-dispatch-size",
        RenderPassStage::PostProcess,
        QueueLane::AsyncCompute,
        ComputeShaderSource::inline_wgsl(
            "plugin-dispatch-size",
            "@compute @workgroup_size(8, 8, 1) fn cs_main() {}",
        ),
        "cs_main",
        [8, 8, 1],
        Vec::new(),
        RenderGraphComputeDispatchExtent::PerPixel {
            target: "dispatch-size".to_string(),
            local_size: [8, 8],
        },
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    )
    .into_feature_pass();

    assert_eq!(pass.resources.len(), 1);
    assert!(pass.resources.iter().any(|resource| {
        resource.name == "dispatch-size"
            && resource.kind == RenderFeatureResourceKind::Texture
            && resource.access == RenderFeatureResourceAccess::Read
            && resource.external_binding == RenderGraphExternalResourceBinding::required_texture()
    }));
}

#[test]
fn plugin_compute_descriptor_compiles_into_the_render_graph() {
    let compute_pass = ComputePassDescriptor::new(
        "plugin-compute-output",
        RenderPassStage::PostProcess,
        QueueLane::AsyncCompute,
        ComputeShaderSource::inline_wgsl(
            "plugin-compute-output",
            "@group(1) @binding(0) var plugin_compute_output: texture_storage_2d<rgba8unorm, write>;\n@compute @workgroup_size(8, 8, 1) fn cs_main() { textureStore(plugin_compute_output, vec2<i32>(0), vec4<f32>(0.0)); }",
        ),
        "cs_main",
        [8, 8, 1],
        vec![BindingSchemaEntry::new(
            0,
            "plugin-compute-output",
            ComputeBindingKind::StorageTextureWrite,
        )],
        RenderGraphComputeDispatchExtent::PerPixel {
            target: "plugin-compute-output".to_string(),
            local_size: [8, 8],
        },
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    )
    .into_feature_pass();
    let pipeline = RenderPipelineAsset::default_forward_plus().with_plugin_render_features([
        RenderFeatureDescriptor::new(
            "plugin.compute-descriptor",
            Vec::new(),
            Vec::new(),
            vec![compute_pass],
        )
        .with_capability_requirement(RenderFeatureCapabilityRequirement::NeuralCompute),
    ]);

    let disabled = pipeline
        .compile(&test_extract())
        .expect("disabled plugin compute descriptor should not enter the render graph");
    assert!(!disabled
        .graph()
        .passes()
        .iter()
        .any(|pass| pass.name == "plugin-compute-output"));

    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::NeuralCompute),
        )
        .expect("plugin compute descriptor should compile into the render graph");
    let pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "plugin-compute-output")
        .expect("compiled graph should include the plugin compute pass");

    assert_eq!(pass.executor_id.as_deref(), Some("compute.generic"));
    assert_eq!(
        pass.compute_workload
            .as_ref()
            .map(|workload| workload.workgroup_size),
        Some([8, 8, 1])
    );
    assert!(pass.compute_pass_metadata.is_some());
}

#[test]
fn compute_pass_lowering_keeps_caller_declared_external_resources() {
    let descriptor = ComputePassDescriptor::new(
        "plugin-output",
        RenderPassStage::PostProcess,
        QueueLane::AsyncCompute,
        ComputeShaderSource::inline_wgsl(
            "plugin-output",
            "@compute @workgroup_size(1) fn cs_main() {}",
        ),
        "cs_main",
        [1, 1, 1],
        vec![BindingSchemaEntry::new(
            0,
            "plugin-output",
            ComputeBindingKind::StorageBufferReadWrite,
        )],
        RenderGraphComputeDispatchExtent::Fixed([1, 1, 1]),
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    );
    let pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::PostProcess,
        "plugin-output",
        QueueLane::AsyncCompute,
    )
    .read_external_buffer("plugin-output")
    .write_storage_external_buffer("plugin-output")
    .with_compute_pass(descriptor);

    assert_eq!(pass.resources.len(), 2);
    assert!(pass.resources.iter().all(|resource| {
        resource.name == "plugin-output" && resource.kind == RenderFeatureResourceKind::External
    }));
}

#[test]
fn compute_pass_lowering_preserves_external_binding_when_adding_missing_read_access() {
    let descriptor = ComputePassDescriptor::new(
        "plugin-output",
        RenderPassStage::PostProcess,
        QueueLane::AsyncCompute,
        ComputeShaderSource::inline_wgsl(
            "plugin-output",
            "@compute @workgroup_size(1) fn cs_main() {}",
        ),
        "cs_main",
        [1, 1, 1],
        vec![BindingSchemaEntry::new(
            0,
            "plugin-output",
            ComputeBindingKind::StorageBufferReadWrite,
        )],
        RenderGraphComputeDispatchExtent::Fixed([1, 1, 1]),
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    );
    let pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::PostProcess,
        "plugin-output",
        QueueLane::AsyncCompute,
    )
    .write_storage_external_buffer("plugin-output")
    .with_compute_pass(descriptor);

    assert_eq!(pass.resources.len(), 2);
    assert!(pass.resources.iter().all(|resource| {
        resource.name == "plugin-output" && resource.kind == RenderFeatureResourceKind::External
    }));
    assert!(pass
        .resources
        .iter()
        .any(|resource| resource.access == RenderFeatureResourceAccess::Read));
    assert!(pass
        .resources
        .iter()
        .any(|resource| resource.access == RenderFeatureResourceAccess::Write));
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}
