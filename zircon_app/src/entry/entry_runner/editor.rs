#[cfg(feature = "target-editor-host")]
use std::env;
use std::error::Error;
#[cfg(feature = "target-editor-host")]
use std::ffi::OsString;
#[cfg(feature = "target-editor-host")]
use std::path::PathBuf;

#[cfg(feature = "target-editor-host")]
use serde_json::Value;
#[cfg(feature = "target-editor-host")]
use zircon_editor::{
    core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
        EditorOperationSource,
    },
    core::project::ProjectAuthority,
    run_editor_with_config,
    ui::host::{EditorHostEventController, EditorManager},
    ui::workbench::state::EditorState,
    EditorGuiStartupRequest, EditorHostRunConfig, EditorPluginRegistrationReport,
    RuntimeCapabilities, SessionProfileKind, EDITOR_MANAGER_NAME,
};
#[cfg(feature = "target-editor-host")]
use zircon_runtime::asset::project::ProjectManager;
#[cfg(feature = "target-editor-host")]
use zircon_runtime::core::math::UVec2;
#[cfg(feature = "target-editor-host")]
use zircon_runtime::plugin::RuntimePluginRegistrationReport;

#[cfg(feature = "target-editor-host")]
use crate::entry::first_party_runtime_plugins::first_party_runtime_plugin_registrations_for_manifest_with_render_profile;
#[cfg(feature = "target-editor-host")]
use crate::entry::{
    cli::EditorLaunchArgs, first_party_editor_plugin_registrations_for_config,
    first_party_editor_plugin_registrations_for_manifest,
    first_party_runtime_plugin_registrations_for_config, EntryConfig, EntryProfile,
};

#[cfg(feature = "target-editor-host")]
use super::super::runtime_library::{LoadedRuntime, RuntimeSession};

#[cfg(feature = "target-editor-host")]
mod composition;
#[cfg(feature = "target-editor-host")]
mod project_automation;
#[cfg(feature = "target-editor-host")]
mod startup_diagnostics;

#[cfg(feature = "target-editor-host")]
pub use composition::EditorApplicationComposition;

#[cfg(feature = "target-editor-host")]
use super::diagnostic_log_args::parse_diagnostic_log_startup_args;
use super::EntryRunner;
#[cfg(all(feature = "target-editor-host", test))]
use startup_diagnostics::editor_startup_argument_summary;
#[cfg(feature = "target-editor-host")]
use startup_diagnostics::{
    editor_automation_startup_error, editor_host_startup_error, editor_operation_startup_error,
    editor_startup_argument_error, editor_startup_diagnostic_error, finish_editor_host,
    finish_editor_operation,
};

#[cfg(feature = "target-editor-host")]
const EDITOR_EXIT_AFTER_FIRST_FRAME_ENV: &str = "ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME";
#[cfg(feature = "target-editor-host")]
const EDITOR_CAPTURE_FIRST_FRAME_PNG_ENV: &str = "ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG";

impl EntryRunner {
    pub fn run_editor() -> Result<(), Box<dyn Error>> {
        Self::run_editor_with_args(std::iter::empty::<String>())
    }

