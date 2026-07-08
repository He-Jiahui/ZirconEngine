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
    pointer_hot: bool,
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
        let pointer_hot = painter_state.hovered
            || painter_state.open
            || painter_state.dragging
            || painter_state.drop_hovered;
        Self {
            family,
            visual_state: painter_state.resolved_state_for_family(family),
            pointer_hot,
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

    pub(super) fn pointer_hot(self) -> bool {
        self.pointer_hot
    }
}
