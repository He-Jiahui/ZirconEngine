use super::*;
use crate::scene::viewport::{CapturedFrame, RenderViewportHandle, RenderViewportProduct};
use crate::ui::retained_host::host_contract::{
    FrameRect, HostPresentationGeneration, HostWindowPresentationData,
};
use crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale;
use crate::ui::retained_host::PaneSurfaceHostContext;
use std::sync::Arc;
use zircon_runtime_interface::ui::component::UiValue;

#[test]
fn native_viewport_products_publish_to_each_child_without_changing_input_identity() {
    let first = native_child("native:first");
    let second = native_child("native:second");
    first
        .global::<PaneSurfaceHostContext>()
        .set_scene_viewport_surface_key("native:first");
    second
        .global::<PaneSurfaceHostContext>()
        .set_scene_viewport_surface_key("native:second");

    assert!(publish_viewport_product(
        &second,
        "native:second",
        RenderViewportProduct::new(RenderViewportHandle::new(22), 800, 600, 7),
    ));
    assert!(publish_viewport_product(
        &first,
        "native:first",
        RenderViewportProduct::new(RenderViewportHandle::new(11), 640, 360, 7),
    ));

    for (ui, surface_key, resource_key) in [
        (&first, "native:first", "viewport:11:7"),
        (&second, "native:second", "viewport:22:7"),
    ] {
        let context = ui.global::<PaneSurfaceHostContext>();
        assert_eq!(
            context.scene_viewport_surface_key().as_deref(),
            Some(surface_key)
        );
        let generation = ui.get_host_presentation_generation();
        assert_eq!(
            generation
                .viewport_images()
                .for_surface(surface_key, "Scene")
                .expect("native child viewport product")
                .resource_key,
            resource_key
        );
        assert!(ui.take_external_redraw_for_test().request_redraw());
    }
}

#[test]
fn secondary_dock_product_damages_its_own_region() {
    let ui = UiHostWindow::new().expect("host window");
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.right_dock.surface_key = "dock:right".into();
    presentation.host_scene_data.right_dock.region_frame = FrameRect {
        x: 900.0,
        y: 80.0,
        width: 300.0,
        height: 640.0,
    };
    ui.set_host_presentation(presentation);
    let _ = ui.take_external_redraw_for_test();

    assert!(publish_viewport_product(
        &ui,
        "dock:right",
        RenderViewportProduct::new(RenderViewportHandle::new(31), 300, 640, 9),
    ));

    assert_eq!(
        ui.take_external_redraw_for_test().damage_region(),
        Some(&FrameRect {
            x: 900.0,
            y: 80.0,
            width: 300.0,
            height: 640.0,
        })
    );
}

#[test]
fn viewport_image_capture_metadata_refreshes_main_product_without_visual_mutation() {
    with_capture_metadata_host(|host| {
        let previous = host.ui.get_host_presentation_generation();
        assert!(publish_viewport_product(
            &host.ui,
            "editor.viewport",
            RenderViewportProduct::new(RenderViewportHandle::new(41), 8, 8, 11),
        ));
        complete_main_image_and_assert_metadata(host, &previous);
    });
}

#[test]
fn viewport_image_capture_metadata_refreshes_main_captured_fallback() {
    with_capture_metadata_host(|host| {
        let previous = host.ui.get_host_presentation_generation();
        assert!(host
            .ui
            .global::<PaneSurfaceHostContext>()
            .set_scene_viewport_capture_for_surface(
                "editor.viewport",
                RenderViewportHandle::new(42),
                CapturedFrame::new(1, 1, vec![0, 0, 0, 255], 12),
            ));
        host.ui
            .request_redraw_region(scene_viewport_surface_damage_frame(
                &host.ui,
                "editor.viewport",
            ));
        complete_main_image_and_assert_metadata(host, &previous);
    });
}

