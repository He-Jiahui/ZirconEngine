use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::UiTemplateActionInvocation,
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{UiContainerKind, UiScrollState, UiScrollableBoxConfig, UiVirtualListConfig},
    template::{UiActionRef, UiBindingMissingValuePolicy},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
    v2::UiV2Repeat,
};

use crate::ui::template_runtime::{
    RetainedUiHostModel, RetainedUiNodeProjection, RetainedUiProjection,
};

use super::{build_host_model_with_surface, resolve_template_action};

#[test]
fn resolves_typed_action_payload_from_a_control_property_snapshot() {
    let action = UiActionRef {
        route: Some("plugin.operation".to_string()),
        action: None,
        payload: BTreeMap::from([(
            "entity".to_string(),
            Value::String("=control.RowList.prop.selected_row_identity".to_string()),
        )]),
        payload_missing_policy: Default::default(),
    };
    let control_attributes = BTreeMap::from([(
        "RowList".to_string(),
        BTreeMap::from([("selected_row_identity".to_string(), Value::Integer(73))]),
    )]);

    assert_eq!(
        resolve_template_action(&action, &BTreeMap::new(), &control_attributes),
        Some(UiTemplateActionInvocation::route(
            "plugin.operation",
            BTreeMap::from([("entity".to_string(), UiValue::Int(73))]),
        ))
    );
}

#[test]
fn authored_editor_action_keeps_action_identity_without_a_route_alias() {
    let action = UiActionRef {
        route: None,
        action: Some("view.console.clear".to_string()),
        payload: BTreeMap::new(),
        payload_missing_policy: Default::default(),
    };

    assert_eq!(
        resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new()),
        Some(UiTemplateActionInvocation::action("view.console.clear"))
    );
}

#[test]
fn authored_action_and_route_aliases_are_rejected_as_ambiguous() {
    let action = UiActionRef {
        route: Some("view.console.clear".to_string()),
        action: Some("view.console.clear".to_string()),
        payload: BTreeMap::new(),
        payload_missing_policy: Default::default(),
    };

    assert_eq!(
        resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new()),
        None
    );
}

#[test]
fn authored_editor_action_with_route_payload_is_rejected() {
    let action = UiActionRef {
        route: None,
        action: Some("view.console.clear".to_string()),
        payload: BTreeMap::from([(
            "legacy_route_argument".to_string(),
            toml::Value::Boolean(true),
        )]),
        payload_missing_policy: Default::default(),
    };

    assert_eq!(
        resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new()),
        None
    );
}

#[test]
fn source_action_missing_value_policy_distinguishes_omit_substitute_and_reject() {
    let mut action = UiActionRef {
        route: Some("plugin.operation".to_string()),
        action: None,
        payload: BTreeMap::from([(
            "entity".to_string(),
            Value::String("=prop.missing".to_string()),
        )]),
        payload_missing_policy: UiBindingMissingValuePolicy::Optional,
    };

    let optional = resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new())
        .expect("optional missing payload should preserve its route");
    assert!(optional.payload.is_empty());

    action.payload_missing_policy = UiBindingMissingValuePolicy::Fallback {
        value: UiValue::Int(73),
    };
    assert_eq!(
        resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new())
            .and_then(|invocation| invocation.payload.get("entity").cloned()),
        Some(UiValue::Int(73))
    );

    action.payload_missing_policy = UiBindingMissingValuePolicy::Error;
    assert!(resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new()).is_none());
}

#[test]
fn console_editor_commands_are_authored_as_actions_not_routes() {
    let source = include_str!("../../../../../assets/ui/editor/host/console_body.zui");

    for command_id in [
        "view.console.filter.all",
        "view.console.filter.error",
        "view.console.filter.warning",
        "view.console.filter.info",
        "view.console.source.all",
        "view.console.source.editor",
        "view.console.source.runtime",
        "view.console.source.play",
        "view.console.source.plugin",
        "view.console.source.import",
        "view.console.source.script_build",
        "view.console.clear",
    ] {
        assert!(
            source.contains(&format!("action = {{ action = \"{command_id}\" }}")),
            "{command_id} must use UiActionRef.action"
        );
        assert!(
            !source.contains(&format!("route = \"{command_id}\"")),
            "{command_id} must not retain a route alias"
        );
    }
}

#[test]
fn host_projection_indexes_bindings_by_reference() {
    let source = include_str!("../projection.rs");
    let builders = source
        .split("pub(super) fn build_host_model")
        .nth(1)
        .expect("host model builders")
        .split("fn merge_projection_only_host_nodes")
        .next()
        .expect("host model builder bodies");
    let cloned_rows = [".cloned", "()"].concat();

    assert!(!builders.contains(&cloned_rows));
}

