#[cfg(feature = "target-editor-host")]
use std::env;
use std::error::Error;
#[cfg(feature = "target-editor-host")]
use std::ffi::OsString;
#[cfg(feature = "target-editor-host")]
use std::path::PathBuf;
#[cfg(feature = "target-editor-host")]
use std::sync::OnceLock;

#[cfg(feature = "target-editor-host")]
use zircon_editor::{
    core::{
        commandlet::run_commandlet_with_host,
        play::{EmbeddedPlayBackend, SharedPlayBackend},
        project::{NewProjectDraft, NewProjectTemplate, ProjectAuthority},
    },
    run_editor_with_config,
    ui::host::EditorManager,
    EditorGuiStartupRequest, EditorHostRunConfig, EditorPluginRegistrationReport,
    RuntimeCapabilities, SessionProfileKind, EDITOR_MANAGER_NAME,
};
#[cfg(feature = "target-editor-host")]
use zircon_runtime::asset::{
    project::{ProjectPaths, ResolvedProjectPath},
    AssetUri,
};
#[cfg(feature = "target-editor-host")]
use zircon_runtime::plugin::RuntimePluginRegistrationReport;
#[cfg(feature = "target-editor-host")]
use zircon_runtime_interface::project::{
    ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId, ProjectLaunchIntent,
    ProjectLaunchProfile, ProjectLaunchSource, ProjectLaunchTarget, ProjectTemplateId,
};

#[cfg(feature = "target-editor-host")]
use crate::entry::{
    cli::{EditorLaunchArgs, EditorLaunchRoute},
    first_party_editor_plugin_registrations_for_config,
    first_party_runtime_plugin_registrations_for_config, EntryConfig, EntryProfile,
    ResolvedProductHostConfig,
};

#[cfg(feature = "target-editor-host")]
use super::super::runtime_library::{LoadedRuntime, RuntimeSession};

#[cfg(feature = "target-editor-host")]
mod composition;
#[cfg(feature = "target-editor-host")]
mod play_session_factory;
#[cfg(feature = "target-editor-host")]
mod project_automation;
#[cfg(feature = "target-editor-host")]
mod startup_diagnostics;

#[cfg(feature = "target-editor-host")]
pub use composition::EditorApplicationComposition;

use super::EntryRunner;
#[cfg(all(feature = "target-editor-host", test))]
pub(crate) use crate::entry::cli::{editor_startup_argument_error, EditorGuiStartupRequestArgs};
#[cfg(feature = "target-editor-host")]
use play_session_factory::AppPlaySessionFactory;
#[cfg(feature = "target-editor-host")]
use startup_diagnostics::{
    editor_host_startup_error, editor_startup_diagnostic_error, finish_editor_host,
    record_editor_host_failure,
};

#[cfg(feature = "target-editor-host")]
const EDITOR_EXIT_AFTER_FIRST_FRAME_ENV: &str = "ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME";
#[cfg(feature = "target-editor-host")]
const EDITOR_CAPTURE_FIRST_FRAME_PNG_ENV: &str = "ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG";
#[cfg(feature = "target-editor-host")]
static APPLICATION_PROJECT_LAUNCH_OPERATION_IDS: OnceLock<ProjectActivationOperationIdGenerator> =
    OnceLock::new();
#[cfg(feature = "target-editor-host")]
const EDITOR_STARTUP_HELP: &str = "\
Usage: zircon_editor [OPTIONS]

Editor GUI:
  --project-launch-intent <json>       Open or create from a versioned cross-process launch intent
  --project <path>                     Open an existing Zircon project
  --scene <res://path.scene.toml>      Open a scene from the requested project
  --builtin-view <descriptor-id>       Open a built-in editor view
  --layout <preset-id>                 Load an existing layout preset after host startup
  --create-project --project-name <name> --location <directory> --template renderable-empty
                                       Create the minimal renderable project template

Hub integration:
  --hub-session <uuid-v4> --hub-protocol 1
                                       Report a versioned project launch outcome to zircon_hub

Headless:
  --run <commandlet>                   Run an editor commandlet
  --run authoring-automation --project <path> --automation <request.json>
                                       Run retained-host authoring bindings

Environment:
  ZIRCON_RUNTIME_LIBRARY                Override the dynamic runtime library with a product-relative or absolute path
  ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG Write the first successfully presented editor frame to a PNG path; relative paths resolve from the launch directory
  ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME  Exit after the first successfully presented editor frame
  ZIRCON_LOG_FILTER                     Override scoped process log filters
  ZIRCON_LOG_LEVEL                      Override the minimum process log level

