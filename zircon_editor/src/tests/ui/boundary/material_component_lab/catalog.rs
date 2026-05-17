use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::ui::component::UiComponentDescriptorRegistry;

use super::support::*;

const DEMO_ONLY_PROTOTYPES: &[&str] = &["about_the_lab"];

const PROTOTYPE_CATALOG_DESCRIPTORS: &[(&str, &[&str])] = &[
    ("accordion", &["Accordion"]),
    ("alert", &["Alert"]),
    ("app_bar", &["AppBar"]),
    ("autocomplete", &["Autocomplete"]),
    ("avatars", &["Avatar", "AvatarGroup"]),
    ("backdrop", &["Backdrop"]),
    ("badges", &["Badge"]),
    ("bottom_navigation", &["BottomNavigation"]),
    ("box", &["Box"]),
    ("breadcrumbs", &["Breadcrumbs"]),
    ("button_group", &["ButtonGroup"]),
    ("buttons", &["Button"]),
    ("cards", &["Card"]),
    ("checkboxes", &["Checkbox"]),
    ("chips", &["Chip"]),
    ("click_away_listener", &["ClickAwayListener"]),
    ("container", &["Container"]),
    ("css_baseline", &["CssBaseline"]),
    ("dialogs", &["Dialog"]),
    ("dividers", &["Divider"]),
    ("drawers", &["Drawer"]),
    ("floating_action_button", &["FloatingActionButton"]),
    ("grid", &["Grid"]),
    ("icons", &["Icon"]),
    ("image_list", &["ImageList"]),
    ("init_color_scheme_script", &["InitColorSchemeScript"]),
    ("links", &["Link"]),
    ("lists", &["List"]),
    ("masonry", &["Masonry"]),
    ("material_icons", &["SvgIcon"]),
    ("menubar", &["Menubar"]),
    ("menus", &["Menu"]),
    ("modal", &["Modal"]),
    ("mui_x_agent_chat", &["AgentChat"]),
    ("mui_x_bar_chart", &["BarChart"]),
    ("mui_x_charts", &["Charts"]),
    ("mui_x_chat_composer", &["ChatComposer"]),
    ("mui_x_data_grid", &["DataGrid"]),
    ("mui_x_date_time_pickers", &["DateTimePickers"]),
    ("mui_x_gauge", &["Gauge"]),
    ("mui_x_line_chart", &["LineChart"]),
    ("mui_x_pie_chart", &["PieChart"]),
    ("mui_x_sparkline", &["SparkLineChart"]),
    ("mui_x_tree_view", &["MaterialTreeView"]),
    ("no_ssr", &["NoSsr"]),
    ("number_field", &["NumberField"]),
    ("pagination", &["Pagination"]),
    ("paper", &["Paper"]),
    ("popover", &["Popover"]),
    ("popper", &["Popper"]),
    ("portal", &["Portal"]),
    ("progress", &["Progress"]),
    ("radio_buttons", &["Radio"]),
    ("rating", &["Rating"]),
    ("selects", &["Select"]),
    ("skeleton", &["Skeleton"]),
    ("slider", &["Slider"]),
    ("snackbars", &["Snackbar"]),
    ("speed_dial", &["SpeedDial"]),
    ("stack", &["Stack"]),
    ("steppers", &["Stepper"]),
    ("switches", &["Switch"]),
    ("table", &["Table"]),
    ("tabs", &["Tabs"]),
    ("text_fields", &["TextField"]),
    ("textarea_autosize", &["TextareaAutosize"]),
    ("timeline", &["Timeline"]),
    ("toggle_button", &["ToggleButton"]),
    ("tooltips", &["Tooltip"]),
    ("transfer_list", &["TransferList"]),
    (
        "transitions",
        &["Collapse", "Fade", "Grow", "Slide", "Zoom"],
    ),
    ("typography", &["Typography"]),
    ("use_media_query", &["UseMediaQuery"]),
];

#[test]
fn material_component_lab_prototypes_map_to_foundation_catalog_descriptors() {
    let prototype_keys = material_prototype_files()
        .iter()
        .map(|path| {
            path.file_stem()
                .and_then(|name| name.to_str())
                .expect("prototype file stem is UTF-8")
                .strip_prefix("material_")
                .expect("prototype files use material_ prefix")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let mapped_keys = PROTOTYPE_CATALOG_DESCRIPTORS
        .iter()
        .map(|(key, _)| (*key).to_string())
        .collect::<BTreeSet<_>>();
    let demo_only_keys = DEMO_ONLY_PROTOTYPES
        .iter()
        .map(|key| (*key).to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        mapped_keys.union(&demo_only_keys).cloned().collect::<BTreeSet<_>>(),
        prototype_keys,
        "every Material Lab prototype should be explicitly mapped to catalog descriptors or marked demo-only"
    );
    assert_eq!(
        mapped_keys.intersection(&demo_only_keys).count(),
        0,
        "demo-only prototypes should not also claim runtime catalog descriptors"
    );

    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    for (prototype_key, descriptor_ids) in PROTOTYPE_CATALOG_DESCRIPTORS {
        for descriptor_id in *descriptor_ids {
            let descriptor = registry.descriptor(descriptor_id).unwrap_or_else(|| {
                panic!(
                    "material_{prototype_key}.zui should map to existing Material foundation descriptor `{descriptor_id}`"
                )
            });
            for prop in [
                "density",
                "surface_variant",
                "corner_radius",
                "border_width",
            ] {
                assert!(
                    descriptor.prop(prop).is_some(),
                    "`{descriptor_id}` should expose shared Material prop `{prop}` for material_{prototype_key}.zui"
                );
            }
            for state in ["hovered", "pressed", "focused", "selected", "disabled"] {
                assert!(
                    descriptor.state_prop(state).is_some(),
                    "`{descriptor_id}` should expose shared Material state `{state}` for material_{prototype_key}.zui"
                );
            }
        }
    }
}

#[test]
fn material_component_lab_catalog_mapping_keeps_expected_mui_x_descriptors() {
    let mapping = PROTOTYPE_CATALOG_DESCRIPTORS
        .iter()
        .map(|(key, descriptors)| (*key, descriptors.iter().copied().collect::<BTreeSet<_>>()))
        .collect::<BTreeMap<_, _>>();

    for (prototype_key, descriptor_id) in [
        ("mui_x_tree_view", "MaterialTreeView"),
        ("mui_x_data_grid", "DataGrid"),
        ("mui_x_date_time_pickers", "DateTimePickers"),
        ("mui_x_charts", "Charts"),
        ("mui_x_line_chart", "LineChart"),
        ("mui_x_bar_chart", "BarChart"),
        ("mui_x_pie_chart", "PieChart"),
        ("mui_x_sparkline", "SparkLineChart"),
        ("mui_x_gauge", "Gauge"),
        ("mui_x_agent_chat", "AgentChat"),
        ("mui_x_chat_composer", "ChatComposer"),
    ] {
        assert!(
            mapping
                .get(prototype_key)
                .is_some_and(|descriptors| descriptors.contains(descriptor_id)),
            "MUI X prototype `{prototype_key}` should stay mapped to `{descriptor_id}`"
        );
    }
}
