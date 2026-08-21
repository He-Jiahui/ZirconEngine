use super::*;

pub(super) fn wire_host_shell_runtime_callbacks(
    host_shell: &UiHostContext,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    let weak = Rc::downgrade(host);
    host_shell.on_frame_requested(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().tick();
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_unhandled_keyboard_input(move |keyboard| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .dispatch_unhandled_native_keyboard_input(keyboard);
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_native_window_focus_lost(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().cancel_viewport_interaction();
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_close_prompt_action_clicked(move |action_id| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .close_prompt_action_clicked(action_id.as_str());
        }
    });
}
