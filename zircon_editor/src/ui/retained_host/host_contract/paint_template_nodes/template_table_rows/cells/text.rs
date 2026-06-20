use super::super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_cells(
    node: &TemplatePaneNodeData,
) -> Vec<String> {
    let option_cells = (0..node.options.row_count())
        .filter_map(|row| node.options.row_data(row))
        .map(|cell| cell.to_string())
        .filter(|cell| !cell.trim().is_empty())
        .collect::<Vec<_>>();
    if !option_cells.is_empty() {
        return option_cells;
    }
    split_legacy_table_text(node.text.as_str())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn split_legacy_table_text(
    text: &str,
) -> Vec<String> {
    let tokens = text.split_whitespace().collect::<Vec<_>>();
    match tokens.as_slice() {
        [] => Vec::new(),
        [name, kind, size, size_unit, modified_value, modified_unit, ..] => vec![
            (*name).to_string(),
            (*kind).to_string(),
            format!("{size} {size_unit}"),
            format!("{modified_value} {modified_unit}"),
        ],
        [name, kind, size, modified, ..] => vec![
            (*name).to_string(),
            (*kind).to_string(),
            (*size).to_string(),
            (*modified).to_string(),
        ],
        _ => vec![text.trim().to_string()],
    }
}
