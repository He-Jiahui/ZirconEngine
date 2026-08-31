use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::UiStateFlags,
    style::{UiPainterFamily, UiPainterResolvedState},
    tree::UiTemplateNodeMetadata,
};

use super::{super::painter_state::UiRenderPainterStateSource, metadata::bool_attribute};

#[derive(Clone, Copy)]
pub(super) struct SegmentedRenderState {
    family: UiPainterFamily,
    visual_state: UiPainterResolvedState,
    active: bool,
    surface_hot: bool,
}

impl SegmentedRenderState {
    pub(super) fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let component_flags = component_state.map(|state| &state.flags);
        let checked = component_flags.is_some_and(|flags| flags.checked)
            || state_flags.checked
            || bool_attribute(metadata, "checked").unwrap_or(false);
        let selected = checked
            || component_flags.is_some_and(|flags| flags.selected)
            || bool_attribute(metadata, "selected").unwrap_or(false);
        let mut painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        painter_state.checked = checked;
        painter_state.selected = selected;
        let family = UiPainterFamily::Tab;
        Self {
            family,
            visual_state: painter_state.resolved_state_for_family(family),
            active: selected,
            surface_hot: painter_state.hovered
                || painter_state.open
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
        self.active
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
}
