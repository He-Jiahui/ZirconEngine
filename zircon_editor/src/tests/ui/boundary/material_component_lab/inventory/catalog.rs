use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::UiV2AssetKind;

use super::super::support::*;

#[test]
fn material_component_prototypes_are_grouped_by_functional_domain_folder() {
    let root = editor_asset("assets/ui/editor/material_components");
    let entries = fs::read_dir(&root)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", root.display()))
        .map(|entry| {
            entry
                .expect("material component dir entry should be readable")
                .path()
        })
        .collect::<Vec<_>>();

    let flat_prototypes = entries
        .iter()
        .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
        .filter(|name| name.starts_with("material_") && name.ends_with(".zui"))
        .collect::<Vec<_>>();

    assert!(
        flat_prototypes.is_empty(),
        "Material component prototypes must live in functional domain folders, not directly under material_components/: {:?}",
        flat_prototypes
    );

    let actual_domains = entries
        .iter()
        .filter(|path| path.is_dir())
        .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
        .collect::<BTreeSet<_>>();
    let expected_domains = MATERIAL_COMPONENT_DOMAINS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();

    assert_eq!(
        actual_domains, expected_domains,
        "material_components should be grouped by the Material Lab functional sections"
    );
}

#[test]
fn material_component_lab_view_loads_and_imports_every_component_prototype() {
    let lab_path = editor_asset("assets/ui/editor/material_component_lab.zui");
    let lab_source = fs::read_to_string(&lab_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", lab_path.display()));
    let lab = UiZuiAssetLoader::load_zui_file(&lab_path).unwrap_or_else(|error| {
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
    let prototype_fixtures = material_prototype_fixtures();
    assert_eq!(
        imports.len(),
        prototype_fixtures.len(),
        "lab should import every material_*.zui prototype exactly once"
    );
    for fixture in prototype_fixtures.values() {
        let component_name = fixture
            .document
            .components
            .keys()
            .next()
            .expect("prototype declares one component");
        let import = material_prototype_res_path(&fixture.path, component_name);
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
            lab_source.contains(label),
            "lab view should expose the `{label}` section"
        );
    }
}

#[test]
fn material_component_lab_places_every_prototype_once_in_visible_sections() {
    let lab_path = editor_asset("assets/ui/editor/material_component_lab.zui");
    let lab = UiZuiAssetLoader::load_zui_file(&lab_path).unwrap_or_else(|error| {
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
    let lab_path = editor_asset("assets/ui/editor/material_component_lab.zui");
    let lab = UiZuiAssetLoader::load_zui_file(&lab_path).unwrap_or_else(|error| {
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
    for (key, fixture) in material_prototype_fixtures() {
        let path = &fixture.path;
        let source = &fixture.source;
        let document = &fixture.document;
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

        if interactive.contains(key.as_str()) {
            assert!(
                document.nodes.values().any(|node| {
                    node.events.iter().any(|event| {
                        event
                            .route
                            .as_deref()
                            .is_some_and(|route| route.starts_with("material_lab."))
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
    for (key, fixture) in material_prototype_fixtures() {
        let path = &fixture.path;
        let document = &fixture.document;
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
