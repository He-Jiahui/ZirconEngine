use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

use zircon_runtime::ui::v2::{UiV2AssetLoader, UiZuiAssetLoader};
use zircon_runtime_interface::ui::v2::{UiV2AssetDocument, UiV2AssetKind};

use super::support::*;

#[test]
fn material_component_lab_view_loads_and_imports_every_component_prototype() {
    let lab_path = editor_asset("assets/ui/editor/material_component_lab.v2.ui.toml");
    let lab = UiV2AssetLoader::load_toml_file(&lab_path).unwrap_or_else(|error| {
        panic!(
            "Material Component Lab should load as runtime UI v2 from {}: {error}",
            lab_path.display()
        )
    });
    assert_eq!(lab.asset.kind, UiV2AssetKind::View);
    assert_eq!(lab.root_node_id(), Some("material_lab_root"));

    let imports = lab
        .imports
        .widgets
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let prototype_files = material_prototype_files();
    assert_eq!(
        imports.len(),
        prototype_files.len(),
        "lab should import every material_*.zui prototype exactly once"
    );
    for path in &prototype_files {
        let source_name = path.file_name().unwrap().to_string_lossy();
        let source = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
        let component_name = document
            .components
            .keys()
            .next()
            .expect("prototype declares one component");
        let import = format!("res://ui/editor/material_components/{source_name}#{component_name}");
        assert!(
            imports.contains(import.as_str()),
            "lab should import `{import}`"
        );
    }

    for label in [
        "Inputs",
        "Data Display",
        "Feedback",
        "Surfaces",
        "Navigation",
        "Layout / Utils",
        "MUI X",
        "Interaction Feedback",
    ] {
        assert!(
            source_contains(&lab_path, label),
            "lab view should expose the `{label}` section"
        );
    }
}

#[test]
fn material_component_lab_places_every_prototype_once_in_visible_sections() {
    let lab_path = editor_asset("assets/ui/editor/material_component_lab.v2.ui.toml");
    let lab = UiV2AssetLoader::load_toml_file(&lab_path).unwrap_or_else(|error| {
        panic!(
            "Material Component Lab should load as runtime UI v2 from {}: {error}",
            lab_path.display()
        )
    });

    let expected_components = material_prototype_component_names();
    let prototype_nodes = lab
        .nodes
        .iter()
        .filter_map(|(node_id, node)| {
            node_id
                .starts_with("prototype_")
                .then(|| (node_id.as_str(), node.component.as_str()))
        })
        .collect::<Vec<_>>();
    let placed_components = prototype_nodes
        .iter()
        .map(|(_, component)| (*component).to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        placed_components, expected_components,
        "Material Lab should visibly place every imported prototype component exactly once"
    );
    assert_eq!(
        prototype_nodes.len(),
        placed_components.len(),
        "Material Lab should not place duplicate prototype components"
    );

    let section_children = material_lab_section_ids()
        .iter()
        .flat_map(|section_id| child_nodes(&lab, section_id))
        .filter(|node_id| node_id.starts_with("prototype_"))
        .collect::<Vec<_>>();
    let section_child_set = section_children
        .iter()
        .map(|node_id| (*node_id).to_string())
        .collect::<BTreeSet<_>>();
    let prototype_node_set = prototype_nodes
        .iter()
        .map(|(node_id, _)| (*node_id).to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        section_child_set, prototype_node_set,
        "Material Lab prototype nodes should all be mounted under the visible component-family sections"
    );
    assert_eq!(
        section_children.len(),
        section_child_set.len(),
        "Material Lab should not mount the same prototype node in multiple visible sections"
    );
}

#[test]
fn material_component_lab_prototype_nodes_match_material_file_stems() {
    let lab_path = editor_asset("assets/ui/editor/material_component_lab.v2.ui.toml");
    let lab = UiV2AssetLoader::load_toml_file(&lab_path).unwrap_or_else(|error| {
        panic!(
            "Material Component Lab should load as runtime UI v2 from {}: {error}",
            lab_path.display()
        )
    });

    let expected = material_prototype_components_by_key();
    let actual = lab
        .nodes
        .iter()
        .filter_map(|(node_id, node)| {
            node_id
                .strip_prefix("prototype_")
                .map(|key| (key.to_string(), node.component.clone()))
        })
        .collect::<BTreeMap<_, _>>();

    assert_eq!(
        actual, expected,
        "Material Lab prototype node ids should mirror material_<component>.zui file stems and component names"
    );
}

