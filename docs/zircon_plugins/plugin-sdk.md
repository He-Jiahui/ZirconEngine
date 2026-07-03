---
related_code:
  - zircon_plugins/plugin_sdk/Cargo.toml
  - zircon_plugins/plugin_sdk/src/lib.rs
  - zircon_plugins/plugin_sdk/src/test.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/runtime_helpers.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/target_resolution.rs
  - zircon_runtime/src/core/framework/physics/query_interface.rs
  - zircon_plugins/physics/runtime/src/plugin.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs
  - zircon_plugins/plugin_sdk/src/editor.rs
  - zircon_plugins/plugin_sdk/src/dist.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/prelude.rs
  - zircon_plugins/plugin_sdk/src/runtime_exports.rs
  - zircon_plugins/plugin_sdk/src/test.rs
  - zircon_plugins/plugin_sdk/src/manifest/defaults.rs
  - zircon_plugins/plugin_sdk/src/manifest/feature_bundle_builder.rs
  - zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs
  - zircon_plugins/plugin_sdk/src/manifest/package_builder.rs
  - zircon_plugins/plugin_sdk/src/manifest/plugin_module_builder.rs
  - zircon_plugins/plugin_sdk/src/manifest/tests.rs
  - zircon_plugins/plugin_sdk/src/runtime.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/Cargo.toml
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/Cargo.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/Cargo.toml
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/net/runtime/Cargo.toml
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/Cargo.toml
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/prefab_tools/runtime/Cargo.toml
  - zircon_plugins/prefab_tools/runtime/src/lib.rs
  - zircon_plugins/rendering/runtime/Cargo.toml
  - zircon_plugins/rendering/runtime/src/lib.rs
  - zircon_plugins/solari/runtime/Cargo.toml
  - zircon_plugins/solari/runtime/src/lib.rs
  - zircon_plugins/terrain/runtime/Cargo.toml
  - zircon_plugins/terrain/runtime/src/lib.rs
  - zircon_plugins/texture/runtime/Cargo.toml
  - zircon_plugins/texture/runtime/src/lib.rs
  - zircon_plugins/tilemap_2d/runtime/Cargo.toml
  - zircon_plugins/tilemap_2d/runtime/src/lib.rs
  - zircon_plugins/virtual_geometry/runtime/Cargo.toml
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/Cargo.toml
  - zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/tests.rs
  - zircon_plugins/animation/editor/Cargo.toml
  - zircon_plugins/animation/editor/src/plugin.rs
  - zircon_plugins/animation/editor/src/tests.rs
  - zircon_plugins/physics/editor/Cargo.toml
  - zircon_plugins/physics/editor/src/plugin.rs
  - zircon_plugins/physics/editor/src/tests.rs
  - zircon_plugins/net/editor/Cargo.toml
  - zircon_plugins/net/editor/src/plugin.rs
  - zircon_plugins/net/editor/src/tests/authoring_extensions.rs
  - zircon_plugins/native_dynamic_fixture/plugin.toml
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_plugins/runtime_diagnostics/plugin.toml
  - zircon_plugins/runtime_diagnostics/dist/Cargo.toml
  - zircon_plugins/runtime_diagnostics/dist/src/lib.rs
  - zircon_plugins/runtime_diagnostics/editor/src/plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation/report.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation/schema.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_plugins/Cargo.toml
  - tools/plugin_structure_audits/manifest_schema.py
  - tools/plugin_structure_audits/manifest_schema_modules.py
  - tools/tests/test_plugin_structure_audit_manifest_schema.py
  - tools/plugin_structure_audits/skeleton.py
