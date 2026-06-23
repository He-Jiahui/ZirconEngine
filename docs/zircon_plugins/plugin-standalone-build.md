---
related_code:
  - zircon_plugins/plugin_sdk/Cargo.toml
  - zircon_plugins/plugin_sdk/src/dist.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/runtime_exports.rs
  - zircon_plugins/native_dynamic_fixture/plugin.toml
  - zircon_plugins/native_dynamic_fixture/assets/shader.wgsl
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/dist/Cargo.toml
  - zircon_plugins/ai/dist/src/lib.rs
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/ai/runtime/src/tests/registration.rs
  - zircon_plugins/solari/plugin.toml
  - zircon_plugins/solari/dist/Cargo.toml
  - zircon_plugins/solari/dist/src/lib.rs
  - zircon_plugins/solari/runtime/src/lib.rs
  - zircon_plugins/solari/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/zr_vm_language/dist/Cargo.toml
  - zircon_plugins/zr_vm_language/dist/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/navigation/dist/Cargo.toml
  - zircon_plugins/navigation/dist/src/lib.rs
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/src/plugin.rs
  - zircon_plugins/navigation/runtime/src/tests/registration.rs
  - zircon_plugins/animation/plugin.toml
  - zircon_plugins/animation/dist/Cargo.toml
  - zircon_plugins/animation/dist/src/lib.rs
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/plugin.rs
  - zircon_plugins/animation/runtime/src/tests.rs
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/dist/Cargo.toml
  - zircon_plugins/hybrid_gi/dist/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/capability.rs
  - zircon_plugins/hybrid_gi/runtime/src/plugin.rs
  - zircon_plugins/hybrid_gi/runtime/src/tests.rs
  - zircon_plugins/physics/plugin.toml
  - zircon_plugins/physics/dist/Cargo.toml
  - zircon_plugins/physics/dist/src/lib.rs
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/plugin.rs
  - zircon_plugins/physics/runtime/src/tests.rs
  - zircon_plugins/particles/plugin.toml
  - zircon_plugins/particles/dist/Cargo.toml
  - zircon_plugins/particles/dist/src/lib.rs
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/plugin.rs
  - zircon_plugins/particles/runtime/src/tests/package_manifest.rs
  - zircon_plugins/texture/plugin.toml
  - zircon_plugins/texture/dist/Cargo.toml
  - zircon_plugins/texture/dist/src/lib.rs
  - zircon_plugins/texture/runtime/src/lib.rs
  - zircon_plugins/texture/runtime/src/plugin.rs
  - zircon_plugins/texture/runtime/src/tests.rs
  - tools/zircon_export/cli.py
  - tools/zircon_export/plugin_build.py
  - tools/zircon_export/tests/test_plugin_build.py
  - tools/tests/test_zircon_build_plugin_carriers.py
  - tools/tests/test_plugin_standalone_ci_matrix.py
  - zircon_plugins/animation/plugin.toml
  - zircon_plugins/animation/dist/Cargo.toml
  - zircon_plugins/animation/dist/src/lib.rs
  - zircon_plugins/animation/runtime/Cargo.toml
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/plugin.rs
  - zircon_plugins/animation/runtime/src/tests.rs
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/dist/Cargo.toml
  - zircon_plugins/hybrid_gi/dist/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/Cargo.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/capability.rs
  - zircon_plugins/hybrid_gi/runtime/src/plugin.rs
  - zircon_plugins/hybrid_gi/runtime/src/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/representation.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime/src/plugin/export_build_plan/native_dynamic_package_plan.rs
  - zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation/report.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation/schema.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/compatibility.rs
  - zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/registration_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/reports.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/runtime_behavior.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_failures.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/registration_replay.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_distribution_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/constructors.rs
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs
  - zircon_runtime/src/tests/plugin_extensions/plugin_workspace_shape.rs
  - zircon_runtime/src/plugin/mod.rs
  - tools/zircon_build.py
  - tools/zircon_export/native_build.py
  - tools/zircon_export/native_signing.py
  - tools/audit_plugin_structure.py
  - tools/plugin_structure_audits/dependency_boundary.py
  - .github/workflows/ci.yml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/audio_importer/runtime/src/capability.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/capability.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/obj_importer/runtime/src/capability.rs
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/src/capability.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/capability.rs
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/capability.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/runtime/src/capability.rs
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/src/capability.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/capability.rs
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/src/capability.rs
  - zircon_plugins/asset_importers/audio/runtime/Cargo.toml
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/capability.rs
  - zircon_plugins/asset_importers/audio/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/texture/runtime/Cargo.toml
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/capability.rs
  - zircon_plugins/asset_importers/texture/runtime/src/plugin.rs
  - zircon_plugins/ai/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/solari/runtime/Cargo.toml
  - zircon_plugins/solari/runtime/src/lib.rs
  - zircon_plugins/solari/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/zr_vm_language/dist/Cargo.toml
  - zircon_plugins/zr_vm_language/dist/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_plugins/native_window_hosting/editor/src/lib.rs
  - zircon_plugins/native_window_hosting/editor/src/capability.rs
  - zircon_plugins/native_window_hosting/editor/src/extension_ids.rs
  - zircon_plugins/native_window_hosting/editor/src/plugin.rs
  - zircon_plugins/native_window_hosting/editor/src/tests.rs
  - zircon_plugins/runtime_diagnostics/editor/src/lib.rs
  - zircon_plugins/runtime_diagnostics/editor/src/capability.rs
  - zircon_plugins/runtime_diagnostics/editor/src/extension_ids.rs
  - zircon_plugins/runtime_diagnostics/editor/src/plugin.rs
  - zircon_plugins/runtime_diagnostics/editor/src/tests.rs
  - zircon_plugins/ui_asset_authoring/editor/src/lib.rs
  - zircon_plugins/ui_asset_authoring/editor/src/capability.rs
  - zircon_plugins/ui_asset_authoring/editor/src/extension_ids.rs
  - zircon_plugins/ui_asset_authoring/editor/src/plugin.rs
  - zircon_plugins/ui_asset_authoring/editor/src/tests.rs
  - zircon_plugins/prefab_tools/runtime/src/lib.rs
  - zircon_plugins/prefab_tools/runtime/src/capability.rs
  - zircon_plugins/prefab_tools/runtime/src/plugin.rs
  - zircon_plugins/prefab_tools/runtime/src/tests.rs
  - zircon_plugins/prefab_tools/editor/src/lib.rs
  - zircon_plugins/prefab_tools/editor/src/authoring.rs
  - zircon_plugins/prefab_tools/editor/src/capability.rs
  - zircon_plugins/prefab_tools/editor/src/extension_ids.rs
  - zircon_plugins/prefab_tools/editor/src/plugin.rs
  - zircon_plugins/prefab_tools/editor/src/tests.rs
  - zircon_plugins/terrain/runtime/src/lib.rs
  - zircon_plugins/terrain/runtime/src/capability.rs
  - zircon_plugins/terrain/runtime/src/plugin.rs
  - zircon_plugins/terrain/runtime/src/tests.rs
  - zircon_plugins/terrain/plugin.toml
  - zircon_plugins/terrain/dist/Cargo.toml
  - zircon_plugins/terrain/dist/src/lib.rs
  - zircon_plugins/terrain/editor/src/lib.rs
  - zircon_plugins/terrain/editor/src/authoring.rs
  - zircon_plugins/terrain/editor/src/capability.rs
  - zircon_plugins/terrain/editor/src/extension_ids.rs
  - zircon_plugins/terrain/editor/src/plugin.rs
  - zircon_plugins/terrain/editor/src/tests.rs
  - zircon_plugins/tilemap_2d/runtime/src/lib.rs
  - zircon_plugins/tilemap_2d/runtime/src/capability.rs
  - zircon_plugins/tilemap_2d/runtime/src/plugin.rs
  - zircon_plugins/tilemap_2d/runtime/src/tests.rs
  - zircon_plugins/tilemap_2d/editor/src/lib.rs
  - zircon_plugins/tilemap_2d/editor/src/authoring.rs
  - zircon_plugins/tilemap_2d/editor/src/capability.rs
  - zircon_plugins/tilemap_2d/editor/src/extension_ids.rs
  - zircon_plugins/tilemap_2d/editor/src/plugin.rs
  - zircon_plugins/tilemap_2d/editor/src/tests.rs
  - zircon_plugins/particles/plugin.toml
  - zircon_plugins/particles/dist/Cargo.toml
  - zircon_plugins/particles/dist/src/lib.rs
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/capability.rs
  - zircon_plugins/particles/runtime/src/plugin.rs
  - zircon_plugins/particles/runtime/src/simulation/cpu.rs
  - zircon_plugins/particles/runtime/src/tests/package_manifest.rs
  - zircon_plugins/particles/editor/src/lib.rs
  - zircon_plugins/particles/editor/src/authoring.rs
  - zircon_plugins/particles/editor/src/capability.rs
  - zircon_plugins/particles/editor/src/extension_ids.rs
  - zircon_plugins/particles/editor/src/plugin.rs
  - zircon_plugins/particles/editor/src/tests.rs
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/capability.rs
  - zircon_plugins/physics/runtime/src/plugin.rs
  - zircon_plugins/physics/runtime/src/tests.rs
  - zircon_plugins/physics/editor/src/lib.rs
  - zircon_plugins/physics/editor/src/capability.rs
  - zircon_plugins/physics/editor/src/extension_ids.rs
  - zircon_plugins/physics/editor/src/plugin.rs
  - zircon_plugins/physics/editor/src/tests.rs
  - zircon_plugins/texture/runtime/src/lib.rs
  - zircon_plugins/texture/runtime/src/capability.rs
  - zircon_plugins/texture/runtime/src/manager.rs
  - zircon_plugins/texture/runtime/src/module.rs
  - zircon_plugins/texture/runtime/src/plugin.rs
  - zircon_plugins/texture/runtime/src/tests.rs
  - zircon_plugins/texture/plugin.toml
  - zircon_plugins/texture/dist/Cargo.toml
  - zircon_plugins/texture/dist/src/lib.rs
  - zircon_plugins/texture/editor/src/lib.rs
  - zircon_plugins/texture/editor/src/capability.rs
  - zircon_plugins/texture/editor/src/extension_ids.rs
  - zircon_plugins/texture/editor/src/plugin.rs
  - zircon_plugins/texture/editor/src/tests.rs
  - zircon_plugins/editor_build_export_desktop/editor/src/lib.rs
  - zircon_plugins/editor_build_export_desktop/editor/src/capability.rs
  - zircon_plugins/editor_build_export_desktop/editor/src/extension_ids.rs
  - zircon_plugins/editor_build_export_desktop/editor/src/plugin.rs
  - zircon_plugins/editor_build_export_desktop/editor/src/tests.rs
  - zircon_plugins/editor_build_export_desktop/editor/src/export_wizard.rs
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/capability.rs
  - zircon_plugins/sound/runtime/src/plugin.rs
  - zircon_plugins/sound/runtime/src/runtime_plugin/descriptor.rs
  - zircon_plugins/sound/runtime/src/runtime_plugin/feature_manifest.rs
  - zircon_plugins/sound/editor/src/lib.rs
  - zircon_plugins/sound/editor/src/authoring_bindings.rs
  - zircon_plugins/sound/editor/src/capability.rs
  - zircon_plugins/sound/editor/src/extension_ids.rs
  - zircon_plugins/sound/editor/src/plugin.rs
  - zircon_plugins/sound/editor/src/tests.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/capability.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/plugin.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/tests.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/capability.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/plugin.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/tests.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/capability.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/plugin.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/tests.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/capability.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/plugin.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/tests.rs
  - zircon_plugins/timeline_sequence/editor/src/lib.rs
  - zircon_plugins/timeline_sequence/editor/src/capability.rs
  - zircon_plugins/timeline_sequence/editor/src/extension_ids.rs
  - zircon_plugins/timeline_sequence/editor/src/plugin.rs
  - zircon_plugins/timeline_sequence/editor/src/tests.rs
