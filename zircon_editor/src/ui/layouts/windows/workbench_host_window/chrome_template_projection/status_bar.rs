use super::*;

pub(super) fn status_bar_nodes(
    status_primary: &SharedString,
    status_secondary: &SharedString,
    viewport_label: &SharedString,
    skin_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    if FAST_PROCEDURAL_CHROME_NODES {
        return fallback_status_bar_nodes(
            status_primary,
            status_secondary,
            viewport_label,
            skin_id,
            width,
            height,
        );
    }

    let mut text_overrides = BTreeMap::new();
    text_overrides.insert(
        STATUS_PRIMARY_CONTROL_ID.to_string(),
        status_primary.to_string(),
    );
    text_overrides.insert(
        STATUS_SECONDARY_CONTROL_ID.to_string(),
        status_secondary.to_string(),
    );
    text_overrides.insert(
        STATUS_VIEWPORT_CONTROL_ID.to_string(),
        viewport_label.to_string(),
    );

    let nodes = template_nodes(
        "host.status.bar",
        STATUS_BAR_ASSET,
        width,
        height,
        &text_overrides,
        &[],
    );
    if skin_id.as_str() == "material_dark" {
        return model_rc(
            (0..nodes.row_count())
                .filter_map(|row| nodes.row_data(row))
                .map(|mut node| {
                    if node.control_id == STATUS_PRIMARY_CONTROL_ID {
                        node.text_tone = "default".into();
                    }
                    node
                })
                .collect(),
        );
    }
    nodes
}

fn fallback_status_bar_nodes(
    status_primary: &SharedString,
    status_secondary: &SharedString,
    viewport_label: &SharedString,
    skin_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let bar_height = height.max(20.0);
    let mut nodes = Vec::with_capacity(4);
    nodes.push(ViewTemplateNodeData {
        node_id: "FallbackStatusBarPanel".into(),
        control_id: STATUS_BAR_PANEL_CONTROL_ID.into(),
        role: "Panel".into(),
        surface_variant: "panel".into(),
        frame: ViewTemplateFrameData {
            x: 0.0,
            y: 0.0,
            width: width.max(1.0),
            height: bar_height,
        },
        ..ViewTemplateNodeData::default()
    });

    let primary_width = (width * 0.48).clamp(120.0, 520.0);
    let viewport_width = (width * 0.18).clamp(96.0, 220.0);
    let secondary_width = (width - primary_width - viewport_width - 36.0).max(60.0);
    let primary_tone: SharedString = if skin_id.as_str() == "material_dark" {
        "default".into()
    } else {
        "subtle".into()
    };
    nodes.push(status_text_node(
        "FallbackStatusPrimaryLabel",
        STATUS_PRIMARY_CONTROL_ID,
        status_primary,
        primary_tone,
        12.0,
        2.0,
        primary_width,
        bar_height,
    ));
    nodes.push(status_text_node(
        "FallbackStatusSecondaryLabel",
        STATUS_SECONDARY_CONTROL_ID,
        status_secondary,
        "muted".into(),
        primary_width + 12.0,
        2.0,
        secondary_width,
        bar_height,
    ));
    nodes.push(status_text_node(
        "FallbackStatusViewportLabel",
        STATUS_VIEWPORT_CONTROL_ID,
        viewport_label,
        "muted".into(),
        (width - viewport_width - 12.0).max(0.0),
        2.0,
        viewport_width,
        bar_height,
    ));
    model_rc(nodes)
}

fn status_text_node(
    node_id: &'static str,
    control_id: &'static str,
    text: &SharedString,
    text_tone: SharedString,
    x: f32,
    y: f32,
    width: f32,
    bar_height: f32,
) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: node_id.into(),
        control_id: control_id.into(),
        role: "Text".into(),
        text: text.clone(),
        text_tone,
        font_size: 11.0,
        frame: ViewTemplateFrameData {
            x,
            y,
            width: width.max(0.0),
            height: (bar_height - 4.0).max(12.0),
        },
        ..ViewTemplateNodeData::default()
    }
}
