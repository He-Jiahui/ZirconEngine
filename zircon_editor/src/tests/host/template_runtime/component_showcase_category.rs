use super::support::showcase_binding;
use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostProjection, RetainedUiHostValue,
    UiComponentShowcaseDemoEventInput,
};

fn apply_showcase_binding(
    runtime: &mut EditorUiHostRuntime,
    binding_id: &str,
    input: UiComponentShowcaseDemoEventInput,
) {
    let binding = showcase_binding(runtime, binding_id);
    runtime
        .apply_showcase_demo_binding(&binding, input)
        .unwrap();
}

fn project_showcase(runtime: &EditorUiHostRuntime) -> RetainedUiHostProjection {
    let projection = runtime
        .project_document("res://ui/editor/component_showcase.zui")
        .unwrap();
    let surface = runtime
        .build_shared_surface("res://ui/editor/component_showcase.zui")
        .unwrap();
    runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .unwrap()
}

const SHOWCASE_CATEGORY_CONTROL_IDS: &[&str] = &[
    "ShowAllCategory",
    "ShowVisualCategory",
    "ShowFeedbackCategory",
    "ShowInputCategory",
    "ShowNumericCategory",
    "ShowSelectionCategory",
    "ShowReferenceCategory",
    "ShowDataCategory",
];

const SHOWCASE_VISUAL_DEMO_CONTROL_IDS: &[&str] = &[
    "LabelDemo",
    "RichLabelDemo",
    "ImageDemo",
    "IconDemo",
    "SvgIconDemo",
    "SeparatorDemo",
];

const SHOWCASE_FEEDBACK_DEMO_CONTROL_IDS: &[&str] = &[
    "ProgressBarDemo",
    "SkeletonDemo",
    "SpinnerDemo",
    "BadgeDemo",
    "HelpRowDemo",
    "DialogDemo",
    "ConfirmDialogDemo",
    "CommandPaletteDemo",
    "NotificationCenterDemo",
];

const SHOWCASE_INPUT_DEMO_CONTROL_IDS: &[&str] = &[
    "ButtonDemo",
    "ButtonOutlinedDemo",
    "ButtonTextDemo",
    "ButtonDangerDemo",
    "ButtonDisabledDemo",
    "IconButtonDemo",
    "ToggleButtonDemo",
    "CheckboxDemo",
    "RadioDemo",
    "SegmentedControlDemo",
    "TabDemo",
    "TabStripDemo",
    "InputFieldDemo",
    "TextFieldDemo",
];

const SHOWCASE_NUMERIC_DEMO_CONTROL_IDS: &[&str] = &[
    "NumberFieldDemo",
    "RangeFieldDemo",
    "SliderDemo",
    "RangeSliderDemo",
    "ColorFieldDemo",
    "Vector2FieldDemo",
    "Vector3FieldDemo",
    "Vector4FieldDemo",
];

const SHOWCASE_SELECTION_DEMO_CONTROL_IDS: &[&str] = &[
    "DropdownDemo",
    "ComboBoxDemo",
    "EnumFieldDemo",
    "FlagsFieldDemo",
    "SearchSelectDemo",
    "ContextMenuDemo",
    "DropdownPopupDemo",
];

const SHOWCASE_REFERENCE_DEMO_CONTROL_IDS: &[&str] =
    &["AssetFieldDemo", "InstanceFieldDemo", "ObjectFieldDemo"];

const SHOWCASE_COLLECTION_DEMO_CONTROL_IDS: &[&str] = &[
    "GroupDemo",
    "FoldoutDemo",
    "PropertyRowDemo",
    "InspectorSectionDemo",
    "ArrayFieldDemo",
    "MapFieldDemo",
    "ListRowDemo",
    "TableRowDemo",
    "VirtualListDemo",
    "PagedListDemo",
    "WorldSpaceSurfaceDemo",
    "TreeRowDemo",
    "ContextActionMenuDemo",
];

const SHOWCASE_DEMO_CONTROL_GROUPS: &[&[&str]] = &[
    SHOWCASE_VISUAL_DEMO_CONTROL_IDS,
    SHOWCASE_FEEDBACK_DEMO_CONTROL_IDS,
    SHOWCASE_INPUT_DEMO_CONTROL_IDS,
    SHOWCASE_NUMERIC_DEMO_CONTROL_IDS,
    SHOWCASE_SELECTION_DEMO_CONTROL_IDS,
    SHOWCASE_REFERENCE_DEMO_CONTROL_IDS,
    SHOWCASE_COLLECTION_DEMO_CONTROL_IDS,
];

