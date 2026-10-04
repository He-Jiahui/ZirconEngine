use std::collections::BTreeMap;

use super::UiAssetEditorSession;
use crate::ui::asset_editor::palette::UiAssetPaletteEntryKind;
use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute, UiAssetPreviewPreset};
use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

const PREVIEW_HIT_INDEX_LAYOUT: &str = r#"
[asset]
kind = "layout"
id = "editor.test.preview_hit_index"
version = 1
display_name = "Preview Hit Index"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 8.0 } }
children = [{ child = "status" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { text = "Ready" }
layout = { width = { stretch = "Stretch" }, height = { min = 24.0, preferred = 24.0, max = 24.0, stretch = "Fixed" } }
"#;

const V2_PALETTE_VIEW: &str = r#"
[asset]
kind = "view"
id = "editor.test.palette_catalog.v2"
version = 2
display_name = "Palette Catalog V2"

[root]
node = "root"

[nodes.root]
component = "VerticalGroup"
"#;

const V2_PALETTE_COMPONENT: &str = r#"
[asset]
kind = "component"
id = "editor.test.palette_catalog.component"
version = 2
display_name = "Palette Catalog Component"

[components.ImportedWidget]
root = "imported_root"

[nodes.imported_root]
component = "Text"
props = { text = "Imported" }
"#;

#[test]
fn palette_drag_reuses_the_hit_index_until_preview_or_document_rebuild() {
    let route = UiAssetEditorRoute::new(
        "editor.test.preview_hit_index",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_HIT_INDEX_LAYOUT,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    session
        .select_palette_index(0)
        .expect("native palette entry");

    session
        .update_palette_drag_target(16.0, 16.0)
        .expect("first palette drag resolution");
    let first_target = session
        .selected_palette_drag_target()
        .cloned()
        .expect("first palette drag target");
    assert_eq!(session.preview_hit_index_build_count(), 1);

    session
        .update_palette_drag_target(16.0, 16.0)
        .expect("stable palette drag resolution");
    assert_eq!(session.preview_hit_index_build_count(), 1);
    assert_eq!(
        session.selected_palette_drag_target(),
        Some(&first_target),
        "stable hover preserves the same drag target semantics"
    );

    session.rebuild_preview_snapshot().expect("preview rebuild");
    assert!(session.preview_hit_index.is_none());
    session
        .update_palette_drag_target(16.0, 16.0)
        .expect("drag after preview rebuild");
    assert_eq!(session.preview_hit_index_build_count(), 2);

    session
        .set_preview_preset(UiAssetPreviewPreset::Dialog)
        .expect("preview preset");
    assert!(session.preview_hit_index.is_none());
    session
        .update_palette_drag_target(16.0, 16.0)
        .expect("drag after preview resize");
    assert_eq!(session.preview_hit_index_build_count(), 3);

    let document = session.last_valid_document.clone();
    session
        .apply_valid_document(document)
        .expect("document replacement");
    assert!(session.preview_hit_index.is_none());
    session
        .update_palette_drag_target(16.0, 16.0)
        .expect("drag after document replacement");
    assert_eq!(session.preview_hit_index_build_count(), 4);
}

#[test]
fn palette_catalog_is_reused_until_the_document_generation_changes() {
    let route = UiAssetEditorRoute::new(
        "editor.test.palette_catalog",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_HIT_INDEX_LAYOUT,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    assert_eq!(session.palette_catalog_build_count(), 1);
    let first_presentation = session.pane_presentation();
    assert!(!first_presentation.palette_items.is_empty());
    session.select_palette_index(0).expect("palette selection");
    let repeated_presentation = session.pane_presentation();
    assert_eq!(
        repeated_presentation.palette_items,
        first_presentation.palette_items
    );
    assert_eq!(session.palette_catalog_build_count(), 1);

    let mut imported_widget = session.last_valid_document.clone();
    imported_widget.asset.kind = UiAssetKind::Widget;
    session
        .register_widget_import(
            "res://ui/widgets/imported.zui#ImportedWidget",
            imported_widget,
        )
        .expect("widget import");
    assert_eq!(session.palette_catalog_build_count(), 2);
    assert!(session
        .pane_presentation()
        .palette_items
        .iter()
        .any(|item| item == "Reference / ImportedWidget"));

    let mut imported_style = session.last_valid_document.clone();
    imported_style.asset.kind = UiAssetKind::Style;
    session
        .register_style_import("res://ui/styles/imported.zss", imported_style)
        .expect("style import");
    assert_eq!(session.palette_catalog_build_count(), 2);

    let imported_index = session.palette_catalog.entries().len() - 1;
    session
        .select_palette_index(imported_index)
        .expect("imported palette selection");
    assert!(session
        .selected_palette_entry
        .as_ref()
        .expect("selected imported palette entry")
        .label
        .starts_with("Reference / "));
    session
        .replace_resolved_imports(
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
        )
        .expect("clear imports");
    assert_eq!(session.palette_catalog_build_count(), 3);
    assert!(session
        .selected_palette_entry
        .as_ref()
        .expect("reconciled palette entry")
        .label
        .starts_with("Native / "));

    let document = session.last_valid_document.clone();
    session
        .apply_valid_document(document)
        .expect("document refresh");
    assert_eq!(session.palette_catalog_build_count(), 4);
}

#[test]
fn v2_widget_import_refreshes_the_palette_catalog() {
    let route = UiAssetEditorRoute::new(
        "editor.test.palette_catalog.v2",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session =
        UiAssetEditorSession::from_v2_source(route, V2_PALETTE_VIEW, UiSize::new(640.0, 360.0))
            .expect("v2 session");
    let component = UiV2AssetLoader::load_toml_str(V2_PALETTE_COMPONENT).expect("v2 component");

    assert_eq!(session.palette_catalog_build_count(), 1);
    session
        .register_v2_widget_import("res://ui/widgets/imported_widget.zui", component)
        .expect("v2 widget import");

    assert_eq!(session.palette_catalog_build_count(), 2);
    assert!(session
        .pane_presentation()
        .palette_items
        .iter()
        .any(|item| item == "Reference / ImportedWidget"));
    let imported_index = session
        .palette_catalog
        .entries()
        .iter()
        .position(|entry| entry.label == "Reference / ImportedWidget")
        .expect("v2 palette reference");
    assert!(session
        .palette_catalog
        .reference_imports()
        .contains_key("res://ui/widgets/imported_widget.zui#ImportedWidget"));
    session
        .select_palette_index(imported_index)
        .expect("select v2 palette reference");
    assert!(session.pane_presentation().can_insert_child);
    assert!(session
        .insert_selected_palette_item_as_child()
        .expect("insert v2 palette reference"));
}

#[test]
fn palette_reference_selection_survives_a_lexically_earlier_import() {
    let route = UiAssetEditorRoute::new(
        "editor.test.palette_catalog.v2.selection",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session =
        UiAssetEditorSession::from_v2_source(route, V2_PALETTE_VIEW, UiSize::new(640.0, 360.0))
            .expect("v2 session");
    let component = UiV2AssetLoader::load_toml_str(V2_PALETTE_COMPONENT).expect("v2 component");
    let selected_reference = "res://ui/widgets/z_last.zui#ImportedWidget";

    session
        .register_v2_widget_import(selected_reference, component.clone())
        .expect("register selected reference");
    let selected_index = session
        .palette_catalog
        .entries()
        .iter()
        .position(|entry| {
            matches!(
                &entry.kind,
                UiAssetPaletteEntryKind::Reference { component_ref }
                    if component_ref == selected_reference
            )
        })
        .expect("selected reference index");
    session
        .select_palette_index(selected_index)
        .expect("select reference B");

    session
        .register_v2_widget_import("res://ui/widgets/a_first.zui#ImportedWidget", component)
        .expect("register lexically earlier reference");

    assert!(matches!(
        session
            .selected_palette_entry
            .as_ref()
            .map(|entry| &entry.kind),
        Some(UiAssetPaletteEntryKind::Reference { component_ref })
            if component_ref == selected_reference
    ));
}