fn repeat_projection_node(
    source_node_id: &str,
    control_id: Option<&str>,
    children: Vec<RetainedUiNodeProjection>,
) -> RetainedUiNodeProjection {
    RetainedUiNodeProjection {
        component: "Container".into(),
        control_id: control_id.map(str::to_owned),
        source_path: Some("fixtures/formal-repeat.zui".into()),
        source_node_id: Some(source_node_id.into()),
        instance_path: Some(Vec::new()),
        attributes: BTreeMap::new(),
        style_tokens: BTreeMap::new(),
        binding_ids: Vec::new(),
        children,
    }
}

fn repeat_surface_node(id: u64, path: &str, source: &RetainedUiNodeProjection) -> UiTreeNode {
    UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(path)).with_template_metadata(
        UiTemplateNodeMetadata {
            component: source.component.clone(),
            control_id: source.control_id.clone(),
            source_path: source.source_path.clone(),
            source_node_id: source.source_node_id.clone(),
            instance_path: source.instance_path.clone(),
            attributes: source.attributes.clone(),
            ..Default::default()
        },
    )
}

fn repeat_projection_fixture() -> (UiSurface, RetainedUiProjection) {
    let label = repeat_projection_node("label", Some("AuthoredRowLabel"), Vec::new());
    let decoration = repeat_projection_node("decoration", None, vec![label.clone()]);
    let prototype =
        repeat_projection_node("prototype", Some("AuthoredRow"), vec![decoration.clone()]);
    let mut injected = repeat_projection_node("injected", Some("InjectedHidden"), Vec::new());
    injected
        .attributes
        .insert("visibility".into(), Value::String("collapsed".into()));
    let mut root = repeat_projection_node(
        "owner",
        Some("RepeatOwner"),
        vec![prototype.clone(), injected],
    );
    root.attributes.insert(
        "repeat".into(),
        UiV2Repeat {
            kind: "virtual_rows".into(),
            prototype: "AuthoredRow".into(),
            virtual_control_prefix: "VirtualRow".into(),
            authored_count: 1,
            node_path_namespace: "surface".into(),
        }
        .metadata_value(),
    );

    let mut surface = UiSurface::new(UiTreeId::new("formal-repeat-projection"));
    surface.tree.insert_root(
        repeat_surface_node(1, "surface/owner", &root)
            .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                virtualization: Some(UiVirtualListConfig {
                    item_extent: 24.0,
                    overscan: 0,
                }),
                ..UiScrollableBoxConfig::default()
            }))
            .with_scroll_state(UiScrollState {
                viewport_extent: 72.0,
                content_extent: 72.0,
                ..UiScrollState::default()
            }),
    );
    for (parent, node) in [
        (1, repeat_surface_node(2, "surface/prototype", &prototype)),
        (2, repeat_surface_node(3, "surface/decoration", &decoration)),
        (3, repeat_surface_node(4, "surface/label", &label)),
    ] {
        surface
            .tree
            .insert_child(UiNodeId::new(parent), node)
            .unwrap();
    }
    (
        surface,
        RetainedUiProjection {
            document_id: "fixtures/formal-repeat.zui".into(),
            root,
            bindings: Vec::new(),
        },
    )
}

fn materialize_repeat_projection(surface: &mut UiSurface, logical_count: usize) {
    surface
        .reconcile_virtual_list_materialization(UiNodeId::new(1), logical_count, &mut Vec::new())
        .unwrap();
    surface
        .ensure_virtual_list_prototype_slots(UiNodeId::new(1), UiNodeId::new(2), |node, context| {
            let metadata = node.template_metadata.as_mut().unwrap();
            if let Some(control_id) = metadata.control_id.as_mut() {
                *control_id = format!("Materialized{}{}", control_id.as_str(), context.slot_index);
            }
            node.node_path = UiNodePath::new(format!(
                "surface/virtual-slot-{}/{}",
                context.slot_index, context.prototype_node_id.0,
            ));
        })
        .unwrap();
}

fn assert_unmatched_repeat_prototype(model: &RetainedUiHostModel) {
    for control_id in ["AuthoredRow", "AuthoredRowLabel"] {
        let node = model.node_by_control_id(control_id).unwrap();
        assert_eq!(node.surface_node_id, None);
    }
}

