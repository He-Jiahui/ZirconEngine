use super::super::data::{
    FrameRect, HostDocumentDockSurfaceData, HostWindowPresentationData, PaneData,
    TemplatePaneNodeData,
};
use crate::ui::retained_host::hierarchy_pointer::HierarchyRowMetrics;

pub(in crate::ui::retained_host) fn owns_ordinary_panes(
    presentation: &HostWindowPresentationData,
) -> bool {
    presentation
        .workbench_window_nodes
        .iter()
        .any(|node| node.control_id.as_str() == "UnrealLevelEditorContent")
}

pub(in crate::ui::retained_host) const PANE_ROOT_CONTROLS: &[&str] = &[
    "WorkbenchMainBandViewportPanel",
    "WorkbenchMainBandActivityRail",
    "WorkbenchMainBandSceneTreePanel",
    "WorkbenchMainBandInspectorPanel",
];

pub(in crate::ui::retained_host) struct AuthoredPane<'a> {
    pub root_row: usize,
    pub root: &'a TemplatePaneNodeData,
    pub frame: FrameRect,
    pub module_workspace: Option<&'a TemplatePaneNodeData>,
}

pub(in crate::ui::retained_host) fn authored_panes(
    presentation: &HostWindowPresentationData,
) -> Vec<AuthoredPane<'_>> {
    // These are the same absolute projected node frames and inherited clips used
    // by the shared template painter and hit index. The actual root bounds avoid
    // treating an off-window preferred-size pane as a visible owner.
    let nodes = &presentation.workbench_window_nodes;
    let Some(bounds) = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "WorkbenchWindowRoot")
        .map(node_frame)
    else {
        return Vec::new();
    };
    nodes
        .iter()
        .enumerate()
        .filter_map(|(root_row, root)| {
            if !PANE_ROOT_CONTROLS.contains(&root.control_id.as_str()) {
                return None;
            }
            let mut frame = super::intersect_rect(&node_frame(root), &bounds)?;
            if root.has_clip_frame {
                let inherited_clip = FrameRect {
                    x: root.clip_frame.x,
                    y: root.clip_frame.y,
                    width: root.clip_frame.width,
                    height: root.clip_frame.height,
                };
                frame = super::intersect_rect(&frame, &inherited_clip)?;
            }
            Some(AuthoredPane {
                root_row,
                root,
                frame,
                module_workspace: mounted_module_workspace(nodes, root),
            })
        })
        .collect()
}

fn mounted_module_workspace<'a>(
    nodes: &'a crate::ui::retained_host::primitives::ModelRc<TemplatePaneNodeData>,
    pane: &TemplatePaneNodeData,
) -> Option<&'a TemplatePaneNodeData> {
    if pane.control_id.as_str() != "WorkbenchMainBandViewportPanel" {
        return None;
    }
    let viewport = nodes.iter().find(|node| {
        node.control_id.as_str() == "UnrealSceneViewViewport"
            && descends_from(nodes, node, pane.node_id.as_str())
    })?;
    let viewport_frame = viewport.source_surface_frame.as_ref()?;
    let viewport_id = viewport.surface_node_id?;
    nodes.iter().find(|node| {
        if node.parent_node_id != viewport.node_id
            || node.frame.width <= 0.0
            || node.frame.height <= 0.0
        {
            return false;
        }
        let (source_node_id, source_suffix) = if let Some((source_node, suffix)) =
            extension_workspace_source_identity(node.control_id.as_str())
        {
            (source_node.to_string(), suffix.to_string())
        } else {
            let (category, module) = match node.control_id.as_str() {
                "WorkbenchModuleEffectWorkspace" => ("gameplay", "effect"),
                "WorkbenchModuleMaterialWorkspace" => ("rendering", "material"),
                "WorkbenchModuleBehaviorWorkspace" => ("ai", "behavior"),
                "WorkbenchModuleAssetsWorkspace" => ("assets", "assets"),
                "WorkbenchModuleVfxWorkspace" => ("rendering", "vfx"),
                "WorkbenchModuleAbilityWorkspace" => ("gameplay", "ability"),
                "WorkbenchModuleTagsWorkspace" => ("gameplay", "tags"),
                "WorkbenchModulePerceptionWorkspace" => ("ai", "perception"),
                "WorkbenchModuleRenderWorkspace" => ("rendering", "render"),
                "WorkbenchModuleHudWorkspace" => ("ui", "hud"),
                _ => return false,
            };
            (
                format!("{module}_workspace"),
                format!("/workbench/modules/core/{category}/workbench_{module}_workspace.zui"),
            )
        };
        let Some(source) = node.source_surface_frame.as_ref() else {
            return false;
        };
        let Some(node_id) = node.surface_node_id else {
            return false;
        };
        std::sync::Arc::ptr_eq(source, viewport_frame)
            && source.tree_id == viewport_frame.tree_id
            && source.arranged_tree.get(node_id).is_some_and(|actual| {
                actual.node_path.0 == node.node_id.as_str()
                    && actual.control_id.as_deref() == Some(node.control_id.as_str())
                    && actual.parent == Some(viewport_id)
                    && actual.enabled
                    && actual.is_render_visible()
            })
            && source.arranged_tree.get(viewport_id).is_some_and(|actual| {
                actual.node_path.0 == viewport.node_id.as_str()
                    && actual.control_id.as_deref() == Some(viewport.control_id.as_str())
            })
            && node.source_node_id.as_str() == source_node_id
            && node
                .source_path
                .as_str()
                .replace('\\', "/")
                .ends_with(&source_suffix)
    })
}

