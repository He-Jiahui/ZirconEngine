use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEvent, UiComponentState,
};

#[derive(Clone, Debug, PartialEq)]
pub(super) enum UiButtonReduceOutcome {
    Applied,
    UseGenericReducer(UiComponentEvent),
}

pub(super) fn is_button_family(descriptor: &UiComponentDescriptor) -> bool {
    matches!(
        descriptor.id.as_str(),
        "Button" | "IconButton" | "FloatingActionButton" | "ButtonBase"
    ) || matches!(
        descriptor.role.as_str(),
        "button" | "icon-button" | "fab" | "button-base"
    )
}

pub(super) fn reduce_button_event(
    state: &mut UiComponentState,
    event: UiComponentEvent,
) -> UiButtonReduceOutcome {
    match event {
        UiComponentEvent::Focus { focused } => {
            state.flags.focused = focused;
            UiButtonReduceOutcome::Applied
        }
        UiComponentEvent::Hover { hovered } => {
            state.flags.hovered = hovered;
            UiButtonReduceOutcome::Applied
        }
        UiComponentEvent::Press { pressed } => {
            state.flags.pressed = pressed;
            UiButtonReduceOutcome::Applied
        }
        event => UiButtonReduceOutcome::UseGenericReducer(event),
    }
}
