#[test]
fn root_workbench_slint_exports_only_generic_host_bootstrap_symbols() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));

    for forbidden in [
        "WorkbenchHostContext",
        "WorkbenchHostScaffold",
        "HostWorkbenchWindowSceneData",
    ] {
        assert!(
            !workbench.contains(forbidden),
            "root workbench.slint still exports workbench-specific host symbol `{forbidden}`"
        );
    }

    for required in [
        "UiHostContext",
        "UiHostScaffold",
        "HostWindowSceneData",
        "export component UiHostWindow inherits Window",
    ] {
        assert!(
            workbench.contains(required),
            "root workbench.slint is missing generic host symbol `{required}`"
        );
    }
}

#[test]
fn slint_host_build_uses_material_style() {
    let build_script = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/build.rs"));

    assert!(
        build_script.contains(".with_style(\"material\".into())"),
        "Slint host build should use the Material style while business UI remains in .ui.toml"
    );
}

#[test]
fn slint_host_presentation_uses_generic_scene_data_property() {
    let host_root = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_root.slint"
    ));
    let host_contract = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_surface_contract.slint"
    ));
    let host_surface = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_surface.slint"
    ));

    for (path, source) in [
        ("ui/workbench/host_root.slint", host_root),
        ("ui/workbench/host_surface_contract.slint", host_contract),
        ("ui/workbench/host_surface.slint", host_surface),
    ] {
        assert!(
            !source.contains("workbench_scene_data"),
            "{path} still exposes workbench-specific scene data property"
        );
        assert!(
            source.contains("host_scene_data"),
            "{path} is missing generic host_scene_data property"
        );
    }
}

#[test]
fn slint_host_scene_uses_generic_surface_metrics_and_orchestration_names() {
    let host_components = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_components.slint"
    ));
    let host_scene = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_scene.slint"
    ));

    for (path, source) in [
        ("ui/workbench/host_components.slint", host_components),
        ("ui/workbench/host_scene.slint", host_scene),
    ] {
        for forbidden in [
            "HostWorkbenchSurfaceMetricsData",
            "HostWorkbenchSurfaceOrchestrationData",
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} still exposes workbench-specific surface contract `{forbidden}`"
            );
        }

        for required in [
            "HostWindowSurfaceMetricsData",
            "HostWindowSurfaceOrchestrationData",
        ] {
            assert!(
                source.contains(required),
                "{path} is missing generic surface contract `{required}`"
            );
        }
    }
}

#[test]
fn slint_host_drag_and_resize_callbacks_use_generic_host_event_names() {
    let host_context = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_context.slint"
    ));
    let host_resize_layer = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_resize_layer.slint"
    ));
    let host_tab_drag_overlay = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_tab_drag_overlay.slint"
    ));

    for (path, source) in [
        ("ui/workbench/host_context.slint", host_context),
        ("ui/workbench/host_resize_layer.slint", host_resize_layer),
        (
            "ui/workbench/host_tab_drag_overlay.slint",
            host_tab_drag_overlay,
        ),
    ] {
        for forbidden in [
            "workbench_drag_pointer_event",
            "workbench_resize_pointer_event",
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} still exposes workbench-specific host callback `{forbidden}`"
            );
        }
    }

    for required in ["host_drag_pointer_event", "host_resize_pointer_event"] {
        assert!(
            host_context.contains(required),
            "ui/workbench/host_context.slint is missing generic host callback `{required}`"
        );
    }
    assert!(
        host_tab_drag_overlay.contains("UiHostContext.host_drag_pointer_event("),
        "host tab drag overlay should dispatch through generic host_drag_pointer_event"
    );
    assert!(
        host_resize_layer.contains("UiHostContext.host_resize_pointer_event("),
        "host resize layer should dispatch through generic host_resize_pointer_event"
    );
}

