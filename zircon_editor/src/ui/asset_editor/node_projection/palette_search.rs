use super::{UiV2AssetDocument, Value};
use crate::ui::asset_editor::palette::UiAssetPaletteSearchSnapshot;

/// Values used by the existing surface cache; row actions remain page-local.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PaletteSearchProjection {
    query: String,
    rows: Vec<(usize, String)>,
    selected_row: Option<usize>,
    start: usize,
    result_count: usize,
    previous: bool,
    next: bool,
    can_select: bool,
}

impl From<&UiAssetPaletteSearchSnapshot> for PaletteSearchProjection {
    fn from(snapshot: &UiAssetPaletteSearchSnapshot) -> Self {
        Self {
            query: snapshot.query().to_string(),
            rows: snapshot
                .rows()
                .map(|(index, entry)| (index, entry.label.clone()))
                .collect(),
            selected_row: snapshot.selected_row(),
            start: snapshot.visible_start(),
            result_count: snapshot.result_count(),
            previous: snapshot.has_previous(),
            next: snapshot.has_next(),
            can_select: snapshot.can_select(),
        }
    }
}

pub(super) fn palette_search_document(
    source: &UiV2AssetDocument,
    view: &PaletteSearchProjection,
) -> UiV2AssetDocument {
    let mut document = source.clone();
    // A ready empty catalog is distinct from the bootstrap/legacy projection.
    // Remove only transient child mounts, never the original source nodes/IDs.
    if let Some(grid) = document.nodes.get_mut("palette_component_grid") {
        grid.children.truncate(view.rows.len());
    }
    for ((catalog_index, label), (row, key)) in view.rows.iter().zip(
        [
            "palette_button",
            "palette_label",
            "palette_image",
            "palette_container",
        ]
        .iter()
        .enumerate(),
    ) {
        if let Some(node) = document.nodes.get_mut(*key) {
            node.control_id = Some(format!("UiAssetPaletteEntry{catalog_index}"));
            node.props
                .insert("text".into(), Value::String(label.clone()));
            node.props
                .insert("disabled".into(), Value::Boolean(!view.can_select));
            node.props.insert(
                "selected".into(),
                Value::Boolean(view.selected_row == Some(row)),
            );
            node.props.insert(
                "dispatch_kind".into(),
                Value::String("ui_asset_palette".into()),
            );
            node.props
                .insert("action_id".into(), Value::String(row.to_string()));
        }
    }
    if let Some(search) = document.nodes.get_mut("palette_search") {
        search
            .props
            .insert("query".into(), Value::String(view.query.clone()));
        search
            .props
            .insert("value_text".into(), Value::String(view.query.clone()));
        search.props.insert(
            "dispatch_kind".into(),
            Value::String("ui_asset_palette_search".into()),
        );
        search.props.insert(
            "edit_action_id".into(),
            Value::String("palette.search.query".into()),
        );
    }
    if let Some(caption) = document.nodes.get_mut("palette_common_caption") {
        caption.props.insert(
            "text".into(),
            Value::String(
                if view.result_count == 0 {
                    "No matching widgets"
                } else {
                    "COMMON"
                }
                .into(),
            ),
        );
    }
    for (key, start, enabled) in [
        (
            "palette_previous",
            view.start.saturating_sub(4),
            view.previous,
        ),
        ("palette_next", view.start.saturating_add(4), view.next),
    ] {
        if let Some(node) = document.nodes.get_mut(key) {
            node.props
                .insert("disabled".into(), Value::Boolean(!enabled));
            node.props.insert(
                "dispatch_kind".into(),
                Value::String("ui_asset_palette_page".into()),
            );
            node.props
                .insert("action_id".into(), Value::String(start.to_string()));
        }
    }
    document
}

#[cfg(test)]
mod tests {
    use super::super::ui_asset_editor_node_projection_with_palette_search;
    use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession};
    use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

    fn session() -> UiAssetEditorSession {
        UiAssetEditorSession::from_v2_source(
            UiAssetEditorRoute::new(
                "editor.test.palette_projection",
                UiAssetKind::Layout,
                UiAssetEditorMode::Design,
            ),
            r#"[asset]
kind = "view"
id = "editor.test.palette_projection"
version = 2
[root]
node = "root"
[nodes.root]
component = "VerticalGroup"
"#,
            UiSize::new(640.0, 360.0),
        )
        .expect("real v2 document session")
    }

    #[test]
    fn ready_empty_keeps_native_search_and_shell_without_old_entry_hits() {
        let mut session = session();
        session.set_palette_search_query("不存在__ZR_NO_MATCH__");
        let view = ui_asset_editor_node_projection_with_palette_search(
            UiSize::new(900.0, 620.0),
            &session.palette_search_snapshot(),
        );
        assert!(
            !view.nodes.is_empty(),
            "failed projection must not look like ready empty"
        );
        assert!(!view
            .nodes
            .iter()
            .any(|node| node.control_id.as_str().starts_with("UiAssetPaletteEntry")));
        assert!(view
            .nodes
            .iter()
            .any(|node| node.control_id.as_str() == "DesignerPanel"));
        for id in ["PalettePreviousPage", "PaletteNextPage"] {
            assert!(
                view.nodes
                    .iter()
                    .find(|node| node.control_id.as_str() == id)
                    .unwrap()
                    .disabled
            );
        }
        let search = view
            .nodes
            .iter()
            .find(|node| node.control_id.as_str() == "PaletteSearchInput")
            .unwrap();
        assert_eq!(search.role.as_str(), "InputField");
        assert_eq!(search.value_text.as_str(), "不存在__ZR_NO_MATCH__");
        assert_eq!(search.dispatch_kind.as_str(), "ui_asset_palette_search");
    }

    #[test]
    fn page_row_action_is_local_and_control_identity_is_original_catalog_index() {
        let mut session = session();
        assert!(session.set_palette_visible_start(4));
        let snapshot = session.palette_search_snapshot();
        let (catalog_index, _) = snapshot.rows().next().unwrap();
        assert!(catalog_index >= 4);
        let view = ui_asset_editor_node_projection_with_palette_search(
            UiSize::new(1280.0, 800.0),
            &snapshot,
        );
        let id = format!("UiAssetPaletteEntry{catalog_index}");
        let row = view
            .nodes
            .iter()
            .find(|node| node.control_id.as_str() == id)
            .unwrap();
        assert_eq!(row.action_id.as_str(), "0");
        assert_eq!(snapshot.catalog_index_for_row(0), Some(catalog_index));
        assert!(!row.disabled);
    }

    #[test]
    fn literal_placeholder_unicode_and_clear_preserve_actual_query_values() {
        let mut session = session();
        let text = session.source_buffer().text().to_string();
        for query in ["Search widgets", "  中文🙂 long query  ", ""] {
            session.set_palette_search_query(query);
            let snapshot = session.palette_search_snapshot();
            let view = ui_asset_editor_node_projection_with_palette_search(
                UiSize::new(640.0, 520.0),
                &snapshot,
            );
            let search = view
                .nodes
                .iter()
                .find(|node| node.control_id.as_str() == "PaletteSearchInput")
                .unwrap();
            assert_eq!(search.value_text.as_str(), query);
            assert_eq!(
                view.nodes
                    .iter()
                    .filter(|node| node.control_id.as_str().starts_with("UiAssetPaletteEntry"))
                    .count(),
                snapshot.rows().count(),
            );
        }
        assert_eq!(session.source_buffer().text(), text);
        assert!(!session.can_undo() && !session.source_buffer().is_dirty());
    }
}
