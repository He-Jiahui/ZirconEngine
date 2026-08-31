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
use zircon_runtime_interface::project::{
    ProjectActivationOperationId, ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
    ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource, ProjectLaunchTarget,
    ProjectTemplateId,
};

#[test]
fn editor_gui_startup_parser_accepts_project_path() {
    let request = EditorGuiStartupRequestArgs::parse([
        "--project".to_string(),
        "E:/Projects/My Game".to_string(),
    ])
    .unwrap()
    .unwrap();

    let EditorGuiStartupRequest::Project { intent } = request else {
        panic!("--project must produce a versioned project launch intent");
    };
    assert_eq!(intent.source(), ProjectLaunchSource::Cli);
    assert_eq!(intent.profile(), ProjectLaunchProfile::Normal);
    assert_eq!(
        intent.target(),
        &ProjectLaunchTarget::OpenExisting {
            requested_path: PathBuf::from("E:/Projects/My Game"),
        }
    );
}

#[test]
fn editor_gui_startup_parser_rejects_retired_operation_control_flags() {
    for args in [
        vec![
            "--operation".to_string(),
            "window.layout.reset".to_string(),
            "--headless".to_string(),
        ],
        vec!["--list-operations".to_string(), "--headless".to_string()],
        vec!["--operation-history".to_string(), "--headless".to_string()],
    ] {
        let error = EditorGuiStartupRequestArgs::parse(args).unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with("unknown editor GUI startup argument `"),
            "retired operation control flags must fail in the unified launch parser: {error}"
        );
    }
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

    let EditorGuiStartupRequest::Project { intent } = request else {
        panic!("--create-project must produce a versioned project launch intent");
    };
    assert_eq!(intent.source(), ProjectLaunchSource::Cli);
    assert_eq!(
        intent.target(),
        &ProjectLaunchTarget::CreateProject {
            project_name: "My Game".to_string(),
            location: PathBuf::from("E:/Projects"),
            template: ProjectTemplateId::RenderableEmpty,
        }
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
    let project = project_request("E:/Projects/My Game");

    assert_eq!(
        editor_host_startup_request(Some(&project)),
        "project:E:/Projects/My Game"
    );
    assert_eq!(editor_host_startup_request(None), "workspace:welcome");
}

#[cfg(windows)]
#[test]
fn editor_host_startup_request_hides_windows_verbatim_project_path_prefixes() {
    let project = project_request(r"\\?\C:\ZirconBuilds\project");

    assert_eq!(
        editor_host_startup_request(Some(&project)),
        r"project:C:\ZirconBuilds\project"
    );
}

#[test]
fn create_startup_request_reenters_the_admission_path_without_a_prepared_project() {
    let location = unique_temp_project_root("editor-startup-create");
    std::fs::create_dir_all(&location).unwrap();
    let project_root = location.join("StartupProject");
    let resolved_project_root = ProjectPaths::resolve_path(&project_root).unwrap();

    let prepared = prepare_editor_gui_startup(Some(create_project_request(
        "StartupProject",
        location.to_string_lossy(),
    )))
    .unwrap();

    assert_startup_project_path(
        prepared.startup_request.as_ref(),
        resolved_project_root.operation_path(),
    );
    assert!(prepared.entry_config.project_plugin_manifest().is_none());
    assert!(project_root.join("zircon-project.toml").is_file());

    drop(prepared);
    std::fs::remove_dir_all(location).unwrap();
}

