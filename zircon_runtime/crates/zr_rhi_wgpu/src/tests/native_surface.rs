use super::*;

#[test]
fn linux_raw_surface_conversion_preserves_connection_and_window() {
    let (display, window) = raw_handles(RenderNativeSurfaceTarget::Xlib {
        display: 0x1234,
        window: 42,
        screen: 0,
        visual_id: 33,
    })
    .unwrap();
    let RawDisplayHandle::Xlib(display) = display else {
        panic!("Xlib display expected")
    };
    let RawWindowHandle::Xlib(window) = window else {
        panic!("Xlib window expected")
    };
    assert_eq!(display.display.unwrap().as_ptr() as usize, 0x1234);
    assert_eq!(window.window, 42);
    assert_eq!(window.visual_id, 33);
    assert!(raw_handles(RenderNativeSurfaceTarget::Xcb {
        connection: 0x1234,
        window: 42,
        screen: 0,
        visual_id: None,
    })
    .is_ok());
    assert!(raw_handles(RenderNativeSurfaceTarget::Wayland {
        display: 0x1234,
        surface: 0x5678,
    })
    .is_ok());
}

#[test]
fn invalid_linux_native_handles_fail_before_driver_access() {
    for target in [
        RenderNativeSurfaceTarget::Xlib {
            display: 0,
            window: 42,
            screen: 0,
            visual_id: 0,
        },
        RenderNativeSurfaceTarget::Xlib {
            display: 0x1234,
            window: 0,
            screen: 0,
            visual_id: 0,
        },
        RenderNativeSurfaceTarget::Xcb {
            connection: 0,
            window: 42,
            screen: 0,
            visual_id: None,
        },
        RenderNativeSurfaceTarget::Xcb {
            connection: 0x1234,
            window: 0,
            screen: 0,
            visual_id: None,
        },
        RenderNativeSurfaceTarget::Wayland {
            display: 0,
            surface: 0x5678,
        },
        RenderNativeSurfaceTarget::Wayland {
            display: 0x1234,
            surface: 0,
        },
    ] {
        assert!(raw_handles(target).is_err());
    }
}
