use zircon_runtime_interface::ui::{
    component::{UiValue, UiValueKind},
    event_ui::{UiNodeId, UiPropertyInvalidationReason, UiReflectedPropertySource},
    tree::{UiDirtyFlags, UiInputPolicy, UiTree, UiTreeError, UiVisibility},
};

use crate::ui::tree::UiRuntimeTreeAccessExt;

#[derive(Clone, Debug, PartialEq)]
pub struct UiPropertyMutationRequest {
    pub node_id: UiNodeId,
    pub property: String,
    pub value: UiValue,
    pub source: UiReflectedPropertySource,
}

impl UiPropertyMutationRequest {
    pub fn new(node_id: UiNodeId, property: impl Into<String>, value: UiValue) -> Self {
        Self {
            node_id,
            property: property.into(),
            value,
            source: UiReflectedPropertySource::RuntimeState,
        }
    }

    pub fn with_source(mut self, source: UiReflectedPropertySource) -> Self {
        self.source = source;
        self
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
    pub message: Option<String>,
}

impl UiPropertyMutationReport {
    fn accepted(request: &UiPropertyMutationRequest, dirty: UiDirtyFlags) -> Self {
        Self {
            node_id: request.node_id,
            property: request.property.clone(),
            status: UiPropertyMutationStatus::Accepted,
            source: request.source,
            invalidation: UiPropertyInvalidationReason::with_dirty(dirty),
            message: None,
        }
    }

    fn unchanged(request: &UiPropertyMutationRequest) -> Self {
        Self {
            node_id: request.node_id,
            property: request.property.clone(),
            status: UiPropertyMutationStatus::Unchanged,
            source: request.source,
            invalidation: UiPropertyInvalidationReason::none(),
            message: None,
        }
    }

    fn rejected(request: &UiPropertyMutationRequest, message: impl Into<String>) -> Self {
        Self {
            node_id: request.node_id,
            property: request.property.clone(),
            status: UiPropertyMutationStatus::Rejected,
            source: request.source,
            invalidation: UiPropertyInvalidationReason::reflection_only(),
            message: Some(message.into()),
        }
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
            Some(next) if node.visibility == next => UiPropertyMutationReport::unchanged(&request),
            Some(next) => {
                let dirty = visibility_transition_dirty(
                    node.visibility,
                    next,
                    node.state_flags.visible,
                );
                node.visibility = next;
                mark_dirty(node, dirty);
                UiPropertyMutationReport::accepted(&request, dirty)
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
        "checked" => mutate_node_state_bool(&request, node, |node| &mut node.state_flags.checked, render_dirty()),
        "input_policy" => match input_policy_value(&request.value) {
            Some(next) if node.input_policy == next => UiPropertyMutationReport::unchanged(&request),
            Some(next) => {
                node.input_policy = next;
                mark_dirty(node, input_dirty());
                UiPropertyMutationReport::accepted(&request, input_dirty())
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
            if metadata.attributes.get(property) == Some(&next) {
                UiPropertyMutationReport::unchanged(&request)
            } else {
                metadata.attributes.insert(property.to_string(), next);
                let dirty = metadata_attribute_dirty(property, request.value.kind());
                mark_dirty(node, dirty);
                UiPropertyMutationReport::accepted(&request, dirty)
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
    if *current == next {
        return UiPropertyMutationReport::unchanged(request);
    }
    *current = next;
    UiPropertyMutationReport::accepted(request, dirty)
}

fn mutate_node_state_bool(
    request: &UiPropertyMutationRequest,
    node: &mut zircon_runtime_interface::ui::tree::UiTreeNode,
    field: impl FnOnce(&mut zircon_runtime_interface::ui::tree::UiTreeNode) -> &mut bool,
    dirty: UiDirtyFlags,
) -> UiPropertyMutationReport {
    let report = mutate_state_bool(request, field(node), dirty);
    if matches!(report.status, UiPropertyMutationStatus::Accepted) {
        mark_dirty(node, dirty);
    }
    report
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
    node.state_flags.dirty = true;
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

fn metadata_attribute_dirty(property: &str, value_kind: UiValueKind) -> UiDirtyFlags {
    match property {
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
        | "composition_restore_text" => UiDirtyFlags {
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
