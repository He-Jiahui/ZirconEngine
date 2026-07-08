use super::fixtures::{fixed_constraints, repeated_text_metadata, text_layout_command_count};
use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{UiContainerKind, UiSize},
    tree::UiTreeNode,
};

#[test]
fn text_measure_cache_is_consumed_by_surface_render_rebuild() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.measure_cache.surface"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::BlockBox)
            .with_constraints(fixed_constraints(160.0, 48.0)),
    );
    for (node_id, path) in [
        (UiNodeId::new(2), "root/first"),
        (UiNodeId::new(3), "root/second"),
    ] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(node_id, UiNodePath::new(path))
                    .with_constraints(fixed_constraints(120.0, 18.0))
                    .with_template_metadata(repeated_text_metadata()),
            )
            .expect("text child should be inserted");
    }

    surface
        .compute_layout(UiSize::new(160.0, 48.0))
        .expect("surface layout should compute");

    assert_eq!(text_layout_command_count(&surface), 2);
    assert_eq!(
        surface.text_measure_cache.frame_shape_count(),
        2,
        "distinct text node frames should not reuse absolute text line geometry"
    );

    surface.rebuild();

    assert_eq!(text_layout_command_count(&surface), 2);
    assert_eq!(
        surface.text_measure_cache.frame_shape_count(),
        0,
        "unchanged surface rebuild should hit retained text measure cache entries"
    );
}

#[test]
fn text_surface_cache_frame_spans_layout_measure_and_render_extract() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.cache.frame-span"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::BlockBox)
            .with_constraints(fixed_constraints(160.0, 48.0)),
    );
    for (node_id, path) in [
        (UiNodeId::new(2), "root/first"),
        (UiNodeId::new(3), "root/second"),
    ] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(node_id, UiNodePath::new(path))
                    .with_constraints(fixed_constraints(120.0, 18.0))
                    .with_template_metadata(repeated_text_metadata()),
            )
            .expect("text child should be inserted");
    }

    surface
        .compute_layout(UiSize::new(160.0, 48.0))
        .expect("surface layout should compute");

    let measure_report = surface.text_measure_cache.frame_measure_report();
    let measure_dedup_report = surface.text_measure_cache.frame_measure_dedup_report();
    let layout_report = surface.text_measure_cache.frame_layout_report();
    let layout_dedup_report = surface.text_measure_cache.frame_layout_dedup_report();

    assert_eq!(
        measure_report.miss_count, 1,
        "two identical metadata text leaves should only populate the persistent measurement cache once"
    );
    assert_eq!(
        measure_dedup_report.hit_count, 1,
        "the second identical leaf should hit the same-frame text measurement dedup table"
    );
    assert_eq!(
        layout_report.miss_count, 2,
        "render extract still needs two absolute layout resolutions for distinct arranged frames"
    );
    assert_eq!(
        layout_dedup_report.hit_count, 0,
        "different arranged frames must not be same-frame deduped as identical layout resolutions"
    );
    assert_eq!(
        measure_report.frame_index, layout_report.frame_index,
        "layout measurement and render extraction must belong to the same text cache frame"
    );
}
