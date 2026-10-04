use super::*;
use std::sync::Arc;
#[test]
fn actual_resize_geometry_patch_carries_current_published_source_frame() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let metrics = EditorWorkbenchReferenceMetrics::default();
    let mut template = build_editor_workbench_template_surface(&runtime, metrics).unwrap();
    let previous = template.surface.surface_frame();
    template.mark_host_projection_committed();
    template
        .recompute_layout(
            &runtime,
            UiSize::new(metrics.target_width - 80.0, metrics.target_height - 40.0),
        )
        .unwrap();
    let current = template.surface.surface_frame();
    assert!(!Arc::ptr_eq(&previous, &current));
    let indices = template
        .pending_host_projection_geometry_patch_indices()
        .unwrap();
    assert!(!indices.is_empty());
    for index in indices {
        let row = &template.host_projection.nodes[index];
        assert!(Arc::ptr_eq(
            row.source_surface_frame.as_ref().unwrap(),
            &current
        ));
        let arranged = current
            .arranged_tree
            .get(row.surface_node_id.unwrap())
            .unwrap();
        assert_eq!(row.frame, arranged.frame);
        assert_eq!(row.clip_frame, Some(arranged.clip_frame));
    }
    assert_eq!(template.last_host_projection_semantic_patch_count(), 0);
}
