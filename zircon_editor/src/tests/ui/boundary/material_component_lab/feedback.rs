use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

use zircon_runtime::ui::v2::UiZuiAssetLoader;

use crate::ui::binding::{EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::template_runtime::builtin::builtin_template_bindings;

use super::support::*;

const STRUCTURAL_CHILD_ROUTE_PROTOTYPES: &[&str] = &["button_group"];

const EXPECTED_INTERACTION_FEEDBACK: &[(&str, &str, &str, &str)] = &[
    (
        "accordion",
        "MaterialLab/Accordion/Toggle",
        "Toggle",
        "material_lab.accordion.toggle",
    ),
    (
        "autocomplete",
        "MaterialLab/Autocomplete/Change",
        "Change",
        "material_lab.autocomplete.change",
    ),
    (
        "backdrop",
        "MaterialLab/Backdrop/Click",
        "Click",
        "material_lab.backdrop.click",
    ),
    (
        "bottom_navigation",
        "MaterialLab/BottomNavigation/Change",
        "Change",
        "material_lab.bottom_navigation.change",
    ),
    (
        "breadcrumbs",
        "MaterialLab/Breadcrumbs/Click",
        "Click",
        "material_lab.breadcrumbs.click",
    ),
    (
        "buttons",
        "MaterialLab/Buttons/Click",
        "Click",
        "material_lab.buttons.click",
    ),
    (
        "cards",
        "MaterialLab/Cards/Click",
        "Click",
        "material_lab.cards.click",
    ),
    (
        "checkboxes",
        "MaterialLab/Checkboxes/Toggle",
        "Toggle",
        "material_lab.checkboxes.toggle",
    ),
    (
        "chips",
        "MaterialLab/Chips/Click",
        "Click",
        "material_lab.chips.click",
    ),
    (
        "dialogs",
        "MaterialLab/Dialogs/Click",
        "Click",
        "material_lab.dialogs.click",
    ),
    (
        "drawers",
        "MaterialLab/Drawers/Click",
        "Click",
        "material_lab.drawers.click",
    ),
    (
        "floating_action_button",
        "MaterialLab/FloatingActionButton/Click",
        "Click",
        "material_lab.floating_action_button.click",
    ),
    (
        "image_list",
        "MaterialLab/ImageList/Click",
        "Click",
        "material_lab.image_list.click",
    ),
    (
        "links",
        "MaterialLab/Links/Click",
        "Click",
        "material_lab.links.click",
    ),
    (
        "lists",
        "MaterialLab/Lists/Click",
        "Click",
        "material_lab.lists.click",
    ),
    (
        "menubar",
        "MaterialLab/Menubar/Click",
        "Click",
        "material_lab.menubar.click",
    ),
    (
        "menus",
        "MaterialLab/Menus/Click",
        "Click",
        "material_lab.menus.click",
    ),
    (
        "modal",
        "MaterialLab/Modal/Click",
        "Click",
        "material_lab.modal.click",
    ),
    (
        "mui_x_agent_chat",
        "MaterialLab/MuiXAgentChat/Submit",
        "Submit",
        "material_lab.mui_x_agent_chat.submit",
    ),
    (
        "mui_x_bar_chart",
        "MaterialLab/MuiXBarChart/Hover",
        "Hover",
        "material_lab.mui_x_bar_chart.hover",
    ),
    (
        "mui_x_charts",
        "MaterialLab/MuiXCharts/Hover",
        "Hover",
        "material_lab.mui_x_charts.hover",
    ),
    (
        "mui_x_chat_composer",
        "MaterialLab/MuiXChatComposer/Submit",
        "Submit",
        "material_lab.mui_x_chat_composer.submit",
    ),
    (
        "mui_x_data_grid",
        "MaterialLab/MuiXDataGrid/Click",
        "Click",
        "material_lab.mui_x_data_grid.click",
    ),
    (
        "mui_x_date_time_pickers",
        "MaterialLab/MuiXDateTimePickers/Submit",
        "Submit",
        "material_lab.mui_x_date_time_pickers.submit",
    ),
    (
        "mui_x_gauge",
        "MaterialLab/MuiXGauge/Hover",
        "Hover",
        "material_lab.mui_x_gauge.hover",
    ),
    (
        "mui_x_line_chart",
        "MaterialLab/MuiXLineChart/Hover",
        "Hover",
        "material_lab.mui_x_line_chart.hover",
    ),
    (
        "mui_x_pie_chart",
        "MaterialLab/MuiXPieChart/Hover",
        "Hover",
        "material_lab.mui_x_pie_chart.hover",
    ),
    (
        "mui_x_sparkline",
        "MaterialLab/MuiXSparkline/Hover",
        "Hover",
        "material_lab.mui_x_sparkline.hover",
    ),
    (
        "mui_x_tree_view",
        "MaterialLab/MuiXTreeView/Toggle",
        "Toggle",
        "material_lab.mui_x_tree_view.toggle",
    ),
    (
        "number_field",
        "MaterialLab/NumberField/Change",
        "Change",
        "material_lab.number_field.change",
    ),
    (
        "pagination",
        "MaterialLab/Pagination/Change",
        "Change",
        "material_lab.pagination.change",
    ),
    (
        "popover",
        "MaterialLab/Popover/Click",
        "Click",
        "material_lab.popover.click",
    ),
    (
        "popper",
        "MaterialLab/Popper/Click",
        "Click",
        "material_lab.popper.click",
    ),
    (
        "radio_buttons",
        "MaterialLab/RadioButtons/Change",
        "Change",
        "material_lab.radio_buttons.change",
    ),
    (
        "rating",
        "MaterialLab/Rating/Change",
        "Change",
        "material_lab.rating.change",
    ),
    (
        "selects",
        "MaterialLab/Selects/Change",
        "Change",
        "material_lab.selects.change",
    ),
    (
        "slider",
        "MaterialLab/Slider/DragUpdate",
        "DragUpdate",
        "material_lab.slider.drag_update",
    ),
    (
        "snackbars",
        "MaterialLab/Snackbars/Click",
        "Click",
        "material_lab.snackbars.click",
    ),
    (
        "speed_dial",
        "MaterialLab/SpeedDial/Click",
        "Click",
        "material_lab.speed_dial.click",
    ),
    (
        "steppers",
        "MaterialLab/Steppers/Change",
        "Change",
        "material_lab.steppers.change",
    ),
    (
        "switches",
        "MaterialLab/Switches/Toggle",
        "Toggle",
        "material_lab.switches.toggle",
    ),
    (
        "table",
        "MaterialLab/Table/Click",
        "Click",
        "material_lab.table.click",
    ),
    (
        "tabs",
        "MaterialLab/Tabs/Change",
        "Change",
        "material_lab.tabs.change",
    ),
    (
        "text_fields",
        "MaterialLab/TextFields/Change",
        "Change",
        "material_lab.text_fields.change",
    ),
    (
        "textarea_autosize",
        "MaterialLab/TextareaAutosize/Change",
        "Change",
        "material_lab.textarea_autosize.change",
    ),
    (
        "toggle_button",
        "MaterialLab/ToggleButton/Toggle",
        "Toggle",
        "material_lab.toggle_button.toggle",
    ),
    (
        "tooltips",
        "MaterialLab/Tooltips/Hover",
        "Hover",
        "material_lab.tooltips.hover",
    ),
    (
        "transfer_list",
        "MaterialLab/TransferList/Change",
        "Change",
        "material_lab.transfer_list.change",
    ),
];

#[test]
fn material_component_lab_mui_x_prototypes_define_feedback_routes() {
    for key in MUI_X_PROTOTYPES {
        let path = material_prototype_path(key);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        assert!(
            source.contains("MaterialLab/") && source.contains("route = \"material_lab."),
            "{} should define a Material Lab feedback route for MUI X interaction evidence",
            path.display()
        );
    }
}

#[test]
fn material_component_lab_non_route_prototypes_are_not_dispatchable_controls() {
    for path in material_prototype_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        let key = prototype_key(&path);
        if source.contains("MaterialLab/") || STRUCTURAL_CHILD_ROUTE_PROTOTYPES.contains(&key) {
            continue;
        }
        for input_flag in [
            "input_interactive = true",
            "input_clickable = true",
            "input_hoverable = true",
            "input_focusable = true",
        ] {
            assert!(
                !source.contains(input_flag),
                "{} should not expose `{input_flag}` without a MaterialLab feedback route",
                path.display()
            );
        }
    }
}

