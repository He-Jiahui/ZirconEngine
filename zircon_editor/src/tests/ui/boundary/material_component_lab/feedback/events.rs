use super::*;

#[test]
fn material_floating_action_button_sample_covers_circular_small_and_extended_shapes() {
    let fixture = material_prototype_fixture("floating_action_button");
    let path = &fixture.path;
    let document = &fixture.document;
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
    for fixture in material_prototype_fixtures().values() {
        let path = &fixture.path;
        let document = &fixture.document;

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
    let actual = material_prototype_fixtures()
        .iter()
        .filter_map(|(key, fixture)| fixture.source.contains("MaterialLab/").then(|| key.clone()))
        .collect::<BTreeSet<_>>();

    assert_eq!(
        actual, expected,
        "interactive prototype inventory should match route-bearing Material Lab prototypes"
    );
}
