#[test]
fn review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi() {
    let panic_guard = include_str!("../../../plugin/native_plugin_loader/ffi_panic_guard.rs");
    let host_api_adapter = include_str!("../../../plugin/native_plugin_loader/host_api_adapter.rs");
    let host_callbacks = include_str!("../../../plugin/native_plugin_loader/host_callbacks.rs");
    let native_loader_doc =
        include_str!("../../../../../docs/zircon_runtime/plugin/native_plugin_loader/index.md");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let module_doc =
        include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
    let status_rows = include_str!(
        "../plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs"
    );
    let status_map = include_str!(
        "../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
    );

    for required in [
        "catch_unwind(AssertUnwindSafe(call))",
        "catch_native_host_api_panic(call: impl FnOnce() -> ZrStatus) -> ZrStatus",
        "ZrStatusCode::Panic",
        "catch_native_plugin_host_callback_panic(call: impl FnOnce() -> u32) -> u32",
        "ZIRCON_NATIVE_PLUGIN_STATUS_PANIC",
    ] {
        assert!(
            panic_guard.contains(required),
            "native loader panic guard should contain `{required}`"
        );
    }

    for callback in [
        "native_host_register_system_v1",
        "native_host_register_component_v1",
        "native_host_spawn_command_v1",
        "native_host_asset_request_v1",
        "native_host_event_emit_v1",
        "native_host_event_drain_v1",
        "native_host_bridge_call_v1",
        "native_host_diagnostics_emit_v1",
        "native_host_diagnostics_metric_v1",
    ] {
        let callback_start = host_api_adapter
            .find(&format!("unsafe extern \"C\" fn {callback}"))
            .unwrap_or_else(|| panic!("missing public native host API callback `{callback}`"));
        let callback_body = &host_api_adapter[callback_start..];
        assert!(
            callback_body
                .lines()
                .take(12)
                .any(|line| line.contains("catch_native_host_api_panic")),
            "`{callback}` should enter catch_native_host_api_panic before host logic"
        );
    }

    for callback in [
        "native_host_abi_version_v3",
        "native_host_has_capability_v3",
        "native_host_log_v3",
        "native_host_diagnostic_v3",
    ] {
        let callback_start = host_callbacks
            .find(&format!("unsafe extern \"C\" fn {callback}"))
            .unwrap_or_else(|| panic!("missing private native host callback `{callback}`"));
        let callback_body = &host_callbacks[callback_start..];
        assert!(
            callback_body
                .lines()
                .take(8)
                .any(|line| line.contains("catch_native_plugin_host_callback_panic")),
            "`{callback}` should enter catch_native_plugin_host_callback_panic before host logic"
        );
    }

    for doc_anchor in [
        "Runtime 15 F1 native host callback panic guard",
        "runtime_15_native_host_callback_panic_guard_static_passed_cargo_deferred",
        "review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi",
        "p0_f1_f2_f4_top_row_closed_status_static_passed_cargo_deferred",
        "catch_native_host_api_panic",
        "catch_native_plugin_host_callback_panic",
        "ZIRCON_NATIVE_PLUGIN_STATUS_PANIC",
    ] {
        assert!(
            native_loader_doc.contains(doc_anchor)
                || review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || module_doc.contains(doc_anchor)
                || status_rows.contains(doc_anchor)
                || status_map.contains(doc_anchor)
                || date_map.contains(doc_anchor),
            "F1 native host callback panic guard docs/status should record `{doc_anchor}`"
        );
    }
    let f1_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F1 |"))
        .expect("F1 row should exist");
    assert!(
        f1_row.ends_with("| Runtime 15 + Runtime 06 + Plugins 11 / review closed |"),
        "F1 row should mark the panic-guard review state closed"
    );
}

