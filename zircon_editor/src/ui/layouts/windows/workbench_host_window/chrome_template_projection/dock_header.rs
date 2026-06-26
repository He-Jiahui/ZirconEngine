use super::*;
use crate::ui::workbench::document_tabs::{
    document_tab_close_x, document_tab_preferred_width, DOCUMENT_TAB_CLOSE_EXTENT,
    DOCUMENT_TAB_CLOSE_TOP_INSET, DOCUMENT_TAB_GAP, DOCUMENT_TAB_HEIGHT, DOCUMENT_TAB_STRIP_X,
    DOCUMENT_TAB_STRIP_Y,
};

pub(super) fn side_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    _panel_preset_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    dock_header_nodes(tabs, &"".into(), width, height)
}

pub(super) fn document_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    subtitle: &SharedString,
    _panel_preset_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    dock_header_nodes(tabs, subtitle, width, height)
}

pub(super) fn bottom_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    _panel_preset_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    dock_header_nodes(tabs, &"".into(), width, height)
}

pub(super) fn floating_window_header_nodes(
    tabs: &ModelRc<TabData>,
    title: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    dock_header_nodes(tabs, title, width, height)
}

fn dock_header_nodes(
    tabs: &ModelRc<TabData>,
    subtitle: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    if FAST_PROCEDURAL_CHROME_NODES {
        return fallback_dock_header_nodes(tabs, subtitle, width, height);
    }

    let mut text_overrides = tab_text_overrides(DOCK_TAB_PREFIX, tabs);
    text_overrides.insert(DOCK_SUBTITLE_CONTROL_ID.to_string(), subtitle.to_string());
    let nodes = template_nodes(
        "host.dock.header",
        DOCK_HEADER_ASSET,
        width,
        height,
        &text_overrides,
        &[
            SlotFilter::new(DOCK_TAB_PREFIX, tabs.row_count()),
            SlotFilter::new(DOCK_TAB_CLOSE_PREFIX, tabs.row_count()),
        ],
    );
    if tab_chrome_needs_fallback(&nodes, DOCK_HEADER_BAR_CONTROL_ID, DOCK_TAB_PREFIX, tabs) {
        return fallback_dock_header_nodes(tabs, subtitle, width, height);
    }

    model_rc(
        (0..nodes.row_count())
            .filter_map(|row| nodes.row_data(row))
            .filter(|node| node_survives_dock_tab_close_filter(node, tabs))
            .map(|node| tab_node_with_state(node, DOCK_TAB_PREFIX, tabs))
            .collect(),
    )
}

pub(super) fn dock_header_frame(nodes: &ModelRc<ViewTemplateNodeData>) -> FrameRect {
    control_frame(nodes, DOCK_HEADER_BAR_CONTROL_ID)
}

pub(super) fn dock_subtitle_frame(nodes: &ModelRc<ViewTemplateNodeData>) -> FrameRect {
    control_frame(nodes, DOCK_SUBTITLE_CONTROL_ID)
}

pub(super) fn dock_tab_frames(
    nodes: &ModelRc<ViewTemplateNodeData>,
    tabs: &ModelRc<TabData>,
) -> ModelRc<HostChromeTabData> {
    tab_frames(nodes, DOCK_TAB_PREFIX, Some(DOCK_TAB_CLOSE_PREFIX), tabs)
}

pub(super) fn fallback_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    subtitle: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let header_height = height.max(DOCK_HEADER_HEIGHT_PX);
    let mut nodes = Vec::with_capacity(tabs.row_count() * 2 + 2);
    nodes.push(ViewTemplateNodeData {
        node_id: "FallbackDockHeaderBar".into(),
        control_id: DOCK_HEADER_BAR_CONTROL_ID.into(),
        role: "Panel".into(),
        surface_variant: "panel".into(),
        frame: ViewTemplateFrameData {
            x: 0.0,
            y: 0.0,
            width: width.max(1.0),
            height: header_height,
        },
        ..ViewTemplateNodeData::default()
    });

    let mut x = DOCUMENT_TAB_STRIP_X;
    for row in 0..tabs.row_count() {
        let Some(tab) = tabs.row_data(row) else {
            continue;
        };
        let tab_width = document_tab_preferred_width(tab.title.as_str(), tab.closeable);
        let text_tone = if tab.active { "default" } else { "subtle" };
        let font_weight = if tab.active { 600 } else { 400 };
        let icon_name = chrome_tab_icon_name(&tab);
        let mut tab_node = ViewTemplateNodeData {
            node_id: format!("FallbackDockTab{row}").into(),
            control_id: format!("{DOCK_TAB_PREFIX}{row}").into(),
            role: "Button".into(),
            text: tab.title.clone(),
            text_tone: text_tone.into(),
            font_size: CHROME_TEXT_FONT_SIZE_PX,
            font_weight,
            surface_variant: if tab.active { "inset" } else { "" }.into(),
            button_variant: "ghost".into(),
            selected: tab.active,
            focused: tab.active,
            frame: ViewTemplateFrameData {
                x,
                y: DOCUMENT_TAB_STRIP_Y,
                width: tab_width,
                height: DOCUMENT_TAB_HEIGHT.min(header_height.max(DOCUMENT_TAB_HEIGHT)),
            },
            ..ViewTemplateNodeData::default()
        };
        apply_template_icon(&mut tab_node, &icon_name);
        nodes.push(tab_node);
        if tab.closeable {
            let mut close_node = ViewTemplateNodeData {
                node_id: format!("FallbackDockTabClose{row}").into(),
                control_id: format!("{DOCK_TAB_CLOSE_PREFIX}{row}").into(),
                role: "IconButton".into(),
                text_tone: "muted".into(),
                font_size: CHROME_TEXT_FONT_SIZE_PX,
                button_variant: "ghost".into(),
                value_number: 14.0,
                frame: ViewTemplateFrameData {
                    x: document_tab_close_x(x, tab_width),
                    y: DOCUMENT_TAB_CLOSE_TOP_INSET,
                    width: DOCUMENT_TAB_CLOSE_EXTENT,
                    height: DOCUMENT_TAB_CLOSE_EXTENT,
                },
                ..ViewTemplateNodeData::default()
            };
            apply_template_icon(&mut close_node, DOCK_TAB_CLOSE_ICON);
            nodes.push(close_node);
        }
        x += tab_width + DOCUMENT_TAB_GAP;
    }

    if !subtitle.is_empty() {
        nodes.push(ViewTemplateNodeData {
            node_id: "FallbackDockSubtitle".into(),
            control_id: DOCK_SUBTITLE_CONTROL_ID.into(),
            role: "Text".into(),
            text: subtitle.clone(),
            text_tone: "muted".into(),
            font_size: 10.0,
            frame: ViewTemplateFrameData {
                x: (x + 8.0).min(width.max(1.0)),
                y: 7.0,
                width: (width - x - 16.0).max(0.0),
                height: 16.0,
            },
            ..ViewTemplateNodeData::default()
        });
    }

    model_rc(nodes)
}