implementation_files:
  - zircon_plugins/plugin_sdk/Cargo.toml
  - zircon_plugins/plugin_sdk/src/lib.rs
  - zircon_plugins/plugin_sdk/src/editor.rs
  - zircon_plugins/plugin_sdk/src/dist.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/prelude.rs
  - zircon_plugins/plugin_sdk/src/runtime_exports.rs
  - zircon_plugins/plugin_sdk/src/test.rs
  - zircon_plugins/plugin_sdk/src/manifest/defaults.rs
  - zircon_plugins/plugin_sdk/src/manifest/feature_bundle_builder.rs
  - zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs
  - zircon_plugins/plugin_sdk/src/manifest/package_builder.rs
  - zircon_plugins/plugin_sdk/src/manifest/plugin_module_builder.rs
  - zircon_plugins/plugin_sdk/src/manifest/tests.rs
  - zircon_plugins/plugin_sdk/src/runtime.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/Cargo.toml
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/Cargo.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/Cargo.toml
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/net/runtime/Cargo.toml
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/Cargo.toml
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/prefab_tools/runtime/Cargo.toml
  - zircon_plugins/prefab_tools/runtime/src/lib.rs
  - zircon_plugins/rendering/runtime/Cargo.toml
  - zircon_plugins/rendering/runtime/src/lib.rs
  - zircon_plugins/solari/runtime/Cargo.toml
  - zircon_plugins/solari/runtime/src/lib.rs
  - zircon_plugins/terrain/runtime/Cargo.toml
  - zircon_plugins/terrain/runtime/src/lib.rs
  - zircon_plugins/texture/runtime/Cargo.toml
  - zircon_plugins/texture/runtime/src/lib.rs
  - zircon_plugins/tilemap_2d/runtime/Cargo.toml
  - zircon_plugins/tilemap_2d/runtime/src/lib.rs
  - zircon_plugins/virtual_geometry/runtime/Cargo.toml
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/Cargo.toml
  - zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/tests.rs
  - zircon_plugins/animation/editor/Cargo.toml
  - zircon_plugins/animation/editor/src/plugin.rs
  - zircon_plugins/animation/editor/src/tests.rs
  - zircon_plugins/physics/editor/Cargo.toml
  - zircon_plugins/physics/editor/src/plugin.rs
  - zircon_plugins/physics/editor/src/tests.rs
  - zircon_plugins/net/editor/Cargo.toml
  - zircon_plugins/net/editor/src/plugin.rs
  - zircon_plugins/net/editor/src/tests/authoring_extensions.rs
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_plugins/runtime_diagnostics/plugin.toml
  - zircon_plugins/runtime_diagnostics/dist/Cargo.toml
  - zircon_plugins/runtime_diagnostics/dist/src/lib.rs
  - zircon_plugins/runtime_diagnostics/editor/src/plugin.rs
  - zircon_plugins/Cargo.toml
  - tools/plugin_structure_audits/manifest_schema.py
  - tools/plugin_structure_audits/manifest_schema_modules.py
  - tools/tests/test_plugin_structure_audit_manifest_schema.py
  - tools/plugin_structure_audits/skeleton.py
