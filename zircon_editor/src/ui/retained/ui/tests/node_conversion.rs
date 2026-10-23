use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::*;

struct CloneProbe(Arc<AtomicUsize>);

impl Clone for CloneProbe {
    fn clone(&self) -> Self {
        self.0.fetch_add(1, Ordering::Relaxed);
        Self(Arc::clone(&self.0))
    }
}

#[test]
fn model_mapping_borrows_source_rows() {
    let clone_count = Arc::new(AtomicUsize::new(0));
    let source = model_rc(vec![CloneProbe(Arc::clone(&clone_count))]);

    let mapped = map_model_rc(&source, |_| 7_u8);

    assert_eq!(mapped.row_data(0), Some(7));
    assert_eq!(clone_count.load(Ordering::Relaxed), 0);
}

#[test]
fn view_template_node_conversion_preserves_v2_interaction_metadata() {
    let data = ViewTemplateNodeData {
        node_id: "asset/search".into(),
        control_id: "SearchEdited".into(),
        role: "InputField".into(),
        component_role: "input-field".into(),
        component_variant: "outlined".into(),
        value_text: "albedo".into(),
        value_number: 42.0,
        value_percent: 0.42,
        options: model_rc(vec![
            "Name".into(),
            "Type".into(),
            "Size".into(),
            "Rev".into(),
        ]),
        z_index: 17,
        transition_kind: "fade".into(),
        transition_in: true,
        transition_entered: false,
        transition_progress: 0.5,
        transition_duration_ms: 225,
        transition_easing: "cubic-bezier(0.4, 0, 0.2, 1)".into(),
        popup_open: true,
        dispatch_kind: "asset".into(),
        binding_id: "AssetSurface/SearchEdited".into(),
        edit_action_id: "workbench.asset.search.edit".into(),
        commit_action_id: "workbench.asset.search.commit".into(),
        ..ViewTemplateNodeData::default()
    };

    let node = to_host_contract_template_node(&data);

    assert_eq!(node.component_role.as_str(), "input-field");
    assert_eq!(node.component_variant.as_str(), "outlined");
    assert_eq!(node.value_text.as_str(), "albedo");
    assert_eq!(node.value_number, 42.0);
    assert_eq!(node.value_percent, 0.42);
    assert_eq!(node.options_text.as_str(), "Name, Type, Size, Rev");
    assert_eq!(node.options.row_count(), 4);
    assert_eq!(node.options.row_data(0).as_deref(), Some("Name"));
    assert_eq!(node.options.row_data(3).as_deref(), Some("Rev"));
    assert_eq!(node.z_index, 17);
    assert_eq!(node.transition_kind.as_str(), "fade");
    assert!(node.transition_in);
    assert!(!node.transition_entered);
    assert_eq!(node.transition_progress, 0.5);
    assert_eq!(node.transition_duration_ms, 225);
    assert_eq!(
        node.transition_easing.as_str(),
        "cubic-bezier(0.4, 0, 0.2, 1)"
    );
    assert!(node.popup_open);
    assert_eq!(node.dispatch_kind.as_str(), "asset");
    assert_eq!(node.binding_id.as_str(), "AssetSurface/SearchEdited");
    assert_eq!(node.edit_action_id.as_str(), "workbench.asset.search.edit");
    assert_eq!(
        node.commit_action_id.as_str(),
        "workbench.asset.search.commit"
    );
}

