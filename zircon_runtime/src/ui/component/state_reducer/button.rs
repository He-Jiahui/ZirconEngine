//! 按钮族先处理高频焦点、悬停和按压反馈；其余事件保留所有权交回通用归约入口，确保专用路径与公共事件契约一致。

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEvent, UiComponentState,
};

// Applied 表示专用路径已消费事件；通用分支携带原事件供上层继续归约，避免双重处理。
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