#[test]
fn host_page_pointer_module_uses_generic_host_type_names() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let host_page_pointer_dir = manifest_dir.join("src/ui/slint_host/host_page_pointer");
    let callback_host_page =
        manifest_dir.join("src/ui/slint_host/callback_dispatch/shared_pointer/host_page.rs");
    let app_root = manifest_dir.join("src/ui/slint_host/app.rs");

    let mut checked_sources = Vec::new();
    collect_rs_sources(&host_page_pointer_dir, &mut checked_sources);
    checked_sources.push(callback_host_page);
    checked_sources.push(app_root);

    for source_path in checked_sources {
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in [
            "WorkbenchHostPagePointer",
            "build_workbench_host_page_pointer_layout",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} still exposes workbench-specific host page pointer symbol `{forbidden}`",
                source_path.display()
            );
        }
    }
}

#[test]
fn menu_binding_host_event_uses_editor_host_names() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    for relative_path in [
        "src/ui/workbench/event/mod.rs",
        "src/ui/workbench/event/dispatch_editor_host_binding.rs",
        "src/ui/workbench/event/editor_host_event.rs",
        "src/ui/workbench/event/editor_host_event_error.rs",
        "src/ui/binding_dispatch/editor_event_normalization.rs",
        "src/ui/slint_host/callback_dispatch/workbench/menu_action.rs",
        "src/tests/workbench/host_events/menu_binding.rs",
    ] {
        let source_path = manifest_dir.join(relative_path);
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in [
            "WorkbenchHostEvent",
            "dispatch_workbench_binding",
            "workbench_host_event",
        ] {
            assert!(
                !source.contains(forbidden),
                "{relative_path} still exposes workbench-specific menu host symbol `{forbidden}`"
            );
        }
    }

    let event_mod =
        std::fs::read_to_string(manifest_dir.join("src/ui/workbench/event/mod.rs")).unwrap();
    for required in [
        "EditorHostEvent",
        "dispatch_editor_host_binding",
        "editor_host_event",
    ] {
        assert!(
            event_mod.contains(required),
            "workbench event module is missing editor host menu symbol `{required}`"
        );
    }
}

#[test]
fn menu_pointer_module_uses_generic_host_type_names() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let menu_pointer_dir = manifest_dir.join("src/ui/slint_host/menu_pointer");

    let mut checked_sources = Vec::new();
    collect_rs_sources(&menu_pointer_dir, &mut checked_sources);
    for relative_path in [
        "src/ui/slint_host/app.rs",
        "src/ui/slint_host/app/host_lifecycle.rs",
        "src/ui/slint_host/app/pointer_layout.rs",
        "src/ui/slint_host/callback_dispatch/shared_pointer/menu.rs",
        "src/tests/host/slint_menu_pointer/support.rs",
        "src/tests/host/slint_menu_pointer/surface_contract.rs",
    ] {
        checked_sources.push(manifest_dir.join(relative_path));
    }

    for source_path in checked_sources {
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in [
            "WorkbenchMenuPointer",
            "build_workbench_menu_pointer_layout",
            "workbench_menu_pointer",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} still exposes workbench-specific menu pointer symbol `{forbidden}`",
                source_path.display()
            );
        }
    }

    let menu_pointer_mod = std::fs::read_to_string(menu_pointer_dir.join("mod.rs")).unwrap();
    for required in [
        "HostMenuPointerBridge",
        "HostMenuPointerLayout",
        "HostMenuPointerState",
        "build_host_menu_pointer_layout",
    ] {
        assert!(
            menu_pointer_mod.contains(required),
            "menu pointer module is missing generic host menu pointer symbol `{required}`"
        );
    }
}

