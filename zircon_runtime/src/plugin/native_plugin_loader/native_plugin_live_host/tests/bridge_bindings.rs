use super::*;

#[test]
fn native_live_host_builds_bridge_call_scope_from_loaded_manifest() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_bridge_manifest("physics"),
        );
    }
    let state = native_live_host_bridge_lifecycle_state(false);
    let slot = state
        .bridge_table()
        .resolve_slot(<dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID)
        .expect("test bridge interface slot");

    let scope = host
        .runtime_bridge_call_scope_from_loaded_manifest(
            "physics",
            &state,
            [NativeBridgeMethodBinding::new(
                <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
                "sample_count",
                NativeBridgeMethodFn::from_rust(native_live_host_bridge_method),
            )],
        )
        .expect("loaded manifest should build native bridge call scope");
    let api = scope.api();
    let payload = b"ping";

    let status = unsafe {
        (api.bridge.call.unwrap())(
            scope.handle(),
            slot.raw(),
            7,
            payload.as_ptr(),
            payload.len(),
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::CapabilityDenied);
}

#[test]
fn native_live_host_reuses_installed_bridge_bindings_for_loaded_manifest_scopes() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_bridge_manifest("physics"),
        );
    }
    host.install_runtime_bridge_method_bindings(
        "physics",
        [NativeBridgeMethodBinding::new(
            <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            "sample_count",
            NativeBridgeMethodFn::from_rust(native_live_host_bridge_method),
        )],
    )
    .expect("loaded manifest should validate installed bridge bindings");
    let state = native_live_host_bridge_lifecycle_state(false);
    let slot = state
        .bridge_table()
        .resolve_slot(<dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID)
        .expect("test bridge interface slot");

    let scope = host
        .runtime_bridge_call_scope_from_installed_bindings("physics", &state)
        .expect("installed bindings should build native bridge call scope");
    let api = scope.api();
    let payload = b"ping";
    let status = unsafe {
        (api.bridge.call.unwrap())(
            scope.handle(),
            slot.raw(),
            7,
            payload.as_ptr(),
            payload.len(),
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::CapabilityDenied);
}

#[test]
fn native_live_host_auto_installs_discovered_bridge_bindings_from_load_report() {
    let host = NativePluginLiveHost::default();
    let load_report = NativePluginLoadReport {
        discovered: Vec::new(),
        loaded: vec![native_live_host_test_plugin_with_discovered_bridge_table(
            "physics",
        )],
        diagnostics: Vec::new(),
    };

    let report = host
        .load_reported_plugins(load_report, PluginModuleKind::Runtime)
        .expect("runtime load report should install discovered bridge bindings");

    assert_eq!(report.loaded_plugin_ids, vec!["physics".to_string()]);
    assert!(report.diagnostics.iter().any(|diagnostic: &String| {
        diagnostic.contains("native.live_host.bridge_bindings_discovered")
            && diagnostic.contains("installed 1 bridge method")
    }));
    let state = native_live_host_bridge_lifecycle_state(false);
    let slot = state
        .bridge_table()
        .resolve_slot(<dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID)
        .expect("test bridge interface slot");
    let scope = host
        .runtime_bridge_call_scope_from_installed_bindings("physics", &state)
        .expect("discovered bindings should be available through installed binding scope");
    let api = scope.api();
    let payload = b"ping";
    let status = unsafe {
        (api.bridge.call.unwrap())(
            scope.handle(),
            slot.raw(),
            7,
            payload.as_ptr(),
            payload.len(),
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::CapabilityDenied);
}

#[test]
fn native_live_host_rebuilds_bridge_scope_from_reloaded_manifest_and_installed_bindings() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_bridge_manifest("physics"),
        );
    }
    host.install_runtime_bridge_method_bindings(
        "physics",
        [NativeBridgeMethodBinding::new(
            <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            "sample_count",
            NativeBridgeMethodFn::from_rust(native_live_host_bridge_method),
        )],
    )
    .expect("initial manifest should validate installed bridge bindings");
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_bridge_manifest_slot("physics", 9),
        );
    }
    let state = native_live_host_bridge_lifecycle_state(false);
    let slot = state
        .bridge_table()
        .resolve_slot(<dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID)
        .expect("test bridge interface slot");

    let scope = host
        .runtime_bridge_call_scope_from_installed_bindings("physics", &state)
        .expect("reloaded manifest should rebuild descriptors from installed bindings");
    let api = scope.api();
    let payload = b"ping";
    let status = unsafe {
        (api.bridge.call.unwrap())(
            scope.handle(),
            slot.raw(),
            9,
            payload.as_ptr(),
            payload.len(),
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::CapabilityDenied);
}

