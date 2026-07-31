use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use super::super::{
    editor_host_startup_error, editor_host_startup_request, editor_startup_argument_error,
    prepare_editor_gui_startup, EditorGuiStartupRequestArgs,
};
use zircon_editor::EditorGuiStartupRequest;
use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{ExportPackagingStrategy, ProjectPluginSelection};

#[test]
fn editor_gui_startup_parser_accepts_project_path() {
    let request = EditorGuiStartupRequestArgs::parse([
        "--project".to_string(),
        "E:/Projects/My Game".to_string(),
    ])
    .unwrap()
    .unwrap();

    assert_eq!(
        request,
        EditorGuiStartupRequest::OpenProject {
            project_path: PathBuf::from("E:/Projects/My Game")
        }
    );
}

#[test]
fn editor_gui_startup_parser_accepts_builtin_view_request() {
    let request = EditorGuiStartupRequestArgs::parse([
        "--builtin-view".to_string(),
        "editor.material_component_lab".to_string(),
    ])
    .unwrap()
    .unwrap();

    assert_eq!(
        request,
        EditorGuiStartupRequest::open_builtin_view("editor.material_component_lab")
    );
}

#[test]
fn editor_gui_startup_parser_accepts_create_project_request() {
    let request = EditorGuiStartupRequestArgs::parse([
        "--create-project".to_string(),
        "--project-name".to_string(),
        "My Game".to_string(),
        "--location".to_string(),
        "E:/Projects".to_string(),
        "--template".to_string(),
        "renderable-empty".to_string(),
    ])
    .unwrap()
    .unwrap();

    assert_eq!(
        request,
        EditorGuiStartupRequest::create_renderable_empty("My Game", "E:/Projects")
    );
}

