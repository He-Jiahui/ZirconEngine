use super::*;

#[test]
fn blend_space_bottom_diagnostics_compose_shared_relative_components() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/modules/extensions/animation/\
             workbench_extension_blend_space_workspace.zui",
    ))
    .expect("Blend Space workspace asset should be readable");
    let sample_weights = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/composites/animation/\
             workbench_sample_weights.zui",
    ))
    .expect("shared sample weights composite should be readable");
    let validation_log = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/composites/feedback/\
             workbench_validation_log.zui",
    ))
    .expect("shared validation log composite should be readable");

    for required in [
        "[components.WorkbenchSampleWeights]",
        "component = \"WorkbenchProgressBar\"",
        "component = \"WorkbenchCheckbox\"",
        "control_id = \"WorkbenchSampleWeightsRunForward\"",
        "value_percent = 1.0",
        "text = \"Normalize\"",
    ] {
        assert!(
            sample_weights.contains(required),
            "missing shared sample-weights contract: {required}"
        );
    }
    for required in [
        "[components.WorkbenchValidationLog]",
        "component = \"WorkbenchStatusItem\"",
        "control_id = \"WorkbenchValidationLogWarningRow\"",
        "validation_level = \"warning\"",
        "text = \"[Warning] 1 sample missing.\"",
        "text = \"Clear\"",
    ] {
        assert!(
            validation_log.contains(required),
            "missing shared validation-log contract: {required}"
        );
    }
    for required in [
        "workbench_sample_weights.zui#WorkbenchSampleWeights",
        "workbench_validation_log.zui#WorkbenchValidationLog",
        "component = \"WorkbenchSampleWeights\"",
        "component = \"WorkbenchValidationLog\"",
        "control_id = \"WorkbenchExtensionBlendSpaceBottomCompositeRow\"",
        "responsive_min_tier = \"wide\"",
    ] {
        assert!(
            workspace.contains(required),
            "Blend Space must compose shared bottom diagnostics: {required}"
        );
    }
    for source in [&sample_weights, &validation_log] {
        assert!(
            source.matches("stretch = \"Stretch\"").count() >= 8,
            "bottom composite geometry must distribute available space through relative stretch"
        );
        for forbidden in [
            "background_color =",
            "border_color =",
            "foreground_color =",
            "font_size =",
            "font_weight =",
        ] {
            assert!(
                !source.contains(forbidden),
                "bottom composites must inherit shared visual tokens: {forbidden}"
            );
        }
    }
}

#[test]
fn blend_space_timeline_uses_shared_unreal_density_transport_controls() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/modules/extensions/animation/\
             workbench_extension_blend_space_workspace.zui",
    ))
    .expect("Blend Space workspace asset should be readable");
    let transport = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/composites/animation/\
             workbench_transport_controls.zui",
    ))
    .expect("shared transport controls composite should be readable");

    for required in [
        "[components.WorkbenchTransportControls]",
        "component = \"WorkbenchIconButton\"",
        "control_id = \"WorkbenchTransportRecord\"",
        "control_id = \"WorkbenchTransportPlay\"",
        "control_id = \"WorkbenchTransportPause\"",
        "control_id = \"WorkbenchTransportPrevious\"",
        "control_id = \"WorkbenchTransportNext\"",
        "control_id = \"WorkbenchTransportLoop\"",
        "layout_icon_size = 20.0",
        "layout_padding_left = 2.0",
        "layout_padding_right = 2.0",
        "layout_padding_top = 2.0",
        "layout_padding_bottom = 2.0",
        "WorkbenchExtension/AnimationTransportRecord",
        "workbench.extension.animation_transport.record.toggle",
        "WorkbenchExtension/AnimationTransportPlay",
        "workbench.extension.animation_transport.play.invoke",
        "WorkbenchExtension/AnimationTransportPause",
        "workbench.extension.animation_transport.pause.invoke",
        "WorkbenchExtension/AnimationTransportPrevious",
        "workbench.extension.animation_transport.previous.invoke",
        "WorkbenchExtension/AnimationTransportNext",
        "workbench.extension.animation_transport.next.invoke",
        "WorkbenchExtension/AnimationTransportLoop",
        "workbench.extension.animation_transport.loop.toggle",
    ] {
        assert!(
            transport.contains(required),
            "missing Unreal-density transport contract: {required}"
        );
    }
    for forbidden in [
        "text = \"Rec\"",
        "text = \"Play\"",
        "text = \"Pause\"",
        "text = \"Prev\"",
        "text = \"Next\"",
        "text = \"Loop\"",
        "background_color =",
        "border_color =",
        "font_size =",
        "font_weight =",
    ] {
        assert!(
            !transport.contains(forbidden),
            "transport controls must use shared icon/palette contracts: {forbidden}"
        );
    }
    for required in [
        "workbench_transport_controls.zui#WorkbenchTransportControls",
        "component = \"WorkbenchTransportControls\"",
        "control_id = \"WorkbenchExtensionBlendSpaceTimelineTransport\"",
    ] {
        assert!(
            workspace.contains(required),
            "Blend Space must compose the shared transport controls: {required}"
        );
    }
}