#[test]
fn formal_repeat_projection_omits_materialized_subtree_and_keeps_hidden_injection() {
    let (mut surface, projection) = repeat_projection_fixture();
    // Match the real lifecycle: no selected properties first, then three rows.
    materialize_repeat_projection(&mut surface, 0);
    materialize_repeat_projection(&mut surface, 3);
    assert_eq!(
        surface
            .virtual_list_prototype_slot_roots(UiNodeId::new(1))
            .unwrap()
            .len(),
        3,
    );
    assert_eq!(surface.unique_control_node_id("AuthoredRow"), None);

    let model = build_host_model_with_surface(&projection, &surface).unwrap();
    assert_eq!(model.nodes.len(), surface.tree.nodes.len() + 1);
    assert!(model.node_by_control_id("AuthoredRow").is_none());
    assert!(model.node_by_control_id("AuthoredRowLabel").is_none());
    for node in model
        .nodes
        .iter()
        .filter(|node| node.surface_node_id.is_some())
    {
        let live = surface.tree.node(node.surface_node_id.unwrap()).unwrap();
        let metadata = live.template_metadata.as_ref().unwrap();
        assert_eq!(node.source_path, metadata.source_path);
        assert_eq!(node.source_node_id, metadata.source_node_id);
        assert_eq!(node.instance_path, metadata.instance_path);
    }
    let injected = model.node_by_control_id("InjectedHidden").unwrap();
    assert_eq!(injected.surface_node_id, None);
    assert_eq!(injected.parent_id.as_deref(), Some("surface/owner"));
    assert_eq!(
        injected.attributes["visibility"].as_str(),
        Some("collapsed")
    );
}

#[test]
fn formal_repeat_projection_omits_zero_capacity_prototype_subtree() {
    let (mut surface, projection) = repeat_projection_fixture();
    materialize_repeat_projection(&mut surface, 0);
    assert!(surface.tree.node(UiNodeId::new(2)).is_none());
    assert!(surface
        .virtual_list_prototype_root_id(UiNodeId::new(1))
        .is_some());
    assert!(surface
        .virtual_list_prototype_slot_roots(UiNodeId::new(1))
        .unwrap()
        .is_empty());

    let model = build_host_model_with_surface(&projection, &surface).unwrap();
    assert_eq!(model.nodes.len(), surface.tree.nodes.len() + 1);
    assert!(model.node_by_control_id("AuthoredRow").is_none());
    assert!(model.node_by_control_id("AuthoredRowLabel").is_none());
    assert!(model.node_by_control_id("InjectedHidden").is_some());
}

#[test]
fn formal_repeat_projection_keeps_prototype_without_runtime_pool() {
    let (mut surface, projection) = repeat_projection_fixture();
    surface.detach_subtree_to_pool(UiNodeId::new(2)).unwrap();
    assert!(surface
        .virtual_list_prototype_root_id(UiNodeId::new(1))
        .is_none());

    let model = build_host_model_with_surface(&projection, &surface).unwrap();
    assert_unmatched_repeat_prototype(&model);
}

#[test]
fn formal_repeat_projection_keeps_named_prototype_without_repeat_declaration() {
    let (mut surface, projection) = repeat_projection_fixture();
    materialize_repeat_projection(&mut surface, 0);
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(1))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .remove("repeat");

    let model = build_host_model_with_surface(&projection, &surface).unwrap();
    assert_unmatched_repeat_prototype(&model);
}

#[test]
fn formal_repeat_projection_keeps_prototype_for_wrong_owner_pool() {
    let (mut surface, mut projection) = repeat_projection_fixture();
    materialize_repeat_projection(&mut surface, 0);
    projection.root.control_id = Some("OtherOwner".into());
    surface.tree.insert_root(repeat_surface_node(
        10,
        "surface/other-owner",
        &projection.root,
    ));
    assert!(surface
        .virtual_list_prototype_root_id(UiNodeId::new(1))
        .is_some());
    assert!(surface
        .virtual_list_prototype_root_id(UiNodeId::new(10))
        .is_none());

    let model = build_host_model_with_surface(&projection, &surface).unwrap();
    assert_unmatched_repeat_prototype(&model);
    assert_eq!(
        model
            .node_by_control_id("AuthoredRow")
            .unwrap()
            .parent_id
            .as_deref(),
        Some("surface/other-owner"),
    );
}

#[test]
fn formal_repeat_projection_keeps_prototype_for_invalid_repeat_declarations() {
    for (field, value) in [
        ("kind", Value::String("unrelated".into())),
        ("prototype", Value::String("OtherPrototype".into())),
        ("authored_count", Value::Integer(2)),
        ("virtual_control_prefix", Value::String(String::new())),
    ] {
        let (mut surface, projection) = repeat_projection_fixture();
        materialize_repeat_projection(&mut surface, 0);
        surface
            .tree
            .nodes
            .get_mut(&UiNodeId::new(1))
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap()
            .attributes
            .get_mut("repeat")
            .unwrap()
            .as_table_mut()
            .unwrap()
            .insert(field.into(), value);

        let model = build_host_model_with_surface(&projection, &surface).unwrap();
        assert_unmatched_repeat_prototype(&model);
    }
}
