use super::*;

struct ReentrantBridgeBindingIterator<'a> {
    host: &'a NativePluginLiveHost,
    yielded: bool,
}

impl Iterator for ReentrantBridgeBindingIterator<'_> {
    type Item = NativeBridgeMethodBinding;

    fn next(&mut self) -> Option<Self::Item> {
        if self.yielded {
            return None;
        }
        self.yielded = true;
        assert!(
            self.host.loaded.is_unlocked(),
            "caller iterators must be consumed before the live-host loaded mutex is acquired"
        );
        Some(NativeBridgeMethodBinding::new(
            <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            "sample_count",
            NativeBridgeMethodFn::from_rust(native_live_host_bridge_method),
        ))
    }
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
fn native_live_host_consumes_caller_binding_iterator_before_locking_loaded_plugins() {
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
        ReentrantBridgeBindingIterator {
            host: &host,
            yielded: false,
        },
    )
    .expect("reentrant-safe caller bindings should install");

    assert_eq!(
        host.installed_runtime_bridge_method_binding_count("physics"),
        Ok(1)
    );
}

unsafe extern "C" fn ownerless_abi_bridge_method(
    _call: super::super::super::abi_declarations::NativePluginBridgeMethodCallV3,
) -> ZrStatus {
    ZrStatus::ok()
}

#[test]
fn native_live_host_safe_binding_install_rejects_ownerless_abi_callback() {
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
        .install_runtime_bridge_method_bindings_result(
            "physics",
            [NativeBridgeMethodBinding::new(
                <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
                "sample_count",
                NativeBridgeMethodFn::from_abi_v3(ownerless_abi_bridge_method, 7),
            )],
        )
        .expect_err("safe manual installation must reject ownerless ABI callbacks");

    assert!(matches!(
        &error,
        NativePluginBridgeMethodError::AbiBindingRequiresLoadedGenerationOwner {
            plugin_id,
            interface_id,
            method_name,
        } if plugin_id == "physics"
            && interface_id == <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID
            && method_name == "sample_count"
    ));
}

#[test]
fn native_live_host_binding_authority_keeps_callback_and_owner_in_one_generation() {
    let host = NativePluginLiveHost::default();
    let old_plugin = native_live_host_test_plugin_with_bridge_manifest("physics");
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            old_plugin.clone(),
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
    .expect("old binding authority should install");

    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_bridge_manifest("physics"),
        );
    }
    host.invalidate_runtime_registration_replay_generation("physics");
    old_plugin
        .begin_lifecycle_transition()
        .expect("old generation should enter transition");

    let lifecycle = native_live_host_bridge_lifecycle_state(false);
    let scope = host
        .runtime_bridge_call_scope_from_installed_bindings("physics", &lifecycle)
        .expect("installed authority should retain its own generation");
    let status = unsafe {
        (scope.api().bridge.call.unwrap())(
            scope.handle(),
            0,
            7,
            std::ptr::null(),
            0,
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::BridgeNotEnabled);
    old_plugin.cancel_lifecycle_transition();
}

#[test]
fn native_live_host_scope_and_slot_helpers_share_one_bridge_generation() {
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
    let loaded_lock_acquisitions = host.live_host_diagnostics().loaded_lock_acquisitions;

    let method_slot = host
        .runtime_bridge_method_slot_result(
            "physics",
            &state,
            <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            "sample_count",
        )
        .expect("cold slot lookup should build the shared bridge generation");
    let first = host
        .runtime_bridge_call_scope_from_installed_bindings("physics", &state)
        .expect("first scope should reuse the bridge generation");
    let second = host
        .runtime_bridge_call_scope_from_installed_bindings("physics", &state)
        .expect("second scope should reuse the bridge generation");
    let counts = host.registration_replay_context_build_counts();
    let loaded_lock_delta = host
        .live_host_diagnostics()
        .loaded_lock_acquisitions
        .saturating_sub(loaded_lock_acquisitions);

    assert_eq!(first.handle(), second.handle());
    assert_eq!(method_slot, 7);
    assert_eq!(loaded_lock_delta, 0);
    assert_eq!(counts.package_manifest_snapshots, 0);
    assert_eq!(counts.binding_snapshots, 0);
    assert_eq!(counts.method_lookup_builds, 1);
    assert_eq!(counts.bridge_call_scope_builds, 1);
}

#[test]
fn native_live_host_runs_one_million_stable_slot_lookups_without_allocating() {
    const LOOKUPS: usize = 1_000_000;

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
    let _scope = host
        .runtime_bridge_call_scope_from_installed_bindings("physics", &state)
        .expect("scope should build the stable bridge generation");

    let (last_slot, allocation_count) = count_native_live_host_test_allocations(|| {
        let mut last_slot = 0;
        for _ in 0..LOOKUPS {
            last_slot = host
                .runtime_bridge_method_slot_result(
                    "physics",
                    &state,
                    <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
                    "sample_count",
                )
                .expect("stable slot lookup should remain indexed");
        }
        last_slot
    });

    let counts = host.registration_replay_context_build_counts();
    assert_eq!(last_slot, 7);
    assert_eq!(allocation_count, 0);
    assert_eq!(counts.package_manifest_snapshots, 0);
    assert_eq!(counts.binding_snapshots, 0);
    assert_eq!(counts.method_lookup_builds, 1);
    assert_eq!(counts.bridge_call_scope_builds, 1);
}

