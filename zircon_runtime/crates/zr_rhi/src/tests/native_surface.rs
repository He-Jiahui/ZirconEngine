use super::*;
use std::num::NonZeroU32;
use std::ptr::NonNull;
use winit::raw_window_handle::*;

#[test]
fn linux_window_targets_preserve_the_owning_display_connection() {
    let pointer = NonNull::new(0x1234usize as *mut std::ffi::c_void).unwrap();
    let mut window = XlibWindowHandle::new(42);
    window.visual_id = 33;
    assert_eq!(
        RenderNativeSurfaceTarget::from_raw_handles(
            RawWindowHandle::Xlib(window),
            RawDisplayHandle::Xlib(XlibDisplayHandle::new(Some(pointer), 0))
        )
        .unwrap(),
        RenderNativeSurfaceTarget::Xlib {
            display: 0x1234,
            window: 42,
            screen: 0,
            visual_id: 33
        }
    );
    let window = XcbWindowHandle::new(NonZeroU32::new(42).unwrap());
    assert!(matches!(
        RenderNativeSurfaceTarget::from_raw_handles(
            RawWindowHandle::Xcb(window),
            RawDisplayHandle::Xcb(XcbDisplayHandle::new(Some(pointer), 0))
        )
        .unwrap(),
        RenderNativeSurfaceTarget::Xcb {
            connection: 0x1234,
            window: 42,
            ..
        }
    ));
    assert_eq!(
        RenderNativeSurfaceTarget::from_raw_handles(
            RawWindowHandle::Wayland(WaylandWindowHandle::new(pointer)),
            RawDisplayHandle::Wayland(WaylandDisplayHandle::new(pointer))
        )
        .unwrap(),
        RenderNativeSurfaceTarget::Wayland {
            display: 0x1234,
            surface: 0x1234
        }
    );
}

#[test]
fn absent_or_mismatched_display_handles_are_rejected() {
    let window = RawWindowHandle::Xlib(XlibWindowHandle::new(42));
    assert!(RenderNativeSurfaceTarget::from_raw_handles(
        window,
        RawDisplayHandle::Xlib(XlibDisplayHandle::new(None, 0))
    )
    .is_err());
    assert!(RenderNativeSurfaceTarget::from_raw_handles(
        window,
        RawDisplayHandle::Windows(WindowsDisplayHandle::new())
    )
    .is_err());
}