plan_sources:
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - rustfmt --edition 2021 --config skip_children=true --check zircon_plugins/plugin_sdk/src/lib.rs zircon_plugins/plugin_sdk/src/prelude.rs zircon_plugins/plugin_sdk/src/runtime.rs zircon_plugins/plugin_sdk/src/manifest/defaults.rs zircon_plugins/plugin_sdk/src/manifest/mod.rs zircon_plugins/plugin_sdk/src/manifest/package_builder.rs zircon_plugins/plugin_sdk/src/manifest/plugin_module_builder.rs zircon_plugins/plugin_sdk/src/manifest/tests.rs: passed 2026-06-22
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-sdk-m2-0622 --message-format short --color never: passed 2026-06-22 with existing zircon_runtime warnings
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-sdk-m2-0622 --message-format short --color never -- --test-threads=1: 3 passed, 0 failed on 2026-06-22 with existing zircon_runtime warnings
  - python tools/audit_plugin_structure.py --json: passed 2026-06-22 with m1_gate_status classified-and-clear, missing_plugin_toml 0, manifest_schema_violations 0, expected_manifest_count 37
  - python tools/audit_plugin_structure.py --json: passed 2026-06-22 with skeleton sample_conformance_status sample-clean, sample_expected_count 1, sample_violation_count 0
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor -p zircon_first_party_runtime_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never: passed 2026-06-22 with existing warning noise
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never -- --test-threads=1: timed out after 1200s on 2026-06-22, not counted as passing
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --no-default-features --features native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-native-sdk-m2-0622 --message-format short --color never: passed 2026-06-22
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --no-default-features --features native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-native-sdk-m2-0622 --message-format short --color never -- --test-threads=1: 4 passed, 0 failed on 2026-06-22
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-native-fixture-sdk-m2-0622 --message-format short --color never: passed 2026-06-22
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-sdk-default-m2-0622 --message-format short --color never: passed 2026-06-22 with existing zircon_runtime warnings
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --features editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-editor-sdk-m2-0622 --message-format short --color never: passed 2026-06-22 with existing zircon_runtime/zircon_editor warnings
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-editor-sdk-m2-0622 --message-format short --color never: passed 2026-06-22 with existing zircon_runtime/zircon_editor warnings
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --features editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-editor-sdk-m2-0622 --message-format short --color never authoring_plugin_macro_generates_descriptor_manifest_and_registration -- --test-threads=1 --nocapture: timed out after 300s on 2026-06-22, no test binary, not counted as passing
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-test-runtime-m2-0622 --message-format short --color never: passed 2026-06-22 with existing zircon_runtime warnings
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-test-runtime-m2-0622 --message-format short --color never test_runtime_builder -- --test-threads=1 --nocapture: 2 passed, 0 failed on 2026-06-22 after an earlier 600s compile timeout that was not counted as passing
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk -p zircon_plugin_animation_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-registration-m3-0622 --message-format short --color never: passed 2026-06-22 with existing zircon_runtime warnings after an offline lock refresh for the new animation SDK dependency
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-registration-m3-0622 --message-format short --color never runtime_registration_builder -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-22
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-registration-m3-0622 --message-format short --color never animation_registration_contributes_runtime_module -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-22
  - rustfmt --edition 2021 --check zircon_plugins/plugin_sdk/src/lib.rs zircon_plugins/plugin_sdk/src/runtime_exports.rs zircon_plugins/animation/runtime/src/lib.rs zircon_plugins/physics/runtime/src/lib.rs zircon_plugins/net/runtime/src/lib.rs: passed 2026-06-23
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk -p zircon_plugin_animation_runtime -p zircon_plugin_physics_runtime -p zircon_plugin_net_runtime --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-d12-export-macro-0623 --message-format short --color never: passed 2026-06-23 and refreshed zircon_plugins/Cargo.lock for physics/net SDK dependencies
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk -p zircon_plugin_animation_runtime -p zircon_plugin_physics_runtime -p zircon_plugin_net_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-d12-export-macro-0623 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime/net/physics warnings
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-d12-export-macro-0623 --message-format short --color never runtime_plugin_exports -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-23 with existing zircon_runtime warnings
  - rustfmt --edition 2021 --check zircon_plugins/plugin_sdk/src/lib.rs zircon_plugins/plugin_sdk/src/runtime_exports.rs zircon_plugins/ai/runtime/src/lib.rs zircon_plugins/animation/runtime/src/lib.rs zircon_plugins/hybrid_gi/runtime/src/lib.rs zircon_plugins/navigation/runtime/src/lib.rs zircon_plugins/net/runtime/src/lib.rs zircon_plugins/particles/runtime/src/lib.rs zircon_plugins/physics/runtime/src/lib.rs zircon_plugins/prefab_tools/runtime/src/lib.rs zircon_plugins/rendering/runtime/src/lib.rs zircon_plugins/solari/runtime/src/lib.rs zircon_plugins/terrain/runtime/src/lib.rs zircon_plugins/texture/runtime/src/lib.rs zircon_plugins/tilemap_2d/runtime/src/lib.rs zircon_plugins/virtual_geometry/runtime/src/lib.rs zircon_plugins/zr_vm_language/runtime/src/lib.rs: passed 2026-06-23
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk -p zircon_plugin_ai_runtime -p zircon_plugin_animation_runtime -p zircon_plugin_hybrid_gi_runtime -p zircon_plugin_navigation_runtime -p zircon_plugin_net_runtime -p zircon_plugin_particles_runtime -p zircon_plugin_physics_runtime -p zircon_plugin_prefab_tools_runtime -p zircon_plugin_rendering_runtime -p zircon_plugin_solari_runtime -p zircon_plugin_terrain_runtime -p zircon_plugin_texture_runtime -p zircon_plugin_tilemap_2d_runtime -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_zr_vm_language_runtime --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-d12-export-macro-0623 --message-format short --color never: passed 2026-06-23 and refreshed zircon_plugins/Cargo.lock for remaining first-party runtime SDK dependencies
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk -p zircon_plugin_ai_runtime -p zircon_plugin_animation_runtime -p zircon_plugin_hybrid_gi_runtime -p zircon_plugin_navigation_runtime -p zircon_plugin_net_runtime -p zircon_plugin_particles_runtime -p zircon_plugin_physics_runtime -p zircon_plugin_prefab_tools_runtime -p zircon_plugin_rendering_runtime -p zircon_plugin_solari_runtime -p zircon_plugin_terrain_runtime -p zircon_plugin_texture_runtime -p zircon_plugin_tilemap_2d_runtime -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_zr_vm_language_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-d12-export-macro-0623 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime and large-plugin warning noise
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --format-version 1 --no-deps --locked: passed 2026-06-23
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/dispatch/mod.rs zircon_plugins/first_party_runtime_catalog/src/lib.rs zircon_plugins/plugin_sdk/src/editor.rs zircon_plugins/plugin_sdk/src/lib.rs zircon_plugins/plugin_sdk/src/prelude.rs zircon_plugins/plugin_sdk/src/manifest/mod.rs zircon_plugins/plugin_sdk/src/manifest/tests.rs zircon_plugins/plugin_sdk/src/manifest/feature_bundle_builder.rs: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: capability_conformance.m4_t2_builder_mirror_gate_status=sdk-builder-mirror-clean, sdk_builder_mirror_violations=0 on 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m4-capability-test-nodebug-0623 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime warnings
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m4-capability-test-nodebug-0623 --message-format short --color never feature_bundle_builder_projects_capability_to_feature_and_modules -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --features editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-editor-sdk-m2-0622 --message-format short --color never: passed 2026-06-23 after zircon_runtime::ui::dispatch façade exported route policy helpers
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --features editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-editor-sdk-m2-0622 --message-format short --color never editor_declaration_mirrors_runtime_manifest_and_keeps_editor_capabilities -- --test-threads=1 --nocapture: timed out after 900s on 2026-06-23, not counted as passing
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --no-default-features --features native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-standalone-m2-registration-0623 --message-format short --color never: passed 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --no-default-features --features native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-standalone-m2-registration-0623 --message-format short --color never native_dynamic_registration_manifest_round_trips -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --no-default-features --features native dist_plugin_one_file_export_compiles --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m2-t3-sdk: 1 passed, 0 failed on 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --no-default-features --features dist --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m2-t3-fixture: passed 2026-06-23
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sdk --no-default-features --features native dist_plugin_one_file_export_compiles --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-sdk-editor-dist-0625 --message-format short --color never -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-25 after adding editor-only dist helper export
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_runtime_diagnostics_dist --no-default-features --features dist --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-runtime-diagnostics-dist-0625 --message-format short --color never -- --test-threads=1 --nocapture: 2 passed, 0 failed on 2026-06-25, covering `native_dist_editor_plugin_v3!`
doc_type: module-detail
---