#[test]
fn activity_rail_pointer_module_uses_generic_host_type_names() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let pointer_dir = manifest_dir.join("src/ui/slint_host/activity_rail_pointer");

    let mut checked_sources = Vec::new();
    collect_rs_sources(&pointer_dir, &mut checked_sources);
    for relative_path in [
        "src/ui/slint_host/app.rs",
        "src/ui/slint_host/app/host_lifecycle.rs",
        "src/ui/slint_host/app/pointer_layout.rs",
        "src/ui/slint_host/app/workbench_pointer.rs",
        "src/ui/slint_host/callback_dispatch/shared_pointer/activity_rail.rs",
        "src/tests/host/slint_activity_rail_pointer/support.rs",
    ] {
        checked_sources.push(manifest_dir.join(relative_path));
    }

    for source_path in checked_sources {
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in [
            "WorkbenchActivityRailPointer",
            "build_workbench_activity_rail_pointer_layout",
            "workbench_activity_rail_pointer",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} still exposes workbench-specific activity rail pointer symbol `{forbidden}`",
                source_path.display()
            );
        }
    }

    let pointer_mod = std::fs::read_to_string(pointer_dir.join("mod.rs")).unwrap();
    for required in [
        "HostActivityRailPointerBridge",
        "HostActivityRailPointerSide",
        "build_host_activity_rail_pointer_layout",
    ] {
        assert!(
            pointer_mod.contains(required),
            "activity rail pointer module is missing generic host symbol `{required}`"
        );
    }
}

#[test]
fn document_tab_pointer_module_uses_generic_host_type_names() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let pointer_dir = manifest_dir.join("src/ui/slint_host/document_tab_pointer");

    let mut checked_sources = Vec::new();
    collect_rs_sources(&pointer_dir, &mut checked_sources);
    for relative_path in [
        "src/ui/slint_host/app.rs",
        "src/ui/slint_host/app/workspace_docking.rs",
        "src/ui/slint_host/callback_dispatch/shared_pointer/document_tab.rs",
        "src/tests/host/slint_document_tab_pointer/dispatch.rs",
        "src/tests/host/slint_document_tab_pointer/floating_strip_bounds.rs",
    ] {
        checked_sources.push(manifest_dir.join(relative_path));
    }

    for source_path in checked_sources {
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in [
            "WorkbenchDocumentTabPointer",
            "build_workbench_document_tab_pointer_layout",
            "workbench_document_tab_pointer",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} still exposes workbench-specific document tab pointer symbol `{forbidden}`",
                source_path.display()
            );
        }
    }

    let pointer_mod = std::fs::read_to_string(pointer_dir.join("mod.rs")).unwrap();
    for required in [
        "HostDocumentTabPointerBridge",
        "HostDocumentTabPointerRoute",
        "build_host_document_tab_pointer_layout",
    ] {
        assert!(
            pointer_mod.contains(required),
            "document tab pointer module is missing generic host symbol `{required}`"
        );
    }
}

#[test]
fn drawer_header_pointer_module_uses_generic_host_type_names() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let pointer_dir = manifest_dir.join("src/ui/slint_host/drawer_header_pointer");

    let mut checked_sources = Vec::new();
    collect_rs_sources(&pointer_dir, &mut checked_sources);
    for relative_path in [
        "src/ui/slint_host/app.rs",
        "src/ui/slint_host/app/host_lifecycle.rs",
        "src/ui/slint_host/app/pointer_layout.rs",
        "src/ui/slint_host/callback_dispatch/shared_pointer/drawer_header.rs",
        "src/tests/host/slint_drawer_header_pointer/dispatch.rs",
        "src/tests/host/slint_drawer_header_pointer/layout_projection.rs",
        "src/tests/host/slint_drawer_header_pointer/pointer_bridge.rs",
        "src/tests/host/slint_drawer_header_pointer/support.rs",
    ] {
        checked_sources.push(manifest_dir.join(relative_path));
    }

    for source_path in checked_sources {
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in [
            "WorkbenchDrawerHeaderPointer",
            "build_workbench_drawer_header_pointer_layout",
            "workbench_drawer_header_pointer",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} still exposes workbench-specific drawer header pointer symbol `{forbidden}`",
                source_path.display()
            );
        }
    }

    let pointer_mod = std::fs::read_to_string(pointer_dir.join("mod.rs")).unwrap();
    for required in [
        "HostDrawerHeaderPointerBridge",
        "HostDrawerHeaderPointerRoute",
        "build_host_drawer_header_pointer_layout",
    ] {
        assert!(
            pointer_mod.contains(required),
            "drawer header pointer module is missing generic host symbol `{required}`"
        );
    }
}

