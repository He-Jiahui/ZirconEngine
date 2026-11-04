use std::{rc::Rc, sync::Arc};

use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize, surface::UiSurfaceFrame};

use crate::tests::support::env_lock;
use crate::ui::retained_host::callback_dispatch::template_bridge::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use crate::ui::retained_host::host_contract::componentized_workbench_regions::authored_panes;
use crate::ui::retained_host::host_contract::surface_hit_test::HostWorkbenchHitIndex;

use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostViewportImageData, HostWindowPresentationData, TemplateNodeFrameData,
    TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::host_contract::paint_template_nodes::TemplateNodePaintTransform;
use crate::ui::retained_host::primitives::{ModelRc, VecModel};

use super::{draw_componentized_workbench_window, ComponentizedChromeFallbackTransform};

const SENTINEL: [u8; 4] = [241, 17, 193, 255];

const WINDOW_WIDTH: u32 = 1672;
const WINDOW_HEIGHT: u32 = 941;
const BLEND_SPACE_ROOT: &str = "WorkbenchExtensionBlendSpaceWorkspace";

fn window_bridge() -> BuiltinWorkbenchWindowTemplateSurfaceBridge {
    BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WINDOW_WIDTH as f32,
        WINDOW_HEIGHT as f32,
    ))
    .expect("the current production Unreal window must load")
}

fn select_module(bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge, item: &str) {
    bridge
        .dispatch_control_state("MenuSlot2", UiEventKind::Click)
        .expect("the real Window control must open its popup")
        .expect("the Window trigger must expose its live typed binding");
    bridge
        .dispatch_workbench_module_overflow_menu_item_state("UnrealWindowMenuPopup", item)
        .expect("the actual Window item must select its workspace")
        .expect("the actual selection must expose its typed binding");
}

fn select_blend_space(bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge) {
    select_module(bridge, "menu.item.ability");
    assert!(bridge.has_control("WorkbenchAbilityBlendSpaceButton"));
    bridge
        .dispatch_control_state("WorkbenchAbilityBlendSpaceButton", UiEventKind::Click)
        .expect("the existing Ability gateway must mount BlendSpace")
        .expect("the gateway must expose its declared typed binding");
    assert!(bridge.has_control(BLEND_SPACE_ROOT));
}

fn current_presentation(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
) -> HostWindowPresentationData {
    let mut presentation = HostWindowPresentationData {
        workbench_window_nodes:
            crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes(Some(
                bridge.host_projection(),
            )),
        ..Default::default()
    };
    presentation.host_layout.center_band_frame =
        node_frame(node(&presentation, "UnrealLevelEditorContent"));
    presentation.host_layout.status_bar_frame =
        node_frame(node(&presentation, "WorkbenchWindowStatusBarRegion"));
    presentation
}