#[test]
fn native_live_host_reloads_bridge_lifecycle_and_installed_binding_scope() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_bridge_manifest("physics"),
        );
    }
    host.install_runtime_bridge_method_bindings(
        "physics",
        [NativeBridgeMethodBinding::new(
            <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            "sample_count",
            NativeBridgeMethodFn::from_rust(native_live_host_bridge_method),
        )],
    )
    .expect("initial manifest should validate installed bridge bindings");
    let state = native_live_host_bridge_lifecycle_state(false);
    let slot = state
        .bridge_table()
        .resolve_slot(<dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID)
        .expect("test bridge interface slot");
    let original_generation = state.bridge_table().entry(slot).unwrap().generation();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_bridge_manifest_slot("physics", 9),
        );
    }

    let reload = host
        .reload_runtime_bridge_provider_and_scope_from_installed_bindings("physics", &state)
        .expect("hot reload should refresh lifecycle and bridge call descriptors");

    assert_eq!(
        reload.bridge_lifecycle_report.event.mode,
        BridgeOwnerTransitionMode::Reload
    );
    assert_eq!(reload.bridge_lifecycle_report.outcome.is_applied(), true);
    assert_eq!(reload.bridge_call_scope.method_count(), 1);
    assert_eq!(
        state.bridge_table().entry(slot).unwrap().generation(),
        original_generation + 2
    );
    let api = reload.bridge_call_scope.api();
    let payload = b"ping";
    let status = unsafe {
        (api.bridge.call.unwrap())(
            reload.bridge_call_scope.handle(),
            slot.raw(),
            9,
            payload.as_ptr(),
            payload.len(),
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::CapabilityDenied);
    assert!(reload
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("native.live_host.bridge_scope_reloaded")));
}

#[test]
fn native_live_host_rejects_installed_bridge_bindings_without_loaded_manifest() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin("physics", PluginModuleKind::Runtime),
        );
    }

    let result = host.install_runtime_bridge_method_bindings(
        "physics",
        [NativeBridgeMethodBinding::new(
            <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            "sample_count",
            NativeBridgeMethodFn::from_rust(native_live_host_bridge_method),
        )],
    );

    assert!(matches!(
        result,
        Err(message) if message == "runtime plugin physics has no package manifest"
    ));
}

#[test]
fn native_live_host_bridge_methods_report_typed_missing_manifest_error() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin("physics", PluginModuleKind::Runtime),
        );
    }

    let error = host
        .install_runtime_bridge_method_bindings_result(
            "physics",
            [NativeBridgeMethodBinding::new(
                <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
                "sample_count",
                NativeBridgeMethodFn::from_rust(native_live_host_bridge_method),
            )],
        )
        .expect_err("missing package manifest should produce typed bridge method error");

    assert!(matches!(
        &error,
        NativePluginBridgeMethodError::MissingPackageManifest { plugin_id }
            if plugin_id == "physics"
    ));
    assert_eq!(
        error.to_string(),
        "runtime plugin physics has no package manifest"
    );
}

#[test]
fn native_live_host_rejects_loaded_manifest_bridge_method_without_binding() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_bridge_manifest("physics"),
        );
    }
    let state = native_live_host_bridge_lifecycle_state(false);

    let error = match host.runtime_bridge_call_scope_from_loaded_manifest("physics", &state, []) {
        Ok(_) => panic!("missing native bridge method binding should be rejected"),
        Err(error) => error,
    };

    assert!(error.contains("native bridge method `native.live_host.bridge.v1.sample_count`"));
    assert!(error.contains("is declared but has no binding"));
}

#[test]
fn native_live_host_bridge_methods_report_typed_missing_method_slot_error() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_bridge_manifest("physics"),
        );
    }

    let error = host
        .runtime_bridge_method_slot_result(
            "physics",
            <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            "resample_count",
        )
        .expect_err("undeclared bridge method slot should produce typed bridge method error");

    assert!(matches!(
        &error,
        NativePluginBridgeMethodError::MissingDeclaredBridgeMethod {
            plugin_id,
            interface_id,
            method_name,
        } if plugin_id == "physics"
            && interface_id == <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID
            && method_name == "resample_count"
    ));
    assert_eq!(
        error.to_string(),
        "runtime plugin physics package manifest does not declare bridge method `native.live_host.bridge.v1.resample_count`"
    );
}