/// Closed source identities for actual selected extension documents, never arbitrary overlays.
fn extension_workspace_source_identity(control_id: &str) -> Option<(&'static str, &'static str)> {
    match control_id {
        "WorkbenchExtensionParticleLibraryWorkspace" => Some(("particle_library_workspace", "/workbench/modules/extensions/rendering/workbench_extension_particle_library_workspace.zui")),
        "WorkbenchExtensionTerrainEditorWorkspace" => Some(("terrain_editor_workspace", "/workbench/modules/extensions/world/workbench_extension_terrain_editor_workspace.zui")),
        "WorkbenchExtensionFoliageEditorWorkspace" => Some(("foliage_editor_workspace", "/workbench/modules/extensions/world/workbench_extension_foliage_editor_workspace.zui")),
        "WorkbenchExtensionLevelStreamingWorkspace" => Some(("level_streaming_workspace", "/workbench/modules/extensions/world/workbench_extension_level_streaming_workspace.zui")),
        "WorkbenchExtensionLevelVariantWorkspace" => Some(("level_variant_workspace", "/workbench/modules/extensions/world/workbench_extension_level_variant_workspace.zui")),
        "WorkbenchExtensionPrefabEditorWorkspace" => Some(("prefab_editor_workspace", "/workbench/modules/extensions/world/workbench_extension_prefab_editor_workspace.zui")),
        "WorkbenchExtensionScatterEditorWorkspace" => Some(("scatter_editor_workspace", "/workbench/modules/extensions/world/workbench_extension_scatter_editor_workspace.zui")),
        "WorkbenchExtensionVolumeEditorWorkspace" => Some(("volume_editor_workspace", "/workbench/modules/extensions/world/workbench_extension_volume_editor_workspace.zui")),
        "WorkbenchExtensionWeatherEditorWorkspace" => Some(("weather_editor_workspace", "/workbench/modules/extensions/world/workbench_extension_weather_editor_workspace.zui")),
        "WorkbenchExtensionSpawnRulesWorkspace" => Some(("spawn_rules_workspace", "/workbench/modules/extensions/gameplay/workbench_extension_spawn_rules_workspace.zui")),
        "WorkbenchExtensionWorldStateWorkspace" => Some(("world_state_workspace", "/workbench/modules/extensions/gameplay/workbench_extension_world_state_workspace.zui")),
        "WorkbenchExtensionCollisionProxyWorkspace" => Some(("collision_proxy_workspace", "/workbench/modules/extensions/simulation/workbench_extension_collision_proxy_workspace.zui")),
        "WorkbenchExtensionPhysicsCollisionWorkspace" => Some(("physics_collision_workspace", "/workbench/modules/extensions/simulation/workbench_extension_physics_collision_workspace.zui")),
        "WorkbenchExtensionNavmeshAiWorkspace" => Some(("navmesh_ai_workspace", "/workbench/modules/extensions/simulation/workbench_extension_navmesh_ai_workspace.zui")),
        "WorkbenchExtensionLobbyEditorWorkspace" => Some(("lobby_editor_workspace", "/workbench/modules/extensions/multiplayer/workbench_extension_lobby_editor_workspace.zui")),
        "WorkbenchExtensionMatchmakingEditorWorkspace" => Some(("matchmaking_editor_workspace", "/workbench/modules/extensions/multiplayer/workbench_extension_matchmaking_editor_workspace.zui")),
        "WorkbenchExtensionDataTableWorkspace" => Some(("data_table_workspace", "/workbench/modules/extensions/data/workbench_extension_data_table_workspace.zui")),
        "WorkbenchExtensionSourceControlWorkspace" => Some(("source_control_workspace", "/workbench/modules/extensions/production/workbench_extension_source_control_workspace.zui")),
        "WorkbenchExtensionBuildExportWorkspace" => Some(("build_export_workspace", "/workbench/modules/extensions/production/workbench_extension_build_export_workspace.zui")),
        "WorkbenchExtensionAutomationReportWorkspace" => Some(("automation_report_workspace", "/workbench/modules/extensions/production/workbench_extension_automation_report_workspace.zui")),
        "WorkbenchExtensionProjectOverviewWorkspace" => Some(("project_overview_workspace", "/workbench/modules/extensions/production/workbench_extension_project_overview_workspace.zui")),
        "WorkbenchExtensionPluginManagerWorkspace" => Some(("plugin_manager_workspace", "/workbench/modules/extensions/production/workbench_extension_plugin_manager_workspace.zui")),
        "WorkbenchExtensionSaveDataWorkspace" => Some(("save_data_workspace", "/workbench/modules/extensions/data/workbench_extension_save_data_workspace.zui")),
        "WorkbenchExtensionSequencerWorkspace" => Some(("sequencer_workspace", "/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui")),
        "WorkbenchExtensionMontageEditorWorkspace" => Some(("montage_editor_workspace", "/workbench/modules/extensions/animation/workbench_extension_montage_editor_workspace.zui")),
        "WorkbenchExtensionBlendSpaceWorkspace" => Some(("blend_space_workspace", "/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui")),
        "WorkbenchExtensionPoseLibraryWorkspace" => Some(("pose_library_workspace", "/workbench/modules/extensions/animation/workbench_extension_pose_library_workspace.zui")),
        "WorkbenchExtensionRetargetWorkspace" => Some(("retarget_workspace", "/workbench/modules/extensions/animation/workbench_extension_retarget_workspace.zui")),
        "WorkbenchExtensionControlRigWorkspace" => Some(("control_rig_workspace", "/workbench/modules/extensions/animation/workbench_extension_control_rig_workspace.zui")),
        "WorkbenchExtensionMotionMatchingWorkspace" => Some(("motion_matching_workspace", "/workbench/modules/extensions/animation/workbench_extension_motion_matching_workspace.zui")),
        "WorkbenchExtensionAnimationCompressionWorkspace" => Some(("animation_compression_workspace", "/workbench/modules/extensions/animation/workbench_extension_animation_compression_workspace.zui")),
        "WorkbenchExtensionShaderEditorWorkspace" => Some(("shader_editor_workspace", "/workbench/modules/extensions/rendering/workbench_extension_shader_editor_workspace.zui")),
        "WorkbenchExtensionLightingBakeWorkspace" => Some(("lighting_bake_workspace", "/workbench/modules/extensions/rendering/workbench_extension_lighting_bake_workspace.zui")),
        "WorkbenchExtensionPostProcessWorkspace" => Some(("post_process_workspace", "/workbench/modules/extensions/rendering/workbench_extension_post_process_workspace.zui")),
        "WorkbenchExtensionConsoleDiagnosticsWorkspace" => Some(("console_diagnostics_workspace", "/workbench/modules/extensions/diagnostics/workbench_extension_console_diagnostics_workspace.zui")),
        "WorkbenchExtensionRuntimeDiagnosticsWorkspace" => Some(("runtime_diagnostics_workspace", "/workbench/modules/extensions/diagnostics/workbench_extension_runtime_diagnostics_workspace.zui")),
        "WorkbenchExtensionTelemetryDashboardWorkspace" => Some(("telemetry_dashboard_workspace", "/workbench/modules/extensions/diagnostics/workbench_extension_telemetry_dashboard_workspace.zui")),
        "WorkbenchExtensionPerformanceWorkspace" => Some(("performance_workspace", "/workbench/modules/extensions/diagnostics/workbench_extension_performance_workspace.zui")),
        "WorkbenchExtensionFontAtlasWorkspace" => Some(("font_atlas_workspace", "/workbench/modules/extensions/ui/workbench_extension_font_atlas_workspace.zui")),
        "WorkbenchExtensionMenuFlowWorkspace" => Some(("menu_flow_workspace", "/workbench/modules/extensions/ui/workbench_extension_menu_flow_workspace.zui")),
        "WorkbenchExtensionAccessibilityAuditWorkspace" => Some(("accessibility_audit_workspace", "/workbench/modules/extensions/ui/workbench_extension_accessibility_audit_workspace.zui")),
        "WorkbenchExtensionIconLibraryWorkspace" => Some(("icon_library_workspace", "/workbench/modules/extensions/ui/workbench_extension_icon_library_workspace.zui")),
        "WorkbenchExtensionUiBindingWorkspace" => Some(("ui_binding_workspace", "/workbench/modules/extensions/ui/workbench_extension_ui_binding_workspace.zui")),
        "WorkbenchExtensionUiAssetEditorWorkspace" => Some(("ui_asset_editor_workspace", "/workbench/modules/extensions/ui/workbench_extension_ui_asset_editor_workspace.zui")),
        _ => None,
    }
}