    pub fn run_editor_with_args<I, S>(args: I) -> Result<(), Box<dyn Error>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let exit_code = Self::run_editor_with_args_exit_code(args)?;
        if exit_code == 0 {
            Ok(())
        } else {
            Err(format!("editor commandlet completed with exit code {exit_code}").into())
        }
    }

    /// Run the editor executable and return its stable process exit code. Commandlet outcomes
    /// are emitted as JSON before this method returns, while GUI and operation-control startup
    /// retain their existing result contracts.
    pub fn run_editor_with_args_exit_code<I, S>(args: I) -> Result<u8, Box<dyn Error>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        #[cfg(not(feature = "target-editor-host"))]
        {
            let _ = args;
            Err("run_editor requires the `target-editor-host` feature".into())
        }
        #[cfg(feature = "target-editor-host")]
        {
            let diagnostic_args = parse_diagnostic_log_startup_args(args)?;
            zircon_runtime::diagnostic_log::initialize_process_log_with_config(
                "editor",
                diagnostic_args.filter,
            );
            #[cfg(feature = "profiling-tracy")]
            let _ = zircon_runtime::core::diagnostics::profiling::initialize_tracy_sink();
            let remaining_args = diagnostic_args.remaining_args;
            let remaining_args = match EditorLaunchArgs::parse(remaining_args) {
                EditorLaunchArgs::Commandlet(request) => {
                    let report = zircon_editor::core::commandlet::run_commandlet(request);
                    println!("{}", serde_json::to_string(&report)?);
                    return Ok(report.exit_code().as_u8());
                }
                EditorLaunchArgs::CommandletRejected(report) => {
                    println!("{}", serde_json::to_string(&report)?);
                    return Ok(report.exit_code().as_u8());
                }
                EditorLaunchArgs::Standard(args) => args,
            };
            if let Some(automation) =
                project_automation::parse_project_automation_args(remaining_args.clone())
                    .map_err(|error| editor_startup_argument_error(&remaining_args, error))?
            {
                let requested_automation = format!(
                    "project_automation:project={}",
                    automation.project_root.display()
                );
                let report = project_automation::execute_project_automation(
                    automation.project_root,
                    &automation.request,
                )
                .map_err(|error| editor_automation_startup_error(&requested_automation, error))?;
                println!("{}", serde_json::to_string(&report)?);
                return Ok(0);
            }
            let gui_startup_request = EditorGuiStartupRequestArgs::parse(remaining_args.clone())
                .map_err(|error| editor_startup_argument_error(&remaining_args, error))?;
            let request = if gui_startup_request.is_none() {
                EditorCliOperationRequest::parse(remaining_args.clone())
                    .map_err(|error| editor_startup_argument_error(&remaining_args, error))?
            } else {
                None
            };
            if let Some(request) = request {
                let requested_operation = request.startup_request();
                let response = Self::run_editor_operation(request)
                    .map_err(|error| editor_operation_startup_error(&requested_operation, error))?;
                println!("{}", serde_json::to_string(&response)?);
                return Ok(0);
            }
            let first_frame_capture_path = editor_first_frame_capture_path()?;
            let requested_startup = editor_host_startup_request(gui_startup_request.as_ref());
            let prepared_startup = prepare_editor_gui_startup(gui_startup_request).map_err(
                |error| {
                    editor_startup_diagnostic_error(
                        "editor_project",
                        &requested_startup,
                        format!("project preparation failed: {error}"),
                        "verify the requested project path, manifest, template inputs, and filesystem permissions",
                    )
                },
            )?;
            #[cfg(feature = "profiling")]
            let profile_capture =
                zircon_runtime::core::diagnostics::profiling::start_capture_from_env("editor");
            let EditorStartupPreparation {
                entry_config,
                startup_request,
                prepared_project,
                editor_plugin_registrations,
                runtime_plugin_registrations,
                runtime_capabilities,
            } = prepared_startup;
            let editor_host_request = editor_host_startup_request(startup_request.as_ref());
            let core = Self::bootstrap_with_runtime_plugin_registrations(
                entry_config,
                runtime_plugin_registrations,
            )
            .map_err(|error| {
                editor_startup_diagnostic_error(
                    "editor_bootstrap",
                    &editor_host_request,
                    format!("application bootstrap failed: {error}"),
                    "verify the selected profile and staged editor and runtime plugins",
                )
            })?;
            let editor_manager = core
                .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
                .map_err(|error| {
                    editor_startup_diagnostic_error(
                        "editor_manager",
                        &editor_host_request,
                        format!("editor manager resolution failed: {error}"),
                        "verify the editor manager registration and selected startup profile",
                    )
                })?;
            let native_editor_plugin_registrations = prepared_project
                .as_ref()
                .map(|project| {
                    editor_manager.selected_native_editor_plugin_registration_reports(
                        project.paths().root(),
                        &project.manifest().plugins,
                    )
                })
                .unwrap_or_default();
            let mut native_editor_capabilities = native_editor_plugin_registrations
                .iter()
                .flat_map(|registration| registration.capabilities.iter().cloned())
                .collect::<Vec<_>>();
            native_editor_capabilities.sort();
            native_editor_capabilities.dedup();
            if !native_editor_capabilities.is_empty() {
                editor_manager
                    .set_editor_capabilities_enabled(&native_editor_capabilities, true)
                    .map_err(|error| {
                        editor_startup_diagnostic_error(
                            "editor_manager",
                            &editor_host_request,
                            format!("native editor capability activation failed: {error}"),
                            "verify the selected native editor plugins and their declared capabilities",
                        )
                    })?;
            }
            let runtime = LoadedRuntime::load_default().map_err(|error| {
                editor_startup_diagnostic_error(
                    "runtime_library",
                    &editor_host_request,
                    format!("runtime library loading failed: {error}"),
                    "stage a compatible runtime library beside zircon_editor or configure its explicit library path",
                )
            })?;
            // The editor host owns project activation and its registry generation. The gateway
            // session must stay projectless so it cannot open the same project a second time.
            let runtime_session = std::sync::Arc::new(
                RuntimeSession::create_with_profile(runtime, b"editor").map_err(|error| {
                    editor_startup_diagnostic_error(
                        "runtime_session",
                        &editor_host_request,
                        format!("runtime session creation failed: {error}"),
                        "verify the runtime ABI, editor profile, and staged runtime dependencies",
                    )
                })?,
            );
            let runtime_teardown_failure = runtime_session.teardown_failure_state();
            let host_result: Result<_, Box<dyn Error>> = (|| {
                let runtime_gateway = runtime_session
                    .editor_gateway(runtime_capabilities)
                    .map_err(|error| {
                        editor_startup_diagnostic_error(
                            "editor_gateway",
                            &editor_host_request,
                            format!("editor gateway creation failed: {error}"),
                            "verify the runtime capabilities and editor gateway ABI compatibility",
                        )
                    })?;
                let host_config = editor_host_run_config_with_first_frame_exit(
                    startup_request,
                    editor_exit_after_first_frame_enabled(),
                    first_frame_capture_path,
                )
                .with_prepared_project(prepared_project)
                .with_editor_plugin_registrations(editor_plugin_registrations)
                .with_editor_plugin_registrations(native_editor_plugin_registrations);
                run_editor_with_config(core, runtime_gateway, host_config)
                    .map_err(|error| editor_host_startup_error(&editor_host_request, error))?;
                Ok(())
            })();
            #[cfg(feature = "profiling")]
            if profile_capture.is_some() {
                match zircon_runtime::core::diagnostics::profiling::stop_and_export_capture_from_env(
                ) {
                    Some(Ok(report)) => eprintln!("profile report exported: {}", report.export_dir),
                    Some(Err(error)) => eprintln!("profile report export failed: {error}"),
                    None => {}
                }
            }
            drop(runtime_session);
            finish_editor_host(
                &editor_host_request,
                host_result,
                runtime_teardown_failure.take(),
            )?;
            Ok(0)
        }
    }

    #[cfg(feature = "target-editor-host")]
    fn run_editor_operation(
        request: EditorCliOperationRequest,
    ) -> Result<zircon_editor::core::editor_operation::EditorOperationControlResponse, Box<dyn Error>>
    {
        let EditorStartupPreparation {
            entry_config,
            runtime_plugin_registrations,
            runtime_capabilities,
            ..
        } = prepare_editor_operation_startup()?;
        let core = Self::bootstrap_with_runtime_plugin_registrations(
            entry_config,
            runtime_plugin_registrations,
        )?;
        let manager = core.resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)?;
        let state = EditorState::with_default_selection_with_context(
            zircon_runtime::scene::create_default_level(&core)?,
            UVec2::new(1280, 720),
            manager.context().clone(),
        );
        let runtime = EditorHostEventController::new(state, manager);
        let runtime_library = LoadedRuntime::load_default()?;
        let runtime_session = std::sync::Arc::new(RuntimeSession::create_with_profile(
            runtime_library,
            b"editor",
        )?);
        let runtime_teardown_failure = runtime_session.teardown_failure_state();
        let operation_result: Result<_, Box<dyn Error>> = (|| {
            let runtime_gateway = runtime_session.editor_gateway(runtime_capabilities)?;
            runtime.attach_play_gateway(runtime_gateway)?;
            let response = runtime.handle_operation_control_request_from_source(
                EditorOperationSource::Cli,
                request.into_control_request()?,
            );
            Ok(response)
        })();
        drop(runtime);
        drop(runtime_session);
        finish_editor_operation(operation_result, runtime_teardown_failure.take())
    }
}

