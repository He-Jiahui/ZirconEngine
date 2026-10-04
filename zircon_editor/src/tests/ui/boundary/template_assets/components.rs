use std::path::Path;

use zircon_runtime::ui::v2::{UiV2PrototypeStoreFileCache, UiZuiAssetLoader};
use zircon_runtime_interface::ui::component::UiComponentEventKind;

use super::support::source;

#[test]
fn imported_zui_components_are_single_component_assets() {
    for (relative, component) in [
        (
            "assets/ui/editor/components/workbench/shell/activity_drawer_window.zui",
            "ActivityDrawerWindow",
        ),
        (
            "assets/ui/editor/components/showcase/showcase_command_toolbar.zui",
            "ShowcaseCommandToolbar",
        ),
        (
            "assets/ui/editor/components/showcase/showcase_bottom_log.zui",
            "ShowcaseBottomLog",
        ),
        (
            "assets/ui/editor/components/showcase/showcase_category_nav.zui",
            "ShowcaseCategoryNav",
        ),
        (
            "assets/ui/editor/components/showcase/showcase_state_panel.zui",
            "ShowcaseStatePanel",
        ),
        (
            "assets/ui/editor/components/showcase/showcase_visual_section.zui",
            "ShowcaseVisualSection",
        ),
        (
            "assets/ui/editor/components/showcase/showcase_input_section.zui",
            "ShowcaseInputSection",
        ),
        (
            "assets/ui/editor/components/showcase/showcase_selection_section.zui",
            "ShowcaseSelectionSection",
        ),
        (
            "assets/ui/editor/components/showcase/showcase_collections_section.zui",
            "ShowcaseCollectionsSection",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/inputs/workbench_button.zui",
            "WorkbenchButton",
        ),
        (
            "assets/ui/editor/components/workbench/shell/workbench_activity_rail.zui",
            "WorkbenchActivityRail",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/inputs/workbench_checkbox.zui",
            "WorkbenchCheckbox",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/chrome/workbench_chip.zui",
            "WorkbenchChip",
        ),
        (
            "assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui",
            "WorkbenchComponentDrawer",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/inputs/workbench_dropdown.zui",
            "WorkbenchDropdown",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/inputs/workbench_field.zui",
            "WorkbenchField",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/data/workbench_icon.zui",
            "WorkbenchIcon",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/inputs/workbench_search_input.zui",
            "WorkbenchSearchInput",
        ),
        (
            "assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui",
            "WorkbenchInspectorPanel",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/inputs/workbench_icon_button.zui",
            "WorkbenchIconButton",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/data/workbench_list_row.zui",
            "WorkbenchListRow",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/data/workbench_label.zui",
            "WorkbenchLabel",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/data/workbench_divider.zui",
            "WorkbenchDivider",
        ),
        (
            "assets/ui/editor/components/workbench/shell/workbench_main_band.zui",
            "WorkbenchMainBand",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/inputs/workbench_number_field.zui",
            "WorkbenchNumberField",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/feedback/workbench_popup_menu.zui",
            "WorkbenchPopupMenu",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/feedback/workbench_progress_bar.zui",
            "WorkbenchProgressBar",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/data/workbench_property_row.zui",
            "WorkbenchPropertyRow",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/inputs/workbench_radio.zui",
            "WorkbenchRadio",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/chrome/workbench_rail_button.zui",
            "WorkbenchRailButton",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/chrome/workbench_section_title.zui",
            "WorkbenchSectionTitle",
        ),
        (
            "assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui",
            "WorkbenchSceneTreePanel",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/inputs/workbench_segmented_control.zui",
            "WorkbenchSegmentedControl",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/inputs/workbench_slider.zui",
            "WorkbenchSlider",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/inputs/workbench_range_slider.zui",
            "WorkbenchRangeSlider",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/feedback/workbench_skeleton.zui",
            "WorkbenchSkeleton",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/feedback/workbench_status_item.zui",
            "WorkbenchStatusItem",
        ),
        (
            "assets/ui/editor/components/workbench/shell/workbench_status_bar.zui",
            "WorkbenchStatusBar",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/inputs/workbench_tab.zui",
            "WorkbenchTab",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/inputs/workbench_tab_strip.zui",
            "WorkbenchTabStrip",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/data/workbench_table_row.zui",
            "WorkbenchTableRow",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/feedback/workbench_toast.zui",
            "WorkbenchToast",
        ),
        (
            "assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui",
            "WorkbenchTopToolbar",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/inputs/workbench_toggle.zui",
            "WorkbenchToggle",
        ),
        (
            "assets/ui/editor/components/workbench/shell/workbench_viewport_panel.zui",
            "WorkbenchViewportPanel",
        ),
        (
            "assets/ui/editor/components/workbench/primitives/data/workbench_tree_row.zui",
            "WorkbenchTreeRow",
        ),
    ] {
        let document = UiZuiAssetLoader::load_zui_str(&source(relative))
            .unwrap_or_else(|error| panic!("{relative} should load as .zui: {error}"));
        assert!(
            document.components.contains_key(component),
            "{relative} should declare `{component}`"
        );
        assert_eq!(
            document.components.len(),
            1,
            "{relative} should stay a single component prototype"
        );
    }

    let nav = UiZuiAssetLoader::load_zui_str(&source(
        "assets/ui/editor/components/showcase/showcase_category_nav.zui",
    ))
    .expect("showcase category nav should load as .zui");
    for control_id in [
        "ComponentCategoryNav",
        "ShowAllCategory",
        "ShowVisualCategory",
        "ShowFeedbackCategory",
        "ShowInputCategory",
        "ShowNumericCategory",
        "ShowSelectionCategory",
        "ShowReferenceCategory",
        "ShowDataCategory",
    ] {
        assert!(
            nav.nodes
                .values()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "showcase category nav .zui should preserve `{control_id}`"
        );
    }
    assert!(
        nav.nodes.values().any(|node| {
            node.events
                .iter()
                .any(|event| event.id == "UiComponentShowcase/ShowAllCategory")
        }),
        "showcase category nav .zui should preserve category click bindings"
    );

    let state_panel = UiZuiAssetLoader::load_zui_str(&source(
        "assets/ui/editor/components/showcase/showcase_state_panel.zui",
    ))
    .expect("showcase state panel should load as .zui");
    for control_id in [
        "ComponentShowcaseStatePanel",
        "ComponentShowcaseStateTitle",
        "ComponentShowcaseSelectedCategory",
        "ComponentShowcaseLastControl",
        "ComponentShowcaseLastAction",
        "ComponentShowcaseCurrentValue",
        "ComponentShowcaseValidation",
        "ComponentShowcaseDragPayload",
        "ComponentShowcaseEventLog",
    ] {
        assert!(
            state_panel
                .nodes
                .values()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "showcase state panel .zui should preserve `{control_id}`"
        );
    }

    let visual_section = UiZuiAssetLoader::load_zui_str(&source(
        "assets/ui/editor/components/showcase/showcase_visual_section.zui",
    ))
    .expect("showcase visual section should load as .zui");
    for control_id in [
        "ComponentShowcaseVisualSectionTitle",
        "LabelDemo",
        "RichLabelDemo",
        "ImageDemo",
        "IconDemo",
        "SvgIconDemo",
        "SeparatorDemo",
        "ProgressBarDemo",
        "SpinnerDemo",
        "BadgeDemo",
        "HelpRowDemo",
    ] {
        assert!(
            visual_section
                .nodes
                .values()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "showcase visual section .zui should preserve `{control_id}`"
        );
    }

    let input_section = UiZuiAssetLoader::load_zui_str(&source(
        "assets/ui/editor/components/showcase/showcase_input_section.zui",
    ))
    .expect("showcase input section should load as .zui");
    for control_id in [
        "ComponentShowcaseInputSectionTitle",
        "ButtonDemo",
        "IconButtonDemo",
        "ToggleButtonDemo",
        "CheckboxDemo",
        "RadioDemo",
        "SegmentedControlDemo",
        "InputFieldDemo",
        "TextFieldDemo",
        "NumberFieldDemo",
        "RangeFieldDemo",
        "ColorFieldDemo",
        "Vector2FieldDemo",
        "Vector3FieldDemo",
        "Vector4FieldDemo",
    ] {
        assert!(
            input_section
                .nodes
                .values()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "showcase input section .zui should preserve `{control_id}`"
        );
    }
    for event_id in [
        "UiComponentShowcase/ButtonCommit",
        "UiComponentShowcase/InputFieldCommitted",
        "UiComponentShowcase/NumberFieldDragUpdate",
        "UiComponentShowcase/RangeFieldChanged",
        "UiComponentShowcase/Vector4FieldChanged",
    ] {
        assert!(
            input_section
                .nodes
                .values()
                .any(|node| node.events.iter().any(|event| event.id == event_id)),
            "showcase input section .zui should preserve `{event_id}`"
        );
    }

    let selection_section = UiZuiAssetLoader::load_zui_str(&source(
        "assets/ui/editor/components/showcase/showcase_selection_section.zui",
    ))
    .expect("showcase selection section should load as .zui");
    for control_id in [
        "ComponentShowcaseSelectionSectionTitle",
        "DropdownDemo",
        "ComboBoxDemo",
        "EnumFieldDemo",
        "FlagsFieldDemo",
        "SearchSelectDemo",
        "AssetFieldDemo",
        "InstanceFieldDemo",
        "ObjectFieldDemo",
    ] {
        assert!(
            selection_section
                .nodes
                .values()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "showcase selection section .zui should preserve `{control_id}`"
        );
    }
    for event_id in [
        "UiComponentShowcase/DropdownChanged",
        "UiComponentShowcase/SearchSelectQueryChanged",
        "UiComponentShowcase/AssetFieldDropped",
        "UiComponentShowcase/AssetFieldClear",
        "UiComponentShowcase/ObjectFieldClear",
    ] {
        assert!(
            selection_section
                .nodes
                .values()
                .any(|node| node.events.iter().any(|event| event.id == event_id)),
            "showcase selection section .zui should preserve `{event_id}`"
        );
    }

    let collections_section = UiZuiAssetLoader::load_zui_str(&source(
        "assets/ui/editor/components/showcase/showcase_collections_section.zui",
    ))
    .expect("showcase collections section should load as .zui");
    for control_id in [
        "ComponentShowcaseCollectionsSectionTitle",
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
    ] {
        assert!(
            collections_section
                .nodes
                .values()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "showcase collections section .zui should preserve `{control_id}`"
        );
    }
    for event_id in [
        "UiComponentShowcase/GroupToggled",
        "UiComponentShowcase/FoldoutToggled",
        "UiComponentShowcase/InspectorSectionToggled",
        "UiComponentShowcase/ArrayFieldAddElement",
        "UiComponentShowcase/MapFieldSetEntry",
        "UiComponentShowcase/ListRowClicked",
        "UiComponentShowcase/VirtualListScrolled",
        "UiComponentShowcase/PagedListNextPage",
        "UiComponentShowcase/WorldSpaceSurfaceMoved",
        "UiComponentShowcase/TreeRowToggled",
        "UiComponentShowcase/ContextActionMenuOpenAt",
    ] {
        assert!(
            collections_section
                .nodes
                .values()
                .any(|node| node.events.iter().any(|event| event.id == event_id)),
            "showcase collections section .zui should preserve `{event_id}`"
        );
    }
}