pub(in crate::ui::retained_host) struct AuthoredLevelViewport<'a> {
    pub root: &'a TemplatePaneNodeData,
    pub media: &'a TemplatePaneNodeData,
    pub frame: FrameRect,
    pub clip: FrameRect,
}

/// One authored viewport belongs to the document region containing its complete source frame.
/// Resolve each leaf independently: a Scene leaf cannot suppress a neighboring document editor.
pub(in crate::ui::retained_host) fn authored_level_viewport<'a>(
    presentation: &'a HostWindowPresentationData,
    document: &HostDocumentDockSurfaceData,
) -> Option<AuthoredLevelViewport<'a>> {
    if !owns_ordinary_panes(presentation)
        || !matches!(document.pane.kind.as_str(), "Scene" | "Game")
        || document.surface_key.is_empty()
    {
        return None;
    }
    let nodes = &presentation.workbench_window_nodes;
    for pane in authored_panes(presentation) {
        if pane.root.control_id.as_str() != "WorkbenchMainBandViewportPanel" {
            continue;
        }
        let Some(media) = nodes.iter().find(|node| {
            node.control_id.as_str() == "WorkbenchViewportSurface"
                && descends_from(nodes, node, pane.root.node_id.as_str())
        }) else {
            continue;
        };
        let frame = node_frame(media);
        if !rect_contains_rect(&document.region_frame, &frame) {
            continue;
        }
        let Some(mut clip) = super::intersect_rect(&frame, &pane.frame) else {
            continue;
        };
        if media.has_clip_frame {
            let inherited = FrameRect {
                x: media.clip_frame.x,
                y: media.clip_frame.y,
                width: media.clip_frame.width,
                height: media.clip_frame.height,
            };
            let Some(clipped) = super::intersect_rect(&clip, &inherited) else {
                continue;
            };
            clip = clipped;
        }
        return Some(AuthoredLevelViewport {
            root: pane.root,
            media,
            frame,
            clip,
        });
    }
    None
}

