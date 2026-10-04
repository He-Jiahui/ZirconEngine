use std::collections::BTreeMap;

use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentFlags, UiComponentKeyboardAction, UiComponentState, UiValue,
};

use crate::ui::component::UiComponentDescriptorRegistry;

use super::{
    CatalogNavigation, CommandCatalog, TableIndex, TreeIndex, UiComponentStateModel,
    UiComponentStatePatch,
};

fn command(id: &str, label: &str) -> UiValue {
    let mut value = BTreeMap::new();
    value.insert("id".to_string(), UiValue::String(id.to_string()));
    value.insert("label".to_string(), UiValue::String(label.to_string()));
    UiValue::Map(value)
}

#[test]
fn patch_commits_changed_values_and_flags_once() {
    let mut state = UiComponentState::new().with_value("focused_index", UiValue::Int(0));
    let mut generation = 7;
    let mut patch = UiComponentStatePatch::new();
    patch.set_value("focusedIndex", UiValue::Int(1));
    patch.set_value("focused_index", UiValue::Int(2));
    let mut flags = UiComponentFlags::default();
    flags.focused = true;
    patch.set_flags(flags);

    let change = patch.commit(&mut state, &mut generation);

    assert_eq!(generation, 8);
    assert_eq!(change.generation, 8);
    assert_eq!(change.changed_fields, ["focused_index"]);
    assert!(change.flags_changed);
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(2)));
    assert!(state.flags.focused);
}

#[test]
fn unchanged_patch_does_not_advance_generation_or_write_fields() {
    let mut state =
        UiComponentState::new().with_value("query", UiValue::String("build".to_string()));
    let mut generation = 3;
    let mut patch = UiComponentStatePatch::new();
    patch.set_value("query", UiValue::String("build".to_string()));

    let change = patch.commit(&mut state, &mut generation);

    assert_eq!(generation, 3);
    assert_eq!(change.generation, 3);
    assert!(change.changed_fields.is_empty());
    assert!(!change.flags_changed);
}

#[test]
fn command_catalog_compiles_normalized_entries_and_reuses_filter_for_arrows() {
    let source = UiValue::Array(vec![
        command("open", "Open Scene"),
        command("build", "Build Project"),
        command("reload", "Reload Runtime"),
    ]);
    let mut catalog = CommandCatalog::compile(&source, 11);
    assert_eq!(catalog.entry_count(), 3);
    assert_eq!(catalog.parse_count(), 1);

    assert!(catalog.filter(Some("build"), None));
    assert_eq!(catalog.filtered_ids().collect::<Vec<_>>(), ["build"]);
    let filter_rebuilds = catalog.filter_rebuild_count();
    assert_eq!(catalog.move_focus(None, CatalogNavigation::Next), Some(0));
    assert_eq!(
        catalog.move_focus(Some(0), CatalogNavigation::Next),
        Some(0)
    );
    assert_eq!(catalog.filter_rebuild_count(), filter_rebuilds);
    assert!(!catalog.filter(Some("build"), None));
}

#[test]
fn tree_index_owns_visible_rows_and_constant_id_lookup() {
    let root = tree_node("root", vec![tree_node("child", Vec::new())]);
    let mut index = TreeIndex::compile(&UiValue::Array(vec![root]), None, None, 4);
    assert_eq!(index.row_for_id("root"), Some(0));
    assert_eq!(index.visible_id_at(0), Some("root"));
    assert_eq!(index.visible_id_at(1), None);
    let delta = index.set_expanded("root", true);
    assert!(delta.changed);
    assert_eq!(index.visible_id_at(1), Some("child"));
    assert_eq!(index.parent_of("child"), Some("root"));
    assert_eq!(index.label_of("child"), Some("child"));
}

#[test]
fn table_index_returns_typed_permutation_and_rejects_stale_result() {
    let rows = UiValue::Array(vec![row("a", 20), row("b", 3), row("c", 11)]);
    let index = TableIndex::compile(None, Some(&rows), 9);
    let result = index.sort_permutation(&rows, "count", "asc");
    assert_eq!(result.permutation, [1, 2, 0]);
    assert!(index.accept_sort_result(result.clone(), 9).is_some());
    assert!(index.accept_sort_result(result, 10).is_none());
}