#[test]
fn component_showcase_is_hard_cut_to_zui_catalog_components() {
    let searchable_assets = [
        "assets/ui/editor/component_showcase.zui",
        "assets/ui/editor/components/showcase/showcase_visual_section.zui",
        "assets/ui/editor/components/showcase/showcase_input_section.zui",
        "assets/ui/editor/components/showcase/showcase_selection_section.zui",
        "assets/ui/editor/components/showcase/showcase_collections_section.zui",
    ]
    .into_iter()
    .map(source)
    .collect::<Vec<_>>()
    .join("\n");

    for forbidden in [
        "material_meta_components.ui.toml",
        "component_widgets.ui.toml#ShowcaseSection",
        "component_ref",
        "kind = \"reference\"",
    ] {
        assert!(
            !searchable_assets.contains(forbidden),
            "component showcase .zui asset should not depend on old recursive `{forbidden}`"
        );
    }

    for required in [
        "component = \"ProgressBar\"",
        "component = \"Spinner\"",
        "component = \"Button\"",
        "component = \"IconButton\"",
        "component = \"ToggleButton\"",
        "component = \"Checkbox\"",
        "component = \"InputField\"",
        "component = \"TextField\"",
        "component = \"NumberField\"",
        "component = \"RangeField\"",
        "component = \"Dropdown\"",
        "component = \"ComboBox\"",
        "component = \"EnumField\"",
        "component = \"FlagsField\"",
        "component = \"SearchSelect\"",
        "component = \"AssetField\"",
        "component = \"InstanceField\"",
        "component = \"ObjectField\"",
        "component = \"Group\"",
        "component = \"Foldout\"",
        "component = \"PropertyRow\"",
        "component = \"InspectorSection\"",
        "component = \"ArrayField\"",
        "component = \"MapField\"",
        "component = \"ListRow\"",
        "component = \"TableRow\"",
        "component = \"VirtualList\"",
        "component = \"PagedList\"",
        "component = \"WorldSpaceSurface\"",
        "component = \"TreeRow\"",
        "component = \"ContextActionMenu\"",
        "res://ui/theme/editor_material.zui",
        "res://ui/editor/components/showcase/showcase_command_toolbar.zui#ShowcaseCommandToolbar",
        "res://ui/editor/components/showcase/showcase_bottom_log.zui#ShowcaseBottomLog",
        "res://ui/editor/components/showcase/showcase_category_nav.zui#ShowcaseCategoryNav",
        "res://ui/editor/components/showcase/showcase_state_panel.zui#ShowcaseStatePanel",
        "res://ui/editor/components/showcase/showcase_visual_section.zui#ShowcaseVisualSection",
        "res://ui/editor/components/showcase/showcase_input_section.zui#ShowcaseInputSection",
        "res://ui/editor/components/showcase/showcase_selection_section.zui#ShowcaseSelectionSection",
        "res://ui/editor/components/showcase/showcase_collections_section.zui#ShowcaseCollectionsSection",
        "component = \"ShowcaseCommandToolbar\"",
        "component = \"ShowcaseBottomLog\"",
        "component = \"ShowcaseCategoryNav\"",
        "component = \"ShowcaseStatePanel\"",
        "component = \"ShowcaseVisualSection\"",
        "component = \"ShowcaseInputSection\"",
        "component = \"ShowcaseSelectionSection\"",
        "component = \"ShowcaseCollectionsSection\"",
    ] {
        assert!(
            searchable_assets.contains(required),
            "component showcase .zui asset missing `{required}`"
        );
    }
}

