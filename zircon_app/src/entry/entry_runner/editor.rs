#[cfg(feature = "target-editor-host")]
use ownership::{close_deadline, finish_owned_editor_host};
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
use zircon_runtime::builtin::RuntimePluginId;
#[cfg(feature = "target-editor-host")]
use zircon_runtime::core::framework::project::{
    ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection,
};
#[cfg(feature = "target-editor-host")]
use zircon_runtime::plugin::RuntimePluginRegistrationReport;
#[cfg(feature = "target-editor-host")]
use zircon_runtime_interface::project::{
    ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId, ProjectLaunchIntent,
    ProjectLaunchProfile, ProjectLaunchSource, ProjectLaunchTarget, ProjectTemplateId,
};
#[cfg(feature = "target-editor-host")]
use zircon_runtime_interface::runtime_build_set::ZrRuntimeModuleCompositionTargetV1;

#[cfg(feature = "target-editor-host")]
use crate::entry::{
    cli::{EditorLaunchArgs, EditorLaunchRoute},
    first_party_editor_plugin_registrations_for_config,
    first_party_runtime_plugin_registrations_for_config, EntryConfig, EntryProfile,
    ResolvedProductHostConfig,
};
#[cfg(feature = "target-editor-host")]
use zircon_editor::core::project::ProjectAuthority;

#[cfg(feature = "target-editor-host")]
use super::super::runtime_library::{LoadedRuntime, RuntimeSession};

#[cfg(feature = "target-editor-host")]
mod composition;
#[cfg(feature = "target-editor-host")]
mod ownership;
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
use crate::entry::product_shutdown::{ProductExitClass, ProductTerminalOutcome};
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
        Self::run_editor_with_args_terminal(args).map(|outcome| outcome.exit_code().code())
    }

    /// Preserve GUI/help versus commandlet origin even when both complete with code zero.
    pub fn run_editor_with_args_terminal<I, S>(
        args: I,
    ) -> Result<ProductTerminalOutcome, Box<dyn Error>>
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
                        return Ok(ProductTerminalOutcome::host(
                            ProductExitClass::Success,
                            "editor_help_completed",
                        ));
                    }
                    EditorLaunchRoute::Commandlet(request) => {
                        let commandlet_host =
                            project_automation::EditorProjectAutomationCommandletHost;
                        let report = run_commandlet_with_host(request, &commandlet_host);
                        println!("{}", serde_json::to_string(&report)?);
                        return Ok(ProductTerminalOutcome::commandlet(
                            report.exit_code().as_u8(),
                        ));
                    }
                    EditorLaunchRoute::CommandletRejected(report) => {
                        println!("{}", serde_json::to_string(&report)?);
                        return Ok(ProductTerminalOutcome::commandlet(
                            report.exit_code().as_u8(),
                        ));
                    }
                    EditorLaunchRoute::Gui(intent) => intent.into_parts(),
                };
            configure_windows_editor_gpu_backend();
            let first_frame_capture_path = editor_first_frame_capture_path()?;
            let requested_startup = editor_host_startup_request(gui_startup_request.as_ref());
            let runtime_preflight = LoadedRuntime::preflight_default().map_err(|error| {
                editor_startup_diagnostic_error(
                    "runtime_build_set",
                    &requested_startup,
                    format!("runtime BuildSet preflight failed: {error}"),
                    "stage a runtime library and sidecar manifest from the same BuildSet as zircon_editor before opening or creating a project",
                )
            })?;
            let project_runtime_build_set = Some(runtime_preflight.build_set_id());
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
                    let requested_project_root = hub_handshake_project_root(intent);
                    let project_root = ProjectPaths::resolve_path(&requested_project_root)
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
            let mut product_composition = Self::compose_resolved_with_runtime_plugin_registrations(
                entry_config,
                runtime_plugin_registrations.iter().cloned(),
            )
            .map_err(|error| {
                editor_startup_diagnostic_error(
                    "editor_bootstrap",
                    &editor_host_request,
                    format!("application bootstrap failed: {error}"),
                    "verify the selected profile and staged editor and runtime plugins",
                )
            })?;
            product_composition.retain_plugin_selection_outcomes(
                runtime_plugin_registrations
                    .outcomes()
                    .iter()
                    .chain(editor_plugin_registrations.outcomes().iter())
                    .cloned(),
            );
            let core = product_composition.core().clone();
            let editor_manager = match core.resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME) {
                Ok(manager) => manager,
                Err(primary) => {
                    drop(core);
                    let failure = product_composition.fail_until(primary, close_deadline());
                    return Err(Box::new(editor_startup_diagnostic_error(
                        "editor_manager", &editor_host_request,
                        format!("editor manager resolution failed: {failure}"),
                        "verify the editor manager registration and selected startup profile; retry retained product cleanup before another admission",
                    )));
                }
            };
            drop(editor_manager);
            let factory = std::sync::Arc::new(AppPlaySessionFactory::new(
                runtime_preflight.clone(),
                runtime_capabilities.clone(),
            ));
            let play_backend =
                std::sync::Arc::new(EmbeddedPlayBackend::new(factory)) as SharedPlayBackend;
            let runtime = match runtime_preflight.load_after_preflight() {
                Ok(runtime) => runtime,
                Err(primary) => {
                    drop(core);
                    let failure = product_composition.fail_until(primary, close_deadline());
                    return Err(Box::new(editor_startup_diagnostic_error(
                        "runtime_library", &editor_host_request,
                        format!("runtime library loading failed: {failure}"),
                        "stage a compatible runtime library beside zircon_editor; retry retained product cleanup before another admission",
                    )));
                }
            };
            // Editor owns project activation; the gateway session remains projectless.
            let runtime_session = match RuntimeSession::create_with_profile(runtime, b"editor") {
                Ok(session) => std::sync::Arc::new(session),
                Err(error) => {
                    drop(core);
                    let diagnostic = error.diagnostic_with_recovery();
                    let failure =
                        product_composition.fail_with_runtime_until(error, close_deadline());
                    return Err(Box::new(editor_startup_diagnostic_error(
                        "runtime_session", &editor_host_request,
                        format!("runtime session creation failed: {diagnostic}; {failure}"),
                        "verify the runtime ABI and staged dependencies; retry_product_cleanup_until retries the exact retained packets on this host thread",
                    )));
                }
            };
            let runtime_teardown_failure = runtime_session.teardown_failure_state();
            let product_failure_ledger = runtime_teardown_failure.failure_ledger();
            let retained_play_backend = play_backend.clone();
            let host_result: Result<_, Box<dyn Error + Send + Sync>> = (|| {
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
            if let Err(error) = &host_result {
                eprintln!("[zircon_editor] editor host failed before runtime teardown: {error}");
            }
            #[cfg(feature = "profiling")]
            if profile_capture.is_some() {
                match zircon_runtime::core::diagnostics::profiling::stop_and_export_capture_from_env(
                ) {
                    Some(Ok(report)) => eprintln!("profile report exported: {}", report.export_dir),
                    Some(Err(error)) => eprintln!("profile report export failed: {error}"),
                    None => {}
                }
            }
            finish_owned_editor_host(
                &editor_host_request,
                host_result,
                product_composition,
                runtime_session,
                retained_play_backend,
                &product_failure_ledger,
                close_deadline(),
            )?;
            Ok(ProductTerminalOutcome::host(
                ProductExitClass::Success,
                "editor_completed",
            ))
        }
    }
}

