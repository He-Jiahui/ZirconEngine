use winit::raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use winit::window::Window;
use zircon_runtime_interface::{ZrRuntimeNativeSurfaceTargetV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::entry::runtime_entry_app) enum NativeSurfaceTargetUnavailable {
    WindowHandleUnavailable,
    DisplayHandleUnavailable,
    UnqualifiedPlatformHandle,
}

impl std::fmt::Display for NativeSurfaceTargetUnavailable {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cause = match self {
            Self::WindowHandleUnavailable => "winit did not expose a live native window handle",
            Self::DisplayHandleUnavailable => "winit did not expose a live native display handle",
            Self::UnqualifiedPlatformHandle => {
                "no qualified native surface backend exists for the current raw window/display pair"
            }
        };
        formatter.write_str(cause)
    }
}

pub(in crate::entry::runtime_entry_app) fn runtime_native_surface_target(
    window: &dyn Window,
) -> Result<ZrRuntimeNativeSurfaceTargetV1, NativeSurfaceTargetUnavailable> {
    let window_handle = window
        .window_handle()
        .map_err(|_| NativeSurfaceTargetUnavailable::WindowHandleUnavailable)?
        .as_raw();
    let display_handle = window
        .display_handle()
        .map_err(|_| NativeSurfaceTargetUnavailable::DisplayHandleUnavailable)?
        .as_raw();
    match (window_handle, display_handle) {
        (RawWindowHandle::Win32(window), RawDisplayHandle::Windows(_display)) => {
            Ok(ZrRuntimeNativeSurfaceTargetV1::win32(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                window.hwnd.get() as usize as u64,
                window
                    .hinstance
                    .map(|value| value.get() as usize as u64)
                    .unwrap_or(0),
            ))
        }
        _ => Err(NativeSurfaceTargetUnavailable::UnqualifiedPlatformHandle),
    }
}