#[test]
fn material_component_lab_route_prototypes_are_dispatchable_controls() {
    for path in material_prototype_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        let key = prototype_key(&path);
        if !source.contains("MaterialLab/") && !STRUCTURAL_CHILD_ROUTE_PROTOTYPES.contains(&key) {
            continue;
        }
        assert!(
            [
                "input_interactive = true",
                "input_clickable = true",
                "input_hoverable = true",
                "input_focusable = true",
            ]
            .iter()
            .any(|input_flag| source.contains(input_flag)),
            "{} should expose a dispatchable input flag when it defines a MaterialLab feedback route",
            path.display()
        );
    }
}

#[test]
fn material_component_lab_feedback_routes_live_on_dispatchable_sample_nodes() {
    for path in material_prototype_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
        let feedback_nodes = document
            .nodes
            .iter()
            .filter(|(_, node)| {
                node.events
                    .iter()
                    .any(|event| event.id.starts_with("MaterialLab/"))
            })
            .collect::<Vec<_>>();

        if !source.contains("MaterialLab/") {
            assert!(
                feedback_nodes.is_empty(),
                "{} should not hide Material Lab feedback events on non-route prototypes",
                path.display()
            );
            continue;
        }

        assert_eq!(
            feedback_nodes.len(),
            1,
            "{} should expose exactly one Material Lab feedback node",
            path.display()
        );
        let (node_id, node) = feedback_nodes[0];
        assert!(
            node.classes
                .iter()
                .any(|class| class == "material-lab-sample"),
            "{} node `{node_id}` should attach feedback to the visible sample node",
            path.display()
        );
        assert_eq!(
            node.events
                .iter()
                .filter(|event| event.id.starts_with("MaterialLab/"))
                .count(),
            1,
            "{} node `{node_id}` should keep one representative feedback route",
            path.display()
        );
        for prop in [
            "input_interactive",
            "input_clickable",
            "input_hoverable",
            "input_focusable",
        ] {
            assert_eq!(
                node.props.get(prop).and_then(|value| value.as_bool()),
                Some(true),
                "{} node `{node_id}` should keep `{prop} = true` for feedback capture",
                path.display()
            );
        }
    }
}