#[cfg(all(feature = "target-editor-host", windows))]
fn configure_windows_editor_gpu_backend() {
    // The editor and its loaded runtime each create a WGPU instance. Restrict the
    // default to one backend so their shared presenter cannot select different APIs.
    // An explicit user selection remains authoritative.
    if env::var_os("WGPU_BACKEND").is_none() {
        env::set_var("WGPU_BACKEND", "vulkan");
        eprintln!("[zircon_editor] gpu_backend_default=vulkan");
    }
}

#[cfg(all(feature = "target-editor-host", not(windows)))]
fn configure_windows_editor_gpu_backend() {}

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
    editor_plugin_registrations:
        zircon_runtime::core::framework::project::PluginSelectionResolutionReport<
            EditorPluginRegistrationReport,
        >,
    runtime_plugin_registrations:
        zircon_runtime::core::framework::project::PluginSelectionResolutionReport<
            RuntimePluginRegistrationReport,
        >,
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
    prepare_editor_startup_from_launch_intent(startup_request, include_editor_plugin_registrations)
}

#[cfg(feature = "target-editor-host")]
fn prepare_editor_startup_from_launch_intent(
    startup_request: Option<EditorGuiStartupRequest>,
    include_editor_plugin_registrations: bool,
) -> Result<EditorStartupPreparation, Box<dyn Error>> {
    let startup_request = startup_request
        .map(|request| request.preflight_project(&ProjectAuthority::default()))
        .transpose()?;
    let mvp_runtime_plugins = editor_mvp_required_runtime_plugins()?;
    let entry_config =
        EntryConfig::new(EntryProfile::Editor).with_required_runtime_plugins(&mvp_runtime_plugins);
    let mut project_plugins = startup_request
        .as_ref()
        .and_then(EditorGuiStartupRequest::project_preflight)
        .map(|preflight| preflight.approved_project_plugins().clone())
        .unwrap_or_default();
    admit_builtin_editor_ui_selection(&mut project_plugins);
    disable_unlinked_editor_render_features(&mut project_plugins);
    let entry_config = entry_config
        .with_project_plugins(project_plugins)
        .resolve()?;
    let runtime_plugin_registrations =
        first_party_runtime_plugin_registrations_for_config(&entry_config)
            .into_registrations_if_required_resolved_where(|selection| {
                !selection.is_runtime_builtin_domain()
            })
            .map_err(|error| Box::new(error) as Box<dyn Error>)?;
    let runtime_capabilities = RuntimeCapabilities::from_runtime_plugin_registrations(
        SessionProfileKind::Editor,
        &runtime_plugin_registrations,
    );
    let editor_plugin_registrations = include_editor_plugin_registrations
        .then(|| {
            first_party_editor_plugin_registrations_for_config(&entry_config)
                .into_registrations_if_required_resolved_where(|selection| {
                    selection.editor_crate.is_some()
                })
        })
        .transpose()
        .map_err(|error| Box::new(error) as Box<dyn Error>)?
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
fn admit_builtin_editor_ui_selection(manifest: &mut ProjectPluginManifest) {
    if let Some(selection) = manifest
        .selections
        .iter_mut()
        .find(|selection| selection.id == RuntimePluginId::Ui.key())
    {
        if selection.is_runtime_builtin_domain() {
            selection.required = false;
        }
    } else {
        // UI is a built-in runtime module; it has no first-party plugin report.
        manifest
            .selections
            .push(ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::Ui,
                true,
                false,
            ));
    }
}

