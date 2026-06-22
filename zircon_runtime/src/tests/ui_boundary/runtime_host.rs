use crate::core::CoreRuntime;

#[test]
fn ui_runtime_module_registers_real_driver_and_manager_services() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(crate::ui::module_descriptor())
        .unwrap();
    runtime.activate_module(crate::ui::UI_MODULE_NAME).unwrap();

    let _driver = runtime
        .resolve_driver::<crate::ui::UiRuntimeDriver>(crate::ui::UI_RUNTIME_DRIVER_NAME)
        .unwrap();
    let _manager = runtime
        .resolve_manager::<crate::ui::event_ui::UiEventManager>(crate::ui::UI_EVENT_MANAGER_NAME)
        .unwrap();
}

#[test]
fn runtime_ui_host_surface_splits_production_frame_from_test_support() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();
    let public_runtime_frame_source =
        std::fs::read_to_string(runtime_root.join("src/ui/public_runtime_frame.rs"))
            .unwrap_or_default();
    let runtime_ui_support_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/tests/runtime_ui_support/mod.rs"))
            .unwrap_or_default();
    let runtime_ui_manager_error_source = std::fs::read_to_string(
        runtime_root.join("src/ui/tests/runtime_ui_support/runtime_ui_manager_error.rs"),
    )
    .unwrap_or_default();
    let graphics_runtime_mod_source =
        std::fs::read_to_string(runtime_root.join("src/graphics/runtime/mod.rs"))
            .unwrap_or_default();
    let graphics_lib_source =
        std::fs::read_to_string(runtime_root.join("src/graphics/mod.rs")).unwrap_or_default();
    let ui_mod_normalized = ui_mod_source.replace("\r\n", "\n");
    let runtime_ui_support_mod_normalized = runtime_ui_support_mod_source.replace("\r\n", "\n");

    assert!(
        !runtime_root.join("src/ui/runtime_ui/mod.rs").exists(),
        "old production runtime UI host subtree should stay removed"
    );
    assert!(
        runtime_root
            .join("src/ui/tests/runtime_ui_support/mod.rs")
            .exists(),
        "runtime UI manager and fixture support should live under ui/tests/runtime_ui_support"
    );
    assert!(
        ui_mod_normalized.contains("mod public_runtime_frame;")
            && ui_mod_normalized.contains(
                "#[cfg(test)]\n#[path = \"tests/runtime_ui_support/mod.rs\"]\nmod runtime_ui_support;",
            )
            && ui_mod_normalized
                .contains("pub(crate) use public_runtime_frame::PublicRuntimeFrame;")
            && ui_mod_normalized.contains(
                "#[cfg(test)]\npub(crate) use runtime_ui_support::{RuntimeUiFixture, RuntimeUiManager};",
            ),
        "zircon_runtime::ui should keep PublicRuntimeFrame production-owned and mount manager/fixtures only for tests"
    );
    assert!(
        !ui_mod_normalized.contains("mod runtime_ui;")
            && !ui_mod_normalized.contains("#[allow(dead_code)]"),
        "zircon_runtime::ui should not keep the old production runtime_ui module or dead-code allowance"
    );
    assert!(
        runtime_ui_support_mod_normalized
            .contains("pub(crate) use runtime_ui_fixture::RuntimeUiFixture;")
            && runtime_ui_support_mod_normalized
                .contains("pub(crate) use runtime_ui_manager::RuntimeUiManager;"),
        "runtime UI test-support subtree should re-export manager and fixtures to zircon_runtime::ui tests"
    );
    assert!(
        !runtime_ui_support_mod_source.contains("PublicRuntimeFrame"),
        "runtime UI test support should not own the production frame DTO"
    );
    for required in ["RuntimeUiFixture", "RuntimeUiManager"] {
        assert!(
            runtime_ui_support_mod_source.contains(required),
            "runtime UI test-support subtree should own `{required}`"
        );
    }
    for required in [
        "pub(crate) struct PublicRuntimeFrame",
        "ui: Option<UiRenderExtract>",
    ] {
        assert!(
            public_runtime_frame_source.contains(required),
            "production runtime UI frame owner should keep `{required}`"
        );
    }
    assert!(
        runtime_ui_manager_error_source.contains("enum RuntimeUiManagerError"),
        "runtime UI test-support subtree should still own the internal runtime UI manager error type"
    );
    for forbidden in [
        "pub use runtime_ui::{RuntimeUiFixture, RuntimeUiManager};",
        "pub use runtime_ui::PublicRuntimeFrame;",
    ] {
        assert!(
            !ui_mod_source.contains(forbidden),
            "zircon_runtime::ui should stop publicly exporting runtime UI host/demo seam `{forbidden}`"
        );
    }
    for forbidden in [
        "mod ui;",
        "RuntimeUiFixture",
        "RuntimeUiManager",
        "RuntimeUiManagerError",
    ] {
        assert!(
            !graphics_runtime_mod_source.contains(forbidden)
                && !graphics_lib_source.contains(forbidden),
            "zircon_runtime::graphics should not leak runtime UI host surface `{forbidden}` at the graphics crate root"
        );
    }
}

