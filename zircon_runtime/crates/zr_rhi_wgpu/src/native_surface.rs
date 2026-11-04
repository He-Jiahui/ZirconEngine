//! Linux window/display handles shared by scene and retained UI surface creation.
use std::ffi::c_void;
use std::ptr::NonNull;
use wgpu::rwh::{RawDisplayHandle, RawWindowHandle};
use zr_rhi::{RenderNativeSurfaceTarget, RhiError};

pub(crate) fn create_linux_surface(
    instance: &wgpu::Instance,
    target: RenderNativeSurfaceTarget,
) -> Result<wgpu::Surface<'static>, RhiError> {
    let (display, window) = raw_handles(target)?;
    let target = wgpu::SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: Some(display),
        raw_window_handle: window,
    };
    // SAFETY: The native surface contract requires the host to retain its window
    // and display connection until this surface is destroyed, as on Win32.
    unsafe { instance.create_surface_unsafe(target) }
        .map_err(|error| RhiError::SurfaceUnavailable(error.to_string()))
}

fn pointer(value: u64) -> Result<NonNull<c_void>, RhiError> {
    usize::try_from(value)
        .ok()
        .and_then(|value| NonNull::new(value as *mut c_void))
        .ok_or_else(|| RhiError::SurfaceUnavailable("invalid Linux display/surface pointer".into()))
}

fn raw_handles(
    target: RenderNativeSurfaceTarget,
) -> Result<(RawDisplayHandle, RawWindowHandle), RhiError> {
    use wgpu::rwh::{
        WaylandDisplayHandle, WaylandWindowHandle, XcbDisplayHandle, XcbWindowHandle,
        XlibDisplayHandle, XlibWindowHandle,
    };
    match target {
        RenderNativeSurfaceTarget::Xlib {
            display,
            window,
            screen,
            visual_id,
        } => {
            if window == 0 || screen < 0 {
                return Err(RhiError::SurfaceUnavailable(
                    "invalid Xlib window/screen".into(),
                ));
            }
            let mut window_handle = XlibWindowHandle::new(window.try_into().map_err(|_| {
                RhiError::SurfaceUnavailable("Xlib window exceeds native width".into())
            })?);
            window_handle.visual_id = visual_id.try_into().map_err(|_| {
                RhiError::SurfaceUnavailable("Xlib visual exceeds native width".into())
            })?;
            Ok((
                RawDisplayHandle::Xlib(XlibDisplayHandle::new(Some(pointer(display)?), screen)),
                RawWindowHandle::Xlib(window_handle),
            ))
        }
        RenderNativeSurfaceTarget::Xcb {
            connection,
            window,
            screen,
            visual_id,
        } => {
            let window = std::num::NonZeroU32::new(window)
                .ok_or_else(|| RhiError::SurfaceUnavailable("invalid Xcb window".into()))?;
            if screen < 0 || visual_id == Some(0) {
                return Err(RhiError::SurfaceUnavailable(
                    "invalid Xcb screen/visual".into(),
                ));
            }
            let mut window_handle = XcbWindowHandle::new(window);
            window_handle.visual_id = visual_id.and_then(std::num::NonZeroU32::new);
            Ok((
                RawDisplayHandle::Xcb(XcbDisplayHandle::new(Some(pointer(connection)?), screen)),
                RawWindowHandle::Xcb(window_handle),
            ))
        }
        RenderNativeSurfaceTarget::Wayland { display, surface } => Ok((
            RawDisplayHandle::Wayland(WaylandDisplayHandle::new(pointer(display)?)),
            RawWindowHandle::Wayland(WaylandWindowHandle::new(pointer(surface)?)),
        )),
        _ => Err(RhiError::SurfaceUnavailable(
            "expected a native Linux surface target".into(),
        )),
    }
}

#[cfg(test)]
#[path = "tests/native_surface.rs"]
mod tests;