# Plugin SDK

`zircon_plugin_sdk` is the Plugins 12 M2 authoring support crate. It gives plugin authors stable builder surfaces for manifest/runtime descriptor declarations and native ABI v3 exports.

## Boundary

The SDK crate is a workspace support crate, not a plugin package root. It has no `plugin.toml` and is skipped by `plugin_structure_audits::manifest_schema` through `SKIPPED_WORKSPACE_ROOTS`.

Ownership stays split as follows:

- `zircon_runtime::plugin` owns `PluginPackageManifest`, `PluginModuleManifest`, `RuntimePluginDescriptor`, target modes, packaging strategies, and maturity/status enums.
- `zircon_plugin_sdk::manifest` owns author-facing builders for package and module manifest declarations.
- `zircon_plugin_sdk::manifest::ImporterRuntimeManifestBuilder` owns shared importer runtime targets/platforms/module/native distribution manifest projection, with `NATIVE_ABI_VERSION_V3` and `NATIVE_DESCRIPTOR_SYMBOL_V3` exported through SDK lib/prelude.
- `zircon_plugin_sdk::runtime` owns a thin declaration helper that projects one runtime descriptor into both `RuntimePluginDescriptor` and `PluginPackageManifest`.
- `zircon_plugin_sdk::registration` owns runtime registration builders that hide plugin module owner-token sequencing for module, option/catalog metadata, typed event, and runtime scene system registration.
- `zircon_plugin_sdk::runtime_plugin_exports!` owns the standard runtime export helper functions for trait-backed runtime plugins.
- `zircon_plugin_sdk::editor` owns editor authoring declaration helpers and the `authoring_plugin!` macro for editor plugin boilerplate.
- `zircon_plugin_sdk::native` owns the author-facing native ABI v3 declarations, callback helpers, SDK-owned byte buffers, entry capability checks, export macros, and registration manifest TOML DTOs. Its feature depends on `zircon_runtime_interface`, `serde`, and `toml`, not the full runtime crate.
- `zircon_plugin_sdk::dist` owns one-file native distribution macros that project crate-local manifest/callback declarations into ABI v3 descriptor, runtime/editor entries, bridge method tables, and symbol exports. It now exposes dual-entry, runtime-only, and editor-only helpers so dist crates can model their real host surface without dummy entries.
- `zircon_plugin_sdk::test` owns runtime test fixture construction for plugin integration tests. It consumes public runtime/catalog APIs and does not bypass `CoreRuntime` lifecycle registration.

## Native ABI

The `native` feature exports ABI v3 structures and helpers for dist plugins. Plugins 13 M2/T1 adds registration manifest support to this owner:

- `NativePluginSchemaVersionsV3::registration_manifest_schema` advertises the schema id for registration declarations.
- `NativePluginBehaviorV3::registration_manifest` carries the ABI-safe TOML text for module, system, resource, event, extension, and capability declarations.
- `NativePluginRegistrationManifestV3` plus `registration_manifest_v3_to_toml(...)`, `registration_manifest_v3_from_toml(...)`, and `registration_manifest_v3_schema_is_current(...)` provide the SDK round-trip surface.

The current schema id is `zircon.native.registration-manifest/3`. `native_dynamic_fixture` publishes a runtime registration manifest through this path; Plugins 13 M2/T2 added system bridge replay on the runtime host side, Plugins 13 M2/T3 moved the fixture's one-file cdylib exports to `zircon_plugin_sdk::dist`, and the 2026-06-25 runtime_diagnostics rollout added the editor-only projection path.

## Manifest Builders

`PluginManifestBuilder::new(id, display_name)` seeds the required Plugins 12 defaults:

- `sdk_api_version = "0.1.0"`
- `supported_platforms = windows/linux/macos`
- `default_packaging = SourceTemplate + LibraryEmbed`

The builder then accepts category, description, maturity, target modes, capabilities, asset/content roots, and explicit module declarations before returning the runtime-owned `PluginPackageManifest`.

`PluginModuleBuilder` standardizes module naming for `runtime`, `editor`, `native`, and `vm` modules. The editor module builder defaults to `RuntimeTargetMode::EditorHost`, while other module kinds stay explicit so future guard work can verify target-mode intent per plugin.