plan_sources:
  - docs/plans/zircon_plugins/13-standalone-plugin-build.md
  - docs/plans/zircon_plugins/09-export-publishing.md
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - python -m py_compile tools/audit_plugin_structure.py tools/plugin_structure_audits/dependency_boundary.py
  - python tools/audit_plugin_structure.py --json: standalone_distribution_conformance.m1_dist_dependency_boundary_gate_status=dist-boundary-clean, dist_capable_plugin_count=1, dist_dependency_boundary_violations=0
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --no-default-features --features dist --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-standalone-dist-m1-0623 --message-format short --color never
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m4-capability-test-nodebug-0623 --message-format short --color never plugins_13_dist_dependency_boundary_clean -- --test-threads=1 --nocapture
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --no-default-features --features native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-standalone-m2-registration-0623 --message-format short --color never
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --no-default-features --features native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-standalone-m2-registration-0623 --message-format short --color never native_dynamic_registration_manifest_round_trips -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib malformed_registration_schema_marks_behavior_invalid --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-native-registration-m2-0623 --message-format short --color never -- --test-threads=1 --nocapture: blocked by unrelated runtime lib-test compile drift after registration ABI field errors were cleared
  - rustfmt --edition 2021 --check zircon_runtime/src/plugin/native.rs zircon_runtime/src/plugin/native_plugin_loader/behavior_validation.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs zircon_runtime/src/plugin/native_plugin_loader/registration_manifest.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/registration_replay.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/reports.rs zircon_runtime/src/plugin/native_plugin_loader/mod.rs zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs: passed 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test -p zircon_runtime --lib --no-default-features --features core-min --offline --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-m2-t2-0623 --message-format short --color never dist_system_plugin_loads_and_ticks_via_bridge -- --test-threads=1 --nocapture: blocked by unrelated runtime lib-test compile drift (`runtime_absorption/structure_convention/test_file_budget.rs` missing child modules) after M2/T2 replay type errors were cleared
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --no-default-features --features native dist_plugin_one_file_export_compiles --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m2-t3-sdk: 1 passed, 0 failed on 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --no-default-features --features dist --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m2-t3-fixture: passed 2026-06-23
  - python -m py_compile tools/zircon_export/plugin_build.py tools/zircon_export/cli.py tools/zircon_export/tests/test_plugin_build.py: passed 2026-06-23
  - python -m unittest tools.zircon_export.tests.test_plugin_build: 4 passed, 0 failed on 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 CARGO_BUILD_JOBS=1 python -m tools.zircon_export plugin build native_dynamic_fixture --form dist --platform windows-x86_64 --mode debug --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-m3-package-fixture --target-dir D:\cargo-targets\zircon-plugin-m3-build-fixture: passed 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 CARGO_BUILD_JOBS=1 python -m tools.zircon_export plugin build native_dynamic_fixture --form dist --platform windows-x86_64 --mode debug --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-m3-t3-package-a --target-dir D:\cargo-targets\zircon-plugin-m3-t3-build-fixture: passed 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 CARGO_BUILD_JOBS=1 python -m tools.zircon_export plugin build native_dynamic_fixture --form dist --platform windows-x86_64 --mode debug --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-m3-t3-package-b --target-dir D:\cargo-targets\zircon-plugin-m3-t3-build-fixture: passed 2026-06-23; package sha256 comparison returned MATCH
  - CARGO_PROFILE_DEV_DEBUG=0 CARGO_BUILD_JOBS=1 python -m tools.zircon_export plugin build native_dynamic_fixture --form dist --platform windows-x86_64 --mode debug --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-m4-t1-package-fixture --target-dir D:\cargo-targets\zircon-plugin-m4-t1-build-fixture: timed out after 604s while building/running zircon_export_pack; not counted as passed
  - python -m py_compile tools/zircon_build.py tools/tests/test_zircon_build_plugin_carriers.py: passed 2026-06-23
  - python -m unittest discover -s tools/tests -p test_zircon_build_plugin_carriers.py: 1 passed, 0 failed on 2026-06-23
  - python -m py_compile tools/audit_plugin_structure.py tools/plugin_structure_audits/dependency_boundary.py tools/tests/test_plugin_standalone_ci_matrix.py: passed 2026-06-23
  - python -m unittest discover -s tools/tests -p test_plugin_standalone_ci_matrix.py: 1 passed, 0 failed on 2026-06-23
  - python tools/audit_plugin_structure.py --json: standalone_distribution_conformance.dist_build_matrix_count=1, dist_capable_plugin_count=1, dist_dependency_boundary_violations=0 on 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 cargo build --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --no-default-features --features dist --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-ci-fixture-build --message-format short --color never: passed 2026-06-23
  - rustfmt --edition 2021 zircon_runtime/src/plugin/mod.rs zircon_runtime/src/plugin/package_manifest/plugin_distribution_manifest.rs zircon_runtime/src/plugin/package_manifest/mod.rs zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs zircon_runtime/src/plugin/package_manifest/constructors.rs zircon_runtime/src/plugin/native_plugin_loader/compatibility.rs zircon_runtime/src/plugin/native_plugin_loader/mod.rs zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs: passed 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m4-t2-loader-compat-check --message-format short --color never: passed 2026-06-23 with existing warning noise
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test -p zircon_runtime --lib --no-default-features --features core-min native_loader_skips_distribution_with_incompatible_engine_range_before_library_probe --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m4-t2-loader-compat-test --message-format short --color never -- --test-threads=1 --nocapture: blocked by unrelated runtime lib-test compile drift (`script/vm/tests.rs` missing child modules, `runtime_absorption/structure_convention/test_file_budget.rs` missing child modules, private gltf fixture re-exports, and existing WgpuRenderFramework test API drift); not counted as passed
  - rustfmt --edition 2021 zircon_runtime\src\builtin\runtime_modules\ids\plugin_id.rs zircon_runtime\src\builtin\runtime_modules\plugin_modules\loader.rs zircon_runtime\src\builtin\runtime_modules\tests\registration\structure.rs zircon_runtime\src\tests\plugin_extensions\plugin_workspace_shape.rs: passed 2026-06-23 for Plugins 13 M5/T3 RuntimePluginId string-newtype
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-plugin-id-check --message-format short --color never: passed 2026-06-23 with existing warning noise
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-plugin-id-sdk-check --message-format short --color never: passed 2026-06-23 with existing warning noise
  - cargo test -p zircon_runtime --lib --no-default-features --features core-min runtime_plugin_id_accepts_external_keys_without_core_variant --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-plugin-id-check --message-format short --color never -- --test-threads=1 --nocapture: blocked before running by unrelated runtime lib-test compile drift; not counted as passed
  - rustfmt --edition 2021 --check importer runtime lib.rs/capability.rs set for audio/gltf/obj/opus/shader_wgsl/texture/ui_document and asset_importers/data/model/shader: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=25, plugin_skeleton_gate.migration_debt_count=25, standalone_distribution_conformance.dist_capable_plugin_count=1 on 2026-06-23
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p each of zircon_plugin_audio_importer_runtime, zircon_plugin_gltf_importer_runtime, zircon_plugin_obj_importer_runtime, zircon_plugin_opus_importer_runtime, zircon_plugin_shader_wgsl_importer_runtime, zircon_plugin_texture_importer_runtime, zircon_plugin_ui_document_importer_runtime, zircon_plugin_asset_importer_data_runtime, zircon_plugin_asset_importer_model_runtime, zircon_plugin_asset_importer_shader_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-skeleton-split-importers-check --message-format short --color never: each package exit=0 on 2026-06-23 with existing warning noise
  - rustfmt --edition 2021 --check runtime-only skeleton owner files for ai, asset_importers/audio, asset_importers/texture, solari, zr_vm_language: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=20, plugin_skeleton_gate.migration_debt_count=20, standalone_distribution_conformance.dist_capable_plugin_count=1 on 2026-06-23
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ai_runtime -p zircon_plugin_asset_importer_audio_runtime -p zircon_plugin_asset_importer_texture_runtime -p zircon_plugin_solari_runtime -p zircon_plugin_zr_vm_language_runtime --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-only-skeleton-check --message-format short --color never: passed 2026-06-23 and refreshed zircon_plugins/Cargo.lock
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ai_runtime -p zircon_plugin_asset_importer_audio_runtime -p zircon_plugin_asset_importer_texture_runtime -p zircon_plugin_solari_runtime -p zircon_plugin_zr_vm_language_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-only-skeleton-check --message-format short --color never: passed 2026-06-23 with existing warning noise
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_audio_runtime -p zircon_plugin_asset_importer_texture_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-only-skeleton-check --message-format short --color never package_declares_ -- --test-threads=1 --nocapture: timed out after 904s on 2026-06-23 with no test result; residual cargo/rustc processes for this target-dir were stopped; not counted as passing
  - python -m py_compile tools/plugin_structure_audits/capability.py tools/audit_plugin_structure.py: passed 2026-06-23 after plugin.rs-owned runtime_capabilities() audit support
  - rustfmt --edition 2021 --check editor-only skeleton owner files for native_window_hosting, runtime_diagnostics, ui_asset_authoring: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=17, plugin_skeleton_gate.migration_debt_count=17, standalone_distribution_conformance.dist_capable_plugin_count=1, capability_conformance.capability_source_mismatches=0 on 2026-06-23
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_native_window_hosting_editor -p zircon_plugin_runtime_diagnostics_editor -p zircon_plugin_ui_asset_authoring_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-editor-small-skeleton-check --message-format short --color never: blocked before target plugin packages by unrelated zircon_editor retained-host compile drift; not counted as passing
  - rustfmt --edition 2021 authoring runtime/editor skeleton owner files for prefab_tools, terrain, tilemap_2d: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=14, plugin_skeleton_gate.migration_debt_count=14, standalone_distribution_conformance.dist_capable_plugin_count=1, capability_conformance.capability_source_mismatches=0 on 2026-06-23
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_prefab_tools_runtime -p zircon_plugin_terrain_runtime -p zircon_plugin_tilemap_2d_runtime --offline --jobs 1 --target-dir E:\cargo-targets\zircon-plugin-m5-authoring-runtime-0623 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime warning noise
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_prefab_tools_editor -p zircon_plugin_terrain_editor -p zircon_plugin_tilemap_2d_editor --offline --jobs 1 --target-dir E:\cargo-targets\zircon-plugin-m5-authoring-editor-0623 --message-format short --color never: timed out twice while building dependencies; no final output captured, not counted as passing
  - rustfmt --edition 2021 particles/physics/texture runtime/editor skeleton owner files plus zircon_plugins/particles/runtime/src/simulation/cpu.rs: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=11, plugin_skeleton_gate.migration_debt_count=11, standalone_distribution_conformance.dist_capable_plugin_count=1, capability_conformance.capability_source_mismatches=0 on 2026-06-23
  - cargo check -p zircon_plugin_particles_runtime --offline; cargo check -p zircon_plugin_physics_runtime --offline; cargo check -p zircon_plugin_texture_runtime --offline: passed 2026-06-23 with existing warning noise
  - cargo check --manifest-path zircon_plugins\particles\editor\Cargo.toml --offline; cargo check --manifest-path zircon_plugins\physics\editor\Cargo.toml --offline --target-dir target\codex-plugin-validation; cargo check --manifest-path zircon_plugins\texture\editor\Cargo.toml --offline --target-dir target\codex-plugin-validation: passed 2026-06-23 with existing warning noise
  - rustfmt --edition 2021 zircon_plugins\editor_build_export_desktop\editor\src\lib.rs zircon_plugins\editor_build_export_desktop\editor\src\capability.rs zircon_plugins\editor_build_export_desktop\editor\src\extension_ids.rs zircon_plugins\editor_build_export_desktop\editor\src\plugin.rs zircon_plugins\editor_build_export_desktop\editor\src\tests.rs zircon_plugins\editor_build_export_desktop\editor\src\export_wizard.rs: passed 2026-06-23
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=10, plugin_skeleton_gate.migration_debt_count=10, standalone_distribution_conformance.dist_capable_plugin_count=1, capability_conformance.capability_source_mismatches=0 on 2026-06-23
  - cargo check --manifest-path zircon_plugins\editor_build_export_desktop\editor\Cargo.toml --all-targets --offline --target-dir target\codex-plugin-validation --message-format short --color never: passed 2026-06-23 with existing runtime/editor warning noise
  - rustfmt --edition 2021 sound runtime/editor skeleton owner files for main crate plus ray_traced_convolution_reverb and timeline_animation_track feature crates: passed 2026-06-23
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=9, plugin_skeleton_gate.migration_debt_count=9, standalone_distribution_conformance.dist_capable_plugin_count=1, capability_conformance.capability_source_mismatches=0 on 2026-06-23 after sound owner rollout
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --lib --offline --target-dir target\codex-plugin-validation --message-format short --color never: blocked before target sound crate by unrelated zircon_runtime render compile drift; not counted as passing
  - rustfmt --edition 2021 --check zircon_plugins\timeline_sequence\editor\src\lib.rs zircon_plugins\timeline_sequence\editor\src\capability.rs zircon_plugins\timeline_sequence\editor\src\extension_ids.rs zircon_plugins\timeline_sequence\editor\src\plugin.rs zircon_plugins\timeline_sequence\editor\src\tests.rs: passed 2026-06-23
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=8, plugin_skeleton_gate.migration_debt_count=8, standalone_distribution_conformance.dist_capable_plugin_count=1, capability_conformance.capability_source_mismatches=0 on 2026-06-23 after timeline_sequence owner rollout
  - cargo check --manifest-path zircon_plugins\timeline_sequence\editor\Cargo.toml --lib --offline --target-dir target\codex-plugin-validation --message-format short --color never: blocked before target timeline_sequence crate by unrelated zircon_runtime render compile drift; not counted as passing
  - python tools\audit_plugin_structure.py --json: plugin_skeleton_gate.m2_gate_status=sample-clean-migration-debt-clear, skeleton_conformance.migration_debt_count=0, plugin_skeleton_gate.migration_debt_count=0, skeleton_conformance.migration_debt_roots=[], standalone_distribution_conformance.dist_capable_plugin_count=1, standalone_distribution_conformance.dist_build_matrix_count=1, standalone_distribution_conformance.dist_dependency_boundary_violations=0 on 2026-06-23 after final owner rollout
  - rustfmt --edition 2021 --check on 139 touched owner/façade files under animation, animation_graph, hybrid_gi, material_editor, navigation, net, rendering, timeline_sequence, and virtual_geometry: passed 2026-06-23
  - cargo check --manifest-path zircon_plugins\Cargo.toml --workspace --offline --locked --target-dir target\codex-plugin-validation --message-format short --color never: passed 2026-06-23 after final owner rollout and `hybrid_gi` RenderLayerSet mask alignment, with existing warning noise
  - rustfmt --edition 2021 zircon_plugins\solari\runtime\src\plugin.rs zircon_plugins\solari\runtime\src\lib.rs zircon_plugins\solari\dist\src\lib.rs zircon_runtime\src\plugin\native_plugin_loader\candidate_from_manifest.rs: passed 2026-06-23 after Solari dist rollout
  - python -m py_compile tools\zircon_export\plugin_build.py tools\zircon_export\tests\test_plugin_build.py tools\audit_plugin_structure.py tools\plugin_structure_audits\dependency_boundary.py tools\tests\test_plugin_standalone_ci_matrix.py: passed 2026-06-23 after Solari dist rollout
  - python -m unittest tools.zircon_export.tests.test_plugin_build: 4 passed, 0 failed on 2026-06-23 after Solari dist rollout
  - python -m unittest discover -s tools/tests -p test_plugin_standalone_ci_matrix.py: 1 passed, 0 failed on 2026-06-23 after Solari dist rollout
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=0, plugin_skeleton_gate.migration_debt_count=0, standalone_distribution_conformance.dist_capable_plugin_count=2, standalone_distribution_conformance.dist_build_matrix_count=2, standalone_distribution_conformance.dist_capable_plugins=["native_dynamic_fixture","solari"], standalone_distribution_conformance.dist_dependency_boundary_violations=0 on 2026-06-23 after Solari dist rollout
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_solari_dist --no-default-features --features dist --offline --locked --target-dir target\codex-plugin-solari-dist-check --message-format short --color never: passed 2026-06-23
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_solari_dist --no-default-features --features dist --offline --locked --target-dir target\codex-plugin-solari-dist-test --message-format short --color never -- --test-threads=1 --nocapture: 2 passed, 0 failed on 2026-06-23
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_solari_runtime --offline --locked --target-dir target\codex-plugin-solari-runtime-check --message-format short --color never: passed 2026-06-23 with existing warning noise
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_solari_runtime solari_package_manifest_declares_dist_contract --offline --locked --target-dir target\codex-plugin-solari-runtime-test --message-format short --color never -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-23 with existing warning noise
  - python -m tools.zircon_export plugin build solari --form dist --repo-root E:\Git\ZirconEngine --out target\codex-plugin-build-solari --target-dir target\codex-plugin-build-solari-target --offline: passed 2026-06-23; emitted `solari\solari.dll`, `solari\native\zircon_plugin_solari_dist.dll`, `native_plugins.toml`, and `plugins\native_plugins.toml`
  - cargo check --manifest-path zircon_plugins\Cargo.toml --workspace --offline --locked --target-dir target\codex-plugin-validation --message-format short --color never: passed 2026-06-23 after Solari dist rollout with existing warning noise
  - rustfmt --edition 2021 zircon_plugins\ai\runtime\src\plugin.rs zircon_plugins\ai\runtime\src\lib.rs zircon_plugins\ai\runtime\src\tests\registration.rs zircon_plugins\ai\dist\src\lib.rs: passed 2026-06-23 after AI dist rollout
  - python -m py_compile tools\zircon_export\plugin_build.py tools\zircon_export\tests\test_plugin_build.py tools\audit_plugin_structure.py tools\plugin_structure_audits\dependency_boundary.py tools\tests\test_plugin_standalone_ci_matrix.py: passed 2026-06-23 after AI dist rollout
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ai_dist --no-default-features --features dist --offline --target-dir target\codex-plugin-ai-dist-check --message-format short --color never: passed 2026-06-23 and refreshed zircon_plugins/Cargo.lock for the new ai/dist workspace member
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ai_dist --no-default-features --features dist --offline --locked --target-dir target\codex-plugin-ai-dist-check --message-format short --color never: passed 2026-06-23
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ai_dist --no-default-features --features dist --offline --locked --target-dir target\codex-plugin-ai-dist-test --message-format short --color never -- --test-threads=1 --nocapture: 2 passed, 0 failed on 2026-06-23
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ai_runtime ai_package_manifest_declares_dist_contract --offline --locked --target-dir target\codex-plugin-ai-runtime-test --message-format short --color never -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-23 with existing warning noise
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=0, plugin_skeleton_gate.migration_debt_count=0, standalone_distribution_conformance.dist_capable_plugin_count=3, standalone_distribution_conformance.dist_build_matrix_count=3, standalone_distribution_conformance.dist_capable_plugins=["ai","native_dynamic_fixture","solari"], standalone_distribution_conformance.dist_dependency_boundary_violations=0 on 2026-06-23 after AI dist rollout
  - python -m unittest discover -s tools/tests -p test_plugin_standalone_ci_matrix.py: 1 passed, 0 failed on 2026-06-23 after AI dist rollout
  - python -m unittest tools.zircon_export.tests.test_plugin_build: 4 passed, 0 failed on 2026-06-23 after AI dist rollout
  - python -m tools.zircon_export plugin build ai --form dist --repo-root E:\Git\ZirconEngine --out target\codex-plugin-build-ai --target-dir target\codex-plugin-build-ai-target --offline: passed 2026-06-23; emitted `ai\ai.dll`, `ai\native\zircon_plugin_ai_dist.dll`, `ai\ai.sig`, `native_plugins.toml`, and `plugins\native_plugins.toml`
  - cargo check --manifest-path zircon_plugins\Cargo.toml --workspace --offline --locked --target-dir target\codex-plugin-validation-ai-final --message-format short --color never: blocked on 2026-06-23 by unrelated `zircon_runtime` UI template compile drift (`mui_slot_name` unresolved imports and private `apply_mui_*_slot_props` imports); not counted as passing
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --offline --locked --target-dir target\codex-runtime-loader-check --message-format short --color never: blocked before compile by root Cargo.lock update requirement on 2026-06-23; root Cargo.lock was not modified
  - rustfmt --edition 2021 zircon_plugins\zr_vm_language\runtime\src\plugin.rs zircon_plugins\zr_vm_language\runtime\src\lib.rs zircon_plugins\zr_vm_language\runtime\src\tests\registration.rs zircon_plugins\zr_vm_language\dist\src\lib.rs: passed 2026-06-23 after ZrVM Language dist rollout
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_zr_vm_language_dist --no-default-features --features dist --offline --target-dir target\codex-plugin-zrvm-dist-check --message-format short --color never: passed 2026-06-23 and refreshed zircon_plugins/Cargo.lock for the new zr_vm_language/dist workspace member
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_zr_vm_language_dist --no-default-features --features dist --offline --locked --target-dir target\codex-plugin-zrvm-dist-check-final --message-format short --color never: passed 2026-06-23
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_zr_vm_language_dist --no-default-features --features dist --offline --locked --target-dir target\codex-plugin-zrvm-dist-test-final --message-format short --color never -- --test-threads=1 --nocapture: 2 passed, 0 failed on 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_zr_vm_language_runtime --offline --locked --target-dir target\codex-plugin-zrvm-runtime-check-final --message-format short --color never: passed 2026-06-23 with existing warning noise
- cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_zr_vm_language_runtime zr_vm_language_package_manifest_declares_dist_contract --offline --locked --target-dir target\codex-plugin-zrvm-runtime-test-final-verify --message-format short --color never -- --test-threads=1 --nocapture: after fixing the test to read the runtime `package_manifest()` helper directly, rerun was blocked before the target test by unrelated `zircon_runtime/src/ui/component/catalog/editor_showcase.rs` `numeric` missing compile drift on 2026-06-23; no runtime unit-test pass is claimed
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=0, plugin_skeleton_gate.migration_debt_count=0, standalone_distribution_conformance.dist_capable_plugin_count=4, standalone_distribution_conformance.dist_build_matrix_count=4, standalone_distribution_conformance.dist_capable_plugins=["ai","native_dynamic_fixture","solari","zr_vm_language"], standalone_distribution_conformance.dist_dependency_boundary_violations=0 on 2026-06-23 after ZrVM Language dist rollout
  - python -m unittest discover -s tools/tests -p test_plugin_standalone_ci_matrix.py: 1 passed, 0 failed on 2026-06-23 after ZrVM Language dist rollout
  - python -m unittest tools.zircon_export.tests.test_plugin_build: 4 passed, 0 failed on 2026-06-23 after ZrVM Language dist rollout
  - python -m py_compile tools\zircon_export\plugin_build.py tools\zircon_export\tests\test_plugin_build.py tools\audit_plugin_structure.py tools\plugin_structure_audits\dependency_boundary.py tools\tests\test_plugin_standalone_ci_matrix.py: passed 2026-06-23 after ZrVM Language dist rollout
  - python -m tools.zircon_export plugin build zr_vm_language --form dist --repo-root E:\Git\ZirconEngine --out target\codex-plugin-build-zrvm-final --target-dir target\codex-plugin-build-zrvm-target-final --offline: passed 2026-06-23; emitted `zr_vm_language\zr_vm_language.dll`, `zr_vm_language\native\zircon_plugin_zr_vm_language_dist.dll`, `zr_vm_language\zr_vm_language.sig`, `native_plugins.toml`, and `plugins\native_plugins.toml`
  - rustfmt --edition 2021 zircon_plugins\navigation\dist\src\lib.rs zircon_plugins\navigation\runtime\src\plugin.rs zircon_plugins\navigation\runtime\src\lib.rs zircon_plugins\navigation\runtime\src\tests\registration.rs: passed 2026-06-24 after Navigation dist rollout
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_navigation_dist --no-default-features --features dist --offline --target-dir target\codex-plugin-navigation-dist-check: passed 2026-06-24 and refreshed zircon_plugins/Cargo.lock for the new navigation/dist workspace member
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_navigation_dist --no-default-features --features dist --offline --locked --target-dir target\codex-plugin-navigation-dist-test: 2 passed, 0 failed on 2026-06-24
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_navigation_runtime --offline --locked --target-dir target\codex-plugin-navigation-runtime-check: passed 2026-06-24 with existing warning noise
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_navigation_runtime navigation_package_manifest_declares_dist_contract --offline --locked --target-dir target\codex-plugin-navigation-runtime-test: blocked before the target test by unrelated `zircon_runtime` UI surface compile drift (`push_pointer_component_events_with_drag_metrics` private method calls and missing `UiRuntimeTreeInteractionExt` import for `scrollable_candidates`); no runtime unit-test pass is claimed
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=0, plugin_skeleton_gate.migration_debt_count=0, standalone_distribution_conformance.dist_capable_plugin_count=5, standalone_distribution_conformance.dist_build_matrix_count=5, standalone_distribution_conformance.dist_capable_plugins=["ai","native_dynamic_fixture","navigation","solari","zr_vm_language"], standalone_distribution_conformance.dist_dependency_boundary_violations=0 on 2026-06-24 after Navigation dist rollout
  - python -m unittest discover -s tools/tests -p test_plugin_standalone_ci_matrix.py: 1 passed, 0 failed on 2026-06-24 after Navigation dist rollout
  - python -m unittest tools.zircon_export.tests.test_plugin_build: 4 passed, 0 failed on 2026-06-24 after Navigation dist rollout
  - python -m py_compile tools\audit_plugin_structure.py tools\zircon_export\cli.py: passed 2026-06-24 after Navigation dist rollout
  - python -m tools.zircon_export plugin build navigation --form dist --repo-root E:\Git\ZirconEngine --out target\codex-plugin-build-navigation-final --target-dir target\codex-plugin-build-navigation-target-final --offline: passed 2026-06-24; emitted `navigation\navigation.dll`, `navigation\native\zircon_plugin_navigation_dist.dll`, `navigation\navigation.sig`, `native_plugins.toml`, and `plugins\native_plugins.toml`
  - rustfmt --edition 2021 zircon_plugins\physics\runtime\src\plugin.rs zircon_plugins\physics\runtime\src\lib.rs zircon_plugins\physics\runtime\src\tests.rs zircon_plugins\physics\dist\src\lib.rs: passed 2026-06-24 after Physics dist rollout
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_physics_dist --no-default-features --features dist --offline --target-dir target\codex-plugin-physics-dist-check: passed 2026-06-24 and refreshed zircon_plugins/Cargo.lock for the new physics/dist workspace member
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_physics_dist --no-default-features --features dist --offline --locked --target-dir target\codex-plugin-physics-dist-test: 2 passed, 0 failed on 2026-06-24
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_physics_runtime --offline --locked --target-dir target\codex-plugin-physics-runtime-check: passed 2026-06-24 with existing warning noise
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_physics_runtime physics_package_manifest_declares_dist_contract --offline --locked --target-dir target\codex-plugin-physics-runtime-test -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-24 with existing warning noise
  - python -m py_compile tools\audit_plugin_structure.py tools\zircon_export\cli.py: passed 2026-06-24 after Physics dist rollout
  - python -m unittest discover -s tools/tests -p test_plugin_standalone_ci_matrix.py: 1 passed, 0 failed on 2026-06-24 after Physics dist rollout
  - python -m unittest tools.zircon_export.tests.test_plugin_build: 4 passed, 0 failed on 2026-06-24 after Physics dist rollout
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=0, plugin_skeleton_gate.migration_debt_count=0, standalone_distribution_conformance.dist_capable_plugin_count=6, standalone_distribution_conformance.dist_build_matrix_count=6, standalone_distribution_conformance.dist_capable_plugins=["ai","native_dynamic_fixture","navigation","physics","solari","zr_vm_language"], standalone_distribution_conformance.dist_dependency_boundary_violations=0 on 2026-06-24 after Physics dist rollout
  - python -m tools.zircon_export plugin build physics --form dist --repo-root E:\Git\ZirconEngine --out target\codex-plugin-build-physics-final --target-dir target\codex-plugin-build-physics-target-final --offline: passed 2026-06-24; emitted `physics\physics.dll`, `physics\native\zircon_plugin_physics_dist.dll`, `physics\physics.sig`, `native_plugins.toml`, and `plugins\native_plugins.toml`
  - rustfmt --edition 2021 zircon_plugins\texture\runtime\src\plugin.rs zircon_plugins\texture\runtime\src\lib.rs zircon_plugins\texture\runtime\src\tests.rs zircon_plugins\texture\dist\src\lib.rs: passed 2026-06-24 after Texture dist rollout
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_texture_dist --no-default-features --features dist --offline --target-dir target\codex-plugin-texture-dist-check: passed 2026-06-24 and refreshed zircon_plugins/Cargo.lock for the new texture/dist workspace member
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_texture_dist --no-default-features --features dist --offline --locked --target-dir target\codex-plugin-texture-dist-test: 2 passed, 0 failed on 2026-06-24
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_texture_runtime --offline --locked --target-dir target\codex-plugin-texture-runtime-check: passed 2026-06-24 with existing warning noise
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_texture_runtime texture_package_manifest_declares_dist_contract --offline --locked --target-dir target\codex-plugin-texture-runtime-test --message-format short --color never -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-24 with existing warning noise after the first too-short timeout was rerun with a larger timeout
  - python -m py_compile tools\audit_plugin_structure.py tools\zircon_export\cli.py: passed 2026-06-24 after Texture dist rollout
  - python -m unittest discover -s tools/tests -p test_plugin_standalone_ci_matrix.py: 1 passed, 0 failed on 2026-06-24 after Texture dist rollout
  - python -m unittest tools.zircon_export.tests.test_plugin_build: 4 passed, 0 failed on 2026-06-24 after Texture dist rollout
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=0, plugin_skeleton_gate.migration_debt_count=0, standalone_distribution_conformance.dist_capable_plugin_count=7, standalone_distribution_conformance.dist_build_matrix_count=7, standalone_distribution_conformance.dist_capable_plugins=["ai","native_dynamic_fixture","navigation","physics","solari","texture","zr_vm_language"], standalone_distribution_conformance.dist_dependency_boundary_violations=0 on 2026-06-24 after Texture dist rollout
  - python -m tools.zircon_export plugin build texture --form dist --repo-root E:\Git\ZirconEngine --out target\codex-plugin-build-texture-final --target-dir target\codex-plugin-build-texture-target-final --offline: passed 2026-06-24; emitted `texture\texture.dll`, `texture\native\zircon_plugin_texture_dist.dll`, `texture\texture.sig`, `native_plugins.toml`, and `plugins\native_plugins.toml`
  - rustfmt --edition 2021 zircon_plugins\particles\runtime\src\plugin.rs zircon_plugins\particles\runtime\src\lib.rs zircon_plugins\particles\runtime\src\tests\mod.rs zircon_plugins\particles\runtime\src\tests\package_manifest.rs zircon_plugins\particles\dist\src\lib.rs: passed 2026-06-24 after Particles dist rollout
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_particles_dist --no-default-features --features dist --offline --target-dir target\codex-plugin-particles-dist-check: passed 2026-06-24 and refreshed zircon_plugins/Cargo.lock for the new particles/dist workspace member
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_particles_dist --no-default-features --features dist --offline --locked --target-dir target\codex-plugin-particles-dist-test: 2 passed, 0 failed on 2026-06-24
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_particles_runtime --offline --locked --target-dir target\codex-plugin-particles-runtime-check: passed 2026-06-24 with existing warning noise
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_particles_runtime particles_package_manifest_declares_dist_contract --offline --locked --target-dir target\codex-plugin-particles-runtime-test --message-format short --color never -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-24 with existing warning noise after the first too-short timeout was rerun with a larger timeout
  - python -m py_compile tools\audit_plugin_structure.py tools\zircon_export\cli.py: passed 2026-06-24 after Particles dist rollout
  - python -m unittest discover -s tools/tests -p test_plugin_standalone_ci_matrix.py: 1 passed, 0 failed on 2026-06-24 after Particles dist rollout
  - python -m unittest tools.zircon_export.tests.test_plugin_build: 4 passed, 0 failed on 2026-06-24 after Particles dist rollout
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=0, plugin_skeleton_gate.migration_debt_count=0, standalone_distribution_conformance.dist_capable_plugin_count=8, standalone_distribution_conformance.dist_build_matrix_count=8, standalone_distribution_conformance.dist_capable_plugins=["ai","native_dynamic_fixture","navigation","particles","physics","solari","texture","zr_vm_language"], standalone_distribution_conformance.dist_dependency_boundary_violations=0 on 2026-06-24 after Particles dist rollout
  - python -m tools.zircon_export plugin build particles --form dist --repo-root E:\Git\ZirconEngine --out target\codex-plugin-build-particles-final --target-dir target\codex-plugin-build-particles-target-final --offline: passed 2026-06-24; emitted `particles\particles.dll`, `particles\native\zircon_plugin_particles_dist.dll`, `particles\particles.sig`, `native_plugins.toml`, and `plugins\native_plugins.toml`
  - rustfmt --edition 2021 zircon_plugins\animation\runtime\src\plugin.rs zircon_plugins\animation\runtime\src\lib.rs zircon_plugins\animation\runtime\src\tests.rs zircon_plugins\animation\dist\src\lib.rs: passed 2026-06-24 after Animation dist rollout
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_animation_dist --no-default-features --features dist --offline --target-dir target\codex-plugin-animation-dist-check --message-format short --color never: passed 2026-06-24 and refreshed zircon_plugins/Cargo.lock for the new animation/dist workspace member
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_animation_dist --no-default-features --features dist --offline --locked --target-dir target\codex-plugin-animation-dist-test --message-format short --color never -- --test-threads=1 --nocapture: 2 passed, 0 failed on 2026-06-24
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_animation_runtime --offline --locked --target-dir target\codex-plugin-animation-runtime-check --message-format short --color never: passed 2026-06-24 with existing warning noise after the first too-short timeout was rerun
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_animation_runtime animation_package_manifest_declares_dist_contract --offline --locked --target-dir target\codex-plugin-animation-runtime-test --message-format short --color never -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-24 with existing warning noise
  - python -m py_compile tools\audit_plugin_structure.py tools\zircon_export\cli.py tools\zircon_export\plugin_build.py: passed 2026-06-24 after Animation dist rollout
  - python -m unittest discover -s tools/tests -p test_plugin_standalone_ci_matrix.py: 1 passed, 0 failed on 2026-06-24 after Animation dist rollout
  - python -m unittest tools.zircon_export.tests.test_plugin_build: 4 passed, 0 failed on 2026-06-24 after Animation dist rollout
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=0, plugin_skeleton_gate.migration_debt_count=0, standalone_distribution_conformance.dist_capable_plugin_count=9, standalone_distribution_conformance.dist_build_matrix_count=9, standalone_distribution_conformance.dist_capable_plugins=["ai","animation","native_dynamic_fixture","navigation","particles","physics","solari","texture","zr_vm_language"], standalone_distribution_conformance.dist_dependency_boundary_violations=0 on 2026-06-24 after Animation dist rollout
  - python -m tools.zircon_export plugin build animation --form dist --repo-root E:\Git\ZirconEngine --out target\codex-plugin-build-animation-final --target-dir target\codex-plugin-build-animation-target-final --offline: passed 2026-06-24; emitted `animation\animation.dll`, `animation\native\zircon_plugin_animation_dist.dll`, `animation\animation.sig`, `native_plugins.toml`, and `plugins\native_plugins.toml`
  - rustfmt --edition 2021 zircon_plugins\hybrid_gi\runtime\src\plugin.rs zircon_plugins\hybrid_gi\runtime\src\tests.rs zircon_plugins\hybrid_gi\dist\src\lib.rs: passed 2026-06-24 after Hybrid GI dist rollout
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_dist --no-default-features --features dist --offline --target-dir target\codex-plugin-hybrid-gi-dist-check --message-format short --color never: passed 2026-06-24 and refreshed zircon_plugins/Cargo.lock for the new hybrid_gi/dist workspace member
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_dist --no-default-features --features dist --offline --locked --target-dir target\codex-plugin-hybrid-gi-dist-test --message-format short --color never -- --test-threads=1 --nocapture: 2 passed, 0 failed on 2026-06-24
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime hybrid_gi_package_manifest_declares_dist_contract --offline --locked --target-dir target\codex-plugin-hybrid-gi-runtime-test --message-format short --color never -- --test-threads=1 --nocapture: blocked before target test by unrelated `zircon_runtime` render handle/value semantics compile drift; no runtime unit-test pass is claimed
  - python -m py_compile tools\audit_plugin_structure.py tools\zircon_export\cli.py tools\zircon_export\plugin_build.py: passed 2026-06-24 after Hybrid GI dist rollout
  - python -m unittest discover -s tools/tests -p test_plugin_standalone_ci_matrix.py: 1 passed, 0 failed on 2026-06-24 after Hybrid GI dist rollout
  - python -m unittest tools.zircon_export.tests.test_plugin_build: 4 passed, 0 failed on 2026-06-24 after Hybrid GI dist rollout
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=0, plugin_skeleton_gate.migration_debt_count=0, standalone_distribution_conformance.dist_capable_plugin_count=10, standalone_distribution_conformance.dist_build_matrix_count=10, standalone_distribution_conformance.dist_capable_plugins=["ai","animation","hybrid_gi","native_dynamic_fixture","navigation","particles","physics","solari","texture","zr_vm_language"], standalone_distribution_conformance.dist_dependency_boundary_violations=0 on 2026-06-24 after Hybrid GI dist rollout
  - python -m tools.zircon_export plugin build hybrid_gi --form dist --repo-root E:\Git\ZirconEngine --out target\codex-plugin-build-hybrid-gi --target-dir target\codex-plugin-build-hybrid-gi-target --offline: passed 2026-06-24; emitted `hybrid_gi\hybrid_gi.dll`, `hybrid_gi\native\zircon_plugin_hybrid_gi_dist.dll`, `hybrid_gi\hybrid_gi.sig`, `native_plugins.toml`, and `plugins\native_plugins.toml`
  - rustfmt --edition 2021 zircon_plugins\terrain\runtime\src\plugin.rs zircon_plugins\terrain\runtime\src\lib.rs zircon_plugins\terrain\runtime\src\tests.rs zircon_plugins\terrain\dist\src\lib.rs: passed 2026-06-24 after Terrain dist rollout
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_terrain_dist --no-default-features --features dist --offline --target-dir target\codex-plugin-terrain-dist-check --message-format short --color never: passed 2026-06-24 and refreshed zircon_plugins/Cargo.lock for the new terrain/dist workspace member
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_terrain_dist --no-default-features --features dist --offline --locked --target-dir target\codex-plugin-terrain-dist-test --message-format short --color never: 2 passed, 0 failed on 2026-06-24
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_terrain_runtime terrain_package_manifest_declares_dist_contract --offline --locked --target-dir target\codex-plugin-terrain-runtime-test --message-format short --color never -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-24 with existing warning noise after waiting for the first too-broad runtime test attempt to finish
  - python -m py_compile tools\audit_plugin_structure.py tools\zircon_export\cli.py tools\zircon_export\plugin_build.py: passed 2026-06-24 after Terrain dist rollout
  - python -m unittest discover -s tools/tests -p test_plugin_standalone_ci_matrix.py: 1 passed, 0 failed on 2026-06-24 after Terrain dist rollout
  - python -m unittest tools.zircon_export.tests.test_plugin_build: 4 passed, 0 failed on 2026-06-24 after Terrain dist rollout
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=0, plugin_skeleton_gate.migration_debt_count=0, standalone_distribution_conformance.dist_capable_plugin_count=11, standalone_distribution_conformance.dist_build_matrix_count=11, standalone_distribution_conformance.dist_capable_plugins=["ai","animation","hybrid_gi","native_dynamic_fixture","navigation","particles","physics","solari","terrain","texture","zr_vm_language"], standalone_distribution_conformance.dist_dependency_boundary_violations=0 on 2026-06-24 after Terrain dist rollout
  - python -m tools.zircon_export plugin build terrain --form dist --repo-root E:\Git\ZirconEngine --out target\codex-plugin-build-terrain-final --target-dir target\codex-plugin-build-terrain-target-final --offline: passed 2026-06-24; emitted `terrain\terrain.dll`, `terrain\native\zircon_plugin_terrain_dist.dll`, `terrain\terrain.sig`, `native_plugins.toml`, and `plugins\native_plugins.toml`