pub(in crate::ui::retained_host) fn owns_authored_level_viewport(
    presentation: &HostWindowPresentationData,
    document: &HostDocumentDockSurfaceData,
) -> bool {
    authored_level_viewport(presentation, document).is_some()
}

fn rect_contains_rect(outer: &FrameRect, inner: &FrameRect) -> bool {
    const ROUNDING_TOLERANCE: f32 = 0.5;
    inner.width > 0.0
        && inner.height > 0.0
        && inner.x >= outer.x - ROUNDING_TOLERANCE
        && inner.y >= outer.y - ROUNDING_TOLERANCE
        && inner.x + inner.width <= outer.x + outer.width + ROUNDING_TOLERANCE
        && inner.y + inner.height <= outer.y + outer.height + ROUNDING_TOLERANCE
}

fn descends_from(
    nodes: &crate::ui::retained_host::primitives::ModelRc<TemplatePaneNodeData>,
    node: &TemplatePaneNodeData,
    root_id: &str,
) -> bool {
    let mut current = node;
    for _ in 0..nodes.row_count() {
        if current.node_id.as_str() == root_id {
            return true;
        }
        if current.parent_node_id.is_empty() {
            return false;
        }
        let Some(parent) = nodes
            .iter()
            .find(|candidate| candidate.node_id.as_str() == current.parent_node_id.as_str())
        else {
            return false;
        };
        current = parent;
    }
    false
}

pub(in crate::ui::retained_host) struct AuthoredHierarchy<'a> {
    pub pane: &'a PaneData,
    pub viewport: FrameRect,
    pub prototype: &'a TemplatePaneNodeData,
    pub metrics: HierarchyRowMetrics,
}

/// The complete runtime hierarchy stays the data owner. The authored viewport and row
/// prototype provide geometry and style to its bounded native row renderer and input bridge.
pub(in crate::ui::retained_host) fn authored_hierarchy(
    presentation: &HostWindowPresentationData,
) -> Option<AuthoredHierarchy<'_>> {
    let nodes = &presentation.workbench_window_nodes;
    let viewport = nodes
        .iter()
        .find(|n| {
            n.control_id.as_str() == "WorkbenchSceneTree"
                && n.frame.width > 0.0
                && n.frame.height > 0.0
        })
        .map(node_frame)?;
    let prototype = nodes
        .iter()
        .find(|n| is_authored_tree_row(n) && n.frame.height > 0.0)?;
    let next = nodes
        .iter()
        .filter(|n| is_authored_tree_row(n) && n.frame.y > prototype.frame.y)
        .map(|n| n.frame.y)
        .min_by(f32::total_cmp);
    let gap = next
        .map(|y| (y - prototype.frame.y - prototype.frame.height).max(0.0))
        .unwrap_or_else(|| {
            crate::ui::retained_host::host_contract::paint_theme::current_host_metrics()
                .border_width
        });
    let pane = [
        &presentation.host_scene_data.left_dock.pane,
        &presentation.host_scene_data.right_dock.pane,
    ]
    .into_iter()
    .find(|pane| pane.kind.as_str() == "Hierarchy")?;
    Some(AuthoredHierarchy {
        pane,
        metrics: HierarchyRowMetrics {
            row_x: (prototype.frame.x - viewport.x).max(0.0),
            row_y: (prototype.frame.y - viewport.y).max(0.0),
            row_height: prototype.frame.height,
            row_gap: gap,
            row_width_inset: (viewport.width - prototype.frame.width).max(0.0),
        },
        viewport,
        prototype,
    })
}

pub(in crate::ui::retained_host) fn is_authored_tree_row(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.control_id.as_str(),
        "WorkbenchSceneRootItem"
            | "WorkbenchSceneEnvironmentItem"
            | "WorkbenchSceneLevelItem"
            | "WorkbenchScenePropsItem"
            | "WorkbenchScenePlayerItem"
            | "WorkbenchSceneAudioItem"
            | "WorkbenchSceneSlot07Item"
            | "WorkbenchSceneSlot08Item"
            | "WorkbenchSceneSlot09Item"
            | "WorkbenchSceneSlot10Item"
    )
}

pub(in crate::ui::retained_host) fn point_in_frame(frame: &FrameRect, x: f32, y: f32) -> bool {
    super::super::frame_geometry::contains_point(frame, x, y)
}