#[cfg(feature = "target-editor-host")]
fn editor_host_run_config_with_first_frame_exit(
    startup_request: Option<EditorGuiStartupRequest>,
    exit_after_first_frame: bool,
    first_presented_frame_capture_path: Option<PathBuf>,
) -> EditorHostRunConfig {
    let config = EditorHostRunConfig::new().with_startup_request(startup_request);
    let config = if exit_after_first_frame {
        config.with_exit_after_first_presented_frame(true)
    } else {
        config
    };
    if let Some(path) = first_presented_frame_capture_path {
        config.with_first_presented_frame_capture_path(path)
    } else {
        config
    }
}

#[cfg(feature = "target-editor-host")]
fn editor_first_frame_capture_path() -> Result<Option<PathBuf>, std::io::Error> {
    editor_first_frame_capture_path_from_value(env::var_os(EDITOR_CAPTURE_FIRST_FRAME_PNG_ENV))
}

#[cfg(feature = "target-editor-host")]
fn editor_first_frame_capture_path_from_value(
    value: Option<OsString>,
) -> Result<Option<PathBuf>, std::io::Error> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_empty() || value.to_str().is_some_and(|value| value.trim().is_empty()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "editor startup diagnostic: component=editor_host requested={EDITOR_CAPTURE_FIRST_FRAME_PNG_ENV} cause=first-frame PNG capture path is empty or blank recovery=set {EDITOR_CAPTURE_FIRST_FRAME_PNG_ENV} to a writable absolute PNG path or unset it"
            ),
        ));
    }
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "editor startup diagnostic: component=editor_host requested={EDITOR_CAPTURE_FIRST_FRAME_PNG_ENV}={} cause=first-frame PNG capture path must be absolute recovery=set {EDITOR_CAPTURE_FIRST_FRAME_PNG_ENV} to a writable absolute PNG path or unset it",
                path.display()
            ),
        ));
    }
    Ok(Some(path))
}

