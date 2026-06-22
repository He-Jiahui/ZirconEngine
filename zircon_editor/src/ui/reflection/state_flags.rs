use zircon_runtime_interface::ui::event_ui::UiStateFlags;

pub(super) fn visible_enabled_flags(visible: bool, enabled: bool) -> UiStateFlags {
    UiStateFlags {
        visible,
        enabled,
        clickable: false,
        hoverable: false,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}
