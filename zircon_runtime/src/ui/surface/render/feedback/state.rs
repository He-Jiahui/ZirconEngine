use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::UiStateFlags,
    style::{UiPainterFamily, UiPainterResolvedState},
    tree::UiTemplateNodeMetadata,
};

use super::super::painter_state::UiRenderPainterStateSource;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FeedbackKind {
    Alert,
    AlertTitle,
    Tooltip,
    Toast,
}

#[derive(Clone, Copy)]
pub(super) struct FeedbackRenderState {
    pub(super) family: UiPainterFamily,
    pub(super) visual_state: UiPainterResolvedState,
}

impl FeedbackRenderState {
    pub(super) fn resolve(
        kind: FeedbackKind,
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        let family = match kind {
            FeedbackKind::Alert | FeedbackKind::AlertTitle => UiPainterFamily::Alert,
            FeedbackKind::Tooltip => UiPainterFamily::Tooltip,
            FeedbackKind::Toast => UiPainterFamily::Toast,
        };
        Self {
            family,
            visual_state: painter_state.resolved_state_for_family(family),
        }
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

    pub(super) fn hot(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Hovered
                | UiPainterResolvedState::Focused
                | UiPainterResolvedState::Open
                | UiPainterResolvedState::Dragging
                | UiPainterResolvedState::DropHovered
        )
    }
}
