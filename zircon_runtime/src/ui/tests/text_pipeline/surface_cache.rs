use super::fixtures::{fixed_constraints, repeated_text_metadata, text_layout_command_count};
use crate::ui::{
    layout::compute_layout_tree,
    surface::{extract_ui_render_tree, UiSurface},
};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{UiContainerKind, UiSize},
    tree::{UiTree, UiTreeNode},
};

#[test]
fn standalone_extract_reuses_one_operation_session_across_text_commands() {
    let mut tree = UiTree::new(UiTreeId::new("runtime.ui.text.one-shot-extract"));
    tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::BlockBox)
            .with_constraints(fixed_constraints(160.0, 48.0)),
    );
    for (node_id, path) in [
        (UiNodeId::new(2), "root/first"),
        (UiNodeId::new(3), "root/second"),
    ] {
        tree.insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(node_id, UiNodePath::new(path))
                .with_constraints(fixed_constraints(120.0, 18.0))
                .with_template_metadata(repeated_text_metadata()),
        )
        .expect("text child should be inserted");
    }
    compute_layout_tree(&mut tree, UiSize::new(160.0, 48.0))
        .expect("standalone layout should compute");

    let constructions_before = crate::text::current_thread_text_layout_session_construction_count();
    let extract = extract_ui_render_tree(&tree);
    let constructions_after = crate::text::current_thread_text_layout_session_construction_count();

    assert_eq!(
        extract
            .list
            .commands
            .iter()
            .filter(|command| command.text_layout.is_some())
            .count(),
        2
    );
    assert_eq!(
        constructions_after.saturating_sub(constructions_before),
        1,
        "one standalone extract owns one operation-local text session, not one per command"
    );
}

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

    let constructions_before = crate::text::current_thread_text_layout_session_construction_count();
    surface
        .compute_layout(UiSize::new(160.0, 48.0))
        .expect("surface layout should compute");
    let constructions_after = crate::text::current_thread_text_layout_session_construction_count();

    assert_eq!(text_layout_command_count(&surface), 2);
    assert_eq!(
        constructions_after.saturating_sub(constructions_before),
        0,
        "surface measure, layout, extract, and artifact preparation must reuse its retained session"
    );
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

#[test]
fn font_generation_change_rebuilds_the_retained_surface_without_a_new_layout_session() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let (_, database) = crate::text::font::shared_font_database_snapshot();
    let root_size = UiSize::new(160.0, 48.0);
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.text.font-generation-surface-owner",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::BlockBox)
            .with_constraints(fixed_constraints(160.0, 48.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/text"))
                .with_constraints(fixed_constraints(120.0, 18.0))
                .with_template_metadata(repeated_text_metadata()),
        )
        .expect("text child should be inserted");

    surface
        .compute_layout(root_size)
        .expect("initial surface layout should compute");
    let stable = surface
        .rebuild_dirty(root_size)
        .expect("stable surface rebuild should succeed");
    assert!(!stable.layout_recomputed);

    let published_generation = crate::text::font::force_publish_shared_font_database(&database);
    let constructions_before = crate::text::current_thread_text_layout_session_construction_count();
    let rebuilt = surface
        .rebuild_dirty(root_size)
        .expect("font generation recovery should rebuild the retained surface");
    let constructions_after = crate::text::current_thread_text_layout_session_construction_count();

    assert!(rebuilt.layout_recomputed);
    assert!(rebuilt.render_rebuilt);
    assert_eq!(text_layout_command_count(&surface), 1);
    assert_eq!(
        constructions_after.saturating_sub(constructions_before),
        0,
        "font generation recovery must reuse the surface-owned text layout session"
    );
    assert!(surface.render_extract.list.commands.iter().all(|command| {
        command
            .text_layout
            .as_ref()
            .and_then(|layout| layout.rich_text_artifact.as_ref())
            .and_then(crate::text::resolve_resolved_text_glyph_artifact)
            .is_none_or(|artifact| artifact.font_generation == published_generation)
    }));
}

#[test]
fn retained_surface_observes_only_its_owned_font_collection_generation() {
    let database = crate::text::font::runtime_default_font_database_for_test();
    let owned_collection =
        crate::text::font::FontCollectionService::from_database(database.clone());
    let foreign_collection = crate::text::font::FontCollectionService::from_database(database);
    assert_ne!(owned_collection.revision(), foreign_collection.revision());
    let root_size = UiSize::new(160.0, 48.0);
    let mut surface = UiSurface::new_with_font_collection(
        UiTreeId::new("runtime.ui.text.owned-font-collection"),
        std::sync::Arc::clone(&owned_collection),
    );
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::BlockBox)
            .with_constraints(fixed_constraints(160.0, 48.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/text"))
                .with_constraints(fixed_constraints(120.0, 18.0))
                .with_template_metadata(repeated_text_metadata()),
        )
        .expect("text child should be inserted");

    surface
        .compute_layout(root_size)
        .expect("initial surface layout should compute");
    foreign_collection.mutate(|database| {
        assert!(database.set_default_ui_family("Foreign Collection Family"));
    });
    let foreign_change = surface
        .rebuild_dirty(root_size)
        .expect("foreign font collection changes should not invalidate the surface");
    assert!(!foreign_change.layout_recomputed);

    let constructions_before = crate::text::current_thread_text_layout_session_construction_count();
    let (owned_generation, _, changed) = owned_collection
        .mutate(|database| database.set_default_ui_family("Owned Collection Family"));
    assert!(changed);
    let owned_change = surface
        .rebuild_dirty(root_size)
        .expect("owned font collection changes should rebuild the surface");
    let constructions_after = crate::text::current_thread_text_layout_session_construction_count();

    assert!(owned_change.layout_recomputed);
    assert_eq!(
        constructions_after.saturating_sub(constructions_before),
        0,
        "owned font collection invalidation must retain the surface layout session"
    );
    assert_eq!(
        surface.text_measure_cache.font_database_generation(),
        owned_generation
    );
    assert!(surface.render_extract.list.commands.iter().all(|command| {
        command
            .text_layout
            .as_ref()
            .and_then(|layout| layout.rich_text_artifact.as_ref())
            .and_then(crate::text::resolve_resolved_text_glyph_artifact)
            .is_none_or(|artifact| artifact.font_lease.revision() == owned_collection.revision())
    }));
}