Frameworks 02 M3 extends the same builder to the runtime kernel descriptor vocabulary. `PluginModuleBuilder` now forwards `with_description(...)`, `with_init_level(...)`, `with_module_dependency(...)`, and `with_module_dependencies(...)` into `PluginModuleManifest`, and the generated manifest row projects through `PluginModuleManifest::module_descriptor()` when native package/feature reports replay it into the runtime registry. `InitLevel` and `ModuleDependencySpec` are re-exported from the SDK prelude so plugin authors do not need to import the runtime crate internals to declare descriptor order.

`PluginFeatureBundleBuilder` standardizes optional feature bundles. It declares feature dependencies, capabilities, runtime modules, editor modules, target modes, enabled-by-default status, and default packaging from one owner. The helper methods `with_runtime_capability_module(...)` and `with_editor_capability_module(...)` project the same feature capability into `PluginFeatureBundleManifest` and the generated `PluginModuleManifest`, so feature-level capability strings do not drift between manifest sections.

The 2026-06-28 D1 capability single-source review/status sync records that M4/T1 and M4/T2 are now mirrored into the Runtime 15 review guard. `review_d1_plugin_capabilities_use_single_source_and_sdk_builder_mirror` checks the 15 trait-backed first-party runtime roots, the `plugins_12_capability_single_source_conformance` catalog guard, `m4_runtime_capability_gate_status = runtime-capability-single-source-clean`, `capability_source_mismatches = 0`, `m4_t2_builder_mirror_gate_status = sdk-builder-mirror-clean`, and `sdk_builder_mirror_violations = 0`. Status anchor: `d1_capability_single_source_review_synced_static_passed_cargo_deferred`.

`ImporterRuntimeManifestBuilder` standardizes the D13 importer runtime manifest shape. The 2026-06-28 D13 importer runtime manifest builder convergence owns the shared `ClientRuntime + EditorHost` target modes, `windows/linux/macos` platforms, runtime module manifest, native dist module manifest, NativeDynamic distribution manifest, ABI v3 descriptor symbol/version, default engine compatibility, and asset-importer manifest projection. Importer crates still own their `RuntimePlugin` descriptor, importer descriptors, and registry mutation, but targets/platforms/module/dist-module distribution boilerplate now lives in the SDK. Guard `review_d13_importer_runtime_manifests_use_sdk_builder` and status `d13_importer_runtime_manifest_builder_convergence_static_passed_cargo_deferred` lock this D13 manifest builder convergence.

The 2026-06-28 D13 importer manifest parity guard adds `importer_runtime_manifest_builder_keeps_targets_platforms_modules_and_distribution_in_parity` so the SDK tests the actual manifest output, not just source text. The test checks shared targets/platforms, runtime/dist modules, NativeDynamic packaging, `NATIVE_ABI_VERSION_V3`, and `NATIVE_DESCRIPTOR_SYMBOL_V3`; guard `review_d13_importer_manifest_parity_guard_lives_in_sdk_builder` and status `d13_importer_manifest_parity_guard_static_passed_cargo_deferred` keep that parity owner from drifting.

The first consumer is `zircon_plugin_sdk_examples_editor`. Its `plugin.rs` now uses `authoring_plugin!` to build the base package manifest and generate the primary editor plugin implementation. The generated `EditorPlugin` implementation still lets `zircon_editor::EditorPlugin::package_manifest(...)` attach the editor module so module ownership stays with the editor plugin descriptor.

## Editor Authoring Macro

The `editor` feature adds `zircon_plugin_sdk::editor`:

- `EditorPluginDeclaration` keeps descriptor, base manifest, capability list, asset/content roots, maturity, and registration projection in one authoring object.
- `EditorPluginDeclaration::mirrors_runtime(...)` and `mirrors_runtime_manifest(...)` let an editor plugin explicitly mirror a runtime package manifest while preserving editor-specific capabilities and roots; `mirrored_runtime_package_id()` exposes the linked runtime id for guards and tests.
- `authoring_plugin!` generates the common editor plugin struct, `Default`/`new`, descriptor access, package manifest projection, capability extraction, registration report helper, and `EditorPlugin::register_editor_extensions` forwarding.
- `authoring_plugin!` accepts `mirrors_runtime: ...` and `mirrors_runtime_manifest: ...` for editor/runtime capability symmetry declarations.
- `zircon_plugins/Cargo.toml` now defines `[workspace.dependencies]` for core path dependencies. All member crates consume `zircon_runtime`, `zircon_editor`, and `zircon_runtime_interface` through `workspace = true`; `zircon_plugins/Cargo.lock` is synchronized with that offline resolution. Plugin-to-plugin path dependencies are tracked separately.
- `plugin_structure_audits::skeleton` checks the blessed sample for workspace dependency inheritance and reports `sample_workspace_dependency_status = sample-workspace-deps-clean`. It also reports the global core dependency guard as `core_workspace_dependency_status = core-workspace-deps-clean`, `core_workspace_dependency_count = 117`, and `core_workspace_dependency_violation_count = 0`.
- Status anchor: `d7_core_workspace_dependency_inheritance_guard_static_passed_cargo_deferred`; `cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --offline` passes, while catalog Cargo test execution remains deferred by compile-time window.