doc_type: module-detail
status: in_progress
---

# 插件独立构建与分发规范（Plugin Standalone Build & Distribution）

> 本文是 ZirconEngine 插件**独立构建与可分发动态包**的唯一规范权威，由 [Plugins 13](../plans/zircon_plugins/13-standalone-plugin-build.md) 落地、[引擎结构规范 §6](../plans/engine-code-structure-convention.md) 引用。与 [`plugin-manifest-schema.md`](plugin-manifest-schema.md)（manifest schema）、[`plugin-crate-skeleton.md`](plugin-crate-skeleton.md)（crate 骨架）、[`plugin-sdk.md`](plugin-sdk.md)（SDK API）配套。
>
> 状态：in_progress（规范定稿；2026-06-23 已落 Plugins 13 M1 首个 dist 代表插件 + 依赖边界 guard、M2/T1 registration manifest ABI schema、M2/T2 system bridge replay 代码路径、M2/T3 `dist` entry helper、M3/T1 `plugin build <id>` 单插件构建入口、M3/T2 `[distribution].forms` carrier 判定、M3/T3 可重复包输出 guard、M4/T1 per-plugin zrpack focused guard、M4/T2 loader distribution compatibility gate、M4/T3 签名/hash + load manifest 汇编、M5/T2 当前 dist-capable CI matrix seed、M5/T3 `RuntimePluginId` string-newtype，以及 M5/T1 importer skeleton/capability-owner、runtime-only skeleton owner、editor-only skeleton owner、authoring runtime/editor skeleton owner、particles/physics/texture skeleton owner、editor_build_export_desktop skeleton owner、sound runtime/editor feature skeleton owner、timeline_sequence editor skeleton owner、最终 8 根 owner rollout、Solari first-party dist rollout、AI first-party dist rollout、ZrVM Language first-party dist rollout、2026-06-24 Navigation first-party dist rollout、Physics first-party dist rollout、Texture first-party dist rollout、Particles first-party dist rollout、Animation first-party dist rollout 与 Hybrid GI first-party dist rollout；当前 skeleton debt 为 0、`migration_debt_roots = []`、`dist_capable_plugin_count = 10`（`ai` + `animation` + `hybrid_gi` + `native_dynamic_fixture` + `navigation` + `particles` + `physics` + `solari` + `texture` + `zr_vm_language`）。M2/T2 focused runtime test 仍被无关 runtime lib-test compile drift 阻塞，M4/T1 真实 Cargo pack smoke 超时未计通过，M4/T2 focused lib-test、M5/T3 focused runtime lib-test、ZrVM Language runtime focused unit test、Navigation runtime focused unit test 与 Hybrid GI runtime focused unit test 未计通过；全量 dist-capable 扩容仍以后续发行切片推进）

