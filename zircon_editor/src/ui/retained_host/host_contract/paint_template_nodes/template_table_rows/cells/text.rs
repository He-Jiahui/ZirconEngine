use super::super::super::super::data::TemplatePaneNodeData;

const TYPE_COLUMN_INDEX: usize = 1;
const SIZE_COLUMN_INDEX: usize = 2;
const REVISION_COLUMN_INDEX: usize = 3;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_cells(
    node: &TemplatePaneNodeData,
) -> Vec<String> {
    let option_cells = (0..node.options.row_count())
        .filter_map(|row| node.options.row_data(row))
        .map(|cell| cell.to_string())
        .filter(|cell| !cell.trim().is_empty())
        .collect::<Vec<_>>();
    if option_cells_look_like_declared_cells(&option_cells) {
        return normalize_table_cells(option_cells);
    }
    display_table_cells_from_archived_text(node.text.as_str())
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
        [name, kind, size, size_unit, revision_label, revision_value, ..]
            if looks_like_size_unit(size_unit) =>
        {
            vec![
                (*name).to_string(),
                (*kind).to_string(),
                format!("{size} {size_unit}"),
                format!("{revision_label} {revision_value}"),
            ]
        }
        [name, kind, size, size_unit, modified, ..] if looks_like_size_unit(size_unit) => vec![
            (*name).to_string(),
            (*kind).to_string(),
            format!("{size} {size_unit}"),
            (*modified).to_string(),
        ],
        [name, kind, size, revision_label, revision_value, ..]
            if looks_like_revision_label(revision_label) =>
        {
            vec![
                (*name).to_string(),
                (*kind).to_string(),
                (*size).to_string(),
                format!("{revision_label} {revision_value}"),
            ]
        }
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

fn display_table_cells_from_archived_text(text: &str) -> Vec<String> {
    normalize_table_cells(split_archived_table_text(text))
}

fn normalize_table_cells(cells: Vec<String>) -> Vec<String> {
    cells
        .into_iter()
        .enumerate()
        .map(|(index, cell)| normalize_table_cell(index, cell.as_str()))
        .collect()
}

fn normalize_table_cell(index: usize, cell: &str) -> String {
    match index {
        TYPE_COLUMN_INDEX => normalize_type_cell(cell),
        SIZE_COLUMN_INDEX => normalize_size_cell(cell),
        REVISION_COLUMN_INDEX => normalize_revision_cell(cell),
        _ => cell.trim().to_string(),
    }
}

fn normalize_type_cell(cell: &str) -> String {
    let trimmed = cell.trim();
    if trimmed.eq_ignore_ascii_case("tex") {
        "Texture".to_string()
    } else if trimmed.eq_ignore_ascii_case("mat") {
        "Material".to_string()
    } else {
        trimmed.to_string()
    }
}

fn normalize_size_cell(cell: &str) -> String {
    let trimmed = cell.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let parts = trimmed.split_whitespace().collect::<Vec<_>>();
    match parts.as_slice() {
        [number, unit] if is_numeric_text(number) => canonical_size_unit(unit)
            .map(|canonical| format!("{number} {canonical}"))
            .unwrap_or_else(|| trimmed.to_string()),
        [compact] => split_number_suffix(compact)
            .and_then(|(number, suffix)| {
                canonical_size_unit(suffix).map(|canonical| format!("{number} {canonical}"))
            })
            .unwrap_or_else(|| trimmed.to_string()),
        _ => trimmed.to_string(),
    }
}

fn normalize_revision_cell(cell: &str) -> String {
    let trimmed = cell.trim();
    if trimmed.eq_ignore_ascii_case("rev") {
        return "Revision".to_string();
    }
    if let Some(digits) = revision_digits_after_prefix(trimmed, "rev") {
        return format!("rev {digits}");
    }
    if let Some(digits) = revision_digits_after_prefix(trimmed, "r") {
        return format!("rev {digits}");
    }
    trimmed.to_string()
}

fn revision_digits_after_prefix<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let candidate = text.get(..prefix.len())?;
    if !candidate.eq_ignore_ascii_case(prefix) {
        return None;
    }
    let digits = text.get(prefix.len()..)?.trim();
    is_digits_text(digits).then_some(digits)
}

fn split_number_suffix(text: &str) -> Option<(&str, &str)> {
    let suffix_start = text
        .char_indices()
        .find_map(|(index, value)| (!value.is_ascii_digit() && value != '.').then_some(index))?;
    (suffix_start > 0 && suffix_start < text.len())
        .then_some((&text[..suffix_start], &text[suffix_start..]))
}

fn is_numeric_text(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|value| value.is_ascii_digit() || value == '.')
}

fn is_digits_text(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|value| value.is_ascii_digit())
}

fn looks_like_size_unit(unit: &str) -> bool {
    canonical_size_unit(unit).is_some()
}

fn canonical_size_unit(unit: &str) -> Option<&'static str> {
    if unit.eq_ignore_ascii_case("b") {
        Some("B")
    } else if unit.eq_ignore_ascii_case("k") || unit.eq_ignore_ascii_case("kb") {
        Some("KB")
    } else if unit.eq_ignore_ascii_case("m") || unit.eq_ignore_ascii_case("mb") {
        Some("MB")
    } else if unit.eq_ignore_ascii_case("g") || unit.eq_ignore_ascii_case("gb") {
        Some("GB")
    } else {
        None
    }
}

fn looks_like_revision_label(label: &str) -> bool {
    label.eq_ignore_ascii_case("r") || label.eq_ignore_ascii_case("rev")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn archived_asset_table_header_expands_revision_label() {
        assert_eq!(
            display_table_cells_from_archived_text("Name Type Size Rev"),
            cells(&["Name", "Type", "Size", "Revision"])
        );
    }

    #[test]
    fn archived_asset_table_row_expands_compact_size_and_revision() {
        assert_eq!(
            display_table_cells_from_archived_text("Host UI 12K r42"),
            cells(&["Host", "UI", "12 KB", "rev 42"])
        );
    }

    #[test]
    fn archived_asset_table_row_expands_compact_asset_type() {
        assert_eq!(
            display_table_cells_from_archived_text("Folder Tex 4K r40"),
            cells(&["Folder", "Texture", "4 KB", "rev 40"])
        );
    }

    #[test]
    fn archived_asset_table_row_preserves_explicit_units() {
        assert_eq!(
            display_table_cells_from_archived_text("Host UI 12 KB rev 42"),
            cells(&["Host", "UI", "12 KB", "rev 42"])
        );
    }

    #[test]
    fn declared_table_cells_use_the_same_display_normalization() {
        assert_eq!(
            normalize_table_cells(cells(&["Folder", "Tex", "1.2M", "r42"])),
            cells(&["Folder", "Texture", "1.2 MB", "rev 42"])
        );
    }
}
