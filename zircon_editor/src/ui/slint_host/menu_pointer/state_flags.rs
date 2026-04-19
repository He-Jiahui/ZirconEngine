use zircon_ui::event_ui::UiStateFlags;

pub(in crate::ui::slint_host::menu_pointer) fn base_state(interactive: bool) -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: interactive,
        clickable: interactive,
        hoverable: interactive,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}