#[test]
fn blend_space_preview_uses_shared_real_image_atom_and_runtime_text_overlays() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let primitive = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/primitives/data/workbench_preview_viewport.zui",
    ))
    .expect("shared preview viewport primitive should be readable");
    let workspace = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/modules/extensions/animation/\
             workbench_extension_blend_space_workspace.zui",
    ))
    .expect("Blend Space workspace asset should be readable");
    let preview_svg = std::fs::read_to_string(
        manifest.join("assets/ui/editor/workbench/preview/animation_mannequin_viewport.svg"),
    )
    .expect("animation preview image should be readable");

    for required in [
        "[components.WorkbenchPreviewViewport]",
        "component = \"Image\"",
        "image = \"ui/editor/workbench/preview/animation_mannequin_viewport.svg\"",
        "preserve_aspect = true",
    ] {
        assert!(
            primitive.contains(required),
            "missing shared preview viewport atom contract: {required}"
        );
    }
    for required in [
        "workbench_preview_viewport.zui#WorkbenchPreviewViewport",
        "control_id = \"WorkbenchExtensionBlendSpacePreviewViewportImage\"",
        "control_id = \"WorkbenchExtensionBlendSpacePreviewCamera\"",
        "control_id = \"WorkbenchExtensionBlendSpacePreviewLighting\"",
        "control_id = \"WorkbenchExtensionBlendSpacePreviewStatus\"",
        "text = \"Perspective\"",
        "text = \"Lit\"",
        "text = \"Run_Fwd  |  Previewing\"",
    ] {
        assert!(
            workspace.contains(required),
            "Blend Space preview must compose the image atom with Runtime Text primitives: {required}"
        );
    }
    assert!(
        preview_svg.contains("<linearGradient")
            && preview_svg.contains("id=\"mannequin\"")
            && preview_svg.contains("id=\"floor-grid\"")
            && preview_svg.contains("id=\"axis-gizmo\""),
        "preview image should carry a real viewport scene, mannequin, floor grid, and axis gizmo"
    );
    assert!(
        !preview_svg.contains("<text"),
        "user-visible viewport labels must remain Runtime Text nodes, not SVG-baked text"
    );
}

#[test]
fn blend_space_bottom_panels_compose_shared_panel_headers() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let panel_header = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/composites/chrome/\
             workbench_panel_header.zui",
    ))
    .expect("shared panel header composite should be readable");
    let sample_weights = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/composites/animation/\
             workbench_sample_weights.zui",
    ))
    .expect("shared sample weights composite should be readable");
    let validation_log = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/composites/feedback/\
             workbench_validation_log.zui",
    ))
    .expect("shared validation log composite should be readable");
    let workspace = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/modules/extensions/animation/\
             workbench_extension_blend_space_workspace.zui",
    ))
    .expect("Blend Space workspace asset should be readable");

    for required in [
        "[components.WorkbenchPanelHeader]",
        "slots = { title = { multiple = false }, actions = { multiple = true } }",
        "classes = [\"workbench-panel-toolbar\"]",
    ] {
        assert!(
            panel_header.contains(required),
            "missing shared panel-header contract: {required}"
        );
    }
    for (name, source) in [
        ("Sample Weights", &sample_weights),
        ("Validation Log", &validation_log),
        ("Preview Timeline", &workspace),
    ] {
        assert!(
            source.contains("workbench_panel_header.zui#WorkbenchPanelHeader"),
            "{name} must import the shared panel header"
        );
        assert!(
            source.contains("component = \"WorkbenchPanelHeader\""),
            "{name} must compose the shared panel header"
        );
        assert!(
            source.contains("slot = { name = \"title\" }"),
            "{name} must mount its Runtime Text title through the title slot"
        );
    }
    assert!(
        workspace.contains("slot = { name = \"actions\" }")
            && workspace.contains("control_id = \"WorkbenchExtensionBlendSpaceTimelineRate\""),
        "Preview Timeline must keep its relative playback-rate action in the header action slot"
    );
}

