---
related_code:
  - zircon_plugins/plugin_sdk_examples/plugin.toml
  - zircon_plugins/plugin_sdk_examples/editor/Cargo.toml
  - zircon_plugins/plugin_sdk_examples/editor/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/capability.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extensions.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extension_ids.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/tests.rs
  - tools/plugin_structure_audits/skeleton.py
  - tools/audit_plugin_structure.py
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/plugin_sdk/src/dist.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/test.rs
  - zircon_plugins/animation/runtime/Cargo.toml
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/representation.rs
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/src/capability.rs
  - zircon_plugins/asset_importers/data/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/capability.rs
  - zircon_plugins/asset_importers/model/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/src/capability.rs
  - zircon_plugins/asset_importers/shader/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/audio/runtime/Cargo.toml
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/capability.rs
  - zircon_plugins/asset_importers/audio/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/texture/runtime/Cargo.toml
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/capability.rs
  - zircon_plugins/asset_importers/texture/runtime/src/plugin.rs
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/solari/runtime/src/lib.rs
  - zircon_plugins/solari/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
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
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/capability.rs
  - zircon_plugins/particles/runtime/src/plugin.rs
  - zircon_plugins/particles/runtime/src/simulation/cpu.rs
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
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/capability.rs
  - zircon_plugins/gltf_importer/runtime/src/plugin.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/obj_importer/runtime/src/capability.rs
  - zircon_plugins/obj_importer/runtime/src/plugin.rs
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/capability.rs
  - zircon_plugins/texture_importer/runtime/src/plugin.rs
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/audio_importer/runtime/src/capability.rs
  - zircon_plugins/audio_importer/runtime/src/plugin.rs
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/src/capability.rs
  - zircon_plugins/opus_importer/runtime/src/plugin.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/capability.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/plugin.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/runtime/src/capability.rs
  - zircon_plugins/ui_document_importer/runtime/src/plugin.rs
  - tools/plugin_structure_audits/registration.py
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
implementation_files:
  - zircon_plugins/plugin_sdk_examples/editor/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/capability.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extensions.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extension_ids.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/tests.rs
  - tools/plugin_structure_audits/skeleton.py
  - tools/audit_plugin_structure.py
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/plugin_sdk/src/dist.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/test.rs
  - zircon_plugins/animation/runtime/Cargo.toml
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/representation.rs
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/src/capability.rs
  - zircon_plugins/asset_importers/data/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/capability.rs
  - zircon_plugins/asset_importers/model/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/src/capability.rs
  - zircon_plugins/asset_importers/shader/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/audio/runtime/Cargo.toml
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/capability.rs
  - zircon_plugins/asset_importers/audio/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/texture/runtime/Cargo.toml
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/capability.rs
  - zircon_plugins/asset_importers/texture/runtime/src/plugin.rs
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/solari/runtime/src/lib.rs
  - zircon_plugins/solari/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
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
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/capability.rs
  - zircon_plugins/particles/runtime/src/plugin.rs
  - zircon_plugins/particles/runtime/src/simulation/cpu.rs
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
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/capability.rs
  - zircon_plugins/gltf_importer/runtime/src/plugin.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/obj_importer/runtime/src/capability.rs
  - zircon_plugins/obj_importer/runtime/src/plugin.rs
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/capability.rs
  - zircon_plugins/texture_importer/runtime/src/plugin.rs
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/audio_importer/runtime/src/capability.rs
  - zircon_plugins/audio_importer/runtime/src/plugin.rs
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/src/capability.rs
  - zircon_plugins/opus_importer/runtime/src/plugin.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/capability.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/plugin.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/runtime/src/capability.rs
  - zircon_plugins/ui_document_importer/runtime/src/plugin.rs
  - tools/plugin_structure_audits/registration.py
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
plan_sources:
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/audit_plugin_structure.py --json: sample_conformance_status=sample-clean, sample_expected_count=1, migration_debt_count=35 on 2026-06-22
  - python -m py_compile tools/audit_plugin_structure.py tools/plugin_structure_audits/__init__.py tools/plugin_structure_audits/manifest_schema.py tools/plugin_structure_audits/skeleton.py: passed 2026-06-22
  - rustfmt --edition 2021 --config skip_children=true --check zircon_plugins/plugin_sdk_examples/editor/src/*.rs zircon_plugins/first_party_runtime_catalog/src/lib.rs: passed 2026-06-22
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor -p zircon_first_party_runtime_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never: passed 2026-06-22 with existing warning noise
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never -- --test-threads=1: timed out after 1200s on 2026-06-22, not counted as passing
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12_crate_skeleton_conformance --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never -- --test-threads=1 --nocapture: timed out after 900s on 2026-06-22, not counted as passing
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-test-runtime-m2-0622 --message-format short --color never: passed 2026-06-22 with existing zircon_runtime warnings
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-test-runtime-m2-0622 --message-format short --color never test_runtime_builder -- --test-threads=1 --nocapture: 2 passed, 0 failed on 2026-06-22
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk -p zircon_plugin_animation_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-registration-m3-0622 --message-format short --color never: passed 2026-06-22
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-registration-m3-0622 --message-format short --color never runtime_registration_builder -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-22
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-registration-m3-0622 --message-format short --color never animation_registration_contributes_runtime_module -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-22
  - python tools/audit_plugin_structure.py --json: registration_conformance.m3_t1_gate_status=family-single-entry-clean, asset_importer_family_free_function_registration_sites=0 on 2026-06-23
  - python -m py_compile tools/audit_plugin_structure.py tools/plugin_structure_audits/__init__.py tools/plugin_structure_audits/manifest_schema.py tools/plugin_structure_audits/skeleton.py tools/plugin_structure_audits/registration.py: passed 2026-06-23
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_data_runtime -p zircon_plugin_asset_importer_model_runtime -p zircon_plugin_asset_importer_shader_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-importer-m3-0622 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime warnings
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_model_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-importer-m3-0622 --message-format short --color never registration_contributes_stl_ply_and_dxf_importers -- --test-threads=1 --nocapture: blocked 2026-06-22 by unrelated zircon_runtime MaterialCaptureSeed / MaterialRuntime::capture_seed lib-test drift
  - rustfmt --edition 2021 --check split importer lib/plugin files plus zircon_runtime builtin plugin id/loader: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: registration_conformance.m3_split_importer_gate_status=split-importer-single-entry-clean, split_importer_free_function_registration_sites=0, split_importer_registration_owner_files=0, m3_importer_gate_status=importer-single-entry-clean on 2026-06-23
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime -p zircon_plugin_obj_importer_runtime -p zircon_plugin_texture_importer_runtime -p zircon_plugin_audio_importer_runtime -p zircon_plugin_opus_importer_runtime -p zircon_plugin_shader_wgsl_importer_runtime -p zircon_plugin_ui_document_importer_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-split-importer-m3-0623 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime warnings
  - rustfmt --edition 2021 --check importer runtime lib.rs/capability.rs set for audio/gltf/obj/opus/shader_wgsl/texture/ui_document and asset_importers/data/model/shader: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=25 and plugin_skeleton_gate.migration_debt_count=25 on 2026-06-23
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p each of zircon_plugin_audio_importer_runtime, zircon_plugin_gltf_importer_runtime, zircon_plugin_obj_importer_runtime, zircon_plugin_opus_importer_runtime, zircon_plugin_shader_wgsl_importer_runtime, zircon_plugin_texture_importer_runtime, zircon_plugin_ui_document_importer_runtime, zircon_plugin_asset_importer_data_runtime, zircon_plugin_asset_importer_model_runtime, zircon_plugin_asset_importer_shader_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-skeleton-split-importers-check --message-format short --color never: each package exit=0 on 2026-06-23 with existing warning noise
  - rustfmt --edition 2021 --check runtime-only skeleton owner files for ai, asset_importers/audio, asset_importers/texture, solari, zr_vm_language: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=20 and plugin_skeleton_gate.migration_debt_count=20 on 2026-06-23
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ai_runtime -p zircon_plugin_asset_importer_audio_runtime -p zircon_plugin_asset_importer_texture_runtime -p zircon_plugin_solari_runtime -p zircon_plugin_zr_vm_language_runtime --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-only-skeleton-check --message-format short --color never: passed 2026-06-23 and refreshed zircon_plugins/Cargo.lock
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ai_runtime -p zircon_plugin_asset_importer_audio_runtime -p zircon_plugin_asset_importer_texture_runtime -p zircon_plugin_solari_runtime -p zircon_plugin_zr_vm_language_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-only-skeleton-check --message-format short --color never: passed 2026-06-23 with existing warning noise
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_audio_runtime -p zircon_plugin_asset_importer_texture_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-only-skeleton-check --message-format short --color never package_declares_ -- --test-threads=1 --nocapture: timed out after 904s on 2026-06-23 with no test result; residual cargo/rustc processes for this target-dir were stopped; not counted as passing
  - python -m py_compile tools/plugin_structure_audits/capability.py tools/audit_plugin_structure.py: passed 2026-06-23 after plugin.rs owner re-export handling
  - rustfmt --edition 2021 --check editor-only skeleton owner files for native_window_hosting, runtime_diagnostics, ui_asset_authoring: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=17, plugin_skeleton_gate.migration_debt_count=17, capability_conformance.capability_source_mismatches=0 on 2026-06-23
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_native_window_hosting_editor -p zircon_plugin_runtime_diagnostics_editor -p zircon_plugin_ui_asset_authoring_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-editor-small-skeleton-check --message-format short --color never: blocked by unrelated zircon_editor retained-host compile drift before target packages; not counted as passing
  - rustfmt --edition 2021 authoring runtime/editor skeleton owner files for prefab_tools, terrain, tilemap_2d: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=14, plugin_skeleton_gate.migration_debt_count=14, capability_conformance.capability_source_mismatches=0 on 2026-06-23
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_prefab_tools_runtime -p zircon_plugin_terrain_runtime -p zircon_plugin_tilemap_2d_runtime --offline --jobs 1 --target-dir E:\cargo-targets\zircon-plugin-m5-authoring-runtime-0623 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime warning noise
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_prefab_tools_editor -p zircon_plugin_terrain_editor -p zircon_plugin_tilemap_2d_editor --offline --jobs 1 --target-dir E:\cargo-targets\zircon-plugin-m5-authoring-editor-0623 --message-format short --color never: timed out twice while building dependencies; no final output captured, not counted as passing
  - rustfmt --edition 2021 particles/physics/texture runtime/editor skeleton owner files plus zircon_plugins/particles/runtime/src/simulation/cpu.rs: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=11, plugin_skeleton_gate.migration_debt_count=11, capability_conformance.capability_source_mismatches=0 on 2026-06-23
  - cargo check -p zircon_plugin_particles_runtime --offline; cargo check -p zircon_plugin_physics_runtime --offline; cargo check -p zircon_plugin_texture_runtime --offline: passed 2026-06-23 with existing warning noise
  - cargo check --manifest-path zircon_plugins\particles\editor\Cargo.toml --offline; cargo check --manifest-path zircon_plugins\physics\editor\Cargo.toml --offline --target-dir target\codex-plugin-validation; cargo check --manifest-path zircon_plugins\texture\editor\Cargo.toml --offline --target-dir target\codex-plugin-validation: passed 2026-06-23 with existing warning noise
  - rustfmt --edition 2021 zircon_plugins\editor_build_export_desktop\editor\src\lib.rs zircon_plugins\editor_build_export_desktop\editor\src\capability.rs zircon_plugins\editor_build_export_desktop\editor\src\extension_ids.rs zircon_plugins\editor_build_export_desktop\editor\src\plugin.rs zircon_plugins\editor_build_export_desktop\editor\src\tests.rs zircon_plugins\editor_build_export_desktop\editor\src\export_wizard.rs: passed 2026-06-23
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=10, plugin_skeleton_gate.migration_debt_count=10, capability_conformance.capability_source_mismatches=0 on 2026-06-23
  - cargo check --manifest-path zircon_plugins\editor_build_export_desktop\editor\Cargo.toml --all-targets --offline --target-dir target\codex-plugin-validation --message-format short --color never: passed 2026-06-23 with existing runtime/editor warning noise
  - rustfmt --edition 2021 sound runtime/editor skeleton owner files for main crate plus ray_traced_convolution_reverb and timeline_animation_track feature crates: passed 2026-06-23
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=9, plugin_skeleton_gate.migration_debt_count=9, capability_conformance.capability_source_mismatches=0 on 2026-06-23 after sound owner rollout
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --lib --offline --target-dir target\codex-plugin-validation --message-format short --color never: blocked before target sound crate by unrelated zircon_runtime render compile drift; not counted as passing
  - rustfmt --edition 2021 --check zircon_plugins\timeline_sequence\editor\src\lib.rs zircon_plugins\timeline_sequence\editor\src\capability.rs zircon_plugins\timeline_sequence\editor\src\extension_ids.rs zircon_plugins\timeline_sequence\editor\src\plugin.rs zircon_plugins\timeline_sequence\editor\src\tests.rs: passed 2026-06-23
  - python tools\audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=8, plugin_skeleton_gate.migration_debt_count=8, capability_conformance.capability_source_mismatches=0 on 2026-06-23 after timeline_sequence owner rollout
  - cargo check --manifest-path zircon_plugins\timeline_sequence\editor\Cargo.toml --lib --offline --target-dir target\codex-plugin-validation --message-format short --color never: blocked before target timeline_sequence crate by unrelated zircon_runtime render compile drift; not counted as passing
  - python tools\audit_plugin_structure.py --json: plugin_skeleton_gate.m2_gate_status=sample-clean-migration-debt-clear, skeleton_conformance.migration_debt_count=0, plugin_skeleton_gate.migration_debt_count=0, skeleton_conformance.migration_debt_roots=[], capability_conformance.capability_source_mismatches=0, capability_conformance.missing_capability_owner_files=0, capability_conformance.missing_runtime_capability_exports=0 on 2026-06-23 after final owner rollout
  - rustfmt --edition 2021 --check on 139 touched owner/façade files under animation, animation_graph, hybrid_gi, material_editor, navigation, net, rendering, timeline_sequence, and virtual_geometry: passed 2026-06-23
  - cargo check --manifest-path zircon_plugins\Cargo.toml --workspace --offline --locked --target-dir target\codex-plugin-validation --message-format short --color never: passed 2026-06-23 after final owner rollout and `hybrid_gi` RenderLayerSet mask alignment, with existing warning noise
doc_type: module-detail
status: in_progress
---

# 插件 Crate 骨架（Plugin Crate Skeleton）

> 唯一插件 crate 目录骨架，由 [Plugins 12](../plans/zircon_plugins/12-plugin-dx-and-structure-framework.md) 落地、[引擎结构规范 §6.1](../plans/engine-code-structure-convention.md) 定义。新插件 day 1 即用此骨架；存量插件 touch-it-conform-it 迁入。
>
> 状态：in_progress（骨架定稿；Plugins 12 M2/T1 `plugin_sdk` builder baseline、M2/T2 首个样例/符合度 guard、M2/T3 native SDK helper、M2/T4 editor authoring macro/workspace dependency inheritance、M2/T5 test runtime fixture、M3/T2 runtime registration builder + animation 代表迁移、M3/T1 `asset_importers/*` family 与 split importers 自由注册函数清零、M5/T2 D6 `RuntimePluginId` string-newtype 已落地；Plugins 13 M2/T3 `plugin_sdk::dist` 一文件 cdylib 导出 helper 已落地；M5/T1 importer capability-owner、runtime-only skeleton owner、editor-only skeleton owner、authoring runtime/editor skeleton owner、particles/physics/texture skeleton owner、editor_build_export_desktop skeleton owner、sound runtime/editor feature skeleton owner、timeline_sequence editor skeleton owner 与最终 8 根 owner rollout 已把迁移债从 35 降至 0；`migration_debt_roots = []`）

## 1. 目录骨架

```
<plugin>/
  plugin.toml          # 包级 manifest（统一 schema，强制，见 plugin-manifest-schema.md）
  runtime/
    Cargo.toml
    src/
      lib.rs             # 薄：pub use 公共 API + 导出 Plugin struct + 常量
      plugin.rs          # 唯一注册 owner：impl RuntimePlugin + descriptor()
      capability.rs      # capability id pub const —— 单一来源
      contract/          # 该插件 ABI-safe DTO（纯消费 interface 则省略）
      backend/           # 实际算法 / importer / 协议实现 owner（按结构规范 §1 拆叶子）
      systems/           # 注册进调度图的 ECS 系统
      tests/             # folder-backed
  editor/                # 镜像同骨架（能力对称）
    Cargo.toml
    src/{lib.rs, plugin.rs, capability.rs, ...}
```

## 2. 各文件职责

| 文件 | 职责 | 约束 |
|---|---|---|
| 根 `plugin.toml` | 包级 manifest，列出所有 runtime/editor/native/vm modules | 与 `package_manifest()` / SDK builder 投影一致 |
| module `Cargo.toml` | 单个 runtime/editor/native/vm crate 声明 | 优先使用 workspace 统一依赖（M2/T4 后强制） |
| `lib.rs` | 薄 façade：导出公共 API + Plugin struct + 常量 | 零行为（规范 R1.1） |
| `plugin.rs` | **唯一注册入口** `impl RuntimePlugin::register` + `descriptor()` | 自由函数注册收编于此（规范 §6.3） |
| `capability.rs` | capability id `pub const` | 单一来源，与 `plugin.toml` 一致（§6.4） |
| `contract/` | 该插件 ABI-safe DTO | 纯消费 interface 时省略 |
| `backend/` | 算法 / importer / 协议实现 | 按 owner 叶子拆，软 800 / 硬 1000 行 |
| `systems/` | ECS 系统 | 与 `plugin.toml` `system_anchors` 核对 |

## 3. 导入器类插件

`backend/` 即 importer 实现；`plugin.rs` 的 `register` 同时 `register_module` + 注册 importer descriptor。M3/T1 已把 `asset_importers/{data,model,shader}` 和 root-level split importers 收编到 `RuntimePlugin` trait 入口并删除/避免 `registration.rs` 自由函数分离写法；M5/T1 importer capability-owner 子切片已进一步把 `audio_importer`、`gltf_importer`、`obj_importer`、`opus_importer`、`shader_wgsl_importer`、`texture_importer`、`ui_document_importer` 与 `asset_importers/{data,model,shader}` 的 capability 常量迁入各自 `runtime/src/capability.rs`。M5/T1 runtime-only skeleton owner 子切片又把 `asset_importers/audio` 与 `asset_importers/texture` 从 declaration-only `package_manifest()` 收口为 trait-backed `plugin.rs` owner；它们仍只声明 importer manifest，不伪造尚不存在的真实导入函数注册。

Editor-only 插件同样遵守薄 `lib.rs`：M5/T1 editor-only skeleton owner 子切片已把 `native_window_hosting`、`runtime_diagnostics` 与 `ui_asset_authoring` 迁到 `src/capability.rs`（插件 id / editor capability）、`src/extension_ids.rs`（view/drawer/template id）、`src/plugin.rs`（`EditorPlugin` owner / manifest / registration）与 `src/tests.rs`。后续 editor 插件新增扩展点时应优先落到对应 owner 文件，不把 descriptor、template id 或 registration helper 堆回 crate root。

Authoring runtime+editor 插件按同根双 crate 完成同一骨架：M5/T1 authoring runtime/editor skeleton owner 子切片已把 `prefab_tools`、`terrain` 与 `tilemap_2d` 的 runtime crate 迁到 `runtime/src/plugin.rs`、`runtime/src/capability.rs` 与 `runtime/src/tests.rs`，editor crate 迁到 `editor/src/authoring.rs`、`capability.rs`、`extension_ids.rs`、`plugin.rs` 与 `tests.rs`。后续 authoring 插件应把领域 helper 放入 `authoring.rs` 或更细 owner，`plugin.rs` 只保留 trait owner、manifest、registration 与批量 extension 组装。

Runtime+editor 成对插件继续按同一 owner 规则递减迁移债：M5/T1 particles/physics/texture skeleton owner 子切片已把 `particles`、`physics` 与 `texture` 的 runtime plugin owner 迁到 `runtime/src/plugin.rs`，runtime `lib.rs` 退为薄 façade；`physics` 与 `texture` 的测试进入 `runtime/src/tests.rs`，`texture` 的领域逻辑拆为 `manager.rs` 与 `module.rs`。三个 editor crate 均使用 `capability.rs`、`extension_ids.rs`、`plugin.rs` 与 `tests.rs` owner；`particles` 另保留 `authoring.rs` 领域 helper。`particles/runtime/src/simulation/cpu.rs` 的 `RenderParticleSpriteSnapshot` 构造已补齐 `render_layer_mask`，这是本轮编译验证所需的既有结构漂移修复。

Standalone/export editor 插件也遵守同一骨架：M5/T1 editor_build_export_desktop skeleton owner 子切片已把 `editor_build_export_desktop/editor` 的插件 id 与 editor capabilities 放入 `capability.rs`，view/drawer/template/report/operation ids 与私有模板路径放入 `extension_ids.rs`，`EditorBuildExportDesktopPlugin`、descriptor、manifest、registration report、导出 operation/menu、NativeDynamic report templates 与 export profile authoring 注册放入 `plugin.rs`，package/registration/template asset 断言放入 `tests.rs`；`lib.rs` 只保留 façade 与 export wizard 精选 re-export。

Timeline authoring editor 插件按同一规则收敛：M5/T1 timeline_sequence skeleton owner 子切片已把 `timeline_sequence/editor` 的插件 id、editor capability 与 animation timeline event track capability 放入 `capability.rs`，view/drawer/template ids 放入 `extension_ids.rs`，`TimelineSequenceEditorPlugin`、descriptor、manifest、registration report、operation/menu 与 timeline authoring batch helper 放入 `plugin.rs`，package/registration/validation/keyframe/event marker 断言放入 `tests.rs`；`lib.rs` 只保留 façade 与 timeline authoring 领域 helper。

最终存量 roots 已按同一骨架清零：M5/T1 final owner rollout 把 `animation`、`animation_graph`、`hybrid_gi`、`material_editor`、`navigation`、`net`、`rendering` 与 `virtual_geometry` 的 runtime/editor crate 迁到 `plugin.rs`、`capability.rs`、`extension_ids.rs`、`tests.rs` 等 owner 文件，`net` 和 `rendering` 的 feature runtime/editor crate 也具备 feature-local `capability.rs` / `plugin.rs` owner。当前结构审计报告 `plugin_skeleton_gate.m2_gate_status = sample-clean-migration-debt-clear`、`skeleton_conformance.migration_debt_count = 0`、`plugin_skeleton_gate.migration_debt_count = 0`、`migration_debt_roots = []`。

## 4. `plugin_sdk` builder（祝福路径）

`zircon_plugins/plugin_sdk/` 提供 builder API，使新插件以一文件声明 manifest module、capability、target modes 与 runtime/editor descriptor 投影，降低样板。当前 M2/T1 baseline 已提供：

- `PluginManifestBuilder`：填充 `sdk_api_version`、默认平台与默认 packaging，并产出 runtime-owned `PluginPackageManifest`。
- `PluginModuleBuilder`：标准化 runtime/editor/native/vm module 声明；editor module 默认 `EditorHost`。
- `RuntimePluginDeclaration`：从同一声明投影 `RuntimePluginDescriptor` 与 `PluginPackageManifest`。

M2/T3 native ABI helper 已提供 `plugin_sdk::native` feature、ABI v3 类型、SDK-owned byte buffers、entry export macros，并让 `native_dynamic_fixture` 改为使用 SDK native helper。M2/T4 editor authoring macro 已提供 `plugin_sdk::editor::EditorPluginDeclaration` 与 `authoring_plugin!`，首个 editor 样例用宏生成主 plugin 样板，`zircon_plugins/Cargo.toml` 提供 `[workspace.dependencies]`，样例 editor crate 和 native fixture 已改用 workspace dependency inheritance。M2/T5 test runtime fixture 已提供 `plugin_sdk::test::TestRuntime::builder()`，把跨插件测试常见的 foundation/asset/scene 基础模块、runtime extension merge、world extension install、插件 module 激活和固定步长 tick helper 收进 SDK。

M3/T2 runtime registration builder 已提供 `plugin_sdk::registration::RuntimePluginRegistrationBuilder` 与 `RuntimePluginModuleRegistration`。runtime 插件必须在 `RuntimePluginDescriptor` 中嵌入唯一 `ModuleDescriptor`；report 先注册该 descriptor，随后 `impl RuntimePlugin::register(...)` 通过 `.module(PLUGIN_RUNTIME_MODULE_NAME)` 打开 owner scope，再由 module handle 声明 runtime scene system、set、order 和 before/after constraint。SDK 内部负责 owner token，不再接受第二份 module descriptor。animation、physics、net 三个原始证据 root 已迁到该路径；net 的 typed event、option、event catalog 也通过 module handle helper 注册。状态锚：`d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred`，守卫：`review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder`。

M3/T1 importer registration slices 已新增并扩展 `plugin_structure_audits::registration`，`tools/audit_plugin_structure.py --json` 当前报告 `registration_conformance.m3_t1_gate_status = family-single-entry-clean`、`registration_conformance.m3_split_importer_gate_status = split-importer-single-entry-clean`、`m3_importer_gate_status = importer-single-entry-clean`，且 importer free-function registration sites 为 0。`asset_importers/{data,model,shader}/runtime/src/plugin.rs` 和 root-level split importer `runtime/src/plugin.rs` 现在拥有 trait-backed plugin entry。M5/T2 D6 已把 `RuntimePluginId` 从 core 封闭 enum 改为开放 string-newtype；data/model/shader/opus importer 的一方 id 保留为关联常量，后续第三方或独立插件合法 key 不再需要新增 core 枚举分支。

## 5. 首个骨架样例

`zircon_plugins/plugin_sdk_examples/editor` 是 M2/T2 的首个 skeleton-conformance 样例：

- `src/lib.rs` 只保留模块声明和精选 `pub use`，不承载扩展注册行为。
- `src/capability.rs` 是 editor capability 常量单源。
- `src/plugin.rs` 拥有 `EditorPlugin` 实现、`authoring_plugin!` 主插件声明、`package_manifest()` 投影和 registration report 构造。
- `src/extensions.rs` 拥有 window、asset importer、asset inspector、UI template、component drawer 等 editor extension 注册。
- `src/extension_ids.rs` 拥有 view/importer/template/component id 常量。
- `src/tests.rs` 验证插件注册贡献和 manifest metadata。

## 6. 符合度 guard（Plugins 12 M2）

- `tools/audit_plugin_structure.py --json` 输出 `skeleton_conformance` 与 `plugin_skeleton_gate`。
- M2/T2 样例门禁字段：`sample_conformance_status = sample-clean`、`sample_expected_count = 1`、`sample_violation_count = 0`。
- M2/T4 样例 workspace dependency 门禁字段：`sample_workspace_dependency_status = sample-workspace-deps-clean`、`sample_workspace_dependency_violation_count = 0`。
- `plugins_12_crate_skeleton_conformance` 消费同一 JSON，锁定首个样例不回退。
- `registration_conformance.m3_t1_gate_status = family-single-entry-clean` 锁定 `asset_importers/*` 家族不再出现公开 `pub fn register(...)` 自由函数或 `runtime/src/registration.rs` owner。
- `registration_conformance.m3_split_importer_gate_status = split-importer-single-entry-clean` 和 `m3_importer_gate_status = importer-single-entry-clean` 锁定 split importers 与 aggregate importer 口径不再出现公开注册自由函数或 `runtime/src/registration.rs` owner。
- 存量插件仍按 `migration_debt_roots` 记录为迁移债；2026-06-23 M5/T1 importer capability-owner、runtime-only skeleton owner、editor-only skeleton owner、authoring runtime/editor skeleton owner、particles/physics/texture skeleton owner、editor_build_export_desktop skeleton owner、sound skeleton owner、timeline_sequence skeleton owner 与 final owner rollout 后当前 `migration_debt_count = 0`，`migration_debt_roots = []`。后续新增插件必须 day 1 进入同一骨架，不再新增迁移债。
- `native_dynamic_fixture` 作为 native-only ABI fixture 继续豁免 runtime/editor 骨架规则；M2/T3 已收编其 ABI 样板到 `plugin_sdk::native`，但它仍不是 runtime/editor 双 crate 骨架样例。

## 7. 双形态（embed / dist）发行维扩展

由 [Plugins 13](../plans/zircon_plugins/13-standalone-plugin-build.md) 落地，规范权威 [`plugin-standalone-build.md`](plugin-standalone-build.md)。骨架在发行维新增 `dist` 产物形态，使每个插件既能静态链接（embed）又能独立编译为可分发 cdylib（dist）：

```
<plugin>/runtime/
  Cargo.toml
    # crate-type = ["rlib", "cdylib"]
    # zircon_runtime = { path = "...", optional = true }
    # [features] default=["embed"]  embed=["dep:zircon_runtime","zircon_plugin_sdk/runtime"]  dist=["zircon_plugin_sdk/native"]
  src/
    lib.rs           # 薄 façade
    plugin.rs        # #[cfg(feature="embed")] impl RuntimePlugin::register
    dist.rs          # #[cfg(feature="dist")]  ABI v3 导出 owner（SDK 宏）
    capability.rs    # 单源（禁 use zircon_runtime）
    backend/         # 纯逻辑：仅 zircon_plugin_sdk + zircon_runtime_interface
    systems/ tests/
```

- **依赖边界铁律**：`backend/`、`capability.rs` 禁 `use zircon_runtime::*`；触碰 `zircon_runtime` 的代码必须 `#[cfg(feature = "embed")]`。`dist` 形态依赖闭包禁含 `zircon_runtime`，由 `tools/plugin_structure_audits/dependency_boundary.py` 守卫（`dist_dependency_boundary_violations = 0`）。
- **单源双投影**：一份 manifest + 一份 `backend/` 逻辑同时喂 embed 注册（`plugin_sdk::registration`）与 dist 导出（`plugin_sdk::native` / `plugin_sdk::dist::{native_dist_plugin_v3!, native_dist_runtime_plugin_v3!}`）；不复制逻辑。
- 逻辑无法干净 feature-gate 时退化为独立 `<plugin>/dist/` cdylib crate 包裹 `backend/`（fallback，非首选）。