#[test]
fn review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest() {
    let fixture =
        include_str!("../../../../../zircon_plugins/native_dynamic_fixture/native/src/lib.rs");
    let plugin_toml =
        include_str!("../../../../../zircon_plugins/native_dynamic_fixture/plugin.toml");
    let native_cargo =
        include_str!("../../../../../zircon_plugins/native_dynamic_fixture/native/Cargo.toml");
    let sdk_dist = include_str!("../../../../../zircon_plugins/plugin_sdk/src/dist.rs");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let native_fixture_record = review_findings
        .lines()
        .find(|line| {
            line.starts_with(
                "| 2026-06-28 | Plugins 13 native_dynamic_fixture 验证插件合法性审查 |",
            )
        })
        .expect("native_dynamic_fixture review completion record should exist");

    for required in [
        "zircon_plugin_sdk::native_dist_plugin_v3!",
        "package_manifest: PLUGIN_MANIFEST",
        "runtime_entry: zircon_native_dynamic_fixture_runtime_entry_v3",
        "editor_entry: zircon_native_dynamic_fixture_editor_entry_v3",
        "invoke_command: Some(fixture_invoke_command)",
        "native::catch_native_callback_panic(STATUS_PANIC_DIAGNOSTICS",
        "owned_bytes(response)",
    ] {
        assert!(
            fixture.contains(required),
            "native_dynamic_fixture should keep SDK macro-backed ABI owner `{required}`"
        );
    }
    assert!(
        fixture.contains(
            "const PLUGIN_MANIFEST: &str = concat!(include_str!(\"../../plugin.toml\"), \"\\0\");"
        ),
        "native_dynamic_fixture should embed plugin.toml as the single manifest source"
    );
    for stale_manual_abi in [
        "#[no_mangle]",
        "NativePluginDescriptorV3 {",
        "#[repr(C)]",
        "zircon_native_plugin_descriptor_v3(",
    ] {
        assert!(
            !fixture.contains(stale_manual_abi),
            "native_dynamic_fixture should not hand-write native ABI surface `{stale_manual_abi}`"
        );
    }
    assert!(
        native_cargo.contains(
            "zircon_plugin_sdk = { workspace = true, default-features = false, features = [\"native\"] }"
        ),
        "native fixture should consume the SDK native feature instead of defining ABI structs locally"
    );
    assert!(
        sdk_dist.contains("macro_rules! native_dist_plugin_v3")
            && sdk_dist.contains("pub use crate::{")
            && sdk_dist.contains("native_dist_editor_plugin_v3, native_dist_plugin_v3"),
        "plugin SDK should own and export the native dist macro surface"
    );

    for required in [
        "id = \"native_dynamic_fixture\"",
        "description = \"Real dynamic library fixture for ABI v3 native plugin loading with ABI v2 fallback coverage.\"",
        "descriptor_symbol = \"zircon_native_plugin_descriptor_v3\"",
        "runtime_entry = \"zircon_native_dynamic_fixture_runtime_entry_v3\"",
        "editor_entry = \"zircon_native_dynamic_fixture_editor_entry_v3\"",
        "\"runtime.plugin.native_dynamic_fixture\"",
        "\"runtime.asset.importer.native_dynamic_fixture.data_json\"",
        "\"editor.extension.native_dynamic_fixture\"",
    ] {
        assert!(
            plugin_toml.contains(required),
            "plugin.toml should keep native fixture manifest source `{required}`"
        );
    }

    let ds8_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D-S8 |"))
        .expect("D-S8 row should exist");
    let d3_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D3 |"))
        .expect("D3 row should exist");
    for required in [
        "native 插件 ABI v3 样板已由 plugin SDK macro 承接",
        "zircon_plugin_sdk::native_dist_plugin_v3!",
        "native_dynamic_fixture_validation_plugin_review_passed_unused_import_warning_fixed",
        "native_dynamic_fixture_importer_manifest_self_description_static_passed_cargo_deferred",
        "review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
        "ds8_d3_native_fixture_top_row_closed_status_static_passed_cargo_deferred",
    ] {
        assert!(
            ds8_row.contains(required),
            "D-S8 top review row should record current SDK macro state `{required}`"
        );
    }
    for required in [
        "native manifest 双写已由 plugin.toml 单源闭合",
        "concat!(include_str!(\"../../plugin.toml\"), \"\\0\")",
        "native_dynamic_fixture_validation_plugin_review_passed_unused_import_warning_fixed",
        "review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
        "ds8_d3_native_fixture_top_row_closed_status_static_passed_cargo_deferred",
    ] {
        assert!(
            d3_row.contains(required),
            "D3 top review row should record current single-manifest state `{required}`"
        );
    }
    for stale_text in [
        "native 插件零 SDK，手写 ~720 行 ABI v3",
        "native manifest 双写且已漂移",
        "native_dynamic_fixture/plugin.toml:6` vs `native/src/lib.rs:21-48",
    ] {
        assert!(
            !ds8_row.contains(stale_text) && !d3_row.contains(stale_text),
            "D-S8/D3 top rows should not keep stale unresolved text `{stale_text}`"
        );
    }
    assert!(
        ds8_row.ends_with("| Plugins 13 M2 + Plugins 12 / closed |"),
        "D-S8 row should mark native fixture SDK macro convergence closed"
    );
    assert!(
        d3_row.ends_with("| Plugins 13 M1 + Plugins 12 / closed |"),
        "D3 row should mark native fixture single-manifest convergence closed"
    );
    for doc_anchor in [
        "D-S8/D3 native dynamic fixture SDK macro and manifest single source top-table sync",
        "D-S8/D3 native fixture top-row closed status sync",
        "native_dynamic_fixture_validation_plugin_review_passed_unused_import_warning_fixed",
        "ds8_d3_native_fixture_top_row_closed_status_static_passed_cargo_deferred",
        "review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
    ] {
        assert!(
            review_findings.contains(doc_anchor) || native_fixture_record.contains(doc_anchor),
            "D-S8/D3 review docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_d13_native_fixture_importer_is_manifest_described() {
    let fixture =
        include_str!("../../../../../zircon_plugins/native_dynamic_fixture/native/src/lib.rs");
    let plugin_toml =
        include_str!("../../../../../zircon_plugins/native_dynamic_fixture/plugin.toml");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let package_doc = include_str!("../../../../../docs/zircon_runtime/plugin/package_manifest.md");
    let native_doc =
        include_str!("../../../../../docs/zircon_runtime/plugin/native_plugin_loader/index.md");

    for required in [
        "\"runtime.asset.importer.native_dynamic_fixture.data_json\"",
        "[[asset_importers]]",
        "id = \"native_dynamic_fixture.data_json\"",
        "plugin_id = \"native_dynamic_fixture\"",
        "source_extensions = [\"json\"]",
        "output_kind = \"Data\"",
        "required_capabilities = [\"runtime.asset.importer.native_dynamic_fixture.data_json\"]",
    ] {
        assert!(
            plugin_toml.contains(required),
            "native fixture plugin.toml should describe importer manifest anchor `{required}`"
        );
    }
    for required in [
        "runtime.asset.importer.native_dynamic_fixture.data_json",
        "[[extensions]]",
        "point = \"runtime.asset.importer.data\"",
        "contribution = \"plugin.native_dynamic_fixture.data_json\"",
        "schema = \"zircon.runtime.asset-importer.data/1\"",
        "command=asset.import/native_dynamic_fixture.data_json;payload=ZRIMP001",
        "\"asset.import/native_dynamic_fixture.data_json\" =>",
    ] {
        assert!(
            fixture.contains(required),
            "native fixture runtime registration/command surface should contain `{required}`"
        );
    }

    for stale_gap in [
        "plugin.toml` 无 `[[asset_importers]]`",
        "registration manifest `extensions` 也未声明 `runtime.importer` 贡献",
        "importer 能力未进可发现清单",
    ] {
        assert!(
            !review_findings.contains(stale_gap),
            "D13 native fixture importer review text should not keep stale gap `{stale_gap}`"
        );
    }
    for doc_anchor in [
        "D13 native_dynamic_fixture importer self-description",
        "native_dynamic_fixture_importer_manifest_self_description_static_passed_cargo_deferred",
        "review_d13_native_fixture_importer_is_manifest_described",
        "runtime.asset.importer.native_dynamic_fixture.data_json",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || package_doc.contains(doc_anchor)
                || native_doc.contains(doc_anchor),
            "D13 native fixture importer docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_priority_recommendation_tracks_current_remaining_work() {
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let priority_section = review_findings
        .split("## 5. 优先级建议")
        .nth(1)
        .and_then(|tail| tail.split("## 6. 审查和修复记录").next())
        .expect("review findings should keep priority recommendation section");

    for required in [
        "当前最高优先剩余项",
        "F1/F2/F4 已有闭合守卫",
        "review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi",
        "review_f2_scene_eventbus_locks_recover_after_poison",
        "review_f4_render_submit_capability_gaps_return_typed_errors",
        "F3 仍需 Runtime 07 full FPS/profiling/full gate",
        "render_camera_loop_source_payload_slot_owned_static_passed_cargo_deferred",
        "D-S7 静态插件 manifest 生成/parity 已由 `ds7_static_plugin_manifest_generation_parity_review_synced_static_passed_cargo_deferred` 闭合",
        "36/37 generated manifest",
        "plugins_12_static_plugin_manifest_is_generated",
        "plugins_12_manifest_schema_uniform_audit_report_is_clean",
        "plugins_12_feature_enabled_runtime_descriptor_manifest_parity",
        "native_dynamic_fixture importer 自描述已闭合",
        "D-S8/D3 native fixture SDK macro/manifest 单源已闭合",
        "D8 runtime registration builder 原始证据路径已由 `d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred` 闭合",
        "D13 runtime export macro/selection helper 已由 `d13_importer_runtime_export_macro_convergence_static_passed_cargo_deferred` 闭合",
        "D13 importer runtime manifest builder 已由 `d13_importer_runtime_manifest_builder_convergence_static_passed_cargo_deferred` 闭合",
        "D13 importer manifest parity guard 已由 `d13_importer_manifest_parity_guard_static_passed_cargo_deferred` 闭合",
        "review_d13_importer_manifest_parity_guard_lives_in_sdk_builder",
        "importer_runtime_manifest_builder_keeps_targets_platforms_modules_and_distribution_in_parity",
        "review_priority_recommendation_tracks_current_remaining_work",
    ] {
        assert!(
            priority_section.contains(required),
            "priority recommendation should record current remaining-work anchor `{required}`"
        );
    }
    for stale_priority in [
        "最高优先两项是安全/崩溃类：**F1",
        "D-S8（native 零 SDK）",
        "D13 importer targets/platforms/module/dist-module builder/parity 系列收编",
        "D13 importer manifest parity guard 仍需",
        "D8 `builder.module(desc).system(sys).register(registry)` 封装三步顺序",
        "DX 侧剩余高 ROI 是 **D-S7 静态插件 manifest 生成/parity**",
        "后续 DX 高 ROI 回到 D-S7 静态插件 manifest 生成/parity",
        "应在 Plugins 12 M1/M2 优先落地",
    ] {
        assert!(
            !priority_section.contains(stale_priority),
            "priority recommendation should not keep stale unclosed wording `{stale_priority}`"
        );
    }

    let ds7_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D-S7 |"))
        .expect("review findings should keep D-S7 top-table row");
    for required in [
        "ds7_static_plugin_manifest_generation_parity_review_synced_static_passed_cargo_deferred",
        "plugins_12_static_manifest_generated_marker_static_passed_cargo_timeout",
        "plugins_12_missing_importer_manifests_guard_passed",
        "plugins_12_manifest_schema_uniform_supported_platforms_guard_passed",
        "plugins_12_manifest_structure_audit_guard_passed",
        "plugins_12_feature_descriptor_parity_guard_passed",
        "plugins_12_static_plugin_manifest_is_generated",
        "plugins_12_manifest_schema_uniform_audit_report_is_clean",
        "plugins_12_feature_enabled_runtime_descriptor_manifest_parity",
        "expected_manifest_count = 37",
        "manifest_count = 37",
        "generated_manifest_count = 36",
        "hand_written_native_manifest_count = 1",
        "manifest_schema_violations = 0",
        "generated_manifest_header_violations = 0",
        "m1_gate_status = classified-and-clear",
    ] {
        assert!(
            ds7_row.contains(required),
            "D-S7 top-table row should record generated/parity closure anchor `{required}`"
        );
    }
    assert!(
        review_findings.contains("D-S7 static plugin manifest generation/parity review sync")
            && review_findings.contains(
                "ds7_static_plugin_manifest_generation_parity_review_synced_static_passed_cargo_deferred"
            ),
        "review findings should record the D-S7 generation/parity sync completion row"
    );
    let d7_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D7 |"))
        .expect("review findings should keep D7 top-table row");
    for required in [
        "d7_core_workspace_dependency_inheritance_guard_static_passed_cargo_deferred",
        "d7_core_workspace_dependency_top_row_closed_status_static_passed_cargo_deferred",
        "zircon_plugins/Cargo.lock",
        "core_workspace_dependency_status = core-workspace-deps-clean",
        "core_workspace_dependency_count = 117",
        "core_workspace_dependency_violation_count = 0",
        "cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --offline",
        "`zircon_runtime = { workspace = true }`",
        "`zircon_editor = { workspace = true }`",
        "`zircon_runtime_interface = { workspace = true }`",
        "插件间 path 依赖仍归后续切片",
    ] {
        assert!(
            d7_row.contains(required),
            "D7 top-table row should record core workspace dependency anchor `{required}`"
        );
    }
    assert!(
        !d7_row.contains("zircon_plugins/Cargo.toml` 无 `[workspace.dependencies]"),
        "D7 top-table row should not keep stale no-workspace-dependencies wording"
    );
    assert!(
        d7_row.ends_with("| M2 / closed |"),
        "D7 top-table row should mark core workspace dependency inheritance closed"
    );
    assert!(
        review_findings.contains("D7 core workspace dependency inheritance guard")
            && review_findings.contains(
                "d7_core_workspace_dependency_inheritance_guard_static_passed_cargo_deferred"
            ),
        "review findings should record the D7 core workspace dependency completion row"
    );
    assert!(
        review_findings.contains("D7 core workspace dependency top-row closed status sync")
            && review_findings.contains(
                "d7_core_workspace_dependency_top_row_closed_status_static_passed_cargo_deferred"
            ),
        "review findings should record the D7 top-row closed status sync completion row"
    );
    let d8_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D8 |"))
        .expect("review findings should keep D8 top-table row");
    for required in [
        "d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred",
        "animation/physics/net 原始证据路径已收敛",
        "RuntimePluginRegistrationBuilder",
        "RuntimePluginModuleRegistration::event",
        "review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
        "runtime_registration_builder_violation_count = 0",
        "m3_t2_runtime_registration_builder_status = runtime-registration-builder-clean",
    ] {
        assert!(
            d8_row.contains(required),
            "D8 top-table row should record runtime registration builder anchor `{required}`"
        );
    }
    assert!(
        !d8_row.contains("三步样板每插件手抄"),
        "D8 top-table row should not keep stale hand-written registration wording"
    );
    assert!(
        review_findings.contains("D8 runtime registration builder original evidence paths")
            && review_findings.contains(
                "d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred"
            ),
        "review findings should record the D8 registration builder completion row"
    );
    assert!(
        review_findings.contains("P0/DX priority recommendation current remaining-work sync")
            && review_findings
                .contains("review_priority_recommendation_tracks_current_remaining_work"),
        "review findings should record the priority recommendation sync completion row"
    );
    assert!(
        review_findings.contains("P0/DX priority recommendation D13 manifest parity sync")
            && review_findings.contains(
                "review_priority_recommendation_d13_parity_sync_static_passed_cargo_deferred"
            ),
        "review findings should record the D13 parity priority recommendation sync row"
    );
}

#[test]
fn review_f2_scene_eventbus_locks_recover_after_poison() {
    let level_system = include_str!("../../../scene/level_system.rs");
    let default_level_manager = include_str!("../../../scene/module/default_level_manager.rs");
    let level_manager_lifecycle = include_str!("../../../scene/module/level_manager_lifecycle.rs");
    let event_bus = include_str!("../../../core/runtime/events.rs");
    let event_publish = include_str!("../../../core/runtime/events/publish.rs");
    let event_subscribe = include_str!("../../../core/runtime/events/subscribe.rs");
    let event_prune = include_str!("../../../core/runtime/events/prune.rs");
    let level_doc = include_str!("../../../../../docs/zircon_runtime/scene/level_system.md");
    let event_doc = include_str!("../../../../../docs/zircon_runtime/core/runtime/events.md");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let module_doc =
        include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
    let status_rows = include_str!(
        "../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs"
    );
    let status_map = include_str!(
        "../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
    );

    for required in [
        "fn lock_poison_recovered<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T>",
        ".unwrap_or_else(|poisoned| poisoned.into_inner())",
        "fn lock_world(&self) -> MutexGuard<'_, World>",
        "fn lock_runtime_state(&self) -> MutexGuard<'_, WorldRuntimeState>",
        "fn lock_metadata(&self) -> MutexGuard<'_, LevelMetadata>",
        "fn lock_lifecycle(&self) -> MutexGuard<'_, LevelLifecycleState>",
        "fn lock_subsystems(&self) -> MutexGuard<'_, Vec<String>>",
        "level_system_accessors_recover_poisoned_state_locks",
    ] {
        assert!(
            level_system.contains(required),
            "LevelSystem should retain poison-safe lock recovery anchor `{required}`"
        );
    }

    for required in [
        "pub(super) fn lock_levels(&self) -> MutexGuard<'_, HashMap<WorldHandle, LevelSystem>>",
        ".unwrap_or_else(|poisoned| poisoned.into_inner())",
    ] {
        assert!(
            default_level_manager.contains(required),
            "DefaultLevelManager should retain poison-safe lock recovery anchor `{required}`"
        );
    }
    for required in [
        "self.lock_levels().insert(handle, level.clone())",
        "self.lock_levels().get(&handle).cloned()",
    ] {
        assert!(
            level_manager_lifecycle.contains(required),
            "level manager lifecycle should delegate level-map access through `{required}`"
        );
    }

    for required in [
        "fn lock_subscribers(&self) -> MutexGuard<'_, EventSubscriberMap>",
        "fn lock_delivery(&self) -> MutexGuard<'_, ()>",
        ".unwrap_or_else(|poisoned| poisoned.into_inner())",
    ] {
        assert!(
            event_bus.contains(required),
            "EventBus should retain poison-safe lock recovery anchor `{required}`"
        );
    }
    for required in [
        "self.lock_delivery()",
        "self.prune_topic_subscribers(",
        "let mut subscribers = self.lock_subscribers();",
    ] {
        assert!(
            event_publish.contains(required)
                || event_subscribe.contains(required)
                || event_prune.contains(required),
            "EventBus publish/subscribe/prune owners should keep helper usage `{required}`"
        );
    }

    for (label, source) in [
        ("level system", level_system),
        ("default level manager", default_level_manager),
        ("level manager lifecycle", level_manager_lifecycle),
        ("event bus root", event_bus),
        ("event publish", event_publish),
        ("event subscribe", event_subscribe),
        ("event prune", event_prune),
    ] {
        let production = source.split("\n#[cfg(test)]").next().unwrap_or(source);
        assert!(
            !production.contains(".lock().unwrap()"),
            "{label} production code should recover poisoned locks instead of direct lock unwrap"
        );
    }

    for doc_anchor in [
        "Runtime 15 M3 F2 lock poison recovery guard",
        "runtime_15_f2_lock_poison_recovery_guard_core_min_cargo_passed_full_sweep_pending",
        "review_f2_scene_eventbus_locks_recover_after_poison",
        "p0_f1_f2_f4_top_row_closed_status_static_passed_cargo_deferred",
        "runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus",
        "level_system_accessors_recover_poisoned_state_locks",
        "scene/EventBus poison-safe lock recovery complete",
    ] {
        assert!(
            level_doc.contains(doc_anchor)
                || event_doc.contains(doc_anchor)
                || review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || module_doc.contains(doc_anchor)
                || status_rows.contains(doc_anchor)
                || status_map.contains(doc_anchor)
                || date_map.contains(doc_anchor),
            "F2 scene/EventBus lock poison docs/status should record `{doc_anchor}`"
        );
    }
    let f2_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F2 |"))
        .expect("F2 row should exist");
    assert!(
        f2_row.ends_with("| Runtime 15 + Runtime 07 / review closed |"),
        "F2 row should mark the lock-poison recovery review state closed"
    );
}

#[test]
fn review_f4_render_submit_capability_gaps_return_typed_errors() {
    let viewport_guard = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/viewport_generation_guard.rs"
    );
    let prepare_runtime_submission = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs"
    );
    let submit_frame_extract = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs"
    );
    let submit_runtime_frame = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs"
    );
    let present_frame_extract = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs"
    );
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_07_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let render_index = include_str!("../../../../../docs/plans/zircon_runtime/render/index.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let advanced_doc =
        include_str!("../../../../../docs/zircon_runtime/core/framework/render/advanced.md");
    let status_rows = include_str!(
        "../plan_status/status_output_tables/expected_status_row_data/runtime_06_09/runtime_07/performance.rs"
    );
    let status_map = include_str!(
        "../plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_06_10.rs"
    );
    let date_map = include_str!(
        "../plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_06_10.rs"
    );

    for required in [
        "pub(super) fn validate_viewport_generation",
        "RenderFrameworkError::UnknownViewport",
        "RenderFrameworkError::ViewportChanged",
        "pub(super) fn viewport_record_mut_after_generation_check",
        "validate_viewport_generation(state, viewport, context)?",
    ] {
        assert!(
            viewport_guard.contains(required),
            "viewport generation guard should keep typed error anchor `{required}`"
        );
    }

    for required in [
        "return Err(missing_runtime_provider(\"hybrid global illumination\"));",
        "return Err(missing_runtime_provider(\"virtual geometry\"));",
        "RenderFrameworkError::UnsupportedCapability",
        "capability: format!(\"{feature} runtime provider\")",
        "record.clear_hybrid_gi_runtimes();",
        "record.clear_virtual_geometry_runtimes();",
    ] {
        assert!(
            prepare_runtime_submission.contains(required),
            "prepare runtime submission should keep provider-missing typed error anchor `{required}`"
        );
    }

    for (label, source) in [
        ("submit generated frame", submit_frame_extract),
        ("submit direct runtime frame", submit_runtime_frame),
        ("present generated frame", present_frame_extract),
        ("prepare runtime submission", prepare_runtime_submission),
    ] {
        let production = production_source(source);
        assert!(
            !production.contains(".unwrap("),
            "{label} production path should not panic through unwrap"
        );
        assert!(
            !production.contains(".expect("),
            "{label} production path should not panic through expect"
        );
    }

    for (label, source) in [
        ("submit generated frame", submit_frame_extract),
        ("submit direct runtime frame", submit_runtime_frame),
        ("present generated frame", present_frame_extract),
    ] {
        let production = production_source(source);
        assert!(
            production.contains("validate_viewport_generation(&state, viewport, &context)"),
            "{label} should validate viewport generation before record writeback"
        );
        assert!(
            production.contains(
                "viewport_record_mut_after_generation_check(&mut state, viewport, &context)?"
            ),
            "{label} should fetch viewport records through the checked helper"
        );
    }

    for doc_anchor in [
        "Runtime 07 render submit viewport/provider errors",
        "render_submit_viewport_provider_errors_review_guard_static_passed_cargo_timeout_no_result_full_runtime07_pending",
        "review_f4_render_submit_capability_gaps_return_typed_errors",
        "p0_f1_f2_f4_top_row_closed_status_static_passed_cargo_deferred",
        "RenderFrameworkError::UnsupportedCapability",
        "viewport_record_mut_after_generation_check",
        "submit_frame_extract production paths must return RenderFrameworkError",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_07_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || render_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || advanced_doc.contains(doc_anchor)
                || status_rows.contains(doc_anchor)
                || status_map.contains(doc_anchor)
                || date_map.contains(doc_anchor),
            "F4 render submit typed-error docs/status should record `{doc_anchor}`"
        );
    }
    let f4_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F4 |"))
        .expect("F4 row should exist");
    assert!(
        f4_row.ends_with("| Runtime 07 + render index / review closed |"),
        "F4 row should mark the typed-error review state closed while full Runtime 07 gate remains separate"
    );
}

fn production_source(source: &str) -> &str {
    source.split("\n#[cfg(test)]").next().unwrap_or(source)
}
