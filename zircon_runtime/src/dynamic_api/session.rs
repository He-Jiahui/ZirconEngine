use std::collections::HashMap;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::{Mutex, OnceLock};

use zircon_runtime_interface::{
    ui::accessibility::UiAccessibilityTreeSnapshot, ProfileControlRequest, ZrByteSlice,
    ZrOwnedByteBuffer, ZrRuntimeAccessibilityTreeRequestV1, ZrRuntimeBindViewportSurfaceRequestV1,
    ZrRuntimeEventV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1, ZrRuntimeHostRequestBatchV1,
    ZrRuntimeHostRequestV1, ZrRuntimeSessionConfigV1, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrStatus, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use crate::core::diagnostics::collect_runtime_diagnostics;
use crate::core::framework::input::InputManager;
use crate::core::framework::render::RenderViewportSurfaceDescriptor;
use crate::core::math::{UVec2, Vec2};
use crate::core::CoreRuntime;
use crate::diagnostic_log::{
    write_diagnostic_store_snapshot, write_log, DiagnosticStoreLogSchedule,
    DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT,
};
use crate::plugin::RuntimeExtensionRegistry;
use crate::scene::components::NodeKind;
use crate::scene::LevelSystem;
use crate::{builtin::runtime_modules_for_target, RuntimeTargetMode};

use super::camera_controller::RuntimeCameraController;
use super::frame::{
    encode_accessibility_tree, encode_frame, encode_host_request_batch, encode_profile_response,
    write_accessibility_tree, write_frame, write_host_requests, write_profile_response,
};
use super::runtime_loop::{resolve_input, RuntimeRenderBridge};
use super::surface::render_surface_descriptor;

mod events;
mod extract;
mod extract_stats;
mod host_requests;
mod hud;
mod input_events;
mod menu;
mod preview;
mod project;
mod status;
#[cfg(test)]
mod tests;

pub(super) use host_requests::{runtime_gamepad_rumble_request, runtime_ime_host_request};
use preview::{dynamic_preview_accessibility_snapshot, empty_captured_frame};
use project::RuntimeProjectConfig;
use status::{error_status, invalid_argument, not_found, unsupported_version};

const DEFAULT_VIEWPORT: ZrRuntimeViewportHandle = ZrRuntimeViewportHandle::new(1);
const DEFAULT_DYNAMIC_RUNTIME_MAX_FIXED_STEPS_PER_FRAME: u32 = 8;
const DYNAMIC_RUNTIME_DIAGNOSTIC_LOG_SCOPE: &str = "runtime_diagnostics";
const RUNTIME_SESSION_PROFILE_RUNTIME: &[u8] = b"runtime";
const RUNTIME_SESSION_PROFILE_EDITOR: &[u8] = b"editor";
const RUNTIME_SESSION_PROFILE_DEV: &[u8] = b"dev";
const RUNTIME_SESSION_PROFILE_MINIMAL: &[u8] = b"minimal";
const RUNTIME_SESSION_PROFILE_HEADLESS: &[u8] = b"headless";

static SESSION_REGISTRY: OnceLock<Mutex<SessionRegistry>> = OnceLock::new();

struct SessionRegistry {
    next_handle: AtomicU64,
    sessions: HashMap<u64, Arc<Mutex<RuntimeDynamicSession>>>,
}

impl Default for SessionRegistry {
    fn default() -> Self {
        Self {
            next_handle: AtomicU64::new(1),
            sessions: HashMap::new(),
        }
    }
}

pub(super) unsafe fn create_session(
    config: ZrRuntimeSessionConfigV1,
    out_session: *mut ZrRuntimeSessionHandle,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "create_session");
    crate::diagnostic_log::initialize_unity_process_log("runtime-dynamic");
    write_log("runtime_session", "dynamic_api_create_session_entered");
    if out_session.is_null() {
        return invalid_argument(b"missing runtime session output");
    }
    if config.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return unsupported_version();
    }

    let profile =
        match RuntimeDynamicSessionProfile::from_bytes(unsafe { config.profile.as_slice() }) {
            Some(profile) => profile,
            None => return invalid_argument(b"unknown runtime session profile"),
        };
    let project_config = match RuntimeProjectConfig::from_abi_slice(config.project_manifest) {
        Ok(project_config) => project_config,
        Err(_) => return invalid_argument(b"invalid runtime project root"),
    };

    match RuntimeDynamicSession::new(profile, project_config) {
        Ok(session) => {
            let handle = insert_session(session);
            ptr::write(out_session, handle);
            ZrStatus::ok()
        }
        Err(error) => error_status(error),
    }
}

