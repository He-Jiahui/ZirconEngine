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
fn runtime_ui_host_surface_stays_internal_to_runtime_ui_subtree() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();
    let runtime_ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/runtime_ui/mod.rs")).unwrap_or_default();
    let runtime_ui_manager_error_source =
        std::fs::read_to_string(runtime_root.join("src/ui/runtime_ui/runtime_ui_manager_error.rs"))
            .unwrap_or_default();
    let graphics_runtime_mod_source =
        std::fs::read_to_string(runtime_root.join("src/graphics/runtime/mod.rs"))
            .unwrap_or_default();
    let graphics_lib_source =
        std::fs::read_to_string(runtime_root.join("src/graphics/mod.rs")).unwrap_or_default();

    assert!(
        runtime_root.join("src/ui/runtime_ui/mod.rs").exists(),
        "runtime ui host subtree should live under zircon_runtime::ui::runtime_ui"
    );
    assert!(
        ui_mod_source.contains("pub(crate) use runtime_ui::{RuntimeUiFixture, RuntimeUiManager};")
            && ui_mod_source.contains("pub(crate) use runtime_ui::PublicRuntimeFrame;"),
        "zircon_runtime::ui should keep runtime UI host/demo surface crate-private"
    );
    for required in ["RuntimeUiFixture", "RuntimeUiManager", "PublicRuntimeFrame"] {
        assert!(
            runtime_ui_mod_source.contains(required),
            "runtime ui host subtree should own `{required}`"
        );
    }
    assert!(
        runtime_ui_manager_error_source.contains("enum RuntimeUiManagerError"),
        "runtime ui host subtree should still own the internal runtime UI manager error type"
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
    use crate::ui::dispatch::{UiNavigationDispatcher, UiPointerDispatcher};
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
    let mut pointer_dispatcher = UiPointerDispatcher::default();
    pointer_dispatcher.register(root_node, UiPointerEventKind::Down, |_| {
        UiPointerDispatchEffect::Captured
    });

    let pointer_result = manager
        .dispatch_pointer_event(
            &pointer_dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(320.0, 180.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(pointer_result.captured_by, Some(root_node));
    assert_eq!(manager.surface().focus.captured, Some(root_node));

    let mut navigation_dispatcher = UiNavigationDispatcher::default();
    navigation_dispatcher.register(root_node, UiNavigationEventKind::Activate, |_| {
        UiNavigationDispatchEffect::Handled
    });

    let navigation_result = manager
        .dispatch_navigation_event(&navigation_dispatcher, UiNavigationEventKind::Activate)
        .unwrap();
    assert_eq!(navigation_result.handled_by, Some(root_node));
}
