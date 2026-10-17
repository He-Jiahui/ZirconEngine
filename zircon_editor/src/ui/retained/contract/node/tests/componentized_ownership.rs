use std::sync::Arc;

use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize};

use crate::tests::support::env_lock;
use crate::ui::binding::EditorUiBindingPayload;
use crate::ui::retained_host::callback_dispatch::template_bridge::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use crate::ui::retained_host::host_contract::componentized_workbench_regions::authored_panes;
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostWindowPresentationData, TemplateNodeFrameData, TemplatePaneNodeData,
};

use super::support::{hit_test_workbench_window_template_node, model};

#[test]
fn componentized_workbench_hit_test_skips_template_controls_outside_painted_regions() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("the current Unreal window must load");
    let mut scene = live_presentation(&bridge);
    let viewport = authored_panes(&scene)
        .into_iter()
        .find(|pane| pane.root.control_id.as_str() == "WorkbenchMainBandViewportPanel")
        .unwrap();
    assert!(viewport.module_workspace.is_none());
    let scene_probe = TemplateNodeFrameData {
        x: viewport.frame.x + viewport.frame.width * 0.4,
        y: viewport.frame.y + viewport.frame.height * 0.5,
        width: 80.0,
        height: 28.0,
    };
    let mut nodes = scene
        .workbench_window_nodes
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    nodes.push(TemplatePaneNodeData {
        node_id: "unowned/scene-button".into(),
        parent_node_id: "unowned".into(),
        control_id: "UnownedSceneButton".into(),
        action_id: "unowned.scene.click".into(),
        frame: scene_probe.clone(),
        z_index: 99,
        ..Default::default()
    });
    scene.workbench_window_nodes = model(nodes);
    assert!(
        hit_test_workbench_window_template_node(&scene, scene_probe.x + 4.0, scene_probe.y + 4.0)
            .is_none(),
        "ordinary template controls cannot claim the native Scene region without its painted owner"
    );

    bridge
        .dispatch_control_state("MenuSlot2", UiEventKind::Click)
        .unwrap()
        .unwrap();
    bridge
        .dispatch_workbench_module_overflow_menu_item_state(
            "UnrealWindowMenuPopup",
            "menu.item.ability",
        )
        .unwrap()
        .unwrap();
    assert!(bridge.has_control("WorkbenchAbilityBlendSpaceButton"));
    bridge
        .dispatch_control_state("WorkbenchAbilityBlendSpaceButton", UiEventKind::Click)
        .unwrap()
        .unwrap();
    let mut presentation = live_presentation(&bridge);
    let current = bridge.surface().surface_frame();
    let root = live_node(&presentation, "WorkbenchExtensionBlendSpaceWorkspace");
    let viewport = live_node(&presentation, "UnrealSceneViewViewport");
    let pane = authored_panes(&presentation)
        .into_iter()
        .find(|pane| pane.root.control_id.as_str() == "WorkbenchMainBandViewportPanel")
        .unwrap();
    assert_eq!(pane.module_workspace.unwrap().node_id, root.node_id);
    assert_eq!(root.parent_node_id, viewport.node_id);
    assert_eq!(root.source_node_id.as_str(), "blend_space_workspace");
    assert_eq!(
        root.source_path.as_str().replace('\\', "/"),
        "zircon_editor/assets/ui/editor/animation/extension_blend_space_workspace.zui"
    );
    assert!(Arc::ptr_eq(
        root.source_surface_frame.as_ref().unwrap(),
        &current
    ));
    assert_eq!(
        current
            .arranged_tree
            .get(root.surface_node_id.unwrap())
            .unwrap()
            .parent,
        viewport.surface_node_id
    );
    let apply = live_node(&presentation, "WorkbenchExtensionBlendSpaceApplyButton").clone();
    assert!(Arc::ptr_eq(
        apply.source_surface_frame.as_ref().unwrap(),
        &current
    ));
    let command_ref = apply
        .surface_render_command_ref
        .expect("the actual Apply control must retain its current paint command");
    assert_eq!(Some(command_ref.node_id), apply.surface_node_id);
    assert!(current.render_extract.command_by_ref(command_ref).is_some());
    let mut nodes = presentation
        .workbench_window_nodes
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    nodes.push(TemplatePaneNodeData {
        node_id: "unowned/inactive-button".into(),
        parent_node_id: "unowned/inactive-workspace".into(),
        control_id: "UnownedInactiveApply".into(),
        action_id: "unowned.apply".into(),
        frame: apply.frame.clone(),
        z_index: 99,
        ..Default::default()
    });
    presentation.workbench_window_nodes = model(nodes);
    for control in [
        "MenuSlot2",
        "WorkbenchStatusSnapToggle",
        "WorkbenchExtensionBlendSpaceApplyButton",
    ] {
        let actual = live_node(&presentation, control);
        let hit = hit_test_workbench_window_template_node(
            &presentation,
            actual.frame.x + actual.frame.width * 0.5,
            actual.frame.y + actual.frame.height * 0.5,
        )
        .unwrap_or_else(|| panic!("painted control {control} must remain hit-testable"));
        assert_eq!(hit.control_id.as_str(), control);
    }
    let expected = bridge
        .binding_for_control(
            "WorkbenchExtensionBlendSpaceApplyButton",
            UiEventKind::Click,
        )
        .unwrap()
        .clone();
    let dispatched = bridge
        .dispatch_control_state(
            "WorkbenchExtensionBlendSpaceApplyButton",
            UiEventKind::Click,
        )
        .unwrap()
        .unwrap();
    assert_eq!(dispatched, expected);
    assert_eq!(
        dispatched.payload,
        EditorUiBindingPayload::MenuAction {
            action_id: "workbench.extension.blend_space.apply.invoke".to_string(),
        }
    );
    bridge
        .dispatch_control_state("MenuSlot2", UiEventKind::Click)
        .unwrap()
        .unwrap();
    bridge
        .dispatch_workbench_module_overflow_menu_item_state(
            "UnrealWindowMenuPopup",
            "menu.item.scene",
        )
        .unwrap()
        .unwrap();
    let restored = live_presentation(&bridge);
    assert!(!bridge.has_control("WorkbenchExtensionBlendSpaceApplyButton"));
    assert!(!Arc::ptr_eq(&current, &bridge.surface().surface_frame()));
    assert!(
        hit_test_workbench_window_template_node(
            &restored,
            apply.frame.x + apply.frame.width * 0.5,
            apply.frame.y + apply.frame.height * 0.5
        )
        .is_none(),
        "the retired module cannot leave an interactive control in the native Scene region"
    );
}