#[test]
fn native_play_images_survive_the_full_workbench_composition() {
    use crate::core::commands::{CommandEvalCtx, EditorCommandRegistry};
    use crate::core::editor_message::PlayStateKind;
    use crate::ui::retained_host::host_contract::data::{HostDocumentDockSurfaceData, PaneData};
    use crate::ui::workbench::model::RunControlsModel;
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    // Painting resolves icons asynchronously. Keep this test's loader owner alive,
    // independent of any earlier native-host fixture's retired runtime.
    let asset_owner = crate::ui::retained_host::UiHostWindow::new().unwrap();
    asset_owner.bind_visual_asset_jobs(crate::core::jobs::test_job_system(), Arc::new(|| {}));
    let mut bridge = window_bridge();
    let context = CommandEvalCtx::interactive()
        .with_project_open(true)
        .with_play_state(PlayStateKind::Playing);
    bridge
        .apply_run_controls(RunControlsModel::from_commands(
            &EditorCommandRegistry::default_workbench(),
            &context,
        ))
        .unwrap();
    bridge.refresh_prepared_state_change().unwrap();
    let mut presentation = current_presentation(&bridge);
    presentation.host_layout.center_band_frame =
        node_frame(node(&presentation, "WorkbenchWindowMainBandRegion"));
    let media = node_frame(node(&presentation, "WorkbenchViewportSurface"));
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document:test-play".to_owned(),
        region_frame: presentation.host_layout.center_band_frame.clone(),
        content_frame: media.clone(),
        pane: PaneData {
            kind: "Game".into(),
            ..Default::default()
        },
        ..Default::default()
    };
    let color = [19, 197, 83, 255];
    presentation
        .viewport_images
        .replace_game(HostViewportImageData {
            resource_key: "play-viewport:test-composition".into(),
            width: 1,
            height: 1,
            rgba: Some(Arc::from(color)),
            ..Default::default()
        });
    let mut frame = HostRgbaFrame::filled(WINDOW_WIDTH, WINDOW_HEIGHT, SENTINEL);
    let stop = node_frame(node(&presentation, "WorkbenchRunStop"));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let visible_glyph = loop {
        draw_componentized_workbench_window(&mut frame, &presentation);
        let visible = (stop.y as usize..(stop.y + stop.height) as usize).any(|y| {
            (stop.x as usize..(stop.x + stop.width) as usize).any(|x| {
                let color = pixel(&frame, x, y);
                color[..3].iter().all(|channel| *channel > 120)
            })
        });
        if visible || std::time::Instant::now() >= deadline {
            break visible;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    assert_eq!(
        pixel(
            &frame,
            (media.x + media.width * 0.5) as usize,
            (media.y + media.height * 0.5) as usize
        ),
        color,
        "the Game product must remain visible after authored pane composition"
    );
    assert!(
        visible_glyph,
        "the visible Stop control must paint its authored glyph"
    );
}

fn node<'a>(
    presentation: &'a HostWindowPresentationData,
    control: &str,
) -> &'a TemplatePaneNodeData {
    presentation
        .workbench_window_nodes
        .iter()
        .find(|node| node.control_id.as_str() == control)
        .unwrap_or_else(|| panic!("{control} must come from the current published projection"))
}

fn node_frame(node: &TemplatePaneNodeData) -> FrameRect {
    FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    }
}

fn window_bounds() -> FrameRect {
    FrameRect {
        x: 0.0,
        y: 0.0,
        width: WINDOW_WIDTH as f32,
        height: WINDOW_HEIGHT as f32,
    }
}

fn module_clip(presentation: &HostWindowPresentationData) -> FrameRect {
    authored_panes(presentation)
        .into_iter()
        .find(|pane| {
            pane.root.control_id.as_str() == "WorkbenchMainBandViewportPanel"
                && pane
                    .module_workspace
                    .is_some_and(|workspace| workspace.control_id.as_str() == BLEND_SPACE_ROOT)
        })
        .expect("the actual viewport pane must own the selected canonical workspace")
        .frame
}

fn assert_current_blend_source(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    presentation: &HostWindowPresentationData,
) -> Arc<UiSurfaceFrame> {
    let current = bridge.surface().surface_frame();
    let pane = node(presentation, "WorkbenchMainBandViewportPanel");
    let viewport = node(presentation, "UnrealSceneViewViewport");
    let root = node(presentation, BLEND_SPACE_ROOT);
    assert_eq!(root.parent_node_id, viewport.node_id);
    assert!(reaches(
        presentation,
        viewport.node_id.as_str(),
        pane.node_id.as_str()
    ));
    assert_eq!(root.source_node_id.as_str(), "blend_space_workspace");
    assert_eq!(
        root.source_path.as_str().replace('\\', "/"),
        "zircon_editor/assets/ui/editor/animation/extension_blend_space_workspace.zui"
    );
    assert_eq!(
        current
            .arranged_tree
            .get(root.surface_node_id.unwrap())
            .unwrap()
            .parent,
        viewport.surface_node_id,
    );
    for control in [
        "WorkbenchMainBandViewportPanel",
        "UnrealSceneViewViewport",
        BLEND_SPACE_ROOT,
        "WorkbenchExtensionBlendSpaceSearch",
        "WorkbenchExtensionBlendSpaceSampleCanvas",
        "WorkbenchExtensionBlendSpaceApplyButton",
    ] {
        let actual = node(presentation, control);
        assert!(Arc::ptr_eq(
            actual.source_surface_frame.as_ref().unwrap(),
            &current
        ));
        let arranged = current
            .arranged_tree
            .get(actual.surface_node_id.unwrap())
            .unwrap();
        assert_eq!(arranged.node_path.0, actual.node_id.as_str());
        assert!(arranged.enabled && arranged.is_render_visible());
    }
    current
}

