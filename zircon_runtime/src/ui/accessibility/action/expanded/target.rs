use zircon_runtime_interface::ui::{
    component::UiComponentEvent, event_ui::UiNodeId, tree::UiTemplateNodeMetadata,
    widget::UiWidgetBehavior,
};

use crate::ui::surface::UiSurface;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UiExpandableActionKind {
    Disclosure,
    Popup,
}

pub(super) struct UiExpandableActionTarget {
    pub(super) property: String,
    pub(super) kind: UiExpandableActionKind,
}

// 用 widget 行为找真实 open 属性，解决组件自定义 alias 与默认 expanded/popup_open 名称不一致的问题。
pub(super) fn expandable_action_target(
    surface: &UiSurface,
    target: UiNodeId,
) -> Option<UiExpandableActionTarget> {
    let metadata = surface
        .tree
        .nodes
        .get(&target)?
        .template_metadata
        .as_ref()?;
    match metadata.widget.resolved_behavior(&metadata.component) {
        UiWidgetBehavior::Disclosure => Some(UiExpandableActionTarget {
            property: widget_open_property(metadata, "expanded").to_string(),
            kind: UiExpandableActionKind::Disclosure,
        }),
        UiWidgetBehavior::Popup => Some(UiExpandableActionTarget {
            property: widget_open_property(metadata, "popup_open").to_string(),
            kind: UiExpandableActionKind::Popup,
        }),
        _ => None,
    }
}

fn widget_open_property<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    fallback: &'static str,
) -> &'a str {
    metadata.widget.open_property.as_deref().unwrap_or(fallback)
}

pub(super) fn expanded_component_event(
    kind: UiExpandableActionKind,
    expanded: bool,
) -> UiComponentEvent {
    match kind {
        UiExpandableActionKind::Disclosure => UiComponentEvent::ToggleExpanded { expanded },
        UiExpandableActionKind::Popup if expanded => UiComponentEvent::OpenPopup,
        UiExpandableActionKind::Popup => UiComponentEvent::ClosePopup,
    }
}
