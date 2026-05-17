use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::UiComponentEventKind;

use super::{assert_has_event, assert_has_prop, assert_has_state};
#[test]
fn runtime_component_catalog_selection_and_state_coverage() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    for (component_id, property, expected_states) in [
        (
            "Dropdown",
            "options",
            &["focused", "popup_open", "selected"][..],
        ),
        (
            "ComboBox",
            "options",
            &["focused", "popup_open", "selected"][..],
        ),
        (
            "EnumField",
            "options",
            &["focused", "popup_open", "selected"][..],
        ),
        (
            "FlagsField",
            "options",
            &["focused", "popup_open", "selected"][..],
        ),
        (
            "SearchSelect",
            "query",
            &["focused", "popup_open", "selected", "query"][..],
        ),
        (
            "SegmentedControl",
            "options",
            &["focused", "selected", "disabled"][..],
        ),
        (
            "AssetField",
            "value",
            &[
                "focused",
                "dragging",
                "drop_hovered",
                "active_drag_target",
                "disabled",
            ][..],
        ),
        (
            "InstanceField",
            "value",
            &[
                "focused",
                "dragging",
                "drop_hovered",
                "active_drag_target",
                "disabled",
            ][..],
        ),
        (
            "ObjectField",
            "value",
            &[
                "focused",
                "dragging",
                "drop_hovered",
                "active_drag_target",
                "disabled",
            ][..],
        ),
        (
            "Group",
            "expanded",
            &["expanded", "focused", "disabled"][..],
        ),
        (
            "Foldout",
            "expanded",
            &["expanded", "focused", "disabled"][..],
        ),
        (
            "ContextActionMenu",
            "options",
            &["focused", "selected", "popup_open"][..],
        ),
        (
            "WorldSpaceSurface",
            "world_position",
            &[
                "world_position",
                "world_rotation",
                "world_scale",
                "world_size",
                "pixels_per_meter",
                "billboard",
                "depth_test",
                "render_order",
                "camera_target",
            ][..],
        ),
        (
            "ListRow",
            "text",
            &["selected", "focused", "hovered", "pressed"][..],
        ),
        (
            "VirtualList",
            "items",
            &[
                "items",
                "total_count",
                "viewport_start",
                "viewport_count",
                "item_extent",
                "overscan",
            ][..],
        ),
        (
            "PagedList",
            "items",
            &[
                "items",
                "total_count",
                "page_index",
                "page_size",
                "page_count",
            ][..],
        ),
        (
            "TreeRow",
            "text",
            &["expanded", "selected", "focused", "hovered", "pressed"][..],
        ),
    ] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing component descriptor `{component_id}`"));
        assert_has_prop(descriptor, property);
        for state in expected_states {
            assert_has_state(descriptor, state);
        }
        if let Some(options_prop) = descriptor.prop("options") {
            assert!(
                options_prop.options.iter().any(|option| option.disabled),
                "component `{component_id}` should advertise at least one disabled option"
            );
            if matches!(
                component_id,
                "Dropdown" | "ComboBox" | "EnumField" | "FlagsField" | "SearchSelect"
            ) {
                assert_has_prop(descriptor, "selection_state");
                assert_has_prop(descriptor, "value_text");
                assert_has_prop(descriptor, "popup_open");
                assert_has_prop(descriptor, "disabled_options");
                assert_has_prop(descriptor, "special_options");
                assert_has_prop(descriptor, "focused_options");
                assert_has_prop(descriptor, "hovered_options");
                assert_has_prop(descriptor, "pressed_options");
            }
        }
    }

    for (component_id, property) in [
        ("IconButton", "text"),
        ("ToggleButton", "text"),
        ("Checkbox", "text"),
        ("Radio", "text"),
        ("Group", "text"),
        ("Foldout", "text"),
        ("PropertyRow", "text"),
        ("PropertyRow", "value"),
        ("InspectorSection", "text"),
        ("WorldSpaceSurface", "world_position"),
        ("ListRow", "text"),
        ("ListRow", "value"),
        ("VirtualList", "items"),
        ("PagedList", "items"),
        ("Separator", "text"),
        ("Spinner", "text"),
        ("SegmentedControl", "value"),
        ("ContextActionMenu", "value"),
    ] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing component descriptor `{component_id}`"));
        assert_has_prop(descriptor, property);
    }

    for component_id in [
        "Dropdown",
        "ComboBox",
        "EnumField",
        "FlagsField",
        "SearchSelect",
        "ContextActionMenu",
    ] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_has_event(descriptor, UiComponentEventKind::Focus);
        assert_has_event(descriptor, UiComponentEventKind::OpenPopup);
        assert_has_event(descriptor, UiComponentEventKind::ClosePopup);
        assert_has_event(descriptor, UiComponentEventKind::SelectOption);
    }

    let segmented = registry.descriptor("SegmentedControl").unwrap();
    assert_has_event(segmented, UiComponentEventKind::Focus);
    assert_has_event(segmented, UiComponentEventKind::SelectOption);

    let context_menu = registry.descriptor("ContextActionMenu").unwrap();
    assert_has_event(context_menu, UiComponentEventKind::OpenPopupAt);

    for component_id in ["NumberField"] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_has_event(descriptor, UiComponentEventKind::Focus);
        assert_has_event(descriptor, UiComponentEventKind::BeginDrag);
        assert_has_event(descriptor, UiComponentEventKind::DragDelta);
        assert_has_event(descriptor, UiComponentEventKind::EndDrag);
        assert_has_event(descriptor, UiComponentEventKind::Commit);
        assert_has_event(descriptor, UiComponentEventKind::ValueChanged);
    }

    for component_id in ["RangeField"] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_has_event(descriptor, UiComponentEventKind::Focus);
        assert_has_event(descriptor, UiComponentEventKind::DragDelta);
        assert_has_event(descriptor, UiComponentEventKind::Commit);
        assert_has_event(descriptor, UiComponentEventKind::ValueChanged);
    }

    for component_id in ["Group", "Foldout"] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_has_event(descriptor, UiComponentEventKind::ToggleExpanded);
    }

    let input = registry.descriptor("InputField").unwrap();
    assert_has_event(input, UiComponentEventKind::ValueChanged);
    assert_has_event(input, UiComponentEventKind::Commit);
    assert_has_event(input, UiComponentEventKind::Focus);

    for component_id in ["ListRow", "TreeRow"] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_has_event(descriptor, UiComponentEventKind::Hover);
        assert_has_event(descriptor, UiComponentEventKind::Press);
    }

    for component_id in ["VirtualList", "PagedList"] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_has_event(descriptor, UiComponentEventKind::ValueChanged);
    }

    let virtual_list = registry.descriptor("VirtualList").unwrap();
    assert_has_event(virtual_list, UiComponentEventKind::SetVisibleRange);

    let paged_list = registry.descriptor("PagedList").unwrap();
    assert_has_event(paged_list, UiComponentEventKind::SetPage);

    let world_space_surface = registry.descriptor("WorldSpaceSurface").unwrap();
    assert_has_event(world_space_surface, UiComponentEventKind::SetWorldTransform);
    assert_has_event(world_space_surface, UiComponentEventKind::SetWorldSurface);

    for component_id in ["AssetField", "InstanceField", "ObjectField"] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_has_event(descriptor, UiComponentEventKind::DropHover);
        assert_has_event(descriptor, UiComponentEventKind::ActiveDragTarget);
    }
}