## 1. 设计原则

- **双形态单源**：一份插件声明（manifest + backend 逻辑）投影两种产物——`embed`（in-tree `rlib`，静态链接，零 FFI）与 `dist`（`cdylib`，ABI-only，可分发可热更）。两形态不复制逻辑。
- **依赖边界硬约束**：`dist` 产物的依赖闭包**只含稳定 ABI**（`zircon_plugin_sdk` → `zircon_runtime_interface`），**禁含 `zircon_runtime` / `zircon_editor` / `zircon_app` / `wgpu` / `slint` / `winit`**（与结构规范 §7.5 E8 边界白名单同源）。可独立于引擎源码树编译。
- **稳定 ABI 唯一通道**：跨 cdylib 边界只传 ABI-safe 值与序列化字节（ABI v3 `repr(C)` 表 + TOML/字节 payload），不传 Rust trait object、wgpu/slint 对象、runtime 世界引用。
- **可重复构建**：同输入（lockfile + 源 + 资产）产出 byte 相同 cdylib 与 zrpack；时间戳清零、路径归一。
- **兼容性显式协商**：产物钉 `abi_version` 与 `engine_compat`；loader 加载期校验，不匹配出结构化诊断而非崩溃。

## 2. 产物形态

| 形态 | crate-type | 依赖 | 注册路径 | 用途 |
|---|---|---|---|---|
| `embed` | `rlib` | `zircon_runtime`（path，behind `embed` feature） | `impl RuntimePlugin::register` + `plugin_sdk::registration` builder | LibraryEmbed 静态链接，发行期性能优化 |
| `dist` | `cdylib` | `zircon_plugin_sdk`(`native`) + `zircon_runtime_interface` | ABI v3 导出（`zircon_native_plugin_descriptor_v3` + entry） | NativeDynamic 可分发包、热更插件 |

