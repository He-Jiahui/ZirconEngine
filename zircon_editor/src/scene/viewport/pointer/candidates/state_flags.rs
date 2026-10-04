//! 指针表面区分被动根与可接收指针的视口节点，候选树不建立键盘焦点或隐式操作权限。

use zircon_runtime_interface::ui::event_ui::UiStateFlags;

pub(in crate::scene::viewport::pointer) fn passive_state_flags() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: false,
        hoverable: false,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

pub(in crate::scene::viewport::pointer) fn interactive_state_flags() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}
