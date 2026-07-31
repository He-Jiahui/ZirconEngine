use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::ui::retained_host::UiHostWindow;
use crate::ui::retained_host::app::RetainedEditorHost;
use crate::ui::retained_host::app::native_windows::NativeFloatingWindowTarget;
use crate::ui::retained_host::primitives::CloseRequestResponse;

use super::super::super::callback_wiring::wire_callbacks;

pub(super) fn wire_native_window_presenter_callbacks(
    ui: &UiHostWindow,
    target: &NativeFloatingWindowTarget,
    host_handle: Option<&Rc<RefCell<RetainedEditorHost>>>,
) {
    let Some(host) = host_handle else {
        return;
    };
    wire_callbacks(ui, host);
    let host_weak: Weak<RefCell<RetainedEditorHost>> = Rc::downgrade(host);
    let window_id = target.window_id.clone();
    ui.window().on_close_requested(move || {
        if let Some(host) = host_weak.upgrade() {
            host.borrow_mut()
                .native_floating_window_close_requested(&window_id)
        } else {
            CloseRequestResponse::KeepWindowShown
        }
    });
}
