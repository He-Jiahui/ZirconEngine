use std::collections::HashMap;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1,
    ZrRuntimeSessionConfigV1, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle, ZrStatus,
    ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1,
    ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1, ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1,
    ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
    ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1, ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
};

use crate::core::framework::input::{InputButton, InputEvent, InputManager};
use crate::core::framework::render::{CapturedFrame, RenderFrameExtract};
use crate::core::math::{UVec2, Vec2};
use crate::core::CoreRuntime;
use crate::scene::components::NodeKind;
use crate::scene::LevelSystem;
use crate::{runtime_modules_for_target, RuntimeTargetMode};

use super::camera_controller::RuntimeCameraController;
use super::frame::{encode_frame, write_frame};
use super::runtime_loop::{resolve_input, RuntimeRenderBridge};

const DEFAULT_VIEWPORT: ZrRuntimeViewportHandle = ZrRuntimeViewportHandle::new(1);

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
    if out_session.is_null() {
        return invalid_argument(b"missing runtime session output");
    }
    if config.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return unsupported_version();
    }

    match RuntimeDynamicSession::new() {
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

struct RuntimeDynamicSession {
    _runtime: CoreRuntime,
    render_bridge: RuntimeRenderBridge,
    level: LevelSystem,
    selected_node: Option<u64>,
    camera_controller: RuntimeCameraController,
    cursor: Vec2,
    input_manager: Arc<dyn InputManager>,
}

impl RuntimeDynamicSession {
    fn new() -> Result<Self, String> {
        let runtime = CoreRuntime::new();
        let core = runtime.handle();
        let modules = runtime_modules_for_target(RuntimeTargetMode::ClientRuntime, None);
        if !modules.errors.is_empty() {
            return Err(modules.errors.join("; "));
        }
        for module in &modules.modules {
            runtime
                .register_module(module.descriptor())
                .map_err(|error| error.to_string())?;
        }
        for module in &modules.modules {
            runtime
                .activate_module(module.module_name())
                .map_err(|error| error.to_string())?;
        }

        let input_manager = resolve_input(&core).map_err(|error| error.to_string())?;
        let render_bridge = RuntimeRenderBridge::new(&core).map_err(|error| error.to_string())?;
        let level = crate::scene::create_default_level(&core).map_err(|error| error.to_string())?;
        let (selected_node, orbit_target) = level.with_world(|world| {
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
        });
        let mut camera_controller = RuntimeCameraController::new(UVec2::new(1280, 720));
        camera_controller.set_orbit_target(orbit_target);

        Ok(Self {
            _runtime: runtime,
            render_bridge,
            level,
            selected_node,
            camera_controller,
            cursor: Vec2::ZERO,
            input_manager,
        })
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
            ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1 => self.handle_mouse_button(event),
            ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1 => {
                self.input_manager
                    .submit_event(InputEvent::WheelScrolled { delta: event.delta });
                self.handle_scroll(event.delta);
                ZrStatus::ok()
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

fn input_button(button: u32) -> Option<InputButton> {
    match button {
        ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1 => Some(InputButton::MouseLeft),
        ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1 => Some(InputButton::MouseRight),
        ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1 => Some(InputButton::MouseMiddle),
        _ => None,
    }
}

fn empty_captured_frame(size: UVec2) -> CapturedFrame {
    let width = size.x.max(1);
    let height = size.y.max(1);
    let rgba = vec![0; width as usize * height as usize * 4];
    CapturedFrame::new(width, height, rgba, 0)
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