#[test]
fn model_command_arrows_reuse_compiled_catalog_and_filter_projection() {
    let descriptor = UiComponentDescriptorRegistry::editor_showcase_shared()
        .descriptor("CommandPalette")
        .expect("command palette descriptor");
    let source = UiValue::Array(vec![
        command("open", "Open Scene"),
        command("build", "Build Project"),
        command("reload", "Reload Runtime"),
    ]);
    let mut model = UiComponentStateModel::new(
        UiComponentState::new()
            .with_value("commands", source)
            .with_value("query", UiValue::String("".to_string())),
    );
    model
        .apply_event(
            descriptor,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::First,
            },
        )
        .expect("first command navigation");
    let (parse_count, filter_rebuild_count) = {
        let catalog = model.command_catalog();
        (catalog.parse_count(), catalog.filter_rebuild_count())
    };
    for action in [
        UiComponentKeyboardAction::Next,
        UiComponentKeyboardAction::Next,
        UiComponentKeyboardAction::Previous,
    ] {
        model
            .apply_event(descriptor, UiComponentEvent::KeyboardAction { action })
            .expect("stable command arrow navigation");
    }
    let catalog = model.command_catalog();
    assert_eq!(catalog.parse_count(), parse_count);
    assert_eq!(catalog.filter_rebuild_count(), filter_rebuild_count);
    assert_eq!(
        model.state().value("filtered_commands"),
        Some(&UiValue::Array(vec![
            UiValue::String("open".to_string()),
            UiValue::String("build".to_string()),
            UiValue::String("reload".to_string()),
        ]))
    );
}

#[test]
fn model_tree_expansion_updates_visible_rows_without_reparsing_source() {
    let descriptor = UiComponentDescriptorRegistry::editor_showcase_shared()
        .descriptor("TreeView")
        .expect("tree view descriptor");
    let mut model = UiComponentStateModel::new(
        UiComponentState::new()
            .with_value(
                "items",
                UiValue::Array(vec![tree_node(
                    "root",
                    vec![tree_node("child", Vec::new())],
                )]),
            )
            .with_value("focused_index", UiValue::Int(0)),
    );
    model
        .apply_event(
            descriptor,
            UiComponentEvent::ToggleExpanded { expanded: true },
        )
        .expect("tree expansion");
    assert_eq!(model.tree_index().visible_id_at(1), Some("child"));
    assert_eq!(
        model.state().value("expanded_items"),
        Some(&UiValue::Array(vec![UiValue::String("root".to_string())]))
    );
}

#[test]
fn model_table_sort_publishes_stable_typed_row_order() {
    let descriptor = UiComponentDescriptorRegistry::editor_showcase_shared()
        .descriptor("Table")
        .expect("table descriptor");
    let rows = UiValue::Array(vec![row("a", 20), row("b", 3), row("c", 11)]);
    let mut model = UiComponentStateModel::new(
        UiComponentState::new()
            .with_value("rows", rows)
            .with_value("sortingMode", UiValue::String("client".to_string())),
    );
    model
        .apply_event(
            descriptor,
            UiComponentEvent::ValueChanged {
                property: "sort_column".to_string(),
                value: UiValue::String("count".to_string()),
            },
        )
        .expect("typed table sort");
    let ids = match model.state().value("rows") {
        Some(UiValue::Array(rows)) => rows
            .iter()
            .filter_map(|row| match row {
                UiValue::Map(values) => values.get("id").and_then(string_value_ref),
                _ => None,
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    assert_eq!(ids, ["b", "c", "a"]);
}

fn string_value_ref(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value),
        _ => None,
    }
}

fn row(id: &str, count: i64) -> UiValue {
    let mut value = BTreeMap::new();
    value.insert("id".to_string(), UiValue::String(id.to_string()));
    value.insert("count".to_string(), UiValue::Int(count));
    UiValue::Map(value)
}

fn tree_node(id: &str, children: Vec<UiValue>) -> UiValue {
    let mut value = BTreeMap::new();
    value.insert("id".to_string(), UiValue::String(id.to_string()));
    value.insert("label".to_string(), UiValue::String(id.to_string()));
    if !children.is_empty() {
        value.insert("children".to_string(), UiValue::Array(children));
    }
    UiValue::Map(value)
}
