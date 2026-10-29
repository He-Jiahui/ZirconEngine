use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    FunctionAssetImporter, ImportedAsset, UiV2ComponentAsset, UiV2StyleAsset, UiV2ViewAsset,
};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::UiV2AssetKind;

use crate::core::project::{NewProjectDraft, ProjectAuthority, ProjectTemplateId};
use crate::tests::editor_event::support::EventRuntimeHarness;
use crate::ui::workbench::project::EditorProjectDocument;

pub(crate) const DUPLICATE_THEME_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.theme_dedupe"
version = 1
display_name = "Theme Dedupe"

[imports]
styles = ["res://ui/theme/shared_theme.zui"]

[tokens]
accent = "#223344"
panel = "$accent"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "$panel" } }
"##;

pub(crate) const DUPLICATE_IMPORTED_THEME_ASSET: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_theme"
version = 1
display_name = "Shared Theme"

[tokens]
accent = "#223344"
panel = "$accent"

[[stylesheets]]
id = "shared_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "$panel" } }
"##;

pub(crate) const THEME_SUMMARY_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.test.theme_summary"
version = 1
display_name = "Theme Summary"

[imports]
styles = ["res://ui/theme/shared_theme.zui"]

[tokens]
accent = "#4488ff"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
control_id = "RootLabel"
props = { text = "Theme Summary" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "#RootLabel"
set = { self = { text = "Theme Summary Local" } }
"##;

pub(crate) const IMPORTED_THEME_COLLISION_ASSET: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_theme"
version = 1
display_name = "Shared Theme"

[tokens]
accent = "#223344"
panel = "$accent"

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "$panel" } }
"##;

pub(crate) fn unique_temp_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}.json"))
}

pub(crate) fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}"))
}

pub(crate) fn write_ui_asset(path: impl AsRef<Path>, source: &str) {
    crate::tests::support::write_test_ui_asset(path, source).unwrap();
}

// Reuse the complete module bootstrap and watcher-before-Core teardown contract.
pub(crate) struct ThemeRuntimeHarness(EventRuntimeHarness);

impl std::ops::Deref for ThemeRuntimeHarness {
    type Target = CoreRuntime;

    fn deref(&self) -> &Self::Target {
        &self.0.core
    }
}

pub(crate) fn editor_runtime_with_config_path(path: &Path) -> ThemeRuntimeHarness {
    let harness = EventRuntimeHarness::with_enabled_subsystems_at_config_path(
        path.to_path_buf(),
        &[
            crate::ui::host::EDITOR_SUBSYSTEM_ANIMATION_AUTHORING,
            crate::ui::host::EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING,
            crate::ui::host::EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
            crate::ui::host::EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING,
        ],
    );
    // This unit fixture has no App plugin assembly. Supply its real ZUI parser
    // and typed import products before the editor opens or saves project assets.
    let core = harness.core.handle();
    let manager = zircon_runtime::core::manager::resolve_manager_service(
        &core,
        zircon_runtime::asset::project_asset_manager_handle(&core).unwrap(),
    )
    .unwrap();
    manager
        .register_asset_importer(FunctionAssetImporter::new(
            AssetImporterDescriptor::new(
                "test.editor.theme.zui_document",
                "test.editor.theme",
                AssetKind::UiWidget,
                2,
            )
            .with_full_suffixes([".zui"])
            .with_additional_output_kinds([AssetKind::UiLayout, AssetKind::UiStyle]),
            import_theme_ui_asset,
        ))
        .unwrap();
    ThemeRuntimeHarness(harness)
}

fn import_theme_ui_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = UiZuiAssetLoader::load_zui_str(context.source_str()?).map_err(|source| {
        let source = zircon_runtime::asset::assets::UiV2AssetDocumentError::from(source);
        AssetImportError::UiV2Document {
            context: "parse theme test .zui asset",
            source: source.into(),
        }
    })?;
    let imported = match document.asset.kind {
        UiV2AssetKind::View => ImportedAsset::UiV2View(UiV2ViewAsset { document }),
        UiV2AssetKind::Style | UiV2AssetKind::ThemeTokens => {
            ImportedAsset::UiV2Style(UiV2StyleAsset { document })
        }
        UiV2AssetKind::Component => ImportedAsset::UiV2Component(UiV2ComponentAsset { document }),
    };
    Ok(AssetImportOutcome::new(context.uri.clone(), imported))
}

pub(crate) fn setup_theme_project(
    config_prefix: &str,
    project_prefix: &str,
    layout_source: &str,
    imported_source: &str,
) -> (PathBuf, PathBuf, ThemeRuntimeHarness) {
    let config_path = unique_temp_path(config_prefix);
    let project_root = unique_temp_dir(project_prefix);
    let runtime = editor_runtime_with_config_path(&config_path);
    let project_name = project_root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("theme project root must have a UTF-8 final component");
    let location = project_root.parent().expect("theme project root parent");
    ProjectAuthority::default()
        .create_project(
            &NewProjectDraft {
                project_name: project_name.to_string(),
                location: location.to_string_lossy().into_owned(),
                template: ProjectTemplateId::RenderableEmpty,
            },
            &crate::tests::support::test_project_creation_provenance(),
        )
        .expect("theme fixture project should be created");
    let mut project =
        ProjectManager::open(&project_root).expect("theme fixture project should open");
    project
        .scan_and_import()
        .expect("theme fixture project should scan");
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    let source_document = EditorProjectDocument::load_from_project_for_tests(&project).unwrap();
    EditorProjectDocument::save_scene_to_project(
        &project,
        &project.manifest().default_scene,
        &world,
        source_document.scene_source_revision.as_ref().unwrap(),
        None,
    )
    .unwrap();

    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.zui");
    let imported_theme_path = project_root
        .join("assets")
        .join("ui")
        .join("theme")
        .join("shared_theme.zui");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    fs::create_dir_all(imported_theme_path.parent().unwrap()).unwrap();
    write_ui_asset(&layout_path, layout_source);
    write_ui_asset(&imported_theme_path, imported_source);

    (config_path, project_root, runtime)
}

pub(crate) fn cleanup_theme_project(config_path: &Path, project_root: &Path) {
    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_dir_all(project_root);
}