#[cfg(feature = "target-editor-host")]
fn disable_unlinked_editor_render_features(manifest: &mut ProjectPluginManifest) {
    let rendering_index = match manifest
        .selections
        .iter()
        .position(|selection| selection.id == RuntimePluginId::Rendering.key())
    {
        Some(index) => index,
        None => {
            manifest
                .selections
                .push(ProjectPluginSelection::runtime_plugin(
                    RuntimePluginId::Rendering,
                    true,
                    true,
                ));
            manifest.selections.len() - 1
        }
    };
    let rendering = &mut manifest.selections[rendering_index];
    // The editor bundle links the base renderer, but these optional providers are
    // not linked into the MVP. An explicit project selection remains authoritative.
    for feature_id in [
        "rendering.post_process",
        "rendering.reflection_probes",
        "rendering.baked_lighting",
    ] {
        if !rendering
            .features
            .iter()
            .any(|feature| feature.id == feature_id)
        {
            rendering
                .features
                .push(ProjectPluginFeatureSelection::new(feature_id).enabled(false));
        }
    }
}

#[cfg(feature = "target-editor-host")]
fn editor_mvp_required_runtime_plugins() -> Result<Vec<RuntimePluginId>, Box<dyn Error>> {
    let descriptor = ProjectTemplateId::RenderableEmpty.descriptor();
    let requirement = descriptor
        .target_requirements()
        .iter()
        .find(|requirement| requirement.target() == ZrRuntimeModuleCompositionTargetV1::EditorHost)
        .ok_or_else(|| {
            std::io::Error::other(
                "renderable-empty template does not define an EditorHost composition requirement",
            )
        })?;
    requirement
        .required_runtime_providers()
        .map(|provider| {
            RuntimePluginId::parse_key(provider).ok_or_else(|| {
                std::io::Error::other(format!(
                    "renderable-empty template requires unknown runtime provider {provider}"
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| Box::new(error) as Box<dyn Error>)
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
        Some(EditorGuiStartupRequest::Project { intent, .. }) => match intent.target() {
            ProjectLaunchTarget::OpenExisting { requested_path } => format!(
                "project:{}",
                ProjectPaths::display_path(requested_path).display()
            ),
            ProjectLaunchTarget::CreateProject { .. } => "project:create".to_string(),
        },
        None => "workspace:welcome".to_string(),
    }
}

#[cfg(feature = "target-editor-host")]
fn hub_handshake_project_root(intent: &ProjectLaunchIntent) -> PathBuf {
    match intent.target() {
        ProjectLaunchTarget::OpenExisting { requested_path } => requested_path.clone(),
        ProjectLaunchTarget::CreateProject {
            project_name,
            location,
            ..
        } => location.join(project_name),
    }
}

#[cfg(all(test, feature = "target-editor-host"))]
mod tests;
