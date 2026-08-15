use super::*;
use crate::ui::workbench::page_tabs::main_page_tab_close_frame;
use zircon_runtime_interface::ui::layout::UiFrame;

pub(super) fn close_view_frame(tab: &ViewTemplateFrameData) -> ViewTemplateFrameData {
    let frame = main_page_tab_close_frame(UiFrame::new(tab.x, tab.y, tab.width, tab.height));
    ViewTemplateFrameData {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

pub(super) fn close_node(row: usize, frame: ViewTemplateFrameData) -> ViewTemplateNodeData {
    let mut node = ViewTemplateNodeData {
        node_id: format!("FallbackPageTabClose{row}").into(),
        control_id: format!("{PAGE_TAB_CLOSE_PREFIX}{row}").into(),
        role: "IconButton".into(),
        text_tone: "muted".into(),
        font_size: EditorTypographyTokens::WORKBENCH_BODY_SIZE,
        button_variant: "ghost".into(),
        value_number: 14.0,
        frame,
        ..ViewTemplateNodeData::default()
    };
    apply_template_icon(&mut node, PAGE_TAB_CLOSE_ICON);
    node
}
