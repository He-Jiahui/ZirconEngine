use zircon_runtime_interface::ui::{
    binding::{
        UiBindingSource, UiBindingSourceKind, UiBindingTarget, UiBindingUpdate,
        UiBindingUpdateReport, UiBindingUpdateStatus,
    },
    component::UiValue,
    event_ui::{UiNodeId, UiReflectedPropertySource},
    tree::UiDirtyFlags,
};

pub fn retained_attribute_update(
    node_id: UiNodeId,
    property: impl Into<String>,
    previous: Option<UiValue>,
    value: UiValue,
    dirty: UiDirtyFlags,
    status: UiBindingUpdateStatus,
) -> UiBindingUpdate {
    let property = property.into();
    update_with_status(
        UiBindingSource::retained_attribute(node_id, property.clone()),
        UiBindingTarget::retained_attribute(node_id, property),
        previous,
        value,
        dirty,
        status,
        None,
    )
}

pub fn component_state_value_update(
    node_id: UiNodeId,
    property: impl Into<String>,
    previous: Option<UiValue>,
    value: UiValue,
    dirty: UiDirtyFlags,
    status: UiBindingUpdateStatus,
) -> UiBindingUpdate {
    component_state_value_update_with_source_kind(
        node_id,
        property,
        UiBindingSourceKind::RuntimeState,
        previous,
        value,
        dirty,
        status,
    )
}

pub fn component_state_value_update_with_source_kind(
    node_id: UiNodeId,
    property: impl Into<String>,
    source_kind: UiBindingSourceKind,
    previous: Option<UiValue>,
    value: UiValue,
    dirty: UiDirtyFlags,
    status: UiBindingUpdateStatus,
) -> UiBindingUpdate {
    let property = property.into();
    update_with_status(
        UiBindingSource {
            kind: source_kind,
            node_id: Some(node_id),
            property: Some(property.clone()),
            path: None,
        },
        UiBindingTarget::component_state_value(node_id, property),
        previous,
        value,
        dirty,
        status,
        None,
    )
}

pub fn runtime_state_update_with_source_kind(
    source_node_id: UiNodeId,
    source_property: impl Into<String>,
    source_kind: UiBindingSourceKind,
    target_node_id: UiNodeId,
    target_property: impl Into<String>,
    previous: Option<UiValue>,
    value: UiValue,
    dirty: UiDirtyFlags,
    status: UiBindingUpdateStatus,
    message: Option<String>,
) -> UiBindingUpdate {
    update_with_status(
        UiBindingSource {
            kind: source_kind,
            node_id: Some(source_node_id),
            property: Some(source_property.into()),
            path: None,
        },
        UiBindingTarget::runtime_state(target_node_id, target_property),
        previous,
        value,
        dirty,
        status,
        message,
    )
}

pub fn rejected_widget_alias_update(
    node_id: UiNodeId,
    property: impl Into<String>,
    value: UiValue,
    message: impl Into<String>,
) -> UiBindingUpdate {
    let property = property.into();
    UiBindingUpdate::rejected(
        UiBindingSource::widget_behavior(node_id, property.clone()),
        UiBindingTarget::widget_alias(node_id, property),
        value,
        message,
    )
}

pub fn reflected_property_update(
    node_id: UiNodeId,
    property: impl Into<String>,
    source: UiReflectedPropertySource,
    previous: Option<UiValue>,
    value: UiValue,
    dirty: UiDirtyFlags,
    status: UiBindingUpdateStatus,
    message: Option<String>,
) -> UiBindingUpdate {
    let property = property.into();
    update_with_status(
        reflected_property_source(source, node_id, property.clone()),
        UiBindingTarget::retained_attribute(node_id, property),
        previous,
        value,
        dirty,
        status,
        message,
    )
}

pub fn reflected_property_update_with_source_kind(
    node_id: UiNodeId,
    property: impl Into<String>,
    source_kind: UiBindingSourceKind,
    previous: Option<UiValue>,
    value: UiValue,
    dirty: UiDirtyFlags,
    status: UiBindingUpdateStatus,
    message: Option<String>,
) -> UiBindingUpdate {
    let property = property.into();
    update_with_status(
        UiBindingSource {
            kind: source_kind,
            node_id: Some(node_id),
            property: Some(property.clone()),
            path: None,
        },
        UiBindingTarget::retained_attribute(node_id, property),
        previous,
        value,
        dirty,
        status,
        message,
    )
}

pub fn binding_update_report(updates: Vec<UiBindingUpdate>) -> UiBindingUpdateReport {
    UiBindingUpdateReport::from_updates(updates)
}

pub(crate) const fn reflected_property_source_kind(
    source: UiReflectedPropertySource,
) -> UiBindingSourceKind {
    match source {
        UiReflectedPropertySource::Authored
        | UiReflectedPropertySource::DescriptorDefault
        | UiReflectedPropertySource::InferredDefault => UiBindingSourceKind::RetainedAttribute,
        UiReflectedPropertySource::RuntimeState | UiReflectedPropertySource::SystemState => {
            UiBindingSourceKind::RuntimeState
        }
        UiReflectedPropertySource::Binding => UiBindingSourceKind::ComponentEvent,
    }
}

fn reflected_property_source(
    source: UiReflectedPropertySource,
    node_id: UiNodeId,
    property: String,
) -> UiBindingSource {
    match reflected_property_source_kind(source) {
        UiBindingSourceKind::RetainedAttribute => {
            UiBindingSource::retained_attribute(node_id, property)
        }
        UiBindingSourceKind::RuntimeState => UiBindingSource::runtime_state(node_id, property),
        UiBindingSourceKind::ComponentEvent => UiBindingSource::component_event(node_id, property),
        _ => unreachable!("reflected property sources map to retained, runtime, or binding state"),
    }
}

fn update_with_status(
    source: UiBindingSource,
    target: UiBindingTarget,
    previous: Option<UiValue>,
    value: UiValue,
    dirty: UiDirtyFlags,
    status: UiBindingUpdateStatus,
    message: Option<String>,
) -> UiBindingUpdate {
    let update = match status {
        UiBindingUpdateStatus::Applied => UiBindingUpdate::applied(source, target, value),
        UiBindingUpdateStatus::Unchanged => UiBindingUpdate::unchanged(source, target, value),
        UiBindingUpdateStatus::Rejected => {
            UiBindingUpdate::rejected(source, target, value, message.unwrap_or_default())
        }
    };
    update.with_previous(previous).with_dirty_flags(dirty)
}
