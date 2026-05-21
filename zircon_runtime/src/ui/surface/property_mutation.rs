use zircon_runtime_interface::ui::{
    binding::{
        UiBindingDirtyDomain, UiBindingSourceKind, UiBindingUpdateReport, UiBindingUpdateStatus,
    },
    component::{UiValue, UiValueKind},
    event_ui::{UiNodeId, UiPropertyInvalidationReason, UiReflectedPropertySource},
    focus::UiFocusChangeEvent,
    tree::{UiDirtyFlags, UiInputPolicy, UiTree, UiTreeError, UiTreeNode, UiVisibility},
};

use crate::ui::{
    binding::{
        component_state_value_update, reflected_property_update,
        reflected_property_update_with_source_kind,
    },
    tree::UiRuntimeTreeAccessExt,
};

#[derive(Clone, Debug, PartialEq)]
pub struct UiPropertyMutationRequest {
    pub node_id: UiNodeId,
    pub property: String,
    pub value: UiValue,
    pub source: UiReflectedPropertySource,
    pub binding_source_kind: Option<UiBindingSourceKind>,
}

impl UiPropertyMutationRequest {
    pub fn new(node_id: UiNodeId, property: impl Into<String>, value: UiValue) -> Self {
        Self {
            node_id,
            property: property.into(),
            value,
            source: UiReflectedPropertySource::RuntimeState,
            binding_source_kind: None,
        }
    }

    pub fn with_source(mut self, source: UiReflectedPropertySource) -> Self {
        self.source = source;
        self
    }

    pub(crate) fn with_binding_source_kind(mut self, kind: UiBindingSourceKind) -> Self {
        self.binding_source_kind = Some(kind);
        self
    }

    pub(crate) fn widget_behavior(
        node_id: UiNodeId,
        property: impl Into<String>,
        value: UiValue,
    ) -> Self {
        Self::new(node_id, property, value)
            .with_binding_source_kind(UiBindingSourceKind::WidgetBehavior)
    }