fn reaches(presentation: &HostWindowPresentationData, descendant: &str, ancestor: &str) -> bool {
    let mut cursor = descendant;
    for _ in 0..=presentation.workbench_window_nodes.row_count() {
        if cursor == ancestor {
            return true;
        }
        let Some(current) = presentation
            .workbench_window_nodes
            .iter()
            .find(|node| node.node_id.as_str() == cursor)
        else {
            return false;
        };
        cursor = current.parent_node_id.as_str();
    }
    false
}

fn indexed_subtree_nodes(
    presentation: &HostWindowPresentationData,
    root_control: &str,
    clip: &FrameRect,
) -> Vec<String> {
    let root = node(presentation, root_control);
    let root_row = presentation
        .workbench_window_nodes
        .iter()
        .position(|node| node.node_id == root.node_id)
        .unwrap();
    let index = HostWorkbenchHitIndex::from_presentation(presentation);
    assert!(index.indexes_presentation(presentation));
    let mut rows = Vec::new();
    assert!(index.visit_paint_rows_for_subtree(root_row, clip, &mut |row| rows.push(row)));
    rows.into_iter()
        .map(|row| {
            let current = presentation.workbench_window_nodes.get(row).unwrap();
            assert!(reaches(
                presentation,
                current.node_id.as_str(),
                root.node_id.as_str()
            ));
            current.node_id.as_str().to_string()
        })
        .collect()
}

fn paint_panes(presentation: &HostWindowPresentationData) -> HostRgbaFrame {
    let mut frame = HostRgbaFrame::filled(WINDOW_WIDTH, WINDOW_HEIGHT, SENTINEL);
    super::panes::draw_authored_panes(&mut frame, presentation, &window_bounds());
    frame
}

fn assert_recorded_source_command(
    presentation: &HostWindowPresentationData,
    control: &str,
    source: &Arc<UiSurfaceFrame>,
) {
    let actual = node(presentation, control);
    let command_ref = actual
        .surface_render_command_ref
        .expect("a visible authored control must keep its actual render command reference");
    assert_eq!(Some(command_ref.node_id), actual.surface_node_id);
    assert!(source.render_extract.command_by_ref(command_ref).is_some());
    let mut recording = HostRgbaFrame::recording_only(WINDOW_WIDTH, WINDOW_HEIGHT);
    super::panes::draw_authored_panes(&mut recording, presentation, &window_bounds());
    let recorded = recording.into_recorded_frame();
    assert!(
        recorded.commands.iter().any(|command| {
            command.frame.width > 0.0
                && command.frame.height > 0.0
                && command.source.is_some_and(|published| {
                    published.command_ref == command_ref
                        && recorded
                            .render_sources
                            .resolve(published.surface_key)
                            .is_some_and(|frame| Arc::ptr_eq(frame, source))
                })
        }),
        "the shared pane painter must emit the actual source command for {control}"
    );
}

#[test]
fn command_entry_paints_authored_chrome_without_host_menu_projection() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![
            template_mount(
                "authored/top",
                "",
                "TestAuthoredTopChrome",
                FrameRect {
                    x: 0.0,
                    y: 0.0,
                    width: 160.0,
                    height: 24.0,
                },
                "accent",
            ),
            template_mount(
                "authored/pages",
                "",
                "TestAuthoredPageBand",
                FrameRect {
                    x: 0.0,
                    y: 24.0,
                    width: 160.0,
                    height: 33.0,
                },
                "panel",
            ),
        ]),
        ..HostWindowPresentationData::default()
    };
    let mut frame = HostRgbaFrame::filled(160, 120, SENTINEL);
    crate::ui::retained_host::host_contract::paint_workbench::draw_workbench_presentation_commands(
        &mut frame,
        &presentation,
    );
    assert_ne!(pixel(&frame, 4, 4), SENTINEL);
    assert_ne!(pixel(&frame, 4, 30), SENTINEL);
}