pub(super) unsafe fn destroy_session(handle: ZrRuntimeSessionHandle) -> ZrStatus {
    if !handle.is_valid() {
        return invalid_argument(b"invalid runtime session handle");
    }
    let mut registry = registry().lock().unwrap();
    if registry.sessions.remove(&handle.raw()).is_none() {
        return not_found(b"runtime session not found");
    }
    ZrStatus::ok()
}

pub(super) unsafe fn handle_event(
    handle: ZrRuntimeSessionHandle,
    event: ZrRuntimeEventV1,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "handle_event");
    if event.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return unsupported_version();
    }
    with_session(handle, |session| session.handle_event(event))
}

pub(super) unsafe fn capture_frame(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeFrameRequestV1,
    out_frame: *mut ZrRuntimeFrameV1,
) -> ZrStatus {
    crate::profile_frame!("runtime", "capture_frame");
    crate::profile_scope!("runtime", "dynamic_api", "capture_frame");
    if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return unsupported_version();
    }
    if request.viewport != DEFAULT_VIEWPORT {
        return not_found(b"runtime viewport not found");
    }
    with_session(handle, |session| match session.capture_frame(request) {
        Ok(frame) => write_frame(out_frame, frame),
        Err(error) => error_status(error),
    })
}

pub(super) unsafe fn capture_accessibility_tree(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeAccessibilityTreeRequestV1,
    out_tree: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "capture_accessibility_tree");
    if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return unsupported_version();
    }
    if request.viewport != DEFAULT_VIEWPORT {
        return not_found(b"runtime viewport not found");
    }
    if out_tree.is_null() {
        return write_accessibility_tree(out_tree, ZrOwnedByteBuffer::empty());
    }
    with_session(handle, |session| {
        match session
            .capture_accessibility_tree(request)
            .and_then(|snapshot| {
                encode_accessibility_tree(&snapshot).map_err(|error| error.to_string())
            }) {
            Ok(buffer) => write_accessibility_tree(out_tree, buffer),
            Err(error) => error_status(error),
        }
    })
}

pub(super) unsafe fn bind_viewport_surface(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeBindViewportSurfaceRequestV1,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "bind_viewport_surface");
    if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return unsupported_version();
    }
    if request.target.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return unsupported_version();
    }
    if request.viewport != DEFAULT_VIEWPORT {
        return not_found(b"runtime viewport not found");
    }
    let descriptor = match render_surface_descriptor(request) {
        Ok(descriptor) => descriptor,
        Err(status) => return status,
    };
    with_session(handle, |session| {
        match session.bind_viewport_surface(descriptor) {
            Ok(()) => ZrStatus::ok(),
            Err(error) => error_status(error),
        }
    })
}

pub(super) unsafe fn unbind_viewport_surface(
    handle: ZrRuntimeSessionHandle,
    viewport: ZrRuntimeViewportHandle,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "unbind_viewport_surface");
    if viewport != DEFAULT_VIEWPORT {
        return not_found(b"runtime viewport not found");
    }
    with_session(handle, |session| match session.unbind_viewport_surface() {
        Ok(()) => ZrStatus::ok(),
        Err(error) => error_status(error),
    })
}