#[test]
fn editor_gui_startup_keeps_existing_projects_as_unmaterialized_launch_intents() {
    let root = unique_temp_project_root("startup_manifest_input");
    write_project_manifest_with_plugins(&root, 1_000);
    let manifest = root.join(zircon_runtime::asset::project::PROJECT_MANIFEST_FILE);

    let prepared = prepare_editor_gui_startup(Some(project_request(&manifest))).unwrap();

    assert_startup_project_path(prepared.startup_request.as_ref(), &manifest);
    assert!(prepared.entry_config.project_plugin_manifest().is_none());

    drop(prepared);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn editor_entry_defers_project_materialization_and_plugin_selection_to_admission() {
    let source = include_str!("../../editor.rs");

    assert!(source.contains("prepare_editor_gui_startup(gui_startup_request)"));
    assert!(
        !source.contains("EditorCliOperationRequest"),
        "the retired operation-control parser must not remain in the editor entry"
    );
    assert!(
        !source.contains("authority.open_project("),
        "App startup preparation must not materialize a project before Editor admission"
    );
    assert!(
        !source.contains("prepare_open_project("),
        "the retired App-side project materialization helper must not remain"
    );
    assert!(
        !source
            .contains("first_party_runtime_plugin_registrations_for_manifest_with_render_profile("),
        "project manifest must not select runtime providers before admission"
    );
    assert!(
        !source.contains("first_party_editor_plugin_registrations_for_manifest("),
        "project manifest must not select editor providers before admission"
    );
    assert!(
        !source.contains("selected_native_editor_plugin_registration_reports("),
        "native project plugins must not be discovered before admission"
    );
    assert!(
        !source.contains(".with_prepared_project("),
        "the retained host must receive a launch intent, not an App-materialized project"
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
fn editor_gui_startup_never_projects_manifest_plugins_into_the_bootstrap_configuration() {
    let root = unique_temp_project_root("startup_project_plugin_boundary");
    write_project_manifest_with_plugins(&root, 1_000);

    let prepared = prepare_editor_gui_startup(Some(project_request(&root))).unwrap();

    assert!(prepared.entry_config.project_plugin_manifest().is_none());

    drop(prepared);
    std::fs::remove_dir_all(root).unwrap();
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
    let executable =
        std::env::current_exe().expect("locate the editor GUI startup test executable");
    let binary_directory = executable
        .parent()
        .expect("editor GUI startup test executable must have a parent directory");
    let binary_directory = ProjectPaths::resolve_existing(binary_directory)
        .expect("resolve the editor GUI startup test binary directory");

    binary_directory
        .operation_path()
        .join("zircon-mvp-fixtures")
        .join(format!(
            "zircon_app_{label}_{}_{}",
            std::process::id(),
            unique
        ))
}

fn project_request(project_path: impl Into<PathBuf>) -> EditorGuiStartupRequest {
    EditorGuiStartupRequest::project(
        ProjectLaunchIntent::open_existing(
            test_operation_id(),
            ProjectLaunchSource::Application,
            ProjectLaunchProfile::Normal,
            project_path,
        )
        .unwrap(),
    )
}

fn create_project_request(
    project_name: impl Into<String>,
    location: impl Into<PathBuf>,
) -> EditorGuiStartupRequest {
    EditorGuiStartupRequest::project(
        ProjectLaunchIntent::create_project(
            test_operation_id(),
            ProjectLaunchSource::Application,
            ProjectLaunchProfile::Normal,
            project_name,
            location,
            ProjectTemplateId::RenderableEmpty,
        )
        .unwrap(),
    )
}

fn test_operation_id() -> ProjectActivationOperationId {
    ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
        .allocate()
        .expect("a fresh test operation generator must have its first sequence")
}

fn assert_startup_project_path(
    request: Option<&EditorGuiStartupRequest>,
    expected_path: &std::path::Path,
) {
    let Some(EditorGuiStartupRequest::Project { intent }) = request else {
        panic!("startup must retain a project launch intent");
    };
    assert_eq!(
        intent.target(),
        &ProjectLaunchTarget::OpenExisting {
            requested_path: expected_path.to_path_buf(),
        }
    );
}

#[test]
fn editor_gui_startup_fixture_roots_follow_the_resolved_test_binary_directory() {
    let root = unique_temp_project_root("physical-root");
    let executable =
        std::env::current_exe().expect("locate the editor GUI startup test executable");
    let binary_directory = executable
        .parent()
        .expect("editor GUI startup test executable must have a parent directory");
    let resolved_binary_directory = ProjectPaths::resolve_existing(binary_directory)
        .expect("resolve editor GUI startup test binary directory");

    assert!(
        root.starts_with(resolved_binary_directory.operation_path()),
        "editor GUI startup fixture output must retain the test binary's physical output root"
    );
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
