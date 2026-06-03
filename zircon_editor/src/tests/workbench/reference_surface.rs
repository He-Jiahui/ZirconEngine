use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostValue, WORKBENCH_WINDOW_DOCUMENT_ID,
};
use crate::ui::workbench::reference::{
    build_editor_workbench_reference_surface, build_editor_workbench_template_surface,
    EditorWorkbenchReferenceMetrics, EditorWorkbenchReferenceSurface,
    EditorWorkbenchTemplateControlIds,
};
use zircon_runtime::ui::{dispatch::UiPointerDispatcher, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    dispatch::UiPointerEvent,
    event_ui::UiNodeId,
    layout::{UiFrame, UiSize},
    surface::{UiPointerButton, UiPointerEventKind, UiRenderCommandKind},
};

#[test]
fn reference_workbench_surface_lays_out_target_editor_chrome() {
    let mut reference = build_editor_workbench_reference_surface().unwrap();
    reference.compute_reference_layout().unwrap();
    let metrics = reference.metrics;

    assert_frame(
        frame(&reference, reference.ids.top_bar),
        UiFrame::new(0.0, 0.0, metrics.target_width, metrics.top_bar_height),
    );
    assert_frame(
        frame(&reference, reference.ids.upper_shell),
        UiFrame::new(
            0.0,
            metrics.top_bar_height,
            metrics.target_width,
            metrics.upper_region_height,
        ),
    );
    assert_frame(
        frame(&reference, reference.ids.activity_rail),
        UiFrame::new(
            0.0,
            metrics.top_bar_height,
            metrics.activity_rail_width,
            metrics.upper_region_height,
        ),
    );
    assert_frame(
        frame(&reference, reference.ids.hierarchy_panel),
        UiFrame::new(
            metrics.activity_rail_width,
            metrics.top_bar_height,
            metrics.hierarchy_panel_width,
            metrics.upper_region_height,
        ),
    );
    assert_frame(
        frame(&reference, reference.ids.viewport_panel),
        UiFrame::new(
            metrics.activity_rail_width + metrics.hierarchy_panel_width,
            metrics.top_bar_height,
            metrics.viewport_width(),
            metrics.upper_region_height,
        ),
    );
    assert_frame(
        frame(&reference, reference.ids.inspector_panel),
        UiFrame::new(
            metrics.target_width - metrics.inspector_panel_width,
            metrics.top_bar_height,
            metrics.inspector_panel_width,
            metrics.upper_region_height,
        ),
    );
    assert_frame(
        frame(&reference, reference.ids.component_gallery),
        UiFrame::new(
            0.0,
            metrics.top_bar_height + metrics.upper_region_height,
            metrics.target_width,
            metrics.component_gallery_height(),
        ),
    );
    assert_frame(
        frame(&reference, reference.ids.status_bar),
        UiFrame::new(
            0.0,
            metrics.target_height - metrics.status_bar_height,
            metrics.target_width,
            metrics.status_bar_height,
        ),
    );

    let report = &reference.surface.surface_frame().layout_engine_report;
    assert!(
        report.taffy_selected_count >= 6,
        "expected the reference workbench shell to exercise Taffy-backed layout"
    );
}

#[test]
fn reference_workbench_surface_extracts_renderable_panels_and_controls() {
    let mut reference = build_editor_workbench_reference_surface().unwrap();
    reference.compute_reference_layout().unwrap();
    let commands = &reference.surface.render_extract.list.commands;

    assert!(commands
        .iter()
        .any(|command| command.node_id == reference.ids.viewport_panel
            && command.kind == UiRenderCommandKind::Quad
            && command.style.background_color.as_deref()
                == Some(reference.palette.viewport_background)));
    assert!(commands
        .iter()
        .any(|command| command.node_id == reference.ids.primary_button
            && command.kind == UiRenderCommandKind::Quad
            && command.text.as_deref() == Some("Primary")));
    for text in ["Scene", "Inspector", "Transform", "No Errors"] {
        assert!(
            commands
                .iter()
                .any(|command| command.text.as_deref() == Some(text)),
            "missing text command for {text}"
        );
    }
}

#[test]
fn reference_workbench_surface_routes_primary_button_pointer_response() {
    let mut reference = build_editor_workbench_reference_surface().unwrap();
    reference.compute_reference_layout().unwrap();
    let point = frame(&reference, reference.ids.primary_button).center();
    let dispatcher = UiPointerDispatcher::default();

    let down = reference
        .surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(down.route.target, Some(reference.ids.primary_button));
    assert_eq!(
        reference.surface.focus.pressed,
        Some(reference.ids.primary_button)
    );

    let up = reference
        .surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(up.route.click_target, Some(reference.ids.primary_button));
    assert!(up.component_events.iter().any(|event| {
        event.node_id == reference.ids.primary_button && event.event_kind == UiEventKind::Click
    }));
}

