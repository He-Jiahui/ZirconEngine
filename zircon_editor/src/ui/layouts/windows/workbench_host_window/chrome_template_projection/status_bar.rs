use super::*;

pub(super) fn status_bar_nodes(
    status_primary: &SharedString,
    status_secondary: &SharedString,
    viewport_label: &SharedString,
    skin_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
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