pub(super) unsafe fn present_viewport(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeFrameRequestV1,
) -> ZrStatus {
    crate::profile_frame!("runtime", "present_viewport");
    crate::profile_scope!("runtime", "dynamic_api", "present_viewport");
    if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return unsupported_version();
    }
    if request.viewport != DEFAULT_VIEWPORT {
        return not_found(b"runtime viewport not found");
    }
    with_session(handle, |session| match session.present_viewport(request) {
        Ok(()) => ZrStatus::ok(),
        Err(error) => error_status(error),
    })
}

pub(super) unsafe fn profile_control(
    handle: ZrRuntimeSessionHandle,
    request_json: ZrByteSlice,
    out_json: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    if out_json.is_null() {
        return write_profile_response(out_json, ZrOwnedByteBuffer::empty());
    }
    if request_json.is_empty() {
        return invalid_argument(b"missing profile control request");
    }
    let request =
        match serde_json::from_slice::<ProfileControlRequest>(unsafe { request_json.as_slice() }) {
            Ok(request) => request,
            Err(_) => return invalid_argument(b"invalid profile control request"),
        };
    with_session(handle, |_session| {
        match encode_profile_response(&crate::core::diagnostics::profiling::control(request)) {
            Ok(buffer) => write_profile_response(out_json, buffer),
            Err(error) => error_status(error.to_string()),
        }
    })
}

pub(super) unsafe fn tick_frame(handle: ZrRuntimeSessionHandle) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "tick_frame");
    with_session(handle, |session| match session.tick_frame() {
        Ok(()) => ZrStatus::ok(),
        Err(error) => error_status(error),
    })
}

pub(super) unsafe fn drain_host_requests(
    handle: ZrRuntimeSessionHandle,
    out_requests: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "drain_host_requests");
    if out_requests.is_null() {
        return write_host_requests(out_requests, ZrOwnedByteBuffer::empty());
    }
    with_session(handle, |session| {
        let batch = session.drain_host_requests();
        match encode_host_request_batch(&batch) {
            Ok(buffer) => write_host_requests(out_requests, buffer),
            Err(error) => error_status(error.to_string()),
        }
    })
}

struct RuntimeDynamicSession {
    runtime: CoreRuntime,
    profile: RuntimeDynamicSessionProfile,
    diagnostic_log_schedule: DiagnosticStoreLogSchedule,
    render_bridge: Option<RuntimeRenderBridge>,
    level: LevelSystem,
    selected_node: Option<u64>,
    camera_controller: RuntimeCameraController,
    cursor: Vec2,
    input_manager: Arc<dyn InputManager>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuntimeDynamicSessionProfile {
    Runtime,
    Editor,
    Dev,
    Minimal,
    Headless,
}

impl RuntimeDynamicSessionProfile {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            [] | RUNTIME_SESSION_PROFILE_RUNTIME => Some(Self::Runtime),
            RUNTIME_SESSION_PROFILE_EDITOR => Some(Self::Editor),
            RUNTIME_SESSION_PROFILE_DEV => Some(Self::Dev),
            RUNTIME_SESSION_PROFILE_MINIMAL => Some(Self::Minimal),
            RUNTIME_SESSION_PROFILE_HEADLESS => Some(Self::Headless),
            _ => None,
        }
    }

    fn max_fixed_steps_per_frame(self) -> u32 {
        DEFAULT_DYNAMIC_RUNTIME_MAX_FIXED_STEPS_PER_FRAME
    }

    fn diagnostic_log_schedule(self) -> DiagnosticStoreLogSchedule {
        match self {
            Self::Dev => DiagnosticStoreLogSchedule::repeating(DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT),
            Self::Runtime | Self::Editor | Self::Minimal | Self::Headless => {
                DiagnosticStoreLogSchedule::disabled()
            }
        }
    }

    fn uses_render_bridge(self) -> bool {
        matches!(self, Self::Runtime | Self::Editor | Self::Dev)
    }
}