The 2026-06-28 D5 editor authoring macro consumer guard moves the first real consumers onto the SDK macro path. `animation`, `physics`, and `net` editor plugins use `zircon_plugin_sdk::authoring_plugin!` with `mirrors_runtime_manifest:` and keep only their plugin-specific extension registration bodies outside the macro. Status anchor: `d5_editor_authoring_macro_consumers_static_passed_cargo_deferred`; guard: `review_d5_editor_authoring_plugins_use_sdk_macro`.

The 2026-06-28 D9 editor/runtime mirror consumer guard keeps those macro consumers tied to runtime package manifests through the same `EditorPluginDeclaration::mirrors_runtime_manifest` projection. `animation`, `physics`, and `net` editor tests assert `mirrored_runtime_package_id()`, and `tools/audit_plugin_structure.py --json` reports `editor_runtime_mirror_violations = 0` plus `d9_editor_runtime_mirror_gate_status = editor-runtime-mirror-clean`. Status anchor: `d9_editor_runtime_mirror_consumers_static_passed_cargo_deferred`; guard: `review_d9_editor_runtime_mirror_consumers_use_sdk_declaration`.

## Native ABI Helper

The `native` feature is intentionally lightweight:

- `default = ["runtime"]` keeps existing runtime/manifest builders available for normal Rust plugin crates.
- `native = ["dep:zircon_runtime_interface"]` compiles without pulling `zircon_runtime`, so native cdylib fixtures are not blocked by runtime crate migration work.
- `NativePluginAbiV3`, `NativePluginEntryReportV3`, `NativePluginBehaviorV3`, host callback tables, status constants, and bridge DTOs are exported from `zircon_plugin_sdk::native`.
- `NativePluginStatic<T>` replaces per-plugin unsafe `Sync` wrappers around `repr(C)` static tables.
- `NativePluginEntryPointV3` centralizes required/denied capability selection and optional host-ready callbacks.
- `owned_bytes(...)`, `free_owned_bytes_v2(...)`, `bytes_from_slice(...)`, `callback_status(...)`, and `catch_native_callback_panic(...)` centralize owner-token memory and panic-boundary boilerplate.
- `export_native_plugin_descriptor_v3!` and `export_native_plugin_entry_v3!` export ABI symbols without hand-writing `#[no_mangle]` functions. `native_command_plugin_v3!` is the simple stateless command-plugin macro for future one-file native authors.
- `zircon_plugin_sdk::dist::{native_dist_plugin_v3!, native_dist_runtime_plugin_v3!, native_dist_editor_plugin_v3!}` generate descriptor/static report/behavior/schema/manifest/bridge-method tables and entry exports from a crate-local declaration. `dist_plugin_one_file_export_compiles` covers descriptor export, runtime entry gating, registration manifest pointer, and bridge table emission; `zircon_plugin_runtime_diagnostics_dist` covers editor-only descriptor/editor entry export with no runtime entry.

`zircon_plugin_native_dynamic_fixture_native` now depends on `zircon_plugin_sdk` with `default-features = false, features = ["native"]`. Its source keeps fixture-specific command handling, state save/restore, asset import, bridge tick callback, and host diagnostics, but no longer defines local ABI structs, owner-token free logic, panic hook handling, capability-list parsing, descriptor/report statics, bridge method table statics, or native export functions by hand.

## Runtime Declaration

`RuntimePluginDeclaration` wraps `RuntimePluginDescriptorBuilder` and exposes the same chainable authoring knobs for category, enabled/required defaults, target modes, capabilities, system sets, system anchors, maturity, capability status, optional features, and default packaging.

Frameworks 02 M3 also exposes descriptor ordering knobs here: `RuntimePluginDeclaration::with_init_level(...)` and `RuntimePluginDeclaration::with_module_dependency(...)` update the embedded `RuntimePluginDescriptor` module descriptor before both descriptor and package-manifest projection. The runtime export macro self-test covers that a declaration-generated package manifest preserves the init level and dependency row.

The declaration can return:

- `descriptor()` for runtime registration paths.
- `package_manifest()` for generated/static manifest parity.
- `into_descriptor()` when ownership should move into a plugin registration path.

## Runtime Registration Builder

`RuntimePluginRegistrationBuilder` is the Plugins 12 M3 authoring surface for runtime registration. It wraps the low-level `RuntimeExtensionRegistry` sequence so plugin authors do not manually pass `PluginModuleId` values between module and system registration.

The intended runtime path is:

