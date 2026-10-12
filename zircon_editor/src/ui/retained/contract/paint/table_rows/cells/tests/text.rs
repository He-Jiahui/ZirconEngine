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