impl RuntimeDynamicSession {
    fn new(
        profile: RuntimeDynamicSessionProfile,
        project_config: Option<RuntimeProjectConfig>,
    ) -> Result<Self, String> {
        crate::profile_scope!("runtime", "dynamic_api", "runtime_dynamic_session_new");
        write_log(
            "runtime_session",
            format!(
                "runtime_dynamic_session_create_start profile={profile:?} project={}",
                project_config
                    .as_ref()
                    .map(RuntimeProjectConfig::root_display)
                    .unwrap_or_else(|| "none".to_string())
            ),
        );
        let runtime = {
            crate::profile_scope!("runtime", "dynamic_api", "runtime_session_core_new");
            CoreRuntime::new()
        };
        write_log("runtime_session", "runtime_dynamic_session_core_created");
        let core = runtime.handle();
        let mut modules = {
            crate::profile_scope!(
                "runtime",
                "dynamic_api",
                "runtime_session_modules_for_target"
            );
            runtime_modules_for_target(RuntimeTargetMode::ClientRuntime, None)
        };
        modules
            .modules
            .push(Arc::new(crate::navigation::BuiltinNavigationModule));
        modules
            .modules
            .push(Arc::new(crate::animation::AnimationModule));
        if !modules.errors.is_empty() {
            return Err(modules.errors.join("; "));
        }
        write_log(
            "runtime_session",
            format!(
                "runtime_dynamic_session_modules_discovered count={}",
                modules.modules.len()
            ),
        );
        {
            crate::profile_scope!("runtime", "dynamic_api", "runtime_session_register_modules");
            for module in &modules.modules {
                runtime
                    .register_module(module.descriptor())
                    .map_err(|error| runtime_session_error("register runtime module", error))?;
            }
        }
        write_log(
            "runtime_session",
            "runtime_dynamic_session_modules_registered",
        );
        {
            crate::profile_scope!("runtime", "dynamic_api", "runtime_session_activate_modules");
            for module in &modules.modules {
                runtime
                    .activate_module(module.module_name())
                    .map_err(|error| runtime_session_error("activate runtime module", error))?;
            }
        }
        write_log(
            "runtime_session",
            "runtime_dynamic_session_modules_activated",
        );
        {
            crate::profile_scope!(
                "runtime",
                "dynamic_api",
                "runtime_session_install_scene_hooks"
            );
            install_builtin_scene_runtime_hooks(&runtime)
                .map_err(|error| runtime_session_error("install scene runtime hooks", error))?;
        }
        write_log(
            "runtime_session",
            "runtime_dynamic_session_scene_hooks_installed",
        );

        let input_manager = {
            crate::profile_scope!("runtime", "dynamic_api", "runtime_session_resolve_input");
            resolve_input(&core).map_err(|error| runtime_session_error("resolve input", error))?
        };
        write_log("runtime_session", "runtime_dynamic_session_input_ready");
        let render_bridge = if profile.uses_render_bridge() {
            crate::profile_scope!("runtime", "dynamic_api", "runtime_session_render_bridge");
            let render_bridge = RuntimeRenderBridge::new(&core)
                .map_err(|error| runtime_session_error("create render bridge", error))?;
            write_log(
                "runtime_session",
                "runtime_dynamic_session_render_bridge_ready",
            );
            Some(render_bridge)
        } else {
            write_log(
                "runtime_session",
                "runtime_dynamic_session_render_bridge_skipped",
            );
            None
        };
        let level = {
            crate::profile_scope!("runtime", "dynamic_api", "runtime_session_level");
            match &project_config {
                Some(project_config) => {
                    write_log("runtime_session", "runtime_project_open_assets_start");
                    project_config
                        .open_project_assets(&core)
                        .map_err(|error| runtime_session_error("open project assets", error))?;
                    write_log("runtime_session", "runtime_project_open_assets_done");
                    write_log("runtime_session", "runtime_project_navigation_load_start");
                    project_config
                        .load_default_navigation(&core)
                        .map_err(|error| {
                            runtime_session_error("load default project navigation", error)
                        })?;
                    write_log("runtime_session", "runtime_project_navigation_load_done");
                    write_log("runtime_session", "runtime_project_scripts_load_start");
                    project_config
                        .load_startup_scripts(&core)
                        .map_err(|error| {
                            runtime_session_error("load startup script packages", error)
                        })?;
                    write_log("runtime_session", "runtime_project_scripts_load_done");
                    write_log("runtime_session", "runtime_project_level_load_start");
                    project_config
                        .load_default_level(&core)
                        .map_err(|error| runtime_session_error("load default level", error))?
                }
                None => crate::scene::create_default_level(&core)
                    .map_err(|error| runtime_session_error("create default level", error))?,
            }
        };
        write_log("runtime_session", "runtime_dynamic_session_level_ready");
        let (selected_node, orbit_target) = {
            crate::profile_scope!(
                "runtime",
                "dynamic_api",
                "runtime_session_select_orbit_target"
            );
            level.with_world(|world| {
                let cube = world
                    .nodes()
                    .iter()
                    .find(|node| matches!(&node.kind, NodeKind::Cube))
                    .map(|node| node.id)
                    .unwrap_or(world.active_camera());
                let orbit_target = world
                    .find_node(cube)
                    .map(|node| node.transform.translation)
                    .unwrap_or_default();
                (Some(cube), orbit_target)
            })
        };
        let mut camera_controller = {
            crate::profile_scope!(
                "runtime",
                "dynamic_api",
                "runtime_session_camera_controller"
            );
            RuntimeCameraController::new(UVec2::new(1280, 720))
        };
        camera_controller.set_orbit_target(orbit_target);
        write_log("runtime_session", "runtime_dynamic_session_create_done");

        Ok(Self {
            runtime,
            profile,
            diagnostic_log_schedule: profile.diagnostic_log_schedule(),
            render_bridge,
            level,
            selected_node,
            camera_controller,
            cursor: Vec2::ZERO,
            input_manager,
        })
    }

