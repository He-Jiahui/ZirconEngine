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

    let node_patches = if skin_id.as_str() == "material_dark" {
        BTreeMap::from([(
            STATUS_PRIMARY_CONTROL_ID.to_string(),
            ViewTemplateNodePatch::default().text_tone("default"),
        )])
    } else {
        BTreeMap::new()
    };
    let Ok(projection) = build_view_template_node_projection_with_patches(
        "host.status.bar",
        STATUS_BAR_ASSET,
        &[],
        UiSize::new(width.max(0.0), height.max(0.0)),
        &text_overrides,
        &node_patches,
    ) else {
        return ModelRc::default();
    };
    projection.into_model()
}