#[test]
fn runtime_ui_manager_builds_all_builtin_fixtures_into_shared_surfaces() {
    let viewport_size = crate::core::math::UVec2::new(1280, 720);
    let mut manager = crate::ui::RuntimeUiManager::new(viewport_size);

    for fixture in [
        crate::ui::RuntimeUiFixture::HudOverlay,
        crate::ui::RuntimeUiFixture::PauseMenu,
        crate::ui::RuntimeUiFixture::SettingsDialog,
        crate::ui::RuntimeUiFixture::InventoryList,
        crate::ui::RuntimeUiFixture::QuestLogDialog,
    ] {
        manager.load_builtin_fixture(fixture).unwrap();

        let surface = manager.surface();
        assert_eq!(surface.tree.roots.len(), 1);
        assert!(
            surface.render_extract.list.commands.len() >= 4,
            "expected fixture {fixture:?} to build a non-trivial shared visual tree"
        );
        assert_eq!(
            manager.build_frame().viewport_size,
            viewport_size,
            "runtime UI frame should preserve viewport size for {fixture:?}"
        );
        assert!(
            manager.build_frame().ui.is_some(),
            "runtime UI frame should carry a shared UI render extract for {fixture:?}"
        );
    }
}

#[test]
fn runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface() {
    use zircon_runtime_interface::ui::dispatch::{
        UiNavigationDispatchEffect, UiPointerDispatchEffect, UiPointerEvent,
    };
    use zircon_runtime_interface::ui::layout::UiPoint;
    use zircon_runtime_interface::ui::surface::{
        UiNavigationEventKind, UiPointerButton, UiPointerEventKind,
    };

    let viewport_size = crate::core::math::UVec2::new(640, 360);
    let mut manager = crate::ui::RuntimeUiManager::new(viewport_size);
    manager
        .load_builtin_fixture(crate::ui::RuntimeUiFixture::PauseMenu)
        .unwrap();

    let root_node = manager.surface().tree.roots[0];
    manager.register_pointer_handler(root_node, UiPointerEventKind::Down, |_| {
        UiPointerDispatchEffect::capture()
    });

    let pointer_result = manager
        .dispatch_pointer_event(
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(320.0, 180.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(pointer_result.captured_by, Some(root_node));
    assert_eq!(manager.surface().focus.captured, Some(root_node));

    manager.register_navigation_handler(root_node, UiNavigationEventKind::Activate, |_| {
        UiNavigationDispatchEffect::Handled
    });

    let navigation_result = manager
        .dispatch_navigation_event(UiNavigationEventKind::Activate)
        .unwrap();
    assert_eq!(navigation_result.handled_by, Some(root_node));
}

#[test]
fn runtime_ui_manager_applies_pointer_render_dirty_to_persistent_surface() {
    use std::collections::BTreeSet;
    use zircon_runtime_interface::ui::dispatch::{UiPointerDispatchEffect, UiPointerEvent};
    use zircon_runtime_interface::ui::layout::UiPoint;
    use zircon_runtime_interface::ui::surface::UiPointerEventKind;
    use zircon_runtime_interface::ui::tree::UiDirtyFlags;

    let viewport_size = crate::core::math::UVec2::new(640, 360);
    let mut manager = crate::ui::RuntimeUiManager::new(viewport_size);
    manager
        .load_builtin_fixture(crate::ui::RuntimeUiFixture::PauseMenu)
        .unwrap();

    let root_node = manager.surface().tree.roots[0];
    manager.register_pointer_handler(root_node, UiPointerEventKind::Move, |_| {
        UiPointerDispatchEffect::request_dirty(UiDirtyFlags {
            render: true,
            ..UiDirtyFlags::default()
        })
    });

    let result = manager
        .dispatch_pointer_event(UiPointerEvent::new(
            UiPointerEventKind::Move,
            UiPoint::new(320.0, 180.0),
        ))
        .unwrap();
    let report = manager.surface().last_rebuild_report;
    let mut expected_dirty_nodes = BTreeSet::from([root_node]);
    expected_dirty_nodes.extend(result.route.entered.iter().copied());

    assert!(result.requested_dirty.render);
    assert!(report.dirty_flags.render);
    assert_eq!(report.dirty_node_count, expected_dirty_nodes.len());
    assert!(report.render_rebuilt);
    assert!(!report.layout_recomputed);
    assert!(!report.arranged_rebuilt);
    assert!(!report.hit_grid_rebuilt);
    assert!(
        !manager.surface().dirty_flags().any(),
        "runtime manager should consume dispatch dirty domains through the persistent surface"
    );
}

#[test]
fn runtime_ui_manager_routes_pointer_layout_dirty_through_incremental_surface_rebuild() {
    use zircon_runtime_interface::ui::dispatch::{UiPointerDispatchEffect, UiPointerEvent};
    use zircon_runtime_interface::ui::layout::UiPoint;
    use zircon_runtime_interface::ui::surface::UiPointerEventKind;
    use zircon_runtime_interface::ui::tree::UiDirtyFlags;

    let viewport_size = crate::core::math::UVec2::new(640, 360);
    let mut manager = crate::ui::RuntimeUiManager::new(viewport_size);
    manager
        .load_builtin_fixture(crate::ui::RuntimeUiFixture::InventoryList)
        .unwrap();

    let root_node = manager.surface().tree.roots[0];
    manager.register_pointer_handler(root_node, UiPointerEventKind::Move, |_| {
        UiPointerDispatchEffect::request_dirty(UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            ..UiDirtyFlags::default()
        })
    });

    let result = manager
        .dispatch_pointer_event(UiPointerEvent::new(
            UiPointerEventKind::Move,
            UiPoint::new(320.0, 180.0),
        ))
        .unwrap();
    let report = manager.surface().last_rebuild_report;

    assert!(result.requested_dirty.layout);
    assert!(report.dirty_flags.layout);
    assert!(report.layout_recomputed);
    assert!(report.arranged_rebuilt);
    assert!(report.hit_grid_rebuilt);
    assert!(report.render_rebuilt);
    assert!(
        !manager.surface().dirty_flags().any(),
        "layout dirty requests should be rebuilt and cleared before the next runtime frame"
    );
}

#[test]
fn runtime_ui_manager_clears_node_bound_input_handlers_when_fixture_changes() {
    use zircon_runtime_interface::ui::dispatch::{UiPointerDispatchEffect, UiPointerEvent};
    use zircon_runtime_interface::ui::layout::UiPoint;
    use zircon_runtime_interface::ui::surface::{UiPointerButton, UiPointerEventKind};

    let viewport_size = crate::core::math::UVec2::new(640, 360);
    let mut manager = crate::ui::RuntimeUiManager::new(viewport_size);
    manager
        .load_builtin_fixture(crate::ui::RuntimeUiFixture::PauseMenu)
        .unwrap();

    let pause_target = manager.surface().tree.roots[0];
    manager.register_pointer_handler(pause_target, UiPointerEventKind::Down, |_| {
        UiPointerDispatchEffect::capture()
    });
    let captured = manager
        .dispatch_pointer_event(
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(320.0, 180.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(captured.captured_by, Some(pause_target));

    manager
        .load_builtin_fixture(crate::ui::RuntimeUiFixture::InventoryList)
        .unwrap();
    let after_reload = manager
        .dispatch_pointer_event(
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(320.0, 180.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(
        after_reload.captured_by, None,
        "node-bound runtime UI handlers must not survive across a rebuilt fixture surface"
    );
}

#[test]
fn runtime_ui_manager_clears_phase_and_navigation_handlers_when_fixture_changes() {
    use zircon_runtime_interface::ui::dispatch::{
        UiDispatchPhase, UiNavigationDispatchEffect, UiPointerDispatchEffect, UiPointerEvent,
    };
    use zircon_runtime_interface::ui::layout::UiPoint;
    use zircon_runtime_interface::ui::surface::{
        UiNavigationEventKind, UiPointerButton, UiPointerEventKind,
    };

    let viewport_size = crate::core::math::UVec2::new(640, 360);
    let mut manager = crate::ui::RuntimeUiManager::new(viewport_size);
    manager
        .load_builtin_fixture(crate::ui::RuntimeUiFixture::PauseMenu)
        .unwrap();

    let pause_root = manager.surface().tree.roots[0];
    manager.register_pointer_phase_handler(
        pause_root,
        UiPointerEventKind::Down,
        UiDispatchPhase::PreviewTunnel,
        |_| UiPointerDispatchEffect::handled(),
    );
    manager.register_navigation_handler(pause_root, UiNavigationEventKind::Activate, |_| {
        UiNavigationDispatchEffect::Handled
    });

    let preview_handled = manager
        .dispatch_pointer_event(
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(320.0, 180.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(preview_handled.handled_by, Some(pause_root));
    assert_eq!(
        preview_handled
            .invocations
            .last()
            .map(|invocation| invocation.phase),
        Some(UiDispatchPhase::PreviewTunnel)
    );

    let navigation_handled = manager
        .dispatch_navigation_event(UiNavigationEventKind::Activate)
        .unwrap();
    assert_eq!(navigation_handled.handled_by, Some(pause_root));

    manager
        .load_builtin_fixture(crate::ui::RuntimeUiFixture::InventoryList)
        .unwrap();

    let after_reload_pointer = manager
        .dispatch_pointer_event(
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(320.0, 180.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert!(
        after_reload_pointer.invocations.is_empty(),
        "phase-qualified runtime UI handlers must not survive across a rebuilt fixture surface"
    );
    assert_eq!(after_reload_pointer.handled_by, None);

    let after_reload_navigation = manager
        .dispatch_navigation_event(UiNavigationEventKind::Activate)
        .unwrap();
    assert!(
        after_reload_navigation.invocations.is_empty(),
        "navigation handlers must not survive across a rebuilt fixture surface"
    );
    assert_eq!(after_reload_navigation.handled_by, None);
}