- `RuntimePluginRegistrationBuilder::new(registry).module(module_name, module_descriptor())`
- `module.runtime_scene_system(system_id, stage, system_fn)`
- optional `module.event::<EventType>(manifest)`, `module.plugin_option(manifest)`, or `module.plugin_event_catalog(manifest)` for plugin-owned runtime metadata
- optional `.in_set(...)`, `.with_order(...)`, `.before(...)`, or `.after(...)`
- `.register()`

The 2026-06-28 D8 runtime registration builder original evidence paths slice extends the representative animation migration to `zircon_plugin_physics_runtime` and `zircon_plugin_net_runtime`. `AnimationRuntimePlugin::register(...)`, `PhysicsRuntimePlugin::register(...)`, and `NetRuntimePlugin::register(...)` now register their runtime module through the SDK builder and pass only a `RuntimePluginModuleRegistration` handle to system registration code. `physics.step`, `net.poll_ingress`, `net.flush_egress`, and the net typed runtime event are all declared through that module handle instead of direct `PluginModuleId` / `RuntimeExtensionRegistry` calls. Guard `review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder` and status `d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred` lock this D8 convergence; module-only and importer-private registry mutation remain separate ownership cases.

The 2026-06-28 D10 animation/physics bridge call migration adds the SDK owner-tracked `RuntimePluginModuleRegistration::export_interface::<T>(...)` helper and re-exports `PluginInterface`, `WeakBridge`, and `BridgeError` through the SDK runtime surface. Physics now exports the runtime-owned `physics.query.v1` contract as `dyn PhysicsQueryInterface`, and the animation/physics contract test resolves it through `WeakBridge<dyn PhysicsQueryInterface>` instead of concrete manager lookup. Guard `review_d10_animation_physics_tests_use_sdk_bridge_call` records status `d10_animation_physics_bridge_call_static_passed_cargo_deferred`; Cargo remains deferred for this implementation slice.

## Runtime Export Macro

`runtime_plugin_exports!(PluginType)` generates the standard runtime crate helper surface:

- `runtime_plugin()`
- `package_manifest()`
- `runtime_selection()`
- `plugin_registration()`

The generated helpers call the `RuntimePlugin` trait methods rather than reading the descriptor directly. This preserves plugin-specific `package_manifest(...)` overrides such as `NetRuntimePlugin`, while still removing the copied forwarding blocks from each crate root.

The D12 runtime helper export macro rollout covers the trait-backed runtime plugin crates that owned copied helper blocks: `ai`, `animation`, `hybrid_gi`, `navigation`, `net`, `particles`, `physics`, `prefab_tools`, `rendering`, `solari`, `terrain`, `texture`, `tilemap_2d`, `virtual_geometry`, and `zr_vm_language`. Each keeps plugin-specific descriptor/register behavior in its `RuntimePlugin` implementation and delegates the repeated helper exports to the SDK macro. The original Plugins 12 status is `plugins_12_runtime_export_macro_rollout_check_passed`; the Runtime 15 mirror guard `review_d12_runtime_helper_exports_use_sdk_macro` records this as `d12_runtime_export_macro_review_synced_static_passed_cargo_deferred` and keeps the guard in `tests/runtime_absorption/code_review_findings/plugin_importer_dx/d12_runtime_exports.rs`.

The 2026-06-28 D13 importer runtime export macro convergence extends the same helper owner to every first-party importer runtime crate: `asset_importers/{audio,data,model,shader,texture}` plus split importers `audio_importer`, `gltf_importer`, `obj_importer`, `opus_importer`, `shader_wgsl_importer`, `texture_importer`, and `ui_document_importer`. These 12/12 importer runtime `plugin.rs` owners now use `zircon_plugin_sdk::runtime_plugin_exports!`; none hand-writes `ProjectPluginSelection` or `RuntimePluginRegistrationReport` helper blocks. The importer `RuntimePlugin` implementations still own descriptors, importer descriptors, and private registry mutation, while repeated targets/platforms/module/dist-module manifest projection goes through `ImporterRuntimeManifestBuilder`. Guards `review_d13_importer_runtime_exports_use_sdk_macro` and `review_d13_importer_runtime_manifests_use_sdk_builder` lock this helper/manifest convergence with statuses `d13_importer_runtime_export_macro_convergence_static_passed_cargo_deferred` and `d13_importer_runtime_manifest_builder_convergence_static_passed_cargo_deferred`.

## Runtime Test Fixture

The `test` module adds `TestRuntime::builder()` for cross-plugin integration tests.

The 2026-06-28 D11 animation/physics TestRuntime fixture migration moves the original animation/physics contract test onto this SDK fixture. `runtime_physics_animation_tick_contract/runtime_helpers.rs` now owns the `TestRuntime::builder()` setup with physics and animation plugins, `runtime_physics_animation_tick_contract/target_resolution.rs` owns the target-id fallback contract tests, and the main test body uses `runtime.create_default_level()` and `runtime.tick_level_seconds(...)` instead of rebuilding CoreRuntime, Scene modules, fixed-step clocks, or world extension installation locally. Guard `review_d11_animation_physics_tests_use_sdk_test_runtime_fixture` and status `d11_animation_physics_test_runtime_fixture_static_passed_cargo_deferred` lock the migration.