#[test]
fn shell_pointer_module_uses_generic_host_type_names() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let pointer_dir = manifest_dir.join("src/ui/slint_host/shell_pointer");

    let mut checked_sources = Vec::new();
    collect_rs_sources(&pointer_dir, &mut checked_sources);
    for relative_path in [
        "src/ui/slint_host/shell_pointer.rs",
        "src/ui/slint_host/app.rs",
        "src/ui/slint_host/app/host_lifecycle.rs",
        "src/ui/slint_host/app/workbench_pointer.rs",
        "src/ui/slint_host/app/workspace_docking.rs",
        "src/ui/slint_host/drawer_resize.rs",
        "src/ui/slint_host/tab_drag/bridge.rs",
        "src/ui/slint_host/tab_drag/group.rs",
        "src/ui/slint_host/tab_drag/route_resolution.rs",
        "src/ui/slint_host/tab_drag.rs",
        "src/tests/host/slint_drawer_resize/pointer_bridge.rs",
        "src/tests/host/slint_tab_drag/support.rs",
        "src/tests/host/slint_tab_drag/document_routes.rs",
        "src/tests/host/slint_tab_drag/floating_pointer.rs",
        "src/tests/host/slint_tab_drag/floating_routes.rs",
        "src/tests/host/slint_tab_drag/root_projection.rs",
    ] {
        checked_sources.push(manifest_dir.join(relative_path));
    }

    for source_path in checked_sources {
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in ["WorkbenchShellPointer", "workbench_shell_pointer"] {
            assert!(
                !source.contains(forbidden),
                "{} still exposes workbench-specific shell pointer symbol `{forbidden}`",
                source_path.display()
            );
        }
    }

    let pointer_mod =
        std::fs::read_to_string(manifest_dir.join("src/ui/slint_host/shell_pointer.rs")).unwrap();
    for required in [
        "HostShellPointerBridge",
        "HostShellPointerRoute",
        "host_shell_pointer_route_group_key",
    ] {
        assert!(
            pointer_mod.contains(required)
                || std::fs::read_to_string(manifest_dir.join("src/ui/slint_host/tab_drag.rs"))
                    .unwrap()
                    .contains(required),
            "shell pointer surface is missing generic host symbol `{required}`"
        );
    }
}

#[test]
fn drawer_resize_module_uses_generic_host_type_names() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let checked_sources = [
        "src/ui/slint_host/drawer_resize.rs",
        "src/ui/slint_host/shell_pointer/bridge.rs",
        "src/ui/slint_host/shell_pointer/route.rs",
        "src/ui/slint_host/app/workspace_docking.rs",
        "src/tests/host/slint_drawer_resize/pointer_bridge.rs",
        "src/tests/host/slint_drawer_resize/resize_target.rs",
    ];

    for relative_path in checked_sources {
        let source_path = manifest_dir.join(relative_path);
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in [
            "WorkbenchResizeTargetGroup",
            "resolve_workbench_resize_target_group",
            "WORKBENCH_POINTER_",
            "WorkbenchPointerFactKind",
            "map_workbench_pointer_kind",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} still exposes workbench-specific resize host symbol `{forbidden}`",
                source_path.display()
            );
        }
    }

    let drawer_resize =
        std::fs::read_to_string(manifest_dir.join("src/ui/slint_host/drawer_resize.rs")).unwrap();
    let workspace_docking =
        std::fs::read_to_string(manifest_dir.join("src/ui/slint_host/app/workspace_docking.rs"))
            .unwrap();
    for required in [
        "HostResizeTargetGroup",
        "resolve_host_resize_target_group",
        "HOST_POINTER_",
        "HostPointerFactKind",
        "map_host_pointer_kind",
    ] {
        assert!(
            drawer_resize.contains(required) || workspace_docking.contains(required),
            "drawer resize host surface is missing generic host symbol `{required}`"
        );
    }
}

