use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventError, UiComponentState, UiValue,
};

#[test]
fn virtual_list_visible_range_is_clamped_to_retained_total_count() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let descriptor = registry.descriptor("VirtualList").unwrap();
    let mut state = UiComponentState::new()
        .with_value("total_count", UiValue::Int(100))
        .with_value("overscan", UiValue::Int(-4));

    state
        .apply_event(
            descriptor,
            UiComponentEvent::SetVisibleRange {
                start: 120,
                count: 50,
            },
        )
        .unwrap();

    assert_eq!(state.value("viewport_start"), Some(&UiValue::Int(100)));
    assert_eq!(state.value("viewport_count"), Some(&UiValue::Int(0)));
    assert_eq!(state.value("visible_end"), Some(&UiValue::Int(100)));
    assert_eq!(state.value("requested_start"), Some(&UiValue::Int(100)));
    assert_eq!(state.value("requested_count"), Some(&UiValue::Int(0)));
    assert_eq!(state.value("overscan"), Some(&UiValue::Int(0)));

    state = state
        .with_value("overscan", UiValue::Int(2))
        .with_value("item_extent", UiValue::Float(32.0));
    state
        .apply_event(
            descriptor,
            UiComponentEvent::SetVisibleRange {
                start: 10,
                count: 12,
            },
        )
        .unwrap();

    assert_eq!(state.value("viewport_start"), Some(&UiValue::Int(10)));
    assert_eq!(state.value("viewport_count"), Some(&UiValue::Int(12)));
    assert_eq!(state.value("visible_end"), Some(&UiValue::Int(22)));
    assert_eq!(state.value("requested_start"), Some(&UiValue::Int(8)));
    assert_eq!(state.value("requested_count"), Some(&UiValue::Int(16)));
    assert_eq!(state.value("scroll_offset"), Some(&UiValue::Float(320.0)));
}

#[test]
fn paged_list_page_window_derives_page_count_and_clamps_index() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let descriptor = registry.descriptor("PagedList").unwrap();
    let mut state = UiComponentState::new().with_value("total_count", UiValue::Int(101));

    state
        .apply_event(
            descriptor,
            UiComponentEvent::SetPage {
                page_index: 99,
                page_size: 20,
            },
        )
        .unwrap();

    assert_eq!(state.value("page_size"), Some(&UiValue::Int(20)));
    assert_eq!(state.value("page_count"), Some(&UiValue::Int(6)));
    assert_eq!(state.value("page_index"), Some(&UiValue::Int(5)));
    assert_eq!(state.value("page_start"), Some(&UiValue::Int(100)));
    assert_eq!(state.value("page_end"), Some(&UiValue::Int(101)));
    assert_eq!(state.value("empty"), Some(&UiValue::Bool(false)));

    state
        .apply_event(
            descriptor,
            UiComponentEvent::SetPage {
                page_index: -5,
                page_size: 0,
            },
        )
        .unwrap();

    assert_eq!(state.value("page_size"), Some(&UiValue::Int(1)));
    assert_eq!(state.value("page_index"), Some(&UiValue::Int(0)));
    assert_eq!(state.value("page_start"), Some(&UiValue::Int(0)));
    assert_eq!(state.value("page_end"), Some(&UiValue::Int(1)));
}

#[test]
fn paged_list_page_count_derivation_avoids_integer_overflow() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let descriptor = registry.descriptor("PagedList").unwrap();
    let mut state = UiComponentState::new().with_value("total_count", UiValue::Int(i64::MAX));

    state
        .apply_event(
            descriptor,
            UiComponentEvent::SetPage {
                page_index: i64::MAX,
                page_size: 2,
            },
        )
        .unwrap();

    let expected_page_count = (i64::MAX - 1) / 2 + 1;
    let expected_page_index = expected_page_count - 1;

    assert_eq!(
        state.value("page_count"),
        Some(&UiValue::Int(expected_page_count))
    );
    assert_eq!(
        state.value("page_index"),
        Some(&UiValue::Int(expected_page_index))
    );
    assert_eq!(state.value("page_start"), Some(&UiValue::Int(i64::MAX - 1)));
    assert_eq!(state.value("page_end"), Some(&UiValue::Int(i64::MAX)));
}

#[test]
fn world_space_surface_updates_transform_and_surface_metadata() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let descriptor = registry.descriptor("WorldSpaceSurface").unwrap();
    let mut state = UiComponentState::new();

    state
        .apply_event(
            descriptor,
            UiComponentEvent::SetWorldTransform {
                position: [1.0, 2.0, 3.0],
                rotation: [10.0, 20.0, 30.0],
                scale: [2.0, 3.0, 4.0],
            },
        )
        .unwrap();
    state
        .apply_event(
            descriptor,
            UiComponentEvent::SetWorldSurface {
                size: [4.0, 2.0],
                pixels_per_meter: 10000.0,
                billboard: true,
                depth_test: false,
                render_order: 7,
                camera_target: "viewport-main".to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("world_position"),
        Some(&UiValue::Vec3([1.0, 2.0, 3.0]))
    );
    assert_eq!(
        state.value("world_rotation"),
        Some(&UiValue::Vec3([10.0, 20.0, 30.0]))
    );
    assert_eq!(
        state.value("world_scale"),
        Some(&UiValue::Vec3([2.0, 3.0, 4.0]))
    );
    assert_eq!(state.value("world_size"), Some(&UiValue::Vec2([4.0, 2.0])));
    assert_eq!(
        state.value("pixels_per_meter"),
        Some(&UiValue::Float(8192.0))
    );
    assert_eq!(state.value("billboard"), Some(&UiValue::Bool(true)));
    assert_eq!(state.value("depth_test"), Some(&UiValue::Bool(false)));
    assert_eq!(state.value("render_order"), Some(&UiValue::Int(7)));
    assert_eq!(
        state.value("camera_target"),
        Some(&UiValue::String("viewport-main".to_string()))
    );
}

#[test]
fn world_space_surface_rejects_non_positive_scale_and_size() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let descriptor = registry.descriptor("WorldSpaceSurface").unwrap();
    let mut state = UiComponentState::new();

    let scale_error = state
        .apply_event(
            descriptor,
            UiComponentEvent::SetWorldTransform {
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 0.0, 1.0],
            },
        )
        .unwrap_err();
    assert!(matches!(
        scale_error,
        UiComponentEventError::InvalidComplexValue { ref property, .. }
            if property == "world_scale"
    ));

    let size_error = state
        .apply_event(
            descriptor,
            UiComponentEvent::SetWorldSurface {
                size: [0.0, 1.0],
                pixels_per_meter: 128.0,
                billboard: false,
                depth_test: true,
                render_order: 0,
                camera_target: String::new(),
            },
        )
        .unwrap_err();
    assert!(matches!(
        size_error,
        UiComponentEventError::InvalidComplexValue { ref property, .. }
            if property == "world_size"
    ));
}

#[test]
fn complex_component_events_reject_the_wrong_descriptor() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let descriptor = registry.descriptor("PagedList").unwrap();
    let mut state = UiComponentState::new();

    let error = state
        .apply_event(
            descriptor,
            UiComponentEvent::SetVisibleRange {
                start: 0,
                count: 10,
            },
        )
        .unwrap_err();

    assert!(matches!(
        error,
        UiComponentEventError::UnsupportedEvent { .. }
            | UiComponentEventError::UnsupportedComponentForEvent { .. }
    ));
}