#[test]
fn live_viewport_fallback_filter_keeps_chrome_and_dynamic_overlays() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.document_dock.pane.kind = "Scene".into();
    presentation
        .viewport_images
        .replace_scene(HostViewportImageData {
            resource_key: "viewport:test".into(),
            resource_generation: 0,
            width: 1,
            height: 1,
            rgba: Some(vec![255; 4].into()),
            play_frame_identity: None,
            overlay: None,
        });
    let filter = ComponentizedChromeFallbackTransform::from_presentation(&presentation);
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };

    assert!(filter
        .transform_row(0, template_node("WorkbenchViewportBackdrop"), clip.clone())
        .is_none());
    for control_id in [
        "WorkbenchViewportToolbar",
        "WorkbenchViewportSelectionTop",
        "WorkbenchViewportAxisX",
        "WorkbenchViewportGizmoCenter",
        "WorkbenchUnrelatedControl",
    ] {
        assert!(filter
            .transform_row(0, template_node(control_id), clip.clone())
            .is_some());
    }
}

#[test]
fn missing_or_invalid_viewport_image_keeps_the_fallback_scene() {
    let mut missing = HostWindowPresentationData::default();
    missing.host_scene_data.document_dock.pane.kind = "Scene".into();
    let mut invalid = missing.clone();
    invalid
        .viewport_images
        .replace_scene(HostViewportImageData::default());
    for presentation in [missing, invalid] {
        let filter = ComponentizedChromeFallbackTransform::from_presentation(&presentation);
        assert!(filter
            .transform_row(
                0,
                template_node("WorkbenchViewportBackdrop"),
                FrameRect {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                },
            )
            .is_some());
    }
}

fn template_node(control_id: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn active_extension_workspace_paints_only_inside_its_adaptive_host_frame() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut bridge = window_bridge();
    let scene = current_presentation(&bridge);
    let scene_pixels = paint_panes(&scene);
    select_blend_space(&mut bridge);
    let presentation = current_presentation(&bridge);
    let source = assert_current_blend_source(&bridge, &presentation);
    let clip = module_clip(&presentation);
    let included = indexed_subtree_nodes(&presentation, "WorkbenchMainBandViewportPanel", &clip);
    for control in [
        "WorkbenchMainBandViewportPanel",
        "UnrealSceneViewViewport",
        BLEND_SPACE_ROOT,
        "WorkbenchExtensionBlendSpaceSampleCanvas",
    ] {
        assert!(included.contains(&node(&presentation, control).node_id.as_str().to_string()));
    }
    let canvas = super::intersect_rect(
        &node_frame(node(
            &presentation,
            "WorkbenchExtensionBlendSpaceSampleCanvas",
        )),
        &clip,
    )
    .unwrap();
    let x = (canvas.x + canvas.width * 0.5) as usize;
    let y = (canvas.y + canvas.height * 0.5) as usize;
    let frame = paint_panes(&presentation);
    assert_ne!(pixel(&frame, x, y), SENTINEL);
    // The production pane fills the horizontal viewport; use its two visible vertical edges.
    assert!(clip.y >= 4.0 && clip.y + clip.height + 4.0 < WINDOW_HEIGHT as f32);
    let above = (clip.y - 4.0) as usize;
    let below = (clip.y + clip.height + 4.0) as usize;
    assert_eq!(pixel(&frame, x, above), pixel(&scene_pixels, x, above));
    assert_eq!(pixel(&frame, x, below), pixel(&scene_pixels, x, below));
    assert_eq!(pixel(&frame, x, above), SENTINEL);
    assert_eq!(pixel(&frame, x, below), SENTINEL);
    assert_recorded_source_command(
        &presentation,
        "WorkbenchExtensionBlendSpaceSampleCanvas",
        &source,
    );

    let baseline = frame.into_bytes();
    let mut with_sibling = presentation.clone();
    let mut nodes = presentation
        .workbench_window_nodes
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let mut sibling = template_mount(
        "unowned/scene_workspace",
        "unowned",
        "UnownedSceneWorkspace",
        clip.clone(),
        "accent",
    );
    sibling.z_index = 99;
    nodes.push(sibling);
    with_sibling.workbench_window_nodes = model(nodes);
    assert!(
        !indexed_subtree_nodes(&with_sibling, "WorkbenchMainBandViewportPanel", &clip)
            .contains(&"unowned/scene_workspace".to_string())
    );
    assert_eq!(
        paint_panes(&with_sibling).into_bytes(),
        baseline,
        "a non-owned Scene sibling must not overpaint the selected source subtree"
    );
}