#[test]
fn viewport_image_capture_metadata_ignores_secondary_no_change_and_empty_batch() {
    with_capture_metadata_host(|host| {
        let previous = host.ui.get_host_presentation_generation();
        let child = native_child("native:metadata");
        assert!(publish_viewport_product(
            &child,
            "native:metadata",
            RenderViewportProduct::new(RenderViewportHandle::new(43), 8, 8, 13),
        ));
        host.complete_viewport_image_update(previous.viewport_generation(), true);
        let child_only = host.ui.get_host_presentation_generation();
        assert_eq!(child_only.cursor(), previous.cursor());
        assert_same_capture_metadata(&previous, &child_only);

        host.complete_viewport_image_update(previous.viewport_generation(), false);
        assert_same_capture_metadata(&previous, &host.ui.get_host_presentation_generation());

        // A mixed batch still captures the actual main cursor after both deliveries.
        assert!(publish_viewport_product(
            &child,
            "native:metadata",
            RenderViewportProduct::new(RenderViewportHandle::new(43), 8, 8, 14),
        ));
        assert!(publish_viewport_product(
            &host.ui,
            "editor.viewport",
            RenderViewportProduct::new(RenderViewportHandle::new(44), 8, 8, 14),
        ));
        complete_main_image_and_assert_metadata(host, &previous);
        let current = host.ui.get_host_presentation_generation();
        let updated = publish_viewport_product(
            &host.ui,
            "editor.viewport",
            RenderViewportProduct::new(RenderViewportHandle::new(44), 8, 8, 14),
        );
        assert!(!updated);
        host.complete_viewport_image_update(current.viewport_generation(), updated);
        let duplicate = host.ui.get_host_presentation_generation();
        assert_eq!(duplicate.cursor(), current.cursor());
        assert_same_capture_metadata(&current, &duplicate);
    });
}

#[test]
fn viewport_image_capture_metadata_waits_for_pending_source_and_projection() {
    with_capture_metadata_host(|host| {
        let previous = host.ui.get_host_presentation_generation();
        host.workbench_window_bridge
            .mutate_control_property_for_test(
                "SetProjectionMode",
                "value_text",
                UiValue::String("Pending source".into()),
            )
            .unwrap();
        assert!(host
            .workbench_window_bridge
            .has_pending_surface_state_change());
        assert!(publish_viewport_product(
            &host.ui,
            "editor.viewport",
            RenderViewportProduct::new(RenderViewportHandle::new(45), 8, 8, 15),
        ));
        host.complete_viewport_image_update(previous.viewport_generation(), true);
        let pending_source = host.ui.get_host_presentation_generation();
        assert_same_capture_metadata(&previous, &pending_source);
        assert!(host
            .workbench_window_bridge
            .has_pending_surface_state_change());

        host.workbench_window_bridge
            .refresh_prepared_state_change()
            .unwrap();
        assert!(!host
            .workbench_window_bridge
            .has_pending_surface_state_change());
        assert!(host
            .workbench_window_bridge
            .has_pending_host_projection_commit());
        assert!(publish_viewport_product(
            &host.ui,
            "editor.viewport",
            RenderViewportProduct::new(RenderViewportHandle::new(45), 8, 8, 16),
        ));
        host.complete_viewport_image_update(pending_source.viewport_generation(), true);
        assert_same_capture_metadata(&previous, &host.ui.get_host_presentation_generation());
        assert!(host
            .workbench_window_bridge
            .has_pending_host_projection_commit());
    });
}

