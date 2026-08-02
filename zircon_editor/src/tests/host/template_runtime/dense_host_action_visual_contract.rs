#[test]
fn dense_host_actions_declare_their_icon_and_text_composition() {
    let cases = [
        (
            "animation_graph_body.zui",
            "props = { label = \"Add Node\", icon = \"editor_pages/graph_editor/nodes/state-node.svg\", icon_placement = \"leading\" }",
        ),
        (
            "animation_sequence_body.zui",
            "props = { label = \"Scrub Timeline\", icon = \"editor_pages/animation_timeline/transport/timeline-play.svg\", icon_placement = \"leading\" }",
        ),
        (
            "module_plugins_body.zui",
            "props = { label = \"Focus Plugins\", icon = \"editor_pages/build_plugins/plugins/plugin.svg\", icon_placement = \"leading\" }",
        ),
        (
            "performance_timeline_body.zui",
            "props = { label = \"Refresh Timeline\", icon = \"editor_pages/console_profiler/profiling/frame-time.svg\", icon_placement = \"leading\" }",
        ),
        (
            "runtime_diagnostics_body.zui",
            "props = { label = \"Focus Diagnostics\", icon = \"editor_pages/console_profiler/diagnostics/watch.svg\", icon_placement = \"icon_only\" }",
        ),
    ];

    for (file_name, expected_props) in cases {
        let template = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets/ui/editor/host")
                .join(file_name),
        )
        .unwrap_or_else(|error| panic!("failed to read `{file_name}`: {error}"));
        assert!(
            template.contains(expected_props),
            "{file_name} must declare the intended icon composition for its dense action"
        );
        assert!(
            !template.contains("gap = 6.0") && !template.contains("gap = 4.0"),
            "{file_name} must use shared density spacing rather than local gap literals"
        );
    }
}

#[test]
fn project_asset_and_hierarchy_actions_use_explicit_dense_composition() {
    for (file_name, expected_leading_action_count) in [
        ("asset_surface_controls.zui", 6),
        ("hierarchy_body.zui", 1),
        ("pane_surface_controls.zui", 1),
        ("startup_welcome_controls.zui", 3),
    ] {
        let template = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets/ui/editor/host")
                .join(file_name),
        )
        .unwrap_or_else(|error| panic!("failed to read `{file_name}`: {error}"));
        assert_eq!(
            template.matches("icon_placement = \"leading\"").count(),
            expected_leading_action_count,
            "{file_name} must declare every visible text action as icon-and-text"
        );

        if file_name == "startup_welcome_controls.zui" {
            assert!(
                !template.contains("border_width = 1.0"),
                "welcome controls must use the shared border-width token"
            );
            assert_eq!(
                template
                    .matches("border_width = \"$editor.control.border_width\"")
                    .count(),
                6,
                "the welcome inputs and actions must share one border-width token"
            );
        }
    }
}

#[test]
fn single_action_headers_expand_to_their_available_pane_width() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for (file_name, control_id, legacy_width) in [
        ("hierarchy_body.zui", "SelectRoot", "120.0"),
        ("build_export_desktop_body.zui", "FocusBuildExport", "128.0"),
    ] {
        let template = std::fs::read_to_string(root.join("assets/ui/editor/host").join(file_name))
            .unwrap_or_else(|error| panic!("failed to read `{file_name}`: {error}"));
        let control_start = template
            .find(&format!("control_id = \"{control_id}\""))
            .expect("single-action header control must remain in its template");
        let control = &template[control_start..];
        let layout_end = control
            .find("\nevents =")
            .expect("single-action header control must declare an event after its layout");

        assert!(
            control[..layout_end].contains("width = { stretch = \"Stretch\" }"),
            "{control_id} must fill the available header row width"
        );
        assert!(
            !template.contains(&format!("preferred = {legacy_width}")),
            "{file_name} must not retain its fixed single-action width"
        );
    }
}

#[test]
fn high_frequency_text_inputs_use_shared_editor_chrome_tokens() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let asset_controls =
        std::fs::read_to_string(root.join("assets/ui/editor/host/asset_surface_controls.zui"))
            .expect("asset surface controls template should be readable");
    let inspector_controls =
        std::fs::read_to_string(root.join("assets/ui/editor/host/inspector_surface_controls.zui"))
            .expect("inspector surface controls template should be readable");

    assert!(
        asset_controls.contains(
            "props = { placeholder = \"Search\", value_text = \"\", surface_variant = \"inset\", border_width = \"$editor.control.border_width\", corner_radius = \"$editor.control.radius.control\", layout_min_height = \"$editor.control.height.dense\", input_focusable = true }"
        ),
        "asset search must use the shared dense input chrome"
    );
    assert!(
        asset_controls.contains(
            "props = { label = \"Kind\", value = \"all\", value_text = \"all\", options = [\"all\", \"scene\", \"mesh\", \"material\", \"ui\"], surface_variant = \"inset\", border_width = \"$editor.control.border_width\", corner_radius = \"$editor.control.radius.control\", layout_min_height = \"$editor.control.height.dense\", input_focusable = true }"
        ),
        "asset kind selection must use the shared dense selection-field chrome"
    );
    assert_eq!(
        inspector_controls
            .matches("border_width = \"$editor.control.border_width\"")
            .count(),
        7,
        "inspector fields and actions must share the editor border-width token"
    );
    assert!(
        !inspector_controls.contains("border_width = 1.0"),
        "inspector inputs must not keep a local border-width literal"
    );
    assert_eq!(
        inspector_controls
            .matches("layout_min_height = \"$editor.control.height.dense\"")
            .count(),
        7,
        "inspector fields and actions must use the shared dense control height"
    );
}