    fn tick_frame(&mut self) -> Result<(), String> {
        let advance = {
            crate::profile_scope!("runtime", "frame", "runtime_frame_time_update");
            self.runtime
                .tick_time(self.profile.max_fixed_steps_per_frame())
        };
        {
            crate::profile_scope!("runtime", "frame", "runtime_frame_update");
            self.level
                .tick(&self.runtime.handle(), advance)
                .map_err(|error| error.to_string())?;
        }
        self.input_manager.begin_frame();
        if self.diagnostic_log_schedule.tick(advance.real_delta()) {
            let snapshot = collect_runtime_diagnostics(&self.runtime.handle()).store;
            write_diagnostic_store_snapshot(DYNAMIC_RUNTIME_DIAGNOSTIC_LOG_SCOPE, &snapshot);
        }
        Ok(())
    }

    fn drain_host_requests(&mut self) -> ZrRuntimeHostRequestBatchV1 {
        let requests = self
            .input_manager
            .drain_ime_host_requests()
            .into_iter()
            .map(runtime_ime_host_request)
            .map(ZrRuntimeHostRequestV1::ime)
            .chain(
                self.input_manager
                    .drain_gamepad_rumble_requests()
                    .into_iter()
                    .map(runtime_gamepad_rumble_request)
                    .map(ZrRuntimeHostRequestV1::gamepad_rumble),
            )
            .collect();
        ZrRuntimeHostRequestBatchV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, requests)
    }

    fn capture_frame(
        &mut self,
        request: ZrRuntimeFrameRequestV1,
    ) -> Result<ZrRuntimeFrameV1, String> {
        let requested = UVec2::new(request.size.width.max(1), request.size.height.max(1));
        self.resize_viewport(requested);
        let extract = self.current_extract();
        let ui = self.current_ui_extract();
        let frame = if let Some(render_bridge) = &mut self.render_bridge {
            render_bridge
                .submit_extract_with_ui(extract, self.camera_controller.viewport_size(), ui)
                .map_err(|error| error.to_string())?
                .unwrap_or_else(|| empty_captured_frame(requested))
        } else {
            empty_captured_frame(requested)
        };
        Ok(encode_frame(frame))
    }

    fn bind_viewport_surface(
        &mut self,
        descriptor: RenderViewportSurfaceDescriptor,
    ) -> Result<(), String> {
        self.resize_viewport(descriptor.size);
        let Some(render_bridge) = &mut self.render_bridge else {
            return Ok(());
        };
        render_bridge
            .bind_surface(descriptor)
            .map_err(|error| error.to_string())
    }

    fn unbind_viewport_surface(&mut self) -> Result<(), String> {
        let Some(render_bridge) = &mut self.render_bridge else {
            return Ok(());
        };
        render_bridge
            .unbind_surface()
            .map_err(|error| error.to_string())
    }

    fn present_viewport(&mut self, request: ZrRuntimeFrameRequestV1) -> Result<(), String> {
        let requested = UVec2::new(request.size.width.max(1), request.size.height.max(1));
        self.resize_viewport(requested);
        let extract = self.current_extract();
        let ui = self.current_ui_extract();
        let Some(render_bridge) = &mut self.render_bridge else {
            return Ok(());
        };
        render_bridge
            .present_extract_with_ui(extract, self.camera_controller.viewport_size(), ui)
            .map_err(|error| error.to_string())
    }

    fn capture_accessibility_tree(
        &mut self,
        request: ZrRuntimeAccessibilityTreeRequestV1,
    ) -> Result<UiAccessibilityTreeSnapshot, String> {
        self.resize_viewport(UVec2::new(
            request.size.width.max(1),
            request.size.height.max(1),
        ));
        Ok(dynamic_preview_accessibility_snapshot())
    }
}