Options:
  -h, --help                            Print this help without loading the editor host
";

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
    /// are emitted as JSON before this method returns, while GUI startup retains its existing
    /// result contract.
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
            let launch_args = EditorLaunchArgs::parse(args)?;
            zircon_runtime::diagnostic_log::initialize_process_log_with_config(
                "editor",
                launch_args.diagnostic_filter().clone(),
            );
            #[cfg(feature = "profiling-tracy")]
            let _ = zircon_runtime::core::diagnostics::profiling::initialize_tracy_sink();
            let (gui_startup_request, startup_scene_uri, startup_layout_preset, hub_handshake) =
                match launch_args.route()? {
                    EditorLaunchRoute::Help => {
                        println!("{EDITOR_STARTUP_HELP}");
                        return Ok(0);
                    }
                    EditorLaunchRoute::Commandlet(request) => {
                        let commandlet_host =
                            project_automation::EditorProjectAutomationCommandletHost;
                        let report = run_commandlet_with_host(request, &commandlet_host);
                        println!("{}", serde_json::to_string(&report)?);
                        return Ok(report.exit_code().as_u8());
                    }
                    EditorLaunchRoute::CommandletRejected(report) => {
                        println!("{}", serde_json::to_string(&report)?);
                        return Ok(report.exit_code().as_u8());
                    }
                    EditorLaunchRoute::Gui(intent) => intent.into_parts(),
                };
            let first_frame_capture_path = editor_first_frame_capture_path()?;
            let requested_startup = editor_host_startup_request(gui_startup_request.as_ref());
            let starts_with_project = gui_startup_request
                .as_ref()
                .and_then(EditorGuiStartupRequest::project_intent)
                .is_some();
            let runtime_preflight = LoadedRuntime::preflight_default().map_err(|error| {
                editor_startup_diagnostic_error(
                    "runtime_build_set",
                    &requested_startup,
                    format!("runtime BuildSet preflight failed: {error}"),
                    "stage a runtime library and sidecar manifest from the same BuildSet as zircon_editor before opening or creating a project",
                )
            })?;
            let project_runtime_build_set =
                starts_with_project.then(|| runtime_preflight.build_set_id());
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
                editor_plugin_registrations,
                runtime_plugin_registrations,
                runtime_capabilities,
            } = prepared_startup;
            let editor_host_request = editor_host_startup_request(startup_request.as_ref());
            let hub_handshake_config = match hub_handshake {
                Some(handshake) => {
                    let intent = startup_request
                        .as_ref()
                        .and_then(EditorGuiStartupRequest::project_intent)
                        .ok_or_else(|| {
                        editor_startup_diagnostic_error(
                            "hub_handshake",
                            &editor_host_request,
                            "Hub launch did not produce a project launch intent".to_string(),
                            "launch Hub handshakes only with --project-launch-intent and verify the intent targets an existing project",
                        )
                    })?;
                    let ProjectLaunchTarget::OpenExisting { requested_path } = intent.target()
                    else {
                        return Err(editor_startup_diagnostic_error(
                            "hub_handshake",
                            &editor_host_request,
                            "Hub launch retained a create request after project creation"
                                .to_string(),
                            "create the project before creating the Hub handshake configuration",
                        )
                        .into());
                    };
                    let project_root = ProjectPaths::resolve_path(requested_path)
                        .map_err(|error| {
                            editor_startup_diagnostic_error(
                                "hub_handshake",
                                &editor_host_request,
                                format!(
                                    "Hub launch project identity could not be resolved: {error}"
                                ),
                                "verify the Hub project target is an existing accessible directory",
                            )
                        })?
                        .into_operation_path();
                    Some((project_root, handshake.session()))
                }
                None => None,
            };
            let product_composition = Self::compose_resolved_with_runtime_plugin_registrations(
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
            let core = product_composition.core().clone();
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
            drop(editor_manager);
            let factory = std::sync::Arc::new(AppPlaySessionFactory::new(
                runtime_preflight.clone(),
                runtime_capabilities.clone(),
            ));
            let play_backend =
                std::sync::Arc::new(EmbeddedPlayBackend::new(factory)) as SharedPlayBackend;
            let runtime = runtime_preflight.load_after_preflight().map_err(|error| {
                editor_startup_diagnostic_error(
                    "runtime_library",
                    &editor_host_request,
                    format!("runtime library loading failed: {error}"),
                    "stage a compatible runtime library beside zircon_editor or configure ZIRCON_RUNTIME_LIBRARY with a path relative to the product executable or an absolute path",
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
            let product_failure_ledger = runtime_teardown_failure.failure_ledger();
            let host_result: Result<_, Box<dyn Error>> = (|| {
                let runtime_gateway = runtime_session
                    .editor_gateway(runtime_capabilities.clone())
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
                    startup_scene_uri,
                    startup_layout_preset,
                    editor_exit_after_first_frame_enabled(),
                    first_frame_capture_path,
                    project_runtime_build_set,
                )
                .with_editor_plugin_registrations(editor_plugin_registrations);
                let host_config = host_config.with_play_backend(play_backend);
                let host_config = match hub_handshake_config {
                    Some((project_root, session)) => {
                        host_config.with_hub_handshake(project_root, session)
                    }
                    None => host_config,
                };
                run_editor_with_config(core, runtime_gateway, host_config)
                    .map_err(|error| editor_host_startup_error(&editor_host_request, error))?;
                Ok(())
            })();
            record_editor_host_failure(&product_failure_ledger, &host_result);
            #[cfg(feature = "profiling")]
            if profile_capture.is_some() {
                match zircon_runtime::core::diagnostics::profiling::stop_and_export_capture_from_env(
                ) {
                    Some(Ok(report)) => eprintln!("profile report exported: {}", report.export_dir),
                    Some(Err(error)) => eprintln!("profile report export failed: {error}"),
                    None => {}
                }
            }
            drop(product_composition);
            drop(runtime_session);
            let failure_report = product_failure_ledger.snapshot();
            finish_editor_host(&editor_host_request, host_result, failure_report)?;
            Ok(0)
        }
    }
}