#[test]
fn root_shell_frames_use_generic_host_type_names() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let checked_sources = [
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench/root_shell_frames.rs",
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs",
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench/host_projection.rs",
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench/mod.rs",
        "src/ui/slint_host/callback_dispatch/template_bridge/mod.rs",
        "src/ui/slint_host/callback_dispatch/mod.rs",
        "src/ui/slint_host/root_shell_projection.rs",
        "src/ui/slint_host/activity_rail_pointer/build_host_activity_rail_pointer_layout.rs",
        "src/ui/slint_host/document_tab_pointer/build_host_document_tab_pointer_layout.rs",
        "src/ui/slint_host/drawer_header_pointer/build_host_drawer_header_pointer_layout.rs",
        "src/ui/slint_host/host_page_pointer/build_host_page_pointer_layout.rs",
        "src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs",
        "src/ui/slint_host/shell_pointer/bridge.rs",
        "src/ui/slint_host/shell_pointer/drag_surface.rs",
        "src/ui/slint_host/tab_drag/bridge.rs",
        "src/ui/slint_host/tab_drag/drop_resolution.rs",
        "src/ui/slint_host/tab_drag/route_resolution.rs",
        "src/ui/slint_host/tab_drag/strip_hitbox.rs",
        "src/ui/slint_host/ui/apply_presentation.rs",
        "src/tests/host/slint_host_page_pointer/layout_projection.rs",
        "src/tests/host/slint_menu_pointer/layout.rs",
        "src/tests/host/slint_menu_pointer/support.rs",
        "src/tests/host/slint_tab_drag/support.rs",
        "src/tests/host/slint_tab_drag/drag_target_groups.rs",
        "src/tests/host/slint_tab_drag/root_projection.rs",
    ];

    for relative_path in checked_sources {
        let source_path = manifest_dir.join(relative_path);
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in [
            "BuiltinWorkbenchRootShellFrames",
            "workbench_body_frame",
            "build_builtin_workbench_host_projection",
            "apply_builtin_workbench_host_strip_layout",
            "WORKBENCH_BODY_CONTROL_ID",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} still exposes workbench-specific root shell frame symbol `{forbidden}`",
                source_path.display()
            );
        }
    }

    let root_shell_frames = std::fs::read_to_string(manifest_dir.join(
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench/root_shell_frames.rs",
    ))
    .unwrap();
    for required in ["BuiltinHostRootShellFrames", "host_body_frame"] {
        assert!(
            root_shell_frames.contains(required),
            "root shell frames DTO is missing generic host symbol `{required}`"
        );
    }

    let bridge = std::fs::read_to_string(
        manifest_dir
            .join("src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs"),
    )
    .unwrap();
    let host_projection =
        std::fs::read_to_string(manifest_dir.join(
            "src/ui/slint_host/callback_dispatch/template_bridge/workbench/host_projection.rs",
        ))
        .unwrap();
    for required in [
        "build_builtin_host_window_projection",
        "apply_builtin_host_window_strip_layout",
        "HOST_BODY_CONTROL_ID",
    ] {
        assert!(
            bridge.contains(required) || host_projection.contains(required),
            "host projection bridge is missing generic host symbol `{required}`"
        );
    }
}

