use std::collections::BTreeMap;

use zircon_runtime_interface::ui::template::{UiActionRef, UiBindingRef};
use zircon_runtime_interface::ui::{
    component::{UiValue, UiValueKind},
    event_ui::{
        UiActionDescriptor, UiReflectedProperty, UiReflectedPropertySource, UiReflectorHitContext,
        UiReflectorNode, UiReflectorSnapshot, UiWidgetLifecycleState,
    },
    surface::UiHitTestQuery,
    tree::{UiDirtyFlags, UiTreeNode, UiVisibility},
};

use super::UiSurface;

type UiPropertyInvalidationReason =
    zircon_runtime_interface::ui::event_ui::UiPropertyInvalidationReason;

pub fn reflector_snapshot(
    surface: &UiSurface,
    query: Option<UiHitTestQuery>,
) -> UiReflectorSnapshot {
    let hit = query.map(|query| (query.clone(), surface.hit_test_with_query(query)));
    let mut nodes = Vec::new();
    for node in surface.tree.nodes.values() {
        let arranged = surface.arranged_tree.get(node.node_id);
        let effective_visibility = arranged
            .map(|arranged| arranged.visibility)
            .unwrap_or_else(|| node.effective_visibility());
        let lifecycle = lifecycle_for_node(node, arranged.is_some(), effective_visibility);
        let class_name = node
            .template_metadata
            .as_ref()
            .map(|metadata| metadata.component.clone())
            .filter(|component| !component.is_empty())
            .unwrap_or_else(|| "Node".to_string());
        let display_name = node
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.clone())
            .unwrap_or_else(|| node.node_path.0.clone());
        let actions = node_actions(node);
        nodes.push(UiReflectorNode {
            node_id: node.node_id,
            node_path: node.node_path.clone(),
            parent: node.parent,
            children: node.children.clone(),
            class_name,
            display_name,
            lifecycle,
            visibility: node.visibility,
            effective_visibility,
            state_flags: node.state_flags.clone(),
            input_policy: node.input_policy,
            z_index: node.z_index,
            paint_order: node.paint_order,
            clip_to_bounds: node.clip_to_bounds,
            frame: arranged
                .map(|arranged| arranged.frame)
                .unwrap_or(node.layout_cache.frame),
            clip_frame: arranged
                .map(|arranged| arranged.clip_frame)
                .or(node.layout_cache.clip_frame)
                .unwrap_or(node.layout_cache.frame),
            dirty: node.dirty,
            properties: reflected_properties(node),
            actions,
            focused: surface.focus.focused == Some(node.node_id),
            hovered: surface.focus.hovered.contains(&node.node_id),
            captured: surface.focus.captured == Some(node.node_id),
            pressed: surface.focus.pressed == Some(node.node_id),
            source_asset: None,
            source_template_path: Some(node.node_path.0.clone()),
        });
    }
    let mut snapshot = UiReflectorSnapshot::new(
        surface.tree.tree_id.clone(),
        surface.tree.roots.clone(),
        nodes,
    );
    snapshot.focused = surface.focus.focused;
    snapshot.captured = surface.focus.captured;
    snapshot.hovered = surface.focus.hovered.clone();
    snapshot.hit_context = hit.map(|(query, hit)| UiReflectorHitContext {
        query_point: query.hit_point(),
        hit_target: hit.top_hit,
        hit_stack: hit.stacked,
        rejected: Vec::new(),
    });
    snapshot
}

fn lifecycle_for_node(
    node: &UiTreeNode,
    arranged: bool,
    effective_visibility: UiVisibility,
) -> UiWidgetLifecycleState {
    if !node.state_flags.visible || matches!(effective_visibility, UiVisibility::Collapsed) {
        return UiWidgetLifecycleState::Detached;
    }
    if node.is_focus_candidate() || node.supports_pointer() {
        return UiWidgetLifecycleState::Interactive;
    }
    if effective_visibility.is_render_visible() {
        return UiWidgetLifecycleState::Visible;
    }
    if arranged {
        return UiWidgetLifecycleState::Arranged;
    }
    if node.template_metadata.is_some() {
        return UiWidgetLifecycleState::PropertiesSynchronized;
    }
    UiWidgetLifecycleState::Constructed
}

