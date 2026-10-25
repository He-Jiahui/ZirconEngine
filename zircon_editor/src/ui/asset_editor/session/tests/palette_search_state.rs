use super::super::command::UiAssetEditorCommand;
use super::{UiAssetEditorSession, UiAssetEditorSessionError};
use crate::ui::asset_editor::palette::UiAssetPaletteEntryKind;
use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute};
use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

const VIEW: &str = r#"
[asset]
kind = "view"
id = "editor.test.palette_search"
version = 2
display_name = "Palette Search"
[root]
node = "root"
[nodes.root]
component = "VerticalGroup"
"#;

const COMPONENT: &str = r#"
[asset]
kind = "component"
id = "editor.test.palette_search.component"
version = 2
[components.ImportedWidget]
root = "imported"
[nodes.imported]
component = "Text"
props = { text = "Imported" }
"#;

fn session() -> UiAssetEditorSession {
    UiAssetEditorSession::from_v2_source(
        UiAssetEditorRoute::new(
            "editor.test.palette_search",
            UiAssetKind::Layout,
            UiAssetEditorMode::Design,
        ),
        VIEW,
        UiSize::new(640.0, 360.0),
    )
    .expect("real v2 session")
}

fn import(session: &mut UiAssetEditorSession, reference: &str) {
    session
        .register_v2_widget_import(
            reference,
            UiV2AssetLoader::load_toml_str(COMPONENT).expect("component"),
        )
        .expect("actual v2 import");
}

#[test]
fn filtered_row_selects_its_real_reference_instead_of_catalog_zero() {
    let mut session = session();
    let reference = "res://ui/widgets/z_last.zui#ImportedWidget";
    import(&mut session, reference);
    let original = session.pane_presentation().palette_items;
    assert!(session.set_palette_search_query("ImportedWidget"));
    let view = session.palette_search_snapshot();
    assert_eq!(view.result_count(), 1);
    let (catalog_index, entry) = view.rows().next().expect("one reference row");
    assert_ne!(catalog_index, 0);
    assert!(matches!(
        &entry.kind,
        UiAssetPaletteEntryKind::Reference { component_ref } if component_ref == reference
    ));
    session
        .select_palette_search_row(&view, 0)
        .expect("select actual filtered reference");
    assert_eq!(session.selected_palette_index, Some(catalog_index));
    let selected = session.palette_search_snapshot();
    assert_eq!(selected.selected_row(), Some(0));
    assert_eq!(selected.visible_start(), 0);
    assert_eq!(session.pane_presentation().palette_items, original);
}

#[test]
fn empty_query_result_clears_palette_selection_without_document_history() {
    let mut session = session();
    import(&mut session, "res://ui/widgets/hud.zui#ImportedWidget");
    session.set_palette_search_query("ImportedWidget");
    let view = session.palette_search_snapshot();
    session.select_palette_search_row(&view, 0).expect("select");
    let source = session.source_buffer().text().to_string();
    let revision = session.source_revision();
    assert!(!session.can_undo());

    assert!(session.set_palette_search_query("不存在__ZR_NO_MATCH__"));
    let empty = session.palette_search_snapshot();
    assert_eq!(empty.result_count(), 0);
    assert!(empty.rows().next().is_none());
    assert!(!empty.has_previous() && !empty.has_next() && !empty.can_select());
    assert_eq!(empty.selected_row(), None);
    assert_eq!(session.selected_palette_index, None);
    assert!(session.selected_palette_entry.is_none());
    assert_eq!(session.source_buffer().text(), source);
    assert_eq!(session.source_revision(), revision);
    assert!(!session.can_undo() && !session.can_redo() && !session.source_buffer().is_dirty());
    assert!(!session.pane_presentation().can_insert_child);
}

#[test]
fn clearing_query_restores_original_catalog_order_and_source_identity() {
    let mut session = session();
    let full = session.palette_search_snapshot();
    let labels = full
        .rows()
        .map(|(index, entry)| (index, entry.label.clone()))
        .collect::<Vec<_>>();
    let total = full.total_count();
    session.set_palette_search_query("不存在__ZR_NO_MATCH__");
    assert!(session.set_palette_search_query(""));
    let restored = session.palette_search_snapshot();
    assert_eq!(restored.result_count(), total);
    assert_eq!(restored.query(), "");
    assert_eq!(restored.asset_id(), session.route().asset_id);
    assert_eq!(restored.source_revision(), full.source_revision());
    assert_eq!(
        restored
            .rows()
            .map(|(index, entry)| (index, entry.label.clone()))
            .collect::<Vec<_>>(),
        labels
    );
    assert!(!session.can_undo());
}