#[test]
fn reference_workbench_componentized_window_surface_matches_reference_chrome_metrics() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let metrics = EditorWorkbenchReferenceMetrics::default();
    let mut template =
        build_editor_workbench_template_surface(&runtime, metrics).expect("template surface");
    let mut reference = build_editor_workbench_reference_surface().unwrap();
    reference.compute_reference_layout().unwrap();

    assert_frame(
        template.frames.root,
        UiFrame::new(0.0, 0.0, metrics.target_width, metrics.target_height),
    );
    assert_frame(
        template.frames.top_toolbar,
        frame(&reference, reference.ids.top_bar),
    );
    assert_frame(
        template.frames.main_band,
        frame(&reference, reference.ids.upper_shell),
    );
    assert_frame(
        template.frames.activity_rail,
        frame(&reference, reference.ids.activity_rail),
    );
    assert_frame(
        template.frames.scene_tree,
        frame(&reference, reference.ids.hierarchy_panel),
    );
    assert_frame(
        template.frames.viewport,
        frame(&reference, reference.ids.viewport_panel),
    );
    assert_frame(
        template.frames.inspector,
        frame(&reference, reference.ids.inspector_panel),
    );
    assert_frame(
        template.frames.component_drawer,
        frame(&reference, reference.ids.component_gallery),
    );
    assert_frame(
        template.frames.status_bar,
        frame(&reference, reference.ids.status_bar),
    );

    let primary = node_by_control(
        &template.surface,
        EditorWorkbenchTemplateControlIds::PRIMARY_BUTTON,
    );
    let metadata = primary.template_metadata.as_ref().unwrap();
    assert_eq!(metadata.component, "Button");
    assert!(metadata
        .classes
        .iter()
        .any(|class| class == "workbench-primary-button"));
    assert!(primary.state_flags.clickable);
    assert!(primary.state_flags.focusable);
    assert!(
        template
            .surface
            .surface_frame()
            .layout_engine_report
            .taffy_selected_count
            >= 10,
        "componentized workbench window should exercise Taffy layout across the shell"
    );

    assert_eq!(
        template.host_projection.document_id,
        WORKBENCH_WINDOW_DOCUMENT_ID
    );
    assert_eq!(
        template
            .host_projection
            .node_by_control_id(EditorWorkbenchTemplateControlIds::VIEWPORT)
            .expect("viewport host projection")
            .frame,
        template.frames.viewport
    );
    let primary_projection = template
        .host_projection
        .node_by_control_id(EditorWorkbenchTemplateControlIds::PRIMARY_BUTTON)
        .expect("primary button retained projection");
    assert_eq!(primary_projection.component, "Button");
    assert_eq!(primary_projection.text.as_deref(), Some("Primary"));
    assert!(primary_projection.routes.iter().any(|route| {
        route.binding_id == "ComponentLab/Primary" && route.event_kind == UiEventKind::Click
    }));
    let move_tool_projection = template
        .host_projection
        .node_by_control_id("WorkbenchToolMove")
        .expect("move tool retained projection");
    assert!(move_tool_projection.routes.iter().any(|route| {
        route.binding_id == "Tool/Move" && route.event_kind == UiEventKind::Click
    }));
    let effect_module_projection = template
        .host_projection
        .node_by_control_id("WorkbenchModuleEffect")
        .expect("effect module tab retained projection");
    assert_eq!(effect_module_projection.component, "ToggleButton");
    assert_eq!(effect_module_projection.text.as_deref(), Some("Effect"));
    assert!(effect_module_projection.checked);
    assert_eq!(
        effect_module_projection.properties.get("selected"),
        Some(&RetainedUiHostValue::Bool(true))
    );
    assert!(effect_module_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/Effect" && route.event_kind == UiEventKind::Click
    }));
    let material_module_projection = template
        .host_projection
        .node_by_control_id("WorkbenchModuleMaterial")
        .expect("material module tab retained projection");
    assert!(material_module_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/Material" && route.event_kind == UiEventKind::Click
    }));
    let ability_module_projection = template
        .host_projection
        .node_by_control_id("WorkbenchModuleAbility")
        .expect("ability module tab retained projection");
    assert_eq!(ability_module_projection.component, "ToggleButton");
    assert_eq!(ability_module_projection.text.as_deref(), Some("Ability"));
    assert!(ability_module_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/Ability" && route.event_kind == UiEventKind::Click
    }));
    for (control_id, binding_id) in [
        ("WorkbenchModuleTags", "WorkbenchModule/Tags"),
        ("WorkbenchModulePerception", "WorkbenchModule/Perception"),
        ("WorkbenchModuleRender", "WorkbenchModule/Render"),
        ("WorkbenchModuleHud", "WorkbenchModule/Hud"),
    ] {
        assert!(template
            .host_projection
            .node_by_control_id(control_id)
            .expect("additional module tab retained projection")
            .routes
            .iter()
            .any(|route| route.binding_id == binding_id && route.event_kind == UiEventKind::Click));
    }
    let compile_module_projection = template
        .host_projection
        .node_by_control_id("WorkbenchModuleCompile")
        .expect("compile module command retained projection");
    assert_eq!(compile_module_projection.component, "Button");
    assert_eq!(compile_module_projection.text.as_deref(), Some("Compile"));
    assert!(compile_module_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/Compile" && route.event_kind == UiEventKind::Click
    }));
    let module_workspace_projection = template
        .host_projection
        .node_by_control_id("WorkbenchMainBandModuleWorkspace")
        .expect("module workspace retained projection");
    assert_eq!(
        module_workspace_projection.properties.get("visibility"),
        Some(&RetainedUiHostValue::String("visible".to_string()))
    );
    assert_eq!(module_workspace_projection.frame, template.frames.main_band);
    let effect_workspace_projection = template
        .host_projection
        .node_by_control_id("WorkbenchModuleEffectWorkspace")
        .expect("effect workspace retained projection");
    assert_eq!(
        effect_workspace_projection.frame.width,
        template.frames.main_band.width
    );
    let effect_search_projection = template
        .host_projection
        .node_by_control_id("WorkbenchEffectAssetSearch")
        .expect("effect asset search retained projection");
    assert_eq!(effect_search_projection.component, "InputField");
    assert!(effect_search_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/EffectSearchEdit"
            && route.event_kind == UiEventKind::Change
    }));
    let effect_asset_projection = template
        .host_projection
        .node_by_control_id("WorkbenchEffectHealthRegenRow")
        .expect("effect asset row retained projection");
    assert_eq!(
        effect_asset_projection.text.as_deref(),
        Some("GE_HealthRegen")
    );
    assert!(effect_asset_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/EffectHealthRegenRow"
            && route.event_kind == UiEventKind::Click
    }));
    let effect_modifier_projection = template
        .host_projection
        .node_by_control_id("WorkbenchEffectModifierHealthRow")
        .expect("effect modifier row retained projection");
    assert!(effect_modifier_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/EffectModifierHealth"
            && route.event_kind == UiEventKind::Click
    }));
    let effect_magnitude_projection = template
        .host_projection
        .node_by_control_id("WorkbenchEffectMagnitudeField")
        .expect("effect magnitude retained projection");
    assert_eq!(
        effect_magnitude_projection.value_text.as_deref(),
        Some("Magnitude: 10.0")
    );
    assert!(effect_magnitude_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/EffectMagnitudeEdit"
            && route.event_kind == UiEventKind::Change
    }));
    for control_id in [
        "WorkbenchAbilityRailGap",
        "WorkbenchTagsRailGap",
        "WorkbenchPerceptionRailGap",
        "WorkbenchMaterialRailGap",
        "WorkbenchBehaviorRailGap",
        "WorkbenchRenderRailGap",
        "WorkbenchAssetsRailGap",
        "WorkbenchVfxRailGap",
        "WorkbenchHudRailGap",
    ] {
        assert!(template
            .host_projection
            .node_by_control_id(control_id)
            .is_some());
    }
    let material_domain_projection = template
        .host_projection
        .node_by_control_id("WorkbenchMaterialDomainDropdown")
        .expect("material domain retained projection");
    assert!(material_domain_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/MaterialDomainEdit"
            && route.event_kind == UiEventKind::Change
    }));
    let material_node_projection = template
        .host_projection
        .node_by_control_id("WorkbenchMaterialNodeRow02")
        .expect("material node row retained projection");
    assert!(material_node_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/MaterialNodeRoughness"
            && route.event_kind == UiEventKind::Click
    }));
    let behavior_node_projection = template
        .host_projection
        .node_by_control_id("WorkbenchBehaviorNodeRow03")
        .expect("behavior node row retained projection");
    assert!(behavior_node_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/BehaviorNodeCooldown"
            && route.event_kind == UiEventKind::Click
    }));
    let assets_table_projection = template
        .host_projection
        .node_by_control_id("WorkbenchAssetsTableRow03")
        .expect("assets table row retained projection");
    assert!(assets_table_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/AssetsTableTexture"
            && route.event_kind == UiEventKind::Click
    }));
    let vfx_system_projection = template
        .host_projection
        .node_by_control_id("WorkbenchVfxSystemField")
        .expect("vfx system field retained projection");
    assert!(vfx_system_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/VfxSystemEdit"
            && route.event_kind == UiEventKind::Change
    }));
    let ability_name_projection = template
        .host_projection
        .node_by_control_id("WorkbenchAbilityNameField")
        .expect("ability name retained projection");
    assert!(ability_name_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/AbilityNameEdit"
            && route.event_kind == UiEventKind::Change
    }));
    let tags_row_projection = template
        .host_projection
        .node_by_control_id("WorkbenchTagsAbilityActivateRow")
        .expect("tags row retained projection");
    assert!(tags_row_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/TagsAbilityActivate"
            && route.event_kind == UiEventKind::Click
    }));
    let perception_config_projection = template
        .host_projection
        .node_by_control_id("WorkbenchPerceptionConfigDropdown")
        .expect("perception config retained projection");
    assert!(perception_config_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/PerceptionConfigEdit"
            && route.event_kind == UiEventKind::Change
    }));
    let render_pass_projection = template
        .host_projection
        .node_by_control_id("WorkbenchRenderLightingPassRow")
        .expect("render pass retained projection");
    assert!(render_pass_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/RenderLightingPass"
            && route.event_kind == UiEventKind::Click
    }));
    let hud_screen_projection = template
        .host_projection
        .node_by_control_id("WorkbenchHudScreenDropdown")
        .expect("hud screen retained projection");
    assert!(hud_screen_projection.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/HudScreenEdit"
            && route.event_kind == UiEventKind::Change
    }));
    let position_x_projection = template
        .host_projection
        .node_by_control_id("WorkbenchTransformPositionX")
        .expect("position X retained projection");
    assert_eq!(position_x_projection.component, "InputField");
    assert_eq!(position_x_projection.value_text.as_deref(), Some("128.4"));
    let position_x_node = node_by_control(&template.surface, "WorkbenchTransformPositionX");
    assert!(position_x_node
        .template_metadata
        .as_ref()
        .is_some_and(|metadata| metadata
            .classes
            .iter()
            .any(|class| class == "workbench-axis-value-field")));
    let position_y_projection = template
        .host_projection
        .node_by_control_id("WorkbenchTransformPositionY")
        .expect("position Y retained projection");
    assert!(position_y_projection.frame.x > position_x_projection.frame.x);
    let position_z_projection = template
        .host_projection
        .node_by_control_id("WorkbenchTransformPositionZ")
        .expect("position Z retained projection");
    assert!(position_z_projection.frame.x > position_y_projection.frame.x);

    template
        .recompute_layout(&runtime, UiSize::new(1440.0, metrics.target_height))
        .unwrap();
    assert_frame(
        template.frames.root,
        UiFrame::new(0.0, 0.0, 1440.0, metrics.target_height),
    );
    assert_frame(
        template.frames.viewport,
        UiFrame::new(
            metrics.activity_rail_width + metrics.hierarchy_panel_width,
            metrics.top_bar_height,
            1440.0
                - metrics.activity_rail_width
                - metrics.hierarchy_panel_width
                - metrics.inspector_panel_width,
            metrics.upper_region_height,
        ),
    );
    assert_eq!(
        template
            .host_projection
            .node_by_control_id(EditorWorkbenchTemplateControlIds::VIEWPORT)
            .expect("viewport host projection after recompute")
            .frame,
        template.frames.viewport
    );
}

fn frame(reference: &EditorWorkbenchReferenceSurface, node_id: UiNodeId) -> UiFrame {
    reference
        .surface
        .tree
        .node(node_id)
        .unwrap()
        .layout_cache
        .frame
}

fn node_by_control<'a>(
    surface: &'a zircon_runtime::ui::surface::UiSurface,
    control_id: &str,
) -> &'a zircon_runtime_interface::ui::tree::UiTreeNode {
    surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some(control_id)
        })
        .unwrap_or_else(|| panic!("missing control `{control_id}`"))
}

fn assert_frame(actual: UiFrame, expected: UiFrame) {
    let epsilon = 0.01;
    assert!(
        (actual.x - expected.x).abs() <= epsilon
            && (actual.y - expected.y).abs() <= epsilon
            && (actual.width - expected.width).abs() <= epsilon
            && (actual.height - expected.height).abs() <= epsilon,
        "actual frame {:?} did not match expected {:?}",
        actual,
        expected
    );
}