#[test]
fn material_component_lab_feedback_route_inventory_matches_expected_interactions() {
    let expected = EXPECTED_INTERACTION_FEEDBACK
        .iter()
        .map(|(key, id, event_kind, route)| {
            (
                (*key).to_string(),
                (
                    (*id).to_string(),
                    (*event_kind).to_string(),
                    (*route).to_string(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut actual = BTreeMap::new();

    for path in material_prototype_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
        let feedback_events = document
            .nodes
            .values()
            .flat_map(|node| node.events.iter())
            .filter(|event| event.id.starts_with("MaterialLab/"))
            .collect::<Vec<_>>();
        if feedback_events.is_empty() {
            continue;
        }
        assert_eq!(
            feedback_events.len(),
            1,
            "{} should define exactly one representative Material Lab event",
            path.display()
        );
        let key = path
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("prototype file stem is UTF-8")
            .strip_prefix("material_")
            .expect("prototype files use material_ prefix")
            .to_string();
        let event = feedback_events[0];
        actual.insert(
            key,
            (
                event.id.clone(),
                format!("{:?}", event.event),
                event.route.clone().unwrap_or_default(),
            ),
        );
    }

    assert_eq!(
        actual, expected,
        "Material Lab feedback inventory should keep each component's representative interaction semantics"
    );
}

#[test]
fn material_button_group_uses_child_segment_routes_without_sample_feedback_route() {
    let path =
        editor_asset("assets/ui/editor/material_components/inputs/material_button_group.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define the ButtonGroup sample", path.display()));

    assert!(
        sample.events.is_empty(),
        "ButtonGroup sample should stay structural; child segments own click routes"
    );
    for prop in [
        "input_interactive",
        "input_clickable",
        "input_hoverable",
        "input_focusable",
    ] {
        assert_eq!(
            sample.props.get(prop).and_then(|value| value.as_bool()),
            Some(false),
            "ButtonGroup sample should keep `{prop} = false` because it is structural"
        );
    }

    for (node_id, expected_id, expected_route) in [
        (
            "segment_first",
            "ButtonGroup/First/Click",
            "button_group.first.click",
        ),
        (
            "segment_middle",
            "ButtonGroup/Middle/Click",
            "button_group.middle.click",
        ),
        (
            "segment_last",
            "ButtonGroup/Last/Click",
            "button_group.last.click",
        ),
    ] {
        let segment = document
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("{} should define `{node_id}`", path.display()));
        assert_eq!(
            segment.component.as_str(),
            "Button",
            "ButtonGroup segment `{node_id}` should remain a child Button"
        );
        assert!(
            segment
                .classes
                .iter()
                .any(|class| class == "material-button-group-segment"),
            "ButtonGroup segment `{node_id}` should keep the shared segment class"
        );
        for prop in [
            "input_interactive",
            "input_clickable",
            "input_hoverable",
            "input_focusable",
        ] {
            assert_eq!(
                segment.props.get(prop).and_then(|value| value.as_bool()),
                Some(true),
                "ButtonGroup segment `{node_id}` should keep `{prop} = true`"
            );
        }
        assert_eq!(
            segment.events.len(),
            1,
            "ButtonGroup segment `{node_id}` should expose exactly one click route"
        );
        let event = &segment.events[0];
        assert_eq!(event.id, expected_id);
        assert_eq!(format!("{:?}", event.event), "Click");
        assert_eq!(event.route.as_deref(), Some(expected_route));
        assert!(
            !event.id.starts_with("MaterialLab/"),
            "ButtonGroup child segment routes should not become Material Lab feedback routes"
        );
    }
}

#[test]
fn material_button_group_child_routes_are_registered_as_structural_builtin_bindings() {
    let bindings = builtin_template_bindings();

    for (binding_id, control_id) in [
        ("ButtonGroup/First/Click", "button-group_SegmentFirst"),
        ("ButtonGroup/Middle/Click", "button-group_SegmentMiddle"),
        ("ButtonGroup/Last/Click", "button-group_SegmentLast"),
    ] {
        let binding = bindings
            .get(binding_id)
            .unwrap_or_else(|| panic!("`{binding_id}` should be a builtin structural binding"));
        assert_eq!(binding.path().view_id, "MaterialComponentLab");
        assert_eq!(binding.path().control_id, control_id);
        assert_eq!(binding.path().event_kind, EditorUiEventKind::Click);
        let EditorUiBindingPayload::Custom(call) = binding.payload() else {
            panic!("`{binding_id}` should use the Material Component Lab feedback payload");
        };
        assert_eq!(call.symbol, "MaterialComponentLab");
        assert!(
            !binding_id.starts_with("MaterialLab/"),
            "`{binding_id}` should stay outside the representative Material Lab feedback inventory"
        );
    }
}

#[test]
fn material_floating_action_button_sample_covers_circular_small_and_extended_shapes() {
    let path = editor_asset(
        "assets/ui/editor/material_components/inputs/material_floating_action_button.zui",
    );
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define the FAB sample", path.display()));

    assert_eq!(sample.component.as_str(), "HorizontalBox");
    assert!(sample
        .classes
        .iter()
        .any(|class| class == "material-lab-sample"));
    assert_eq!(sample.events.len(), 1);
    assert_eq!(
        sample.events[0].id,
        "MaterialLab/FloatingActionButton/Click"
    );
    for prop in [
        "input_interactive",
        "input_clickable",
        "input_hoverable",
        "input_focusable",
    ] {
        assert_eq!(
            sample.props.get(prop).and_then(|value| value.as_bool()),
            Some(true),
            "FAB sample should keep `{prop} = true` for representative feedback capture"
        );
    }

    for (node_id, component, shape, size, icon_placement, fab_style, corner_radius, elevation) in [
        (
            "fab_circular",
            "IconButton",
            "circular",
            "medium",
            "icon_only",
            "standard",
            16.0,
            3.0,
        ),
        (
            "fab_small",
            "IconButton",
            "circular",
            "small",
            "icon_only",
            "small",
            12.0,
            3.0,
        ),
        (
            "fab_extended",
            "Button",
            "extended",
            "medium",
            "leading",
            "large",
            28.0,
            3.0,
        ),
    ] {
        let node = document
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("{} should define `{node_id}`", path.display()));
        assert_eq!(node.component.as_str(), component);
        assert_eq!(
            node.props
                .get("button_shape")
                .and_then(|value| value.as_str()),
            Some(shape),
            "FAB `{node_id}` should freeze button_shape metadata"
        );
        assert_eq!(
            node.props
                .get("button_size")
                .and_then(|value| value.as_str()),
            Some(size),
            "FAB `{node_id}` should freeze button_size metadata"
        );
        assert_eq!(
            node.props
                .get("icon_placement")
                .and_then(|value| value.as_str()),
            Some(icon_placement),
            "FAB `{node_id}` should freeze icon placement metadata"
        );
        assert_eq!(
            node.props.get("fab_style").and_then(|value| value.as_str()),
            Some(fab_style),
            "FAB `{node_id}` should freeze Slint FABStyle metadata"
        );
        assert_eq!(
            numeric_prop(node.props.get("corner_radius")),
            Some(corner_radius),
            "FAB `{node_id}` should use source-derived FABStyle radius"
        );
        assert_eq!(
            numeric_prop(node.props.get("elevation")),
            Some(elevation),
            "FAB `{node_id}` should carry source-derived elevation metadata"
        );
        assert!(
            node.events.is_empty(),
            "FAB child examples should not add extra representative feedback routes"
        );
        for prop in [
            "input_interactive",
            "input_clickable",
            "input_hoverable",
            "input_focusable",
        ] {
            assert_eq!(
                node.props.get(prop).and_then(|value| value.as_bool()),
                Some(false),
                "FAB child `{node_id}` should leave dispatchability on the visible sample"
            );
        }
    }
}

#[test]
fn material_component_lab_feedback_events_use_consistent_ids_routes_and_kinds() {
    for path in material_prototype_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));

        for node in document.nodes.values() {
            for event in &node.events {
                if !event.id.starts_with("MaterialLab/") {
                    continue;
                }
                let expected_route = event.id.replace('/', ".");
                assert_eq!(
                    event.route.as_deref(),
                    Some(expected_route.as_str()),
                    "{} should keep `{}` route aligned with its event id",
                    path.display(),
                    event.id
                );
                let (_, event_tail) = event
                    .id
                    .rsplit_once('/')
                    .unwrap_or_else(|| panic!("{} should end with an event kind", event.id));
                assert_eq!(
                    event_tail,
                    format!("{:?}", event.event),
                    "{} should keep `{}` event kind aligned with its id tail",
                    path.display(),
                    event.id
                );
            }
        }
    }
}

