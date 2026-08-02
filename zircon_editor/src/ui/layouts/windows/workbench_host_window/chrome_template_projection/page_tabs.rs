use super::*;
use crate::ui::workbench::page_tabs::main_page_tab_close_frame;
use zircon_runtime_interface::ui::layout::UiFrame;

pub(super) fn append_missing_close_nodes(
    nodes: ModelRc<ViewTemplateNodeData>,
    tabs: &ModelRc<TabData>,
) -> ModelRc<ViewTemplateNodeData> {
    let mut projected = (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .collect::<Vec<_>>();
    for row in 0..tabs.row_count() {
        if !tabs.row_data(row).is_some_and(|tab| tab.closeable) {
            continue;
        }
        let close_control_id = format!("{PAGE_TAB_CLOSE_PREFIX}{row}");
        if projected
            .iter()
            .any(|node| node.control_id.as_str() == close_control_id.as_str())
        {
            continue;
        }
        let tab_frame = control_frame(&nodes, &format!("{PAGE_TAB_PREFIX}{row}"));
        if tab_frame.width <= 0.0 || tab_frame.height <= 0.0 {
            continue;
        }
        projected.push(close_node(
            row,
            close_view_frame(&ViewTemplateFrameData {
                x: tab_frame.x,
                y: tab_frame.y,
                width: tab_frame.width,
                height: tab_frame.height,
            }),
        ));
    }
    model_rc(projected)
}

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