#[cfg(feature = "target-editor-host")]
fn editor_host_run_config_with_first_frame_exit(
    startup_request: Option<EditorGuiStartupRequest>,
    startup_scene_uri: Option<AssetUri>,
    startup_layout_preset: Option<String>,
    exit_after_first_frame: bool,
    first_presented_frame_capture_path: Option<ResolvedProjectPath>,
    project_runtime_build_set: Option<
        zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId,
    >,
) -> EditorHostRunConfig {
    let config = EditorHostRunConfig::new().with_startup_request(startup_request);
    let config = match project_runtime_build_set {
        Some(build_set_id) => config.with_project_runtime_build_set(build_set_id),
        None => config,
    };
    let config = match startup_scene_uri {
        Some(scene_uri) => config.with_startup_scene_uri(scene_uri),
        None => config,
    };
    let config = match startup_layout_preset {
        Some(preset) => config.with_startup_layout_preset(preset),
        None => config,
    };
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
fn editor_first_frame_capture_path() -> Result<Option<ResolvedProjectPath>, std::io::Error> {
    editor_first_frame_capture_path_from_value(env::var_os(EDITOR_CAPTURE_FIRST_FRAME_PNG_ENV))
}

#[cfg(feature = "target-editor-host")]
fn editor_first_frame_capture_path_from_value(
    value: Option<OsString>,
) -> Result<Option<ResolvedProjectPath>, std::io::Error> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_empty() || value.to_str().is_some_and(|value| value.trim().is_empty()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "editor startup diagnostic: component=editor_host requested={EDITOR_CAPTURE_FIRST_FRAME_PNG_ENV} cause=first-frame PNG capture path is empty or blank recovery=set {EDITOR_CAPTURE_FIRST_FRAME_PNG_ENV} to a writable PNG path or unset it"
            ),
        ));
    }
    let path = PathBuf::from(value);
    let display_path = ProjectPaths::display_path(&path);
    ProjectPaths::resolve_path(&path)
        .map(Some)
        .map_err(|error| {
            std::io::Error::new(
                error.kind(),
                format!(
                    "editor startup diagnostic: component=editor_host requested={EDITOR_CAPTURE_FIRST_FRAME_PNG_ENV}={} cause=could not resolve first-frame PNG capture path: {error} recovery=set {EDITOR_CAPTURE_FIRST_FRAME_PNG_ENV} to a writable PNG path or unset it",
                    display_path.display()
                ),
            )
        })
}