#[test]
fn material_component_prototypes_cover_mui_core_and_mui_x_inventory() {
    let expected = mui_docs_keys()
        .into_iter()
        .map(|key| key.replace('-', "_"))
        .chain(MUI_X_PROTOTYPES.iter().map(|key| key.to_string()))
        .collect::<BTreeSet<_>>();
    let actual = material_prototype_files()
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

    assert_eq!(
        actual, expected,
        "material component prototypes must mirror MUI core docs plus explicit MUI X rows"
    );
}

#[test]
fn material_component_prototypes_are_single_component_zui_with_state_and_routes() {
    let interactive = INTERACTIVE_PROTOTYPES
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    for path in material_prototype_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
        assert_eq!(
            document.components.len(),
            1,
            "{} should stay a single-component prototype",
            path.display()
        );
        let component = document
            .components
            .values()
            .next()
            .expect("prototype component exists");
        assert!(
            document.nodes.contains_key(&component.root),
            "{} root node should exist",
            path.display()
        );
        for state in [
            "hovered",
            "pressed",
            "focused",
            "selected",
            "checked",
            "disabled",
            "open",
            "popup_open",
            "corner_radius",
            "border_width",
        ] {
            assert!(
                source.contains(state),
                "{} should carry Material state prop `{state}`",
                path.display()
            );
        }

        let key = path
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("prototype file stem is UTF-8")
            .strip_prefix("material_")
            .expect("prototype files use material_ prefix");
        if interactive.contains(key) {
            assert!(
                document.nodes.values().any(|node| {
                    node.events.iter().any(|event| {
                        event
                            .route
                            .as_deref()
                            .is_some_and(|route| route.starts_with("MaterialLab."))
                    })
                }),
                "{} should define an interaction feedback route",
                path.display()
            );
        }
    }
}

#[test]
fn material_component_prototype_sample_nodes_carry_typed_material_props() {
    for path in material_prototype_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
        let sample_nodes = document
            .nodes
            .iter()
            .filter(|(_, node)| {
                node.classes
                    .iter()
                    .any(|class| class == "material-lab-sample")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            sample_nodes.len(),
            1,
            "{} should expose exactly one rendered Material Lab sample node",
            path.display()
        );
        let (sample_id, sample) = sample_nodes[0];

        assert!(
            sample
                .classes
                .iter()
                .any(|class| class == "material-control"),
            "{} node `{sample_id}` should keep the shared material-control class",
            path.display()
        );
        let key = path
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("prototype file stem is UTF-8")
            .strip_prefix("material_")
            .expect("prototype files use material_ prefix");
        for (prop, expected) in [
            ("surface_variant", "elevated"),
            ("button_variant", "contained"),
            ("text_tone", "primary"),
            (
                "validation_level",
                if key == "alert" { "warning" } else { "normal" },
            ),
        ] {
            assert_eq!(
                sample.props.get(prop).and_then(|value| value.as_str()),
                Some(expected),
                "{} node `{sample_id}` should set `{prop}` to `{expected}`",
                path.display()
            );
        }
        for prop in [
            "hovered",
            "pressed",
            "focused",
            "selected",
            "checked",
            "disabled",
            "open",
            "popup_open",
            "input_interactive",
            "input_clickable",
            "input_hoverable",
            "input_focusable",
        ] {
            assert!(
                sample
                    .props
                    .get(prop)
                    .is_some_and(|value| value.as_bool().is_some()),
                "{} node `{sample_id}` should carry boolean Material prop `{prop}`",
                path.display()
            );
        }
        let expected_corner_radius = if key == "buttons" { 17.0 } else { 10.0 };
        for (prop, expected) in [
            ("corner_radius", expected_corner_radius),
            ("border_width", 1.0),
        ] {
            assert_eq!(
                numeric_prop(sample.props.get(prop)),
                Some(expected),
                "{} node `{sample_id}` should set numeric Material prop `{prop}` to `{expected}`",
                path.display()
            );
        }
    }
}