#[test]
fn native_live_host_binding_reload_rebuilds_only_the_affected_bridge_generation() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        for plugin_id in ["physics", "audio"] {
            loaded.insert(
                live_key(PluginModuleKind::Runtime, plugin_id),
                native_live_host_test_plugin_with_bridge_manifest(plugin_id),
            );
        }
    }
    for plugin_id in ["physics", "audio"] {
        host.install_runtime_bridge_method_bindings(
            plugin_id,
            [NativeBridgeMethodBinding::new(
                <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
                "sample_count",
                NativeBridgeMethodFn::from_rust(native_live_host_bridge_method),
            )],
        )
        .expect("loaded manifest should validate installed bridge bindings");
    }
    let state = native_live_host_bridge_lifecycle_state(false);
    let physics = host
        .runtime_bridge_call_scope_from_installed_bindings("physics", &state)
        .expect("physics generation should build");
    let audio = host
        .runtime_bridge_call_scope_from_installed_bindings("audio", &state)
        .expect("audio generation should build");
    assert_eq!(
        host.registration_replay_context_build_counts()
            .method_lookup_builds,
        2
    );

    host.install_runtime_bridge_method_bindings(
        "physics",
        [NativeBridgeMethodBinding::new(
            <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            "sample_count",
            NativeBridgeMethodFn::from_rust(native_live_host_bridge_method),
        )],
    )
    .expect("replacement physics binding generation should install");
    let retained_audio = host
        .runtime_bridge_call_scope_from_installed_bindings("audio", &state)
        .expect("unaffected audio generation should be retained");
    let reloaded_physics = host
        .runtime_bridge_call_scope_from_installed_bindings("physics", &state)
        .expect("affected physics generation should rebuild");
    let counts = host.registration_replay_context_build_counts();

    assert_eq!(retained_audio.handle(), audio.handle());
    assert_ne!(reloaded_physics.handle(), physics.handle());
    assert_eq!(counts.package_manifest_snapshots, 0);
    assert_eq!(counts.binding_snapshots, 0);
    assert_eq!(counts.method_lookup_builds, 3);
    assert_eq!(counts.bridge_call_scope_builds, 3);
}

#[test]
fn native_live_host_concurrent_scope_builds_publish_one_bridge_generation() {
    const WORKERS: usize = 16;

    let host = Arc::new(NativePluginLiveHost::default());
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
    let lifecycle = Arc::new(native_live_host_bridge_lifecycle_state(false));
    let barrier = Arc::new(std::sync::Barrier::new(WORKERS));

    let handles = (0..WORKERS)
        .map(|_| {
            let host = host.clone();
            let lifecycle = lifecycle.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                host.runtime_bridge_call_scope_from_installed_bindings("physics", &lifecycle)
                    .expect("concurrent scope should reuse the published generation")
                    .handle()
                    .raw()
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|worker| worker.join().expect("scope worker should join"))
        .collect::<Vec<_>>();
    let counts = host.registration_replay_context_build_counts();

    assert!(handles.iter().all(|handle| *handle == handles[0]));
    assert_eq!(counts.package_manifest_snapshots, 0);
    assert_eq!(counts.binding_snapshots, 0);
    assert_eq!(counts.method_lookup_builds, 1);
    assert_eq!(counts.bridge_call_scope_builds, 1);
}

#[test]
fn native_live_host_auto_installs_discovered_bridge_bindings_from_load_report() {
    let host = NativePluginLiveHost::default();
    let mut load_report = NativePluginLoadReport::default();
    load_report.push_loaded(native_live_host_test_plugin_with_discovered_bridge_table(
        "physics",
    ));

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
    host.install_runtime_bridge_method_bindings(
        "physics",
        [NativeBridgeMethodBinding::new(
            <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            "sample_count",
            NativeBridgeMethodFn::from_rust(native_live_host_bridge_method),
        )],
    )
    .expect("reloaded manifest should publish a replacement validated binding generation");
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
    host.install_runtime_bridge_method_bindings(
        "physics",
        [NativeBridgeMethodBinding::new(
            <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            "sample_count",
            NativeBridgeMethodFn::from_rust(native_live_host_bridge_method),
        )],
    )
    .expect("reloaded manifest should publish a replacement validated binding generation");

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
    let error = host
        .install_runtime_bridge_method_bindings("physics", std::iter::empty())
        .expect_err("missing native bridge method binding should be rejected");

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

    let error = host
        .runtime_bridge_method_slot_result(
            "physics",
            &state,
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
