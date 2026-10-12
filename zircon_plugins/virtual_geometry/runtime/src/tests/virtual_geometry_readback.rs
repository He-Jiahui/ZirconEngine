use super::*;

#[test]
fn take_neutral_readback_outputs_projects_and_consumes_gpu_readback() {
    let mut outputs = VirtualGeometryReadbackOutputs::default();
    outputs.store_gpu_readback(Some(VirtualGeometryGpuReadback::new(
        vec![(44, 6)],
        vec![44],
        vec![(44, 6)],
        vec![(44, 11)],
    )));

    let neutral = outputs.take_neutral_readback_outputs();

    assert_eq!(neutral.page_table_entries, vec![44, 6]);
    assert_eq!(neutral.completed_page_assignments[0].page_id, 44);
    assert_eq!(neutral.completed_page_assignments[0].physical_slot, 6);
    assert_eq!(neutral.page_replacements[0].old_page_id, 11);
    assert_eq!(neutral.page_replacements[0].new_page_id, 44);
    assert_eq!(neutral.page_replacements[0].physical_slot, 6);
    assert_eq!(
        outputs.take_neutral_readback_outputs(),
        RenderVirtualGeometryReadbackOutputs::default()
    );
}

#[test]
fn take_neutral_readback_outputs_projects_node_cluster_cull_without_gpu_completion() {
    let mut outputs = VirtualGeometryReadbackOutputs::default();
    outputs.store_node_cluster_cull_readback(RenderVirtualGeometryNodeClusterCullReadbackOutputs {
        page_request_ids: vec![300, 301],
        ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
    });

    let neutral = outputs.take_neutral_readback_outputs();

    assert_eq!(neutral.node_cluster_cull.page_request_ids, vec![300, 301]);
    assert!(neutral.page_table_entries.is_empty());
    assert_eq!(
        outputs.take_neutral_readback_outputs(),
        RenderVirtualGeometryReadbackOutputs::default()
    );
}
