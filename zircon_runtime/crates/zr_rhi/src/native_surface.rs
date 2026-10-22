//! 平台原生表面目标的最小句柄描述；具体窗口桥接由 WGPU/宿主层负责。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 当前支持的原生窗口目标，数值仅用于跨边界传递，不承担生命周期所有权。
pub enum RenderNativeSurfaceTarget {
    Win32 {
        hwnd: u64,
        hinstance: Option<u64>,
    },
    /// Xlib connection and window; the host retains both until surface teardown.
    #[cfg(target_os = "linux")]
    Xlib {
        display: u64,
        window: u64,
        screen: i32,
        visual_id: u64,
    },
    /// Xcb connection and window; the host retains both until surface teardown.
    #[cfg(target_os = "linux")]
    Xcb {
        connection: u64,
        window: u32,
        screen: i32,
        visual_id: Option<u32>,
    },
    /// Wayland display and wl_surface; the host retains both until surface teardown.
    #[cfg(target_os = "linux")]
    Wayland {
        display: u64,
        surface: u64,
    },
    /// Opaque browser host identity; the Canvas itself is owned by the WebGPU backend.
    #[cfg(target_arch = "wasm32")]
    Canvas {
        identity: u64,
    },
}

#[cfg(feature = "platform-winit")]
impl RenderNativeSurfaceTarget {
    pub(crate) fn from_raw_handles(
        window: winit::raw_window_handle::RawWindowHandle,
        display: winit::raw_window_handle::RawDisplayHandle,
    ) -> Result<Self, crate::RhiError> {
        use winit::raw_window_handle::{RawDisplayHandle as D, RawWindowHandle as W};
        match (window, display) {
            (W::Win32(window), D::Windows(_)) => Ok(Self::Win32 {
                hwnd: window.hwnd.get() as u64,
                hinstance: window.hinstance.map(|value| value.get() as u64),
            }),
            #[cfg(target_os = "linux")]
            (W::Xlib(window), D::Xlib(display)) if window.window != 0 => {
                let pointer = display.display.ok_or_else(|| crate::RhiError::SurfaceUnavailable(
                    "Xlib surface requires the owning display connection".into()))?;
                Ok(Self::Xlib { display: pointer.as_ptr() as usize as u64,
                    window: window.window as u64, screen: display.screen, visual_id: window.visual_id as u64 })
            }
            #[cfg(target_os = "linux")]
            (W::Xcb(window), D::Xcb(display)) => {
                let pointer = display.connection.ok_or_else(|| crate::RhiError::SurfaceUnavailable(
                    "Xcb surface requires the owning display connection".into()))?;
                Ok(Self::Xcb { connection: pointer.as_ptr() as usize as u64,
                    window: window.window.get(), screen: display.screen,
                    visual_id: window.visual_id.map(|value| value.get()) })
            }
            #[cfg(target_os = "linux")]
            (W::Wayland(window), D::Wayland(display)) => Ok(Self::Wayland {
                display: display.display.as_ptr() as usize as u64,
                surface: window.surface.as_ptr() as usize as u64,
            }),
            (window, display) => Err(crate::RhiError::SurfaceUnavailable(format!(
                "unsupported native window/display pair for retained UI surface: {window:?}, {display:?}"))),
        }
    }
}

#[cfg(all(test, feature = "platform-winit", target_os = "linux"))]
#[path = "tests/native_surface.rs"]
mod tests;
