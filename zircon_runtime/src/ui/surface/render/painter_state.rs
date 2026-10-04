use toml::Value;
use zircon_runtime_interface::ui::{
    component::{UiComponentFlags, UiComponentState},
    event_ui::UiStateFlags,
    style::UiPainterState,
    tree::UiTemplateNodeMetadata,
};

/// 统一汇集运行时组件标志、节点状态与模板预览标志；各 painter 再按自身族的共享优先级选色。
/// 模板布尔真值与运行时状态合并，而非覆盖后者；focus_visible 的静态回退在运行时焦点建立后让位。
#[derive(Clone, Copy)]
pub(super) struct UiRenderPainterStateSource<'a> {
    metadata: Option<&'a UiTemplateNodeMetadata>,
    state_flags: &'a UiStateFlags,
    component_state: Option<&'a UiComponentState>,
}

impl<'a> UiRenderPainterStateSource<'a> {
    pub(super) fn new(
        metadata: Option<&'a UiTemplateNodeMetadata>,
        state_flags: &'a UiStateFlags,
        component_state: Option<&'a UiComponentState>,
    ) -> Self {
        Self {
            metadata,
            state_flags,
            component_state,
        }
    }

    pub(super) fn painter_state(self) -> UiPainterState {
        painter_state_from_source(self)
    }

    /// 仅供把布尔 value 作为勾选值的选择控件调用，避免普通值字段意外变成 checked 视觉状态。
    pub(super) fn painter_state_with_value_checked(self) -> UiPainterState {
        painter_state_with_value_checked_from_source(self)
    }
}

fn painter_state_from_source(source: UiRenderPainterStateSource<'_>) -> UiPainterState {
    let component_flags = source.component_state.map(|state| &state.flags);
    let metadata_focused = metadata_focused(source.metadata);
    UiPainterState {
        hovered: component_bool(component_flags, |flags| flags.hovered)
            || bool_attribute(source.metadata, "hovered").unwrap_or(false),
        pressed: component_bool(component_flags, |flags| flags.pressed)
            || source.state_flags.pressed
            || bool_attribute(source.metadata, "pressed").unwrap_or(false),
        focused: component_bool(component_flags, |flags| flags.focused) || metadata_focused,
        focus_visible: component_bool(component_flags, |flags| flags.focus_visible)
            || bool_attribute(source.metadata, "focus_visible")
                .or_else(|| bool_attribute(source.metadata, "focusVisible"))
                // Static painter fixtures predate a live focus owner. Preserve their focus
                // styling until runtime input establishes semantic focus for the component.
                .unwrap_or(
                    metadata_focused && !component_bool(component_flags, |flags| flags.focused),
                ),
        disabled: component_bool(component_flags, |flags| flags.disabled)
            || !source.state_flags.enabled
            || bool_attribute(source.metadata, "disabled").unwrap_or(false),
        checked: component_bool(component_flags, |flags| flags.checked)
            || source.state_flags.checked
            || bool_attribute(source.metadata, "checked").unwrap_or(false),
        selected: component_bool(component_flags, |flags| flags.selected)
            || bool_attribute(source.metadata, "selected").unwrap_or(false),
        open: component_bool(component_flags, |flags| flags.popup_open)
            || bool_attribute(source.metadata, "open")
                .or_else(|| bool_attribute(source.metadata, "popup_open"))
                .unwrap_or(false),
        dragging: component_bool(component_flags, |flags| flags.dragging)
            || bool_attribute(source.metadata, "dragging").unwrap_or(false),
        drop_hovered: component_bool(component_flags, |flags| {
            flags.drop_hovered || flags.active_drag_target
        }) || bool_attribute(source.metadata, "drop_hovered")
            .or_else(|| bool_attribute(source.metadata, "active_drag_target"))
            .unwrap_or(false),
        loading: component_bool(component_flags, |flags| flags.loading)
            || bool_attribute(source.metadata, "loading").unwrap_or(false),
    }
}

fn metadata_focused(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    bool_attribute(metadata, "focused").unwrap_or(false)
}

fn painter_state_with_value_checked_from_source(
    source: UiRenderPainterStateSource<'_>,
) -> UiPainterState {
    let metadata = source.metadata;
    let mut state = source.painter_state();
    state.checked = state.checked || bool_attribute(metadata, "value").unwrap_or(false);
    state
}

fn component_bool(
    component_flags: Option<&UiComponentFlags>,
    selector: impl FnOnce(&UiComponentFlags) -> bool,
) -> bool {
    component_flags.is_some_and(selector)
}

fn bool_attribute(metadata: Option<&UiTemplateNodeMetadata>, key: &str) -> Option<bool> {
    metadata
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(Value::as_bool)
}

#[cfg(test)]
#[path = "tests/painter_state.rs"]
mod tests;
