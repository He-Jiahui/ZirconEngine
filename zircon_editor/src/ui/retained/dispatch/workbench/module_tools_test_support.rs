//! Real public child-admission failures preserve the last published workspace.
//! This fixture uses compiled production documents and a legitimate live focus change.
use super::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use crate::ui::retained_host::host_contract::componentized_workbench_regions::{
    assert_control_source_paint_and_hit, authored_panes,
};
use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use std::sync::Arc;
use zircon_runtime::ui::v2::UiV2SurfaceBuilder;
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize};

fn presentation(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
) -> HostWindowPresentationData {
    let mut data = HostWindowPresentationData {
        workbench_window_nodes:
            crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes(Some(
                bridge.host_projection(),
            )),
        ..Default::default()
    };
    let frame = |control_id: &str| {
        let node = data
            .workbench_window_nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap();
        FrameRect {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        }
    };
    data.host_layout.center_band_frame = frame("UnrealLevelEditorContent");
    data.host_layout.status_bar_frame = frame("WorkbenchWindowStatusBarRegion");
    data
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    /// Stage the real Shader source, then change the current live input owner through its public API.
    /// The returned closure commits that exact proof against this bridge's actual private surface.
    pub(crate) fn stage_shader_source_then_change_live_focus_for_test(
        &mut self,
    ) -> impl FnOnce(&mut Self) -> Result<(), zircon_runtime_interface::ui::v2::UiV2AssetError>
    {
        let (document, component) =
            super::extension_module_navigation::workbench_extension_workspace_document(
                "workbench.extension.shader_editor.open",
            )
            .expect("the canonical Shader descriptor is implemented");
        let (_, incoming, _) = self
            .runtime
            .project_workbench_module_workspace(document, component)
            .unwrap();
        let parent = self.control_node_id("UnrealSceneViewViewport").unwrap();
        let previous_root = self
            .control_node_id("WorkbenchModuleRenderWorkspace")
            .unwrap();
        let (mut staged, _, proof) = UiV2SurfaceBuilder::stage_child_surface(
            self.surface(),
            parent,
            Some(previous_root),
            Some(incoming),
        )
        .unwrap();
        let candidates = self
            .surface()
            .tree
            .nodes
            .iter()
            .filter_map(|(id, node)| {
                (node.is_focus_candidate() && self.surface().focus.focused != Some(*id))
                    .then_some(*id)
            })
            .collect::<Vec<_>>();
        let focus = candidates
            .into_iter()
            .find(|id| self.template_surface.surface.focus_node(*id).is_ok())
            .expect("the actual current surface has a different available focus owner");
        assert_eq!(self.surface().focus.focused, Some(focus));
        self.refresh_prepared_state_change().unwrap();
        move |bridge: &mut Self| {
            UiV2SurfaceBuilder::commit_child_surface(
                &mut staged,
                &mut bridge.template_surface.surface,
                &proof,
            )
        }
    }
    pub(crate) fn assert_actual_stale_child_rejection_preserves_module_state() {
        for (item, root_control, gateway, gateway_binding, live_command, live_binding, extension) in [
            (
                "menu.item.v_f_x",
                "WorkbenchModuleVfxWorkspace",
                "WorkbenchVfxParticleLibraryButton",
                "WorkbenchExtension/ParticleLibraryOpen",
                "WorkbenchVfxSimulateButton",
                "WorkbenchModule/VfxSimulate",
                "WorkbenchExtensionParticleLibraryWorkspace",
            ),
            (
                "menu.item.render",
                "WorkbenchModuleRenderWorkspace",
                "WorkbenchRenderCompileButton",
                "WorkbenchModule/RenderCompile",
                "WorkbenchRenderCompileButton",
                "WorkbenchModule/RenderCompile",
                "WorkbenchExtensionShaderEditorWorkspace",
            ),
        ] {
            let mut bridge = Self::new(UiSize::new(1672.0, 941.0)).unwrap();
            assert!(bridge
                .dispatch_control_state("MenuSlot2", UiEventKind::Click)
                .unwrap()
                .is_some());
            assert!(bridge
                .dispatch_workbench_module_overflow_menu_item_state("UnrealWindowMenuPopup", item)
                .unwrap()
                .is_some());
            assert!(bridge.has_control(gateway));
            assert!(!bridge.has_control("WorkbenchExtensionModuleWorkspacesHost"));
            assert!(!bridge.has_control(extension));
            let gateway_route = bridge
                .binding_for_control(gateway, UiEventKind::Click)
                .unwrap()
                .clone();
            assert!(bridge.binding_by_id(gateway_binding).is_some());
            for _ in 0..2 {
                let parent = bridge.control_node_id("UnrealSceneViewViewport").unwrap();
                let (_, incoming, _) = bridge
                    .runtime
                    .project_workbench_module_workspace(
                        "res://ui/editor/rendering/extension_shader_editor_workspace.zui",
                        "WorkbenchExtensionShaderEditorWorkspace",
                    )
                    .unwrap();
                let previous_root = bridge.control_node_id(root_control).unwrap();
                let (mut staged, _, proof) = UiV2SurfaceBuilder::stage_child_surface(
                    bridge.surface(),
                    parent,
                    Some(previous_root),
                    Some(incoming),
                )
                .unwrap();
                let candidates = bridge
                    .surface()
                    .tree
                    .nodes
                    .iter()
                    .filter_map(|(id, node)| {
                        (node.is_focus_candidate() && bridge.surface().focus.focused != Some(*id))
                            .then_some(*id)
                    })
                    .collect::<Vec<_>>();
                let focused = candidates
                    .into_iter()
                    .find(|id| bridge.template_surface.surface.focus_node(*id).is_ok())
                    .expect("actual live controls supply a new focus owner");
                assert_eq!(bridge.surface().focus.focused, Some(focused));
                bridge.refresh_prepared_state_change().unwrap();
                let before_surface = bridge.surface().clone();
                let before_projection = bridge.host_projection().clone();
                let before_presentation = presentation(&bridge);
                let before_source = before_presentation
                    .workbench_window_nodes
                    .iter()
                    .find(|node| node.control_id.as_str() == root_control)
                    .unwrap()
                    .source_surface_frame
                    .as_ref()
                    .unwrap()
                    .clone();
                let before_frame = bridge.control_frame(root_control).unwrap();
                assert_control_source_paint_and_hit(&before_presentation, gateway);
                assert!(
                    UiV2SurfaceBuilder::commit_child_surface(
                        &mut staged,
                        &mut bridge.template_surface.surface,
                        &proof
                    )
                    .is_err(),
                    "a public proof cannot overwrite a legitimate live focus update"
                );
                assert_eq!(bridge.surface(), &before_surface);
                assert_eq!(bridge.surface().tree, before_surface.tree);
                assert_eq!(bridge.surface().navigation, before_surface.navigation);
                assert_eq!(bridge.surface().focus, before_surface.focus);
                assert_eq!(bridge.surface().input, before_surface.input);
                assert_eq!(bridge.host_projection(), &before_projection);
                assert_eq!(
                    bridge.binding_for_control(gateway, UiEventKind::Click),
                    Some(&gateway_route)
                );
                assert!(bridge.binding_by_id(gateway_binding).is_some());
                let current = presentation(&bridge);
                assert!(authored_panes(&current).into_iter().any(|pane| pane
                    .module_workspace
                    .is_some_and(|root| root.control_id.as_str() == root_control)));
                let current_source = current
                    .workbench_window_nodes
                    .iter()
                    .find(|node| node.control_id.as_str() == root_control)
                    .unwrap()
                    .source_surface_frame
                    .as_ref()
                    .unwrap();
                assert!(Arc::ptr_eq(current_source, &before_source));
                assert_eq!(bridge.control_frame(root_control), Some(before_frame));
                assert!(!bridge.has_control(extension));
                assert!(!bridge.has_control("WorkbenchExtensionModuleWorkspacesHost"));
                assert_control_source_paint_and_hit(&current, gateway);
            }
            assert!(
                bridge
                    .dispatch_binding_state_for_control(live_command, live_binding)
                    .unwrap()
                    .is_some(),
                "the retained workspace's original command remains live"
            );
        }
    }
}