#[test]
fn material_component_prototype_roots_keep_card_layout_contract() {
    for path in material_prototype_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
        let component = document
            .components
            .values()
            .next()
            .expect("prototype component exists");
        let root = document
            .nodes
            .get(&component.root)
            .unwrap_or_else(|| panic!("{} root node should exist", path.display()));
        let key = path
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("prototype file stem is UTF-8")
            .strip_prefix("material_")
            .expect("prototype files use material_ prefix");

        assert_eq!(
            component
                .default_classes
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["material-lab-prototype", "material-surface"],
            "{} component should keep the shared prototype default classes",
            path.display()
        );
        assert_eq!(
            root.component.as_str(),
            "VerticalBox",
            "{} root should remain a vertical Material Lab card",
            path.display()
        );
        let component_class = format!("material-{key}");
        for class in ["material-lab-card", "material-surface", &component_class] {
            assert!(
                root.classes.iter().any(|root_class| root_class == class),
                "{} root should keep class `{class}`",
                path.display()
            );
        }

        assert_eq!(
            root.children
                .iter()
                .map(|child| child.node.as_str())
                .collect::<Vec<_>>(),
            vec!["title", "meta", "sample", "state_strip"],
            "{} root should keep title/meta/sample/state_strip child order",
            path.display()
        );
        assert_child_contract(
            &document,
            &path,
            "title",
            "Label",
            "material-lab-title",
            22.0,
        );
        assert_child_contract(
            &document,
            &path,
            "meta",
            "HorizontalBox",
            "material-lab-meta-strip",
            18.0,
        );
        assert_child_contract(
            &document,
            &path,
            "state_strip",
            "HorizontalBox",
            "material-lab-state-strip",
            20.0,
        );

        let layout = root
            .layout
            .as_ref()
            .unwrap_or_else(|| panic!("{} root should define layout", path.display()));
        let width = table_prop(layout, "width")
            .unwrap_or_else(|| panic!("{} root should define width layout", path.display()));
        assert_eq!(
            width.get("stretch").and_then(|value| value.as_str()),
            Some("Stretch"),
            "{} root width should stretch in the lab grid",
            path.display()
        );
        let height = table_prop(layout, "height")
            .unwrap_or_else(|| panic!("{} root should define height layout", path.display()));
        let expected_height = match key {
            "image_list" => [("min", 114.0), ("preferred", 128.0), ("max", 150.0)],
            "masonry" => [("min", 260.0), ("preferred", 268.0), ("max", 292.0)],
            _ => [("min", 104.0), ("preferred", 120.0), ("max", 140.0)],
        };
        for (prop, expected) in expected_height {
            assert_eq!(
                numeric_prop(height.get(prop)),
                Some(expected),
                "{} root height `{prop}` should stay `{expected}`",
                path.display()
            );
        }
        assert_eq!(
            height.get("stretch").and_then(|value| value.as_str()),
            Some("Fixed"),
            "{} root height should stay fixed for grid stability",
            path.display()
        );
        let container = table_prop(layout, "container")
            .unwrap_or_else(|| panic!("{} root should define container layout", path.display()));
        assert_eq!(
            container.get("kind").and_then(|value| value.as_str()),
            Some("VerticalBox"),
            "{} root container should stay VerticalBox",
            path.display()
        );
        assert_eq!(
            numeric_prop(container.get("gap")),
            Some(6.0),
            "{} root container gap should stay 6px",
            path.display()
        );
    }
}

