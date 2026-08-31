use super::*;

#[test]
fn render_cache_patches_position_only_geometry_without_reextracting() {
    let node_id = UiNodeId::new(11);
    let old_frame = UiFrame::new(4.0, 8.0, 12.0, 16.0);
    let mut arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.cache.geometry"),
        roots: vec![node_id].into(),
        nodes: vec![zircon_runtime_interface::ui::surface::UiArrangedNode {
            node_id,
            node_path: UiNodePath::new("root/node"),
            parent: None,
            children: Vec::new(),
            frame: old_frame,
            clip_frame: old_frame,
            z_index: 0,
            paint_order: 0,
            visibility: UiVisibility::Visible,
            input_policy: UiInputPolicy::Receive,
            pointer_events: Default::default(),
            enabled: true,
            clickable: false,
            hoverable: false,
            focusable: false,
            clip_to_bounds: false,
            control_id: None,
            slot: None,
        }]
        .into(),
        draw_order: vec![node_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let mut cache = UiSurfaceRenderCache::default();
    let mut owner_command = quad(11, old_frame);
    owner_command.clip_frame = Some(old_frame);
    let arranged_node_indices = BTreeMap::from([(node_id, 0)]);
    let first = cache.update_for_arranged(
        &extract(Vec::new()),
        extract(vec![owner_command]),
        false,
        &arranged_tree,
        &arranged_node_indices,
    );
    let mut patched_extract = first.extract;
    arranged_tree.nodes[0].frame.x = 24.0;
    arranged_tree.nodes[0].clip_frame.x = 24.0;

    let stats = cache
        .patch_geometry(
            &mut patched_extract,
            &arranged_tree,
            &arranged_node_indices,
            &BTreeSet::from([node_id]),
        )
        .expect("position-only geometry should patch");

    assert_eq!(stats.rebuilt_command_count, 0);
    assert_eq!(stats.reused_command_count, 1);
    assert_eq!(stats.damage_rect_count, 1);
    assert_eq!(patched_extract.list.commands[0].frame.x, 24.0);
}

#[test]
fn render_cache_rejects_text_layout_geometry_patch_without_mutating_extract() {
    let node_id = UiNodeId::new(14);
    let old_frame = UiFrame::new(4.0, 8.0, 12.0, 16.0);
    let mut arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.cache.text-geometry"),
        roots: vec![node_id].into(),
        nodes: vec![zircon_runtime_interface::ui::surface::UiArrangedNode {
            node_id,
            node_path: UiNodePath::new("root/text"),
            parent: None,
            children: Vec::new(),
            frame: old_frame,
            clip_frame: old_frame,
            z_index: 0,
            paint_order: 0,
            visibility: UiVisibility::Visible,
            input_policy: UiInputPolicy::Receive,
            pointer_events: Default::default(),
            enabled: true,
            clickable: false,
            hoverable: false,
            focusable: false,
            clip_to_bounds: false,
            control_id: None,
            slot: None,
        }]
        .into(),
        draw_order: vec![node_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let arranged_node_indices = BTreeMap::from([(node_id, 0)]);
    let mut text_command = quad(14, old_frame);
    text_command.clip_frame = Some(old_frame);
    text_command.text_layout = Some(Default::default());
    let mut cache = UiSurfaceRenderCache::default();
    let first = cache.update_for_arranged(
        &extract(Vec::new()),
        extract(vec![text_command]),
        false,
        &arranged_tree,
        &arranged_node_indices,
    );
    let mut retained_extract = first.extract;
    let before = retained_extract.clone();
    arranged_tree.nodes[0].frame.x = 24.0;
    arranged_tree.nodes[0].clip_frame.x = 24.0;

    assert_eq!(
        cache.patch_geometry(
            &mut retained_extract,
            &arranged_tree,
            &arranged_node_indices,
            &BTreeSet::from([node_id]),
        ),
        Err(())
    );
    assert_eq!(retained_extract, before);
}

#[test]
fn render_cache_rejects_single_command_with_non_owner_geometry() {
    let node_id = UiNodeId::new(12);
    let owner_frame = UiFrame::new(4.0, 8.0, 12.0, 16.0);
    let arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.cache.non-owner-geometry"),
        roots: vec![node_id].into(),
        nodes: vec![zircon_runtime_interface::ui::surface::UiArrangedNode {
            node_id,
            node_path: UiNodePath::new("root/node"),
            parent: None,
            children: Vec::new(),
            frame: owner_frame,
            clip_frame: owner_frame,
            z_index: 0,
            paint_order: 0,
            visibility: UiVisibility::Visible,
            input_policy: UiInputPolicy::Receive,
            pointer_events: Default::default(),
            enabled: true,
            clickable: false,
            hoverable: false,
            focusable: false,
            clip_to_bounds: false,
            control_id: None,
            slot: None,
        }]
        .into(),
        draw_order: vec![node_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let arranged_node_indices = BTreeMap::from([(node_id, 0)]);
    let mut command = quad(12, UiFrame::new(6.0, 8.0, 12.0, 16.0));
    command.clip_frame = Some(owner_frame);
    let mut cache = UiSurfaceRenderCache::default();
    let first = cache.update_for_arranged(
        &extract(Vec::new()),
        extract(vec![command]),
        false,
        &arranged_tree,
        &arranged_node_indices,
    );
    let mut patched_extract = first.extract;

    assert_eq!(
        cache.patch_geometry(
            &mut patched_extract,
            &arranged_tree,
            &arranged_node_indices,
            &BTreeSet::from([node_id]),
        ),
        Err(())
    );
}

#[test]
fn local_reextract_keeps_exact_owner_command_geometry_patchable() {
    let node_id = UiNodeId::new(13);
    let old_frame = UiFrame::new(4.0, 8.0, 12.0, 16.0);
    let mut arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.cache.local-reextract"),
        roots: vec![node_id].into(),
        nodes: vec![zircon_runtime_interface::ui::surface::UiArrangedNode {
            node_id,
            node_path: UiNodePath::new("root/node"),
            parent: None,
            children: Vec::new(),
            frame: old_frame,
            clip_frame: old_frame,
            z_index: 0,
            paint_order: 0,
            visibility: UiVisibility::Visible,
            input_policy: UiInputPolicy::Receive,
            pointer_events: Default::default(),
            enabled: true,
            clickable: false,
            hoverable: false,
            focusable: false,
            clip_to_bounds: false,
            control_id: None,
            slot: None,
        }]
        .into(),
        draw_order: vec![node_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let arranged_node_indices = BTreeMap::from([(node_id, 0)]);
    let mut owner_command = quad(13, old_frame);
    owner_command.clip_frame = Some(old_frame);
    let mut cache = UiSurfaceRenderCache::default();
    let first = cache.update_for_arranged(
        &extract(Vec::new()),
        extract(vec![owner_command.clone()]),
        false,
        &arranged_tree,
        &arranged_node_indices,
    );
    let mut retained_extract = first.extract;

    cache
        .patch_nodes(
            &mut retained_extract,
            &BTreeSet::from([node_id]),
            extract(vec![owner_command]),
            &arranged_tree,
            &arranged_node_indices,
        )
        .expect("same owner command should reextract locally");
    arranged_tree.nodes[0].frame.x = 24.0;
    arranged_tree.nodes[0].clip_frame.x = 24.0;

    cache
        .patch_geometry(
            &mut retained_extract,
            &arranged_tree,
            &arranged_node_indices,
            &BTreeSet::from([node_id]),
        )
        .expect("local reextract should preserve later geometry patching");
    assert_eq!(retained_extract.list.commands[0].frame.x, 24.0);
}