#[test]
fn editor_gui_startup_parser_rejects_empty_required_values() {
    for (args, expected) in [
        (
            vec!["--project".to_string(), " ".to_string()],
            "--project requires a non-empty project path",
        ),
        (
            vec!["--builtin-view".to_string(), " ".to_string()],
            "--builtin-view requires a non-empty view descriptor id",
        ),
        (
            vec![
                "--create-project".to_string(),
                "--project-name".to_string(),
                " ".to_string(),
                "--location".to_string(),
                "E:/Projects".to_string(),
                "--template".to_string(),
                "renderable-empty".to_string(),
            ],
            "--project-name requires a non-empty value",
        ),
        (
            vec![
                "--create-project".to_string(),
                "--project-name".to_string(),
                "My Game".to_string(),
                "--location".to_string(),
                " ".to_string(),
                "--template".to_string(),
                "renderable-empty".to_string(),
            ],
            "--location requires a non-empty directory",
        ),
    ] {
        let error = EditorGuiStartupRequestArgs::parse(args).unwrap_err();

        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn editor_gui_startup_product_error_is_actionable() {
    let args = vec![
        "--project".to_string(),
        "E:/Projects/MyGame".to_string(),
        "--builtin-view".to_string(),
        "editor.scene".to_string(),
    ];
    let source = EditorGuiStartupRequestArgs::parse(args.clone()).unwrap_err();
    let error = editor_startup_argument_error(&args, source);

    assert_eq!(
        error.to_string(),
        "editor startup diagnostic: component=editor_app requested=--project E:/Projects/MyGame --builtin-view editor.scene cause=--project cannot be combined with --builtin-view recovery=provide one valid editor startup mode and run zircon_editor --help to inspect supported arguments"
    );
}

#[test]
fn editor_host_startup_error_is_actionable_for_an_invalid_builtin_view() {
    let request = EditorGuiStartupRequest::open_builtin_view("editor.missing");
    let source: Box<dyn std::error::Error> =
        "view descriptor `editor.missing` is not registered".into();
    let requested = editor_host_startup_request(Some(&request));
    let error = editor_host_startup_error(&requested, source);

    assert_eq!(
        error.to_string(),
        "editor startup diagnostic: component=editor_host requested=builtin_view:editor.missing cause=editor host execution failed: view descriptor `editor.missing` is not registered recovery=verify the requested project or view and the staged editor assets before retrying zircon_editor"
    );
}

#[test]
fn editor_host_startup_request_identifies_project_and_welcome_modes() {
    let project = EditorGuiStartupRequest::open_project("E:/Projects/My Game");

    assert_eq!(
        editor_host_startup_request(Some(&project)),
        "project:E:/Projects/My Game"
    );
    assert_eq!(editor_host_startup_request(None), "workspace:welcome");
}

#[test]
fn create_startup_request_is_materialized_before_runtime_session_startup() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let location = std::env::temp_dir().join(format!(
        "zircon_app_editor_startup_create_{}_{}",
        std::process::id(),
        unique
    ));
    std::fs::create_dir_all(&location).unwrap();
    let project_root = location.join("StartupProject");

    let prepared =
        prepare_editor_gui_startup(Some(EditorGuiStartupRequest::create_renderable_empty(
            "StartupProject",
            location.to_string_lossy(),
        )))
        .unwrap();

    assert_eq!(
        prepared.startup_request,
        Some(EditorGuiStartupRequest::open_project(&project_root))
    );
    assert_eq!(
        prepared.prepared_project.as_ref().unwrap().paths().root(),
        project_root.as_path()
    );
    assert!(project_root.join("zircon-project.toml").is_file());

    drop(prepared);
    std::fs::remove_dir_all(location).unwrap();
}

#[test]
fn editor_entry_has_one_prepared_startup_projection_owner() {
    let source = include_str!("../../editor.rs");

    assert!(source.contains("prepare_editor_gui_startup(gui_startup_request)"));
    assert!(source.contains("} = prepare_editor_operation_startup()?;"));
    assert_eq!(
        source
            .matches("ProjectAuthority::default().open_project(")
            .count(),
        1,
        "the entry preparation owns the one project open for a startup generation"
    );
    assert_eq!(
        source
            .matches("first_party_runtime_plugin_registrations_for_config(")
            .count(),
        1,
        "GUI and CLI startup must consume one runtime registration projection"
    );
    assert_eq!(
        source
            .matches("first_party_editor_plugin_registrations_for_config(")
            .count(),
        1,
        "the host must receive the one editor registration projection"
    );
    assert!(
        !source.contains("ProjectManager::open("),
        "entry configuration must consume the prepared project instead of reopening it"
    );
    assert!(
        !source.contains("bootstrap_with_first_party_runtime_plugin_registrations("),
        "bootstrap must consume the prepared runtime registrations"
    );
    assert!(
        !source.contains("RuntimeSession::create_with_profile_and_project("),
        "editor entry sessions must remain projectless; the host owns prepared project activation"
    );
    assert!(
        !source.contains("core::editor_plugin::"),
        "the hard cut moved editor plugin contracts to the canonical core::plugin surface"
    );
}

#[test]
fn editor_gui_startup_reuses_the_prepared_manifest_for_editor_registrations() {
    let source = include_str!("../../editor.rs");

    assert!(
        source.contains("Some(manifest) => first_party_editor_plugin_registrations_for_manifest("),
        "a prepared project must lend its manifest to the editor registration projection"
    );
    assert!(
        source.contains("entry_config.project_plugins.as_ref()"),
        "the GUI startup path must borrow the one prepared project-plugin manifest"
    );
}

#[test]
fn editor_startup_projection_scans_1_100_1000_navigation_manifest_rows_once() {
    const SAMPLES: usize = 20;

    for plugin_count in [1usize, 100, 1_000] {
        let root = unique_temp_project_root(&format!("startup_projection_{plugin_count}"));
        write_project_manifest_with_plugins(&root, plugin_count);
        let mut elapsed_micros = Vec::with_capacity(SAMPLES);
        let mut first_sample_micros = None;
        let mut clone_heap_bytes = None;

        for _ in 0..SAMPLES {
            let started = Instant::now();
            let prepared =
                prepare_editor_gui_startup(Some(EditorGuiStartupRequest::open_project(&root)))
                    .unwrap();
            let elapsed = started.elapsed().as_micros();
            first_sample_micros.get_or_insert(elapsed);
            elapsed_micros.push(elapsed);

            assert_eq!(prepared.startup_metrics.project_open_count, 1);
            assert_eq!(
                prepared.startup_metrics.runtime_manifest_projection_count,
                1
            );
            assert_eq!(prepared.startup_metrics.editor_manifest_projection_count, 1);
            assert_eq!(prepared.startup_metrics.project_manifest_clone_count, 1);
            assert_eq!(
                prepared
                    .entry_config
                    .project_plugins
                    .as_ref()
                    .unwrap()
                    .selections
                    .len(),
                plugin_count
            );
            assert!(prepared.startup_metrics.project_manifest_clone_heap_bytes > 0);
            assert_eq!(prepared.runtime_plugin_registrations.len(), 2);
            assert_eq!(
                prepared.runtime_plugin_registrations[0].package_manifest.id,
                RuntimePluginId::Navigation.key()
            );
            assert_eq!(
                prepared.runtime_plugin_registrations[1].package_manifest.id,
                RuntimePluginId::HybridGi.key()
            );
            assert_eq!(prepared.editor_plugin_registrations.len(), 1);
            assert_eq!(
                prepared.editor_plugin_registrations[0].package_manifest.id,
                RuntimePluginId::Navigation.key()
            );
            clone_heap_bytes = Some(prepared.startup_metrics.project_manifest_clone_heap_bytes);
        }

        elapsed_micros.sort_unstable();
        let p95_index = (SAMPLES * 95).div_ceil(100).saturating_sub(1);
        println!(
            "EDITOR01_STARTUP_PROJECTION plugins={plugin_count} samples={SAMPLES} \
             project_open_count=1 runtime_manifest_projection_count=1 \
             editor_manifest_projection_count=1 runtime_registration_count=2 \
             editor_registration_count=1 project_manifest_clone_count=1 \
             project_manifest_clone_heap_bytes={} f0_wall_us={} p95_us={}",
            clone_heap_bytes.unwrap(),
            first_sample_micros.unwrap(),
            elapsed_micros[p95_index],
        );

        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn editor_gui_startup_injects_selected_native_registrations_from_the_prepared_project() {
    let source = include_str!("../../editor.rs");

    assert_eq!(
        source
            .matches("selected_native_editor_plugin_registration_reports(")
            .count(),
        1,
        "a prepared GUI project must produce one native registration projection"
    );
    assert!(source.contains("prepared_project.as_ref()"));
    assert!(source.contains("set_editor_capabilities_enabled(&native_editor_capabilities, true)"));
    assert_eq!(
        source.matches(".with_editor_plugin_registrations(").count(),
        2,
        "first-party and selected native registrations must both be appended to the host config"
    );
    assert_eq!(
        source.matches("NativePluginLoader").count(),
        0,
        "the editor entry must consume the manager's one native load report instead of discovering plugins itself"
    );
}

#[test]
fn editor_gui_startup_parser_leaves_headless_args_for_operation_parser() {
    assert!(EditorGuiStartupRequestArgs::parse([
        "--operation".to_string(),
        "window.layout.reset".to_string(),
        "--headless".to_string(),
    ])
    .unwrap()
    .is_none());
}

#[test]
fn editor_gui_startup_parser_rejects_create_without_required_fields() {
    let error = EditorGuiStartupRequestArgs::parse([
        "--create-project".to_string(),
        "--project-name".to_string(),
        "My Game".to_string(),
    ])
    .unwrap_err();

    assert_eq!(error.to_string(), "--create-project requires --location");
}

#[test]
fn editor_gui_startup_parser_rejects_builtin_view_with_project_path() {
    let error = EditorGuiStartupRequestArgs::parse([
        "--project".to_string(),
        "E:/Projects/My Game".to_string(),
        "--builtin-view".to_string(),
        "editor.material_component_lab".to_string(),
    ])
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "--project cannot be combined with --builtin-view"
    );
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon_app_{label}_{}_{}",
        std::process::id(),
        unique
    ))
}

fn write_project_manifest_with_plugins(root: &std::path::Path, plugin_count: usize) {
    let paths = ProjectPaths::from_root(root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let mut manifest = ProjectManifest::new(
        "Startup Projection",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    manifest.plugins.selections = (0..plugin_count)
        .map(|_| ProjectPluginSelection {
            id: RuntimePluginId::Navigation.key().to_owned(),
            enabled: true,
            required: false,
            target_modes: vec![RuntimeTargetMode::EditorHost],
            packaging: ExportPackagingStrategy::LibraryEmbed,
            runtime_crate: None,
            editor_crate: None,
            features: Vec::new(),
        })
        .collect();
    manifest.save(paths.manifest_path()).unwrap();
}
