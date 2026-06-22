mod body;
mod close;

use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::primitives::SharedString;

use self::body::dispatch_document_tab_body_press;
use self::close::dispatch_document_tab_close_press;

type DocumentTabPressDispatch = fn(&UiHostWindow, SharedString, usize, f32, f32, f32, f32);

#[allow(clippy::too_many_arguments)]
pub(in crate::ui::retained_host::host_contract::native_pointer::button_dispatch::chrome_press) fn dispatch_document_tab_press(
    ui: &UiHostWindow,
    surface_key: SharedString,
    index: usize,
    tab_x: f32,
    tab_width: f32,
    local_x: f32,
    local_y: f32,
    close: bool,
) {
    let dispatch = if close {
        dispatch_document_tab_close_press as DocumentTabPressDispatch
    } else {
        dispatch_document_tab_body_press as DocumentTabPressDispatch
    };
    dispatch(ui, surface_key, index, tab_x, tab_width, local_x, local_y);
}