#[test]
fn viewport_image_capture_metadata_clears_invalid_current_getter_after_real_commit() {
    with_capture_metadata_host(|host| {
        let previous = host.ui.get_host_presentation_generation();
        host.workbench_window_bridge
            .mutate_control_property_for_test(
                "UnrealViewportToolbar",
                "toolbar_overflow_row_control",
                UiValue::String("MissingActualRow".into()),
            )
            .unwrap();
        host.workbench_window_bridge
            .refresh_prepared_state_change()
            .unwrap();
        commit_workbench_source(host);
        assert!(host
            .workbench_window_bridge
            .prepared_viewport_toolbar_publication()
            .is_none());
        assert!(host
            .ui
            .get_host_presentation_generation()
            .structure()
            .normal_viewport_toolbar
            .is_some());
        let committed = host.ui.get_host_presentation_generation();
        assert!(publish_viewport_product(
            &host.ui,
            "editor.viewport",
            RenderViewportProduct::new(RenderViewportHandle::new(46), 8, 8, 17),
        ));
        let image = host.ui.get_host_presentation_generation();
        host.complete_viewport_image_update(committed.viewport_generation(), true);
        let captured = host.ui.get_host_presentation_generation();
        assert_eq!(captured.cursor(), image.cursor());
        assert!(captured.structure().normal_viewport_toolbar.is_none());
        assert_eq!(
            captured
                .structure()
                .normal_arranged_inventory
                .as_ref()
                .unwrap()
                .0,
            captured.cursor(),
        );
        assert!(previous.structure().normal_viewport_toolbar.is_some());
    });
}

fn complete_main_image_and_assert_metadata(
    host: &mut RetainedEditorHost,
    previous: &HostPresentationGeneration,
) {
    let image = host.ui.get_host_presentation_generation();
    assert_eq!(
        image.viewport_generation(),
        previous.viewport_generation() + 1
    );
    assert_eq!(
        image.structure_generation(),
        previous.structure_generation()
    );
    assert_eq!(image.geometry_generation(), previous.geometry_generation());
    assert_eq!(
        image.interaction_generation(),
        previous.interaction_generation()
    );
    assert_eq!(image.hit_test_generation(), previous.hit_test_generation());
    assert_eq!(image.theme_generation(), previous.theme_generation());
    assert_eq!(
        image.diagnostics_generation(),
        previous.diagnostics_generation()
    );
    let (source, _) = host
        .workbench_window_bridge
        .prepared_viewport_toolbar_publication()
        .expect("the real current prepared bridge must remain valid");
    let layout_pass_count = host.workbench_window_bridge.layout_pass_count();
    let rebuild_count = host.ui.presentation_rebuild_count_for_test();
    let _ = host.ui.take_external_redraw_for_test();

    host.complete_viewport_image_update(previous.viewport_generation(), true);

    let captured = host.ui.get_host_presentation_generation();
    assert_eq!(captured.cursor(), image.cursor());
    assert!(captured.text_input_focus() == image.text_input_focus());
    assert_eq!(
        host.workbench_window_bridge.layout_pass_count(),
        layout_pass_count
    );
    assert_eq!(host.ui.presentation_rebuild_count_for_test(), rebuild_count);
    assert!(!host.ui.take_external_redraw_for_test().request_redraw());
    let toolbar = captured
        .structure()
        .normal_viewport_toolbar
        .as_ref()
        .unwrap();
    assert_eq!(toolbar.0, captured.cursor());
    assert!(Arc::ptr_eq(&toolbar.1, &source));
    assert_eq!(toolbar.2["surfaceGeneration"], source.generation);
    assert_eq!(
        captured
            .structure()
            .normal_arranged_inventory
            .as_ref()
            .unwrap()
            .0,
        captured.cursor(),
    );
    let old_toolbar = previous
        .structure()
        .normal_viewport_toolbar
        .as_ref()
        .unwrap();
    assert_eq!(old_toolbar.0, previous.cursor());
    assert_ne!(old_toolbar.0, toolbar.0);
    assert_eq!(old_toolbar.2, toolbar.2);
    match (
        previous.structure().normal_workbench_business.as_ref(),
        captured.structure().normal_workbench_business.as_ref(),
    ) {
        (Some(before), Some(after)) => {
            assert_eq!(before.0, after.0);
            assert!(Arc::ptr_eq(&before.1, &after.1));
        }
        (None, None) => {}
        _ => panic!("image-only metadata capture must not rebuild business state"),
    }
}

