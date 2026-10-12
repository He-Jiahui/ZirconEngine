use std::collections::HashSet;

use super::*;

#[test]
fn material_shader_binding_contract_has_one_row_per_fixed_binding() {
    let contract = material_shader_binding_contract();
    assert_eq!(contract.len(), MATERIAL_BINDING_COUNT);
    assert_eq!(
        contract
            .iter()
            .map(|binding| binding.binding)
            .collect::<HashSet<_>>()
            .len(),
        MATERIAL_BINDING_COUNT
    );
    assert!(contract
        .iter()
        .enumerate()
        .all(|(index, binding)| binding.binding == index as u32));
}

#[test]
fn gpu_scene_shader_binding_contract_matches_the_draw_facing_subset() {
    let contract = gpu_scene_shader_binding_contract();
    assert_eq!(GPU_SCENE_DRAW_BIND_GROUP, 3);
    assert_eq!(contract.len(), GPU_SCENE_DRAW_BINDING_COUNT);
    assert_eq!(
        contract
            .iter()
            .map(|binding| binding.binding)
            .collect::<Vec<_>>(),
        vec![
            GPU_SCENE_PRIMITIVE_DATA_BINDING,
            GPU_SCENE_INSTANCE_DATA_BINDING,
            GPU_SCENE_LIGHT_DATA_BINDING,
            GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
            GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING,
        ]
    );
    assert!(contract.iter().all(|binding| {
        binding.resource_type == RenderShaderBindingResourceType::StorageBuffer
    }));
    assert_eq!(contract[3].allowed_visibility, GPU_SCENE_VERTEX_VISIBILITY);
    assert_eq!(contract[4].allowed_visibility, GPU_SCENE_VERTEX_VISIBILITY);
}