fn reflected_properties(node: &UiTreeNode) -> BTreeMap<String, UiReflectedProperty> {
    let mut properties = BTreeMap::new();
    insert_system_property(
        &mut properties,
        "visibility",
        UiValue::Enum(format!("{:?}", node.visibility)),
        visibility_dirty(node.visibility),
        true,
    );
    insert_system_property(
        &mut properties,
        "input_policy",
        UiValue::Enum(format!("{:?}", node.input_policy)),
        input_dirty(),
        false,
    );
    insert_state_property(
        &mut properties,
        "visible",
        node.state_flags.visible,
        visibility_dirty(node.visibility),
    );
    insert_state_property(
        &mut properties,
        "enabled",
        node.state_flags.enabled,
        input_dirty(),
    );
    insert_state_property(
        &mut properties,
        "clickable",
        node.state_flags.clickable,
        input_dirty(),
    );
    insert_state_property(
        &mut properties,
        "hoverable",
        node.state_flags.hoverable,
        input_dirty(),
    );
    insert_state_property(
        &mut properties,
        "focusable",
        node.state_flags.focusable,
        input_dirty(),
    );
    insert_state_property(
        &mut properties,
        "pressed",
        node.state_flags.pressed,
        render_dirty(),
    );
    insert_state_property(
        &mut properties,
        "checked",
        node.state_flags.checked,
        render_dirty(),
    );

    if let Some(metadata) = &node.template_metadata {
        for (name, value) in &metadata.attributes {
            let resolved = UiValue::from_toml(value);
            let dirty = metadata_attribute_dirty(name, resolved.kind());
            let property = UiReflectedProperty::new(
                name.clone(),
                UiReflectedPropertySource::Authored,
                resolved.clone(),
            )
            .writable(true)
            .authored_value(resolved)
            .invalidation(UiPropertyInvalidationReason::with_dirty(dirty));
            properties.insert(name.clone(), property);
        }
        if !metadata.component.is_empty() {
            properties.insert(
                "component".to_string(),
                UiReflectedProperty::new(
                    "component",
                    UiReflectedPropertySource::SystemState,
                    UiValue::String(metadata.component.clone()),
                ),
            );
        }
    }
    properties
}

fn node_actions(node: &UiTreeNode) -> BTreeMap<String, UiActionDescriptor> {
    let mut actions = BTreeMap::new();
    let Some(metadata) = &node.template_metadata else {
        return actions;
    };
    for binding in &metadata.bindings {
        let action = UiActionDescriptor::new(
            binding.id.clone(),
            binding.event,
            binding_symbol(binding, &binding.id),
        );
        actions.insert(binding.id.clone(), action);
    }
    actions
}

fn binding_symbol(binding: &UiBindingRef, fallback: &str) -> String {
    binding
        .action
        .as_ref()
        .and_then(action_symbol)
        .or_else(|| binding.route.clone())
        .unwrap_or_else(|| fallback.to_string())
}

fn action_symbol(action: &UiActionRef) -> Option<String> {
    action.action.clone().or_else(|| action.route.clone())
}

fn insert_system_property(
    properties: &mut BTreeMap<String, UiReflectedProperty>,
    name: &str,
    value: UiValue,
    dirty: UiDirtyFlags,
    visibility_affecting: bool,
) {
    let property = UiReflectedProperty::new(name, UiReflectedPropertySource::SystemState, value)
        .writable(true)
        .visibility_affecting(visibility_affecting)
        .invalidation(UiPropertyInvalidationReason::with_dirty(dirty));
    properties.insert(name.to_string(), property);
}

fn insert_state_property(
    properties: &mut BTreeMap<String, UiReflectedProperty>,
    name: &str,
    value: bool,
    dirty: UiDirtyFlags,
) {
    let property = UiReflectedProperty::new(
        name,
        UiReflectedPropertySource::RuntimeState,
        UiValue::Bool(value),
    )
    .writable(true)
    .invalidation(UiPropertyInvalidationReason::with_dirty(dirty));
    properties.insert(name.to_string(), property);
}

fn visibility_dirty(visibility: UiVisibility) -> UiDirtyFlags {
    UiDirtyFlags {
        layout: matches!(visibility, UiVisibility::Collapsed),
        hit_test: true,
        render: true,
        input: true,
        ..UiDirtyFlags::default()
    }
}

fn input_dirty() -> UiDirtyFlags {
    UiDirtyFlags {
        hit_test: true,
        input: true,
        render: true,
        ..UiDirtyFlags::default()
    }
}

fn render_dirty() -> UiDirtyFlags {
    UiDirtyFlags {
        render: true,
        ..UiDirtyFlags::default()
    }
}

fn metadata_attribute_dirty(property: &str, value_kind: UiValueKind) -> UiDirtyFlags {
    match property {
        "text" | "label" | "font_size" | "line_height" => UiDirtyFlags {
            layout: true,
            render: true,
            text: true,
            ..UiDirtyFlags::default()
        },
        _ if is_layout_metadata_attribute(property) => UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            ..UiDirtyFlags::default()
        },
        _ if matches!(value_kind, UiValueKind::Bool) && property.contains("visible") => {
            UiDirtyFlags {
                layout: true,
                hit_test: true,
                render: true,
                input: true,
                ..UiDirtyFlags::default()
            }
        }
        _ => render_dirty(),
    }
}

fn is_layout_metadata_attribute(property: &str) -> bool {
    matches!(
        property,
        "layout" | "width" | "height" | "min_width" | "min_height" | "padding" | "gap"
    ) || property.starts_with("layout_")
}