- 默认 feature = `embed`；`dist` 形态以 `--no-default-features --features dist` 构建。
- 单 crate `crate-type = ["rlib", "cdylib"]` + feature-gated `zircon_runtime`（`optional = true`）为**首选**；逻辑无法干净 feature-gate 时退化为独立 `<plugin>/dist/` cdylib crate 包裹 `backend/`（fallback）。

## 3. crate 骨架（发行维扩展）

在 [`plugin-crate-skeleton.md`](plugin-crate-skeleton.md) 骨架基础上：

```
<plugin>/runtime/
  Cargo.toml
    # crate-type = ["rlib", "cdylib"]
    # [dependencies] zircon_plugin_sdk = { workspace = true, default-features = false }
    #                zircon_runtime = { path = "...", optional = true }
    # [features] default = ["embed"]
    #            embed = ["dep:zircon_runtime", "zircon_plugin_sdk/runtime"]
    #            dist  = ["zircon_plugin_sdk/native"]
  src/
    lib.rs           # 薄 façade
    plugin.rs        # #[cfg(feature="embed")] impl RuntimePlugin::register
    dist.rs          # #[cfg(feature="dist")]  ABI v3 导出 owner（SDK 宏）
    capability.rs    # capability 单源（禁 use zircon_runtime）
    backend/         # 纯逻辑：仅 zircon_plugin_sdk + zircon_runtime_interface
    systems/         # embed 注册进调度图；dist 经 §6 编组
    tests/
```

- 铁律：`backend/`、`capability.rs` 禁 `use zircon_runtime::*`；触碰 `zircon_runtime` 的代码必须 `#[cfg(feature = "embed")]`。

## 4. manifest `[distribution]` 段

`plugin.toml` 新增可选段（schema 详见 [`plugin-manifest-schema.md`](plugin-manifest-schema.md) §3）：

```toml
[distribution]
forms = ["embed", "dist"]                 # 该插件支持的产物形态
default_packaging = ["library_embed", "native_dynamic"]
abi_version = 3                            # 与 ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 钉
engine_compat = ">=0.1, <0.2"             # 引擎兼容区间
dist_crate = "zircon_plugin_<plugin>_runtime"
descriptor_symbol = "zircon_native_plugin_descriptor_v3"
runtime_entry = "zircon_plugin_<plugin>_runtime_entry_v3"
editor_entry  = "zircon_plugin_<plugin>_editor_entry_v3"   # 可选
assets = ["assets/**"]                     # 随包资产（进 per-plugin zrpack 子包）
```

- 非 native 插件的 `[distribution]` 由 descriptor `package_manifest()` 投影（`@generated`），不手写漂移（延续 12-M1 生成纪律）。

## 5. 产物包布局

`zircon plugin build <id> --form dist` 产出：

```
plugins/<id>/
  <id>.{dll|so|dylib}            # cdylib，导出 descriptor 符号 + entry
  plugin.toml                     # 包级 manifest（@generated 投影）
  native_dynamic_package.toml     # ABI v3 契约报告（native_dynamic_package_plan.rs 格式）
  <id>.zrpack                     # 可选：随包资产子包（内容寻址 chunk，09 zrpack 格式）
  <id>.sig                        # hash sidecar；外部签名启用时记录 signer before/after hash
```

- 目录名由 `package_id` 净化（`native_dynamic_package_directory`：非 `[A-Za-z0-9_-]` → `_`），冲突出诊断。
- loader 经 `NativePluginLoadManifest`（`{ id, path, manifest, package_report, abi }`）收集 `plugins/<id>/`，按 ABI v3 契约加载。

## 6. 注册跨 ABI 编组（dist 形态）

`embed` 形态直接调 runtime registry；`dist` 形态把注册意图序列化、由宿主回放（依赖 Plugins 01 register 通道 + 11 bridge dense 通道）：

- **声明序列化**：dist 插件导出 registration manifest（module / system anchors + 读写访问集 / resources / events / 扩展点贡献 / capability），经 `NativePluginEntryReportV3.package_manifest_toml` + registration 段（ABI-safe TOML/字节）承载。
- **当前 ABI 承载（M2/T1）**：ABI v3 的 `NativePluginSchemaVersionsV3.registration_manifest_schema` 声明 registration manifest schema，`NativePluginBehaviorV3.registration_manifest` 承载 TOML 文本。SDK 提供 `NativePluginRegistrationManifestV3` 与 `registration_manifest_v3_to_toml` / `registration_manifest_v3_from_toml`，loader 读取后在行为验证报告里输出 `registration_manifest_schema` 与 `has_registration_manifest`。当前 schema id 为 `zircon.native.registration-manifest/3`。
- **当前回放实现（M2/T2）**：runtime loader 解析 `zircon.native.registration-manifest/3` TOML，live host 通过 `replay_runtime_registration_manifests_via_bridge(...)` 把 dist 插件声明的 system anchor 注册为宿主拥有的 conservative native system。system tick 捕获 `NativeHostBridgeCallScope`，按 package manifest 的 bridge method slot 与 `RuntimePluginBridgeLifecycleState` 的 dense interface slot 调插件方法；不跨 FFI 传裸函数指针进调度。当前 focused test 已实现但被无关 runtime lib-test 编译漂移阻塞，未计通过。
- **当前导出 helper（M2/T3）**：`zircon_plugin_sdk::dist` 提供 `native_dist_plugin_v3!` 与 `native_dist_runtime_plugin_v3!`，从一份 crate-local 声明生成 descriptor、entry report、behavior schema/manifest、bridge method table、capability gating 与 ABI symbol export。`native_dynamic_fixture` 已改用该 helper，runtime `tick` bridge method 由 package manifest `provides_interfaces` + dist macro bridge table 双侧声明。
- **当前独立构建入口（M3/T1）**：`tools/zircon_export/plugin_build.py` 提供 `python -m tools.zircon_export plugin build <id>`，从 `[distribution]` 读取 `dist_crate` 和 ABI 约束，使用独立 target dir 构建 `dist` feature，并物化 `<out>/<id>/` 包目录；该入口是 per-plugin 切片，不经过 profile 级 `<out>/stages`。
- **当前 carrier 判定（M3/T2）**：`tools/zircon_build.py` 在 profile 级插件构建中读取 `[distribution].forms`；`dist` 映射 `native_dynamic`，`embed` 映射 `rlib_static`，`dist_crate` 精确选择动态 crate。未迁移、未声明 forms 的旧清单暂保留 crate-type 回退，避免全量 rollout 前排除现有插件。
- **当前可重复构建 guard（M3/T3）**：`tools/zircon_export/tests/test_plugin_build.py::test_plugin_dist_build_is_byte_reproducible` 对同一输入双跑 `plugin build <id>` 并比较包目录内相对路径到 bytes 的完整映射；真实 `native_dynamic_fixture` 双跑到两个 out 目录后，loadable cdylib、`plugin.toml` 与 `native_dynamic_package.toml` sha256 一致。
- **当前资产子包（M4/T1）**：`plugin build <id>` 读取 `[distribution].assets`，生成 Pack 阶段输入清单，调用 `zircon_export_pack`（或 `--packer` 指向的预构建 packer）写入 `<id>.zrpack`，随后由 `native_dynamic_package.toml` payload manifest 收录该子包。`native_dynamic_fixture` 已声明 `assets = ["assets/**"]` 并带最小 WGSL fixture；focused Python guard 通过，真实 Cargo pack smoke 当前因 packer 构建超时未计通过。
- **当前兼容性协商（M4/T2）**：`PluginPackageManifest` 已 typed 承载 `[distribution]`，native loader 在探测库文件前检查 `forms`、`abi_version` 与 `engine_compat`。不含 `dist`、ABI 缺失/不等于 v3、engine range 为空/格式错误/不包含当前引擎版本时，loader 写诊断并跳过插件，不继续报缺库。production `core-min` 编译通过；focused lib-test 受无关 runtime test-tree 编译漂移阻塞，未计通过。
- **当前签名/hash 与 load manifest（M4/T3）**：`plugin build <id>` 复用 `native_signing.py` 的签名前置语义，支持 `--sign-command` / `--sign-arg` / `--sign-profile` / `--sign-platform`。包内总是生成 `<id>.sig` hash sidecar，记录 loadable artifact bytes/sha256；启用外部 signer 时记录 before/after sha256。命令还会在 `<out>/native_plugins.toml` 汇编 `NativePluginLoadManifest` 条目，并让 `native_dynamic_package.toml` payload 收录签名后的 loadable artifact 与 `.sig`。
- **当前插件 ID 形态（M5/T3）**：`RuntimePluginId` 已由封闭 enum 改为开放 string-newtype。内建一方插件继续使用 `RuntimePluginId::Ui` 等关联常量，第三方/独立插件的合法 key 通过 `RuntimePluginId::parse_key(...)` / `RuntimePluginId::new(...)` 承载并以字符串序列化；loader 对未知合法 runtime plugin id 走 externalized diagnostic fallback，不要求修改 engine core。
- **当前存量骨架迁移（M5/T1 子切片）**：7 个 split importer 与 `asset_importers/{data,model,shader}` 已把 capability 常量迁入 `runtime/src/capability.rs`，`ai`、`solari`、`zr_vm_language` 与 `asset_importers/{audio,texture}` 已迁入 runtime-only `plugin.rs`/`capability.rs` owner 骨架，`native_window_hosting`、`runtime_diagnostics`、`ui_asset_authoring` 已迁入 editor-only `capability.rs`/`extension_ids.rs`/`plugin.rs`/`tests.rs` owner 骨架，`prefab_tools`、`terrain`、`tilemap_2d` 已迁入 authoring runtime/editor `plugin.rs`/`authoring.rs`/`tests.rs` owner 骨架，`particles`、`physics`、`texture` 已迁入 runtime/editor skeleton owner 骨架，`editor_build_export_desktop` 已迁入 export/editor skeleton owner 骨架，`sound` main runtime/editor 与两个 feature runtime/editor crate 已迁入 skeleton owner 骨架，`timeline_sequence` editor crate 已迁入 skeleton owner 骨架，最后 `animation`、`animation_graph`、`hybrid_gi`、`material_editor`、`navigation`、`net`、`rendering`、`virtual_geometry` 已完成 owner rollout；`plugin_skeleton_gate.migration_debt_count = 0`。该历史切片只降低后续 dual-form 拆分阻力，不新增 dist-capable 插件；当前 dist-capable 计数已由 Solari、AI、ZrVM Language、Navigation、Physics、Texture、Particles、Animation、Hybrid GI 与 Terrain rollout 更新为 11。
- **当前真实一方 dist rollout（M5/T1）**：`solari`、`ai`、`zr_vm_language`、`navigation`、`physics`、`texture`、`particles`、`animation`、`hybrid_gi` 与 `terrain` 已新增 fallback `<plugin>/dist/` cdylib crate（`zircon_plugin_solari_dist` / `zircon_plugin_ai_dist` / `zircon_plugin_zr_vm_language_dist` / `zircon_plugin_navigation_dist` / `zircon_plugin_physics_dist` / `zircon_plugin_texture_dist` / `zircon_plugin_particles_dist` / `zircon_plugin_animation_dist` / `zircon_plugin_hybrid_gi_dist` / `zircon_plugin_terrain_dist`），使用 `native_dist_runtime_plugin_v3!` 导出 ABI v3 runtime entry 和 registration manifest；各自 `plugin.toml` 与 runtime package manifest 均声明 `[distribution]`、native module、NativeDynamic default packaging、ABI v3、engine compatibility、descriptor/runtime entry 与 runtime capabilities。结构审计与 CI matrix 当前覆盖 `ai` + `animation` + `hybrid_gi` + `native_dynamic_fixture` + `navigation` + `particles` + `physics` + `solari` + `terrain` + `texture` + `zr_vm_language`，`dist_capable_plugin_count = 11`，`dist_build_matrix_count = 11`。
- **行为编组**：
  - command/bridge 型 → `invoke_command` + `NativePluginBridgeMethodTableV3`（宿主调用插件方法）。
  - system 型 → 插件声明 system anchor + `SystemParamAccess`；宿主据此在调度图占位，tick 时经 bridge 回调插件执行体。不跨 FFI 传裸函数指针进调度。