#[test]
fn host_window_template_bridge_uses_generic_host_type_names() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let checked_roots = [
        "src/ui/slint_host/callback_dispatch",
        "src/ui/slint_host/app.rs",
        "src/ui/slint_host/app/host_lifecycle.rs",
        "src/ui/slint_host/ui/tests.rs",
        "src/tests/host",
        "tests/integration_contracts/structure_roots.rs",
    ];

    let mut checked_sources = Vec::new();
    for relative_path in checked_roots {
        let path = manifest_dir.join(relative_path);
        if path.is_dir() {
            collect_rs_sources(&path, &mut checked_sources);
        } else {
            checked_sources.push(path);
        }
    }

    checked_sources.retain(|path| {
        path.file_name().and_then(|name| name.to_str()) != Some("generic_host_boundary.rs")
    });

    for source_path in checked_sources {
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in [
            "BuiltinWorkbenchTemplateBridge",
            "BuiltinWorkbenchTemplateBridgeError",
            "builtin_workbench_template_bridge",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} still exposes workbench-specific host window template bridge symbol `{forbidden}`",
                source_path.display()
            );
        }
    }

    let bridge_mod = std::fs::read_to_string(
        manifest_dir.join("src/ui/slint_host/callback_dispatch/template_bridge/workbench/mod.rs"),
    )
    .unwrap();
    let bridge = std::fs::read_to_string(
        manifest_dir
            .join("src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs"),
    )
    .unwrap();
    let error = std::fs::read_to_string(
        manifest_dir.join("src/ui/slint_host/callback_dispatch/template_bridge/workbench/error.rs"),
    )
    .unwrap();

    for required in [
        "BuiltinHostWindowTemplateBridge",
        "BuiltinHostWindowTemplateBridgeError",
    ] {
        assert!(
            bridge_mod.contains(required) || bridge.contains(required) || error.contains(required),
            "host window template bridge is missing generic host symbol `{required}`"
        );
    }
}

#[test]
fn host_template_runtime_loads_through_generic_host_entry() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let checked_sources = [
        "src/ui/template_runtime/slint_adapter.rs",
        "src/ui/template_runtime/runtime/runtime_host.rs",
        "src/ui/template_runtime/builtin/template_documents.rs",
        "src/ui/template_runtime/builtin/mod.rs",
        "src/ui/template_runtime/mod.rs",
        "src/ui/slint_host/callback_dispatch/template_bridge/projection_support.rs",
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench_drawer_source/bridge.rs",
        "src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/bridge.rs",
        "src/tests/host/slint_callback_dispatch/template_bridge/drawer_source_surface.rs",
        "src/tests/host/template_runtime/support.rs",
        "src/tests/host/template_runtime/host_window_document.rs",
        "src/tests/ui/template/binding_resolution.rs",
        "src/tests/ui/template/catalog_registry.rs",
        "src/tests/ui/template/repository_assets.rs",
        "src/tests/ui/template/support.rs",
    ];

    for relative_path in checked_sources {
        let source_path = manifest_dir.join(relative_path);
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in [
            "load_builtin_workbench_shell",
            "WORKBENCH_SHELL_DOCUMENT_ID",
            "WORKBENCH_DRAWER_SOURCE_DOCUMENT_ID",
            "EDITOR_WORKBENCH_ASSET_TOML",
            "WorkbenchShell",
            "\"WorkbenchShell\" => Self::Root",
            "builtin_workbench_drawer_source_document",
            "projects_builtin_workbench_template",
            "keeps_legacy_workbench_shell_document_alias",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} still exposes workbench-specific builtin host runtime symbol `{forbidden}`",
                source_path.display()
            );
        }
    }
}

#[test]
fn layout_dispatch_uses_generic_host_entry_names() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let checked_sources = [
        "src/ui/slint_host/callback_dispatch/layout/document_tab.rs",
        "src/ui/slint_host/callback_dispatch/layout/drawer_toggle.rs",
        "src/ui/slint_host/callback_dispatch/layout/main_page.rs",
        "src/ui/slint_host/callback_dispatch/layout/mod.rs",
        "src/ui/slint_host/callback_dispatch/mod.rs",
        "src/ui/slint_host/callback_dispatch/shared_pointer/activity_rail.rs",
        "src/ui/slint_host/callback_dispatch/shared_pointer/document_tab.rs",
        "src/ui/slint_host/callback_dispatch/shared_pointer/drawer_header.rs",
        "src/ui/slint_host/callback_dispatch/shared_pointer/host_page.rs",
        "src/tests/host/slint_callback_dispatch/support.rs",
        "src/tests/host/slint_callback_dispatch/layout/document_tabs.rs",
        "src/tests/host/slint_callback_dispatch/layout/drawer_toggle.rs",
        "src/tests/host/slint_callback_dispatch/layout/host_page_activation.rs",
    ];

    for relative_path in checked_sources {
        let source_path = manifest_dir.join(relative_path);
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in [
            "dispatch_builtin_workbench_document_tab",
            "dispatch_builtin_workbench_drawer_toggle",
            "dispatch_builtin_workbench_host_page",
            "builtin_workbench_document_tab",
            "builtin_workbench_activity_toggle",
            "builtin_workbench_host_page",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} still exposes workbench-specific layout dispatch symbol `{forbidden}`",
                source_path.display()
            );
        }
    }
}

