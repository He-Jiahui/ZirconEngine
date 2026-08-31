use super::*;

#[test]
fn material_component_lab_mui_x_prototypes_define_feedback_routes() {
    for key in MUI_X_PROTOTYPES {
        let fixture = material_prototype_fixture(key);
        let path = &fixture.path;
        let source = &fixture.source;
        assert!(
            source.contains("MaterialLab/") && source.contains("route = \"material_lab."),
            "{} should define a Material Lab feedback route for MUI X interaction evidence",
            path.display()
        );
    }
}

#[test]
fn material_component_lab_non_route_prototypes_are_not_dispatchable_controls() {
    for (key, fixture) in material_prototype_fixtures() {
        let path = &fixture.path;
        let source = &fixture.source;
        if source.contains("MaterialLab/")
            || STRUCTURAL_CHILD_ROUTE_PROTOTYPES.contains(&key.as_str())
        {
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
    for (key, fixture) in material_prototype_fixtures() {
        let path = &fixture.path;
        let source = &fixture.source;
        if !source.contains("MaterialLab/")
            && !STRUCTURAL_CHILD_ROUTE_PROTOTYPES.contains(&key.as_str())
        {
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
    for fixture in material_prototype_fixtures().values() {
        let path = &fixture.path;
        let source = &fixture.source;
        let document = &fixture.document;
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

    for (key, fixture) in material_prototype_fixtures() {
        let path = &fixture.path;
        let document = &fixture.document;
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
        let event = feedback_events[0];
        actual.insert(
            key.clone(),
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
    let fixture = material_prototype_fixture("button_group");
    let path = &fixture.path;
    let document = &fixture.document;
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
