use std::rc::Rc;

use super::*;
use crate::ui::retained_host::host_contract::data::{HostPageOverflowMenuStateData, TabData};
use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

mod popup_geometry;
mod scrolling;

fn overflow_presentation(shell_width: f32, title: &str) -> HostWindowPresentationData {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_layout.status_bar_frame = FrameRect {
        x: 0.0,
        y: 160.0,
        width: shell_width,
        height: 20.0,
    };
    presentation.host_scene_data.page_chrome.overflow_frame = FrameRect {
        x: (shell_width - 42.0).max(0.0),
        y: 24.0,
        width: 34.0,
        height: 28.0,
    };
    presentation.host_scene_data.page_chrome.tabs = model_rc(vec![TabData {
        id: "long-tab".into(),
        title: title.into(),
        ..TabData::default()
    }]);
    let metrics = current_host_metrics();
    presentation
        .host_scene_data
        .page_chrome
        .overflow_widest_title_width_px =
        host_page_overflow_title_width(title, metrics.font_body, metrics.text_clip_guard);
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = vec![0];
    presentation.host_page_overflow_menu_state = HostPageOverflowMenuStateData {
        open: true,
        hovered_page_index: -1,
        scroll_offset: 0.0,
    };
    presentation
}

fn model_rc<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(rows)))
}
