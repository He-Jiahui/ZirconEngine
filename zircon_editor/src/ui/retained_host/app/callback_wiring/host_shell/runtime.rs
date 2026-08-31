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
    host_shell.on_interactive_frame_requested(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().commit_interactive_frame_update();
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_workbench_pointer_input(move |pointer, tooltip_target| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .observe_workbench_pointer_input(pointer, tooltip_target);
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_workbench_input_activity(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().dismiss_workbench_tooltip();
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
            let mut host = host.borrow_mut();
            host.dismiss_workbench_tooltip();
            host.native_window_focus_lost();
            host.cancel_viewport_interaction();
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_asset_deletion_blocker_closed(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow().dismiss_asset_deletion_blocker();
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
