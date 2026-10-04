use crate::render_graph::RenderGraphComputePipelineFamily;

use super::{ComputePipelineBindingLayout, ComputePipelineFamilyKey};

#[test]
fn family_key_isolates_interface_workgroup_and_binding_abi() {
    let baseline = ComputePipelineFamilyKey::new(
        RenderGraphComputePipelineFamily::new("ambient-occlusion.evaluate", 2),
        "cs_main",
        [8, 8, 1],
        &[ComputePipelineBindingLayout::uniform_buffer(0)],
    );
    let same = ComputePipelineFamilyKey::new(
        RenderGraphComputePipelineFamily::new("ambient-occlusion.evaluate", 2),
        "cs_main",
        [8, 8, 1],
        &[ComputePipelineBindingLayout::uniform_buffer(0)],
    );
    let changed_interface = ComputePipelineFamilyKey::new(
        RenderGraphComputePipelineFamily::new("ambient-occlusion.evaluate", 3),
        "cs_main",
        [8, 8, 1],
        &[ComputePipelineBindingLayout::uniform_buffer(0)],
    );
    let changed_workgroup = ComputePipelineFamilyKey::new(
        RenderGraphComputePipelineFamily::new("ambient-occlusion.evaluate", 2),
        "cs_main",
        [16, 8, 1],
        &[ComputePipelineBindingLayout::uniform_buffer(0)],
    );
    let changed_binding = ComputePipelineFamilyKey::new(
        RenderGraphComputePipelineFamily::new("ambient-occlusion.evaluate", 2),
        "cs_main",
        [8, 8, 1],
        &[ComputePipelineBindingLayout::storage_buffer_read(0)],
    );

    assert_eq!(baseline, same);
    assert_ne!(baseline, changed_interface);
    assert_ne!(baseline, changed_workgroup);
    assert_ne!(baseline, changed_binding);
}
