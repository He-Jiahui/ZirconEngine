//! Read-only observations of the exact Arc owned by RuntimeEntryApp.
//! Weak is never upgraded; these observations cannot extend the Window lifetime.

use std::{ffi::c_void, ptr, sync::Weak};

use winit::window::Window;
use zircon_runtime_interface::ZrRuntimeNativeSurfaceTargetV1;

use super::Call;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::entry) struct NativeWindowObservation {
    pub call: Call,
    pub strong_count: usize,
    pub hwnd_live: bool,
    pub visible: bool,
    pub callback_thread: u32,
    pub window_thread: u32,
    pub owner_thread: u32,
}

pub(super) struct NativeWindowProbe {
    owner: Weak<dyn Window>,
    target: ZrRuntimeNativeSurfaceTargetV1,
    owner_thread: u32,
}

impl NativeWindowProbe {
    pub(super) fn new(owner: Weak<dyn Window>, target: ZrRuntimeNativeSurfaceTargetV1) -> Self {
        Self {
            owner,
            target,
            owner_thread: current_native_thread_id(),
        }
    }

    pub(super) fn observe(&self, call: Call) -> NativeWindowObservation {
        let hwnd = native_hwnd(self.target);
        // SAFETY: Win32 accepts an HWND value for these read-only queries, including an expired
        // value. No callback, allocation, destruction or Window ownership is introduced here.
        unsafe {
            NativeWindowObservation {
                call,
                strong_count: self.owner.strong_count(),
                hwnd_live: IsWindow(hwnd) != 0,
                visible: IsWindowVisible(hwnd) != 0,
                callback_thread: GetCurrentThreadId(),
                window_thread: GetWindowThreadProcessId(hwnd, ptr::null_mut()),
                owner_thread: self.owner_thread,
            }
        }
    }
}

pub(in crate::entry) fn current_native_thread_id() -> u32 {
    // SAFETY: The process-local thread ID query takes no pointers.
    unsafe { GetCurrentThreadId() }
}

pub(in crate::entry) fn native_hwnd_is_live(target: ZrRuntimeNativeSurfaceTargetV1) -> bool {
    // SAFETY: IsWindow accepts an expired HWND value; the target is never dereferenced by Rust.
    unsafe { IsWindow(native_hwnd(target)) != 0 }
}

fn native_hwnd(target: ZrRuntimeNativeSurfaceTargetV1) -> *mut c_void {
    let bits = usize::try_from(target.window_handle)
        .expect("the real native target must fit the current Windows pointer width");
    bits as *mut c_void
}

// These test-only declarations match windows-sys 0.61.2's user32/kernel32 signatures.
// Keeping them in this leaf avoids changing production Cargo features for an ignored test.
#[link(name = "user32")]
unsafe extern "system" {
    fn IsWindow(hwnd: *mut c_void) -> i32;
    fn IsWindowVisible(hwnd: *mut c_void) -> i32;
    fn GetWindowThreadProcessId(hwnd: *mut c_void, process: *mut u32) -> u32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentThreadId() -> u32;
}
