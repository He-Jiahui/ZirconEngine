use std::cell::RefCell;
#[cfg(windows)]
use std::ffi::c_void;
use std::rc::Rc;
use std::time::{Duration, Instant};

use slint::{ComponentHandle, PhysicalPosition, PhysicalSize};

use crate::app::HubWindow;
use crate::error::HubError;

use super::HubRuntime;

const MIN_WINDOW_WIDTH: u32 = 960;
const MIN_WINDOW_HEIGHT: u32 = 640;
const TITLE_DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(420);

#[derive(Clone, Copy, Debug)]
struct WindowDragState {
    origin: PhysicalPosition,
    press_cursor: PhysicalPosition,
}

#[cfg(windows)]
const GA_ROOT: u32 = 2;
#[cfg(windows)]
const HTCAPTION: usize = 2;
#[cfg(windows)]
const WM_NCLBUTTONDOWN: u32 = 0x00A1;

#[cfg(windows)]
#[repr(C)]
#[derive(Clone, Copy)]
struct WinPoint {
    x: i32,
    y: i32,
}

#[cfg(windows)]
#[link(name = "user32")]
unsafe extern "system" {
    fn GetCursorPos(point: *mut WinPoint) -> i32;
    fn GetAncestor(hwnd: *mut c_void, ga_flags: u32) -> *mut c_void;
    fn ReleaseCapture() -> i32;
    fn SendMessageW(hwnd: *mut c_void, msg: u32, wparam: usize, lparam: isize) -> isize;
    fn WindowFromPoint(point: WinPoint) -> *mut c_void;
}

#[cfg(windows)]
fn current_cursor_position() -> Option<PhysicalPosition> {
    let mut point = WinPoint { x: 0, y: 0 };
    // GetCursorPos returns physical screen coordinates, matching Window::position().
    let ok = unsafe { GetCursorPos(&mut point as *mut WinPoint) };
    (ok != 0).then(|| PhysicalPosition::new(point.x, point.y))
}

#[cfg(windows)]
fn begin_native_window_drag() -> bool {
    let mut point = WinPoint { x: 0, y: 0 };
    let ok = unsafe { GetCursorPos(&mut point as *mut WinPoint) };
    if ok == 0 {
        return false;
    }

    let hwnd = unsafe { WindowFromPoint(point) };
    if hwnd.is_null() {
        return false;
    }

    let root = unsafe { GetAncestor(hwnd, GA_ROOT) };
    let target = if root.is_null() { hwnd } else { root };
    unsafe {
        ReleaseCapture();
        SendMessageW(target, WM_NCLBUTTONDOWN, HTCAPTION, 0);
    }
    true
}

#[cfg(not(windows))]
fn current_cursor_position() -> Option<PhysicalPosition> {
    None
}

#[cfg(not(windows))]
fn begin_native_window_drag() -> bool {
    false
}

impl HubRuntime {
    pub(super) fn apply_window_state(&self, ui: &HubWindow) {
        if let (Some(width), Some(height)) = (self.config.window.width, self.config.window.height) {
            ui.window().set_size(PhysicalSize::new(
                width.max(MIN_WINDOW_WIDTH),
                height.max(MIN_WINDOW_HEIGHT),
            ));
        }
        if let (Some(x), Some(y)) = (self.config.window.position_x, self.config.window.position_y) {
            ui.window().set_position(PhysicalPosition::new(x, y));
        }
        if self.config.window.maximized {
            ui.window().set_maximized(true);
        }
    }

    fn capture_window_state(&mut self, ui: &HubWindow) {
        self.config.window.maximized = ui.window().is_maximized();
        if self.config.window.maximized {
            return;
        }
        let position = ui.window().position();
        let size = ui.window().size();
        self.config.window.position_x = Some(position.x);
        self.config.window.position_y = Some(position.y);
        self.config.window.width = Some(size.width.max(MIN_WINDOW_WIDTH));
        self.config.window.height = Some(size.height.max(MIN_WINDOW_HEIGHT));
    }

    fn persist_window_state(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.capture_window_state(ui);
        self.persist_hub_config()
    }

    fn toggle_window_maximized(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        let maximized = ui.window().is_maximized();
        if !maximized {
            self.capture_window_state(ui);
        }
        ui.window().set_maximized(!maximized);
        self.config.window.maximized = !maximized;
        self.persist_hub_config()
    }
}

pub(super) fn wire_window_controls(ui: &HubWindow, runtime: Rc<RefCell<HubRuntime>>) {
    let weak = ui.as_weak();
    let runtime_for_minimize = Rc::clone(&runtime);
    ui.on_window_minimize(move || {
        if let Some(ui) = weak.upgrade() {
            let _ = runtime_for_minimize.borrow_mut().persist_window_state(&ui);
            ui.window().set_minimized(true);
        }
    });

    let weak = ui.as_weak();
    let runtime_for_maximize = Rc::clone(&runtime);
    ui.on_window_toggle_maximize(move || {
        if let Some(ui) = weak.upgrade() {
            let _ = runtime_for_maximize
                .borrow_mut()
                .toggle_window_maximized(&ui);
        }
    });

    let weak = ui.as_weak();
    let runtime_for_close = Rc::clone(&runtime);
    ui.on_window_close(move || {
        if let Some(ui) = weak.upgrade() {
            let _ = runtime_for_close.borrow_mut().persist_window_state(&ui);
            let _ = ui.window().hide();
        }
        let _ = slint::quit_event_loop();
    });

    let drag_state = Rc::new(RefCell::new(None::<WindowDragState>));
    let last_title_press = Rc::new(RefCell::new(None::<Instant>));
    let weak = ui.as_weak();
    let drag_for_start = Rc::clone(&drag_state);
    let press_for_start = Rc::clone(&last_title_press);
    let runtime_for_drag_start = Rc::clone(&runtime);
    ui.on_window_drag_start(move |_press_x, _press_y| {
        if let Some(ui) = weak.upgrade() {
            let now = Instant::now();
            let is_double_click = press_for_start
                .borrow()
                .is_some_and(|previous| now.duration_since(previous) <= TITLE_DOUBLE_CLICK_WINDOW);
            *press_for_start.borrow_mut() = Some(now);
            if is_double_click {
                *drag_for_start.borrow_mut() = None;
                let _ = runtime_for_drag_start
                    .borrow_mut()
                    .toggle_window_maximized(&ui);
                return;
            }
            if begin_native_window_drag() {
                *drag_for_start.borrow_mut() = None;
                return;
            }
            let Some(press_cursor) = current_cursor_position() else {
                return;
            };
            *drag_for_start.borrow_mut() = Some(WindowDragState {
                origin: ui.window().position(),
                press_cursor,
            });
        }
    });

    let weak = ui.as_weak();
    let drag_for_move = Rc::clone(&drag_state);
    ui.on_window_drag_move(move |_mouse_x, _mouse_y| {
        let Some(ui) = weak.upgrade() else {
            return;
        };
        let Some(state) = *drag_for_move.borrow() else {
            return;
        };
        let Some(cursor) = current_cursor_position() else {
            return;
        };
        ui.window().set_position(PhysicalPosition::new(
            state.origin.x + cursor.x - state.press_cursor.x,
            state.origin.y + cursor.y - state.press_cursor.y,
        ));
    });

    let weak = ui.as_weak();
    let runtime_for_drag_end = Rc::clone(&runtime);
    ui.on_window_drag_end(move || {
        *drag_state.borrow_mut() = None;
        if let Some(ui) = weak.upgrade() {
            let _ = runtime_for_drag_end.borrow_mut().persist_window_state(&ui);
        }
    });
}
