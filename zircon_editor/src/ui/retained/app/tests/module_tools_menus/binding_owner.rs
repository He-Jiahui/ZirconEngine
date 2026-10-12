use super::super::support::{lock_env, pane_surface_host};
use super::cases::{MENUS, WORKSPACES};
use super::support::{assert_preview, harness, node, open, select_module, sequence};
use crate::ui::retained_host::host_contract::componentized_workbench_regions::assert_control_source_paint_and_hit;
use std::sync::Arc;

struct PreviewRoute {
    module_item: &'static str,
    tools_row: Option<&'static str>,
    gateway: Option<(&'static str, &'static str)>,
    control: &'static str,
    other: &'static str,
    action: &'static str,
    binding: &'static str,
    source_suffix: &'static str,
    observed: &'static [&'static str],
}

#[test]
fn workbench_public_preview_binding_requires_current_control_owner() {
    let _guard = lock_env();
    let harness = harness();
    for case in [
        PreviewRoute {
            module_item: "menu.item.material",
            tools_row: None,
            gateway: None,
            control: "WorkbenchMaterialBaseColorRow",
            other: "WorkbenchMaterialNormalRow",
            action: "workbench.module.material.normal_row.select",
            binding: "WorkbenchModule/MaterialNormalRow",
            source_suffix: "modules/core/rendering/workbench_material_workspace.zui",
            observed: &[
                "WorkbenchMaterialBaseColorRow",
                "WorkbenchMaterialNormalRow",
                "WorkbenchMaterialNodeRow01",
                "WorkbenchMaterialNodeRow03",
                "WorkbenchMaterialCenterTitle",
                "WorkbenchMaterialOutputRow",
            ],
        },
        PreviewRoute {
            module_item: "menu.item.render",
            tools_row: None,
            gateway: None,
            control: "WorkbenchRenderFrameStartRow",
            other: "WorkbenchRenderSceneColorRow",
            action: "workbench.module.render.scene_color.select",
            binding: "WorkbenchModule/RenderSceneColor",
            source_suffix: "modules/core/rendering/workbench_render_workspace.zui",
            observed: &[
                "WorkbenchRenderFrameStartRow",
                "WorkbenchRenderSceneColorRow",
                "WorkbenchRenderCaptureRow",
            ],
        },
        PreviewRoute {
            module_item: "menu.item.render",
            tools_row: Some("menu.item.render.shader_editor"),
            gateway: None,
            control: "WorkbenchExtensionShaderSourceRow",
            other: "WorkbenchExtensionShaderIncludeRow",
            action: "workbench.extension.shader_editor.include_row.select",
            binding: "WorkbenchExtension/ShaderEditorIncludeRow",
            source_suffix:
                "modules/extensions/rendering/workbench_extension_shader_editor_workspace.zui",
            observed: &[
                "WorkbenchExtensionShaderSourceRow",
                "WorkbenchExtensionShaderIncludeRow",
                "WorkbenchExtensionShaderOutputRow",
            ],
        },
        PreviewRoute {
            module_item: "menu.item.v_f_x",
            tools_row: None,
            gateway: Some((
                "WorkbenchVfxParticleLibraryButton",
                "workbench.extension.particle_library.open",
            )),
            control: "WorkbenchExtensionParticleLibrarySparksRow",
            other: "WorkbenchExtensionParticleLibraryDustRow",
            action: "workbench.extension.particle_library.dust_row.select",
            binding: "WorkbenchExtension/ParticleLibraryDustRow",
            source_suffix:
                "modules/extensions/rendering/workbench_extension_particle_library_workspace.zui",
            observed: &[
                "WorkbenchExtensionParticleLibrarySparksRow",
                "WorkbenchExtensionParticleLibraryDustRow",
                "WorkbenchExtensionParticleLibraryOutputRow",
            ],
        },
    ] {
        if let Some(row) = case.tools_row {
            let workspace = WORKSPACES
                .iter()
                .find(|workspace| workspace.row_action == row)
                .unwrap();
            let menu = &MENUS[workspace.menu_index];
            open(&harness, menu);
            pane_surface_host(&harness.root_ui)
                .invoke_surface_control_clicked(menu.menu.into(), row.into());
            harness.host.borrow_mut().recompute_if_dirty();
        } else {
            select_module(&harness, case.module_item);
        }
        if let Some((control, action)) = case.gateway {
            let presentation = harness.root_ui.get_host_presentation();
            assert_control_source_paint_and_hit(&presentation, control);
            pane_surface_host(&harness.root_ui)
                .invoke_surface_control_clicked(control.into(), action.into());
            harness.host.borrow_mut().recompute_if_dirty();
        }

        let before = harness.root_ui.get_host_presentation();
        assert!(node(&before, case.control).selected);
        assert!(!node(&before, case.other).selected);
        for control in [case.control, case.other] {
            assert_control_source_paint_and_hit(&before, control);
        }
        let before_values: Vec<_> = case
            .observed
            .iter()
            .map(|control| {
                let actual = node(&before, control);
                (
                    actual.selected,
                    actual.text.clone(),
                    actual.value_text.clone(),
                    (
                        actual.frame.x,
                        actual.frame.y,
                        actual.frame.width,
                        actual.frame.height,
                    ),
                )
            })
            .collect();
        let before_sequence = sequence(&harness);
        let before_address = harness.root_ui.has_retained_pane_input_address();
        let (binding, focus, input, navigation, window_state) = {
            let host = harness.host.borrow();
            let bridge = &host.workbench_window_bridge;
            let projection = bridge.host_projection();
            let owner = projection.node_by_control_id(case.control).unwrap();
            let other = projection.node_by_control_id(case.other).unwrap();
            assert!(!owner
                .routes
                .iter()
                .any(|route| route.binding_id == case.binding));
            assert!(other
                .routes
                .iter()
                .any(|route| route.binding_id == case.binding));
            assert!(host
                .workbench_window_bridge
                .mounted_module_source_receipts_for_test()
                .iter()
                .any(|receipt| receipt
                    .physical_path
                    .to_string_lossy()
                    .replace('\\', "/")
                    .ends_with(case.source_suffix)));
            let current = bridge.surface().surface_frame();
            for control in [case.control, case.other] {
                let actual = node(&before, control);
                assert!(Arc::ptr_eq(
                    actual.source_surface_frame.as_ref().unwrap(),
                    &current
                ));
            }
            (
                bridge.binding_by_id(case.binding).unwrap().clone(),
                bridge.surface().focus.clone(),
                bridge.surface().input.clone(),
                bridge.surface().navigation.clone(),
                bridge.surface().window_state.clone(),
            )
        };

        // The real public callback cannot borrow another current row's action.
        pane_surface_host(&harness.root_ui)
            .invoke_surface_control_clicked(case.control.into(), case.action.into());
        harness.host.borrow_mut().recompute_if_dirty();
        assert_eq!(sequence(&harness), before_sequence);
        assert_eq!(
            harness.root_ui.has_retained_pane_input_address(),
            before_address
        );
        {
            let host = harness.host.borrow();
            let surface = host.workbench_window_bridge.surface();
            assert_eq!(surface.focus, focus);
            assert_eq!(surface.input, input);
            assert_eq!(surface.navigation, navigation);
            assert_eq!(surface.window_state, window_state);
        }
        let rejected = harness.root_ui.get_host_presentation();
        for (control, expected) in case.observed.iter().zip(&before_values) {
            let actual = node(&rejected, control);
            assert_eq!(
                (
                    actual.selected,
                    actual.text.clone(),
                    actual.value_text.clone(),
                    (
                        actual.frame.x,
                        actual.frame.y,
                        actual.frame.width,
                        actual.frame.height
                    ),
                ),
                *expected,
                "wrong action owner changed {control}",
            );
        }
        // Preserve the existing public rejection feedback rather than journaling a success.
        assert_eq!(
            node(&rejected, "WorkbenchStatusReady").text.as_str(),
            format!("Unknown componentized workbench control {}", case.control),
        );
        for control in [case.control, case.other] {
            assert_control_source_paint_and_hit(&rejected, control);
        }

        // The same action still reaches its authored owner and the ordinary journal.
        pane_surface_host(&harness.root_ui)
            .invoke_surface_control_clicked(case.other.into(), case.action.into());
        harness.host.borrow_mut().recompute_if_dirty();
        assert_preview(&harness, before_sequence, &binding, case.action);
        let accepted = harness.root_ui.get_host_presentation();
        assert!(!node(&accepted, case.control).selected);
        assert!(node(&accepted, case.other).selected);
        for control in [case.control, case.other] {
            assert_control_source_paint_and_hit(&accepted, control);
        }
    }
}
