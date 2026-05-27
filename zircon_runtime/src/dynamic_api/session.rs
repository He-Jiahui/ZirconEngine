use std::collections::HashMap;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use zircon_runtime_interface::{
    ui::{
        accessibility::{
            UiA11yRole, UiAccessibilityActionRequest, UiAccessibilityDiagnostic,
            UiAccessibilityDiagnosticCode, UiAccessibilityDiagnosticSeverity, UiAccessibilityNode,
            UiAccessibilityTreeSnapshot,
        },
        event_ui::{UiNodeId, UiTreeId},
    },
    ProfileControlRequest, ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeAccessibilityTreeRequestV1,
    ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1, ZrRuntimeFrameRequestV1,
    ZrRuntimeFrameV1, ZrRuntimeGamepadRumbleRequestV1, ZrRuntimeHostRequestBatchV1,
    ZrRuntimeHostRequestV1, ZrRuntimeImeCursorAreaV1, ZrRuntimeImeHostRequestV1,
    ZrRuntimeImeSurroundingTextV1, ZrRuntimeSessionConfigV1, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
    ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1, ZR_RUNTIME_EVENT_KIND_CURSOR_ENTERED_V1,
    ZR_RUNTIME_EVENT_KIND_CURSOR_LEFT_V1, ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1,
    ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V1, ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V1,
    ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1, ZR_RUNTIME_EVENT_KIND_IME_V1,
    ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1, ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1,
    ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1, ZR_RUNTIME_EVENT_KIND_MOUSE_MOTION_V1,
    ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1, ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1,
    ZR_RUNTIME_EVENT_KIND_TOUCH_V1, ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1,
    ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1, ZR_RUNTIME_FILE_DRAG_CANCELLED_V1,
    ZR_RUNTIME_FILE_DRAG_DROPPED_V1, ZR_RUNTIME_FILE_DRAG_HOVERED_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_DPAD_X_V1, ZR_RUNTIME_GAMEPAD_AXIS_DPAD_Y_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1, ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_LEFT_Z_V1, ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_X_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_Y_V1, ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_Z_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_UNKNOWN_V1, ZR_RUNTIME_GAMEPAD_BUTTON_C_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1, ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_THUMB_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER2_V1, ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_MODE_V1, ZR_RUNTIME_GAMEPAD_BUTTON_NORTH_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_THUMB_V1, ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER2_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER_V1, ZR_RUNTIME_GAMEPAD_BUTTON_SELECT_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1, ZR_RUNTIME_GAMEPAD_BUTTON_START_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_UNKNOWN_V1, ZR_RUNTIME_GAMEPAD_BUTTON_WEST_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_Z_V1, ZR_RUNTIME_GAMEPAD_CONNECTION_CONNECTED_V1,
    ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1, ZR_RUNTIME_IME_CURSOR_HIDDEN_V1,
    ZR_RUNTIME_IME_STATE_COMMIT_V1, ZR_RUNTIME_IME_STATE_CURSOR_AREA_V1,
    ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1, ZR_RUNTIME_IME_STATE_DISABLED_V1,
    ZR_RUNTIME_IME_STATE_ENABLED_V1, ZR_RUNTIME_IME_STATE_PREEDIT_V1,
    ZR_RUNTIME_IME_STATE_REQUEST_DISABLE_V1, ZR_RUNTIME_IME_STATE_REQUEST_ENABLE_V1,
    ZR_RUNTIME_IME_STATE_SURROUNDING_TEXT_V1, ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
    ZR_RUNTIME_KEY_ACTION_RELEASED_V1, ZR_RUNTIME_KEY_ACTION_TEXT_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1, ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
    ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1, ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
    ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1, ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1,
    ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1, ZR_RUNTIME_TOUCH_PHASE_ENDED_V1,
    ZR_RUNTIME_TOUCH_PHASE_MOVED_V1, ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
    ZR_RUNTIME_WINDOW_BOOL_FALSE_V1, ZR_RUNTIME_WINDOW_BOOL_TRUE_V1,
    ZR_RUNTIME_WINDOW_STATUS_BACKEND_SCALE_FACTOR_CHANGED_V1,
    ZR_RUNTIME_WINDOW_STATUS_CLOSE_REQUESTED_V1, ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1,
    ZR_RUNTIME_WINDOW_STATUS_MOVED_V1, ZR_RUNTIME_WINDOW_STATUS_OCCLUDED_V1,
    ZR_RUNTIME_WINDOW_STATUS_SCALE_FACTOR_CHANGED_V1, ZR_RUNTIME_WINDOW_STATUS_THEME_CHANGED_V1,
    ZR_RUNTIME_WINDOW_THEME_DARK_V1, ZR_RUNTIME_WINDOW_THEME_LIGHT_V1,
};