fn live_presentation(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
) -> HostWindowPresentationData {
    let mut presentation = HostWindowPresentationData {
        workbench_window_nodes:
            crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes(Some(
                bridge.host_projection(),
            )),
        ..Default::default()
    };
    let content = live_node(&presentation, "UnrealLevelEditorContent");
    presentation.host_layout.center_band_frame = FrameRect {
        x: content.frame.x,
        y: content.frame.y,
        width: content.frame.width,
        height: content.frame.height,
    };
    let status = live_node(&presentation, "WorkbenchWindowStatusBarRegion");
    presentation.host_layout.status_bar_frame = FrameRect {
        x: status.frame.x,
        y: status.frame.y,
        width: status.frame.width,
        height: status.frame.height,
    };
    presentation
}

fn live_node<'a>(
    presentation: &'a HostWindowPresentationData,
    control: &str,
) -> &'a TemplatePaneNodeData {
    presentation
        .workbench_window_nodes
        .iter()
        .find(|node| node.control_id.as_str() == control)
        .unwrap_or_else(|| panic!("{control} must be in the current production projection"))
}

fn authored_viewport_hit_fixture() -> HostWindowPresentationData {
    let node = |id: &str, parent: &str, control: &str, action: &str, x, y, width, height| {
        TemplatePaneNodeData {
            node_id: id.into(),
            parent_node_id: parent.into(),
            control_id: control.into(),
            action_id: action.into(),
            frame: TemplateNodeFrameData {
                x,
                y,
                width,
                height,
            },
            ..Default::default()
        }
    };
    let mut presentation = HostWindowPresentationData {
        host_layout: crate::ui::retained_host::host_contract::data::HostWindowLayoutData {
            authoritative: true,
            center_band_frame: FrameRect {
                x: 0.0,
                y: 74.0,
                width: 900.0,
                height: 520.0,
            },
            status_bar_frame: FrameRect {
                x: 0.0,
                y: 594.0,
                width: 900.0,
                height: 26.0,
            },
            ..Default::default()
        },
        workbench_window_nodes: model(vec![
            node(
                "root",
                "",
                "WorkbenchWindowRoot",
                "",
                0.0,
                0.0,
                900.0,
                620.0,
            ),
            node(
                "content",
                "root",
                "UnrealLevelEditorContent",
                "",
                0.0,
                74.0,
                900.0,
                520.0,
            ),
            node(
                "viewport",
                "content",
                "WorkbenchMainBandViewportPanel",
                "",
                40.0,
                96.0,
                600.0,
                420.0,
            ),
            node(
                "media",
                "viewport",
                "WorkbenchViewportSurface",
                "",
                50.0,
                130.0,
                580.0,
                380.0,
            ),
            node(
                "toolbar",
                "viewport",
                "UnrealViewportToolbar",
                "",
                40.0,
                96.0,
                600.0,
                34.0,
            ),
            node(
                "toolbar-button",
                "toolbar",
                "UnrealViewportCamera",
                "viewport.camera",
                52.0,
                101.0,
                120.0,
                24.0,
            ),
            node(
                "outliner",
                "content",
                "WorkbenchMainBandSceneTreePanel",
                "",
                660.0,
                96.0,
                220.0,
                480.0,
            ),
            node(
                "outliner-button",
                "outliner",
                "UnrealOutlinerFilter",
                "outliner.filter",
                680.0,
                120.0,
                160.0,
                24.0,
            ),
            node(
                "top-button",
                "root",
                "UnrealFileMenu",
                "menu.file",
                16.0,
                16.0,
                100.0,
                24.0,
            ),
            node(
                "status-button",
                "root",
                "UnrealContentBrowser",
                "content.toggle",
                16.0,
                598.0,
                100.0,
                18.0,
            ),
        ]),
        ..Default::default()
    };
    let document = &mut presentation.host_scene_data.document_dock;
    document.surface_key = "document:scene:authored".into();
    document.pane.kind = "Scene".into();
    document.region_frame = FrameRect {
        x: 40.0,
        y: 96.0,
        width: 600.0,
        height: 420.0,
    };
    presentation
}