fn node_frame(node: &TemplatePaneNodeData) -> FrameRect {
    FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    }
}

#[cfg(test)]
#[path = "tests/panes.rs"]
mod tests;

#[cfg(test)]
mod mounted_module_tests {
    use super::*;
    use crate::tests::support::env_lock;
    use crate::ui::retained_host::callback_dispatch::template_bridge::{
        BuiltinHostWindowTemplateBridgeError, BuiltinWorkbenchWindowTemplateSurfaceBridge,
    };
    use crate::ui::retained_host::host_contract::{
        paint_frame::HostRgbaFrame,
        surface_hit_test::{
            hit_test_workbench_window_template_node_with_index, HostWorkbenchHitIndex,
        },
    };
    use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize};

    const MODULE_BUTTONS: &[(&str, &str, &str)] = &[
        (
            "menu.item.effect",
            "WorkbenchEffectApplyButton",
            "WorkbenchModule/EffectApply",
        ),
        (
            "menu.item.material",
            "WorkbenchMaterialCompileButton",
            "WorkbenchModule/MaterialCompile",
        ),
        (
            "menu.item.behavior",
            "WorkbenchBehaviorValidateButton",
            "WorkbenchModule/BehaviorValidate",
        ),
        (
            "menu.item.assets",
            "WorkbenchAssetsImportButton",
            "WorkbenchModule/AssetsImport",
        ),
        (
            "menu.item.v_f_x",
            "WorkbenchVfxSimulateButton",
            "WorkbenchModule/VfxSimulate",
        ),
        (
            "menu.item.ability",
            "WorkbenchAbilityPlaytestButton",
            "WorkbenchModule/AbilityPlaytest",
        ),
        (
            "menu.item.tags",
            "WorkbenchTagsAddButton",
            "WorkbenchModule/TagsAdd",
        ),
        (
            "menu.item.perception",
            "WorkbenchPerceptionSimulateButton",
            "WorkbenchModule/PerceptionSimulate",
        ),
        (
            "menu.item.render",
            "WorkbenchRenderCompileButton",
            "WorkbenchModule/RenderCompile",
        ),
        (
            "menu.item.h_u_d",
            "WorkbenchHudPreviewButton",
            "WorkbenchModule/HudPreview",
        ),
    ];

    fn select(bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge, item: &str) {
        assert!(bridge
            .dispatch_control_state("MenuSlot2", UiEventKind::Click)
            .unwrap()
            .is_some());
        assert!(bridge
            .dispatch_workbench_module_overflow_menu_item_state("UnrealWindowMenuPopup", item)
            .unwrap()
            .is_some());
    }

    fn presentation(
        bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    ) -> HostWindowPresentationData {
        let mut presentation = HostWindowPresentationData {
            workbench_window_nodes:
                crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes(Some(
                    bridge.host_projection(),
                )),
            ..Default::default()
        };
        presentation.host_layout.center_band_frame = node_frame(
            presentation
                .workbench_window_nodes
                .iter()
                .find(|node| node.control_id.as_str() == "UnrealLevelEditorContent")
                .unwrap(),
        );
        presentation.host_layout.status_bar_frame = node_frame(
            presentation
                .workbench_window_nodes
                .iter()
                .find(|node| {
                    matches!(
                        node.control_id.as_str(),
                        "WorkbenchWindowStatusBarRegion" | "WorkbenchStatusBar"
                    )
                })
                .unwrap(),
        );
        presentation
    }

    #[test]
    fn selected_modules_produce_visible_source_bound_paint_hits_and_live_events() {
        let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
        BuiltinWorkbenchWindowTemplateSurfaceBridge::exercise_core_module_sources_for_test();
    }

    #[test]
    fn module_pane_ownership_rejects_unproven_source_identity() {
        let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
        select(&mut bridge, "menu.item.effect");
        let mut presentation = presentation(&bridge);
        let button = presentation
            .workbench_window_nodes
            .iter()
            .find(|node| node.control_id.as_str() == "WorkbenchEffectApplyButton")
            .unwrap()
            .clone();
        let mut nodes = presentation
            .workbench_window_nodes
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        nodes
            .iter_mut()
            .find(|node| node.control_id.as_str() == "WorkbenchModuleEffectWorkspace")
            .unwrap()
            .source_surface_frame = None;
        presentation.workbench_window_nodes = crate::ui::retained_host::primitives::ModelRc::from(
            std::rc::Rc::new(crate::ui::retained_host::primitives::VecModel::from(nodes)),
        );
        assert!(!authored_panes(&presentation)
            .into_iter()
            .any(|pane| pane.module_workspace.is_some()));
        let index = HostWorkbenchHitIndex::from_presentation(&presentation);
        assert!(hit_test_workbench_window_template_node_with_index(
            &presentation,
            &index,
            button.frame.x + button.frame.width * 0.5,
            button.frame.y + button.frame.height * 0.5
        )
        .is_none());
    }

    #[test]
    fn stale_extension_proof_preserves_render_paint_hits_and_workspace_state() {
        let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
        select(&mut bridge, "menu.item.render");
        assert!(bridge.has_control("WorkbenchRenderTools"));
        assert!(!bridge.has_control("WorkbenchExtensionModuleWorkspacesHost"));
        assert!(!bridge.has_control("WorkbenchExtensionShaderEditorWorkspace"));
        for _ in 0..2 {
            let reject_stage = bridge.stage_shader_source_then_change_live_focus_for_test();
            let before_projection = bridge.host_projection().clone();
            let before_tree = bridge.surface().tree.clone();
            let before_navigation = bridge.surface().navigation.clone();
            let before_focus = bridge.surface().focus.clone();
            let before_input = bridge.surface().input.clone();
            let before_presentation = presentation(&bridge);
            let before_source = before_presentation
                .workbench_window_nodes
                .iter()
                .find(|node| node.control_id.as_str() == "WorkbenchModuleRenderWorkspace")
                .unwrap()
                .source_surface_frame
                .as_ref()
                .unwrap()
                .clone();
            let before_workspace_frame = bridge
                .control_frame("WorkbenchModuleRenderWorkspace")
                .expect("selected Render workspace must be visible");
            let presentation = presentation(&bridge);
            assert!(authored_panes(&presentation).into_iter().any(|pane| {
                pane.module_workspace.is_some_and(|root| {
                    root.control_id.as_str() == "WorkbenchModuleRenderWorkspace"
                })
            }));
            let node = presentation
                .workbench_window_nodes
                .iter()
                .find(|node| node.control_id.as_str() == "WorkbenchRenderTools")
                .expect("the live Render gateway must remain in the host projection");
            let source_frame = node
                .source_surface_frame
                .as_ref()
                .expect("live source publication");
            let command_ref = node
                .surface_render_command_ref
                .expect("visible Render gateway paint");
            let mut frame = HostRgbaFrame::recording_only(1672, 941);
            crate::ui::retained_host::host_contract::paint_workbench::draw_workbench_presentation_commands(&mut frame, &presentation);
            let recorded = frame.into_recorded_frame();
            assert!(
                recorded.commands.iter().any(|command| {
                    command.frame.width > 0.0
                        && command.frame.height > 0.0
                        && command.source.is_some_and(|source| {
                            source.command_ref == command_ref
                                && recorded
                                    .render_sources
                                    .resolve(source.surface_key)
                                    .is_some_and(|published| {
                                        std::sync::Arc::ptr_eq(published, source_frame)
                                    })
                        })
                }),
                "the Render gateway must emit paint before and after stale child rejection"
            );
            let index = HostWorkbenchHitIndex::from_presentation(&presentation);
            let hit = hit_test_workbench_window_template_node_with_index(
                &presentation,
                &index,
                node.frame.x + node.frame.width * 0.5,
                node.frame.y + node.frame.height * 0.5,
            )
            .expect("the visible Render gateway must remain interactive");
            assert_eq!(hit.control_id.as_str(), "WorkbenchRenderTools");
            let error = reject_stage(&mut bridge)
                .expect_err("an actual stale public proof cannot overwrite the live Render owner");
            assert!(
                matches!(error, zircon_runtime_interface::ui::v2::UiV2AssetError::InvalidDocument { detail, .. }
                if detail == "child mount commit no longer matches the staged source state or session")
            );
            assert_eq!(bridge.host_projection(), &before_projection);
            assert_eq!(bridge.surface().tree, before_tree);
            assert_eq!(bridge.surface().navigation, before_navigation);
            assert_eq!(bridge.surface().focus, before_focus);
            assert_eq!(bridge.surface().input, before_input);
            let current_presentation = self::presentation(&bridge);
            let current_source = current_presentation
                .workbench_window_nodes
                .iter()
                .find(|node| node.control_id.as_str() == "WorkbenchModuleRenderWorkspace")
                .unwrap()
                .source_surface_frame
                .as_ref()
                .unwrap();
            assert!(std::sync::Arc::ptr_eq(current_source, &before_source));
            assert_eq!(
                bridge.control_frame("WorkbenchModuleRenderWorkspace"),
                Some(before_workspace_frame),
            );
            assert!(!bridge.has_control("WorkbenchExtensionShaderEditorWorkspace"));
        }
        assert!(bridge
            .dispatch_binding_state_for_control(
                "WorkbenchRenderCompileButton",
                "WorkbenchModule/RenderCompile",
            )
            .unwrap()
            .is_some());
    }

    #[test]
    fn particle_library_gateway_and_extension_share_live_source_bound_paint_hits_and_actions() {
        let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
        BuiltinWorkbenchWindowTemplateSurfaceBridge::exercise_particle_library_sources_for_test();
    }
    fn verified_mounted_sources_for_test(
        bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
        root: &TemplatePaneNodeData,
    ) -> Vec<zircon_runtime::ui::v2::UiV2SourceFileReceipt> {
        let receipts = bridge.mounted_module_source_receipts_for_test();
        assert!(
            !receipts.is_empty(),
            "actual mounted source closure is required"
        );
        let selected = receipts
            .iter()
            .find(|receipt| receipt.source_path.as_deref() == Some(root.source_path.as_str()))
            .expect("the current projected root must retain its actual physical source receipt");
        let mut cache = zircon_runtime::ui::v2::UiV2PrototypeStoreFileCache::new();
        assert!(cache.persistent_store().is_none());
        let loaded = cache
            .load_store([
                selected.physical_path.clone(),
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("assets/ui/theme/editor_workbench_strict.zui"),
            ])
            .expect("fresh real physical source closure must load");
        assert!(loaded.unresolved_imports.is_empty());
        assert_eq!(loaded.root_asset_id, selected.asset_id);
        assert_eq!(loaded.source_receipts, receipts);
        assert_eq!(
            loaded.root_document.asset.kind,
            zircon_runtime_interface::ui::v2::UiV2AssetKind::Component
        );
        assert!(loaded.root_document.root.is_none());
        assert!(loaded.root_document.components.values()
            .any(|component| component.root.as_str() == root.source_node_id.as_str()),
            "the current published root must be an authored root of the selected component document");
        receipts
    }

    impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
        pub(crate) fn exercise_core_module_sources_for_test(
        ) -> Vec<zircon_runtime::ui::v2::UiV2SourceFileReceipt> {
            let mut bridge =
                BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
                    .unwrap();
            let baseline_ids = bridge
                .surface()
                .tree
                .nodes
                .keys()
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            let mut consumed_sources = Vec::new();
            for (item, control_id, binding_id) in MODULE_BUTTONS {
                select(&mut bridge, item);
                let presentation = presentation(&bridge);
                assert!(authored_panes(&presentation)
                    .into_iter()
                    .any(|pane| pane.module_workspace.is_some()));
                let root = authored_panes(&presentation)
                    .into_iter()
                    .find_map(|pane| pane.module_workspace)
                    .expect("selected core must have its actual authored source owner");
                let receipts = verified_mounted_sources_for_test(&bridge, root);
                let node = presentation
                    .workbench_window_nodes
                    .iter()
                    .find(|node| node.control_id.as_str() == *control_id)
                    .unwrap();
                let source_frame = node
                    .source_surface_frame
                    .as_ref()
                    .expect("live source publication");
                assert!(std::sync::Arc::ptr_eq(
                    source_frame,
                    root.source_surface_frame
                        .as_ref()
                        .expect("actual core root source frame"),
                ));
                let command_ref = node
                    .surface_render_command_ref
                    .expect("visible module button paint source");
                let mut frame = HostRgbaFrame::recording_only(1672, 941);
                crate::ui::retained_host::host_contract::paint_workbench::draw_workbench_presentation_commands(&mut frame, &presentation);
                let recorded = frame.into_recorded_frame();
                assert!(
                    recorded.commands.iter().any(|command| {
                        command.frame.width > 0.0
                            && command.frame.height > 0.0
                            && command.source.is_some_and(|source| {
                                source.command_ref == command_ref
                                    && recorded
                                        .render_sources
                                        .resolve(source.surface_key)
                                        .is_some_and(|published| {
                                            std::sync::Arc::ptr_eq(published, source_frame)
                                        })
                            })
                    }),
                    "{control_id} must actually emit visible paint with its live source identity"
                );
                let index = HostWorkbenchHitIndex::from_presentation(&presentation);
                let hit = hit_test_workbench_window_template_node_with_index(
                    &presentation,
                    &index,
                    node.frame.x + node.frame.width * 0.5,
                    node.frame.y + node.frame.height * 0.5,
                )
                .expect("painted module button must be a live host hit");
                assert_eq!(hit.control_id.as_str(), *control_id);
                assert!(bridge
                    .dispatch_binding_state_for_control(hit.control_id.as_str(), binding_id)
                    .unwrap()
                    .is_some());
                select(&mut bridge, "menu.item.scene");
                assert!(!bridge.has_control(control_id));
                assert!(bridge.binding_by_id(binding_id).is_none());
                assert!(bridge.mounted_module_source_receipts_for_test().is_empty());
                assert!(!authored_panes(&self::presentation(&bridge))
                    .into_iter()
                    .any(|pane| pane.module_workspace.is_some()));
                assert_eq!(
                    bridge
                        .surface()
                        .tree
                        .nodes
                        .keys()
                        .copied()
                        .collect::<std::collections::BTreeSet<_>>(),
                    baseline_ids
                );
                assert!(bridge
                    .surface()
                    .focus
                    .focused
                    .is_none_or(|node| baseline_ids.contains(&node)));
                consumed_sources.extend(receipts);
            }
            select(&mut bridge, "menu.item.scene");
            assert!(!authored_panes(&presentation(&bridge))
                .into_iter()
                .any(|pane| pane.module_workspace.is_some()));
            consumed_sources
        }

        pub(crate) fn exercise_particle_library_sources_for_test(
        ) -> Vec<zircon_runtime::ui::v2::UiV2SourceFileReceipt> {
            let harness = crate::tests::editor_event::support::EventRuntimeHarness::new(
                "zircon_particle_library_live_paint_hit_callback",
            );
            let mut bridge =
                BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
                    .unwrap();
            let baseline_ids = bridge
                .surface()
                .tree
                .nodes
                .keys()
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            let mut consumed_sources = Vec::new();
            select(&mut bridge, "menu.item.v_f_x");
            for (control_id, root_id, action_id) in [
                (
                    "WorkbenchVfxParticleLibraryButton",
                    "WorkbenchModuleVfxWorkspace",
                    "workbench.extension.particle_library.open",
                ),
                (
                    "WorkbenchExtensionParticleLibrarySimulateButton",
                    "WorkbenchExtensionParticleLibraryWorkspace",
                    "workbench.extension.particle_library.simulate.invoke",
                ),
            ] {
                let presentation = presentation(&bridge);
                let pane = authored_panes(&presentation)
                    .into_iter()
                    .find(|pane| {
                        pane.module_workspace
                            .is_some_and(|root| root.control_id.as_str() == root_id)
                    })
                    .expect("selected workspace must retain the shared authored pane owner");
                let root = pane.module_workspace.unwrap();
                let expected_source = if root_id == "WorkbenchModuleVfxWorkspace" {
                    "/workbench/modules/core/rendering/workbench_vfx_workspace.zui"
                } else {
                    assert_eq!(root.source_node_id.as_str(), "particle_library_workspace");
                    "/workbench/modules/extensions/rendering/workbench_extension_particle_library_workspace.zui"
                };
                assert!(root
                    .source_path
                    .as_str()
                    .replace('\\', "/")
                    .ends_with(expected_source));
                let receipts = verified_mounted_sources_for_test(&bridge, root);
                let node = presentation
                    .workbench_window_nodes
                    .iter()
                    .find(|node| node.control_id.as_str() == control_id)
                    .unwrap();
                let source_frame = node
                    .source_surface_frame
                    .as_ref()
                    .expect("live source publication");
                assert!(std::sync::Arc::ptr_eq(
                    source_frame,
                    root.source_surface_frame.as_ref().unwrap()
                ));
                let command_ref = node
                    .surface_render_command_ref
                    .expect("actual visible paint source");
                let mut frame = HostRgbaFrame::recording_only(1672, 941);
                crate::ui::retained_host::host_contract::paint_workbench::draw_workbench_presentation_commands(&mut frame, &presentation);
                let recorded = frame.into_recorded_frame();
                assert!(recorded.commands.iter().any(|command| {
                    command.frame.width > 0.0
                        && command.frame.height > 0.0
                        && command.source.is_some_and(|source| {
                            source.command_ref == command_ref
                                && recorded
                                    .render_sources
                                    .resolve(source.surface_key)
                                    .is_some_and(|published| {
                                        std::sync::Arc::ptr_eq(published, source_frame)
                                    })
                        })
                }));
                let index = HostWorkbenchHitIndex::from_presentation(&presentation);
                let hit = hit_test_workbench_window_template_node_with_index(
                    &presentation,
                    &index,
                    node.frame.x + node.frame.width * 0.5,
                    node.frame.y + node.frame.height * 0.5,
                )
                .expect("source-bound visible control must remain a host hit");
                assert_eq!(hit.control_id.as_str(), control_id);
                let binding = bridge
                    .binding_for_control(hit.control_id.as_str(), UiEventKind::Click)
                    .expect("real hit must own its authored source binding");
                assert_eq!(
                    binding.payload(),
                    &crate::ui::binding::EditorUiBindingPayload::menu_action(action_id)
                );
                bridge.mark_host_projection_committed();
                let previous_sequence = harness
                    .runtime
                    .journal()
                    .records()
                    .last()
                    .map(|record| record.sequence.0);
                crate::ui::retained_host::callback_dispatch::dispatch_componentized_workbench_control(
                    &harness.runtime,
                    &mut bridge,
                    hit.control_id.as_str(),
                    UiEventKind::Click,
                )
                .expect("real hit must enter the production callback")
                .expect("existing host preview admission must succeed");
                let journal = harness.runtime.journal();
                let record = journal.records().last().unwrap();
                if let Some(previous_sequence) = previous_sequence {
                    assert_eq!(record.sequence.0, previous_sequence + 1);
                }
                assert_eq!(
                    record.operation_arguments.as_ref().unwrap()["action_id"].as_str(),
                    Some(action_id)
                );
                assert!(
                    record.operation_id.is_none(),
                    "local feedback must keep its existing admission"
                );
                assert!(
                    bridge.has_pending_host_projection_commit()
                        || bridge.has_pending_surface_state_change()
                );
                consumed_sources.extend(receipts);
            }
            assert!(!bridge.has_control("WorkbenchModuleVfxWorkspace"));
            select(&mut bridge, "menu.item.scene");
            assert!(!authored_panes(&presentation(&bridge))
                .into_iter()
                .any(|pane| pane.module_workspace.is_some()));
            assert!(!bridge.has_control("WorkbenchExtensionParticleLibraryWorkspace"));
            assert!(bridge.mounted_module_source_receipts_for_test().is_empty());
            assert_eq!(
                bridge
                    .surface()
                    .tree
                    .nodes
                    .keys()
                    .copied()
                    .collect::<std::collections::BTreeSet<_>>(),
                baseline_ids
            );
            assert!(bridge
                .surface()
                .focus
                .focused
                .is_none_or(|node| baseline_ids.contains(&node)));
            consumed_sources
        }
    }
}
