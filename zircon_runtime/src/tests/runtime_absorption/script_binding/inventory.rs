pub(super) const SCRIPT_LEDGER_TEST_MAX_LINES: usize = 700;
pub(super) const GAMEPLAY_TEST_MAX_LINES: usize = 1000;
pub(super) const GAMEPLAY_HOST_OWNER_MAX_LINES: usize = 400;

pub(super) const EXPECTED_RUNTIME_13_SOURCE_FILES: &[&str] = &[
    "src/script/vm/host/builtin_host_modules.rs",
    "src/script/vm/gameplay_host.rs",
    "src/script/vm/gameplay_host/combat.rs",
    "src/script/vm/gameplay_host/components.rs",
    "src/script/vm/gameplay_host/error.rs",
    "src/script/vm/gameplay_host/input.rs",
    "src/script/vm/gameplay_host/lifecycle.rs",
    "src/script/vm/gameplay_host/navigation.rs",
    "src/script/vm/gameplay_host/scene_transition.rs",
    "src/script/vm/gameplay_host/script_bindings.rs",
    "src/script/vm/gameplay_host/transform.rs",
    "src/script/vm/gameplay_host/values.rs",
    "src/script/vm/host/bridge_host_module.rs",
    "src/script/vm/host/host_export_registry.rs",
    "src/script/vm/host/script_call_table.rs",
    "src/core/framework/script.rs",
    "src/core/framework/script/argument_views.rs",
    "src/core/framework/script/argument_views/argument_source.rs",
    "src/core/framework/script/argument_views/byte_view.rs",
    "src/core/framework/script/argument_views/typed_conversion.rs",
    "src/core/framework/script/argument_views/value_ref.rs",
    "src/core/framework/script/call_frame.rs",
    "src/core/framework/script/descriptors.rs",
    "src/core/framework/script/hot_path_metrics.rs",
    "src/core/framework/script/value_contracts.rs",
    "src/script/vm/capability_set.rs",
    "src/script/vm/handles.rs",
    "src/script/vm/runtime_context.rs",
];

pub(super) const EXPECTED_RUNTIME_13_TEST_FILES: &[&str] = &[
    "src/tests/runtime_absorption/script_host_ledger.rs",
    "src/tests/runtime_absorption/script_binding.rs",
    "src/script/vm/gameplay_host/tests.rs",
];

pub(super) const GAMEPLAY_HOST_OWNER_FILES: &[&str] = &[
    "src/script/vm/gameplay_host.rs",
    "src/script/vm/gameplay_host/combat.rs",
    "src/script/vm/gameplay_host/components.rs",
    "src/script/vm/gameplay_host/error.rs",
    "src/script/vm/gameplay_host/input.rs",
    "src/script/vm/gameplay_host/lifecycle.rs",
    "src/script/vm/gameplay_host/navigation.rs",
    "src/script/vm/gameplay_host/scene_transition.rs",
    "src/script/vm/gameplay_host/script_bindings.rs",
    "src/script/vm/gameplay_host/transform.rs",
    "src/script/vm/gameplay_host/values.rs",
];

pub(super) const RUNTIME_13_GUARD_ANCHORS: &[&str] = &[
    "host_function_registry_matches_documented_ledger",
    "host_function_registry_ledger_guard_rejects_missing_entry",
    "host_capability_representatives_are_declared_on_registered_modules",
    "host_function_without_required_capability_is_rejected_with_explicit_error",
    "script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi",
    "script_held_entity_handle_reports_invalid_after_despawn",
    "runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass",
    "runtime_13_script_binding_mirror_docs_match_structure_audit_counts",
    "runtime_13_gameplay_host_owner_split_keeps_domain_files",
];

pub(super) const SCRIPT_BINDING_MIRROR_DOC_ANCHORS: &[&str] = &[
    "script_binding_boundary",
    "expected_source_file_count = 28",
    "expected_test_file_count = 3",
    "expected_guard_file_count = 8",
    "missing_guard_files = []",
    "fixed_host_module_count = 6",
    "fixed_host_function_count = 61",
    "type_descriptor_count = 2",
    "builtin_callback_count = 20",
    "gameplay_callback_count = 40",
    "macro_host_function_count = 2",
    "host_capability_count = 13",
    "guard_anchor_count = 9",
    "native_ecs_abi_references = []",
    "oversized_test_files = []",
    "mirror_docs_guard_present = true",
    "risks = []",
    "runtime_13_script_binding_mirror_docs_match_structure_audit_counts",
];

pub(super) const GAMEPLAY_HOST_MODULE_ANCHORS: &[&str] = &[
    "mod combat;",
    "mod components;",
    "mod error;",
    "mod input;",
    "mod lifecycle;",
    "mod navigation;",
    "mod scene_transition;",
    "mod script_bindings;",
    "mod transform;",
    "mod values;",
];

pub(super) const GAMEPLAY_HOST_REGISTRATION_ANCHORS: &[&str] = &[
    "HostExportFunction::new(\"key_pressed\"",
    "HostExportFunction::new(\"request_scene_transition\"",
    "HostExportFunction::new(\"position_json\"",
    "HostExportFunction::new(\"component_json\"",
    "HostExportFunction::new(\"damage_entity\"",
    "HostExportFunction::new(\"spawn_empty\"",
    "HostExportFunction::new(\"nav_next_point_json\"",
];

pub(super) const GAMEPLAY_VALUE_ANCHORS: &[&str] = &[
    "expect_vec3_json",
    "resource_handle_from_script_ref",
    "script_core_error",
];