#[cfg(feature = "target-editor-host")]
struct EditorStartupPreparation {
    entry_config: EntryConfig,
    startup_request: Option<EditorGuiStartupRequest>,
    prepared_project: Option<ProjectManager>,
    editor_plugin_registrations: Vec<EditorPluginRegistrationReport>,
    runtime_plugin_registrations: Vec<RuntimePluginRegistrationReport>,
    runtime_capabilities: RuntimeCapabilities,
    #[cfg(test)]
    startup_metrics: EditorStartupPreparationMetrics,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct EditorStartupPreparationMetrics {
    project_open_count: usize,
    runtime_manifest_projection_count: usize,
    editor_manifest_projection_count: usize,
    project_manifest_clone_count: usize,
    project_manifest_clone_heap_bytes: usize,
}

#[cfg(feature = "target-editor-host")]
fn prepare_editor_gui_startup(
    startup_request: Option<EditorGuiStartupRequest>,
) -> Result<EditorStartupPreparation, Box<dyn Error>> {
    prepare_editor_startup(startup_request, true)
}

#[cfg(feature = "target-editor-host")]
fn prepare_editor_operation_startup() -> Result<EditorStartupPreparation, Box<dyn Error>> {
    prepare_editor_startup(None, false)
}

#[cfg(feature = "target-editor-host")]
fn prepare_editor_startup(
    startup_request: Option<EditorGuiStartupRequest>,
    include_editor_plugin_registrations: bool,
) -> Result<EditorStartupPreparation, Box<dyn Error>> {
    #[cfg(test)]
    let project_open_count = if matches!(
        &startup_request,
        Some(EditorGuiStartupRequest::OpenProject { .. })
    ) {
        1
    } else {
        0
    };
    let (startup_request, prepared_project) = match startup_request {
        Some(EditorGuiStartupRequest::CreateProject(draft)) => {
            let created = ProjectAuthority::default().create_project(&draft)?;
            let request = EditorGuiStartupRequest::open_project(created.root.clone());
            (Some(request), Some(created.into_project()))
        }
        Some(EditorGuiStartupRequest::OpenProject { project_path }) => {
            let (request, project) = prepare_open_project(project_path)?;
            (Some(request), Some(project))
        }
        request => (request, None),
    };
    let entry_config = match prepared_project.as_ref() {
        Some(project) => EntryConfig::new(EntryProfile::Editor)
            .with_project_plugins(project.manifest().plugins.clone()),
        None => EntryConfig::new(EntryProfile::Editor),
    };
    #[cfg(test)]
    let (project_manifest_clone_count, project_manifest_clone_heap_bytes) = entry_config
        .project_plugins
        .as_ref()
        .map(|manifest| (1, project_plugin_manifest_clone_heap_bytes(manifest)))
        .unwrap_or_default();
    let runtime_plugin_registrations = match prepared_project.as_ref() {
        Some(project) => first_party_runtime_plugin_registrations_for_manifest_with_render_profile(
            entry_config.target_mode,
            &project.manifest().plugins,
            &entry_config.render_profile,
        ),
        None => first_party_runtime_plugin_registrations_for_config(&entry_config),
    };
    let runtime_capabilities = RuntimeCapabilities::from_runtime_plugin_registrations(
        SessionProfileKind::Editor,
        &runtime_plugin_registrations,
    );
    let editor_plugin_registrations = include_editor_plugin_registrations
        .then(|| match entry_config.project_plugins.as_ref() {
            Some(manifest) => first_party_editor_plugin_registrations_for_manifest(
                entry_config.target_mode,
                manifest,
            ),
            None => first_party_editor_plugin_registrations_for_config(&entry_config),
        })
        .unwrap_or_default();

    Ok(EditorStartupPreparation {
        entry_config,
        startup_request,
        prepared_project,
        editor_plugin_registrations,
        runtime_plugin_registrations,
        runtime_capabilities,
        #[cfg(test)]
        startup_metrics: EditorStartupPreparationMetrics {
            project_open_count,
            runtime_manifest_projection_count: 1,
            editor_manifest_projection_count: if include_editor_plugin_registrations {
                1
            } else {
                0
            },
            project_manifest_clone_count,
            project_manifest_clone_heap_bytes,
        },
    })
}

#[cfg(test)]
fn project_plugin_manifest_clone_heap_bytes(
    manifest: &zircon_runtime::core::framework::project::ProjectPluginManifest,
) -> usize {
    manifest.selections.capacity()
        * std::mem::size_of::<zircon_runtime::core::framework::project::ProjectPluginSelection>()
        + manifest
            .selections
            .iter()
            .map(|selection| {
                selection.id.capacity()
                    + selection.target_modes.capacity()
                        * std::mem::size_of::<zircon_runtime::core::framework::platform::RuntimeTargetMode>()
                    + selection.runtime_crate.as_ref().map_or(0, String::capacity)
                    + selection.editor_crate.as_ref().map_or(0, String::capacity)
                    + selection.features.capacity()
                        * std::mem::size_of::<zircon_runtime::core::framework::project::ProjectPluginFeatureSelection>()
                    + selection
                        .features
                        .iter()
                        .map(|feature| {
                            feature.id.capacity()
                                + feature.target_modes.capacity()
                                    * std::mem::size_of::<zircon_runtime::core::framework::platform::RuntimeTargetMode>()
                                + feature.runtime_crate.as_ref().map_or(0, String::capacity)
                                + feature.editor_crate.as_ref().map_or(0, String::capacity)
                                + feature.provider_package_id.as_ref().map_or(0, String::capacity)
                        })
                        .sum::<usize>()
            })
            .sum::<usize>()
}

#[cfg(feature = "target-editor-host")]
fn prepare_open_project(
    project_root: std::path::PathBuf,
) -> Result<(EditorGuiStartupRequest, ProjectManager), Box<dyn Error>> {
    let opened = ProjectAuthority::default().open_project(project_root)?;
    let root = opened.root().to_path_buf();
    Ok((
        EditorGuiStartupRequest::open_project(root),
        opened.into_project(),
    ))
}

#[cfg(feature = "target-editor-host")]
fn editor_exit_after_first_frame_enabled() -> bool {
    editor_exit_after_first_frame_enabled_value(
        env::var_os(EDITOR_EXIT_AFTER_FIRST_FRAME_ENV)
            .as_deref()
            .and_then(|value| value.to_str()),
    )
}

#[cfg(feature = "target-editor-host")]
fn editor_exit_after_first_frame_enabled_value(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        value == "1" || value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("yes")
    })
}

