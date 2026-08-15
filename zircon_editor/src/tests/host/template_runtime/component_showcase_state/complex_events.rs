use super::support::apply_showcase_binding;
use crate::ui::template_runtime::{EditorUiHostRuntime, UiComponentShowcaseDemoEventInput};

#[test]
fn showcase_demo_state_applies_complex_component_runtime_events() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/VirtualListScrolled",
        UiComponentShowcaseDemoEventInput::SetVisibleRange {
            start: 240,
            count: 36,
        },
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("VirtualListDemo", "viewport_start")
            .as_deref(),
        Some("240")
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("VirtualListDemo", "requested_start")
            .as_deref(),
        Some("236")
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("VirtualListDemo", "requested_count")
            .as_deref(),
        Some("44")
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("VirtualListDemo", "scroll_offset")
            .as_deref(),
        Some("6720")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/PagedListNextPage",
        UiComponentShowcaseDemoEventInput::SetPage {
            page_index: 1,
            page_size: 100,
        },
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("PagedListDemo", "page_index")
            .as_deref(),
        Some("1")
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("PagedListDemo", "page_start")
            .as_deref(),
        Some("100")
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("PagedListDemo", "page_end")
            .as_deref(),
        Some("200")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/WorldSpaceSurfaceMoved",
        UiComponentShowcaseDemoEventInput::SetWorldTransform {
            position: [1.0, 2.0, 4.0],
            rotation: [0.0, 180.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        },
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/WorldSpaceSurfaceConfigured",
        UiComponentShowcaseDemoEventInput::SetWorldSurface {
            size: [2.5, 1.25],
            pixels_per_meter: 256.0,
            billboard: true,
            depth_test: true,
            render_order: 4,
            camera_target: "viewport-main".to_string(),
        },
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("WorldSpaceSurfaceDemo", "world_position")
            .as_deref(),
        Some("1, 2, 4")
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("WorldSpaceSurfaceDemo", "world_size")
            .as_deref(),
        Some("2.5, 1.25")
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("WorldSpaceSurfaceDemo", "render_order")
            .as_deref(),
        Some("4")
    );

    let log = runtime.showcase_demo_state().event_log();
    assert!(
        log.iter()
            .any(|entry| entry.action == "SetVisibleRange.VirtualList")
    );
    assert!(log.iter().any(|entry| entry.action == "SetPage.PagedList"));
    assert!(
        log.iter()
            .any(|entry| entry.action == "SetWorldSurface.WorldSpaceSurface")
    );
}