- **panic 边界**：所有 `extern "C"` 边界（出站导出 + 入站 host 回调）必须 panic guard（`catch_native_callback_panic`），panic 转状态码不跨 FFI（结构规范 §7.5 E7）。

## 7. 构建命令契约

```bash
# 单插件独立构建（独立 target dir、独立产物目录，不全量编译 workspace）
python -m tools.zircon_export plugin build <id> \
    --form dist --platform <triple> --mode release \
    --out <out>/plugins --target-dir <isolated-target-dir>
```

- 复用 `tools/zircon_export/native_build.py`（真编译）+ `native_signing.py`（签名）+ zrpack writer（09-M2）；profile 级整包导出（09）调用同一底座。
- `tools/zircon_build.py` 的 carrier 形态判定（`native_dynamic`/`rlib_static`）已升级为读 `[distribution].forms`；仅无 forms 的 legacy manifest 使用 crate-type 回退。
- 当前 M3/T1-M4/T3 实现已支持 `--repo-root`、`--out`、`--target-dir`、`--platform`、`--mode`、重复 `--build-feature`、`--offline`、`--no-locked`、`--dry-run`、`--sign-command`、重复 `--sign-arg`、`--sign-profile` 与重复 `--sign-platform`。成功输出 `<out>/<id>/{<id>.dll|so|dylib, native/<dist_crate>.dll|so|dylib, plugin.toml, native_dynamic_package.toml, <id>.sig, <id>.zrpack?}`、`<out>/native_plugins.toml`、`<out>/plugins/native_plugins.toml` 和 JSON 报告；M5 全量 rollout 仍在后续切片。

## 8. 校验器与 guard（Plugins 13）

- 依赖边界：`tools/plugin_structure_audits/dependency_boundary.py` 解析 `[distribution]`、workspace crate manifest 与 `dist` feature，字段 `dist_dependency_boundary_violations`（→ 0）、`distribution_section_violations`（→ 0）、`dist_capable_plugin_count`。当前 guard 已覆盖 `native_dynamic_fixture`、`solari`、`ai`、`zr_vm_language`、`navigation`、`physics`、`texture`、`particles` 与 `animation` 的独立 `dist` 形态；里程碑末继续叠加 per-plugin cargo metadata 闭包核验。
- `[distribution]` 段一致性：当前由 `plugins_13_dist_dependency_boundary_clean` 同时锁定首个 dist manifest 形状；全量 rollout 后拆/扩为 `plugins_13_distribution_section_uniform`。
- registration manifest schema：`native_dynamic_registration_manifest_round_trips` 锁定 SDK TOML round-trip；runtime loader validation 读取 `registration_manifest_schema` / `registration_manifest` 并在 schema 不匹配时标记行为 invalid。当前 runtime focused test 被无关 runtime lib-test compile drift 阻塞，未计通过。
- system bridge replay：`dist_system_plugin_loads_and_ticks_via_bridge` 锁定 load report 自动安装 bridge binding、registration manifest 回放进 `RuntimeExtensionRegistry`、world 应用与 native system tick 经 bridge 调用插件方法。当前本切片编译错误已清除，但 runtime lib-test 被无关 `test_file_budget.rs` missing modules 阻塞，未计通过。
- dist entry helper：`dist_plugin_one_file_export_compiles` 锁定 `plugin_sdk::dist` 一文件导出宏会生成 descriptor、runtime/editor entry、registration manifest pointer 和 bridge method table；`native_dynamic_fixture` 的 `dist` cargo check 锁定真实 cdylib fixture 仍能独立编译。
- per-plugin build：`plugin_build_emits_isolated_package_dir` 锁定单插件 CLI 会使用独立 target dir、dist feature 和 `<out>/<id>` 包目录，并写出 loadable cdylib、`plugin.toml`、`native_dynamic_package.toml`，同时不创建 profile 级 `<out>/stages`。
- carrier 判定：`zircon_build_classifies_forms_from_manifest` 锁定 `tools/zircon_build.py` 以 `[distribution].forms` 判定 `dist`/`embed` 对应的 `native_dynamic`/`rlib_static` carrier，并保留无 forms legacy manifest 的 crate-type 回退。
- 可重复构建：`plugin_dist_build_is_byte_reproducible`（双跑 byte 比对）已覆盖 fake cargo 单元测试与真实 `native_dynamic_fixture` 包 sha256 比对。
- per-plugin zrpack：`plugin_build_includes_plugin_zrpack_asset_subpackage` 锁定 `[distribution].assets` 会生成 pack 输入清单、调用 pack writer，并把 `<id>.zrpack` 作为 package payload 文件写入 `native_dynamic_package.toml`。
- 兼容性协商：`native_loader_skips_distribution_with_incompatible_engine_range_before_library_probe` 与 `distribution_diagnostic_rejects_unsupported_abi_version` 锁定 loader 在探测库路径前拒绝 incompatible distribution contract；当前 production `core-min` check 通过，focused lib-test 被无关 runtime test-tree 编译漂移阻塞，未计通过。
- CI matrix：`test_plugin_standalone_ci_matrix.py` 比对 `.github/workflows/ci.yml` 的 `plugin-standalone-dist` job matrix 与 `standalone_distribution_conformance.dist_build_matrix_entries`，当前覆盖 `native_dynamic_fixture`、`ai`、`animation`、`navigation`、`particles`、`physics`、`solari`、`texture` 与 `zr_vm_language`，并在新增 dist-capable 插件后要求同步 CI。
- runtime plugin id：`runtime_plugin_id_accepts_external_keys_without_core_variant` 锁定第三方 key 不再需要 core enum 分支；`runtime_module_assembly_keeps_specialized_flows_in_child_owners` 的源码守卫拒绝 `RuntimePluginId` 回退到 enum。当前 production `core-min` check 通过，focused lib-test 被无关 runtime test-tree 编译漂移阻塞，未计通过。
- M5/T1 skeleton debt：`tools/audit_plugin_structure.py --json` 的 `skeleton_conformance.migration_debt_count` 与 `plugin_skeleton_gate.migration_debt_count` 当前为 0，`migration_debt_roots = []`；当前真实 dist-capable 插件为 `ai`、`animation`、`native_dynamic_fixture`、`navigation`、`particles`、`physics`、`solari`、`texture` 与 `zr_vm_language` 九项，全量 dual-form rollout 仍未关闭。
- 与 12 四源一致性 guard（plugin.toml / capability.rs / descriptor / workspace member）联合执行。

## 9. 当前落地状态

