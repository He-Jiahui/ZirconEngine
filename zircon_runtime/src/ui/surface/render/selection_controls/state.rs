use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::UiStateFlags,
    style::{UiPainterFamily, UiPainterResolvedState},
    tree::UiTemplateNodeMetadata,
};

use super::{
    super::painter_state::UiRenderPainterStateSource,
    metadata::{bool_attribute, selection_painter_family},
};

#[derive(Clone, Copy)]
pub(super) struct SelectionRenderState {
    family: UiPainterFamily,
    checked: bool,
    selected: bool,
    visual_state: UiPainterResolvedState,
    surface_hot: bool,
}

impl SelectionRenderState {
    pub(super) fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let component_flags = component_state.map(|state| &state.flags);
        let selected = component_flags.is_some_and(|flags| flags.selected)
            || bool_attribute(metadata, "selected").unwrap_or(false);
        let checked = component_flags.is_some_and(|flags| flags.checked)
            || state_flags.checked
            || selected
            || bool_attribute(metadata, "checked")
                .or_else(|| bool_attribute(metadata, "value"))
                .unwrap_or(false);
        let disabled = component_flags.is_some_and(|flags| flags.disabled)
            || !state_flags.enabled
            || bool_attribute(metadata, "disabled").unwrap_or(false);
        let mut painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state_with_value_checked();
        painter_state.disabled = disabled;
        painter_state.checked = checked;
        painter_state.selected = selected;
        let family = selection_painter_family(metadata);
        Self {
            family,
            checked,
            selected,
            visual_state: painter_state.resolved_state_for_family(family),
            surface_hot: painter_state.hovered
                || painter_state.dragging
                || painter_state.drop_hovered,
        }
    }

    pub(super) fn family(self) -> UiPainterFamily {
        self.family
    }

    pub(super) fn visual_state(self) -> UiPainterResolvedState {
        self.visual_state
    }

    pub(super) fn active(self) -> bool {
        self.checked || self.selected
    }

    pub(super) fn unavailable(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
        )
    }

    pub(super) fn pressed(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Pressed)
    }

    pub(super) fn focused(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Focused)
    }

    pub(super) fn surface_hot(self) -> bool {
        self.surface_hot
    }

    pub(super) fn focus_border(self) -> bool {
        self.pressed() || self.focused() || (!self.active() && self.surface_hot())
    }
}
