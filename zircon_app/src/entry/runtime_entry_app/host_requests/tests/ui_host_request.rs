use super::should_report_count;

#[test]
fn unhandled_ui_host_request_diagnostics_are_logarithmically_bounded() {
    assert!(should_report_count(1));
    assert!(should_report_count(2));
    assert!(!should_report_count(3));
    assert!(should_report_count(4));
}

#[test]
fn diagnostic_source_never_formats_dynamic_host_request_content() {
    let source = include_str!("../ui_host_request.rs");

    assert!(!source.contains(concat!("request.kind.", "href")));
    assert!(!source.contains(concat!("request.kind.", "popup_id")));
    assert!(!source.contains(concat!("request.kind.", "tooltip_id")));
    assert!(!source.contains(concat!("request.", "tree_id")));
}

#[test]
fn typed_ui_cursor_adapter_calls_platform_sink_for_each_shape_and_skips_other_kinds() {
    use winit::cursor::CursorIcon;
    use zircon_runtime_interface::{
        ui::{dispatch::UiPointerLockPolicy, tree::UiCursor},
        ZrRuntimeUiHostRequestKindV1,
    };
    let pairs = [
        (UiCursor::Default, CursorIcon::Default),
        (UiCursor::Pointer, CursorIcon::Pointer),
        (UiCursor::Text, CursorIcon::Text),
        (UiCursor::ResizeEw, CursorIcon::EwResize),
        (UiCursor::ResizeNs, CursorIcon::NsResize),
        (UiCursor::Grab, CursorIcon::Grab),
        (UiCursor::Grabbing, CursorIcon::Grabbing),
    ];
    let mut calls = Vec::new();
    for (cursor, icon) in pairs {
        let before = calls.len();
        assert!(super::apply_ui_cursor_with(
            &ZrRuntimeUiHostRequestKindV1::Cursor(cursor),
            |actual| calls.push(actual)
        ));
        assert_eq!(calls.len(), before + 1);
        assert_eq!(calls[before], icon);
    }
    assert!(!super::apply_ui_cursor_with(
        &ZrRuntimeUiHostRequestKindV1::PointerUnlock {
            policy: UiPointerLockPolicy::RawDelta
        },
        |_| panic!("noncursor must keep original unhandled route"),
    ));
    assert_eq!(calls.len(), 7);
}
