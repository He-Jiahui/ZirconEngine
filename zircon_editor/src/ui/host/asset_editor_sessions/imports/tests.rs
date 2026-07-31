use std::cell::Cell;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::ui::template::UiAssetKind;

use super::parsed_document::{parse_ui_asset_import_source, ParsedUiAssetImportDocument};
use crate::ui::host::EditorError;

use super::{collect_ui_asset_import_document, UiAssetImportTraversal};

const WIDGET_SOURCE: &str = r#"
[asset]
kind = "widget"
id = "ui.widgets.cached"
version = 1
display_name = "Cached Widget"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
control_id = "CachedWidget"
"#;

fn parsed_widget() -> ParsedUiAssetImportDocument {
    parse_ui_asset_import_source("cached.toml", WIDGET_SOURCE).expect("widget fixture")
}

#[test]
fn physical_document_is_loaded_once_across_generation_traversals() {
    let mut traversal = UiAssetImportTraversal::default();
    let load_count = Cell::new(0);
    let path = Path::new("cached.toml");

    for _ in 0..2 {
        traversal
            .generation_mut()
            .load_physical_document(path, || {
                load_count.set(load_count.get() + 1);
                Ok(parsed_widget())
            })
            .expect("cached widget");
        let _ = traversal.finish_resolution();
    }

    assert_eq!(load_count.get(), 1);
}

#[test]
fn fragment_aliases_keep_logical_rows_and_expand_physical_source_once() {
    let mut traversal = UiAssetImportTraversal::default();
    let parsed = parsed_widget();
    let path = Path::new("cached.toml");

    assert!(traversal
        .materialize_reference(
            "res://ui/cached.toml#One",
            UiAssetKind::Widget,
            path,
            &parsed,
        )
        .expect("first alias"));
    assert!(!traversal
        .materialize_reference(
            "res://ui/cached.toml#Two",
            UiAssetKind::Widget,
            path,
            &parsed,
        )
        .expect("second alias"));
    let resolution = traversal.finish_resolution();
    assert!(resolution
        .documents
        .widgets
        .contains_key("res://ui/cached.toml#One"));
    assert!(resolution
        .documents
        .widgets
        .contains_key("res://ui/cached.toml#Two"));
    assert_eq!(resolution.dependencies.len(), 1);
}

#[test]
fn failed_physical_parse_is_cached_for_the_generation() {
    let mut traversal = UiAssetImportTraversal::default();
    let load_count = Cell::new(0);
    let path = Path::new("broken.toml");

    for _ in 0..2 {
        let error = traversal
            .generation_mut()
            .load_physical_document(path, || {
                load_count.set(load_count.get() + 1);
                Err("broken import".to_string())
            })
            .expect_err("broken import remains an error");
        assert_eq!(error.to_string(), "broken import");
    }

    assert_eq!(load_count.get(), 1);
}

#[test]
fn unresolved_reference_remains_in_the_dependency_generation() {
    let mut traversal = UiAssetImportTraversal::default();
    let resolve = |_reference: &str| -> Result<PathBuf, EditorError> {
        Err(EditorError::UiAsset("missing import".to_string()))
    };

    collect_ui_asset_import_document(
        &resolve,
        "res://ui/missing.zui#Primary",
        UiAssetKind::Widget,
        &mut traversal,
    )
    .expect_err("missing import must remain diagnostic");

    assert_eq!(
        traversal.finish_resolution().dependencies,
        ["res://ui/missing.zui".to_string()].into_iter().collect()
    );
}