#[test]
fn host_menu_dispatch_uses_generic_host_entry_names() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let checked_sources = [
        "src/ui/slint_host/callback_dispatch/workbench/control.rs",
        "src/ui/slint_host/callback_dispatch/workbench/menu_action.rs",
        "src/ui/slint_host/callback_dispatch/workbench/mod.rs",
        "src/ui/slint_host/callback_dispatch/mod.rs",
        "src/ui/slint_host/callback_dispatch/shared_pointer/menu.rs",
        "src/tests/host/slint_callback_dispatch/support.rs",
        "src/tests/host/slint_callback_dispatch/workbench/template_bridge.rs",
    ];

    for relative_path in checked_sources {
        let source_path = manifest_dir.join(relative_path);
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in [
            "dispatch_builtin_workbench_control",
            "dispatch_builtin_workbench_menu_action",
            "dispatch_workbench_menu_action_with_template_fallback",
            "builtin_workbench_open_project",
            "builtin_workbench_reset_layout",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} still exposes workbench-specific host menu dispatch symbol `{forbidden}`",
                source_path.display()
            );
        }
    }
}

#[test]
fn host_layout_helpers_use_generic_document_root_names() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let checked_sources = [
        "src/ui/host/layout_hosts/ensure_host_document_root.rs",
        "src/ui/host/layout_hosts/mod.rs",
        "src/ui/host/layout_hosts/repair_builtin_shell_layout.rs",
    ];

    for relative_path in checked_sources {
        let source_path = manifest_dir.join(relative_path);
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in ["ensure_workbench_document_root"] {
            assert!(
                !source.contains(forbidden),
                "{} still exposes workbench-specific host layout helper `{forbidden}`",
                source_path.display()
            );
        }
    }
}

#[test]
fn drawer_source_bridge_uses_generic_host_type_names() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let checked_sources = [
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs",
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench/error.rs",
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench_drawer_source/bridge.rs",
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench_drawer_source/error.rs",
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench_drawer_source/layout.rs",
        "src/ui/slint_host/callback_dispatch/constants.rs",
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench_drawer_source/mod.rs",
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench_drawer_source/source_frames.rs",
        "src/ui/slint_host/callback_dispatch/template_bridge/mod.rs",
        "src/ui/slint_host/callback_dispatch/mod.rs",
        "src/tests/host/slint_callback_dispatch/support.rs",
        "src/tests/host/slint_callback_dispatch/template_bridge/drawer_source_projection.rs",
    ];

    for relative_path in checked_sources {
        let source_path = manifest_dir.join(relative_path);
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in [
            "BuiltinWorkbenchDrawerSourceTemplateBridge",
            "BuiltinWorkbenchDrawerSourceFrames",
            "BuiltinWorkbenchDrawerLayoutInputs",
            "BuiltinWorkbenchDrawerRegionInput",
            "BUILTIN_WORKBENCH_DRAWER_SOURCE_DOCUMENT_ID",
            "build_builtin_workbench_drawer_source_surface",
            "apply_builtin_workbench_drawer_source_layout",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} still exposes workbench-specific drawer source symbol `{forbidden}`",
                source_path.display()
            );
        }
    }

    let drawer_source_mod = std::fs::read_to_string(manifest_dir.join(
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench_drawer_source/mod.rs",
    ))
    .unwrap();
    let drawer_source_layout = std::fs::read_to_string(manifest_dir.join(
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench_drawer_source/layout.rs",
    ))
    .unwrap();
    let drawer_source_frames = std::fs::read_to_string(manifest_dir.join(
        "src/ui/slint_host/callback_dispatch/template_bridge/workbench_drawer_source/source_frames.rs",
    ))
    .unwrap();
    for required in [
        "BuiltinHostDrawerSourceTemplateBridge",
        "BuiltinHostDrawerSourceFrames",
        "BuiltinHostDrawerLayoutInputs",
        "build_builtin_host_drawer_source_surface",
        "apply_builtin_host_drawer_source_layout",
    ] {
        assert!(
            drawer_source_mod.contains(required)
                || drawer_source_layout.contains(required)
                || drawer_source_frames.contains(required),
            "drawer source bridge is missing generic host symbol `{required}`"
        );
    }
}

