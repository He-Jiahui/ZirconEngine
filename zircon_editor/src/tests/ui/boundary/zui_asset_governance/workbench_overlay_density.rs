use toml::Value;
use zircon_runtime_interface::ui::v2::UiV2NodeDefinition;

use super::support::{editor_asset_root, load_zui_document};

const FEEDBACK_ROOT: &str = "ui/editor/components/workbench/primitives/feedback";
const EPSILON: f64 = 0.000_1;

fn root_node(file_name: &str, component_name: &str) -> UiV2NodeDefinition {
    let path = editor_asset_root().join(FEEDBACK_ROOT).join(file_name);
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

fn assert_near(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= EPSILON,
        "{label} should be {expected}, got {actual}"
    );
}

#[test]
fn authored_workbench_menus_share_slate_popup_density_and_flat_surface() {
    for (file_name, component_name) in [
        ("workbench_context_menu.zui", "WorkbenchContextMenu"),
        ("workbench_dropdown_popup.zui", "WorkbenchDropdownPopup"),
        ("workbench_popup_menu.zui", "WorkbenchPopupMenu"),
    ] {
        let node = root_node(file_name, component_name);
        for (name, expected) in [
            ("layout_padding_left", 8.0),
            ("layout_padding_right", 8.0),
            ("layout_padding_top", 3.0),
            ("layout_padding_bottom", 3.0),
            ("layout_spacing", 4.0),
            ("layout_min_height", 28.0),
            ("border_width", 1.0),
            ("corner_radius", 4.0),
            ("elevation", 0.0),
        ] {
            assert_near(
                numeric_prop(&node, name),
                expected,
                &format!("{component_name}.{name}"),
            );
        }
        assert!(
            !node.props.contains_key("box_shadow"),
            "{component_name} should not declare a shadow"
        );
    }
}