fn registry() -> &'static Mutex<SessionRegistry> {
    SESSION_REGISTRY.get_or_init(|| Mutex::new(SessionRegistry::default()))
}

fn insert_session(session: RuntimeDynamicSession) -> ZrRuntimeSessionHandle {
    let mut registry = registry().lock().unwrap();
    let handle = registry.next_handle.fetch_add(1, Ordering::SeqCst);
    registry
        .sessions
        .insert(handle, Arc::new(Mutex::new(session)));
    ZrRuntimeSessionHandle::new(handle)
}

fn install_builtin_scene_runtime_hooks(runtime: &CoreRuntime) -> Result<(), String> {
    let mut extensions = RuntimeExtensionRegistry::default();
    register_missing_scene_hook(
        runtime,
        &mut extensions,
        crate::animation::scene_hook_registration(),
    )?;
    register_missing_scene_hook(
        runtime,
        &mut extensions,
        crate::script::script_scene_fixed_update_hook_registration(),
    )?;
    register_missing_scene_hook(
        runtime,
        &mut extensions,
        crate::script::script_scene_update_hook_registration(),
    )?;
    runtime
        .install_scene_runtime_hooks(&extensions)
        .map_err(|error| error.to_string())
}

fn register_missing_scene_hook(
    runtime: &CoreRuntime,
    extensions: &mut RuntimeExtensionRegistry,
    registration: crate::plugin::SceneRuntimeHookRegistration,
) -> Result<(), String> {
    let descriptor = registration.descriptor();
    let already_installed = runtime
        .handle()
        .scene_runtime_hooks_for_stage(descriptor.stage)
        .iter()
        .any(|hook| hook.descriptor().id == descriptor.id);
    if already_installed {
        return Ok(());
    }
    extensions
        .register_scene_hook(registration)
        .map_err(|error| error.to_string())
}

fn runtime_session_error(step: &'static str, error: impl ToString) -> String {
    let error = error.to_string();
    let error = error.trim();
    if error.is_empty() {
        return format!("{step} failed without additional diagnostics");
    }
    format!("{step}: {error}")
}

fn with_session(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession) -> ZrStatus,
) -> ZrStatus {
    if !handle.is_valid() {
        return invalid_argument(b"invalid runtime session handle");
    }
    let session = {
        let registry = registry().lock().unwrap();
        registry.sessions.get(&handle.raw()).cloned()
    };
    let Some(session) = session else {
        return not_found(b"runtime session not found");
    };
    let mut session = session.lock().unwrap();
    action(&mut session)
}