fn assert_authored_non_viewport_controls_remain_hit_testable(
    presentation: &HostWindowPresentationData,
) {
    for (x, y, control) in [
        (690.0, 130.0, "UnrealOutlinerFilter"),
        (24.0, 24.0, "UnrealFileMenu"),
        (24.0, 606.0, "UnrealContentBrowser"),
    ] {
        let hit = hit_test_workbench_window_template_node(presentation, x, y)
            .expect("other authored panes and chrome retain their painted hit regions");
        assert_eq!(hit.control_id.as_str(), control);
    }
}

#[test]
fn componentized_workbench_hit_test_requires_visible_authored_viewport_owner() {
    use crate::ui::retained_host::host_contract::surface_hit_test::HostWorkbenchHitIndex;

    let mut presentation = authored_viewport_hit_fixture();
    for kind in ["Scene", "Game"] {
        presentation.host_scene_data.document_dock.pane.kind = kind.into();
        let hit = hit_test_workbench_window_template_node(&presentation, 60.0, 110.0)
            .expect("painted authored Scene/Game toolbar remains hit-testable");
        assert_eq!(hit.control_id.as_str(), "UnrealViewportCamera");
    }
    let painted_index = HostWorkbenchHitIndex::from_presentation(&presentation);
    assert!(painted_index.indexes_presentation(&presentation));
    for kind in ["Inspector", "ScriptEditor", "Welcome"] {
        presentation.host_scene_data.document_dock.pane.kind = kind.into();
        assert!(hit_test_workbench_window_template_node(&presentation, 60.0, 110.0).is_none());
        assert!(
            !painted_index.indexes_presentation(&presentation),
            "document-only owner changes must invalidate the cached painted hit ownership"
        );
        assert_authored_non_viewport_controls_remain_hit_testable(&presentation);
    }
    presentation.host_scene_data.document_dock.pane.kind = "Scene".into();
    presentation
        .host_scene_data
        .document_dock
        .region_frame
        .width = 100.0;
    assert!(
        hit_test_workbench_window_template_node(&presentation, 60.0, 110.0).is_none(),
        "a media frame outside its document region leaves the complete authored viewport unpainted"
    );
    assert!(!painted_index.indexes_presentation(&presentation));
    assert_authored_non_viewport_controls_remain_hit_testable(&presentation);
}