fn assert_same_capture_metadata(
    previous: &HostPresentationGeneration,
    current: &HostPresentationGeneration,
) {
    let before = previous
        .structure()
        .normal_viewport_toolbar
        .as_ref()
        .unwrap();
    let after = current
        .structure()
        .normal_viewport_toolbar
        .as_ref()
        .unwrap();
    assert_eq!(before.0, after.0);
    assert!(Arc::ptr_eq(&before.1, &after.1));
    assert!(Arc::ptr_eq(&before.2, &after.2));
    let before = previous
        .structure()
        .normal_arranged_inventory
        .as_ref()
        .unwrap();
    let after = current
        .structure()
        .normal_arranged_inventory
        .as_ref()
        .unwrap();
    assert_eq!(before.0, after.0);
    assert!(Arc::ptr_eq(&before.1, &after.1));
}

fn commit_workbench_source(host: &mut RetainedEditorHost) {
    let bridge = &host.workbench_window_bridge;
    let mut presentation = host.ui.get_host_presentation();
    presentation.workbench_window_nodes =
        to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
            Some(bridge.host_projection()),
            None,
            bridge.layout_frames().mount_frame,
            bridge.presentation_scale_factor(),
        );
    host.ui.set_host_presentation(presentation);
    host.workbench_window_bridge
        .mark_host_projection_committed();
    assert!(!host
        .workbench_window_bridge
        .has_pending_surface_state_change());
    assert!(!host
        .workbench_window_bridge
        .has_pending_host_projection_commit());
}

fn with_capture_metadata_host(run: impl FnOnce(&mut RetainedEditorHost)) {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let _restore_config =
        crate::tests::editor_event::support::EnvironmentVariableRestoreGuard::capture(
            "ZIRCON_CONFIG_PATH",
        );
    let _restore_review =
        crate::tests::editor_event::support::EnvironmentVariableRestoreGuard::capture(
            "ZIRCON_EDITOR_WORKBENCH_REVIEW_PROJECT_ROOT",
        );
    struct OwnedConfig(std::path::PathBuf);
    impl Drop for OwnedConfig {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let config = OwnedConfig(std::env::temp_dir().join(format!(
        "zircon_viewport_capture_metadata_{}_{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    )));
    std::env::set_var("ZIRCON_CONFIG_PATH", &config.0);
    std::env::set_var(
        "ZIRCON_EDITOR_WORKBENCH_REVIEW_PROJECT_ROOT",
        env!("CARGO_MANIFEST_DIR"),
    );
    let core = zircon_runtime::core::CoreRuntime::new();
    for engine_module in zircon_runtime::builtin::runtime_core_modules() {
        core.register_module(engine_module.descriptor()).unwrap();
    }
    core.register_module(zircon_runtime::ui::module_descriptor())
        .unwrap();
    core.register_module(crate::ui::host::module::module_descriptor())
        .unwrap();
    core.activate_module(zircon_runtime::foundation::FOUNDATION_MODULE_NAME)
        .unwrap();
    core.activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
        .unwrap();
    core.activate_module(crate::ui::host::module::EDITOR_MODULE_NAME)
        .unwrap();
    crate::tests::support::configure_editor_test_runtime_build_set(&core);
    let ui = UiHostWindow::new().unwrap();
    let mut host = RetainedEditorHost::new_for_test(core.handle(), ui).unwrap();
    commit_workbench_source(&mut host);
    host.publish_current_normal_source_capture_metadata();
    let prepared = host.ui.get_host_presentation_generation();
    assert_eq!(
        prepared
            .structure()
            .normal_viewport_toolbar
            .as_ref()
            .unwrap()
            .0,
        prepared.cursor()
    );
    assert_eq!(
        prepared
            .structure()
            .normal_arranged_inventory
            .as_ref()
            .unwrap()
            .0,
        prepared.cursor()
    );
    let _ = host.ui.take_external_redraw_for_test();
    run(&mut host);
}

fn native_child(surface_key: &str) -> UiHostWindow {
    let ui = UiHostWindow::new().expect("native child window");
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_shell.native_floating_window_mode = true;
    presentation.host_shell.native_floating_window_id = surface_key.into();
    ui.set_host_presentation(presentation);
    let _ = ui.take_external_redraw_for_test();
    ui
}
