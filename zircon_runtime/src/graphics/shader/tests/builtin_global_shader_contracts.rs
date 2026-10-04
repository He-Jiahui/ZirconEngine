use crate::graphics::shader::invocation::{
    ShaderDispatchExtent, ShaderResourceAccess, ShaderResourceKind, COMPUTE_SHADER_PARAMS_BINDING,
    FULLSCREEN_PASS_INPUT_GROUP, FULLSCREEN_TRIANGLE_VERTEX_ENTRY,
};

use super::*;

#[test]
fn hzb_compute_contract_assigns_generated_parameter_and_resource_bindings() {
    let plan = hzb_build_dispatch_plan();

    assert_eq!(plan.pipeline_label, HZB_BUILD_PIPELINE_LABEL);
    assert_eq!(plan.workgroup_size, [8, 8, 1]);
    assert_eq!(plan.dispatch_extent, ShaderDispatchExtent::HzbFurthest);
    assert_eq!(COMPUTE_SHADER_PARAMS_BINDING.binding, 0);
    assert_eq!(
        plan.resources
            .iter()
            .map(|resource| (
                resource.name.as_str(),
                resource.kind,
                resource.access,
                resource.abi.binding,
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                HZB_SCENE_DEPTH_RESOURCE,
                ShaderResourceKind::Texture,
                ShaderResourceAccess::Read,
                1,
            ),
            (
                HZB_SOURCE_RESOURCE,
                ShaderResourceKind::Texture,
                ShaderResourceAccess::Read,
                2,
            ),
            (
                HZB_TARGET_RESOURCE,
                ShaderResourceKind::StorageTexture,
                ShaderResourceAccess::Write,
                3,
            ),
        ]
    );
}

#[test]
fn motion_vector_fullscreen_contract_uses_generated_triangle_and_pass_input_group() {
    let plan = motion_vector_tile_max_pass_plan();

    assert_eq!(plan.pipeline_label, MOTION_VECTOR_TILE_MAX_PIPELINE_LABEL);
    assert_eq!(plan.vertex_entry, FULLSCREEN_TRIANGLE_VERTEX_ENTRY);
    assert_eq!(plan.resources.len(), 1);
    assert_eq!(plan.resources[0].name, MOTION_VECTOR_SOURCE_RESOURCE);
    assert_eq!(plan.resources[0].abi.group, FULLSCREEN_PASS_INPUT_GROUP);
    assert_eq!(plan.resources[0].abi.binding, 0);
    assert_eq!(
        plan.parameters.get(MOTION_VECTOR_TILE_SPAN_PARAMETER),
        Some(
            &crate::graphics::shader::invocation::ShaderParameterValue::Vec4 {
                value: [2.0, 2.0, 0.0, 0.0],
            }
        )
    );
}