    pub(crate) fn accessibility_action(
        node_id: UiNodeId,
        property: impl Into<String>,
        value: UiValue,
    ) -> Self {
        Self::new(node_id, property, value)
            .with_binding_source_kind(UiBindingSourceKind::AccessibilityAction)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiPropertyMutationStatus {
    Accepted,
    Unchanged,
    Rejected,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiPropertyMutationReport {
    pub node_id: UiNodeId,
    pub property: String,
    pub status: UiPropertyMutationStatus,
    pub source: UiReflectedPropertySource,
    pub invalidation: UiPropertyInvalidationReason,
    pub binding: UiBindingUpdateReport,
    pub message: Option<String>,
    pub focus_change: Option<UiFocusChangeEvent>,
}

impl UiPropertyMutationReport {
    fn accepted(
        request: &UiPropertyMutationRequest,
        previous: Option<UiValue>,
        dirty: UiDirtyFlags,
    ) -> Self {
        Self {
            node_id: request.node_id,
            property: request.property.clone(),
            status: UiPropertyMutationStatus::Accepted,
            source: request.source,
            invalidation: UiPropertyInvalidationReason::with_dirty(dirty),
            binding: property_binding_report(
                request,
                previous,
                dirty,
                UiBindingUpdateStatus::Applied,
                None,
            ),
            message: None,
            focus_change: None,
        }
    }

    fn unchanged(request: &UiPropertyMutationRequest, previous: Option<UiValue>) -> Self {
        Self {
            node_id: request.node_id,
            property: request.property.clone(),
            status: UiPropertyMutationStatus::Unchanged,
            source: request.source,
            invalidation: UiPropertyInvalidationReason::none(),
            binding: property_binding_report(
                request,
                previous,
                UiDirtyFlags::default(),
                UiBindingUpdateStatus::Unchanged,
                None,
            ),
            message: None,
            focus_change: None,
        }
    }

    fn rejected(request: &UiPropertyMutationRequest, message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            node_id: request.node_id,
            property: request.property.clone(),
            status: UiPropertyMutationStatus::Rejected,
            source: request.source,
            invalidation: UiPropertyInvalidationReason::reflection_only(),
            binding: property_binding_report(
                request,
                None,
                UiDirtyFlags::default(),
                UiBindingUpdateStatus::Rejected,
                Some(message.clone()),
            ),
            message: Some(message),
            focus_change: None,
        }
    }

    pub(crate) fn mark_render_dirty(&mut self) {
        self.invalidation.dirty.render = true;
        self.sync_binding_dirty_from_invalidation();
    }

    pub(crate) fn record_component_state_value_update(
        &mut self,
        node_id: UiNodeId,
        property: impl Into<String>,
        previous: Option<UiValue>,
        value: UiValue,
    ) {
        self.binding.updates.push(component_state_value_update(
            node_id,
            property,
            previous,
            value,
            self.invalidation.dirty,
            UiBindingUpdateStatus::Applied,
        ));
        self.binding.recompute();
    }

    fn sync_binding_dirty_from_invalidation(&mut self) {
        let dirty = UiBindingDirtyDomain::from_dirty_flags(self.invalidation.dirty);
        for update in &mut self.binding.updates {
            update.dirty = dirty.clone();
        }
        self.binding.recompute();
    }
}

pub fn mutate_tree_property(
    tree: &mut UiTree,
    request: UiPropertyMutationRequest,
) -> Result<UiPropertyMutationReport, UiTreeError> {
    let node = tree
        .node_mut(request.node_id)
        .ok_or(UiTreeError::MissingNode(request.node_id))?;

    let report = match request.property.as_str() {
        "visibility" => match visibility_value(&request.value) {
            Some(next) if node.visibility == next => {
                UiPropertyMutationReport::unchanged(&request, Some(visibility_ui_value(node.visibility)))
            }
            Some(next) => {
                let previous = Some(visibility_ui_value(node.visibility));
                let dirty = visibility_transition_dirty(
                    node.visibility,
                    next,
                    node.state_flags.visible,
                );
                node.visibility = next;
                mark_state_dirty(node, dirty);
                UiPropertyMutationReport::accepted(&request, previous, dirty)
            }
            None => UiPropertyMutationReport::rejected(
                &request,
                "visibility expects one of visible, hidden, collapsed, hit_test_invisible, or self_hit_test_invisible",
            ),
        },
        "enabled" => mutate_node_state_bool(&request, node, |node| &mut node.state_flags.enabled, input_dirty()),
        "visible" => mutate_node_state_bool(
            &request,
            node,
            |node| &mut node.state_flags.visible,
            visibility_dirty(node.visibility),
        ),
        "clickable" => mutate_node_state_bool(&request, node, |node| &mut node.state_flags.clickable, input_dirty()),
        "hoverable" => mutate_node_state_bool(&request, node, |node| &mut node.state_flags.hoverable, input_dirty()),
        "focusable" => mutate_node_state_bool(&request, node, |node| &mut node.state_flags.focusable, input_dirty()),
        "pressed" => mutate_node_state_bool(&request, node, |node| &mut node.state_flags.pressed, render_dirty()),
        "checked" => mutate_node_state_bool_and_optional_attribute(
            &request,
            node,
            |node| &mut node.state_flags.checked,
            "checked",
            render_dirty(),
        ),
        "input_policy" => match input_policy_value(&request.value) {
            Some(next) if node.input_policy == next => UiPropertyMutationReport::unchanged(
                &request,
                Some(input_policy_ui_value(node.input_policy)),
            ),
            Some(next) => {
                let previous = Some(input_policy_ui_value(node.input_policy));
                node.input_policy = next;
                mark_state_dirty(node, input_dirty());
                UiPropertyMutationReport::accepted(&request, previous, input_dirty())
            }
            None => UiPropertyMutationReport::rejected(
                &request,
                "input_policy expects inherit, receive, or ignore",
            ),
        },
        property => {
            let Some(metadata) = node.template_metadata.as_mut() else {
                return Ok(UiPropertyMutationReport::rejected(
                    &request,
                    format!("node has no template metadata for `{property}`"),
                ));
            };
            let next = request.value.to_toml();
            let previous = metadata.attributes.get(property).map(UiValue::from_toml);
            if metadata.attributes.get(property) == Some(&next) {
                UiPropertyMutationReport::unchanged(&request, previous)
            } else {
                metadata.attributes.insert(property.to_string(), next);
                let dirty =
                    metadata_attribute_dirty(metadata.component.as_str(), property, request.value.kind());
                mark_dirty(node, dirty);
                UiPropertyMutationReport::accepted(&request, previous, dirty)
            }
        }
    };
    Ok(report)
}

fn mutate_state_bool(
    request: &UiPropertyMutationRequest,
    current: &mut bool,
    dirty: UiDirtyFlags,
) -> UiPropertyMutationReport {
    let Some(next) = bool_value(&request.value) else {
        return UiPropertyMutationReport::rejected(
            request,
            format!("{} expects a boolean value", request.property),
        );
    };
    let previous = Some(UiValue::Bool(*current));
    if *current == next {
        return UiPropertyMutationReport::unchanged(request, previous);
    }
    *current = next;
    UiPropertyMutationReport::accepted(request, previous, dirty)
}

fn mutate_node_state_bool(
    request: &UiPropertyMutationRequest,
    node: &mut UiTreeNode,
    field: impl FnOnce(&mut UiTreeNode) -> &mut bool,
    dirty: UiDirtyFlags,
) -> UiPropertyMutationReport {
    let report = mutate_state_bool(request, field(node), dirty);
    if matches!(report.status, UiPropertyMutationStatus::Accepted) {
        mark_state_dirty(node, dirty);
    }
    report
}

fn mutate_node_state_bool_and_optional_attribute(
    request: &UiPropertyMutationRequest,
    node: &mut UiTreeNode,
    field: impl FnOnce(&mut UiTreeNode) -> &mut bool,
    attribute: &str,
    dirty: UiDirtyFlags,
) -> UiPropertyMutationReport {
    let Some(next) = bool_value(&request.value) else {
        return UiPropertyMutationReport::rejected(
            request,
            format!("{} expects a boolean value", request.property),
        );
    };

    let mut changed = false;
    let previous;
    {
        let current = field(node);
        previous = Some(UiValue::Bool(*current));
        if *current != next {
            *current = next;
            changed = true;
        }
    }

    if let Some(metadata) = node.template_metadata.as_mut() {
        if metadata.attributes.contains_key(attribute) {
            let next_value = request.value.to_toml();
            if metadata.attributes.get(attribute) != Some(&next_value) {
                metadata
                    .attributes
                    .insert(attribute.to_string(), next_value);
                changed = true;
            }
        }
    }

    if !changed {
        return UiPropertyMutationReport::unchanged(request, previous);
    }

    mark_state_dirty(node, dirty);
    UiPropertyMutationReport::accepted(request, previous, dirty)
}

fn property_binding_report(
    request: &UiPropertyMutationRequest,
    previous: Option<UiValue>,
    dirty: UiDirtyFlags,
    status: UiBindingUpdateStatus,
    message: Option<String>,
) -> UiBindingUpdateReport {
    let update = match request.binding_source_kind {
        Some(source_kind) => reflected_property_update_with_source_kind(
            request.node_id,
            request.property.clone(),
            source_kind,
            previous,
            request.value.clone(),
            dirty,
            status,
            message,
        ),
        None => reflected_property_update(
            request.node_id,
            request.property.clone(),
            request.source,
            previous,
            request.value.clone(),
            dirty,
            status,
            message,
        ),
    };
    UiBindingUpdateReport::from_updates(vec![update])
}

fn bool_value(value: &UiValue) -> Option<bool> {
    match value {
        UiValue::Bool(value) => Some(*value),
        _ => None,
    }
}

fn visibility_value(value: &UiValue) -> Option<UiVisibility> {
    match value {
        UiValue::Enum(value) | UiValue::String(value) => match normalize_token(value).as_str() {
            "visible" => Some(UiVisibility::Visible),
            "hidden" => Some(UiVisibility::Hidden),
            "collapsed" => Some(UiVisibility::Collapsed),
            "hittestinvisible" => Some(UiVisibility::HitTestInvisible),
            "selfhittestinvisible" => Some(UiVisibility::SelfHitTestInvisible),
            _ => None,
        },
        _ => None,
    }
}

fn input_policy_value(value: &UiValue) -> Option<UiInputPolicy> {
    match value {
        UiValue::Enum(value) | UiValue::String(value) => match normalize_token(value).as_str() {
            "inherit" => Some(UiInputPolicy::Inherit),
            "receive" => Some(UiInputPolicy::Receive),
            "ignore" => Some(UiInputPolicy::Ignore),
            _ => None,
        },
        _ => None,
    }
}

fn visibility_ui_value(visibility: UiVisibility) -> UiValue {
    UiValue::Enum(
        match visibility {
            UiVisibility::Visible => "visible",
            UiVisibility::Hidden => "hidden",
            UiVisibility::Collapsed => "collapsed",
            UiVisibility::HitTestInvisible => "hit_test_invisible",
            UiVisibility::SelfHitTestInvisible => "self_hit_test_invisible",
        }
        .to_string(),
    )
}

fn input_policy_ui_value(input_policy: UiInputPolicy) -> UiValue {
    UiValue::Enum(
        match input_policy {
            UiInputPolicy::Inherit => "inherit",
            UiInputPolicy::Receive => "receive",
            UiInputPolicy::Ignore => "ignore",
        }
        .to_string(),
    )
}

fn normalize_token(value: &str) -> String {
    value
        .chars()
        .filter(|ch| *ch != '_' && *ch != '-' && !ch.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

fn mark_dirty(node: &mut zircon_runtime_interface::ui::tree::UiTreeNode, dirty: UiDirtyFlags) {
    node.dirty.layout |= dirty.layout;
    node.dirty.hit_test |= dirty.hit_test;
    node.dirty.render |= dirty.render;
    node.dirty.style |= dirty.style;
    node.dirty.text |= dirty.text;
    node.dirty.input |= dirty.input;
    node.dirty.visible_range |= dirty.visible_range;
}

fn mark_state_dirty(
    node: &mut zircon_runtime_interface::ui::tree::UiTreeNode,
    dirty: UiDirtyFlags,
) {
    mark_dirty(node, dirty);
    if dirty.hit_test || dirty.input {
        node.state_flags.dirty = true;
    }
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

fn visibility_transition_dirty(
    current: UiVisibility,
    next: UiVisibility,
    legacy_visible: bool,
) -> UiDirtyFlags {
    let mut dirty = visibility_dirty(next);
    dirty.layout |= current.effective(legacy_visible).occupies_layout()
        != next.effective(legacy_visible).occupies_layout();
    dirty
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

fn metadata_attribute_dirty(
    component: &str,
    property: &str,
    value_kind: UiValueKind,
) -> UiDirtyFlags {
    match property {
        "value" if is_render_only_numeric_value_component(component) => render_dirty(),
        "text" | "label" | "value" | "value_text" | "font_size" | "line_height" => UiDirtyFlags {
            layout: true,
            render: true,
            text: true,
            ..UiDirtyFlags::default()
        },
        "caret_offset"
        | "selection_anchor"
        | "selection_focus"
        | "composition_start"
        | "composition_end"
        | "composition_text"
        | "composition_restore_text" => render_dirty(),
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

fn is_render_only_numeric_value_component(component: &str) -> bool {
    matches!(component, "RangeField" | "Slider" | "ProgressBar")
}

fn is_layout_metadata_attribute(property: &str) -> bool {
    matches!(
        property,
        "layout" | "width" | "height" | "min_width" | "min_height" | "padding" | "gap"
    ) || property.starts_with("layout_")
}
