use super::super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_cells(
    node: &TemplatePaneNodeData,
) -> Vec<String> {
    let option_cells = (0..node.options.row_count())
        .filter_map(|row| node.options.row_data(row))
        .map(|cell| cell.to_string())
        .filter(|cell| !cell.trim().is_empty())
        .collect::<Vec<_>>();
    if option_cells_look_like_declared_cells(&option_cells) {
        return option_cells;
    }
    split_archived_table_text(node.text.as_str())
}

fn option_cells_look_like_declared_cells(cells: &[String]) -> bool {
    if cells.is_empty() {
        return false;
    }
    let whole_row_like_count = cells
        .iter()
        .filter(|cell| split_archived_table_text(cell.as_str()).len() > 1)
        .count();
    whole_row_like_count * 2 <= cells.len()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn split_archived_table_text(
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
