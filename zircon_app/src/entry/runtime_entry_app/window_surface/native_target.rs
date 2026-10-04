//! Winit 原生句柄到动态 Runtime ABI surface target 的准入边界。
//! Win32 仅在窗口/display 配对受支持时提交；target 不拥有 Window，宿主须覆盖绑定寿命保留窗口。
//! 当前解绑失败后的持有契约见 TODO 0015，不能仅凭构造时的 Window Arc 推断释放已成功。

use winit::raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use winit::window::Window;
use zircon_runtime_interface::{ZrRuntimeNativeSurfaceTargetV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 供绑定层区分缺失句柄与平台配对不受支持，避免模糊的表面错误。
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

/// 从仍存活的主窗口取得合格 Win32 target；结果只可用于该窗口的绑定期间。
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
