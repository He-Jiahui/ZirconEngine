pub(super) struct UiAssetDetailFieldSection {
    pub(super) section_control_id: &'static str,
    pub(super) detail_id: &'static str,
    pub(super) rows: Vec<UiAssetDetailFieldRow>,
}

pub(super) struct UiAssetDetailFieldRow {
    pub(super) label: String,
    pub(super) value: String,
    pub(super) action_id: String,
    pub(super) label_control_id: String,
    pub(super) value_control_id: String,
    pub(super) disabled: bool,
}

pub(super) fn push_detail_row(
    rows: &mut Vec<UiAssetDetailFieldRow>,
    label: &str,
    value: &str,
    action_id: &str,
    control_id_prefix: &str,
    disabled: bool,
    force_visible: bool,
) {
    if !force_visible && value.is_empty() {
        return;
    }
    rows.push(UiAssetDetailFieldRow {
        label: label.to_string(),
        value: value.to_string(),
        action_id: action_id.to_string(),
        label_control_id: format!("{control_id_prefix}Label"),
        value_control_id: format!("{control_id_prefix}Value"),
        disabled,
    });
}

pub(super) fn semantic_label(prefix: &str, path: &str) -> String {
    if path.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix} {path}")
    }
}