#[cfg(feature = "target-editor-host")]
struct EditorStartupPreparation {
    entry_config: ResolvedProductHostConfig,
    startup_request: Option<EditorGuiStartupRequest>,
    editor_plugin_registrations: Vec<EditorPluginRegistrationReport>,
    runtime_plugin_registrations: Vec<RuntimePluginRegistrationReport>,
    runtime_capabilities: RuntimeCapabilities,
}

#[cfg(feature = "target-editor-host")]
fn prepare_editor_gui_startup(
    startup_request: Option<EditorGuiStartupRequest>,
) -> Result<EditorStartupPreparation, Box<dyn Error>> {
    prepare_editor_startup(startup_request, true)
}

#[cfg(feature = "target-editor-host")]
fn prepare_editor_gui_startup_with_resolved_project(
    project_root: ResolvedProjectPath,
) -> Result<EditorStartupPreparation, Box<dyn Error>> {
    let intent = application_open_project_intent(project_root.operation_path())?;
    prepare_editor_gui_startup(Some(EditorGuiStartupRequest::project(intent)))
}

#[cfg(feature = "target-editor-host")]
fn prepare_editor_startup(
    startup_request: Option<EditorGuiStartupRequest>,
    include_editor_plugin_registrations: bool,
) -> Result<EditorStartupPreparation, Box<dyn Error>> {
    let startup_request = match startup_request {
        Some(EditorGuiStartupRequest::Project { intent }) => match intent.target() {
            ProjectLaunchTarget::CreateProject {
                project_name,
                location,
                template,
            } => {
                let template = match template {
                    ProjectTemplateId::RenderableEmpty => NewProjectTemplate::RenderableEmpty,
                };
                let draft = NewProjectDraft {
                    project_name: project_name.clone(),
                    location: location.to_string_lossy().into_owned(),
                    template,
                };
                let created = ProjectAuthority::default().create_project(&draft)?;
                let request = EditorGuiStartupRequest::project(
                    intent.retarget_open_existing_project(created.root.clone())?,
                );
                // App07 owns durable template creation. Its transient manager must not cross
                // this boundary: Editor admission later materializes the created project.
                drop(created);
                Some(request)
            }
            ProjectLaunchTarget::OpenExisting { .. } => {
                Some(EditorGuiStartupRequest::project(intent))
            }
        },
        request => request,
    };
    prepare_editor_startup_from_launch_intent(startup_request, include_editor_plugin_registrations)
}

#[cfg(feature = "target-editor-host")]
fn prepare_editor_startup_from_launch_intent(
    startup_request: Option<EditorGuiStartupRequest>,
    include_editor_plugin_registrations: bool,
) -> Result<EditorStartupPreparation, Box<dyn Error>> {
    let entry_config = EntryConfig::new(EntryProfile::Editor).resolve()?;
    let runtime_plugin_registrations =
        first_party_runtime_plugin_registrations_for_config(&entry_config);
    let runtime_capabilities = RuntimeCapabilities::from_runtime_plugin_registrations(
        SessionProfileKind::Editor,
        &runtime_plugin_registrations,
    );
    let editor_plugin_registrations = include_editor_plugin_registrations
        .then(|| first_party_editor_plugin_registrations_for_config(&entry_config))
        .unwrap_or_default();

    Ok(EditorStartupPreparation {
        entry_config,
        startup_request,
        editor_plugin_registrations,
        runtime_plugin_registrations,
        runtime_capabilities,
    })
}

#[cfg(feature = "target-editor-host")]
pub(super) fn application_open_project_intent(
    requested_path: impl Into<std::path::PathBuf>,
) -> Result<ProjectLaunchIntent, Box<dyn Error>> {
    let operation_id = APPLICATION_PROJECT_LAUNCH_OPERATION_IDS
        .get_or_init(|| ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new()))
        .allocate()
        .ok_or_else(|| std::io::Error::other("project launch operation sequence is exhausted"))?;
    ProjectLaunchIntent::open_existing(
        operation_id,
        ProjectLaunchSource::Application,
        ProjectLaunchProfile::Normal,
        requested_path,
    )
    .map_err(|error| Box::new(error) as Box<dyn Error>)
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
        Some(EditorGuiStartupRequest::Project { intent }) => match intent.target() {
            ProjectLaunchTarget::OpenExisting { requested_path } => format!(
                "project:{}",
                ProjectPaths::display_path(requested_path).display()
            ),
            ProjectLaunchTarget::CreateProject { .. } => "project:create".to_string(),
        },
        None => "workspace:welcome".to_string(),
    }
}

#[cfg(all(test, feature = "target-editor-host"))]
mod tests;
