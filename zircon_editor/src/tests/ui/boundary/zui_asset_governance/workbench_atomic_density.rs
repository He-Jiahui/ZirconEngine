use toml::Value;
use zircon_runtime_interface::ui::v2::UiV2NodeDefinition;

use super::support::{editor_asset_root, load_zui_document};

const INPUT_ROOT: &str = "ui/editor/components/workbench/primitives/inputs";
const DATA_ROOT: &str = "ui/editor/components/workbench/primitives/data";
const EPSILON: f64 = 0.000_1;

fn root_node(file_name: &str, component_name: &str) -> UiV2NodeDefinition {
    root_node_under(INPUT_ROOT, file_name, component_name)
}

fn root_node_under(
    component_root: &str,
    file_name: &str,
    component_name: &str,
) -> UiV2NodeDefinition {
    let path = editor_asset_root().join(component_root).join(file_name);
    let document = load_zui_document(&path);
    let component = document.components.get(component_name).unwrap_or_else(|| {
        panic!(
            "{} should declare component `{component_name}`",
            path.display()
        )
    });
    document
        .nodes
        .get(&component.root)
        .unwrap_or_else(|| {
            panic!(
                "{} component `{component_name}` should resolve root `{}`",
                path.display(),
                component.root
            )
        })
        .clone()
}

fn numeric_prop(node: &UiV2NodeDefinition, name: &str) -> f64 {
    node.props
        .get(name)
        .and_then(Value::as_float)
        .unwrap_or_else(|| panic!("node should declare numeric prop `{name}`"))
}

fn layout_axis_value(node: &UiV2NodeDefinition, axis: &str, name: &str) -> f64 {
    node.layout
        .as_ref()
        .and_then(|layout| layout.get(axis))
        .and_then(Value::as_table)
        .and_then(|range| range.get(name))
        .and_then(Value::as_float)
        .unwrap_or_else(|| panic!("node should declare layout.{axis}.{name}"))
}

fn assert_near(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= EPSILON,
        "{label} should be {expected}, got {actual}"
    );
}

fn assert_height_contract(
    node: &UiV2NodeDefinition,
    expected_min: f64,
    expected_preferred: f64,
    expected_max: f64,
    label: &str,
) {
    assert_near(
        numeric_prop(node, "layout_min_height"),
        expected_min,
        &format!("{label}.layout_min_height"),
    );
    for (name, expected) in [
        ("min", expected_min),
        ("preferred", expected_preferred),
        ("max", expected_max),
    ] {
        assert_near(
            layout_axis_value(node, "height", name),
            expected,
            &format!("{label}.layout.height.{name}"),
        );
    }
}

#[test]
fn authored_workbench_buttons_match_compact_unreal_density() {
    let button = root_node("workbench_button.zui", "WorkbenchButton");
    for (name, expected) in [
        ("layout_padding_left", 12.0),
        ("layout_padding_right", 12.0),
        ("layout_padding_top", 6.0),
        ("layout_padding_bottom", 6.0),
        ("layout_spacing", 7.0),
        ("layout_icon_size", 16.0),
    ] {
        assert_near(numeric_prop(&button, name), expected, name);
    }
    assert_height_contract(&button, 28.0, 30.0, 32.0, "WorkbenchButton");

    let icon_button = root_node("workbench_icon_button.zui", "WorkbenchIconButton");
    assert_near(
        numeric_prop(&icon_button, "layout_icon_size"),
        16.0,
        "WorkbenchIconButton.layout_icon_size",
    );
    assert_near(
        numeric_prop(&icon_button, "layout_min_width"),
        32.0,
        "WorkbenchIconButton.layout_min_width",
    );
    assert_height_contract(&icon_button, 32.0, 32.0, 40.0, "WorkbenchIconButton");
    assert_near(
        layout_axis_value(&icon_button, "width", "min"),
        32.0,
        "WorkbenchIconButton.layout.width.min",
    );
    assert_near(
        layout_axis_value(&icon_button, "width", "preferred"),
        32.0,
        "WorkbenchIconButton.layout.width.preferred",
    );
    assert_near(
        layout_axis_value(&icon_button, "width", "max"),
        40.0,
        "WorkbenchIconButton.layout.width.max",
    );
}