use crate::core::diagnostics::collect_runtime_diagnostics;
use crate::core::framework::input::{
    FileDragDropEvent, GamepadAxis, GamepadButton, GamepadConnectionInfo, GamepadId,
    GamepadRumbleRequest, ImeCursorArea, ImeCursorRange, ImeDeleteSurrounding, ImeEvent,
    ImeHostRequest, ImePreedit, ImeSurroundingText, InputButton, InputEvent, InputManager,
    MouseScrollUnit, MouseWheelEvent, TouchPhase, WindowStatusEvent, WindowTheme,
};
use crate::core::framework::render::{
    CapturedFrame, RenderFrameExtract, RenderViewportSurfaceDescriptor,
};
use crate::core::math::{UVec2, Vec2};
use crate::core::CoreRuntime;
use crate::diagnostic_log::{
    write_diagnostic_store_snapshot, DiagnosticStoreLogSchedule, DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT,
};
use crate::scene::components::NodeKind;
use crate::scene::LevelSystem;
use crate::{runtime_modules_for_target, RuntimeTargetMode};

use super::camera_controller::RuntimeCameraController;
use super::frame::{
    encode_accessibility_tree, encode_frame, encode_host_request_batch, encode_profile_response,
    write_accessibility_tree, write_frame, write_host_requests, write_profile_response,
};
use super::runtime_loop::{resolve_input, RuntimeRenderBridge};
use super::surface::render_surface_descriptor;

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

pub(super) unsafe extern "C" fn create_session(
    config: ZrRuntimeSessionConfigV1,
    out_session: *mut ZrRuntimeSessionHandle,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "create_session");
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

    match RuntimeDynamicSession::new(profile) {
        Ok(session) => {
            let handle = insert_session(session);
            ptr::write(out_session, handle);
            ZrStatus::ok()
        }
        Err(error) => error_status(error),
    }
}

pub(super) unsafe extern "C" fn destroy_session(handle: ZrRuntimeSessionHandle) -> ZrStatus {
    if !handle.is_valid() {
        return invalid_argument(b"invalid runtime session handle");
    }
    let mut registry = registry().lock().unwrap();
    if registry.sessions.remove(&handle.raw()).is_none() {
        return not_found(b"runtime session not found");
    }
    ZrStatus::ok()
}

pub(super) unsafe extern "C" fn handle_event(
    handle: ZrRuntimeSessionHandle,
    event: ZrRuntimeEventV1,
) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "handle_event");
    if event.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return unsupported_version();
    }
    with_session(handle, |session| session.handle_event(event))
}

