use super::default_menu_bar_with_sources;

use crate::core::commands::{
    CommandEvalCtx, EditorCommandMenuPath, EditorCommandRegistry, EditorKeymap, MenuBarModel,
    MenuItemModel,
};
use crate::core::editor_extension::EditorMenuItemDescriptor;
use crate::core::editor_message::DocumentId;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::{
    CapabilitySet, ContributionBatch, ContributionSource, ContributionStore,
    DocumentToolkitDescriptor, ToolkitInstanceId, ToolkitLayout,
};
use crate::core::i18n::{EditorI18nService, EditorLocale};

#[test]
fn focused_toolkit_is_the_third_menu_source_with_stable_cross_source_deduplication() {
    let registry = EditorCommandRegistry::default_workbench();
    let keymap = EditorKeymap::default_workbench();
    let mut batch = ContributionBatch::default();
    batch
        .register_menu_item(
            EditorMenuItemDescriptor::new(
                EditorCommandMenuPath::builtin(
                    &operation("view.editor.ui_asset.open"),
                    "document",
                    &["zeta_extension"],
                ),
                operation("view.editor.ui_asset.open"),
            )
            .with_priority(20),
        )
        .unwrap();
    let mut store = ContributionStore::default();
    store
        .contribute(ContributionSource::Builtin, batch)
        .unwrap();
    let contributions = store.snapshot();
    let toolkit = DocumentToolkitDescriptor::new(
        DocumentId::new(7),
        ToolkitInstanceId::parse("view.animation.7").unwrap(),
        "Animation",
        ToolkitLayout::single_tab("animation.layout", "animation.tab").unwrap(),
    )
    .with_menu_items([
        EditorMenuItemDescriptor::new(
            EditorCommandMenuPath::builtin(
                &operation("timeline_sequence.authoring.open"),
                "document",
                &["alpha_toolkit"],
            ),
            operation("timeline_sequence.authoring.open"),
        )
        .with_priority(-10),
        EditorMenuItemDescriptor::new(
            EditorCommandMenuPath::builtin(
                &operation("file.project.open"),
                "document",
                &["duplicate_project_open"],
            ),
            operation("file.project.open"),
        )
        .with_priority(-20),
        EditorMenuItemDescriptor::new(
            EditorCommandMenuPath::builtin(
                &operation("editor.command.palette"),
                "document",
                &["beta_palette"],
            ),
            operation("editor.command.palette"),
        ),
    ]);
    let context = CommandEvalCtx::interactive().with_project_open(true);

    let menu_bar = default_menu_bar_with_sources(
        &registry,
        &keymap,
        &EditorI18nService::default(),
        &EditorLocale::english(),
        &contributions,
        &CapabilitySet::default(),
        Some(&toolkit),
        &context,
    );

    assert_eq!(operation_count(&menu_bar, "file.project.open"), 1);
    assert_eq!(
        menu_labels(&menu_bar, "Document"),
        vec![
            "menu.document.alpha_toolkit.label",
            "menu.document.beta_palette.label",
            "menu.document.zeta_extension.label",
        ]
    );
    assert_eq!(
        menu_item(&menu_bar, "editor.command.palette")
            .shortcut
            .as_deref(),
        Some("Ctrl+Shift+P")
    );

    let without_focus = default_menu_bar_with_sources(
        &registry,
        &keymap,
        &EditorI18nService::default(),
        &EditorLocale::english(),
        &contributions,
        &CapabilitySet::default(),
        None,
        &context,
    );
    assert_eq!(
        operation_count(&without_focus, "timeline_sequence.authoring.open"),
        0
    );
}

#[test]
fn focused_toolkit_does_not_expose_an_operation_without_a_canonical_command() {
    let registry = EditorCommandRegistry::default_workbench();
    let keymap = EditorKeymap::default_workbench();
    let contributions = ContributionStore::default().snapshot();
    let toolkit = DocumentToolkitDescriptor::new(
        DocumentId::new(8),
        ToolkitInstanceId::parse("view.animation.8").unwrap(),
        "Animation",
        ToolkitLayout::single_tab("animation.layout", "animation.tab").unwrap(),
    )
    .with_menu_items([EditorMenuItemDescriptor::new(
        EditorCommandMenuPath::builtin(
            &operation("plugin.missing.operation"),
            "document",
            &["unknown_operation"],
        ),
        operation("plugin.missing.operation"),
    )]);

    let menu_bar = default_menu_bar_with_sources(
        &registry,
        &keymap,
        &EditorI18nService::default(),
        &EditorLocale::english(),
        &contributions,
        &CapabilitySet::default(),
        Some(&toolkit),
        &CommandEvalCtx::interactive(),
    );

    assert_eq!(
        operation_count(&menu_bar, "plugin.missing.operation"),
        0,
        "a menu item cannot outlive the canonical command that owns its operation"
    );
}

fn operation(value: &str) -> EditorOperationPath {
    EditorOperationPath::parse(value).expect("valid fixture operation path")
}

fn operation_count(menu_bar: &MenuBarModel, operation: &str) -> usize {
    menu_bar
        .menus
        .iter()
        .flat_map(|menu| menu.items.iter())
        .map(|item| item_operation_count(item, operation))
        .sum()
}

fn menu_item<'a>(menu_bar: &'a MenuBarModel, operation: &str) -> &'a MenuItemModel {
    menu_bar
        .menus
        .iter()
        .flat_map(|menu| menu.items.iter())
        .find_map(|item| find_menu_item(item, operation))
        .expect("operation should be present in the projected menu")
}

fn find_menu_item<'a>(item: &'a MenuItemModel, operation: &str) -> Option<&'a MenuItemModel> {
    if item
        .operation_path
        .as_ref()
        .is_some_and(|path| path.as_str() == operation)
    {
        return Some(item);
    }
    item.children
        .iter()
        .find_map(|child| find_menu_item(child, operation))
}

fn item_operation_count(item: &MenuItemModel, operation: &str) -> usize {
    usize::from(
        item.operation_path
            .as_ref()
            .is_some_and(|path| path.as_str() == operation),
    ) + item
        .children
        .iter()
        .map(|child| item_operation_count(child, operation))
        .sum::<usize>()
}

fn menu_labels<'a>(menu_bar: &'a MenuBarModel, menu_label: &str) -> Vec<&'a str> {
    menu_bar
        .menus
        .iter()
        .find(|menu| menu.label == menu_label)
        .expect("projected menu should exist")
        .items
        .iter()
        .map(|item| item.label.as_str())
        .collect()
}