#[test]
fn authored_workbench_fields_match_unreal_input_padding_and_height() {
    for (file_name, component_name) in [
        ("workbench_field.zui", "WorkbenchField"),
        ("workbench_dropdown.zui", "WorkbenchDropdown"),
        ("workbench_number_field.zui", "WorkbenchNumberField"),
    ] {
        let node = root_node(file_name, component_name);
        for (name, expected) in [
            ("layout_padding_left", 8.0),
            ("layout_padding_right", 8.0),
            ("layout_padding_top", 3.0),
            ("layout_padding_bottom", 4.0),
        ] {
            assert_near(
                numeric_prop(&node, name),
                expected,
                &format!("{component_name}.{name}"),
            );
        }
        assert_height_contract(&node, 28.0, 30.0, 32.0, component_name);
    }

    let search = root_node("workbench_search_input.zui", "WorkbenchSearchInput");
    assert_near(
        numeric_prop(&search, "layout_padding_top"),
        3.0,
        "WorkbenchSearchInput.layout_padding_top",
    );
    assert_near(
        numeric_prop(&search, "layout_padding_bottom"),
        4.0,
        "WorkbenchSearchInput.layout_padding_bottom",
    );
    assert_height_contract(&search, 28.0, 30.0, 32.0, "WorkbenchSearchInput");
}

#[test]
fn authored_workbench_tabs_use_slate_padding_and_theme_surface() {
    let tab = root_node("workbench_tab.zui", "WorkbenchTab");
    for (name, expected) in [
        ("layout_padding_left", 4.0),
        ("layout_padding_right", 10.0),
        ("layout_padding_top", 3.0),
        ("layout_padding_bottom", 4.0),
        ("layout_spacing", 4.0),
        ("layout_min_height", 28.0),
    ] {
        assert_near(
            numeric_prop(&tab, name),
            expected,
            &format!("WorkbenchTab.{name}"),
        );
    }

    let tab_strip = root_node("workbench_tab_strip.zui", "WorkbenchTabStrip");
    assert_height_contract(&tab_strip, 28.0, 30.0, 32.0, "WorkbenchTabStrip");
    assert!(
        !tab_strip.props.contains_key("background_color"),
        "WorkbenchTabStrip should inherit its central theme surface"
    );

    let segmented = root_node(
        "workbench_segmented_control.zui",
        "WorkbenchSegmentedControl",
    );
    assert_height_contract(&segmented, 28.0, 30.0, 32.0, "WorkbenchSegmentedControl");
}

#[test]
fn authored_workbench_data_rows_share_the_compact_row_height() {
    for (file_name, component_name) in [
        ("workbench_list_row.zui", "WorkbenchListRow"),
        ("workbench_tree_row.zui", "WorkbenchTreeRow"),
        ("workbench_table_row.zui", "WorkbenchTableRow"),
        ("workbench_property_row.zui", "WorkbenchPropertyRow"),
        (
            "workbench_component_property_row.zui",
            "WorkbenchComponentPropertyRow",
        ),
    ] {
        let node = root_node_under(DATA_ROOT, file_name, component_name);
        assert_height_contract(&node, 28.0, 28.0, 28.0, component_name);
    }

    for (file_name, component_name) in [
        ("workbench_list_row.zui", "WorkbenchListRow"),
        ("workbench_tree_row.zui", "WorkbenchTreeRow"),
        ("workbench_table_row.zui", "WorkbenchTableRow"),
        ("workbench_property_row.zui", "WorkbenchPropertyRow"),
    ] {
        let node = root_node_under(DATA_ROOT, file_name, component_name);
        for (name, expected) in [
            ("layout_padding_left", 8.0),
            ("layout_padding_right", 8.0),
            ("layout_spacing", 4.0),
        ] {
            assert_near(
                numeric_prop(&node, name),
                expected,
                &format!("{component_name}.{name}"),
            );
        }
    }
}