#[test]
fn scene_workspace_without_extension_host_keeps_legacy_dock_pixels_untouched() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut bridge = window_bridge();
    let scene = current_presentation(&bridge);
    assert!(!authored_panes(&scene)
        .iter()
        .any(|pane| pane.module_workspace.is_some()));
    let baseline = paint_panes(&scene).into_bytes();
    select_blend_space(&mut bridge);
    let selected = current_presentation(&bridge);
    let old_source = assert_current_blend_source(&bridge, &selected);
    let retired_root = node(&selected, BLEND_SPACE_ROOT).surface_node_id.unwrap();
    select_module(&mut bridge, "menu.item.scene");
    let restored = current_presentation(&bridge);
    assert!(!bridge.has_control(BLEND_SPACE_ROOT));
    assert!(bridge.surface().tree.node(retired_root).is_none());
    assert!(bridge
        .surface()
        .surface_frame()
        .arranged_tree
        .get(retired_root)
        .is_none());
    assert!(!Arc::ptr_eq(&old_source, &bridge.surface().surface_frame()));
    assert!(!authored_panes(&restored)
        .iter()
        .any(|pane| pane.module_workspace.is_some()));
    let frame = paint_panes(&restored);
    let viewport = authored_panes(&restored)
        .into_iter()
        .find(|pane| pane.root.control_id.as_str() == "WorkbenchMainBandViewportPanel")
        .unwrap();
    for y in viewport.frame.y.ceil() as usize
        ..(viewport.frame.y + viewport.frame.height).floor() as usize
    {
        for x in viewport.frame.x.ceil() as usize
            ..(viewport.frame.x + viewport.frame.width).floor() as usize
        {
            assert_eq!(
                pixel(&frame, x, y),
                SENTINEL,
                "the empty native Scene owner must not gain template paint from the retired module"
            );
        }
    }
    assert_eq!(
        frame.into_bytes(),
        baseline,
        "returning to the actual Scene owner must leave ordinary dock pixels unchanged"
    );
}

#[test]
fn inactive_extension_workspace_hosts_do_not_overpaint_the_visible_workspace() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut bridge = window_bridge();
    select_blend_space(&mut bridge);
    let presentation = current_presentation(&bridge);
    let source = assert_current_blend_source(&bridge, &presentation);
    let clip = module_clip(&presentation);
    assert_recorded_source_command(
        &presentation,
        "WorkbenchExtensionBlendSpaceApplyButton",
        &source,
    );
    let painted = paint_panes(&presentation);
    let apply = node(&presentation, "WorkbenchExtensionBlendSpaceApplyButton");
    let apply_frame = super::intersect_rect(&node_frame(apply), &clip).unwrap();
    assert!(apply_frame.width > 0.0 && apply_frame.height > 0.0);
    assert_ne!(
        pixel(
            &painted,
            (apply_frame.x + apply_frame.width * 0.5) as usize,
            (apply_frame.y + apply_frame.height * 0.5) as usize,
        ),
        SENTINEL,
        "the selected Apply control must actually paint before the inactive sibling is added"
    );
    let baseline = painted.into_bytes();
    let mut with_inactive = presentation.clone();
    let mut nodes = presentation
        .workbench_window_nodes
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let mut inactive = template_mount(
        "unowned/inactive_workspace",
        "unowned",
        "UnownedInactiveWorkspace",
        clip.clone(),
        "accent",
    );
    inactive.z_index = 99;
    nodes.push(inactive);
    with_inactive.workbench_window_nodes = model(nodes);
    assert!(
        !indexed_subtree_nodes(&with_inactive, "WorkbenchMainBandViewportPanel", &clip)
            .contains(&"unowned/inactive_workspace".to_string())
    );
    assert_eq!(
        paint_panes(&with_inactive).into_bytes(),
        baseline,
        "visible workspace selection must exclude a higher-order non-owned workspace"
    );
    select_module(&mut bridge, "menu.item.scene");
    assert!(!bridge.has_control(BLEND_SPACE_ROOT));
}

