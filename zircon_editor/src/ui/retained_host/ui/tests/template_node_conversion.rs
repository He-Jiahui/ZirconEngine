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
