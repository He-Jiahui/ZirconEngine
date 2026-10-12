use super::*;

#[test]
fn gpu_scene_shader_contract_keeps_dynamic_palette_minimums_late_bound() {
    let entries = gpu_scene_shader_contract_layout_entries();

    for binding in [
        GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
        GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING,
    ] {
        let entry = entries
            .iter()
            .find(|entry| entry.binding == binding)
            .expect("GPU Scene palette binding must exist");
        assert!(matches!(
            &entry.ty,
            wgpu::BindingType::Buffer {
                min_binding_size: None,
                ..
            }
        ));
    }
}
