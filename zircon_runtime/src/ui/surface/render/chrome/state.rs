use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::UiStateFlags,
    style::{UiPainterFamily, UiPainterResolvedState},
    tree::UiTemplateNodeMetadata,
};

use super::super::painter_state::UiRenderPainterStateSource;

#[derive(Clone, Copy)]
pub(super) struct ChromeRenderState {
    visual_state: UiPainterResolvedState,
}

impl ChromeRenderState {
    pub(super) fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let visual_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state()
                .resolved_state_for_family(UiPainterFamily::Chrome);
        Self { visual_state }
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

    pub(super) fn active(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Focused
                | UiPainterResolvedState::Open
                | UiPainterResolvedState::Selected
                | UiPainterResolvedState::Checked
                | UiPainterResolvedState::Dragging
                | UiPainterResolvedState::DropHovered
        )
    }

    pub(super) fn selected_surface_active(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Open
                | UiPainterResolvedState::Selected
                | UiPainterResolvedState::Checked
                | UiPainterResolvedState::Dragging
                | UiPainterResolvedState::DropHovered
        )
    }
}
