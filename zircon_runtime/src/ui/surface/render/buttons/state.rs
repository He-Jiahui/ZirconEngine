use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::UiStateFlags,
    style::{UiPainterFamily, UiPainterResolvedState, UiPainterStyleSelector},
    tree::UiTemplateNodeMetadata,
};

use super::{
    super::painter_state::UiRenderPainterStateSource,
    metadata::{ButtonKind, bool_attribute, button_kind, is_icon_button},
};

#[derive(Clone, Copy)]
pub(super) struct ButtonRenderState {
    family: UiPainterFamily,
    kind: ButtonKind,
    visual_state: UiPainterResolvedState,
    surface_hot: bool,
    marked: bool,
}

impl ButtonRenderState {
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
            || bool_attribute(metadata, "checked").unwrap_or(false);
        let mut painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        painter_state.checked = checked;
        painter_state.selected = selected;
        let family = if is_icon_button(metadata) {
            UiPainterFamily::IconButton
        } else {
            UiPainterFamily::Button
        };
        let kind = button_kind(metadata);
        let surface_hot = painter_state.hovered
            || painter_state.open
            || painter_state.dragging
            || painter_state.drop_hovered;
        let marked = selected || checked;
        Self {
            family,
            kind,
            visual_state: UiPainterStyleSelector::resolved_state_for_family(painter_state, family),
            surface_hot,
            marked,
        }
    }

    pub(super) fn family(self) -> UiPainterFamily {
        self.family
    }

    pub(super) fn kind(self) -> ButtonKind {
        self.kind
    }

    pub(super) fn visual_state(self) -> UiPainterResolvedState {
        self.visual_state
    }

    pub(super) fn unavailable(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
        )
    }

    pub(super) fn selected(self) -> bool {
        self.marked
            || matches!(
                self.visual_state,
                UiPainterResolvedState::Selected | UiPainterResolvedState::Checked
            )
    }

    pub(super) fn focused(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Focused)
    }

    pub(super) fn pressed(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Pressed)
    }

    pub(super) fn surface_hot(self) -> bool {
        self.surface_hot
            || matches!(
                self.visual_state,
                UiPainterResolvedState::Hovered
                    | UiPainterResolvedState::Open
                    | UiPainterResolvedState::Dragging
                    | UiPainterResolvedState::DropHovered
            )
    }
}