#[test]
fn blend_space_center_panels_compose_shared_panel_headers() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/modules/extensions/animation/\
             workbench_extension_blend_space_workspace.zui",
    ))
    .expect("Blend Space workspace asset should be readable");
    let parsed = toml::from_str::<toml::Value>(&workspace)
        .expect("Blend Space workspace asset should parse as TOML");
    let nodes = parsed
        .get("nodes")
        .and_then(toml::Value::as_table)
        .expect("Blend Space workspace should declare nodes");

    for (header, mounts) in [
        (
            "blend_space_canvas_toolbar",
            &[
                ("blend_space_canvas_title", "title"),
                ("blend_space_canvas_status", "actions"),
            ][..],
        ),
        (
            "blend_space_preview_header",
            &[("blend_space_preview_title", "title")][..],
        ),
        (
            "blend_space_heatmap_header",
            &[("blend_space_heatmap_title", "title")][..],
        ),
    ] {
        let node = nodes
            .get(header)
            .and_then(toml::Value::as_table)
            .unwrap_or_else(|| panic!("center panel header `{header}` should exist"));
        assert_eq!(
            node.get("component").and_then(toml::Value::as_str),
            Some("WorkbenchPanelHeader"),
            "center panel header `{header}` must consume the shared composite"
        );
        let children = node
            .get("children")
            .and_then(toml::Value::as_array)
            .unwrap_or_else(|| panic!("center panel header `{header}` should mount children"));
        for &(child_id, slot_name) in mounts {
            assert!(
                children.iter().any(|child| {
                    child.get("node").and_then(toml::Value::as_str) == Some(child_id)
                        && child
                            .get("slot")
                            .and_then(toml::Value::as_table)
                            .and_then(|slot| slot.get("name"))
                            .and_then(toml::Value::as_str)
                            == Some(slot_name)
                }),
                "center panel header `{header}` must mount `{child_id}` through `{slot_name}`"
            );
        }
    }
    assert!(
        !nodes.contains_key("blend_space_canvas_fill"),
        "the shared stretch title slot must replace the feature-local toolbar fill node"
    );
}

#[test]
fn blend_space_panel_header_slots_expand_with_one_shared_layout_contract() {
    let bridge = open_blend_space_bridge(1260, 780);
    let projection = bridge.host_projection();

    for (header_id, title_id) in [
        (
            "WorkbenchExtensionBlendSpaceCanvasToolbar",
            "WorkbenchExtensionBlendSpaceCanvasTitle",
        ),
        (
            "WorkbenchExtensionBlendSpacePreviewHeader",
            "WorkbenchExtensionBlendSpacePreviewTitle",
        ),
        (
            "WorkbenchExtensionBlendSpaceHeatmapHeader",
            "WorkbenchExtensionBlendSpaceHeatmapTitle",
        ),
        (
            "WorkbenchExtensionBlendSpaceOutputHeader",
            "WorkbenchExtensionBlendSpaceOutputTitle",
        ),
        (
            "WorkbenchSampleWeightsHeader",
            "WorkbenchSampleWeightsTitle",
        ),
        (
            "WorkbenchValidationLogHeader",
            "WorkbenchValidationLogTitle",
        ),
    ] {
        let header = projection
            .node_by_control_id(header_id)
            .unwrap_or_else(|| panic!("expanded panel header `{header_id}` should be projected"));
        let title = projection
            .node_by_control_id(title_id)
            .unwrap_or_else(|| panic!("mounted panel title `{title_id}` should be projected"));

        assert_eq!(header.component, "HorizontalGroup");
        assert_eq!(title.parent_id.as_deref(), Some(header.node_id.as_str()));
        assert!(
            (header.frame.height - 28.0).abs() <= 0.5,
            "all panel-header consumers must inherit the shared 28px preferred height: {header_id}={:?}",
            header.frame
        );
        assert!(title.frame.x >= header.frame.x - 0.5);
        assert!(title.frame.right() <= header.frame.right() + 0.5);
        assert!(title.frame.y >= header.frame.y - 0.5);
        assert!(title.frame.bottom() <= header.frame.bottom() + 0.5);
    }

    let output_header = projection
        .node_by_control_id("WorkbenchExtensionBlendSpaceOutputHeader")
        .expect("expanded Preview Timeline header should be projected");
    let title = projection
        .node_by_control_id("WorkbenchExtensionBlendSpaceOutputTitle")
        .expect("mounted Preview Timeline title should be projected");
    let action = projection
        .node_by_control_id("WorkbenchExtensionBlendSpaceTimelineRate")
        .expect("mounted Preview Timeline action should be projected");

    assert_eq!(
        action.parent_id.as_deref(),
        Some(output_header.node_id.as_str())
    );
    assert!(title.frame.right() <= action.frame.x + 0.5);
    assert!(action.frame.right() <= output_header.frame.right() + 0.5);
    assert!(action.frame.bottom() <= output_header.frame.bottom() + 0.5);

    let canvas_header = projection
        .node_by_control_id("WorkbenchExtensionBlendSpaceCanvasToolbar")
        .expect("expanded Sample Grid header should be projected");
    let canvas_title = projection
        .node_by_control_id("WorkbenchExtensionBlendSpaceCanvasTitle")
        .expect("mounted Sample Grid title should be projected");
    let canvas_action = projection
        .node_by_control_id("WorkbenchExtensionBlendSpaceCanvasStatus")
        .expect("mounted Sample Grid status action should be projected");
    assert_eq!(
        canvas_action.parent_id.as_deref(),
        Some(canvas_header.node_id.as_str())
    );
    assert!(canvas_title.frame.right() <= canvas_action.frame.x + 0.5);
    assert!(canvas_action.frame.right() <= canvas_header.frame.right() + 0.5);
}