fn assert_selected_category(host_projection: &RetainedUiHostProjection, selected_control_id: &str) {
    for control_id in SHOWCASE_CATEGORY_CONTROL_IDS {
        assert_eq!(
            host_projection
                .node_by_control_id(control_id)
                .and_then(|node| node.properties.get("selected")),
            Some(&RetainedUiHostValue::Bool(
                *control_id == selected_control_id
            )),
            "unexpected selected state for `{control_id}`"
        );
    }
}

fn assert_demo_controls_for_category(
    host_projection: &RetainedUiHostProjection,
    visible_control_ids: &[&str],
) {
    for control_group in SHOWCASE_DEMO_CONTROL_GROUPS {
        for control_id in *control_group {
            assert_eq!(
                host_projection.node_by_control_id(control_id).is_some(),
                visible_control_ids.contains(control_id),
                "unexpected category visibility for `{control_id}`"
            );
        }
    }
}

fn assert_all_demo_controls_visible(host_projection: &RetainedUiHostProjection) {
    for control_group in SHOWCASE_DEMO_CONTROL_GROUPS {
        for control_id in *control_group {
            assert!(
                host_projection.node_by_control_id(control_id).is_some(),
                "expected `{control_id}` to be visible in All category"
            );
        }
    }
}

#[test]
fn showcase_category_selection_filters_projected_demo_controls() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowDataCategory",
        UiComponentShowcaseDemoEventInput::None,
    );

    let host_projection = project_showcase(&runtime);
    assert_selected_category(&host_projection, "ShowDataCategory");
    assert_demo_controls_for_category(&host_projection, SHOWCASE_COLLECTION_DEMO_CONTROL_IDS);
    assert!(host_projection
        .node_by_control_id("ArrayFieldDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("MapFieldDemo").is_some());
    assert!(host_projection
        .node_by_control_id("PropertyRowDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("InspectorSectionDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("TableRowDemo").is_some());
    assert!(host_projection
        .node_by_control_id("VirtualListDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("PagedListDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("WorldSpaceSurfaceDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("ButtonDemo").is_none());
    assert!(host_projection
        .node_by_control_id("ButtonDisabledDemo")
        .is_none());
    assert!(host_projection.node_by_control_id("DropdownDemo").is_none());
    assert!(host_projection
        .node_by_control_id("ContextMenuDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("DropdownPopupDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("CommandPaletteDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("NotificationCenterDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("ColorFieldDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("AssetFieldDemo")
        .is_none());
    assert!(host_projection.node_by_control_id("LabelDemo").is_none());
    assert!(host_projection.node_by_control_id("TabDemo").is_none());
    assert!(host_projection.node_by_control_id("TabStripDemo").is_none());
    assert!(host_projection.node_by_control_id("SliderDemo").is_none());
    assert!(host_projection
        .node_by_control_id("RangeSliderDemo")
        .is_none());
    assert!(host_projection.node_by_control_id("SkeletonDemo").is_none());
    assert_eq!(
        host_projection
            .node_by_control_id("ShowDataCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(true))
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ShowInputCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(false))
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowVisualCategory",
        UiComponentShowcaseDemoEventInput::None,
    );
    let host_projection = project_showcase(&runtime);
    assert_selected_category(&host_projection, "ShowVisualCategory");
    assert_demo_controls_for_category(&host_projection, SHOWCASE_VISUAL_DEMO_CONTROL_IDS);
    assert!(host_projection.node_by_control_id("LabelDemo").is_some());
    assert!(host_projection
        .node_by_control_id("RichLabelDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("IconDemo").is_some());
    assert!(host_projection.node_by_control_id("SvgIconDemo").is_some());
    assert!(host_projection
        .node_by_control_id("SeparatorDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("ProgressBarDemo")
        .is_none());
    assert!(host_projection.node_by_control_id("SkeletonDemo").is_none());
    assert!(host_projection.node_by_control_id("ButtonDemo").is_none());
    assert!(host_projection.node_by_control_id("DropdownDemo").is_none());
    assert!(host_projection
        .node_by_control_id("ArrayFieldDemo")
        .is_none());
    assert_eq!(
        host_projection
            .node_by_control_id("ShowVisualCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(true))
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ShowDataCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(false))
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowInputCategory",
        UiComponentShowcaseDemoEventInput::None,
    );
    let host_projection = project_showcase(&runtime);
    assert_selected_category(&host_projection, "ShowInputCategory");
    assert_demo_controls_for_category(&host_projection, SHOWCASE_INPUT_DEMO_CONTROL_IDS);
    assert!(host_projection.node_by_control_id("ButtonDemo").is_some());
    assert!(host_projection
        .node_by_control_id("ButtonDisabledDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("InputFieldDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("TextFieldDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("TabDemo").is_some());
    assert!(host_projection.node_by_control_id("TabStripDemo").is_some());
    assert!(host_projection.node_by_control_id("SliderDemo").is_none());
    assert!(host_projection
        .node_by_control_id("RangeSliderDemo")
        .is_none());
    assert!(host_projection.node_by_control_id("SkeletonDemo").is_none());
    assert!(host_projection.node_by_control_id("DropdownDemo").is_none());
    assert!(host_projection
        .node_by_control_id("ContextMenuDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("DropdownPopupDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("ColorFieldDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("AssetFieldDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("ArrayFieldDemo")
        .is_none());
    assert!(host_projection.node_by_control_id("TableRowDemo").is_none());
    assert!(host_projection
        .node_by_control_id("VirtualListDemo")
        .is_none());
    assert!(host_projection.node_by_control_id("LabelDemo").is_none());
    assert_eq!(
        host_projection
            .node_by_control_id("ShowInputCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(true))
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ShowVisualCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(false))
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowNumericCategory",
        UiComponentShowcaseDemoEventInput::None,
    );
    let host_projection = project_showcase(&runtime);
    assert_selected_category(&host_projection, "ShowNumericCategory");
    assert_demo_controls_for_category(&host_projection, SHOWCASE_NUMERIC_DEMO_CONTROL_IDS);
    assert!(host_projection
        .node_by_control_id("NumberFieldDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("SliderDemo").is_some());
    assert!(host_projection
        .node_by_control_id("RangeSliderDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("ColorFieldDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("Vector3FieldDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("TabDemo").is_none());
    assert!(host_projection.node_by_control_id("TabStripDemo").is_none());
    assert!(host_projection.node_by_control_id("ButtonDemo").is_none());
    assert!(host_projection.node_by_control_id("DropdownDemo").is_none());
    assert!(host_projection.node_by_control_id("SkeletonDemo").is_none());
    assert!(host_projection
        .node_by_control_id("AssetFieldDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("ArrayFieldDemo")
        .is_none());
    assert!(host_projection.node_by_control_id("TableRowDemo").is_none());
    assert_eq!(
        host_projection
            .node_by_control_id("ShowNumericCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(true))
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ShowInputCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(false))
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowSelectionCategory",
        UiComponentShowcaseDemoEventInput::None,
    );
    let host_projection = project_showcase(&runtime);
    assert_selected_category(&host_projection, "ShowSelectionCategory");
    assert_demo_controls_for_category(&host_projection, SHOWCASE_SELECTION_DEMO_CONTROL_IDS);
    assert!(host_projection.node_by_control_id("DropdownDemo").is_some());
    assert!(host_projection
        .node_by_control_id("SearchSelectDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("ContextMenuDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("DropdownPopupDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("DialogDemo").is_none());
    assert!(host_projection
        .node_by_control_id("ConfirmDialogDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("CommandPaletteDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("NotificationCenterDemo")
        .is_none());
    assert!(host_projection.node_by_control_id("TabDemo").is_none());
    assert!(host_projection.node_by_control_id("TabStripDemo").is_none());
    assert!(host_projection.node_by_control_id("SliderDemo").is_none());
    assert!(host_projection
        .node_by_control_id("RangeSliderDemo")
        .is_none());
    assert!(host_projection.node_by_control_id("SkeletonDemo").is_none());
    assert!(host_projection.node_by_control_id("ButtonDemo").is_none());
    assert!(host_projection
        .node_by_control_id("ColorFieldDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("WorldSpaceSurfaceDemo")
        .is_none());
    assert_eq!(
        host_projection
            .node_by_control_id("ShowSelectionCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(true))
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ShowNumericCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(false))
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowReferenceCategory",
        UiComponentShowcaseDemoEventInput::None,
    );
    let host_projection = project_showcase(&runtime);
    assert_selected_category(&host_projection, "ShowReferenceCategory");
    assert_demo_controls_for_category(&host_projection, SHOWCASE_REFERENCE_DEMO_CONTROL_IDS);
    assert!(host_projection
        .node_by_control_id("AssetFieldDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("InstanceFieldDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("ButtonDemo").is_none());
    assert!(host_projection.node_by_control_id("DropdownDemo").is_none());
    assert!(host_projection.node_by_control_id("TabDemo").is_none());
    assert!(host_projection.node_by_control_id("TabStripDemo").is_none());
    assert!(host_projection.node_by_control_id("SliderDemo").is_none());
    assert!(host_projection
        .node_by_control_id("RangeSliderDemo")
        .is_none());
    assert!(host_projection.node_by_control_id("SkeletonDemo").is_none());
    assert!(host_projection
        .node_by_control_id("ContextMenuDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("DropdownPopupDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("ArrayFieldDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("VirtualListDemo")
        .is_none());
    assert_eq!(
        host_projection
            .node_by_control_id("ShowReferenceCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(true))
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ShowSelectionCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(false))
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowFeedbackCategory",
        UiComponentShowcaseDemoEventInput::None,
    );
    let host_projection = project_showcase(&runtime);
    assert_selected_category(&host_projection, "ShowFeedbackCategory");
    assert_demo_controls_for_category(&host_projection, SHOWCASE_FEEDBACK_DEMO_CONTROL_IDS);
    assert!(host_projection
        .node_by_control_id("ProgressBarDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("SkeletonDemo").is_some());
    assert!(host_projection.node_by_control_id("HelpRowDemo").is_some());
    assert!(host_projection.node_by_control_id("DialogDemo").is_some());
    assert!(host_projection
        .node_by_control_id("ConfirmDialogDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("CommandPaletteDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("NotificationCenterDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("LabelDemo").is_none());
    assert!(host_projection.node_by_control_id("ButtonDemo").is_none());
    assert!(host_projection.node_by_control_id("TabDemo").is_none());
    assert!(host_projection.node_by_control_id("TabStripDemo").is_none());
    assert!(host_projection.node_by_control_id("SliderDemo").is_none());
    assert!(host_projection
        .node_by_control_id("RangeSliderDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("WorldSpaceSurfaceDemo")
        .is_none());
    assert_eq!(
        host_projection
            .node_by_control_id("ShowFeedbackCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(true))
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ShowReferenceCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(false))
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowAllCategory",
        UiComponentShowcaseDemoEventInput::None,
    );
    let host_projection = project_showcase(&runtime);
    assert_selected_category(&host_projection, "ShowAllCategory");
    assert_all_demo_controls_visible(&host_projection);
    assert!(host_projection.node_by_control_id("LabelDemo").is_some());
    assert!(host_projection.node_by_control_id("ButtonDemo").is_some());
    assert!(host_projection
        .node_by_control_id("ButtonDisabledDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("TabDemo").is_some());
    assert!(host_projection.node_by_control_id("TabStripDemo").is_some());
    assert!(host_projection.node_by_control_id("SliderDemo").is_some());
    assert!(host_projection
        .node_by_control_id("RangeSliderDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("ColorFieldDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("SkeletonDemo").is_some());
    assert!(host_projection.node_by_control_id("DropdownDemo").is_some());
    assert!(host_projection
        .node_by_control_id("ContextMenuDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("DropdownPopupDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("DialogDemo").is_some());
    assert!(host_projection
        .node_by_control_id("ConfirmDialogDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("CommandPaletteDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("NotificationCenterDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("AssetFieldDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("ArrayFieldDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("TableRowDemo").is_some());
    assert!(host_projection
        .node_by_control_id("VirtualListDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("PagedListDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("WorldSpaceSurfaceDemo")
        .is_some());
    assert_eq!(
        host_projection
            .node_by_control_id("ShowAllCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(true))
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ShowFeedbackCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&RetainedUiHostValue::Bool(false))
    );
}