| 日期 | 范围 | 状态 | 证据 |
|---|---|---|---|
| 2026-06-24 | Plugins 13 M5/T1 Terrain first-party dist rollout | plugins_13_m5_t1_terrain_dist_rollout | `terrain` 已具备独立 dist 形态：新增 `terrain/dist` cdylib crate，`terrain/plugin.toml` 与 runtime package manifest 声明 `[distribution]`、native module、ABI v3、engine compatibility、descriptor/runtime entry 和 NativeDynamic default packaging；CI matrix 同步新增 `terrain`。`plugin build terrain --form dist` 输出根 loadable、`native/<dist_crate>` loader 兼容副本、根 `native_plugins.toml`、`plugins/native_plugins.toml` 与 `.sig` sidecar。验证：scoped rustfmt、py_compile、`test_plugin_build` 4/4、CI matrix unittest 1/1、audit JSON、Terrain dist check/test、Terrain runtime focused test、真实 `plugin build terrain` smoke 均通过；audit 报告 `dist_capable_plugin_count = 11`、`dist_build_matrix_count = 11`、`dist_capable_plugins = ["ai", "animation", "hybrid_gi", "native_dynamic_fixture", "navigation", "particles", "physics", "solari", "terrain", "texture", "zr_vm_language"]`、`dist_dependency_boundary_violations = 0`。 |
| 2026-06-24 | Plugins 13 M5/T1 Hybrid GI first-party dist rollout | plugins_13_m5_t1_hybrid_gi_dist_rollout | `hybrid_gi` 已具备独立 dist 形态：新增 `hybrid_gi/dist` cdylib crate，`hybrid_gi/plugin.toml` 与 runtime package manifest 声明 `[distribution]`、native module、ABI v3、engine compatibility、descriptor/runtime entry 和 NativeDynamic default packaging；CI matrix 同步新增 `hybrid_gi`。`plugin build hybrid_gi --form dist` 输出根 loadable、`native/<dist_crate>` loader 兼容副本、根 `native_plugins.toml`、`plugins/native_plugins.toml` 与 `.sig` sidecar。验证：scoped rustfmt、py_compile、`test_plugin_build` 4/4、CI matrix unittest 1/1、audit JSON、Hybrid GI dist check/test、真实 `plugin build hybrid_gi` smoke 均通过；audit 报告 `dist_capable_plugin_count = 10`、`dist_build_matrix_count = 10`、`dist_capable_plugins = ["ai", "animation", "hybrid_gi", "native_dynamic_fixture", "navigation", "particles", "physics", "solari", "texture", "zr_vm_language"]`、`dist_dependency_boundary_violations = 0`。runtime focused unit test 被无关 `zircon_runtime` render handle/value semantics 编译漂移阻塞，未计通过。 |
| 2026-06-24 | Plugins 13 M5/T1 Animation first-party dist rollout | plugins_13_m5_t1_animation_dist_rollout | `animation` 已具备独立 dist 形态：新增 `animation/dist` cdylib crate，`animation/plugin.toml` 与 runtime package manifest 声明 `[distribution]`、native module、ABI v3、engine compatibility、descriptor/runtime entry 和 NativeDynamic default packaging；CI matrix 同步新增 `animation`。`plugin build animation --form dist` 输出根 loadable、`native/<dist_crate>` loader 兼容副本、根 `native_plugins.toml`、`plugins/native_plugins.toml` 与 `.sig` sidecar。验证：scoped rustfmt、py_compile、`test_plugin_build` 4/4、CI matrix unittest 1/1、audit JSON、Animation dist check/test、Animation runtime check、Animation runtime focused test、真实 `plugin build animation` smoke 均通过；audit 报告 `dist_capable_plugin_count = 9`、`dist_build_matrix_count = 9`、`dist_capable_plugins = ["ai", "animation", "native_dynamic_fixture", "navigation", "particles", "physics", "solari", "texture", "zr_vm_language"]`、`dist_dependency_boundary_violations = 0`。 |
| 2026-06-24 | Plugins 13 M5/T1 Particles first-party dist rollout | plugins_13_m5_t1_particles_dist_rollout | `particles` 已具备独立 dist 形态：新增 `particles/dist` cdylib crate，`particles/plugin.toml` 与 runtime package manifest 声明 `[distribution]`、native module、ABI v3、engine compatibility、descriptor/runtime entry 和 NativeDynamic default packaging；CI matrix 同步新增 `particles`。`plugin build particles --form dist` 输出根 loadable、`native/<dist_crate>` loader 兼容副本、根 `native_plugins.toml`、`plugins/native_plugins.toml` 与 `.sig` sidecar。验证：scoped rustfmt、py_compile、`test_plugin_build` 4/4、CI matrix unittest 1/1、audit JSON、Particles dist check/test、Particles runtime check、Particles runtime focused test、真实 `plugin build particles` smoke 均通过；audit 报告 `dist_capable_plugin_count = 8`、`dist_build_matrix_count = 8`、`dist_capable_plugins = ["ai", "native_dynamic_fixture", "navigation", "particles", "physics", "solari", "texture", "zr_vm_language"]`、`dist_dependency_boundary_violations = 0`。 |
| 2026-06-24 | Plugins 13 M5/T1 Texture first-party dist rollout | plugins_13_m5_t1_texture_dist_rollout | `texture` 已具备独立 dist 形态：新增 `texture/dist` cdylib crate，`texture/plugin.toml` 与 runtime package manifest 声明 `[distribution]`、native module、ABI v3、engine compatibility、descriptor/runtime entry 和 NativeDynamic default packaging；CI matrix 同步新增 `texture`。`plugin build texture --form dist` 输出根 loadable、`native/<dist_crate>` loader 兼容副本、根 `native_plugins.toml`、`plugins/native_plugins.toml` 与 `.sig` sidecar。验证：scoped rustfmt、py_compile、`test_plugin_build` 4/4、CI matrix unittest 1/1、audit JSON、Texture dist check/test、Texture runtime check、Texture runtime focused test、真实 `plugin build texture` smoke 均通过；audit 报告 `dist_capable_plugin_count = 7`、`dist_build_matrix_count = 7`、`dist_capable_plugins = ["ai", "native_dynamic_fixture", "navigation", "physics", "solari", "texture", "zr_vm_language"]`、`dist_dependency_boundary_violations = 0`。 |
| 2026-06-24 | Plugins 13 M5/T1 Physics first-party dist rollout | plugins_13_m5_t1_physics_dist_rollout | `physics` 已具备独立 dist 形态：新增 `physics/dist` cdylib crate，`physics/plugin.toml` 与 runtime package manifest 声明 `[distribution]`、native module、ABI v3、engine compatibility、descriptor/runtime entry 和 NativeDynamic default packaging；CI matrix 同步新增 `physics`。`plugin build physics --form dist` 输出根 loadable、`native/<dist_crate>` loader 兼容副本、根 `native_plugins.toml`、`plugins/native_plugins.toml` 与 `.sig` sidecar。验证：scoped rustfmt、py_compile、`test_plugin_build` 4/4、CI matrix unittest 1/1、audit JSON、Physics dist check/test、Physics runtime check、Physics runtime focused test、真实 `plugin build physics` smoke 均通过；audit 报告 `dist_capable_plugin_count = 6`、`dist_build_matrix_count = 6`、`dist_capable_plugins = ["ai", "native_dynamic_fixture", "navigation", "physics", "solari", "zr_vm_language"]`、`dist_dependency_boundary_violations = 0`。 |
| 2026-06-24 | Plugins 13 M5/T1 Navigation first-party dist rollout | plugins_13_m5_t1_navigation_dist_rollout | `navigation` 已具备独立 dist 形态：新增 `navigation/dist` cdylib crate，`navigation/plugin.toml` 与 runtime package manifest 声明 `[distribution]`、native module、ABI v3、engine compatibility、descriptor/runtime entry 和 NativeDynamic default packaging；CI matrix 同步新增 `navigation`。`plugin build navigation --form dist` 输出根 loadable、`native/<dist_crate>` loader 兼容副本、根 `native_plugins.toml`、`plugins/native_plugins.toml` 与 `.sig` sidecar。验证：scoped rustfmt、py_compile、`test_plugin_build` 4/4、CI matrix unittest 1/1、audit JSON、Navigation dist check/test、Navigation runtime check、真实 `plugin build navigation` smoke 均通过；audit 报告 `dist_capable_plugin_count = 5`、`dist_build_matrix_count = 5`、`dist_capable_plugins = ["ai", "native_dynamic_fixture", "navigation", "solari", "zr_vm_language"]`、`dist_dependency_boundary_violations = 0`。runtime focused unit test 被无关 `zircon_runtime` UI surface private method / trait import 编译漂移阻塞，未计通过。 |
| 2026-06-23 | Plugins 13 M5/T1 ZrVM Language first-party dist rollout | plugins_13_m5_t1_zr_vm_language_dist_rollout | `zr_vm_language` 已具备独立 dist 形态：新增 `zr_vm_language/dist` cdylib crate，`zr_vm_language/plugin.toml` 与 runtime package manifest 声明 `[distribution]`、native module、ABI v3、engine compatibility、descriptor/runtime entry 和 NativeDynamic default packaging；CI matrix 同步新增 `zr_vm_language`。`plugin build zr_vm_language --form dist` 输出根 loadable、`native/<dist_crate>` loader 兼容副本、根 `native_plugins.toml`、`plugins/native_plugins.toml` 与 `.sig` sidecar。验证：scoped rustfmt、py_compile、`test_plugin_build` 4/4、CI matrix unittest 1/1、audit JSON、ZrVM Language dist check/test、ZrVM Language runtime check、真实 `plugin build zr_vm_language` smoke 均通过；audit 报告 `dist_capable_plugin_count = 4`、`dist_build_matrix_count = 4`、`dist_capable_plugins = ["ai", "native_dynamic_fixture", "solari", "zr_vm_language"]`、`dist_dependency_boundary_violations = 0`。runtime focused unit test 的 manifest 入口断言已修正，但复跑被无关 `zircon_runtime/src/ui/component/catalog/editor_showcase.rs` `numeric` 未定义编译漂移阻塞，未计通过。 |
| 2026-06-23 | Plugins 13 M5/T1 AI first-party dist rollout | plugins_13_m5_t1_ai_dist_rollout | `ai` 已具备独立 dist 形态：新增 `ai/dist` cdylib crate，`ai/plugin.toml` 与 runtime package manifest 声明 `[distribution]`、native module、ABI v3、engine compatibility、descriptor/runtime entry 和 NativeDynamic default packaging；CI matrix 同步新增 `ai`。`plugin build ai --form dist` 输出根 loadable、`native/<dist_crate>` loader 兼容副本、根 `native_plugins.toml`、`plugins/native_plugins.toml` 与 `.sig` sidecar。验证：scoped rustfmt、py_compile、`test_plugin_build` 4/4、CI matrix unittest 1/1、audit JSON、AI dist check/test、AI runtime focused test、真实 `plugin build ai` smoke 均通过；audit 报告 `dist_capable_plugin_count = 3`、`dist_build_matrix_count = 3`、`dist_capable_plugins = ["ai", "native_dynamic_fixture", "solari"]`、`dist_dependency_boundary_violations = 0`。插件 workspace 全量 locked check 被无关 `zircon_runtime` UI template 编译漂移阻塞，未计通过。 |
| 2026-06-23 | Plugins 13 M5/T1 Solari first-party dist rollout | plugins_13_m5_t1_solari_dist_rollout | `solari` 已具备独立 dist 形态：新增 `solari/dist` cdylib crate，`solari/plugin.toml` 与 runtime package manifest 声明 `[distribution]`、native module、ABI v3、engine compatibility、descriptor/runtime entry 和 NativeDynamic default packaging；CI matrix 同步新增 `solari`。`plugin build solari --form dist` 输出根 loadable、`native/<dist_crate>` loader 兼容副本、根 `native_plugins.toml` 与 `plugins/native_plugins.toml`。验证：scoped rustfmt、py_compile、`test_plugin_build` 4/4、CI matrix unittest 1/1、audit JSON、Solari dist check/test、Solari runtime focused test、真实 `plugin build solari` smoke 和插件 workspace locked check 均通过；audit 报告 `dist_capable_plugin_count = 2`、`dist_build_matrix_count = 2`、`dist_dependency_boundary_violations = 0`。根 `zircon_runtime` focused locked check 被 root `Cargo.lock` 更新需求阻塞，未计通过。 |
| 2026-06-23 | Plugins 13 M5/T1 final skeleton owner rollout | plugins_13_m5_t1_final_skeleton_owner_rollout | `animation`、`animation_graph`、`hybrid_gi`、`material_editor`、`navigation`、`net`、`rendering`、`virtual_geometry` 已完成 owner 骨架迁移，结构审计报告 `plugin_skeleton_gate.m2_gate_status = sample-clean-migration-debt-clear`、`skeleton_conformance.migration_debt_count = 0`、`plugin_skeleton_gate.migration_debt_count = 0`、`migration_debt_roots = []`；该历史快照当时 `dist_capable_plugin_count = 1`、`dist_build_matrix_count = 1`、`dist_dependency_boundary_violations = 0` 保持 seed 状态，当前已由 Solari、AI、ZrVM Language、Navigation、Physics、Texture、Particles、Animation 与 Hybrid GI rollout 更新为 10。139 个 touched owner/façade 文件 rustfmt check 通过；插件 workspace cargo check 被无关 `zircon_runtime` render 编译漂移阻塞，未计通过。 |
| 2026-06-23 | Plugins 13 M5/T1 timeline_sequence editor skeleton owner sub-rollout | plugins_13_m5_t1_timeline_sequence_skeleton_owner_rollout | `timeline_sequence/editor` 已拆出 `capability.rs`、`extension_ids.rs`、`plugin.rs` 与 `tests.rs` owner，`lib.rs` 退为薄 façade 并保留 timeline authoring 领域 helper。该切片为后续 timeline sequence authoring embed/dist 或扩展包拆分降低 crate-root 耦合，但不新增 dist-capable 插件。验证：scoped rustfmt 通过；`python tools\audit_plugin_structure.py --json` 报告 `skeleton_conformance.migration_debt_count = 8`、`plugin_skeleton_gate.migration_debt_count = 8`、`standalone_distribution_conformance.dist_capable_plugin_count = 1`、`capability_conformance.capability_source_mismatches = 0`；timeline_sequence Cargo check 被当前无关 `zircon_runtime` render 编译漂移挡在目标 crate 前，未计通过。该记录只减少 skeleton debt；M5/T1 full dual-form rollout 与 dist-capable 扩容仍未关闭。 |
| 2026-06-23 | Plugins 13 M5/T1 sound runtime/editor feature skeleton owner sub-rollout | plugins_13_m5_t1_sound_skeleton_owner_rollout | `sound` main runtime/editor 与 `ray_traced_convolution_reverb`、`timeline_animation_track` feature runtime/editor crate 已拆出 `capability.rs`、`plugin.rs` 与 `tests.rs` owner（main editor 另有 `extension_ids.rs`），`runtime_plugin/registration.rs` 删除，`lib.rs` 均退为薄 façade。该切片为后续 sound embed/dist 双形态和 feature bundle 分发拆分降低 crate-root 耦合，但不新增 dist-capable 插件。验证：scoped rustfmt 通过；`python tools\audit_plugin_structure.py --json` 报告 `skeleton_conformance.migration_debt_count = 9`、`plugin_skeleton_gate.migration_debt_count = 9`、`standalone_distribution_conformance.dist_capable_plugin_count = 1`、`capability_conformance.capability_source_mismatches = 0`；sound Cargo check 被当前无关 `zircon_runtime` render 编译漂移挡在目标 crate 前，未计通过。该记录只减少 skeleton debt；M5/T1 full dual-form rollout 与 dist-capable 扩容仍未关闭。 |
| 2026-06-23 | Plugins 13 M5/T1 editor build/export desktop skeleton owner sub-rollout | plugins_13_m5_t1_editor_build_export_desktop_skeleton_owner_rollout | `editor_build_export_desktop` editor crate 已拆出 `capability.rs`、`extension_ids.rs`、`plugin.rs` 与 `tests.rs` owner，`lib.rs` 退为 74 行薄 façade；NativeDynamic report templates、导出 operation/menu 与 export profile authoring 注册不再堆在 crate root。验证：scoped rustfmt 通过；`python tools\audit_plugin_structure.py --json` 报告 `skeleton_conformance.migration_debt_count = 10`、`plugin_skeleton_gate.migration_debt_count = 10`、`standalone_distribution_conformance.dist_capable_plugin_count = 1`、`capability_conformance.capability_source_mismatches = 0`；`cargo check --manifest-path zircon_plugins\editor_build_export_desktop\editor\Cargo.toml --all-targets --offline --target-dir target\codex-plugin-validation --message-format short --color never` 通过（仅既有 warning 噪声）。该记录只减少 skeleton debt；M5/T1 full dual-form rollout 与 dist-capable 扩容仍未关闭。 |
| 2026-06-23 | Plugins 13 M5/T1 particles / physics / texture skeleton owner sub-rollout | plugins_13_m5_t1_particles_physics_texture_skeleton_owner_rollout | `particles`、`physics`、`texture` 的 runtime crate 已拆出 `plugin.rs` owner 并让 `lib.rs` 退为薄 façade；`physics`/`texture` 测试移入 `tests.rs`，`texture` 拆出 `manager.rs`/`module.rs`；三个 editor crate 已拆出 `capability.rs`、`extension_ids.rs`、`plugin.rs` 与 `tests.rs`。`particles/runtime/src/simulation/cpu.rs` 补齐 `RenderParticleSpriteSnapshot.render_layer_mask` 字段。验证：scoped rustfmt 通过；`python tools/audit_plugin_structure.py --json` 报告 `skeleton_conformance.migration_debt_count = 11`、`plugin_skeleton_gate.migration_debt_count = 11`、`standalone_distribution_conformance.dist_capable_plugin_count = 1`、`capability_conformance.capability_source_mismatches = 0`。三包 runtime 与三包 editor `cargo check --offline` 均通过（仅既有 warning 噪声）。该记录只减少 skeleton debt；M5/T1 full dual-form rollout 与 dist-capable 扩容仍未关闭。 |
| 2026-06-23 | Plugins 13 M5/T1 authoring runtime/editor skeleton owner sub-rollout | plugins_13_m5_t1_authoring_runtime_editor_skeleton_owner_rollout | `prefab_tools`、`terrain`、`tilemap_2d` 的 runtime crate 已拆出 `plugin.rs`、`capability.rs` 与 `tests.rs`，editor crate 已拆出 `authoring.rs`、`capability.rs`、`extension_ids.rs`、`plugin.rs` 与 `tests.rs`，`lib.rs` 退为薄 façade。验证：scoped rustfmt 通过；`python tools/audit_plugin_structure.py --json` 报告 `skeleton_conformance.migration_debt_count = 14`、`plugin_skeleton_gate.migration_debt_count = 14`、`standalone_distribution_conformance.dist_capable_plugin_count = 1`、`capability_conformance.capability_source_mismatches = 0`。三包 runtime `cargo check --offline` 通过（仅既有 warning 噪声）；三包 editor `cargo check --offline` 两次依赖构建超时，未计通过。该记录只减少 skeleton debt；M5/T1 full dual-form rollout 与 dist-capable 扩容仍未关闭。 |
| 2026-06-23 | Plugins 13 M5/T1 editor-only skeleton owner sub-rollout | plugins_13_m5_t1_editor_only_skeleton_owner_rollout | `native_window_hosting`、`runtime_diagnostics`、`ui_asset_authoring` 的 editor crate 已拆出 `capability.rs`、`extension_ids.rs`、`plugin.rs` 与 `tests.rs`，`lib.rs` 退为薄 façade。验证：scoped rustfmt 通过；py_compile 通过；`python tools/audit_plugin_structure.py --json` 报告 `skeleton_conformance.migration_debt_count = 17`、`plugin_skeleton_gate.migration_debt_count = 17`、`standalone_distribution_conformance.dist_capable_plugin_count = 1`、`capability_conformance.capability_source_mismatches = 0`。三包 editor `cargo check` 被无关 `zircon_editor` retained-host compile drift 阻塞，未计通过。该记录只减少 skeleton debt；M5/T1 full dual-form rollout 与 dist-capable 扩容仍未关闭。 |
| 2026-06-23 | Plugins 13 M5/T1 runtime-only skeleton owner sub-rollout | plugins_13_m5_t1_runtime_only_skeleton_owner_rollout | `ai`、`solari`、`zr_vm_language` 已把 runtime plugin trait owner 迁到 `runtime/src/plugin.rs`；`asset_importers/audio` 与 `asset_importers/texture` 新增 `capability.rs`/`plugin.rs` 并补 SDK workspace dependency，从 legacy declaration-only `package_manifest()` 收口为 trait-backed runtime plugin。验证：scoped rustfmt 通过；offline cargo check 刷新 lock 后 locked cargo check 通过；`python tools/audit_plugin_structure.py --json` 报告 `skeleton_conformance.migration_debt_count = 20`、`plugin_skeleton_gate.migration_debt_count = 20`、`standalone_distribution_conformance.dist_capable_plugin_count = 1`。该记录只减少 skeleton debt；M5/T1 full dual-form rollout 与 dist-capable 扩容仍未关闭。 |
| 2026-06-23 | Plugins 13 M5/T1 importer skeleton/capability-owner sub-rollout | plugins_13_m5_t1_importer_capability_owner_rollout | `audio_importer`、`gltf_importer`、`obj_importer`、`opus_importer`、`shader_wgsl_importer`、`texture_importer`、`ui_document_importer` 与 `asset_importers/{data,model,shader}` 已新增 `runtime/src/capability.rs` 并让 `lib.rs` 退为薄 façade + 常量 re-export。验证：scoped rustfmt 通过；`python tools/audit_plugin_structure.py --json` 报告 `skeleton_conformance.migration_debt_count = 25`、`plugin_skeleton_gate.migration_debt_count = 25`、`standalone_distribution_conformance.dist_capable_plugin_count = 1`；10 个 importer runtime package 逐包 locked `cargo check` 均 exit=0。该记录只减少 skeleton debt；M5/T1 full dual-form rollout 与 dist-capable 扩容仍未关闭。 |
| 2026-06-23 | Plugins 13 M5/T3 RuntimePluginId string-newtype | plugins_13_runtime_plugin_id_string_newtype_accepts_external_ids | `RuntimePluginId` 已从封闭 enum 改为开放 string-newtype，内建插件 id 保留关联常量，合法第三方 key 可由 `parse_key(...)` / `new(...)` 承载并稳定序列化为字符串；runtime plugin loader 对未知合法 id 走 externalized diagnostic fallback；结构守卫拒绝回退 enum，plugin workspace shape guard 改为按 runtime catalog membership 区分 editor-only 插件。验证：scoped rustfmt 通过；runtime `core-min` cargo check 通过；plugin SDK locked check 通过（均仅既有 warning 噪声）；focused runtime lib-test 被无关 test-tree 编译漂移阻塞，未计通过。该记录关闭 M5/T3；M5/T1 全量双形态 rollout 仍未关闭。 |
| 2026-06-23 | Plugins 13 M5/T2 current dist-capable CI matrix seed | plugins_13_standalone_dist_ci_matrix_covers_dist_capable_plugins | `.github/workflows/ci.yml` 新增 `plugin-standalone-dist` job，当前 matrix 覆盖 `native_dynamic_fixture` 的 locked `dist` build；`dependency_boundary.py` 输出 `dist_build_matrix_entries`/`dist_build_matrix_count`，`test_plugin_standalone_ci_matrix.py` 锁定 CI matrix 与审计集合一致。验证：py_compile、CI matrix focused unittest 1/1、audit JSON 与真实 `cargo build -p zircon_plugin_native_dynamic_fixture_native --no-default-features --features dist --locked` 均通过。该记录只关闭 M5/T2 当前 dist-capable CI seed；当前 matrix 已由 Solari、AI、ZrVM Language、Navigation、Physics、Texture、Particles、Animation、Hybrid GI 与 Terrain rollout 扩到 11，M5/T1 全量双形态 rollout 仍未关闭，M5/T3 已由上方记录关闭。 |
| 2026-06-23 | Plugins 13 M4/T3 signing/hash + load manifest assembly | plugins_13_native_plugin_load_manifest_assembles_signed_entries | `plugin_build.py` 现在在写 `native_dynamic_package.toml` 前执行可选外部 signer，并生成确定性的 `<id>.sig` hash sidecar；`native_dynamic_package.toml` payload 收录签名后的 loadable artifact 与 `.sig`。单插件 out 根同时写出 `native_plugins.toml`，其中 `[[plugins]]` 行携带 `id/path/manifest/package_report/[plugins.abi]`，可被 runtime `NativePluginLoadManifest` 消费。验证：`python -m unittest tools.zircon_export.tests.test_plugin_build` 4/4 通过；py_compile 与 carrier focused unittest 通过。该记录关闭 M4/T3；M5 未关闭。 |
| 2026-06-23 | Plugins 13 M4/T2 loader distribution compatibility gate | plugins_13_loader_distribution_compatibility_gate | `PluginPackageManifest` 新增 typed `distribution: Option<PluginDistributionManifest>`；runtime native loader 在 module kind 命中后、library path probe 前执行 compatibility gate。`forms` 不含 `dist`、`abi_version` 缺失或不等于 ABI v3、`engine_compat` 为空/格式错误/不包含当前 `CARGO_PKG_VERSION` 时写入诊断并跳过插件，避免后续误报 `library is missing`。验证：scoped `rustfmt` 通过；`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked` 通过；focused lib-test 被无关 runtime test-tree 编译漂移阻塞，未计通过。该记录关闭 M4/T2 production 代码路径与编译检查；M4/T3 已由上方记录关闭，M5 未关闭。 |
| 2026-06-23 | Plugins 13 M4/T1 per-plugin zrpack 资产子包 | plugins_13_native_dynamic_package_includes_plugin_zrpack | `plugin_build.py` 现在读取 `[distribution].assets`，对匹配的插件内资产生成 Pack 阶段 JSON 清单，并通过 `zircon_export_pack` / `--packer` 写出 `<id>.zrpack` 到单插件包目录；`native_dynamic_package.toml` 的 payload manifest 自动把 `.zrpack` 与 loadable cdylib、`plugin.toml` 一起计入。`native_dynamic_fixture` 新增最小 `assets/shader.wgsl` 并在 manifest 声明 `assets = ["assets/**"]`。验证：`python -m unittest tools.zircon_export.tests.test_plugin_build` 3/3 通过；真实 `native_dynamic_fixture` M4/T1 smoke 在 packer Cargo 构建/运行阶段 604s 超时，未计通过。该记录只关闭 focused guard 与代码路径；M4/T3 已由上方签名/hash/load manifest 记录关闭，M5 未关闭。 |
| 2026-06-23 | Plugins 13 M3/T3 可重复包输出 | plugins_13_plugin_dist_build_is_byte_reproducible | `test_plugin_dist_build_is_byte_reproducible` 对同一插件 manifest、同一 fake cargo 输出双跑 `plugin build` 并比较包目录 bytes 映射；真实 `native_dynamic_fixture` 复用同一 target dir 双跑到 `D:\cargo-targets\zircon-plugin-m3-t3-package-a` 与 `D:\cargo-targets\zircon-plugin-m3-t3-package-b`，`native_dynamic_fixture.dll`、`plugin.toml` 与 `native_dynamic_package.toml` sha256 一致。验证：`python -m py_compile tools/zircon_export/plugin_build.py tools/zircon_export/cli.py tools/zircon_export/tests/test_plugin_build.py` 通过；`python -m unittest tools.zircon_export.tests.test_plugin_build` 2/2 通过；真实双跑比较输出 `MATCH`。该记录只关闭 M3/T3；M4/T1 已由上方记录关闭，M4/T3 已由上方签名/hash/load manifest 记录关闭，M5 未关闭。 |
| 2026-06-23 | Plugins 13 M3/T2 `[distribution].forms` carrier 判定 | plugins_13_zircon_build_classifies_forms_from_manifest | `tools/zircon_build.py` 现在把 manifest forms 作为 profile 级插件 carrier 判定权威：`forms=["dist"]` 选 `native_dynamic`，`forms=["embed"]` 选 `rlib_static`，`dist_crate` 约束动态 crate；无 forms 的 legacy manifest 保留 crate-type 回退。新增 focused test 覆盖 dist-only、embed-only 与 legacy cdylib 分类。验证：`python -m py_compile tools/zircon_build.py tools/tests/test_zircon_build_plugin_carriers.py` 通过；`python -m unittest discover -s tools/tests -p test_zircon_build_plugin_carriers.py` 1/1 通过。该记录只关闭 M3/T2；M3/T3 与 M4/T1 已由上方记录关闭，M4/T3 已由上方签名/hash/load manifest 记录关闭，M5 未关闭。 |
| 2026-06-23 | Plugins 13 M3/T1 `plugin build <id>` 单插件构建入口 | plugins_13_plugin_build_emits_isolated_package_dir | `python -m tools.zircon_export plugin build native_dynamic_fixture --form dist ...` 已可从 `plugin.toml` `[distribution]` 解析 dist crate，执行 `cargo build -p zircon_plugin_native_dynamic_fixture_native --no-default-features --features dist --locked --target-dir <isolated>`，并物化 `<out>/native_dynamic_fixture/native_dynamic_fixture.dll`、`plugin.toml` 与 `native_dynamic_package.toml`。Python focused test 锁定命令参数、包目录布局和不写 `<out>/stages`；真实 fixture 构建/打包 smoke 通过。M3/T2/T3 与 M4/T1 已由上方记录关闭；M4/T3 已由上方签名/hash/load manifest 记录关闭，M5 未关闭。 |
| 2026-06-23 | Plugins 13 M2/T3 dist entry helper | plugins_13_dist_entry_helper_compiles | `zircon_plugin_sdk::dist` 新增 `native_dist_plugin_v3!` / `native_dist_runtime_plugin_v3!`，由一份声明生成 ABI descriptor、runtime/editor entry report、behavior schema/manifest 指针、capability gating、bridge method table 与 symbol export。`native_dynamic_fixture` 已删除手写 ABI static 样板并改用该 helper，`plugin.toml` 也声明 `native_dynamic_fixture.runtime.tick` method slot。验证：SDK focused `dist_plugin_one_file_export_compiles` 1/1 通过；fixture `--no-default-features --features dist` check 通过。M3/T1/T2/T3 与 M4/T1 已由上方记录关闭；M4/T3 已由上方签名/hash/load manifest 记录关闭，M5 未关闭。 |
| 2026-06-23 | Plugins 13 M2/T2 system bridge replay | plugins_13_registration_bridge_replay_implemented | runtime loader 新增 `registration_manifest.rs` 解析 `zircon.native.registration-manifest/3`；`NativePluginLiveHost::replay_runtime_registration_manifests_via_bridge(...)` 将已加载 dist 插件的 registration manifest system anchor 回放到 `RuntimeExtensionRegistry`，复用 `NativeDynamicAccess` 保守访问并捕获 `NativeHostBridgeCallScope`，tick 时通过 dense bridge slot/method slot 调插件执行体。新增公开回放报告 `NativePluginRuntimeRegistrationReplayReport` 与 focused test `dist_system_plugin_loads_and_ticks_via_bridge`。验证：scoped rustfmt 通过；focused cargo test 被无关 runtime lib-test compile drift 阻塞，未计通过。M2/T3、M3/T1/T2/T3 与 M4/T1 见上方记录，M4/T3 已由上方签名/hash/load manifest 记录关闭，M5 未关闭。 |
| 2026-06-23 | Plugins 13 M2/T1 registration manifest ABI schema | plugins_13_registration_manifest_schema_round_trips | SDK native ABI v3 已增加 `registration_manifest_schema` 与 `registration_manifest`，SDK TOML DTO 覆盖 module/system/resource/event/extension/capability registration manifest，runtime loader ABI declaration、behavior reader、validation report、loaded-plugin accessor 与 live-host runtime descriptor 同步承载。`native_dynamic_fixture` runtime entry 导出 `zircon.native.registration-manifest/3` registration manifest，包含 runtime module、`native_dynamic_fixture.runtime_tick` system、event 与 capability。SDK native-only check、`native_dynamic_registration_manifest_round_trips` 1/1、native fixture dist check、audit JSON、py_compile 与 scoped rustfmt 已通过；runtime loader focused test 当前被无关 runtime lib-test compile drift 阻塞，不计通过；带默认 runtime feature 的 SDK native test 当前被无关 render compile drift 阻塞，不计通过。M2/T2、M2/T3、M3/T1/T2/T3 与 M4/T1 见上方记录，M4/T3 已由上方签名/hash/load manifest 记录关闭，M5 未关闭。 |
| 2026-06-23 | Plugins 13 M1 首个 dist 形态 + 依赖边界 guard | plugins_13_dist_dependency_boundary_clean | `native_dynamic_fixture/plugin.toml` 已声明 `[distribution]`，`native_dynamic_fixture/native/Cargo.toml` 支持 `--no-default-features --features dist`；`tools/audit_plugin_structure.py --json` 输出 `dist_capable_plugin_count = 1`、`distribution_section_violations = 0`、`dist_dependency_boundary_violations = 0`、`m1_dist_dependency_boundary_gate_status = dist-boundary-clean`；`zircon_first_party_runtime_catalog::tests::plugins_13_dist_dependency_boundary_clean` 1/1 通过。M2 注册跨 ABI 编组、M3/T1/T2/T3 与 M4/T1 已由上方记录推进；M4/T3 已由上方签名/hash/load manifest 记录关闭，M5 全量 rollout 未关闭。 |