#[test]
fn material_component_lab_feedback_events_are_registered_as_builtin_bindings() {
    let binding_path =
        editor_asset("src/ui/template_runtime/builtin/material_lab_template_bindings.rs");
    let binding_source = fs::read_to_string(&binding_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", binding_path.display()));

    assert!(
        binding_source.contains("MATERIAL_LAB_BINDING_SPECS"),
        "Material Lab bindings should stay centralized in the Material Lab binding registry"
    );
    for (binding_id, event_kind) in material_lab_event_specs() {
        assert!(
            binding_source.contains(&format!("\"{binding_id}\"")),
            "{} should register `{binding_id}` as a builtin Material Lab binding",
            binding_path.display()
        );
        let binding_offset = binding_source
            .find(&format!("\"{binding_id}\""))
            .unwrap_or_else(|| panic!("`{binding_id}` should be present in binding source"));
        let binding_snippet =
            &binding_source[binding_offset..(binding_offset + 160).min(binding_source.len())];
        assert!(
            binding_snippet.contains(&format!("EditorUiEventKind::{event_kind}")),
            "{} should register `{binding_id}` with EditorUiEventKind::{event_kind}",
            binding_path.display()
        );
    }
}

#[test]
fn material_component_lab_interactive_inventory_matches_route_bearing_prototypes() {
    let expected = INTERACTIVE_PROTOTYPES
        .iter()
        .map(|key| (*key).to_string())
        .collect::<BTreeSet<_>>();
    let actual = material_prototype_files()
        .iter()
        .filter_map(|path| {
            let source = fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
            source
                .contains("MaterialLab/")
                .then(|| prototype_key(path).to_string())
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(
        actual, expected,
        "interactive prototype inventory should match route-bearing Material Lab prototypes"
    );
}

fn prototype_key(path: &std::path::Path) -> &str {
    path.file_stem()
        .and_then(|name| name.to_str())
        .expect("prototype file stem is UTF-8")
        .strip_prefix("material_")
        .expect("prototype files use material_ prefix")
}
