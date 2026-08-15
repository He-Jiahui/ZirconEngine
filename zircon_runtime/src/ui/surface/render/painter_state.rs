use toml::Value;
use zircon_runtime_interface::ui::{
    component::{UiComponentFlags, UiComponentState},
    event_ui::UiStateFlags,
    style::UiPainterState,
    tree::UiTemplateNodeMetadata,
};

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

    pub(super) fn painter_state_with_value_checked(self) -> UiPainterState {
        painter_state_with_value_checked_from_source(self)
    }
}

fn painter_state_from_source(source: UiRenderPainterStateSource<'_>) -> UiPainterState {
    let component_flags = source.component_state.map(|state| &state.flags);
    UiPainterState {
        hovered: component_bool(component_flags, |flags| flags.hovered)
            || bool_attribute(source.metadata, "hovered").unwrap_or(false),
        pressed: component_bool(component_flags, |flags| flags.pressed)
            || source.state_flags.pressed
            || bool_attribute(source.metadata, "pressed").unwrap_or(false),
        focused: component_bool(component_flags, |flags| flags.focused)
            || bool_attribute(source.metadata, "focused").unwrap_or(false),
        focus_visible: component_bool(component_flags, |flags| flags.focus_visible)
            || bool_attribute(source.metadata, "focus_visible")
                .or_else(|| bool_attribute(source.metadata, "focusVisible"))
                // Static painter fixtures predate a live focus owner. Preserve their focus
                // styling until runtime input establishes semantic focus for the component.
                .unwrap_or(
                    bool_attribute(source.metadata, "focused").unwrap_or(false)
                        && !component_bool(component_flags, |flags| flags.focused),
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
mod tests {
    use super::*;

    #[test]
    fn static_focus_preview_yields_to_runtime_focus_visibility() {
        let mut metadata = UiTemplateNodeMetadata::default();
        metadata
            .attributes
            .insert("focused".to_string(), Value::Boolean(true));
        let state_flags = UiStateFlags {
            enabled: true,
            ..UiStateFlags::default()
        };

        let static_preview =
            UiRenderPainterStateSource::new(Some(&metadata), &state_flags, None).painter_state();
        assert!(static_preview.focused);
        assert!(static_preview.focus_visible);

        let mut pointer_focus = UiComponentState::default();
        pointer_focus.flags.focused = true;
        let pointer =
            UiRenderPainterStateSource::new(Some(&metadata), &state_flags, Some(&pointer_focus))
                .painter_state();
        assert!(pointer.focused);
        assert!(!pointer.focus_visible);

        pointer_focus.flags.focus_visible = true;
        let keyboard =
            UiRenderPainterStateSource::new(Some(&metadata), &state_flags, Some(&pointer_focus))
                .painter_state();
        assert!(keyboard.focus_visible);
    }
}