#[test]
fn owned_view_template_node_conversion_matches_the_borrowed_projection() {
    let data = ViewTemplateNodeData {
        node_id: "asset/owned".into(),
        surface_node_id: Some(zircon_runtime_interface::ui::event_ui::UiNodeId::new(27)),
        surface_render_command_ref: Some(
            zircon_runtime_interface::ui::surface::UiRenderFrameCommandRef::new(
                zircon_runtime_interface::ui::event_ui::UiNodeId::new(27),
                3,
            ),
        ),
        control_id: "OwnedEdited".into(),
        role: "InputField".into(),
        text: "Owned".into(),
        component_role: "input-field".into(),
        component_variant: "outlined".into(),
        value_text: "value".into(),
        options: model_rc(vec!["First".into(), "Second".into()]),
        collection_items: model_rc(vec!["user|Question".into(), "agent|Answer".into()]),
        transition_kind: "fade".into(),
        transition_easing: "linear".into(),
        transition_direction: "in".into(),
        dispatch_kind: "asset".into(),
        action_id: "asset.action".into(),
        binding_id: "AssetSurface/OwnedEdited".into(),
        media_source: "asset://preview".into(),
        icon_name: "search".into(),
        value_number: 7.0,
        selected: true,
        focused: true,
        frame: ViewTemplateFrameData {
            x: 1.0,
            y: 2.0,
            width: 300.0,
            height: 40.0,
        },
        ..ViewTemplateNodeData::default()
    };
    let borrowed = to_host_contract_template_node(&data);
    let surface_node_id = data.surface_node_id;
    let surface_render_command_ref = data.surface_render_command_ref;

    let owned = to_host_contract_template_node_owned(data);

    assert_eq!(owned.node_id, borrowed.node_id);
    assert_eq!(owned.surface_node_id, borrowed.surface_node_id);
    assert_eq!(owned.surface_node_id, surface_node_id);
    assert_eq!(
        owned.surface_render_command_ref,
        borrowed.surface_render_command_ref
    );
    assert_eq!(owned.surface_render_command_ref, surface_render_command_ref);
    assert_eq!(owned.control_id, borrowed.control_id);
    assert_eq!(owned.text, borrowed.text);
    assert_eq!(owned.value_text, borrowed.value_text);
    assert_eq!(owned.options_text, borrowed.options_text);
    assert_eq!(owned.options.row_count(), borrowed.options.row_count());
    assert_eq!(owned.collection_items.row_count(), 2);
    assert_eq!(owned.collection_items, borrowed.collection_items);
    assert_eq!(
        owned.collection_items.row_data(1).as_deref(),
        Some("agent|Answer")
    );
    assert_eq!(owned.transition_kind, borrowed.transition_kind);
    assert_eq!(owned.dispatch_kind, borrowed.dispatch_kind);
    assert_eq!(owned.media_source, borrowed.media_source);
    assert_eq!(owned.icon_name, borrowed.icon_name);
    assert_eq!(owned.value_number, borrowed.value_number);
    assert_eq!(owned.selected, borrowed.selected);
    assert_eq!(owned.focused, borrowed.focused);
    assert_eq!(owned.frame.x, borrowed.frame.x);
    assert_eq!(owned.frame.y, borrowed.frame.y);
    assert_eq!(owned.frame.width, borrowed.frame.width);
    assert_eq!(owned.frame.height, borrowed.frame.height);
}

