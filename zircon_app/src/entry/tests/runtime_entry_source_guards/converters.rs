use super::sources::runtime_entry_app_root;

#[test]
fn runtime_entry_converter_helpers_stay_family_split() {
    let runtime_converter_root_source = include_str!("../../runtime_entry_app/converters/mod.rs");
    let runtime_converter_abi_source = include_str!("../../runtime_entry_app/converters/abi.rs");
    let runtime_converter_keyboard_source =
        include_str!("../../runtime_entry_app/converters/keyboard.rs");
    let runtime_converter_pointer_source =
        include_str!("../../runtime_entry_app/converters/pointer.rs");
    let runtime_converter_window_source =
        include_str!("../../runtime_entry_app/converters/window.rs");
    let root = runtime_entry_app_root();

    assert!(
        !root.join("converters.rs").exists(),
        "runtime entry converters should be folder-backed instead of returning to an umbrella converters.rs file"
    );
    for required in [
        "mod abi;",
        "mod keyboard;",
        "mod pointer;",
        "mod window;",
        "pub(super) use abi::{byte_slice, usize_to_u32};",
        "pub(super) use keyboard::{key_action, physical_key_code};",
        "pub(super) use pointer::{",
        "pub(super) use window::window_theme;",
    ] {
        assert!(
            runtime_converter_root_source.contains(required),
            "runtime converter root should keep structural re-export `{required}`"
        );
    }
    for required in [
        "fn byte_slice",
        "ZrByteSlice",
        "fn usize_to_u32",
        "u32::try_from(value).unwrap_or(u32::MAX - 1)",
    ] {
        assert!(
            runtime_converter_abi_source.contains(required),
            "runtime ABI converter module should preserve `{required}`"
        );
    }
    for required in [
        "fn key_action",
        "ZR_RUNTIME_KEY_ACTION_PRESSED_V1",
        "ZR_RUNTIME_KEY_ACTION_RELEASED_V1",
        "fn physical_key_code",
        "fn stable_key_code",
        "FNV_OFFSET",
    ] {
        assert!(
            runtime_converter_keyboard_source.contains(required),
            "runtime keyboard converter module should preserve `{required}`"
        );
    }
    for required in [
        "fn pointer_source_touch_id",
        "fn pointer_kind_touch_id",
        "fn touch_button_phase",
        "fn mouse_button",
        "fn button_state",
        "fn mouse_wheel_delta",
        "ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1",
        "ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1",
    ] {
        assert!(
            runtime_converter_pointer_source.contains(required),
            "runtime pointer converter module should preserve `{required}`"
        );
    }
    for required in [
        "fn window_theme",
        "ZR_RUNTIME_WINDOW_THEME_LIGHT_V1",
        "ZR_RUNTIME_WINDOW_THEME_DARK_V1",
    ] {
        assert!(
            runtime_converter_window_source.contains(required),
            "runtime window converter module should preserve `{required}`"
        );
    }
}
