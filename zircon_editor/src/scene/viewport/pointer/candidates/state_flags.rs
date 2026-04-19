use zircon_ui::event_ui::UiStateFlags;

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