#[test]
fn unicode_terms_and_reference_path_disambiguate_identical_display_labels() {
    let mut session = session();
    import(&mut session, "res://ui/widgets/a_same.zui#ImportedWidget");
    let reference = "res://ui/widgets/用户库.zui#ImportedWidget";
    import(&mut session, reference);
    let query = "  用户库   IMPORTEDwidget  ";
    session.set_palette_search_query(query);
    let view = session.palette_search_snapshot();
    assert_eq!(view.query(), query, "keep the complete editable query");
    assert_eq!(view.result_count(), 1);
    assert!(matches!(
        &view.rows().next().expect("reference").1.kind,
        UiAssetPaletteEntryKind::Reference { component_ref } if component_ref == reference
    ));
}

#[test]
fn query_change_retires_old_rows_even_when_the_query_returns_to_its_old_value() {
    let mut session = session();
    let old = session.palette_search_snapshot();
    session.set_palette_search_query("不存在__ZR_NO_MATCH__");
    session.set_palette_search_query("");
    let selected = session.selected_palette_index;
    assert!(matches!(
        session.select_palette_search_row(&old, 0),
        Err(UiAssetEditorSessionError::RetiredPaletteSearch)
    ));
    assert_eq!(session.selected_palette_index, selected);
    assert!(!session.can_undo());
}

#[test]
fn page_change_retires_rows_and_extreme_page_request_is_clamped() {
    let mut session = session();
    let first = session.palette_search_snapshot();
    assert!(first.result_count() > 4, "use actual registered catalog");
    assert!(session.set_palette_visible_start(usize::MAX));
    let last = session.palette_search_snapshot();
    assert_eq!(last.visible_start(), (last.result_count() - 1) / 4 * 4);
    assert!(!last.has_next() && last.has_previous());
    assert!((1..=4).contains(&last.rows().count()));
    session.set_palette_visible_start(0);
    assert!(matches!(
        session.select_palette_search_row(&first, 0),
        Err(UiAssetEditorSessionError::RetiredPaletteSearch)
    ));
    session.set_palette_search_query("不存在__ZR_NO_MATCH__");
    assert!(!session.set_palette_visible_start(usize::MAX));
    assert_eq!(session.palette_search_snapshot().visible_start(), 0);
}

#[test]
fn same_query_does_not_retire_an_unchanged_live_view() {
    let mut session = session();
    import(&mut session, "res://ui/widgets/hud.zui#ImportedWidget");
    session.set_palette_search_query("ImportedWidget");
    let view = session.palette_search_snapshot();
    assert!(!session.set_palette_search_query("ImportedWidget"));
    assert!(!session.set_palette_visible_start(0));
    assert!(session.select_palette_search_row(&view, 0).is_ok());
}

#[test]
fn catalog_refresh_preserves_query_and_retires_old_index_mapping() {
    let mut session = session();
    let reference = "res://ui/widgets/z_last.zui#ImportedWidget";
    import(&mut session, reference);
    session.set_palette_search_query("ImportedWidget");
    let old = session.palette_search_snapshot();
    session.select_palette_search_row(&old, 0).expect("select");
    let old_index = session.selected_palette_index.expect("reference index");
    import(&mut session, "res://ui/widgets/a_first.zui#ImportedWidget");
    let current = session.palette_search_snapshot();
    assert_eq!(current.query(), "ImportedWidget");
    assert_eq!(current.result_count(), 2);
    assert_ne!(session.selected_palette_index, Some(old_index));
    assert!(matches!(
        session.selected_palette_entry.as_ref().map(|entry| &entry.kind),
        Some(UiAssetPaletteEntryKind::Reference { component_ref }) if component_ref == reference
    ));
    assert!(matches!(
        session.select_palette_search_row(&old, 0),
        Err(UiAssetEditorSessionError::RetiredPaletteSearch)
    ));
}

#[test]
fn source_edit_and_undo_do_not_reactivate_an_old_row_snapshot() {
    let mut session = session();
    let old = session.palette_search_snapshot();
    session
        .apply_command(UiAssetEditorCommand::edit_source(
            VIEW.replace("Palette Search", "Edited Palette Search"),
        ))
        .expect("source edit");
    assert!(session.undo().expect("undo edit"));
    assert_eq!(session.source_buffer().text(), VIEW);
    assert!(matches!(
        session.select_palette_search_row(&old, 0),
        Err(UiAssetEditorSessionError::RetiredPaletteSearch)
    ));
}

#[test]
fn invalid_source_and_fresh_document_instance_reject_old_or_disabled_rows() {
    let mut first = session();
    let original = first.palette_search_snapshot();
    let mut reopened = session();
    assert!(matches!(
        reopened.select_palette_search_row(&original, 0),
        Err(UiAssetEditorSessionError::RetiredPaletteSearch)
    ));
    first
        .apply_command(UiAssetEditorCommand::edit_source("[asset\n"))
        .expect("invalid editable source is retained");
    let invalid = first.palette_search_snapshot();
    assert!(!invalid.can_select());
    let selected = first.selected_palette_index;
    assert!(matches!(
        first.select_palette_search_row(&invalid, 0),
        Err(UiAssetEditorSessionError::InvalidSourceBuffer)
    ));
    assert_eq!(first.selected_palette_index, selected);
}