#[test]
fn material_meta_components_cover_retained_material_exports() {
    let asset = source("src/tests/fixtures/ui_zui/editor/material_meta_components.zui");
    for component in [
        "ButtonBase",
        "Button",
        "TextButton",
        "IconButton",
        "CheckBox",
        "ComboBox",
        "Ripple",
        "StateLayer",
        "ListItem",
        "DatePickerPopup",
        "GroupBox",
        "LineEdit",
        "MenuBarItem",
        "MenuBar",
        "MenuFrame",
        "MenuItem",
        "ProgressIndicator",
        "ScrollView",
        "Slider",
        "SpinBox",
        "Spinner",
        "Switch",
        "StandardTableView",
        "TabWidgetImpl",
        "TabImpl",
        "TabBarHorizontalImpl",
        "TabBarVerticalImpl",
        "TabWidget",
        "TextEdit",
        "TimePickerPopup",
    ] {
        let marker = format!("[components.Material{component}]");
        assert!(
            asset.contains(&marker),
            "material_meta_components.zui missing Retained Material export `{component}`"
        );
    }
}

#[test]
fn typed_component_event_product_assets_declare_lower_snake_routes() {
    let expectations = [
        (
            "assets/ui/editor/components/showcase/showcase_collections_section.zui",
            "UiComponentShowcase/GroupToggled",
            UiComponentEventKind::ToggleExpanded,
        ),
        (
            "assets/ui/editor/components/showcase/showcase_collections_section.zui",
            "UiComponentShowcase/ContextActionMenuOpenAt",
            UiComponentEventKind::OpenPopupAt,
        ),
        (
            "assets/ui/editor/components/showcase/showcase_input_section.zui",
            "UiComponentShowcase/NumberFieldLargeDragUpdate",
            UiComponentEventKind::LargeDragDelta,
        ),
        (
            "assets/ui/editor/components/showcase/showcase_input_section.zui",
            "UiComponentShowcase/TextFieldCommitted",
            UiComponentEventKind::Commit,
        ),
        (
            "assets/ui/editor/components/showcase/showcase_input_section.zui",
            "UiComponentShowcase/TabChanged",
            UiComponentEventKind::ValueChanged,
        ),
        (
            "assets/ui/editor/components/showcase/showcase_input_section.zui",
            "UiComponentShowcase/TabStripChanged",
            UiComponentEventKind::ValueChanged,
        ),
        (
            "assets/ui/editor/components/showcase/showcase_selection_section.zui",
            "UiComponentShowcase/SearchSelectQueryChanged",
            UiComponentEventKind::ValueChanged,
        ),
        (
            "assets/ui/editor/components/showcase/showcase_selection_section.zui",
            "UiComponentShowcase/AssetFieldDropHovered",
            UiComponentEventKind::DropHover,
        ),
        (
            "assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui",
            "ComponentLab/ButtonDropdownOpen",
            UiComponentEventKind::OpenPopup,
        ),
        (
            "assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui",
            "ComponentLab/InputSearchCommit",
            UiComponentEventKind::Commit,
        ),
    ];

    let mut cache = UiV2PrototypeStoreFileCache::new();
    for (relative, binding_id, expected_event) in expectations {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        let outcome = cache
            .load_store(std::iter::once(path))
            .unwrap_or_else(|error| panic!("load and compile `{relative}`: {error}"));
        let document = outcome.root_document;
        let binding = document
            .nodes
            .values()
            .flat_map(|node| &node.events)
            .find(|binding| binding.id == binding_id)
            .unwrap_or_else(|| panic!("`{relative}` missing binding `{binding_id}`"));
        let route = binding.route.as_deref().expect("product binding route");
        assert_eq!(route, route.to_ascii_lowercase());
        assert_eq!(binding.component_event, Some(expected_event));
    }
}
