use winit::raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use winit::window::Window;
use zircon_runtime_interface::{ZrRuntimeNativeSurfaceTargetV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

pub(super) fn runtime_native_surface_target(
    window: &dyn Window,
) -> Option<ZrRuntimeNativeSurfaceTargetV1> {
    let window_handle = window.window_handle().ok()?.as_raw();
    let display_handle = window.display_handle().ok()?.as_raw();
    match (window_handle, display_handle) {
        (RawWindowHandle::Win32(window), RawDisplayHandle::Windows(_display)) => {
            Some(ZrRuntimeNativeSurfaceTargetV1::win32(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                window.hwnd.get() as usize as u64,
                window
                    .hinstance
                    .map(|value| value.get() as usize as u64)
                    .unwrap_or(0),
            ))
        }
        _ => None,
    }
}