#[test]
fn material_component_prototype_meta_strips_cover_variants_and_layout_modes() {
    let allowed_groups = [
        "Inputs",
        "Data Display",
        "Feedback",
        "Surfaces",
        "Navigation",
        "Layout",
        "MUI X",
        "Utils/Lab",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let required_variant_examples = [
        ("buttons", "filled / outlined / text / tonal / elevated"),
        ("text_fields", "outlined / filled / standard"),
        ("textarea_autosize", "min rows / max rows"),
        ("switches", "checked / unchecked"),
        ("dialogs", "open / anchored"),
        ("cards", "elevated / outlined"),
        ("tabs", "active / inactive"),
        ("toggle_button", "exclusive / multiple"),
        ("mui_x_data_grid", "sorted / selected"),
        ("mui_x_date_time_pickers", "date / time / date-time"),
        ("mui_x_agent_chat", "streaming / error"),
    ]
    .into_iter()
    .collect::<BTreeMap<_, _>>();

    for path in material_prototype_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
        let key = path
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("prototype file stem is UTF-8")
            .strip_prefix("material_")
            .expect("prototype files use material_ prefix");
        let meta = document
            .nodes
            .get("meta")
            .unwrap_or_else(|| panic!("{} should define a visible meta strip", path.display()));

        assert_eq!(
            meta.component.as_str(),
            "HorizontalBox",
            "{} meta strip should stay a horizontal variant summary",
            path.display()
        );
        assert!(
            meta.classes
                .iter()
                .any(|class| class == "material-lab-meta-strip"),
            "{} meta strip should keep the Material Lab meta-strip class",
            path.display()
        );
        assert_eq!(
            child_nodes(&document, "meta"),
            vec![
                "meta_group",
                "meta_response",
                "meta_variant",
                "meta_layout"
            ],
            "{} meta strip should expose component group, response mechanism, appearance variant, and layout mode chips",
            path.display()
        );

        assert_meta_chip(
            &document,
            &path,
            "meta_group",
            "material-lab-meta-group",
            None,
        );
        assert_meta_chip(
            &document,
            &path,
            "meta_response",
            "material-lab-meta-response",
            Some("response_mechanism"),
        );
        assert_meta_chip(
            &document,
            &path,
            "meta_variant",
            "material-lab-meta-variant",
            Some("variant"),
        );
        assert_meta_chip(
            &document,
            &path,
            "meta_layout",
            "material-lab-meta-layout",
            Some("layout_mode"),
        );

        let group = text_prop(&document, "meta_group");
        assert!(
            allowed_groups.contains(group),
            "{} meta group `{group}` should be one of the Material component families",
            path.display()
        );
        let variant = text_prop(&document, "meta_variant");
        let layout = text_prop(&document, "meta_layout");
        let response = text_prop(&document, "meta_response");
        assert!(
            response.contains(" / "),
            "{} meta response `{response}` should show a response mechanism pair",
            path.display()
        );
        assert!(
            variant.contains(" / "),
            "{} meta variant `{variant}` should show at least two appearance states",
            path.display()
        );
        assert!(
            !layout.is_empty(),
            "{} meta layout should name the layout mode",
            path.display()
        );
        if let Some(expected_variant) = required_variant_examples.get(key) {
            assert_eq!(
                variant,
                *expected_variant,
                "{} should keep the planned variant summary for `{key}`",
                path.display()
            );
        }
        if let Some(expected_response) = required_response_examples().get(key) {
            assert_eq!(
                response,
                *expected_response,
                "{} should keep the planned response summary for `{key}`",
                path.display()
            );
        }
    }
}