#[test]
fn componentized_workbench_hit_test_resolves_visible_viewport_from_document_leaves() {
    let mut presentation = authored_viewport_hit_fixture();
    let viewport_leaf = presentation.host_scene_data.document_dock.clone();
    let mut neighbor = viewport_leaf.clone();
    neighbor.surface_key = "document:inspector:neighbor".into();
    neighbor.pane.kind = "Inspector".into();
    neighbor.region_frame = FrameRect {
        x: 660.0,
        y: 96.0,
        width: 220.0,
        height: 480.0,
    };
    presentation.host_scene_data.document_dock.pane.kind = "Welcome".into();
    presentation.host_scene_data.document_leaves = vec![neighbor, viewport_leaf];
    for kind in ["Scene", "Game"] {
        presentation.host_scene_data.document_leaves[1].pane.kind = kind.into();
        let hit = hit_test_workbench_window_template_node(&presentation, 60.0, 110.0)
            .expect("the painted viewport belongs to its Scene/Game leaf, independent of the primary document");
        assert_eq!(hit.control_id.as_str(), "UnrealViewportCamera");
    }
    presentation.host_scene_data.document_dock.pane.kind = "Scene".into();
    presentation.host_scene_data.document_leaves.pop();
    assert!(hit_test_workbench_window_template_node(&presentation, 60.0, 110.0).is_none(),
        "the unused primary document cannot claim a viewport when active document leaves have no visible owner");
    assert_authored_non_viewport_controls_remain_hit_testable(&presentation);
}

#[test]
fn componentized_workbench_hit_test_rejects_toolbar_when_media_clip_is_disjoint() {
    let mut presentation = authored_viewport_hit_fixture();
    let mut nodes = presentation
        .workbench_window_nodes
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let media = nodes
        .iter_mut()
        .find(|node| node.control_id.as_str() == "WorkbenchViewportSurface")
        .unwrap();
    // Projected clips already contain the inherited arrangement clip. Only the media is clipped:
    // the toolbar's own frame remains nonempty, but the painter skips its invisible viewport owner.
    media.has_clip_frame = true;
    media.clip_frame = TemplateNodeFrameData {
        x: 700.0,
        y: 130.0,
        width: 100.0,
        height: 30.0,
    };
    presentation.workbench_window_nodes = model(nodes);
    assert!(hit_test_workbench_window_template_node(&presentation, 60.0, 110.0).is_none());
    assert_authored_non_viewport_controls_remain_hit_testable(&presentation);
}