#[cfg(feature = "target-editor-host")]
fn editor_host_startup_request(request: Option<&EditorGuiStartupRequest>) -> String {
    match request {
        Some(EditorGuiStartupRequest::OpenBuiltinView { descriptor_id }) => {
            format!("builtin_view:{descriptor_id}")
        }
        Some(EditorGuiStartupRequest::OpenProject { project_path }) => {
            format!("project:{}", project_path.display())
        }
        Some(EditorGuiStartupRequest::CreateProject(_)) => "project:create".to_string(),
        None => "workspace:welcome".to_string(),
    }
}

#[cfg(feature = "target-editor-host")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct EditorGuiStartupRequestArgs;

#[cfg(feature = "target-editor-host")]
impl EditorGuiStartupRequestArgs {
    fn parse<I>(args: I) -> Result<Option<EditorGuiStartupRequest>, Box<dyn Error>>
    where
        I: IntoIterator<Item = String>,
    {
        let mut args = args.into_iter();
        let mut project_path = None;
        let mut builtin_view = None;
        let mut create_project = false;
        let mut project_name = None;
        let mut location = None;
        let mut template = None;
        let mut saw_gui_arg = false;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--project" => {
                    if project_path.is_some() {
                        return Err("--project was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--project requires a project path".into());
                    };
                    if value.trim().is_empty() {
                        return Err("--project requires a non-empty project path".into());
                    }
                    project_path = Some(value);
                    saw_gui_arg = true;
                }
                "--builtin-view" => {
                    if builtin_view.is_some() {
                        return Err("--builtin-view was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--builtin-view requires a view descriptor id".into());
                    };
                    if value.trim().is_empty() {
                        return Err("--builtin-view requires a non-empty view descriptor id".into());
                    }
                    builtin_view = Some(value);
                    saw_gui_arg = true;
                }
                "--create-project" => {
                    if create_project {
                        return Err("--create-project was provided more than once".into());
                    }
                    create_project = true;
                    saw_gui_arg = true;
                }
                "--project-name" => {
                    if project_name.is_some() {
                        return Err("--project-name was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--project-name requires a value".into());
                    };
                    if value.trim().is_empty() {
                        return Err("--project-name requires a non-empty value".into());
                    }
                    project_name = Some(value);
                    saw_gui_arg = true;
                }
                "--location" => {
                    if location.is_some() {
                        return Err("--location was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--location requires a directory".into());
                    };
                    if value.trim().is_empty() {
                        return Err("--location requires a non-empty directory".into());
                    }
                    location = Some(value);
                    saw_gui_arg = true;
                }
                "--template" => {
                    if template.is_some() {
                        return Err("--template was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--template requires a value".into());
                    };
                    if value != "renderable-empty" {
                        return Err(format!("unsupported project template `{value}`").into());
                    }
                    template = Some(value);
                    saw_gui_arg = true;
                }
                other if saw_gui_arg => {
                    return Err(format!("unknown editor GUI startup argument `{other}`").into());
                }
                _ => return Ok(None),
            }
        }

        if !saw_gui_arg {
            return Ok(None);
        }
        if create_project {
            if project_path.is_some() || builtin_view.is_some() {
                return Err(
                    "--project and --builtin-view cannot be combined with --create-project".into(),
                );
            }
            let Some(project_name) = project_name else {
                return Err("--create-project requires --project-name".into());
            };
            let Some(location) = location else {
                return Err("--create-project requires --location".into());
            };
            if template.as_deref() != Some("renderable-empty") {
                return Err("--create-project requires --template renderable-empty".into());
            }
            return Ok(Some(EditorGuiStartupRequest::create_renderable_empty(
                project_name,
                location,
            )));
        }
        if project_name.is_some() || location.is_some() || template.is_some() {
            return Err(
                "--project-name, --location, and --template require --create-project".into(),
            );
        }
        if project_path.is_some() && builtin_view.is_some() {
            return Err("--project cannot be combined with --builtin-view".into());
        }
        if let Some(descriptor_id) = builtin_view {
            return Ok(Some(EditorGuiStartupRequest::open_builtin_view(
                descriptor_id,
            )));
        }
        let Some(project_path) = project_path else {
            return Ok(None);
        };
        Ok(Some(EditorGuiStartupRequest::open_project(project_path)))
    }
}