#[test]
fn material_component_prototype_state_strips_cover_core_feedback_examples() {
    let expected_states = [
        (
            "state_default",
            "Default",
            "material-lab-state-default",
            [false, false, false, false, false, false, false],
            "normal",
            "secondary",
        ),
        (
            "state_hover",
            "Hover",
            "material-lab-state-hover",
            [true, false, false, false, false, false, false],
            "normal",
            "primary",
        ),
        (
            "state_pressed",
            "Pressed",
            "material-lab-state-pressed",
            [true, true, false, false, false, false, false],
            "normal",
            "primary",
        ),
        (
            "state_focus",
            "Focus",
            "material-lab-state-focus",
            [false, false, true, false, false, false, false],
            "normal",
            "primary",
        ),
        (
            "state_disabled",
            "Disabled",
            "material-lab-state-disabled",
            [false, false, false, false, false, true, false],
            "normal",
            "disabled",
        ),
        (
            "state_selected",
            "Selected",
            "material-lab-state-selected",
            [false, false, false, true, true, false, false],
            "normal",
            "primary",
        ),
        (
            "state_open",
            "Open",
            "material-lab-state-open",
            [false, false, false, true, false, false, true],
            "normal",
            "primary",
        ),
        (
            "state_error",
            "Error",
            "material-lab-state-error",
            [false, false, false, false, false, false, false],
            "error",
            "error",
        ),
    ];

    for path in material_prototype_files() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
        let state_strip = document
            .nodes
            .get("state_strip")
            .unwrap_or_else(|| panic!("{} should define a visible state strip", path.display()));

        assert_eq!(
            state_strip.component.as_str(),
            "HorizontalBox",
            "{} state strip should stay a compact horizontal state row",
            path.display()
        );
        assert!(
            state_strip
                .classes
                .iter()
                .any(|class| class == "material-lab-state-strip"),
            "{} state strip should keep the Material Lab state-strip class",
            path.display()
        );

        assert_eq!(
            child_nodes(&document, "state_strip"),
            expected_states
                .iter()
                .map(|(node_id, _, _, _, _, _)| *node_id)
                .collect::<Vec<_>>(),
            "{} state strip should expose one visible pill per Material feedback state",
            path.display()
        );

        for (node_id, label, state_class, flags, validation_level, text_tone) in expected_states {
            let pill = document.nodes.get(node_id).unwrap_or_else(|| {
                panic!(
                    "{} state strip should define visible pill `{node_id}`",
                    path.display()
                )
            });
            assert_eq!(
                pill.component.as_str(),
                "Label",
                "{} state pill `{node_id}` should remain a Label",
                path.display()
            );
            for class in ["material-lab-state-pill", state_class] {
                assert!(
                    pill.classes.iter().any(|pill_class| pill_class == class),
                    "{} state pill `{node_id}` should keep class `{class}`",
                    path.display()
                );
            }
            assert_eq!(
                pill.props.get("text").and_then(|value| value.as_str()),
                Some(label),
                "{} state pill `{node_id}` should display `{label}`",
                path.display()
            );
            assert_eq!(
                pill.props
                    .get("validation_level")
                    .and_then(|value| value.as_str()),
                Some(validation_level),
                "{} state pill `{node_id}` should set validation_level `{validation_level}`",
                path.display()
            );
            assert_eq!(
                pill.props.get("text_tone").and_then(|value| value.as_str()),
                Some(text_tone),
                "{} state pill `{node_id}` should set text_tone `{text_tone}`",
                path.display()
            );

            for (prop, expected) in [
                ("hovered", flags[0]),
                ("pressed", flags[1]),
                ("focused", flags[2]),
                ("selected", flags[3]),
                ("checked", flags[4]),
                ("disabled", flags[5]),
                ("open", flags[6]),
            ] {
                assert_eq!(
                    pill.props.get(prop).and_then(|value| value.as_bool()),
                    Some(expected),
                    "{} state pill `{node_id}` should set `{prop}` to `{expected}`",
                    path.display()
                );
            }
            assert_eq!(
                pill.props
                    .get("popup_open")
                    .and_then(|value| value.as_bool()),
                Some(node_id == "state_open"),
                "{} state pill `{node_id}` should only open popup feedback for the open state",
                path.display()
            );
            assert_eq!(
                numeric_prop(pill.props.get("corner_radius")),
                Some(999.0),
                "{} state pill `{node_id}` should render as a rounded Material pill",
                path.display()
            );
        }
    }
}