#[test]
fn extension_workspace_search_field_paints_surface_glyph_and_placeholder_text() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut bridge = window_bridge();
    select_blend_space(&mut bridge);
    let presentation = current_presentation(&bridge);
    let source = assert_current_blend_source(&bridge, &presentation);
    let search = node(&presentation, "WorkbenchExtensionBlendSpaceSearch");
    assert_eq!(search.component_role.as_str(), "search-field");
    assert!(search.search_query.is_empty());
    let search_frame =
        super::intersect_rect(&node_frame(search), &module_clip(&presentation)).unwrap();
    assert_recorded_source_command(&presentation, "WorkbenchExtensionBlendSpaceSearch", &source);
    let frame = paint_panes(&presentation);
    assert!(distinct_colors(&frame, &search_frame) >= 4,
        "SearchField should paint its inset surface, border, search glyph, and Runtime text placeholder");
}

fn template_mount(
    node_id: &str,
    parent_node_id: &str,
    control_id: &str,
    frame: FrameRect,
    surface_variant: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id.into(),
        parent_node_id: parent_node_id.into(),
        control_id: control_id.into(),
        role: "Mount".into(),
        surface_variant: surface_variant.into(),
        frame: TemplateNodeFrameData {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn pixel(frame: &HostRgbaFrame, x: usize, y: usize) -> [u8; 4] {
    let offset = (y * frame.width() as usize + x) * 4;
    frame.as_bytes()[offset..offset + 4]
        .try_into()
        .expect("pixel should expose four RGBA channels")
}

fn distinct_colors(frame: &HostRgbaFrame, rect: &FrameRect) -> usize {
    let left = rect.x.max(0.0).floor() as usize;
    let top = rect.y.max(0.0).floor() as usize;
    let right = (rect.x + rect.width).max(0.0).ceil() as usize;
    let bottom = (rect.y + rect.height).max(0.0).ceil() as usize;
    (top..bottom)
        .flat_map(|y| (left..right).map(move |x| pixel(frame, x, y)))
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

#[test]
fn extension_workspace_projection_borrows_committed_nodes_and_visits_only_its_subtree() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut bridge = window_bridge();
    select_blend_space(&mut bridge);
    let presentation = current_presentation(&bridge);
    assert_current_blend_source(&bridge, &presentation);
    let committed = presentation.workbench_window_nodes.clone();
    assert!(committed.shares_values_with(&presentation.workbench_window_nodes));
    let clip = module_clip(&presentation);
    let included = indexed_subtree_nodes(&presentation, BLEND_SPACE_ROOT, &clip);
    assert!(included.contains(
        &node(&presentation, BLEND_SPACE_ROOT)
            .node_id
            .as_str()
            .to_string()
    ));
    assert!(included.contains(
        &node(&presentation, "WorkbenchExtensionBlendSpaceSampleCanvas")
            .node_id
            .as_str()
            .to_string()
    ));
    assert!(!included.contains(
        &node(&presentation, "UnrealOutlinerFilter")
            .node_id
            .as_str()
            .to_string()
    ));
    assert!(committed.shares_values_with(&presentation.workbench_window_nodes));
    let production = include_str!("../../workbench/overlay/componentized.rs");
    let subtree = production
        .split_once("fn stream_row_visit_indices")
        .map(|(_, subtree)| subtree)
        .expect("the existing subtree must keep its streaming row visitor");
    assert!(subtree.contains("included_rows"));
    assert!(
        !production.contains("filter_map(|row| presentation.workbench_window_nodes.row_data(row))")
    );
    assert!(!production.contains("parents_by_node_id"));
}

fn model<T: Clone>(values: Vec<T>) -> ModelRc<T> {
    Rc::new(VecModel::from(values)).into()
}