`TestRuntimeBuilder` defaults to the common runtime stack used by plugin tests:

- register and activate foundation, asset, and scene modules;
- collect runtime plugin registration reports;
- merge them through `RuntimePluginCatalog::runtime_extensions()`;
- register plugin-contributed runtime modules;
- install scene runtime hooks and world runtime extensions;
- activate plugin-contributed modules after the base stack.

The fixture exposes `handle()`, `runtime()`, `extension_report()`, `activated_modules()`, typed `resolve_manager(...)`, `create_default_level()`, `advance_time_by_seconds(...)`, and `tick_level_seconds(...)`. Tests can turn off base modules, base activation, plugin activation, scene hooks, or world extensions when they are testing narrower contracts.

This closes Plugins 12 M2/T5 for D11 by moving the repeated `CoreRuntime + foundation + asset + scene + plugin extension registry + fixed-step clock` setup into the SDK. It does not migrate every existing long test yet; M5 touch-it-conform-it will replace local fixtures as affected plugin tests are edited.

## Open Work

Completed in this area:

- M2/T1 builder baseline for package, module, and runtime declaration builders.
- M2/T2 first skeleton sample using `zircon_plugin_sdk_examples_editor`, with `plugins_12_crate_skeleton_conformance` keeping that sample clean.
- M2/T3 native ABI helper feature and macros, with `native_dynamic_fixture` consuming SDK-owned runtime ABI helpers and `runtime_diagnostics` consuming the editor-only dist helper.
- Plugins 13 M2/T1 native registration manifest schema/DTO round-trip for dist registration declarations.
- M2/T4 editor `authoring_plugin!` macro plus sample and global core workspace dependency inheritance guards.
- M2/T5 `plugin_sdk::test::TestRuntime::builder()` fixture with SDK self-tests covering module activation and level ticking.
- D11 animation/physics TestRuntime fixture migration with `review_d11_animation_physics_tests_use_sdk_test_runtime_fixture` and `d11_animation_physics_test_runtime_fixture_static_passed_cargo_deferred`.
- D10 animation/physics bridge call migration with `physics.query.v1`, `WeakBridge<dyn PhysicsQueryInterface>`, `review_d10_animation_physics_tests_use_sdk_bridge_call`, and `d10_animation_physics_bridge_call_static_passed_cargo_deferred`.
- M3/T1 importer family and split importer registration entry cutover, with trait-backed reports and descriptor-derived selections.
- Frameworks 02 M3 native/SDK module descriptor projection, with SDK module builders and runtime declarations exposing `InitLevel` / `ModuleDependencySpec`; status `frameworks_02_m3_native_sdk_module_descriptor_projection_rustfmt_python_app_check_passed_plugin_sdk_locked_blocked`.
- M3/T2 runtime registration builder plus animation runtime representative migration.
- M3/D12 `runtime_plugin_exports!` macro plus first-party trait-backed runtime helper rollout across ai, animation, hybrid_gi, navigation, net, particles, physics, prefab_tools, rendering, solari, terrain, texture, tilemap_2d, virtual_geometry, and zr_vm_language.
- D12 runtime helper export macro review/status sync with `review_d12_runtime_helper_exports_use_sdk_macro` and `d12_runtime_export_macro_review_synced_static_passed_cargo_deferred`.
- M3/D13 importer runtime export macro and `ImporterRuntimeManifestBuilder` rollout across all 12 first-party importer runtime owners.
- D1 capability single-source review/status sync with `review_d1_plugin_capabilities_use_single_source_and_sdk_builder_mirror`, `plugins_12_runtime_capability_single_source_guard_passed`, `plugins_12_capability_single_source_conformance`, and `d1_capability_single_source_review_synced_static_passed_cargo_deferred`.
- M4/T2 SDK guard for `PluginFeatureBundleBuilder` and editor `mirrors_runtime(...)`, with `m4_t2_builder_mirror_gate_status = sdk-builder-mirror-clean`.
- M2/D5 editor authoring macro consumer guard for animation/physics/net editor plugins, with `zircon_plugin_sdk::authoring_plugin!`, `d5_editor_authoring_macro_consumers_static_passed_cargo_deferred`, and `review_d5_editor_authoring_plugins_use_sdk_macro`.
- M4/D9 editor/runtime mirror consumer guard for animation/physics/net editor plugins, with `d9_editor_runtime_mirror_consumers_static_passed_cargo_deferred` and `d9_editor_runtime_mirror_gate_status = editor-runtime-mirror-clean`.
- M5 RuntimePluginId open string-newtype convergence; plugin ids can now carry valid external keys without adding engine core enum variants.

Still open:

- Broader editor/runtime capability rollout beyond the D1 audited first-party runtime roots, SDK builder mirror, and animation/physics/net editor mirror consumers.
- M5 touch-it-conform-it replacement of remaining local long-test fixtures with `plugin_sdk::test`.