#[test]
fn tab_drag_module_uses_generic_host_type_names() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let tab_drag_dir = manifest_dir.join("src/ui/slint_host/tab_drag");

    let mut checked_sources = Vec::new();
    collect_rs_sources(&tab_drag_dir, &mut checked_sources);
    for relative_path in [
        "src/ui/slint_host/tab_drag.rs",
        "src/ui/slint_host/shell_pointer/bridge.rs",
        "src/ui/slint_host/shell_pointer/drag_surface.rs",
        "src/ui/slint_host/shell_pointer/effects.rs",
        "src/ui/slint_host/shell_pointer/route.rs",
        "src/ui/slint_host/app/workspace_docking.rs",
        "src/ui/slint_host/callback_dispatch/layout/tab_drop.rs",
        "src/ui/template_runtime/harness.rs",
        "src/tests/host/slint_callback_dispatch/layout/tab_drop.rs",
        "src/tests/host/slint_callback_dispatch/support.rs",
        "src/tests/host/slint_tab_drag/drag_target_groups.rs",
        "src/tests/host/slint_tab_drag/document_routes.rs",
        "src/tests/host/slint_tab_drag/floating_pointer.rs",
        "src/tests/host/slint_tab_drag/floating_routes.rs",
        "src/tests/host/slint_tab_drag/root_projection.rs",
        "src/tests/host/slint_tab_drag/support.rs",
        "tests/integration_contracts/host_drag_targets.rs",
    ] {
        checked_sources.push(manifest_dir.join(relative_path));
    }

    for source_path in checked_sources {
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        for forbidden in [
            "WorkbenchDragTargetGroup",
            "WorkbenchDragTargetBridge",
            "ResolvedWorkbenchTabDrop",
            "resolve_workbench_drag_target_group",
            "resolve_workbench_tab_drop",
            "workbench_drag_target",
            "workbench_tab_drop",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} still exposes workbench-specific tab drag symbol `{forbidden}`",
                source_path.display()
            );
        }
    }

    let tab_drag_mod =
        std::fs::read_to_string(manifest_dir.join("src/ui/slint_host/tab_drag.rs")).unwrap();
    let tab_drag_bridge =
        std::fs::read_to_string(manifest_dir.join("src/ui/slint_host/tab_drag/bridge.rs")).unwrap();
    for required in [
        "HostDragTargetGroup",
        "HostDragTargetBridge",
        "ResolvedHostTabDropRoute",
        "ResolvedHostTabDropTarget",
        "resolve_host_drag_target_group",
        "resolve_host_tab_drop_route",
    ] {
        assert!(
            tab_drag_mod.contains(required) || tab_drag_bridge.contains(required),
            "tab drag module is missing generic host symbol `{required}`"
        );
    }
}

fn collect_rs_sources(dir: &std::path::Path, sources: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
    {
        let entry = entry
            .unwrap_or_else(|error| panic!("failed to read entry in {}: {error}", dir.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_rs_sources(&path, sources);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            sources.push(path);
        }
    }
}