fn assert_meta_chip(
    document: &UiV2AssetDocument,
    path: &std::path::Path,
    node_id: &str,
    expected_class: &str,
    expected_echo_prop: Option<&str>,
) {
    let chip = document
        .nodes
        .get(node_id)
        .unwrap_or_else(|| panic!("{} should define meta chip `{node_id}`", path.display()));
    assert_eq!(
        chip.component.as_str(),
        "Label",
        "{} meta chip `{node_id}` should remain a Label",
        path.display()
    );
    for class in ["material-lab-meta-chip", expected_class] {
        assert!(
            chip.classes.iter().any(|chip_class| chip_class == class),
            "{} meta chip `{node_id}` should keep class `{class}`",
            path.display()
        );
    }
    let text = chip
        .props
        .get("text")
        .and_then(|value| value.as_str())
        .unwrap_or_else(|| {
            panic!(
                "{} meta chip `{node_id}` should expose text",
                path.display()
            )
        });
    assert!(
        !text.is_empty(),
        "{} meta chip `{node_id}` should not be blank",
        path.display()
    );
    if let Some(prop) = expected_echo_prop {
        assert_eq!(
            chip.props.get(prop).and_then(|value| value.as_str()),
            Some(text),
            "{} meta chip `{node_id}` should mirror text into `{prop}`",
            path.display()
        );
    }
    if node_id != "meta_group" {
        assert_eq!(
            chip.props.get("density").and_then(|value| value.as_str()),
            Some("compact"),
            "{} meta chip `{node_id}` should freeze compact density",
            path.display()
        );
    }
    assert_eq!(
        numeric_prop(chip.props.get("corner_radius")),
        Some(999.0),
        "{} meta chip `{node_id}` should render as a rounded Material pill",
        path.display()
    );
    assert_eq!(
        numeric_prop(chip.props.get("border_width")),
        Some(1.0),
        "{} meta chip `{node_id}` should keep a 1px Material outline",
        path.display()
    );
}

fn required_response_examples() -> BTreeMap<&'static str, &'static str> {
    [
        ("buttons", "click / press"),
        ("text_fields", "change / commit"),
        ("switches", "toggle / checked"),
        ("slider", "drag / update"),
        ("tooltips", "hover / preview"),
        ("mui_x_agent_chat", "submit / send"),
        ("mui_x_charts", "hover / preview"),
        ("mui_x_date_time_pickers", "open / commit"),
        ("portal", "utility / none"),
        ("typography", "static / display"),
    ]
    .into_iter()
    .collect()
}

fn text_prop<'a>(document: &'a UiV2AssetDocument, node_id: &str) -> &'a str {
    document
        .nodes
        .get(node_id)
        .and_then(|node| node.props.get("text"))
        .and_then(|value| value.as_str())
        .unwrap_or_else(|| panic!("node `{node_id}` should expose text"))
}

fn assert_child_contract(
    document: &UiV2AssetDocument,
    path: &std::path::Path,
    node_id: &str,
    expected_component: &str,
    expected_class: &str,
    expected_height: f64,
) {
    let node = document
        .nodes
        .get(node_id)
        .unwrap_or_else(|| panic!("{} should define node `{node_id}`", path.display()));
    assert_eq!(
        node.component.as_str(),
        expected_component,
        "{} node `{node_id}` should remain `{expected_component}`",
        path.display()
    );
    assert!(
        node.classes.iter().any(|class| class == expected_class),
        "{} node `{node_id}` should keep class `{expected_class}`",
        path.display()
    );
    let layout = node
        .layout
        .as_ref()
        .unwrap_or_else(|| panic!("{} node `{node_id}` should define layout", path.display()));
    let height = table_prop(layout, "height").unwrap_or_else(|| {
        panic!(
            "{} node `{node_id}` should define height layout",
            path.display()
        )
    });
    for prop in ["min", "preferred", "max"] {
        assert_eq!(
            numeric_prop(height.get(prop)),
            Some(expected_height),
            "{} node `{node_id}` height `{prop}` should stay `{expected_height}`",
            path.display()
        );
    }
    assert_eq!(
        height.get("stretch").and_then(|value| value.as_str()),
        Some("Fixed"),
        "{} node `{node_id}` height should stay fixed",
        path.display()
    );
}

fn table_prop<'a>(
    values: &'a std::collections::BTreeMap<String, toml::Value>,
    key: &str,
) -> Option<&'a toml::Table> {
    values.get(key).and_then(|value| value.as_table())
}