#[test]
fn row_outside_current_page_does_not_mutate_palette_selection() {
    let mut session = session();
    let view = session.palette_search_snapshot();
    let selected = session.selected_palette_index;
    for row in [4, usize::MAX] {
        assert!(matches!(
            session.select_palette_search_row(&view, row),
            Err(UiAssetEditorSessionError::InvalidPaletteSearchRow { row: actual }) if actual == row
        ));
    }
    assert_eq!(session.selected_palette_index, selected);
    assert!(!session.can_undo());
}

#[test]
fn filtered_reference_insert_updates_real_v2_source_and_undo_keeps_query() {
    let mut session = session();
    let reference = "res://ui/widgets/hud.zui#ImportedWidget";
    import(&mut session, reference);
    session.set_palette_search_query("ImportedWidget");
    let view = session.palette_search_snapshot();
    session.select_palette_search_row(&view, 0).expect("select");
    assert!(session
        .insert_selected_palette_item_as_child()
        .expect("insert real reference"));
    let edited = session.source_buffer().text().to_string();
    assert!(edited.contains(reference));
    UiV2AssetLoader::load_toml_str(&edited).expect("still real v2 source");
    assert_eq!(session.palette_search_snapshot().query(), "ImportedWidget");
    assert!(session.can_undo());
    assert!(session.undo().expect("undo insertion"));
    assert_eq!(session.source_buffer().text(), VIEW);
    assert_eq!(session.palette_search_snapshot().query(), "ImportedWidget");
    assert!(session.redo().expect("redo insertion"));
    assert_eq!(session.source_buffer().text(), edited);
    assert_eq!(session.palette_search_snapshot().query(), "ImportedWidget");
}

#[test]
fn focused_query_continues_on_same_source_but_old_rows_and_pages_retire() {
    let mut session = session();
    let focused = session.palette_search_snapshot();
    let text = session.source_buffer().text().to_string();
    assert!(session
        .set_palette_search_query_from_snapshot(&focused, "Button")
        .unwrap());
    assert!(session
        .set_palette_search_query_from_snapshot(&focused, "不存在")
        .unwrap());
    assert!(session
        .set_palette_search_query_from_snapshot(&focused, "")
        .unwrap());
    assert!(matches!(
        session.select_palette_search_row(&focused, 0),
        Err(UiAssetEditorSessionError::RetiredPaletteSearch)
    ));
    assert!(matches!(
        session.set_palette_search_page_from_snapshot(&focused, 4),
        Err(UiAssetEditorSessionError::RetiredPaletteSearch)
    ));
    let current = session.palette_search_snapshot();
    assert!(session
        .set_palette_search_page_from_snapshot(&current, 4)
        .unwrap());
    assert_eq!(session.source_buffer().text(), text);
    assert!(!session.can_undo() && !session.source_buffer().is_dirty());
}

#[test]
fn query_continuation_rejects_reopened_and_edited_then_undone_sources() {
    let mut first = session();
    let focused = first.palette_search_snapshot();
    assert!(matches!(
        session().set_palette_search_query_from_snapshot(&focused, "Button"),
        Err(UiAssetEditorSessionError::RetiredPaletteSearch)
    ));
    first
        .apply_command(UiAssetEditorCommand::edit_source(format!("{VIEW}\n")))
        .unwrap();
    assert!(first.undo().unwrap());
    assert_eq!(first.source_buffer().text(), VIEW);
    assert!(matches!(
        first.set_palette_search_query_from_snapshot(&focused, "Button"),
        Err(UiAssetEditorSessionError::RetiredPaletteSearch)
    ));
}

#[test]
fn pane_publication_keeps_same_filtered_snapshot_and_full_legacy_catalog() {
    let mut session = session();
    let legacy = session.pane_presentation().palette_items;
    import(&mut session, "res://ui/widgets/checked.zui#ImportedWidget");
    let full = session.pane_presentation().palette_items;
    assert!(full.len() > legacy.len());
    session.set_palette_search_query("ImportedWidget");
    let pane = session.pane_presentation();
    let snapshot = pane.palette_search.expect("real domain snapshot");
    assert_eq!(pane.palette_items, full);
    assert_eq!(snapshot.query(), "ImportedWidget");
    assert_eq!(snapshot, session.palette_search_snapshot());
    let (catalog_index, entry) = snapshot.rows().next().expect("filtered reference");
    assert_ne!(catalog_index, 0);
    assert_eq!(full[catalog_index], entry.label);
    assert_eq!(snapshot.catalog_index_for_row(0), Some(catalog_index));
}