pub(super) unsafe extern "C" fn capture_frame(
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

pub(super) unsafe extern "C" fn capture_accessibility_tree(
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

pub(super) unsafe extern "C" fn bind_viewport_surface(
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

pub(super) unsafe extern "C" fn unbind_viewport_surface(
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

pub(super) unsafe extern "C" fn present_viewport(
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

pub(super) unsafe extern "C" fn profile_control(
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

pub(super) unsafe extern "C" fn tick_frame(handle: ZrRuntimeSessionHandle) -> ZrStatus {
    crate::profile_scope!("runtime", "dynamic_api", "tick_frame");
    with_session(handle, |session| {
        session.tick_frame();
        ZrStatus::ok()
    })
}

pub(super) unsafe extern "C" fn drain_host_requests(
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
    render_bridge: RuntimeRenderBridge,
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
}

impl RuntimeDynamicSession {
    fn new(profile: RuntimeDynamicSessionProfile) -> Result<Self, String> {
        crate::profile_scope!("runtime", "dynamic_api", "runtime_dynamic_session_new");
        let runtime = {
            crate::profile_scope!("runtime", "dynamic_api", "runtime_session_core_new");
            CoreRuntime::new()
        };
        let core = runtime.handle();
        let modules = {
            crate::profile_scope!(
                "runtime",
                "dynamic_api",
                "runtime_session_modules_for_target"
            );
            runtime_modules_for_target(RuntimeTargetMode::ClientRuntime, None)
        };
        if !modules.errors.is_empty() {
            return Err(modules.errors.join("; "));
        }
        {
            crate::profile_scope!("runtime", "dynamic_api", "runtime_session_register_modules");
            for module in &modules.modules {
                runtime
                    .register_module(module.descriptor())
                    .map_err(|error| error.to_string())?;
            }
        }
        {
            crate::profile_scope!("runtime", "dynamic_api", "runtime_session_activate_modules");
            for module in &modules.modules {
                runtime
                    .activate_module(module.module_name())
                    .map_err(|error| error.to_string())?;
            }
        }

        let input_manager = {
            crate::profile_scope!("runtime", "dynamic_api", "runtime_session_resolve_input");
            resolve_input(&core).map_err(|error| error.to_string())?
        };
        let render_bridge = {
            crate::profile_scope!("runtime", "dynamic_api", "runtime_session_render_bridge");
            RuntimeRenderBridge::new(&core).map_err(|error| error.to_string())?
        };
        let level = {
            crate::profile_scope!("runtime", "dynamic_api", "runtime_session_default_level");
            crate::scene::create_default_level(&core).map_err(|error| error.to_string())?
        };
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

    fn tick_frame(&mut self) {
        let advance = self
            .runtime
            .tick_time(self.profile.max_fixed_steps_per_frame());
        if self.diagnostic_log_schedule.tick(advance.real_delta()) {
            let snapshot = collect_runtime_diagnostics(&self.runtime.handle()).store;
            write_diagnostic_store_snapshot(DYNAMIC_RUNTIME_DIAGNOSTIC_LOG_SCOPE, &snapshot);
        }
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

    fn handle_event(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        if event.viewport != DEFAULT_VIEWPORT {
            return not_found(b"runtime viewport not found");
        }
        match event.kind {
            ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1 => {
                self.resize_viewport(UVec2::new(event.size.width, event.size.height));
                ZrStatus::ok()
            }
            ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1 => {
                let cursor = Vec2::new(event.x, event.y);
                self.input_manager.submit_event(InputEvent::CursorMoved {
                    x: cursor.x,
                    y: cursor.y,
                });
                self.handle_cursor_moved(cursor);
                ZrStatus::ok()
            }
            ZR_RUNTIME_EVENT_KIND_CURSOR_ENTERED_V1 => {
                self.input_manager.submit_event(InputEvent::CursorEntered);
                ZrStatus::ok()
            }
            ZR_RUNTIME_EVENT_KIND_CURSOR_LEFT_V1 => {
                self.input_manager.submit_event(InputEvent::CursorLeft);
                ZrStatus::ok()
            }
            ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1 => self.handle_mouse_button(event),
            ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1 => self.handle_mouse_wheel(event),
            ZR_RUNTIME_EVENT_KIND_MOUSE_MOTION_V1 => {
                self.input_manager.submit_event(InputEvent::MouseMotion {
                    delta_x: event.x,
                    delta_y: event.y,
                });
                ZrStatus::ok()
            }
            ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1 => self.handle_lifecycle(event),
            ZR_RUNTIME_EVENT_KIND_TOUCH_V1 => self.handle_touch(event),
            ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1 => self.handle_keyboard(event),
            ZR_RUNTIME_EVENT_KIND_IME_V1 => self.handle_ime(event),
            ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1 => self.handle_file_drag_drop(event),
            ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1 => self.handle_window_status(event),
            ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1 => self.handle_gamepad_connection(event),
            ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V1 => self.handle_gamepad_button(event),
            ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V1 => self.handle_gamepad_axis(event),
            ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1 => {
                self.handle_accessibility_action(event)
            }
            _ => invalid_argument(b"unknown runtime event kind"),
        }
    }

    fn capture_frame(
        &mut self,
        request: ZrRuntimeFrameRequestV1,
    ) -> Result<ZrRuntimeFrameV1, String> {
        let requested = UVec2::new(request.size.width.max(1), request.size.height.max(1));
        self.resize_viewport(requested);
        let extract = self.current_extract();
        let frame = self
            .render_bridge
            .submit_extract(extract, self.camera_controller.viewport_size())
            .map_err(|error| error.to_string())?
            .unwrap_or_else(|| empty_captured_frame(requested));
        Ok(encode_frame(frame))
    }

    fn bind_viewport_surface(
        &mut self,
        descriptor: RenderViewportSurfaceDescriptor,
    ) -> Result<(), String> {
        self.resize_viewport(descriptor.size);
        self.render_bridge
            .bind_surface(descriptor)
            .map_err(|error| error.to_string())
    }

    fn unbind_viewport_surface(&mut self) -> Result<(), String> {
        self.render_bridge
            .unbind_surface()
            .map_err(|error| error.to_string())
    }

    fn present_viewport(&mut self, request: ZrRuntimeFrameRequestV1) -> Result<(), String> {
        let requested = UVec2::new(request.size.width.max(1), request.size.height.max(1));
        self.resize_viewport(requested);
        let extract = self.current_extract();
        self.render_bridge
            .present_extract(extract, self.camera_controller.viewport_size())
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

    fn handle_accessibility_action(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let payload = unsafe { event.payload.as_slice() };
        if serde_json::from_slice::<UiAccessibilityActionRequest>(payload).is_err() {
            return invalid_argument(b"invalid accessibility action payload");
        }
        not_found(
            b"runtime UI surface accessibility action dispatch unavailable in dynamic preview",
        )
    }

    fn current_extract(&self) -> RenderFrameExtract {
        self.level.with_world(|world| {
            world
                .to_render_frame_extract()
                .with_viewport_size(self.camera_controller.viewport_size())
        })
    }

    fn resize_viewport(&mut self, size: UVec2) {
        self.camera_controller.resize(size);
    }

    fn handle_cursor_moved(&mut self, position: Vec2) {
        self.cursor = position;
        self.level
            .with_world_mut(|world| self.camera_controller.pointer_moved(world, position));
        self.sync_orbit_target_from_selection();
    }

    fn handle_mouse_button(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        self.cursor = Vec2::new(event.x, event.y);
        let Some(button) = input_button(event.button) else {
            return invalid_argument(b"unknown runtime mouse button");
        };
        match event.state {
            ZR_RUNTIME_BUTTON_STATE_PRESSED_V1 => {
                self.input_manager
                    .submit_event(InputEvent::ButtonPressed(button));
                self.handle_pressed(event.button);
            }
            ZR_RUNTIME_BUTTON_STATE_RELEASED_V1 => {
                self.input_manager
                    .submit_event(InputEvent::ButtonReleased(button));
                self.handle_released(event.button);
            }
            _ => return invalid_argument(b"unknown runtime button state"),
        }
        ZrStatus::ok()
    }

    fn handle_mouse_wheel(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let unit = match mouse_scroll_unit(event.state) {
            Ok(unit) => unit,
            Err(status) => return status,
        };
        let Some(unit) = unit else {
            self.input_manager
                .submit_event(InputEvent::WheelScrolled { delta: event.delta });
            self.handle_scroll(event.delta);
            return ZrStatus::ok();
        };
        if !event.x.is_finite() || !event.y.is_finite() {
            return invalid_argument(b"invalid runtime mouse wheel delta");
        }
        let wheel = MouseWheelEvent::new(unit, event.x, event.y);
        self.input_manager
            .submit_event(InputEvent::MouseWheel(wheel));
        self.handle_scroll(wheel.legacy_vertical_delta());
        ZrStatus::ok()
    }

    fn handle_lifecycle(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        match event.state {
            ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1
            | ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1
            | ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1 => {
                self.input_manager
                    .submit_event(InputEvent::KeyboardFocusLost);
            }
            _ => {}
        }
        ZrStatus::ok()
    }

    fn handle_touch(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let cursor = Vec2::new(event.x, event.y);
        let Some(phase) = touch_phase(event.state) else {
            return invalid_argument(b"unknown runtime touch phase");
        };
        self.input_manager.submit_event(InputEvent::CursorMoved {
            x: cursor.x,
            y: cursor.y,
        });
        self.input_manager.submit_event(InputEvent::Touch {
            id: event.pointer_id,
            phase,
            x: cursor.x,
            y: cursor.y,
        });
        match event.state {
            ZR_RUNTIME_TOUCH_PHASE_STARTED_V1 => {
                self.handle_cursor_moved(cursor);
                self.handle_pressed(ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1);
            }
            ZR_RUNTIME_TOUCH_PHASE_MOVED_V1 => self.handle_cursor_moved(cursor),
            ZR_RUNTIME_TOUCH_PHASE_ENDED_V1 | ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1 => {
                self.cursor = cursor;
                self.handle_released(ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1);
            }
            _ => unreachable!("touch phase was validated before dispatch"),
        }
        ZrStatus::ok()
    }

    fn handle_keyboard(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let payload = unsafe { event.payload.as_slice() };
        let text = if payload.is_empty() {
            None
        } else {
            String::from_utf8(payload.to_vec()).ok()
        };
        if event.button == ZR_RUNTIME_KEY_ACTION_TEXT_V1 {
            if let Some(text) = text {
                self.input_manager.submit_event(InputEvent::KeyboardInput {
                    key_code: event.key_code,
                    logical_key: None,
                    text: Some(text),
                    pressed: false,
                    repeat: false,
                });
            }
            return ZrStatus::ok();
        }

        let pressed = match event.button {
            ZR_RUNTIME_KEY_ACTION_PRESSED_V1 => true,
            ZR_RUNTIME_KEY_ACTION_RELEASED_V1 => false,
            _ => return ZrStatus::ok(),
        };
        self.input_manager.submit_event(InputEvent::KeyboardInput {
            key_code: event.key_code,
            logical_key: keyboard_logical_key(event.key_code, text.as_deref()),
            text,
            pressed,
            repeat: false,
        });
        ZrStatus::ok()
    }

    fn handle_ime(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let payload = unsafe { event.payload.as_slice() };
        let text_payload = || match String::from_utf8(payload.to_vec()) {
            Ok(text) => Ok(text),
            Err(_) => Err(invalid_argument(b"invalid runtime ime payload")),
        };
        let input_event = match event.state {
            ZR_RUNTIME_IME_STATE_ENABLED_V1 => InputEvent::Ime(ImeEvent::Enabled),
            ZR_RUNTIME_IME_STATE_DISABLED_V1 => InputEvent::Ime(ImeEvent::Disabled),
            ZR_RUNTIME_IME_STATE_PREEDIT_V1 => InputEvent::Ime(ImeEvent::Preedit(ImePreedit {
                value: match text_payload() {
                    Ok(text) => text,
                    Err(status) => return status,
                },
                cursor: ime_cursor(event),
            })),
            ZR_RUNTIME_IME_STATE_COMMIT_V1 => match text_payload() {
                Ok(text) => InputEvent::Ime(ImeEvent::Commit(text)),
                Err(status) => return status,
            },
            ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1 => {
                InputEvent::Ime(ImeEvent::DeleteSurrounding(ImeDeleteSurrounding::new(
                    event.key_code as usize,
                    event.scan_code as usize,
                )))
            }
            ZR_RUNTIME_IME_STATE_REQUEST_ENABLE_V1 => {
                InputEvent::ImeHostRequest(ImeHostRequest::Enable)
            }
            ZR_RUNTIME_IME_STATE_REQUEST_DISABLE_V1 => {
                InputEvent::ImeHostRequest(ImeHostRequest::Disable)
            }
            ZR_RUNTIME_IME_STATE_CURSOR_AREA_V1 => match ime_cursor_area(event) {
                Some(area) => InputEvent::ImeHostRequest(ImeHostRequest::SetCursorArea(area)),
                None => return invalid_argument(b"invalid runtime ime cursor area"),
            },
            ZR_RUNTIME_IME_STATE_SURROUNDING_TEXT_V1 => {
                match ime_surrounding_text(event, payload) {
                    Ok(text) => {
                        InputEvent::ImeHostRequest(ImeHostRequest::SetSurroundingText(text))
                    }
                    Err(status) => return status,
                }
            }
            _ => return invalid_argument(b"unknown runtime ime state"),
        };
        self.input_manager.submit_event(input_event);
        ZrStatus::ok()
    }

    fn handle_file_drag_drop(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let payload = unsafe { event.payload.as_slice() };
        let path_payload = || match String::from_utf8(payload.to_vec()) {
            Ok(path) => Ok(path),
            Err(_) => Err(invalid_argument(b"invalid runtime file drag path")),
        };
        let file_event = match event.state {
            ZR_RUNTIME_FILE_DRAG_HOVERED_V1 => match path_payload() {
                Ok(path) => FileDragDropEvent::Hovered { path },
                Err(status) => return status,
            },
            ZR_RUNTIME_FILE_DRAG_DROPPED_V1 => match path_payload() {
                Ok(path) => FileDragDropEvent::Dropped { path },
                Err(status) => return status,
            },
            ZR_RUNTIME_FILE_DRAG_CANCELLED_V1 => FileDragDropEvent::Cancelled,
            _ => return invalid_argument(b"unknown runtime file drag state"),
        };
        self.input_manager
            .submit_event(InputEvent::FileDragDrop(file_event));
        ZrStatus::ok()
    }

    fn handle_window_status(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let window_event = match event.state {
            ZR_RUNTIME_WINDOW_STATUS_MOVED_V1 => WindowStatusEvent::Moved {
                x: event.x as i32,
                y: event.y as i32,
            },
            ZR_RUNTIME_WINDOW_STATUS_OCCLUDED_V1 => match window_bool(event.button) {
                Some(occluded) => WindowStatusEvent::Occluded(occluded),
                None => return invalid_argument(b"unknown runtime window bool"),
            },
            ZR_RUNTIME_WINDOW_STATUS_THEME_CHANGED_V1 => {
                WindowStatusEvent::ThemeChanged(window_theme(event.button))
            }
            ZR_RUNTIME_WINDOW_STATUS_SCALE_FACTOR_CHANGED_V1 => {
                WindowStatusEvent::ScaleFactorChanged {
                    scale_factor: match window_scale_factor(event.delta) {
                        Some(scale_factor) => scale_factor,
                        None => return invalid_argument(b"invalid runtime window scale factor"),
                    },
                }
            }
            ZR_RUNTIME_WINDOW_STATUS_BACKEND_SCALE_FACTOR_CHANGED_V1 => {
                WindowStatusEvent::BackendScaleFactorChanged {
                    scale_factor: match window_scale_factor(event.delta) {
                        Some(scale_factor) => scale_factor,
                        None => return invalid_argument(b"invalid runtime window scale factor"),
                    },
                }
            }
            ZR_RUNTIME_WINDOW_STATUS_CLOSE_REQUESTED_V1 => WindowStatusEvent::CloseRequested,
            ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1 => WindowStatusEvent::Destroyed,
            _ => return invalid_argument(b"unknown runtime window status"),
        };
        self.input_manager
            .submit_event(InputEvent::WindowStatus(window_event));
        ZrStatus::ok()
    }

    fn handle_gamepad_connection(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let connected = match event.state {
            ZR_RUNTIME_GAMEPAD_CONNECTION_CONNECTED_V1 => true,
            ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1 => false,
            _ => return invalid_argument(b"unknown runtime gamepad connection state"),
        };
        let payload = unsafe { event.payload.as_slice() };
        let name = if payload.is_empty() {
            None
        } else {
            String::from_utf8(payload.to_vec()).ok()
        };
        self.input_manager
            .submit_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
                gamepad: GamepadId(event.pointer_id),
                connected,
                name,
                vendor_id: nonzero_u16(event.key_code),
                product_id: nonzero_u16(event.scan_code),
            }));
        ZrStatus::ok()
    }

    fn handle_gamepad_button(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let pressed = match event.state {
            ZR_RUNTIME_BUTTON_STATE_PRESSED_V1 => true,
            ZR_RUNTIME_BUTTON_STATE_RELEASED_V1 => false,
            _ => return invalid_argument(b"unknown runtime gamepad button state"),
        };
        self.input_manager.submit_event(InputEvent::GamepadButton {
            gamepad: GamepadId(event.pointer_id),
            button: gamepad_button(event.button),
            value: event.delta,
            pressed,
        });
        ZrStatus::ok()
    }

    fn handle_gamepad_axis(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        self.input_manager.submit_event(InputEvent::GamepadAxis {
            gamepad: GamepadId(event.pointer_id),
            axis: gamepad_axis(event.button),
            value: event.delta,
        });
        ZrStatus::ok()
    }

    fn handle_pressed(&mut self, button: u32) {
        match button {
            ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1 => self.camera_controller.left_pressed(self.cursor),
            ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1 => self.camera_controller.right_pressed(self.cursor),
            ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1 => self.camera_controller.middle_pressed(self.cursor),
            _ => {}
        }
    }

    fn handle_released(&mut self, button: u32) {
        match button {
            ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1 => self.camera_controller.left_released(),
            ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1 => self.camera_controller.right_released(),
            ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1 => self.camera_controller.middle_released(),
            _ => {}
        }
    }

    fn handle_scroll(&mut self, delta: f32) {
        self.level
            .with_world_mut(|world| self.camera_controller.scrolled(world, delta));
        self.sync_orbit_target_from_selection();
    }

    fn sync_orbit_target_from_selection(&mut self) {
        let selected = self.selected_node;
        let orbit_target = self.level.with_world(|world| {
            selected
                .and_then(|selected| world.find_node(selected))
                .map(|node| node.transform.translation)
        });
        if let Some(target) = orbit_target {
            self.camera_controller.set_orbit_target(target);
        }
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

pub(super) fn runtime_ime_host_request(request: ImeHostRequest) -> ZrRuntimeImeHostRequestV1 {
    match request {
        ImeHostRequest::Enable => ZrRuntimeImeHostRequestV1::enable(),
        ImeHostRequest::Disable => ZrRuntimeImeHostRequestV1::disable(),
        ImeHostRequest::SetCursorArea(area) => ZrRuntimeImeHostRequestV1::set_cursor_area(
            ZrRuntimeImeCursorAreaV1::new(area.x, area.y, area.width, area.height),
        ),
        ImeHostRequest::SetSurroundingText(text) => {
            ZrRuntimeImeHostRequestV1::set_surrounding_text(ZrRuntimeImeSurroundingTextV1::new(
                text.value,
                text.cursor,
                text.anchor,
            ))
        }
    }
}

pub(super) fn runtime_gamepad_rumble_request(
    request: GamepadRumbleRequest,
) -> ZrRuntimeGamepadRumbleRequestV1 {
    match request {
        GamepadRumbleRequest::Add {
            gamepad,
            intensity,
            duration_millis,
        } => {
            let intensity = intensity.clamped();
            ZrRuntimeGamepadRumbleRequestV1::add(
                gamepad.0,
                intensity.strong_motor,
                intensity.weak_motor,
                duration_millis,
            )
        }
        GamepadRumbleRequest::Stop { gamepad } => ZrRuntimeGamepadRumbleRequestV1::stop(gamepad.0),
    }
}

fn input_button(button: u32) -> Option<InputButton> {
    match button {
        ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1 => Some(InputButton::MouseLeft),
        ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1 => Some(InputButton::MouseRight),
        ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1 => Some(InputButton::MouseMiddle),
        _ => None,
    }
}

fn mouse_scroll_unit(unit: u32) -> Result<Option<MouseScrollUnit>, ZrStatus> {
    match unit {
        0 => Ok(None),
        ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1 => Ok(Some(MouseScrollUnit::Line)),
        ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1 => Ok(Some(MouseScrollUnit::Pixel)),
        _ => Err(invalid_argument(b"unknown runtime mouse wheel unit")),
    }
}

fn touch_phase(phase: u32) -> Option<TouchPhase> {
    match phase {
        ZR_RUNTIME_TOUCH_PHASE_STARTED_V1 => Some(TouchPhase::Started),
        ZR_RUNTIME_TOUCH_PHASE_MOVED_V1 => Some(TouchPhase::Moved),
        ZR_RUNTIME_TOUCH_PHASE_ENDED_V1 => Some(TouchPhase::Ended),
        ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1 => Some(TouchPhase::Cancelled),
        _ => None,
    }
}

fn keyboard_logical_key(key_code: u32, _text: Option<&str>) -> Option<String> {
    keyboard_button_name(key_code).map(str::to_string)
}

fn keyboard_button_name(key_code: u32) -> Option<&'static str> {
    match key_code {
        16 => Some("Shift"),
        17 => Some("Control"),
        18 => Some("Alt"),
        _ => None,
    }
}

fn ime_cursor(event: ZrRuntimeEventV1) -> Option<ImeCursorRange> {
    if event.key_code == ZR_RUNTIME_IME_CURSOR_HIDDEN_V1
        || event.scan_code == ZR_RUNTIME_IME_CURSOR_HIDDEN_V1
    {
        None
    } else {
        Some(ImeCursorRange::new(
            event.key_code as usize,
            event.scan_code as usize,
        ))
    }
}

fn ime_cursor_area(event: ZrRuntimeEventV1) -> Option<ImeCursorArea> {
    if event.x.is_finite() && event.y.is_finite() && event.size.width > 0 && event.size.height > 0 {
        Some(ImeCursorArea::new(
            event.x,
            event.y,
            event.size.width as f32,
            event.size.height as f32,
        ))
    } else {
        None
    }
}

fn ime_surrounding_text(
    event: ZrRuntimeEventV1,
    payload: &[u8],
) -> Result<ImeSurroundingText, ZrStatus> {
    let value = match String::from_utf8(payload.to_vec()) {
        Ok(value) => value,
        Err(_) => return Err(invalid_argument(b"invalid runtime ime payload")),
    };
    let cursor = event.key_code as usize;
    let anchor = event.scan_code as usize;
    if cursor > value.len()
        || anchor > value.len()
        || !value.is_char_boundary(cursor)
        || !value.is_char_boundary(anchor)
    {
        return Err(invalid_argument(b"invalid runtime ime surrounding text"));
    }
    Ok(ImeSurroundingText::new(value, cursor, anchor))
}

fn window_bool(value: u32) -> Option<bool> {
    match value {
        ZR_RUNTIME_WINDOW_BOOL_FALSE_V1 => Some(false),
        ZR_RUNTIME_WINDOW_BOOL_TRUE_V1 => Some(true),
        _ => None,
    }
}

fn window_theme(theme: u32) -> WindowTheme {
    match theme {
        ZR_RUNTIME_WINDOW_THEME_LIGHT_V1 => WindowTheme::Light,
        ZR_RUNTIME_WINDOW_THEME_DARK_V1 => WindowTheme::Dark,
        _ => WindowTheme::Unknown,
    }
}

fn window_scale_factor(value: f32) -> Option<f32> {
    if value.is_finite() && value > 0.0 {
        Some(value)
    } else {
        None
    }
}

fn gamepad_button(button: u32) -> GamepadButton {
    match button {
        ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1 => GamepadButton::South,
        ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1 => GamepadButton::East,
        ZR_RUNTIME_GAMEPAD_BUTTON_NORTH_V1 => GamepadButton::North,
        ZR_RUNTIME_GAMEPAD_BUTTON_WEST_V1 => GamepadButton::West,
        ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER_V1 => GamepadButton::LeftTrigger,
        ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER2_V1 => GamepadButton::LeftTrigger2,
        ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER_V1 => GamepadButton::RightTrigger,
        ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER2_V1 => GamepadButton::RightTrigger2,
        ZR_RUNTIME_GAMEPAD_BUTTON_SELECT_V1 => GamepadButton::Select,
        ZR_RUNTIME_GAMEPAD_BUTTON_START_V1 => GamepadButton::Start,
        ZR_RUNTIME_GAMEPAD_BUTTON_MODE_V1 => GamepadButton::Mode,
        ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_THUMB_V1 => GamepadButton::LeftThumb,
        ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_THUMB_V1 => GamepadButton::RightThumb,
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1 => GamepadButton::DPadUp,
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1 => GamepadButton::DPadDown,
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1 => GamepadButton::DPadLeft,
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1 => GamepadButton::DPadRight,
        ZR_RUNTIME_GAMEPAD_BUTTON_C_V1
        | ZR_RUNTIME_GAMEPAD_BUTTON_Z_V1
        | ZR_RUNTIME_GAMEPAD_BUTTON_UNKNOWN_V1 => GamepadButton::Other(button as u16),
        _ => GamepadButton::Other(button as u16),
    }
}

fn gamepad_axis(axis: u32) -> GamepadAxis {
    match axis {
        ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1 => GamepadAxis::LeftStickX,
        ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1 => GamepadAxis::LeftStickY,
        ZR_RUNTIME_GAMEPAD_AXIS_LEFT_Z_V1 => GamepadAxis::LeftZ,
        ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_X_V1 => GamepadAxis::RightStickX,
        ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_Y_V1 => GamepadAxis::RightStickY,
        ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_Z_V1 => GamepadAxis::RightZ,
        ZR_RUNTIME_GAMEPAD_AXIS_DPAD_X_V1 => GamepadAxis::DPadX,
        ZR_RUNTIME_GAMEPAD_AXIS_DPAD_Y_V1 => GamepadAxis::DPadY,
        ZR_RUNTIME_GAMEPAD_AXIS_UNKNOWN_V1 => GamepadAxis::Other(axis as u16),
        _ => GamepadAxis::Other(axis as u16),
    }
}

fn nonzero_u16(value: u32) -> Option<u16> {
    u16::try_from(value).ok().filter(|value| *value != 0)
}

fn empty_captured_frame(size: UVec2) -> CapturedFrame {
    let width = size.x.max(1);
    let height = size.y.max(1);
    let rgba = vec![0; width as usize * height as usize * 4];
    CapturedFrame::new(width, height, rgba, 0)
}

fn dynamic_preview_accessibility_snapshot() -> UiAccessibilityTreeSnapshot {
    let root = UiNodeId::new(1);
    UiAccessibilityTreeSnapshot {
        tree_id: UiTreeId::new("zircon-runtime-dynamic-preview"),
        roots: vec![root],
        nodes: vec![UiAccessibilityNode {
            node_id: root,
            role: UiA11yRole::Panel,
            name: Some("Zircon Runtime Preview".to_string()),
            ..UiAccessibilityNode::default()
        }],
        focused: None,
        diagnostics: vec![UiAccessibilityDiagnostic {
            severity: UiAccessibilityDiagnosticSeverity::Info,
            code: UiAccessibilityDiagnosticCode::MissingBounds,
            node_id: Some(root),
            message: "runtime UI surface accessibility extraction unavailable in dynamic preview"
                .to_string(),
        }],
    }
}

fn unsupported_version() -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::UnsupportedVersion,
        ZrByteSlice::from_static(b"unsupported runtime ABI version"),
    )
}

fn invalid_argument(message: &'static [u8]) -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::InvalidArgument,
        ZrByteSlice::from_static(message),
    )
}

fn not_found(message: &'static [u8]) -> ZrStatus {
    ZrStatus::new(ZrStatusCode::NotFound, ZrByteSlice::from_static(message))
}

fn error_status(_message: impl Into<String>) -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::Error,
        ZrByteSlice::from_static(b"runtime dynamic API error"),
    )
}