#[cfg(feature = "target-editor-host")]
#[derive(Clone, Debug, PartialEq)]
struct EditorCliOperationRequest {
    operation_id: Option<EditorOperationPath>,
    arguments: Value,
    operation_group: Option<String>,
    headless: bool,
    list_operations: bool,
    query_operation_history: bool,
}

#[cfg(feature = "target-editor-host")]
impl EditorCliOperationRequest {
    fn startup_request(&self) -> String {
        if self.list_operations {
            return "operation:list".to_string();
        }
        if self.query_operation_history {
            return "operation:history".to_string();
        }
        match self.operation_id.as_ref() {
            Some(operation_id) => format!("operation:{operation_id}"),
            None => "operation:<invalid>".to_string(),
        }
    }

    fn into_control_request(self) -> Result<EditorOperationControlRequest, Box<dyn Error>> {
        if self.list_operations {
            return Ok(EditorOperationControlRequest::ListOperations);
        }
        if self.query_operation_history {
            return Ok(EditorOperationControlRequest::QueryOperationHistory);
        }
        let Some(operation_id) = self.operation_id else {
            return Err(
                "--operation is required unless --list-operations or --operation-history is set"
                    .into(),
            );
        };
        let mut invocation =
            EditorOperationInvocation::new(operation_id).with_arguments(self.arguments);
        if let Some(operation_group) = self.operation_group {
            invocation = invocation.with_operation_group(operation_group);
        }
        Ok(EditorOperationControlRequest::InvokeOperation(invocation))
    }

