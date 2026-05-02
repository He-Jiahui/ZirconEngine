use crate::ui::template::{UiAssetLoader, UiAssetSchemaMigrator};
use zircon_runtime_interface::ui::template::{
    UiAssetError, UiAssetKind, UiAssetMigrationStep, UiAssetSchemaSourceKind,
    UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
};

const CURRENT_TREE_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.schema.current"
version = 3
display_name = "Current Schema"

[root]
node_id = "current_root"
kind = "native"
type = "VerticalBox"
control_id = "CurrentRoot"
"##;

const VERSION_1_TREE_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.schema.v1"
version = 1
display_name = "Version One Schema"

[root]
node_id = "v1_root"
kind = "native"
type = "VerticalBox"
control_id = "V1Root"
"##;

const VERSION_0_TREE_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.schema.v0"
version = 0
display_name = "Version Zero Schema"

[root]
node_id = "v0_root"
kind = "native"
type = "VerticalBox"
control_id = "V0Root"
"##;

const FLAT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.schema.flat"
version = 2
display_name = "Flat Schema"

[root]
node = "editor_root"

[nodes.editor_root]
kind = "native"
type = "VerticalBox"
control_id = "EditorRoot"
children = [{ child = "toolbar", mount = "content" }]

[nodes.toolbar]
kind = "native"
type = "HorizontalBox"
control_id = "Toolbar"
children = [{ child = "open_button" }]

[nodes.open_button]
kind = "native"
type = "Button"
control_id = "OpenButton"
props = { text = "Open" }
"##;

const FUTURE_TREE_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.schema.future"
version = 99
display_name = "Future Schema"

[root]
node_id = "future_root"
kind = "native"
type = "VerticalBox"
"##;

const LEGACY_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "VerticalBox"
control_id = "LegacyRoot"
attributes = { layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" } } }
children = [
  { component = "Button", control_id = "LegacyButton", attributes = { text = "Open" } }
]
"#;

#[test]
fn current_tree_asset_returns_no_op_schema_report() {
    let outcome = UiAssetSchemaMigrator::migrate_toml_str(CURRENT_TREE_ASSET_TOML).unwrap();

    assert_eq!(outcome.document.asset.id, "editor.schema.current");
    assert_eq!(
        outcome.document.asset.version,
        UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
    );
    assert_eq!(
        outcome.report.source_kind,
        UiAssetSchemaSourceKind::CurrentTree
    );
    assert_eq!(
        outcome.report.source_schema_version,
        Some(UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION)
    );
    assert_eq!(
        outcome.report.target_schema_version,
        UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
    );
    assert!(outcome.report.can_edit);
    assert_eq!(
        outcome.report.steps,
        vec![UiAssetMigrationStep::CurrentTreeValidated]
    );
}

#[test]
fn older_tree_asset_bumps_to_current_source_schema_version() {
    let outcome =
        UiAssetLoader::load_toml_str_with_migration_report(VERSION_1_TREE_ASSET_TOML).unwrap();

    assert_eq!(
        outcome.document.asset.version,
        UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
    );
    assert_eq!(
        outcome.report.source_kind,
        UiAssetSchemaSourceKind::OlderTree
    );
    assert_eq!(outcome.report.source_schema_version, Some(1));
    assert!(outcome
        .report
        .steps
        .contains(&UiAssetMigrationStep::SourceVersionBumped {
            from: 1,
            to: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
        }));

    let document = UiAssetLoader::load_toml_str(VERSION_1_TREE_ASSET_TOML).unwrap();
    assert_eq!(
        document.asset.version,
        UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
    );
}

#[test]
fn flat_node_table_asset_materializes_recursive_tree_authority() {
    let outcome = UiAssetSchemaMigrator::migrate_toml_str(FLAT_LAYOUT_ASSET_TOML).unwrap();

    assert_eq!(
        outcome.report.source_kind,
        UiAssetSchemaSourceKind::FlatNodeTable
    );
    assert_eq!(outcome.report.source_schema_version, Some(2));
    assert!(outcome
        .report
        .steps
        .contains(&UiAssetMigrationStep::FlatNodeTableMaterialized));
    assert!(outcome
        .report
        .steps
        .contains(&UiAssetMigrationStep::SourceVersionBumped {
            from: 2,
            to: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
        }));

    let root = outcome.document.root.as_ref().expect("migrated root");
    assert_eq!(root.node_id, "editor_root");
    assert_eq!(root.children[0].mount.as_deref(), Some("content"));
    assert_eq!(root.children[0].node.node_id, "toolbar");
    assert_eq!(
        root.children[0].node.children[0].node.node_id,
        "open_button"
    );

    let canonical = toml::to_string_pretty(&outcome.document).unwrap();
    assert!(
        !canonical.contains("[nodes."),
        "canonical migrated source must not keep flat node tables"
    );
}

#[test]
fn future_schema_version_is_rejected_with_structured_error() {
    let error = UiAssetSchemaMigrator::migrate_toml_str(FUTURE_TREE_ASSET_TOML)
        .expect_err("future schema versions must not be silently downgraded");

    assert_eq!(
        error,
        UiAssetError::UnsupportedSchemaVersion {
            asset_id: "editor.schema.future".to_string(),
            version: 99,
            current: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
        }
    );
}

#[test]
fn below_minimum_schema_version_is_rejected_with_structured_error() {
    let error = UiAssetSchemaMigrator::migrate_toml_str(VERSION_0_TREE_ASSET_TOML)
        .expect_err("schema versions below the supported minimum must be rejected");

    assert_eq!(
        error,
        UiAssetError::UnsupportedSchemaVersion {
            asset_id: "editor.schema.v0".to_string(),
            version: 0,
            current: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
        }
    );
}

#[test]
fn legacy_template_fixture_converts_through_schema_migrator() {
    let outcome = UiAssetSchemaMigrator::migrate_legacy_template_str(
        "legacy.workbench",
        "Legacy Workbench",
        LEGACY_TEMPLATE_TOML,
    )
    .unwrap();

    assert_eq!(
        outcome.report.source_kind,
        UiAssetSchemaSourceKind::LegacyTemplateFixture
    );
    assert_eq!(outcome.report.source_schema_version, Some(1));
    assert_eq!(outcome.document.asset.kind, UiAssetKind::Layout);
    assert_eq!(outcome.document.asset.id, "legacy.workbench");
    assert_eq!(outcome.document.asset.display_name, "Legacy Workbench");
    assert_eq!(
        outcome.document.asset.version,
        UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
    );
    assert!(outcome
        .report
        .steps
        .contains(&UiAssetMigrationStep::LegacyTemplateConverted));

    let root = outcome.document.root.as_ref().expect("legacy root");
    assert_eq!(root.node_id, "root");
    assert_eq!(root.widget_type.as_deref(), Some("VerticalBox"));
    assert_eq!(root.children[0].node.widget_type.as_deref(), Some("Button"));
}
