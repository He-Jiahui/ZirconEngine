use std::rc::Rc;

use super::*;
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPageOverflowMenuStateData, HostWindowPresentationData, TabData,
};
use crate::ui::retained_host::host_contract::native_keyboard::WorkbenchPopupKeyboardCommand;
use crate::ui::retained_host::primitives::{ModelRc, VecModel};

#[test]
fn host_page_overflow_target_stays_local_and_never_requests_a_page_window() {
    let target = host_page_overflow_keyboard_target(&overflow_presentation())
        .expect("open host-page overflow should expose its keyboard target");

    assert_eq!(target.window_offset, 0);
    assert_eq!(target.window_count, target.rows.len());
    assert_eq!(target.total_count, target.rows.len());
    assert!(!target.window_navigation_enabled);
    assert!(target.window_query.is_empty());
    assert!(target
        .next_move(WorkbenchPopupKeyboardCommand::PageDown)
        .is_none());
    assert!(target
        .next_move(WorkbenchPopupKeyboardCommand::PageUp)
        .is_none());
}

fn overflow_presentation() -> HostWindowPresentationData {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.page_chrome.overflow_frame = FrameRect {
        x: 188.0,
        y: 29.0,
        width: 34.0,
        height: 28.0,
    };
    presentation.host_scene_data.page_chrome.tabs = model_rc(vec![
        tab("workbench", "Workbench", true),
        tab("assets", "Assets", false),
        tab("animation", "Animation", false),
        tab("tags", "Tags", false),
    ]);
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = vec![1, 2, 3];
    presentation.host_page_overflow_menu_state = HostPageOverflowMenuStateData {
        open: true,
        hovered_page_index: 1,
        scroll_offset: 0.0,
    };
    presentation
}

fn model_rc<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(rows)))
}

fn tab(id: &str, title: &str, active: bool) -> TabData {
    TabData {
        id: id.into(),
        title: title.into(),
        active,
        ..TabData::default()
    }
}