    fn parse<I>(args: I) -> Result<Option<Self>, Box<dyn Error>>
    where
        I: IntoIterator<Item = String>,
    {
        let mut args = args.into_iter();
        let mut operation_id = None;
        let mut arguments = Value::Null;
        let mut arguments_provided = false;
        let mut operation_group = None;
        let mut headless = false;
        let mut list_operations = false;
        let mut query_operation_history = false;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--operation" => {
                    if operation_id.is_some() {
                        return Err("--operation was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--operation requires an operation id".into());
                    };
                    operation_id = Some(EditorOperationPath::parse(value)?);
                }
                "--args" => {
                    if arguments_provided {
                        return Err("--args was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--args requires a JSON value".into());
                    };
                    arguments = serde_json::from_str(&value)?;
                    arguments_provided = true;
                }
                "--operation-group" => {
                    if operation_group.is_some() {
                        return Err("--operation-group was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--operation-group requires a group id".into());
                    };
                    if value.trim().is_empty() {
                        return Err("--operation-group requires a non-empty group id".into());
                    }
                    operation_group = Some(value);
                }
                "--list-operations" => {
                    if list_operations {
                        return Err("--list-operations was provided more than once".into());
                    }
                    list_operations = true;
                }
                "--operation-history" => {
                    if query_operation_history {
                        return Err("--operation-history was provided more than once".into());
                    }
                    query_operation_history = true;
                }
                "--headless" => {
                    if headless {
                        return Err("--headless was provided more than once".into());
                    }
                    headless = true;
                }
                other => return Err(format!("unknown editor argument `{other}`").into()),
            }
        }

        if operation_id.is_none() {
            if arguments_provided {
                return Err("--args requires --operation".into());
            }
            if operation_group.is_some() {
                return Err("--operation-group requires --operation".into());
            }
        }
        let operation_mode_count = usize::from(operation_id.is_some())
            + usize::from(list_operations)
            + usize::from(query_operation_history);
        if operation_mode_count > 1 {
            return Err(
                "--operation, --list-operations, and --operation-history are mutually exclusive"
                    .into(),
            );
        }
        if operation_mode_count == 0 {
            if headless {
                return Err(
                    "--headless requires --operation, --list-operations, or --operation-history"
                        .into(),
                );
            }
            return Ok(None);
        }
        if !headless {
            return Err("editor operation control requests require --headless".into());
        }
        Ok(Some(Self {
            operation_id,
            arguments,
            operation_group,
            headless,
            list_operations,
            query_operation_history,
        }))
    }
}

#[cfg(all(test, feature = "target-editor-host"))]
mod tests;