#[test]
fn unreal_header_conversion_retains_consumed_source_frame_and_live_instance_input() {
    use crate::ui::layouts::views::{
        build_view_template_node_projection_with_patches, compose_view_template_node_model,
        ViewTemplateNodePatch,
    };
    use std::collections::BTreeMap;
    use zircon_runtime_interface::ui::layout::UiSize;

    let captions = BTreeMap::from([
        (
            "DockTab0".to_owned(),
            "A long authored document caption".to_owned(),
        ),
        ("DockSubtitle".to_owned(), "Actual project".to_owned()),
    ]);
    let patch = ViewTemplateNodePatch {
        selected: Some(true),
        surface_variant: Some("inset".into()),
        text_tone: Some("default".into()),
        ..ViewTemplateNodePatch::default()
    };
    let patches = BTreeMap::from([
        ("DockTab0".to_owned(), patch.clone()),
        ("DockTabGroup0".to_owned(), patch),
    ]);
    let projection = build_view_template_node_projection_with_patches(
        "test.unreal.header.consumed",
        "/assets/ui/editor/unreal/components/dock_header.zui",
        &[],
        UiSize::new(900.0, 31.0),
        &captions,
        &patches,
    )
    .expect("the actual imported Unreal Header builds");
    let source = projection
        .unreal_dock_header_source()
        .expect("actual Header source context");
    let captured = Arc::clone(&source);
    let model = compose_view_template_node_model(
        "test.unreal.header.consumed.model",
        projection,
        &1_u8,
        move |_| Some(captured),
    );
    let converted = to_host_contract_template_nodes(&model);
    let carried = converted
        .metadata::<Option<Arc<crate::ui::layouts::views::UnrealDockHeaderSource>>>()
        .and_then(Option::as_ref)
        .expect("actual metadata survived the ordinary converter");
    assert!(Arc::ptr_eq(&carried.frame, &source.frame));
    assert!(!source.source_receipts.is_empty());
    assert_eq!(source.document.nodes["dock_tab_strip"].children.len(), 1);
    assert!(!source.document.nodes.contains_key("dock_tab_group_1"));
    assert_eq!(
        source.document.nodes["dock_tab_0"].props["text"].as_str(),
        Some("A long authored document caption"),
    );
    for node in converted.iter() {
        let actual = node
            .source_surface_frame
            .as_ref()
            .expect("real arranged Source node");
        assert!(Arc::ptr_eq(actual, &source.frame));
        assert_eq!(
            node.source_path.as_str(),
            "zircon_editor/assets/ui/editor/unreal/components/dock_header.zui"
        );
        assert!(!node.source_node_id.is_empty());
        assert_eq!(node.instance_path.as_str(), "[]");
        let id = node.surface_node_id.expect("actual Source node id");
        assert!(actual.arranged_tree.get(id).is_some());
        if let Some(reference) = node.surface_render_command_ref {
            assert_eq!(reference.node_id, id);
            assert!(actual.render_extract.command_by_ref(reference).is_some());
        }
    }
    let first = converted
        .iter()
        .find(|node| node.control_id.as_str() == "DockTab0")
        .expect("actual caption row");
    assert!(first.selected);
    assert_eq!(first.font_weight, 400);
    let invalid = model.map_preserving_metadata(|row| {
        let mut row = row.clone();
        row.surface_node_id = None;
        row
    });
    assert!(to_host_contract_template_nodes(&invalid)
        .row_data(0)
        .unwrap()
        .source_surface_frame
        .is_none());

    let changed = BTreeMap::from([
        ("DockTab0".to_owned(), "Changed caption".to_owned()),
        ("DockSubtitle".to_owned(), "Actual project".to_owned()),
    ]);
    let next = build_view_template_node_projection_with_patches(
        "test.unreal.header.consumed",
        "/assets/ui/editor/unreal/components/dock_header.zui",
        &[],
        UiSize::new(900.0, 31.0),
        &changed,
        &patches,
    )
    .expect("ordinary retained caption mutation");
    assert_eq!(
        next.unreal_dock_header_source().unwrap().document.nodes["dock_tab_0"].props["text"]
            .as_str(),
        Some("Changed caption")
    );
    assert_eq!(
        carried.document.nodes["dock_tab_0"].props["text"].as_str(),
        Some("A long authored document caption")
    );
}

#[test]
fn welcome_form_captions_survive_composited_template_surfaces() {
    use crate::ui::layouts::views::welcome_pane_nodes;
    use crate::ui::retained_host::paint_template_nodes_for_test;
    use zircon_runtime_interface::ui::layout::UiSize;
    let source = welcome_pane_nodes(UiSize::new(316.0, 242.0));
    let nodes = to_host_contract_template_nodes(&source);
    let pixels = paint_template_nodes_for_test(316, 242, nodes.clone());
    for control in ["WelcomeProjectNameLabel", "WelcomeLocationLabel"] {
        let label = nodes
            .iter()
            .find(|node| node.control_id == control)
            .expect("caption");
        let mut colors = std::collections::BTreeSet::new();
        for y in
            label.frame.y.ceil() as usize..(label.frame.y + label.frame.height).floor() as usize
        {
            for x in
                label.frame.x.ceil() as usize..(label.frame.x + label.frame.width).floor() as usize
            {
                let offset = (y * 316 + x) * 4;
                colors.insert(&pixels[offset..offset + 4]);
            }
        }
        assert!(
            colors.len() > 3,
            "{control} must paint glyphs above the enclosing surfaces"
        );
    }
}
