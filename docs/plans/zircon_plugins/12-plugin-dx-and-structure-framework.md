---
related_code:
  - zircon_plugins/gltf_importer/plugin.toml
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
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/capability.rs
  - zircon_plugins/sound/plugin.toml
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
  - zircon_plugins/native_dynamic_fixture/plugin.toml
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_plugins/plugin_sdk/Cargo.toml
  - zircon_plugins/plugin_sdk/src/lib.rs
  - zircon_plugins/plugin_sdk/src/editor.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/prelude.rs
  - zircon_plugins/plugin_sdk/src/manifest/feature_bundle_builder.rs
  - zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs
  - zircon_plugins/plugin_sdk/src/runtime_exports.rs
  - zircon_plugins/plugin_sdk/src/test.rs
  - tools/plugin_structure_audits/registration.py
  - tools/plugin_structure_audits/capability.py
  - tools/audit_plugin_structure.py
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_runtime/src/ui/surface/input/editable_text.rs
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs
  - zircon_runtime/src/tests/plugin_extensions/plugin_workspace_shape.rs
  - zircon_plugins/animation/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/src/capability.rs
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/physics/runtime/Cargo.toml
  - zircon_plugins/physics/runtime/src/capability.rs
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/plugin.rs
  - zircon_plugins/physics/runtime/src/tests.rs
  - zircon_plugins/physics/editor/src/lib.rs
  - zircon_plugins/physics/editor/src/capability.rs
  - zircon_plugins/physics/editor/src/extension_ids.rs
  - zircon_plugins/physics/editor/src/plugin.rs
  - zircon_plugins/physics/editor/src/tests.rs
  - zircon_plugins/hybrid_gi/runtime/Cargo.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/capability.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/representation.rs
  - zircon_plugins/navigation/runtime/Cargo.toml
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/src/capability.rs
  - zircon_plugins/net/runtime/Cargo.toml
  - zircon_plugins/net/runtime/src/capability.rs
  - zircon_plugins/particles/runtime/Cargo.toml
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
  - zircon_plugins/prefab_tools/runtime/Cargo.toml
  - zircon_plugins/prefab_tools/runtime/src/lib.rs
  - zircon_plugins/prefab_tools/runtime/src/capability.rs
  - zircon_plugins/rendering/runtime/Cargo.toml
  - zircon_plugins/rendering/runtime/src/lib.rs
  - zircon_plugins/rendering/runtime/src/capability.rs
  - zircon_plugins/solari/runtime/Cargo.toml
  - zircon_plugins/solari/runtime/src/lib.rs
  - zircon_plugins/solari/runtime/src/capability.rs
  - zircon_plugins/solari/runtime/src/plugin.rs
  - zircon_plugins/terrain/runtime/Cargo.toml
  - zircon_plugins/terrain/runtime/src/lib.rs
  - zircon_plugins/terrain/runtime/src/capability.rs
  - zircon_plugins/texture/runtime/Cargo.toml
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
  - zircon_plugins/tilemap_2d/runtime/Cargo.toml
  - zircon_plugins/tilemap_2d/runtime/src/lib.rs
  - zircon_plugins/tilemap_2d/runtime/src/capability.rs
  - zircon_plugins/virtual_geometry/runtime/Cargo.toml
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/virtual_geometry/runtime/src/capability.rs
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/capability.rs
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
  - zircon_plugins/prefab_tools/runtime/src/plugin.rs
  - zircon_plugins/prefab_tools/runtime/src/tests.rs
  - zircon_plugins/prefab_tools/editor/src/lib.rs
  - zircon_plugins/prefab_tools/editor/src/authoring.rs
  - zircon_plugins/prefab_tools/editor/src/capability.rs
  - zircon_plugins/prefab_tools/editor/src/extension_ids.rs
  - zircon_plugins/prefab_tools/editor/src/plugin.rs
  - zircon_plugins/prefab_tools/editor/src/tests.rs
  - zircon_plugins/terrain/runtime/src/plugin.rs
  - zircon_plugins/terrain/runtime/src/tests.rs
  - zircon_plugins/terrain/editor/src/lib.rs
  - zircon_plugins/terrain/editor/src/authoring.rs
  - zircon_plugins/terrain/editor/src/capability.rs
  - zircon_plugins/terrain/editor/src/extension_ids.rs
  - zircon_plugins/terrain/editor/src/plugin.rs
  - zircon_plugins/terrain/editor/src/tests.rs
  - zircon_plugins/tilemap_2d/runtime/src/plugin.rs
  - zircon_plugins/tilemap_2d/runtime/src/tests.rs
  - zircon_plugins/tilemap_2d/editor/src/lib.rs
  - zircon_plugins/tilemap_2d/editor/src/authoring.rs
  - zircon_plugins/tilemap_2d/editor/src/capability.rs
  - zircon_plugins/tilemap_2d/editor/src/extension_ids.rs
  - zircon_plugins/tilemap_2d/editor/src/plugin.rs
  - zircon_plugins/tilemap_2d/editor/src/tests.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/plugin_sdk_examples/editor
  - zircon_plugins/plugin_sdk_examples/plugin.toml
  - zircon_plugins/plugin_sdk_examples/editor/Cargo.toml
  - zircon_plugins/plugin_sdk_examples/editor/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/capability.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extensions.rs
  - tools/plugin_structure_audits/skeleton.py
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime_interface/src/plugin_api.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/engine-architecture/large-file-ownership-m1.md
implementation_files:
  - docs/zircon_plugins/plugin-manifest-schema.md
  - docs/zircon_plugins/plugin-crate-skeleton.md
  - docs/zircon_plugins/plugin-sdk.md
  - docs/zircon_plugins/plugin-sdk-examples-editor.md
  - tools/audit_plugin_structure.py
  - tools/plugin_structure_audits/manifest_schema.py
  - tools/plugin_structure_audits/skeleton.py
  - tools/plugin_structure_audits/registration.py
  - tools/plugin_structure_audits/capability.py
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/editor.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/runtime_exports.rs
  - zircon_plugins/plugin_sdk/src/manifest/feature_bundle_builder.rs
  - zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs
  - zircon_plugins/plugin_sdk/src/test.rs
  - zircon_plugins/animation/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/src/capability.rs
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/audio/runtime/Cargo.toml
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/capability.rs
  - zircon_plugins/asset_importers/audio/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/texture/runtime/Cargo.toml
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/capability.rs
  - zircon_plugins/asset_importers/texture/runtime/src/plugin.rs
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/capability.rs
  - zircon_plugins/physics/runtime/Cargo.toml
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/capability.rs
  - zircon_plugins/physics/runtime/src/plugin.rs
  - zircon_plugins/physics/runtime/src/tests.rs
  - zircon_plugins/physics/editor/src/lib.rs
  - zircon_plugins/physics/editor/src/capability.rs
  - zircon_plugins/physics/editor/src/extension_ids.rs
  - zircon_plugins/physics/editor/src/plugin.rs
  - zircon_plugins/physics/editor/src/tests.rs
  - zircon_plugins/hybrid_gi/runtime/Cargo.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/capability.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/representation.rs
  - zircon_plugins/navigation/runtime/Cargo.toml
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/src/capability.rs
  - zircon_plugins/net/runtime/Cargo.toml
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/capability.rs
  - zircon_plugins/particles/runtime/Cargo.toml
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
  - zircon_plugins/prefab_tools/runtime/Cargo.toml
  - zircon_plugins/prefab_tools/runtime/src/lib.rs
  - zircon_plugins/prefab_tools/runtime/src/capability.rs
  - zircon_plugins/rendering/runtime/Cargo.toml
  - zircon_plugins/rendering/runtime/src/lib.rs
  - zircon_plugins/rendering/runtime/src/capability.rs
  - zircon_plugins/solari/runtime/Cargo.toml
  - zircon_plugins/solari/runtime/src/lib.rs
  - zircon_plugins/solari/runtime/src/capability.rs
  - zircon_plugins/solari/runtime/src/plugin.rs
  - zircon_plugins/terrain/runtime/Cargo.toml
  - zircon_plugins/terrain/runtime/src/lib.rs
  - zircon_plugins/terrain/runtime/src/capability.rs
  - zircon_plugins/texture/runtime/Cargo.toml
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
  - zircon_plugins/tilemap_2d/runtime/Cargo.toml
  - zircon_plugins/tilemap_2d/runtime/src/lib.rs
  - zircon_plugins/tilemap_2d/runtime/src/capability.rs
  - zircon_plugins/virtual_geometry/runtime/Cargo.toml
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/virtual_geometry/runtime/src/capability.rs
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/capability.rs
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
  - zircon_plugins/prefab_tools/runtime/src/plugin.rs
  - zircon_plugins/prefab_tools/runtime/src/tests.rs
  - zircon_plugins/prefab_tools/editor/src/lib.rs
  - zircon_plugins/prefab_tools/editor/src/authoring.rs
  - zircon_plugins/prefab_tools/editor/src/capability.rs
  - zircon_plugins/prefab_tools/editor/src/extension_ids.rs
  - zircon_plugins/prefab_tools/editor/src/plugin.rs
  - zircon_plugins/prefab_tools/editor/src/tests.rs
  - zircon_plugins/terrain/runtime/src/plugin.rs
  - zircon_plugins/terrain/runtime/src/tests.rs
  - zircon_plugins/terrain/editor/src/lib.rs
  - zircon_plugins/terrain/editor/src/authoring.rs
  - zircon_plugins/terrain/editor/src/capability.rs
  - zircon_plugins/terrain/editor/src/extension_ids.rs
  - zircon_plugins/terrain/editor/src/plugin.rs
  - zircon_plugins/terrain/editor/src/tests.rs
  - zircon_plugins/tilemap_2d/runtime/src/plugin.rs
  - zircon_plugins/tilemap_2d/runtime/src/tests.rs
  - zircon_plugins/tilemap_2d/editor/src/lib.rs
  - zircon_plugins/tilemap_2d/editor/src/authoring.rs
  - zircon_plugins/tilemap_2d/editor/src/capability.rs
  - zircon_plugins/tilemap_2d/editor/src/extension_ids.rs
  - zircon_plugins/tilemap_2d/editor/src/plugin.rs
  - zircon_plugins/tilemap_2d/editor/src/tests.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/src/capability.rs
  - zircon_plugins/asset_importers/data/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/capability.rs
  - zircon_plugins/asset_importers/model/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/src/capability.rs
  - zircon_plugins/asset_importers/shader/runtime/src/plugin.rs
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
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs
  - zircon_runtime/src/tests/plugin_extensions/plugin_workspace_shape.rs
  - zircon_plugins/Cargo.lock
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/Cargo.toml
  - zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_runtime/src/ui/surface/input/editable_text.rs
tests:
  - cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked
  - cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked
  - cargo test -p zircon_runtime --lib plugin_manifest --no-default-features --features core-min --locked
  - python tools/audit_plugin_structure.py --json
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --no-default-features --features native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-native-sdk-m2-0622 --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --no-default-features --features native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-native-sdk-m2-0622 --message-format short --color never -- --test-threads=1
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-native-fixture-sdk-m2-0622 --message-format short --color never
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --features editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-editor-sdk-m2-0622 --message-format short --color never
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-editor-sdk-m2-0622 --message-format short --color never
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-test-runtime-m2-0622 --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-test-runtime-m2-0622 --message-format short --color never test_runtime_builder -- --test-threads=1 --nocapture
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk -p zircon_plugin_animation_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-registration-m3-0622 --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-registration-m3-0622 --message-format short --color never runtime_registration_builder -- --test-threads=1 --nocapture
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-registration-m3-0622 --message-format short --color never animation_registration_contributes_runtime_module -- --test-threads=1 --nocapture
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_data_runtime -p zircon_plugin_asset_importer_model_runtime -p zircon_plugin_asset_importer_shader_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-importer-m3-0622 --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_model_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-importer-m3-0622 --message-format short --color never registration_contributes_stl_ply_and_dxf_importers -- --test-threads=1 --nocapture: blocked by unrelated `MaterialCaptureSeed` / `MaterialRuntime::capture_seed` lib-test drift
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --format-version 1 --no-deps --locked
  - rustfmt --edition 2021 --check zircon_plugins/gltf_importer/runtime/src/lib.rs zircon_plugins/gltf_importer/runtime/src/plugin.rs zircon_plugins/obj_importer/runtime/src/lib.rs zircon_plugins/obj_importer/runtime/src/plugin.rs zircon_plugins/texture_importer/runtime/src/lib.rs zircon_plugins/texture_importer/runtime/src/plugin.rs zircon_plugins/audio_importer/runtime/src/lib.rs zircon_plugins/audio_importer/runtime/src/plugin.rs zircon_plugins/opus_importer/runtime/src/lib.rs zircon_plugins/opus_importer/runtime/src/plugin.rs zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs zircon_plugins/shader_wgsl_importer/runtime/src/plugin.rs zircon_plugins/ui_document_importer/runtime/src/lib.rs zircon_plugins/ui_document_importer/runtime/src/plugin.rs zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: registration_conformance.m3_split_importer_gate_status=split-importer-single-entry-clean, split_importer_free_function_registration_sites=0, split_importer_registration_owner_files=0, m3_importer_gate_status=importer-single-entry-clean on 2026-06-23
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime -p zircon_plugin_obj_importer_runtime -p zircon_plugin_texture_importer_runtime -p zircon_plugin_audio_importer_runtime -p zircon_plugin_opus_importer_runtime -p zircon_plugin_shader_wgsl_importer_runtime -p zircon_plugin_ui_document_importer_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-split-importer-m3-0623 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime warnings
  - rustfmt --edition 2021 --check zircon_plugins/plugin_sdk/src/lib.rs zircon_plugins/plugin_sdk/src/runtime_exports.rs zircon_plugins/animation/runtime/src/lib.rs zircon_plugins/physics/runtime/src/lib.rs zircon_plugins/net/runtime/src/lib.rs: passed 2026-06-23
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk -p zircon_plugin_animation_runtime -p zircon_plugin_physics_runtime -p zircon_plugin_net_runtime --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-d12-export-macro-0623 --message-format short --color never: passed 2026-06-23 and refreshed zircon_plugins/Cargo.lock for physics/net SDK dependencies
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk -p zircon_plugin_animation_runtime -p zircon_plugin_physics_runtime -p zircon_plugin_net_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-d12-export-macro-0623 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime/net/physics warnings
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-d12-export-macro-0623 --message-format short --color never runtime_plugin_exports -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-23 with existing zircon_runtime warnings
  - rustfmt --edition 2021 --check zircon_plugins/plugin_sdk/src/lib.rs zircon_plugins/plugin_sdk/src/runtime_exports.rs zircon_plugins/ai/runtime/src/lib.rs zircon_plugins/animation/runtime/src/lib.rs zircon_plugins/hybrid_gi/runtime/src/lib.rs zircon_plugins/navigation/runtime/src/lib.rs zircon_plugins/net/runtime/src/lib.rs zircon_plugins/particles/runtime/src/lib.rs zircon_plugins/physics/runtime/src/lib.rs zircon_plugins/prefab_tools/runtime/src/lib.rs zircon_plugins/rendering/runtime/src/lib.rs zircon_plugins/solari/runtime/src/lib.rs zircon_plugins/terrain/runtime/src/lib.rs zircon_plugins/texture/runtime/src/lib.rs zircon_plugins/tilemap_2d/runtime/src/lib.rs zircon_plugins/virtual_geometry/runtime/src/lib.rs zircon_plugins/zr_vm_language/runtime/src/lib.rs: passed 2026-06-23
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk -p zircon_plugin_ai_runtime -p zircon_plugin_animation_runtime -p zircon_plugin_hybrid_gi_runtime -p zircon_plugin_navigation_runtime -p zircon_plugin_net_runtime -p zircon_plugin_particles_runtime -p zircon_plugin_physics_runtime -p zircon_plugin_prefab_tools_runtime -p zircon_plugin_rendering_runtime -p zircon_plugin_solari_runtime -p zircon_plugin_terrain_runtime -p zircon_plugin_texture_runtime -p zircon_plugin_tilemap_2d_runtime -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_zr_vm_language_runtime --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-d12-export-macro-0623 --message-format short --color never: passed 2026-06-23 and refreshed zircon_plugins/Cargo.lock for remaining first-party runtime SDK dependencies
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk -p zircon_plugin_ai_runtime -p zircon_plugin_animation_runtime -p zircon_plugin_hybrid_gi_runtime -p zircon_plugin_navigation_runtime -p zircon_plugin_net_runtime -p zircon_plugin_particles_runtime -p zircon_plugin_physics_runtime -p zircon_plugin_prefab_tools_runtime -p zircon_plugin_rendering_runtime -p zircon_plugin_solari_runtime -p zircon_plugin_terrain_runtime -p zircon_plugin_texture_runtime -p zircon_plugin_tilemap_2d_runtime -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_zr_vm_language_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-d12-export-macro-0623 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime and large-plugin warning noise
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --format-version 1 --no-deps --locked: passed 2026-06-23
  - python -m py_compile tools/audit_plugin_structure.py tools/plugin_structure_audits/__init__.py tools/plugin_structure_audits/manifest_schema.py tools/plugin_structure_audits/skeleton.py tools/plugin_structure_audits/registration.py tools/plugin_structure_audits/capability.py: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: capability_conformance.m4_runtime_capability_gate_status=runtime-capability-single-source-clean, audited_runtime_root_count=15, capability_source_mismatches=0 on 2026-06-23
  - rustfmt --edition 2021 --check first-party runtime lib.rs/capability.rs set + zircon_plugins/first_party_runtime_catalog/src/lib.rs + zircon_runtime/src/ui/surface/input/editable_text.rs: passed 2026-06-23
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog -p zircon_plugin_ai_runtime -p zircon_plugin_animation_runtime -p zircon_plugin_hybrid_gi_runtime -p zircon_plugin_navigation_runtime -p zircon_plugin_net_runtime -p zircon_plugin_particles_runtime -p zircon_plugin_physics_runtime -p zircon_plugin_prefab_tools_runtime -p zircon_plugin_rendering_runtime -p zircon_plugin_solari_runtime -p zircon_plugin_terrain_runtime -p zircon_plugin_texture_runtime -p zircon_plugin_tilemap_2d_runtime -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_zr_vm_language_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m4-capability-0623 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime and large-plugin warning noise
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m4-capability-test-nodebug-0623 --message-format short --color never plugins_12_capability_single_source_conformance -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-23 after fixing test-build-only UI IME DeleteSurrounding match exhaustiveness in zircon_runtime/src/ui/surface/input/editable_text.rs
  - python -m py_compile tools/audit_plugin_structure.py tools/plugin_structure_audits/__init__.py tools/plugin_structure_audits/manifest_schema.py tools/plugin_structure_audits/skeleton.py tools/plugin_structure_audits/registration.py tools/plugin_structure_audits/capability.py tools/plugin_structure_audits/dependency_boundary.py: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: capability_conformance.m4_t2_builder_mirror_gate_status=sdk-builder-mirror-clean, sdk_builder_mirror_violations=0 on 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m4-capability-test-nodebug-0623 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime warnings
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m4-capability-test-nodebug-0623 --message-format short --color never feature_bundle_builder_projects_capability_to_feature_and_modules -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --features editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-editor-sdk-m2-0622 --message-format short --color never: passed 2026-06-23 after exporting route policy helpers through zircon_runtime::ui::dispatch
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --features editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-editor-sdk-m2-0622 --message-format short --color never editor_declaration_mirrors_runtime_manifest_and_keeps_editor_capabilities -- --test-threads=1 --nocapture: timed out after 900s on 2026-06-23, not counted as passing; residual cargo/rustc for that target dir cleaned
  - rustfmt --edition 2021 zircon_runtime\src\builtin\runtime_modules\ids\plugin_id.rs zircon_runtime\src\builtin\runtime_modules\plugin_modules\loader.rs zircon_runtime\src\builtin\runtime_modules\tests\registration\structure.rs zircon_runtime\src\tests\plugin_extensions\plugin_workspace_shape.rs: passed 2026-06-23 for Plugins 12 M5/T2 D6 RuntimePluginId string-newtype
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-plugin-id-check --message-format short --color never: passed 2026-06-23 with existing warning noise
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-plugin-id-sdk-check --message-format short --color never: passed 2026-06-23 with existing warning noise
  - cargo test -p zircon_runtime --lib --no-default-features --features core-min runtime_plugin_id_accepts_external_keys_without_core_variant --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-plugin-id-check --message-format short --color never -- --test-threads=1 --nocapture: blocked before running by unrelated runtime lib-test compile drift; not counted as passed
  - rustfmt --edition 2021 --check importer runtime lib.rs/capability.rs set for audio/gltf/obj/opus/shader_wgsl/texture/ui_document and asset_importers/data/model/shader: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=25 and plugin_skeleton_gate.migration_debt_count=25 on 2026-06-23
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p each of zircon_plugin_audio_importer_runtime, zircon_plugin_gltf_importer_runtime, zircon_plugin_obj_importer_runtime, zircon_plugin_opus_importer_runtime, zircon_plugin_shader_wgsl_importer_runtime, zircon_plugin_texture_importer_runtime, zircon_plugin_ui_document_importer_runtime, zircon_plugin_asset_importer_data_runtime, zircon_plugin_asset_importer_model_runtime, zircon_plugin_asset_importer_shader_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-skeleton-split-importers-check --message-format short --color never: each package exit=0 on 2026-06-23 with existing warning noise
  - rustfmt --edition 2021 --check runtime-only skeleton owner files for ai, asset_importers/audio, asset_importers/texture, solari, zr_vm_language: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: skeleton_conformance.migration_debt_count=20, plugin_skeleton_gate.migration_debt_count=20, standalone_distribution_conformance.dist_capable_plugin_count=1 on 2026-06-23
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ai_runtime -p zircon_plugin_asset_importer_audio_runtime -p zircon_plugin_asset_importer_texture_runtime -p zircon_plugin_solari_runtime -p zircon_plugin_zr_vm_language_runtime --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-only-skeleton-check --message-format short --color never: passed 2026-06-23 and refreshed zircon_plugins/Cargo.lock for audio/texture SDK dependencies
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ai_runtime -p zircon_plugin_asset_importer_audio_runtime -p zircon_plugin_asset_importer_texture_runtime -p zircon_plugin_solari_runtime -p zircon_plugin_zr_vm_language_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-only-skeleton-check --message-format short --color never: passed 2026-06-23 with existing zircon_runtime warning noise
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_audio_runtime -p zircon_plugin_asset_importer_texture_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-runtime-only-skeleton-check --message-format short --color never package_declares_ -- --test-threads=1 --nocapture: timed out after 904s on 2026-06-23 with no test result; residual cargo/rustc processes for this target-dir were stopped; not counted as passing
  - python -m py_compile tools/plugin_structure_audits/capability.py tools/audit_plugin_structure.py: passed 2026-06-23 after allowing runtime_capabilities() to be owned in plugin.rs and re-exported from lib.rs
  - rustfmt --edition 2021 --check editor-only skeleton owner files for native_window_hosting, runtime_diagnostics, ui_asset_authoring: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: capability_source_mismatches=0, m4_runtime_capability_gate_status=runtime-capability-single-source-clean, skeleton_conformance.migration_debt_count=17, plugin_skeleton_gate.migration_debt_count=17, standalone_distribution_conformance.dist_capable_plugin_count=1 on 2026-06-23
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_native_window_hosting_editor -p zircon_plugin_runtime_diagnostics_editor -p zircon_plugin_ui_asset_authoring_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-m5-editor-small-skeleton-check --message-format short --color never: blocked before compiling the target editor plugin packages by unrelated zircon_editor retained-host compile drift (`retained_host/app/viewport/toolbar_pointer/click.rs` unresolved HostWindowPresentationData import and E0282 inference); not counted as passing
  - rustfmt --edition 2021 authoring runtime/editor skeleton owner files for prefab_tools, terrain, tilemap_2d: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: capability_source_mismatches=0, skeleton_conformance.migration_debt_count=14, plugin_skeleton_gate.migration_debt_count=14, standalone_distribution_conformance.dist_capable_plugin_count=1 on 2026-06-23
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_prefab_tools_runtime -p zircon_plugin_terrain_runtime -p zircon_plugin_tilemap_2d_runtime --offline --jobs 1 --target-dir E:\cargo-targets\zircon-plugin-m5-authoring-runtime-0623 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime warning noise
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_prefab_tools_editor -p zircon_plugin_terrain_editor -p zircon_plugin_tilemap_2d_editor --offline --jobs 1 --target-dir E:\cargo-targets\zircon-plugin-m5-authoring-editor-0623 --message-format short --color never: timed out twice while building dependencies; no final output captured, not counted as passing
  - rustfmt --edition 2021 particles/physics/texture runtime/editor skeleton owner files plus zircon_plugins/particles/runtime/src/simulation/cpu.rs: passed 2026-06-23
  - python tools/audit_plugin_structure.py --json: capability_source_mismatches=0, skeleton_conformance.migration_debt_count=11, plugin_skeleton_gate.migration_debt_count=11, standalone_distribution_conformance.dist_capable_plugin_count=1 on 2026-06-23
  - cargo check -p zircon_plugin_particles_runtime --offline; cargo check -p zircon_plugin_physics_runtime --offline; cargo check -p zircon_plugin_texture_runtime --offline: passed 2026-06-23 with existing warning noise
  - cargo check --manifest-path zircon_plugins\particles\editor\Cargo.toml --offline; cargo check --manifest-path zircon_plugins\physics\editor\Cargo.toml --offline --target-dir target\codex-plugin-validation; cargo check --manifest-path zircon_plugins\texture\editor\Cargo.toml --offline --target-dir target\codex-plugin-validation: passed 2026-06-23 with existing warning noise
  - rustfmt --edition 2021 zircon_plugins\editor_build_export_desktop\editor\src\lib.rs zircon_plugins\editor_build_export_desktop\editor\src\capability.rs zircon_plugins\editor_build_export_desktop\editor\src\extension_ids.rs zircon_plugins\editor_build_export_desktop\editor\src\plugin.rs zircon_plugins\editor_build_export_desktop\editor\src\tests.rs zircon_plugins\editor_build_export_desktop\editor\src\export_wizard.rs: passed 2026-06-23
  - python tools\audit_plugin_structure.py --json: capability_source_mismatches=0, skeleton_conformance.migration_debt_count=10, plugin_skeleton_gate.migration_debt_count=10, standalone_distribution_conformance.dist_capable_plugin_count=1 on 2026-06-23
  - cargo check --manifest-path zircon_plugins\editor_build_export_desktop\editor\Cargo.toml --all-targets --offline --target-dir target\codex-plugin-validation --message-format short --color never: passed 2026-06-23 with existing runtime/editor warning noise
  - rustfmt --edition 2021 sound runtime/editor skeleton owner files for main crate plus ray_traced_convolution_reverb and timeline_animation_track feature crates: passed 2026-06-23
  - python tools\audit_plugin_structure.py --json: capability_source_mismatches=0, skeleton_conformance.migration_debt_count=9, plugin_skeleton_gate.migration_debt_count=9, standalone_distribution_conformance.dist_capable_plugin_count=1 on 2026-06-23 after sound owner rollout
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --lib --offline --target-dir target\codex-plugin-validation --message-format short --color never: blocked before target sound crate by unrelated zircon_runtime render compile drift (`MeshPassCommandBuffers` / `CachedMeshDrawLookup` / `mesh_draw` / `mesh_pipeline_cache` imports); not counted as passing
  - rustfmt --edition 2021 --check zircon_plugins\timeline_sequence\editor\src\lib.rs zircon_plugins\timeline_sequence\editor\src\capability.rs zircon_plugins\timeline_sequence\editor\src\extension_ids.rs zircon_plugins\timeline_sequence\editor\src\plugin.rs zircon_plugins\timeline_sequence\editor\src\tests.rs: passed 2026-06-23
  - python tools\audit_plugin_structure.py --json: capability_source_mismatches=0, skeleton_conformance.migration_debt_count=8, plugin_skeleton_gate.migration_debt_count=8, standalone_distribution_conformance.dist_capable_plugin_count=1 on 2026-06-23 after timeline_sequence owner rollout
  - cargo check --manifest-path zircon_plugins\timeline_sequence\editor\Cargo.toml --lib --offline --target-dir target\codex-plugin-validation --message-format short --color never: blocked before target timeline_sequence crate by unrelated zircon_runtime render compile drift (`MeshPassCommandBuffers` / `CachedMeshDrawLookup` / `mesh_draw` / `mesh_pipeline_cache` imports); not counted as passing
  - python tools\audit_plugin_structure.py --json: missing_plugin_toml=0, manifest_schema_violations=0, capability_source_mismatches=0, missing_capability_owner_files=0, missing_runtime_capability_exports=0, plugin_skeleton_gate.m2_gate_status=sample-clean-migration-debt-clear, skeleton_conformance.migration_debt_count=0, plugin_skeleton_gate.migration_debt_count=0, skeleton_conformance.migration_debt_roots=[] on 2026-06-23 after final owner rollout for animation, animation_graph, hybrid_gi, material_editor, navigation, net, rendering, and virtual_geometry
  - rustfmt --edition 2021 --check on 139 touched owner/façade files under animation, animation_graph, hybrid_gi, material_editor, navigation, net, rendering, timeline_sequence, and virtual_geometry: passed 2026-06-23
  - cargo check --manifest-path zircon_plugins\Cargo.toml --workspace --offline --locked --target-dir target\codex-plugin-validation --message-format short --color never: passed 2026-06-23 after final owner rollout and `hybrid_gi` RenderLayerSet mask alignment, with existing warning noise
  - cargo fmt --all --check
doc_type: structure-plan
status: in_progress
---

# 12 · 插件 DX 与结构框架统一计划（P1 横切）

> 状态：in_progress · 优先级：P1（横切，与 10 编辑器集成、11 调用桥并列横切层）
> 上游权威：[`docs/plans/engine-code-structure-convention.md`](../engine-code-structure-convention.md) §6（Plugin DX）
> 锚定：以 Plugins [01 插件架构核心](01-plugin-architecture-core.md) 的定稿名（`RuntimePlugin::register`、`PluginPackageManifest`、ABI v3）为基；本计划把"统一 manifest + 唯一 crate 骨架 + 注册单入口 + capability 单源 + plugin_sdk builder"落成可开工框架，让插件开发体验统一友好。

## 1. 目标

当前 81 个插件 crate 的开发体验割裂：双导入器架构、`plugin.toml` schema 发散（30–105 行，部分缺失）、注册模式二分、capability 常量分散无单源、native 双 crate 同名。本计划定稿**唯一插件骨架 + 统一 manifest schema**，使：

1. 新插件 ≈ 一文件声明（`plugin_sdk` builder），day 1 即合规。
2. 存量插件按 touch-it-conform-it 增量硬切到骨架，`migration_debt` 递减至 0。
3. reviewer 面对任意插件看到同一目录骨架、同一 manifest schema、同一注册入口、单源 capability。

## 2. 现状缺口（按代码实查，带路径证据）

| # | 缺口 | 规范条目 | 证据路径 |
|---|------|---------|---------|
| S1 | 双导入器架构（注册逻辑位置不一） | §6.1 / §6.3 | `gltf_importer/runtime/src/lib.rs`（含注册）vs `asset_importers/model/runtime/src/registration.rs`（自由函数分离） |
| S2 | `plugin.toml` schema 发散 / 缺失 | §6.2 | `sound/plugin.toml`(105) vs `gltf_importer/plugin.toml`(40) vs `asset_importers/*`（**无 manifest**，配置在 `registration.rs`） |
| S3 | 注册模式二分 | §6.3 | `impl RuntimePlugin`（animation 等）vs `plugin_registration()` 自由函数（asset_importers） |
| S4 | capability 常量分散、无单源 | §6.4 | `gltf_importer` 有 `RUNTIME_CAPABILITY`、`animation` 无该常量；与 `plugin.toml` 人工同步 |
| S5 | native crate_name 双声明同名 | §6.1 | `native_dynamic_fixture/plugin.toml` 两 `[[modules]]` 指向同一 `..._native` crate |
| S6 | `registration.rs` 单文件多职责 | R1.3 | `asset_importers/model/runtime/src/registration.rs`(161 行：descriptors + manifest + plugin_registration) |
| S7 | **`plugin.toml` 对 ~28 静态链接插件是"死副本"**：仅 native_dynamic 打包与 scripts 读取，静态插件真源是 Rust descriptor，toml 无消费方却要人工同步（ROI 高于 S1-S6） | §6.2 | `plugin/export_build_plan/materialize/package_lookup.rs:17,32`(唯一消费) vs `animation/plugin.toml`(无加载方) |
| S8 | **native 插件零 SDK 复用**：手写 ~720 行 ABI v3 `repr(C)`（~15 结构 + owner_token 内存 + panic catch + capability 切分） | §6.1 / §6.5 | `native_dynamic_fixture/native/src/lib.rs:102-242,587-719` |
| S9 | capability 名单插件内重复 3 次 + manifest 3 段 = 改一名动 6 处；常量定义三套不一（const / 字面量 / import） | §6.4 | `physics/runtime/src/lib.rs:67-73,74-101,122-131` + `physics/plugin.toml:9,16-45,52` |
| S10 | core workspace dependency inheritance 已进入全局 guard | §6.1 | `zircon_plugins/Cargo.toml` 拥有 `[workspace.dependencies]`；117 个 `zircon_runtime` / `zircon_editor` / `zircon_runtime_interface` dependency 引用已统一为 `workspace = true`，`zircon_plugins/Cargo.lock` 已随离线解析同步，`core_workspace_dependency_status = core-workspace-deps-clean` |
| S11 | editor authoring macro consumer guard 已覆盖 animation/physics/net；runtime↔editor mirror 由 SDK declaration path 统一表达 | §6.1 / §6.4 | `zircon_plugin_sdk::authoring_plugin!` 现在生成 animation/physics/net editor plugin struct/Default/declaration/EditorPlugin impl，并通过 `mirrors_runtime_manifest:` mirror 对应 runtime manifest；`d5_editor_authoring_macro_consumers_static_passed_cargo_deferred` 与 `review_d5_editor_authoring_plugins_use_sdk_macro` 锁定不回退手写样板 |
| S12 | `RuntimePluginId` 曾是 **core 封闭枚举**，新一方插件必须改引擎核心 + 同步 key/label match（插件无法自带 id） | 架构 | 已由 M5/T2 D6 改为开放 string-newtype；内建 id 保留关联常量，外部合法 key 不再需要 core 分支 |

> S7-S12 来自 [`engine-code-review-findings-2026-06.md`](../engine-code-review-findings-2026-06.md)（D 系列）。另有 native manifest 双写已漂移（D3）、三步注册样板（D8）、6 个转发自由函数（D12）、importer 样板分叉（D13）、跨插件测试 fixture（D11，当前已迁移到 SDK TestRuntime）、插件间调用无桥（D10）等已并入 §4 里程碑。

## 3. 目标骨架（收敛后形态，见规范 §6.1）

```
<plugin>/
  plugin.toml          # 统一 schema（强制）
  runtime/
    Cargo.toml
    src/
      lib.rs           # 薄：pub use 公共 API + Plugin struct + 常量
      plugin.rs        # 唯一注册 owner：impl RuntimePlugin::register
      capability.rs    # capability id pub const —— 单一来源
      contract/        # ABI-safe DTO（纯消费 interface 则省略）
      backend/         # 算法 / importer / 协议实现 owner
      systems/         # 注册进调度图的 ECS 系统
      tests/
  editor/              # 镜像同骨架（能力对称）
    Cargo.toml
    src/{lib.rs, plugin.rs, capability.rs, ...}
```

## 4. 里程碑（任务级执行蓝本）

切片期 `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked`；里程碑末进测试。
**波次定稿**：M1–M4 = **波次零（结构前置基建）**，先于 plugins index §2 能力波次一~五；M5 = touch-it-conform-it，存量插件随其 02–11 能力波次同窗口迁骨架。

| 里程碑 | 任务 | 改动文件（代表） | 依赖 | 验收命令 / 测试函数 |
|---|---|---|---|---|
| **M1 统一 manifest schema** | T1 schema owner 文档 | `docs/zircon_plugins/plugin-manifest-schema.md`（必选 / 可选段定稿） | 01 定稿名 | 人工 review + 链接可达 |
| | T2 manifest 校验器 | `zircon_runtime/src/plugin/package_manifest/*`（扩 schema 校验） | T1 | `cargo test -p zircon_runtime --lib plugin_manifest` |
| | T3 补齐缺失 / 对齐发散 manifest | `asset_importers/*/plugin.toml`(新增)、`sound`/`gltf_importer` 段形对齐 | T2 | `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets` |
| | T4 schema 一致 guard | `tools/plugin_structure_audits/*` + workspace guard | T3 | `plugins_12_manifest_schema_uniform`（`missing_plugin_toml = 0`、`manifest_schema_violations = 0`） |
| | T5 静态插件 `plugin.toml` 改 `@generated`（descriptor `package_manifest()` 派生）+ native 双写改 `include_str!` | 生成器 / `native_dynamic_fixture` | T1 | `plugins_12_static_plugin_manifest_is_generated`（D-S7/D3） |
| **M2 骨架 + SDK** | T1 `plugin_sdk` builder crate | `zircon_plugins/plugin_sdk/`（固化 `plugin_sdk_examples`） | M1 | `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets` |
| | T2 骨架模板文档 | `docs/zircon_plugins/plugin-crate-skeleton.md` | M1 | `plugins_12_crate_skeleton_conformance`（首批样板插件） |
| | T3 `plugin_sdk` native feature（导出 ABI 结构 + helper 宏） | `plugin_sdk/native` | T1 | native 插件作者只写 `invoke_command`（D-S8） |
| | T4 `authoring_plugin!` editor 宏 + `[workspace.dependencies]` 继承 | `plugin_sdk/editor`、`zircon_plugins/Cargo.toml` | T1 | editor 插件一行生成（D5）；`*.workspace = true`（D7） |
| | T5 `plugin_sdk::test::TestRuntime::builder()` fixture | `plugin_sdk/test` | T1 | 消跨插件测试 fixture 样板（D11） |
| **M3 注册收编** | T1 自由函数 → trait | `asset_importers/*/runtime/src/{registration.rs → plugin.rs}` | M2 | `plugins_12_single_registration_entry`（`free_function_registration_sites = 0`） |
| | T2 `builder.module(desc).system(sys).register(registry)` 封装三步顺序 + 收编 6 个转发自由函数 | `plugin_sdk` builder、各插件 `plugin.rs` | M2,T1 | 作者不接触 owner token（D8/D12）；importer 收编含 selection 派生（D13） |
| **M4 capability 单源** | T1 `capability.rs` 常量 + 四源 guard | 各插件 `capability.rs`、四源 guard | M3 | `plugins_12_capability_single_source`（`capability_source_mismatches = 0`） |
| | T2 builder 一次声明喂 descriptor/manifest/数组/status + editor `mirrors_runtime(...)` 对称 guard | `plugin_sdk` builder | T1 | 改一名只动 1 处（D1）；runtime↔editor 对称校验（D9） |
| | T3 弱依赖调用桥对齐（与 Plugins 11）：`bridge.call("physics.query", payload)` | `plugin_sdk`、对齐 11 | M3 | core 不再充当跨插件类型枢纽（D10） |
| **M5 存量硬切** | T1 touch-it-conform-it 增量迁移 | 各插件随其 02–11 能力波次同窗口迁骨架 | M2–M4 | `plugin_skeleton_gate.migration_debt_count` 递减至 0 |
| | T2 `RuntimePluginId` 封闭枚举 → string-newtype / `Custom(&str)`（触及 core，跨计划） | `builtin/runtime_modules/ids/plugin_id.rs` | M2 | 新插件不再改引擎核心（D6） |

## 5. 审计与 guard 契约（本计划新增）

- **审计脚本**：新建仓库根聚合器 `tools/audit_plugin_structure.py` 与 owner 域目录 `tools/plugin_structure_audits/`；当前 M1/T4 先落 `manifest_schema.py`，后续 M2/M3/M4 继续补 skeleton / registration / capability owner。字段：
  - `skeleton_conformance`、`missing_plugin_toml`、`manifest_schema_violations`
  - `capability_conformance.capability_source_mismatches`、`capability_conformance.m4_runtime_capability_gate_status`、`capability_conformance.m4_t2_builder_mirror_gate_status`（四源一致性 + SDK builder/editor mirror guard）
  - `free_function_registration_sites`、`native_crate_name_collisions`
  - `registration_conformance.m3_t1_gate_status`、`registration_conformance.m3_split_importer_gate_status`、`registration_conformance.m3_importer_gate_status`
  - `asset_importer_family_free_function_registration_sites`、`split_importer_free_function_registration_sites`、`importer_free_function_registration_sites`
  - `registration_conformance.runtime_registration_builder_violation_count`、`registration_conformance.m3_t2_runtime_registration_builder_status`（D8 animation/physics/net 原始证据路径）
  - `capability_conformance.editor_runtime_mirror_root_count`、`capability_conformance.editor_runtime_mirror_violations`、`capability_conformance.d9_editor_runtime_mirror_gate_status`（D9 animation/physics/net editor/runtime mirror consumers）
  - `review_d5_editor_authoring_plugins_use_sdk_macro` + `d5_editor_authoring_macro_consumers_static_passed_cargo_deferred`（D5 animation/physics/net editor authoring macro consumers）
  - `oversized_files`、`exempt`（vendored upstream / `native_dynamic_fixture` / `plugin_sdk_examples` / `@generated`）
  - `plugin_skeleton_gate.m2_gate_status`、`classification_counts`、`migration_debt_count`
- **guard 测试**：落 `zircon_plugins` workspace（`first_party_runtime_catalog` 或新建 `plugin_sdk` crate 的 `tests/`），含 `plugins_12_plugin_skeleton_mirror_docs_match_structure_audit_counts`。
- **行数阈值**：软 800 / 硬 1000，沿用全引擎口径；豁免须登记 `exempt`。

## 6. 硬切 checklist

- [ ] 旧自由函数注册路径已删除，调用方走 `impl RuntimePlugin::register`
- [ ] 缺失 `plugin.toml` 已补齐且过 schema 校验
- [x] capability 常量为 `capability.rs` 单源，四源一致（2026-06-23 已完成 15 个 trait-backed first-party runtime 根的 M4/T1 首批 guard，并完成 `PluginFeatureBundleBuilder` + editor `mirrors_runtime(...)` 的 M4/T2 SDK guard；2026-06-28 D5 editor authoring macro consumer guard 已让 animation/physics/net editor plugin 使用 `zircon_plugin_sdk::authoring_plugin!` 生成 struct/Default/declaration/EditorPlugin impl，状态锚 `d5_editor_authoring_macro_consumers_static_passed_cargo_deferred`，守卫 `review_d5_editor_authoring_plugins_use_sdk_macro`；同日 D9 editor/runtime mirror consumer guard 已让 animation/physics/net editor plugin 通过 SDK macro 的 `mirrors_runtime_manifest:` / `EditorPluginDeclaration::mirrors_runtime_manifest` mirror runtime manifest，审计输出 `editor_runtime_mirror_violations = 0`、`d9_editor_runtime_mirror_gate_status = editor-runtime-mirror-clean`；同日 M5/T1 importer capability-owner、runtime-only skeleton owner、editor-only skeleton owner、authoring runtime/editor skeleton owner、particles/physics/texture skeleton owner、editor_build_export_desktop skeleton owner、sound runtime/editor feature skeleton owner、timeline_sequence editor skeleton owner 与最终 8 根 owner rollout 已把 skeleton migration debt 从 35 降至 0；`migration_debt_roots = []`）
- [ ] native 双 crate_name 已显式区分 runtime / editor
- [ ] 无兼容 re-export / 双轨；删除清单写进提交说明

## 7. 完成定义

`m1_gate_status = classified-and-clear`、`plugin_skeleton_gate.m2_gate_status = sample-clean-migration-debt-clear`、M5 收口后 `migration_debt_count = 0`、各 violation 字段 = 0、`exempt` 仅含登记豁免项、镜像守卫绿；`cargo test --manifest-path zircon_plugins/Cargo.toml --workspace`、`cargo build --manifest-path zircon_plugins/Cargo.toml --workspace`、`cargo fmt --all --check`、`tools/audit_plugin_structure.py --json` 无 risk。

## 8. 联动

- M1 manifest schema 与 Plugins 01 §3 的 `PluginPackageManifest` 字段对齐，扩 schema 校验不破坏 01 已落 `declared_system_anchors_are_registered`；capability 四源 guard 是 01 已提"四源一致性"的机器化落地。
- plugins index §2 依赖图：01 定稿 → 12 schema/骨架 → 02–11 照用；本计划列为波次零前置。
- M5 与各插件能力计划（02 Sound … 11 调用桥）同窗口：插件被实质改动时同一变更迁骨架，避免一次性大爆炸冻结全工作区。

## 9. 状态与产出记录

| 日期 | 里程碑 | 切片 | 状态 | 证据 |
|---|---|---|---|---|
| 2026-06-28 | M4 capability 单源 | D10 animation/physics bridge call migration | d10_animation_physics_bridge_call_static_passed_cargo_deferred | `zircon_runtime/src/core/framework/physics/query_interface.rs` 新增 runtime-owned `PhysicsQueryInterface` / `physics.query.v1`，`zircon_plugins/plugin_sdk/src/registration.rs` 新增 owner-tracked `RuntimePluginModuleRegistration::export_interface::<T>(...)`，SDK runtime surface re-export `PluginInterface`、`WeakBridge`、`BridgeError`。`zircon_plugins/physics/runtime/src/plugin.rs` 声明并导出 `physics.query.v1`，`zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/runtime_helpers.rs` 从 `runtime.extension_report().registry.frozen_bridge_table()` 解析 `WeakBridge<dyn PhysicsQueryInterface>`，主合约测试通过 bridge `.call(...)` 覆盖 ray cast、shape overlap 与 shape cast，不再解析 concrete manager。`review_d10_animation_physics_tests_use_sdk_bridge_call` 锁定源码/docs/status；Cargo gate deferred。 |
| 2026-06-28 | M2 骨架 + SDK | D11 animation/physics TestRuntime fixture migration | d11_animation_physics_test_runtime_fixture_static_passed_cargo_deferred | `zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs` 已从手写 CoreRuntime/Scene/fixed-step fixture 迁到 `zircon_plugin_sdk::TestRuntime::builder()`：helper 通过 `.with_runtime_plugin(&physics_plugin)` / `.with_runtime_plugin(&animation_plugin)` 组装 runtime，测试用 `runtime.create_default_level()` 与 `runtime.tick_level_seconds(...)` 驱动行为覆盖。大块 animation asset fixture 拆入 `runtime_physics_animation_tick_contract/animation_assets.rs`，runtime helper 拆入 `runtime_physics_animation_tick_contract/runtime_helpers.rs`，target-id fallback 行为拆入 `runtime_physics_animation_tick_contract/target_resolution.rs`，主测试保持 969 行。新增 `review_d11_animation_physics_tests_use_sdk_test_runtime_fixture` 锁定源码、SDK fixture、docs/status 和旧 CoreRuntime fixture 样板不回流；Cargo gate deferred。 |
| 2026-06-28 | M3 注册收编 | D12 runtime helper export macro rollout review/status sync | d12_runtime_export_macro_review_synced_static_passed_cargo_deferred | 本轮复核 Plugins 12 M3/D12 的现状并补 D12 runtime helper export macro rollout review/status guard：`zircon_plugin_sdk::runtime_plugin_exports!` 继续拥有 `runtime_plugin()`、`package_manifest()`、`runtime_selection()` 与 `plugin_registration()` 四个标准 runtime helper；15 个 first-party trait-backed runtime roots（ai、animation、hybrid_gi、navigation、net、particles、physics、prefab_tools、rendering、solari、terrain、texture、tilemap_2d、virtual_geometry、zr_vm_language）均在 `runtime/src/plugin.rs` 使用该 macro，且不保留 crate-local 手写 helper block。新增 `review_d12_runtime_helper_exports_use_sdk_macro` 与 folder-backed child owner `tests/runtime_absorption/code_review_findings/plugin_importer_dx/d12_runtime_exports.rs`，同步 Runtime 15、engine review findings、结构规范、plugin SDK 和 module-convention docs；本切片只锁定已完成状态，不改插件生产行为，Cargo gate deferred。 |
| 2026-06-28 | M4 capability 单源 | D1 capability single-source review/status sync | d1_capability_single_source_review_synced_static_passed_cargo_deferred | 本轮复核 Plugins 12 M4/T1 + T2 并补 D1 review/status guard：15 个 trait-backed first-party runtime roots 继续以 `runtime/src/capability.rs` 的 `RUNTIME_CAPABILITIES` 作为唯一 capability source，`zircon_plugins/first_party_runtime_catalog/src/lib.rs::plugins_12_capability_single_source_conformance` 消费 `tools/audit_plugin_structure.py --json` 并锁定 `plugins_12_runtime_capability_single_source_guard_passed`、`m4_runtime_capability_gate_status = runtime-capability-single-source-clean`、`capability_source_mismatches = 0`。SDK 侧 `PluginFeatureBundleBuilder` 继续把一次声明投影到 feature/runtime/editor module manifest，审计输出 `m4_t2_builder_mirror_gate_status = sdk-builder-mirror-clean`、`sdk_builder_mirror_violations = 0`。新增 `review_d1_plugin_capabilities_use_single_source_and_sdk_builder_mirror` 与 folder-backed child owner `tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source.rs`，同步 Runtime 15、engine review findings、结构规范、plugin SDK、plugin structure audit docs、module-convention docs 与 session note；本切片只锁定已完成状态，不改插件生产行为，Cargo gate deferred。 |
| 2026-06-28 | M2 骨架 + SDK | D5 editor authoring macro consumer guard | d5_editor_authoring_macro_consumers_static_passed_cargo_deferred | `zircon_plugins/plugin_sdk/src/editor.rs` 的 `zircon_plugin_sdk::authoring_plugin!` 现在接受 `mirrors_runtime_manifest:`，并继续生成 editor plugin struct、`Default`/`new`、declaration access、package manifest projection、registration report helper 与 `zircon_editor::EditorPlugin` impl。`zircon_plugins/animation/editor/src/plugin.rs`、`zircon_plugins/physics/editor/src/plugin.rs`、`zircon_plugins/net/editor/src/plugin.rs` 均删除本地 `impl zircon_editor::EditorPlugin` 与 `EditorPluginDeclaration::new(...)` 样板，改由宏声明 package id/display/category/maturity/capabilities/runtime manifest mirror 和 plugin-specific extension registration body。`review_d5_editor_authoring_plugins_use_sdk_macro` 锁定源码/SDK macro/docs/status，Cargo gate deferred。 |
| 2026-06-28 | M4 capability 单源 | D9 editor/runtime mirror consumer guard | d9_editor_runtime_mirror_consumers_static_passed_cargo_deferred | `zircon_plugins/animation/editor/src/plugin.rs`、`zircon_plugins/physics/editor/src/plugin.rs`、`zircon_plugins/net/editor/src/plugin.rs` 现在通过 `zircon_plugin_sdk::authoring_plugin!` 的 `mirrors_runtime_manifest:` 进入 `EditorPluginDeclaration::mirrors_runtime_manifest` projection 并继承对应 runtime `package_manifest()`；三个 editor tests 均断言 `mirrored_runtime_package_id()`，并验证 package manifest 同时包含 runtime capability 与 editor authoring capability。`tools/plugin_structure_audits/capability.py` 扫描 `editor_runtime_mirror_roots = ["animation", "physics", "net"]`，`tools/audit_plugin_structure.py --json` 报告 `editor_runtime_mirror_violations = 0`、`d9_editor_runtime_mirror_gate_status = editor-runtime-mirror-clean`；`review_d9_editor_runtime_mirror_consumers_use_sdk_declaration` 锁定源码/审计/docs/status。Cargo gate deferred。 |
| 2026-06-28 | M3 注册收编 | D8 runtime registration builder original evidence paths | d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred | `RuntimePluginRegistrationBuilder` / `RuntimePluginModuleRegistration` 现在覆盖 D8 原始证据路径 `animation`、`physics`、`net`：三个 runtime plugin owner 均通过 `RuntimePluginRegistrationBuilder::new(registry).module(PLUGIN_RUNTIME_MODULE_NAME, module_descriptor())` 完成 module owner + descriptor registration，通过 module handle 注册 runtime scene systems；`RuntimePluginModuleRegistration::event(...)`、`plugin_option(...)` 与 `plugin_event_catalog(...)` 让 net 的 typed event、option、catalog registration 保持在同一 authoring path 内。`tools/plugin_structure_audits/registration.py` 新增 `D8_RUNTIME_REGISTRATION_ROOTS` guard，`tools/audit_plugin_structure.py --json` 报告 `runtime_registration_builder_violation_count = 0`、`m3_t2_runtime_registration_builder_status = runtime-registration-builder-clean`。`review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder` 锁定源码/审计/docs/status。该切片只关闭 animation/physics/net 原始证据路径，不声明 importer private registry mutation 或 leaf module-only registration 全量清零。 |
| 2026-06-28 | M2 骨架 + SDK | D7 core workspace dependency inheritance guard | d7_core_workspace_dependency_inheritance_guard_static_passed_cargo_deferred | `zircon_plugins/Cargo.toml` 继续作为核心 path dependency 单源；所有 member crate 中 `zircon_runtime`、`zircon_editor`、`zircon_runtime_interface` 的本地 `path = ...` 声明已改为 `workspace = true`，`zircon_plugins/Cargo.lock` 已随离线解析同步。`tools/plugin_structure_audits/skeleton.py` 新增全局 core workspace dependency guard，`tools/audit_plugin_structure.py --json` 报告 `core_workspace_dependency_status = core-workspace-deps-clean`、`core_workspace_dependency_count = 117`、`core_workspace_dependency_violation_count = 0`，并由 `plugins_12_crate_skeleton_conformance` 锁定。验证：审计 JSON/markdown、py_compile、rustfmt、静态 path 扫描与 `cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --offline` 通过；catalog Cargo 测试仍因依赖图编译超出窗口而 deferred，不计通过。本切片只关闭核心引擎依赖继承，不声明插件间 path 依赖清零。 |
| 2026-06-28 | M1 统一 manifest schema | D-S7 static plugin manifest generation/parity review sync | ds7_static_plugin_manifest_generation_parity_review_synced_static_passed_cargo_deferred | 本轮复核并同步 D-S7 评审镜像：当前 `tools/audit_plugin_structure.py --json` 报告 `expected_manifest_count = 37`、`manifest_count = 37`、`generated_manifest_count = 36`、`hand_written_native_manifest_count = 1`、`manifest_schema_violations = 0`、`generated_manifest_header_violations = 0`、`m1_gate_status = classified-and-clear`。已有 `plugins_12_static_plugin_manifest_is_generated`、`plugins_12_manifest_schema_uniform_audit_report_is_clean` 与 `plugins_12_feature_enabled_runtime_descriptor_manifest_parity` 锁定 generated header、native 单源和 feature-enabled descriptor/static parity；本切片只同步 `engine-code-review-findings-2026-06.md` 与 Runtime 15 P0/DX priority guard，不改插件生产代码。Cargo gate deferred。 |
| 2026-06-28 | M3 注册收编 | D13 importer manifest parity guard | d13_importer_manifest_parity_guard_static_passed_cargo_deferred | `zircon_plugins/plugin_sdk/src/manifest/tests.rs::importer_runtime_manifest_builder_keeps_targets_platforms_modules_and_distribution_in_parity` 现在直接验证 `ImporterRuntimeManifestBuilder` 输出的 shared targets/platforms、runtime module、native dist module、NativeDynamic distribution、`NATIVE_ABI_VERSION_V3` 与 `NATIVE_DESCRIPTOR_SYMBOL_V3`。`zircon_plugins/plugin_sdk/src/lib.rs` 与 `prelude.rs` re-export 这些 v3 constants，避免 importer runtime crate 回流本地 ABI 常量；`tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs::review_d13_importer_manifest_parity_guard_lives_in_sdk_builder` 锁定 SDK helper/export/docs/status 链。Cargo gate deferred。 |
| 2026-06-28 | M3 注册收编 | D13 importer runtime manifest builder convergence | d13_importer_runtime_manifest_builder_convergence_static_passed_cargo_deferred | `zircon_plugin_sdk::manifest::ImporterRuntimeManifestBuilder` 现在统一拥有 importer runtime 默认 targets、platforms、runtime module manifest、native dist module manifest、NativeDynamic distribution、ABI v3 symbol/version、engine compat 与 asset-importer manifest 投影。12/12 importer runtime `plugin.rs` 保留 descriptor/importer/register owner，但 supported targets/platforms、runtime/dist module 和 package manifest distribution 样板全部走 SDK builder；本地 `PluginDistributionManifest`、`ExportPackagingStrategy`、ABI v3 常量与 `PluginModuleManifest::{runtime,native}` 构造样板已清空。新增 `review_d13_importer_runtime_manifests_use_sdk_builder` 锁定本状态，并同步 plugin SDK、asset importer skeleton、engine review findings、Runtime 15、结构规范与 session note。Cargo gate deferred。 |
| 2026-06-28 | M3 注册收编 | D13 importer runtime export macro convergence | d13_importer_runtime_export_macro_convergence_static_passed_cargo_deferred | 12/12 importer runtime `plugin.rs` owner 现在通过 `zircon_plugin_sdk::runtime_plugin_exports!` 生成 `runtime_plugin()`、`package_manifest()`、`runtime_selection()` 与 `plugin_registration()` helper；本轮迁移剩余 10 个 importer runtime crate（`asset_importers/{data,model,shader}` 与 `audio_importer/gltf_importer/obj_importer/opus_importer/shader_wgsl_importer/texture_importer/ui_document_importer`）并补 workspace `zircon_plugin_sdk` runtime feature 依赖，连同既有 `asset_importers/audio`、`asset_importers/texture` 形成同一 helper owner。新增 `review_d13_importer_runtime_exports_use_sdk_macro` 锁定无手写 `ProjectPluginSelection` / `RuntimePluginRegistrationReport` helper 回流；同步 plugin SDK、asset importer skeleton、engine review findings、结构规范与 session note。Cargo gate deferred；D13 targets/platforms/module/dist-module builder/parity 已由同日 manifest builder convergence 行闭合。 |
| 2026-06-23 | M5 存量硬切 | T1 final owner rollout（animation / animation_graph / hybrid_gi / material_editor / navigation / net / rendering / virtual_geometry） | plugins_12_final_skeleton_owner_rollout_clears_migration_debt | 最后 8 个 migration-debt roots 已迁入统一 owner 骨架：runtime crate 把 trait owner、descriptor/manifest/registration/helper 收进 `plugin.rs`，capability 单源保留在 `capability.rs`；editor crate 拆出 `capability.rs`、`extension_ids.rs`、`plugin.rs`、`tests.rs`；`net` 与 `rendering` 的 feature runtime/editor crate 同步拥有 feature-local `capability.rs` / `plugin.rs` owner。`lib.rs` 均退为薄 façade + 精选 re-export。验证：`python tools\audit_plugin_structure.py --json` 通过，报告 `missing_plugin_toml = 0`、`manifest_schema_violations = 0`、`capability_source_mismatches = 0`、`missing_capability_owner_files = 0`、`missing_runtime_capability_exports = 0`、`plugin_skeleton_gate.m2_gate_status = sample-clean-migration-debt-clear`、`skeleton_conformance.migration_debt_count = 0`、`plugin_skeleton_gate.migration_debt_count = 0`、`migration_debt_roots = []`；139 个 touched owner/façade 文件 `rustfmt --edition 2021 --check` 通过；`cargo check --manifest-path zircon_plugins\Cargo.toml --workspace --offline --locked` 在补齐 `hybrid_gi` placeholder mesh 的 `RenderLayerSet::from_legacy_mask(...)` 后通过（仅既有 warning 噪声）。该记录关闭 Plugins 12 M5/T1 skeleton migration debt。 |
| 2026-06-23 | M5 存量硬切 | T1 timeline_sequence editor skeleton owner rollout | plugins_12_timeline_sequence_skeleton_owner_rollout_reduces_migration_debt | `timeline_sequence/editor` 已迁入 owner 骨架：`capability.rs` 承接插件 id、editor capability 与 animation timeline event track capability 单源，`extension_ids.rs` 承接 view/drawer/template ids，`plugin.rs` 承接 `TimelineSequenceEditorPlugin`、descriptor、manifest、registration report、operation/menu 与 timeline authoring batch helper，`tests.rs` 承接 package/registration/validation/keyframe/event marker 断言；`lib.rs` 退为薄 façade，并保留 timeline sequence validation、keyframe move、track path sorting 与 event marker payload helper。验证：scoped rustfmt 通过；`python tools\audit_plugin_structure.py --json` 报告 `capability_source_mismatches = 0`、`skeleton_conformance.migration_debt_count = 8`、`plugin_skeleton_gate.migration_debt_count = 8`、`standalone_distribution_conformance.dist_capable_plugin_count = 1`。`cargo check --manifest-path zircon_plugins\timeline_sequence\editor\Cargo.toml --lib --offline --target-dir target\codex-plugin-validation --message-format short --color never` 被当前无关 `zircon_runtime` render 编译漂移挡在目标 timeline_sequence crate 前（`MeshPassCommandBuffers` / `CachedMeshDrawLookup` / `mesh_draw` / `mesh_pipeline_cache` imports），未计通过。该记录只关闭 timeline_sequence skeleton owner 子切片；剩余 8 个 migration-debt roots 与 Plugins 13 M5/T1 full dual-form rollout 仍未关闭。 |
| 2026-06-23 | M5 存量硬切 | T1 sound runtime/editor feature skeleton owner rollout | plugins_12_sound_skeleton_owner_rollout_reduces_migration_debt | `sound` 已完成 main runtime/editor 与两个 feature crate 的 owner 骨架迁移：main runtime `capability.rs` 拥有插件 id 与 runtime/feature capability 单源，`plugin.rs` 承接 `SoundRuntimePlugin`、descriptor、package manifest、feature manifest 与 SDK exports，`runtime_plugin` 子目录退回 descriptor/feature manifest owner；main editor 拆出 `capability.rs`、`extension_ids.rs`、`plugin.rs` 与 `tests.rs`，`authoring_bindings.rs` 改用 extension id/capability 常量；`ray_traced_convolution_reverb` 与 `timeline_animation_track` 的 runtime/editor feature crate 均拆出 `capability.rs`、`plugin.rs`、`tests.rs`，`lib.rs` 退为薄 façade。验证：scoped rustfmt 通过；`python tools\audit_plugin_structure.py --json` 报告 `capability_source_mismatches = 0`、`skeleton_conformance.migration_debt_count = 9`、`plugin_skeleton_gate.migration_debt_count = 9`、`standalone_distribution_conformance.dist_capable_plugin_count = 1`。`cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --lib --offline --target-dir target\codex-plugin-validation --message-format short --color never` 被当前无关 `zircon_runtime` render 编译漂移挡在目标 sound crate 前（`MeshPassCommandBuffers` / `CachedMeshDrawLookup` / `mesh_draw` / `mesh_pipeline_cache` imports），未计通过。该记录只关闭 sound skeleton owner 子切片；剩余 9 个 migration-debt roots 与 Plugins 13 M5/T1 full dual-form rollout 仍未关闭。 |
| 2026-06-23 | M5 存量硬切 | T1 editor build/export desktop skeleton owner rollout | plugins_12_editor_build_export_desktop_skeleton_owner_rollout_reduces_migration_debt | `editor_build_export_desktop` editor crate 已迁入 owner 骨架：`capability.rs` 拥有插件 id 与三个 editor capability，`extension_ids.rs` 拥有 view/drawer/template/report/operation ids 与私有模板资产路径，`plugin.rs` 拥有 `EditorBuildExportDesktopPlugin`、descriptor、package manifest、registration report、导出 operation/menu、NativeDynamic report templates 与 export profile authoring 注册，`tests.rs` 承接 package/registration/template asset 断言；`lib.rs` 从 639 行退为 74 行薄 façade，并保留 export wizard 精选 re-export。验证：scoped rustfmt 通过；`python tools\audit_plugin_structure.py --json` 报告 `capability_source_mismatches = 0`、`skeleton_conformance.migration_debt_count = 10`、`plugin_skeleton_gate.migration_debt_count = 10`、`standalone_distribution_conformance.dist_capable_plugin_count = 1`；`cargo check --manifest-path zircon_plugins\editor_build_export_desktop\editor\Cargo.toml --all-targets --offline --target-dir target\codex-plugin-validation --message-format short --color never` 通过（仅既有 runtime/editor warning 噪声）。该记录只关闭 M5/T1 editor_build_export_desktop skeleton owner 子切片；剩余 10 个 migration-debt roots 与 Plugins 13 M5/T1 full dual-form rollout 仍未关闭。 |
| 2026-06-23 | M5 存量硬切 | T1 particles / physics / texture skeleton owner rollout | plugins_12_particles_physics_texture_skeleton_owner_rollout_reduces_migration_debt | `particles`、`physics`、`texture` 已完成 runtime/editor 骨架迁移：runtime crate 的 `RuntimePlugin` owner、descriptor、manifest/registration helper 迁入 `runtime/src/plugin.rs`，runtime crate root 只保留模块声明与精选 re-export；`physics` 与 `texture` 测试迁入 `runtime/src/tests.rs`，`texture` 进一步拆出 `manager.rs` 与 `module.rs` owner；三个 editor crate 均拆出 `capability.rs`、`extension_ids.rs`、`plugin.rs` 与 `tests.rs`（`particles` 保留 `authoring.rs` 领域 helper）。`particles/runtime/src/simulation/cpu.rs` 同步补齐 `RenderParticleSpriteSnapshot.render_layer_mask` 构造字段，解除本轮 runtime check 的既有结构漂移。验证：scoped rustfmt 通过；`python tools/audit_plugin_structure.py --json` 报告 `capability_source_mismatches = 0`、`skeleton_conformance.migration_debt_count = 11`、`plugin_skeleton_gate.migration_debt_count = 11`、`standalone_distribution_conformance.dist_capable_plugin_count = 1`；三包 runtime `cargo check --offline` 与三包 editor `cargo check --offline` 均通过（仅既有 warning 噪声，physics/texture editor 使用隔离 target dir 避开外部并发 Cargo 产物干扰）。该记录只关闭 M5/T1 particles/physics/texture skeleton owner 子切片；剩余 11 个 migration-debt roots 与 Plugins 13 M5/T1 full dual-form rollout 仍未关闭。 |
| 2026-06-23 | M5 存量硬切 | T1 authoring runtime/editor skeleton owner rollout | plugins_12_authoring_runtime_editor_skeleton_owner_rollout_reduces_migration_debt | `prefab_tools`、`terrain`、`tilemap_2d` 已完成同根 runtime+editor 骨架迁移：runtime crate 的 `RuntimePlugin` owner、descriptor、component/importer manifest override 与 `runtime_plugin_exports!` 移入 `runtime/src/plugin.rs`，测试移入 `runtime/src/tests.rs`；editor crate 拆为 `capability.rs`、`extension_ids.rs`、`plugin.rs`、`authoring.rs` 与 `tests.rs`，`lib.rs` 只保留模块声明和精选 re-export。验证：scoped rustfmt 通过；`python tools/audit_plugin_structure.py --json` 报告 `capability_source_mismatches = 0`、`skeleton_conformance.migration_debt_count = 14`、`plugin_skeleton_gate.migration_debt_count = 14`、`standalone_distribution_conformance.dist_capable_plugin_count = 1`；三包 runtime `cargo check --offline` 通过（仅既有 `zircon_runtime` warnings）。三包 editor `cargo check --offline` 两次超时且无最终输出，不计通过。该记录只关闭 M5/T1 authoring runtime/editor skeleton owner 子切片；剩余 14 个 migration-debt roots 与 Plugins 13 M5/T1 full dual-form rollout 仍未关闭。 |
| 2026-06-23 | M5 存量硬切 | T1 editor-only skeleton owner rollout | plugins_12_editor_only_skeleton_owner_rollout_reduces_migration_debt | `native_window_hosting`、`runtime_diagnostics`、`ui_asset_authoring` 的 editor crate 已迁入 `src/capability.rs`（插件 id / editor capability 单源）、`src/extension_ids.rs`（view/drawer/template id）、`src/plugin.rs`（`EditorPlugin` owner、descriptor、package_manifest、registration report）与 `src/tests.rs`，`lib.rs` 退为模块声明和精选 re-export 的薄 façade。`tools/plugin_structure_audits/capability.py` 同步接受 M5 后 `runtime_capabilities()` 由 `plugin.rs` owner 实现并从 `lib.rs` re-export，避免结构迁移误触 M4 四源 guard。验证：scoped rustfmt 通过；py_compile 通过；`python tools/audit_plugin_structure.py --json` 报告 `capability_source_mismatches = 0`、`skeleton_conformance.migration_debt_count = 17`、`plugin_skeleton_gate.migration_debt_count = 17`、`standalone_distribution_conformance.dist_capable_plugin_count = 1`。三包 editor `cargo check` 被当前无关 `zircon_editor` retained-host compile drift（`HostWindowPresentationData` import 与 E0282）挡在目标包编译前，未计通过。该记录只关闭 M5/T1 editor-only skeleton owner 子切片；剩余 17 个 migration-debt roots 与 Plugins 13 M5/T1 full dual-form rollout 仍未关闭。 |
| 2026-06-23 | M5 存量硬切 | T1 runtime-only skeleton owner rollout | plugins_12_runtime_only_skeleton_owner_rollout_reduces_migration_debt | `ai`、`solari`、`zr_vm_language` 的 runtime crate 已把 `RuntimePlugin` 入口类型、descriptor、`register` 实现、`runtime_plugin_exports!` 与标准 helpers 从 `lib.rs` 迁到 `runtime/src/plugin.rs`，crate root 只保留 `mod plugin;` 与精选 re-export。`asset_importers/audio` 与 `asset_importers/texture` 从 legacy declaration-only `package_manifest()` 收口为 trait-backed runtime plugin：新增 `capability.rs` / `plugin.rs`，补 `zircon_plugin_sdk` workspace dependency，`package_manifest()` 仍通过 `RuntimePlugin::package_manifest()` 投影并附加原有 asset importer descriptor 清单，`register()` 只注册 runtime module。验证：scoped rustfmt 通过；`python tools/audit_plugin_structure.py --json` 报告 `skeleton_conformance.migration_debt_count = 20`、`plugin_skeleton_gate.migration_debt_count = 20`，较 importer 子切片后再减少 5 个 runtime-only roots；上述 5 个 package 的 offline cargo check 通过并刷新 lock，随后 locked cargo check 通过（仅既有 `zircon_runtime` warning 噪声）。该记录只关闭 M5/T1 runtime-only skeleton owner 子切片；剩余 20 个 migration-debt roots、editor/sound 更广 rollout 与 Plugins 13 M5/T1 full dual-form rollout 仍未关闭。 |
| 2026-06-23 | M5 存量硬切 | T1 importer capability owner rollout | plugins_12_importer_capability_owner_rollout_reduces_migration_debt | `audio_importer`、`gltf_importer`、`obj_importer`、`opus_importer`、`shader_wgsl_importer`、`texture_importer`、`ui_document_importer` 与 `asset_importers/{data,model,shader}` 的 runtime crate 已新增 `runtime/src/capability.rs`，crate root 退为 `mod capability;` + 常量 re-export 的薄 façade，capability 常量不再堆在 `lib.rs`。验证：scoped rustfmt 通过；`python tools/audit_plugin_structure.py --json` 报告 `skeleton_conformance.migration_debt_count = 25`、`plugin_skeleton_gate.migration_debt_count = 25`，较上一状态 35 减少 10 个 importer runtime root；10 个 importer runtime package 逐包 `cargo check --manifest-path zircon_plugins\Cargo.toml -p <pkg> --locked --jobs 1` 均 exit=0（仅既有 warning 噪声）。该记录只关闭 M5/T1 importer capability-owner 子切片；剩余 25 个 migration-debt roots、`asset_importers/audio` 与 `asset_importers/texture` 的 legacy declaration-only 根、以及 Plugins 13 M5/T1 full dual-form rollout 仍未关闭。 |
| 2026-06-23 | M5 存量硬切 | T2 D6 `RuntimePluginId` 封闭枚举 → string-newtype | plugins_12_runtime_plugin_id_string_newtype_accepts_external_ids | `RuntimePluginId` 已从封闭 enum 改为 `Copy` 的开放 string-newtype，内建插件 id 保留 `RuntimePluginId::Ui`、`::Physics` 等关联常量以兼容现有调用面；`parse_key(...)` / `new(...)` 现在接受合法第三方 key 并稳定序列化为字符串，第三方插件声明 id 不再需要新增 core variant。`plugin_modules/loader.rs` 对未知合法 runtime plugin id 走 externalized warning fallback；runtime module structure guard 锁定 `pub struct RuntimePluginId` 并拒绝回退 enum；plugin workspace shape guard 改为用 runtime catalog membership 区分 editor-only 插件，而不是用封闭 enum 解析表。验证：scoped rustfmt 通过；`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked` 通过；`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sdk --locked` 通过（均仅既有 warning 噪声）。focused runtime test `runtime_plugin_id_accepts_external_keys_without_core_variant` 已加入，但当前 runtime lib-test lane 在执行前被无关 test-tree compile drift 阻塞，未计通过。该记录关闭 D6 / M5/T2；M5/T1 存量骨架 touch-it-conform-it 仍按能力波次推进。 |
| 2026-06-23 | M4 capability 单源 | T2 SDK feature-bundle builder + editor `mirrors_runtime(...)` 对称 guard | plugins_12_sdk_builder_mirror_guard_passed | `zircon_plugin_sdk` 新增 `PluginFeatureBundleBuilder`，一次声明 optional feature 的依赖、capability、runtime/editor module 与默认 packaging，并从 `manifest/mod.rs`、crate root 和 prelude 暴露；`EditorPluginDeclaration` 新增 `mirrors_runtime(...)` / `mirrors_runtime_manifest(...)` / `mirrored_runtime_package_id()`，`authoring_plugin!` 支持 `mirrors_runtime: ...`，可把 editor declaration 投影到 runtime package manifest 并保留 editor capability/asset/content roots。`tools/plugin_structure_audits/capability.py` 新增 SDK builder/mirror 静态 guard，`tools/audit_plugin_structure.py --json` 报告 `sdk_builder_mirror_violations = 0`、`m4_t2_builder_mirror_gate_status = sdk-builder-mirror-clean`；`zircon_first_party_runtime_catalog::tests::plugins_12_capability_single_source_conformance` 锁定这些字段。验证：py_compile、plugin audit JSON、scoped rustfmt、SDK default check、`feature_bundle_builder_projects_capability_to_feature_and_modules` focused test 1/1、SDK editor feature check 均通过（仅既有 runtime/editor warning）。`editor_declaration_mirrors_runtime_manifest_and_keeps_editor_capabilities` focused cargo test 900s 超时且不计通过，残留该 target-dir 进程已清理；为恢复 editor feature check，`zircon_runtime::ui::dispatch` façade 补导出现有 route policy helper，行为 owner 仍在 `input_manager::routing`。sound/importer/editor 的更广 capability 迁移仍 pending。 |
| 2026-06-23 | M4 capability 单源 | T1 15 个 trait-backed first-party runtime 根 `capability.rs` + 四源 guard | plugins_12_runtime_capability_single_source_guard_passed | `ai`、`animation`、`hybrid_gi`、`navigation`、`net`、`particles`、`physics`、`prefab_tools`、`rendering`、`solari`、`terrain`、`texture`、`tilemap_2d`、`virtual_geometry`、`zr_vm_language` 的 runtime package capability 已迁入各自 `runtime/src/capability.rs`，crate root 通过 `RUNTIME_CAPABILITIES` 投影 descriptor/status/importer requirement/runtime_capabilities。新增 `tools/plugin_structure_audits/capability.py` 并接入 `tools/audit_plugin_structure.py --json`，报告 `audited_runtime_root_count = 15`、`capability_source_mismatches = 0`、`m4_runtime_capability_gate_status = runtime-capability-single-source-clean`；`zircon_first_party_runtime_catalog::tests::plugins_12_capability_single_source_conformance` 消费该报告。验证：py_compile、plugin audit JSON、scoped rustfmt、16-package focused `cargo check --locked` 与 catalog focused test 1/1 通过（仅既有 warning）。为让 catalog test 构建通过，顺带修复 `zircon_runtime/src/ui/surface/input/editable_text.rs` 中 test-build-only `UiImeInputEventKind::DeleteSurrounding` match 穷尽性缺口；不实现 delete-surrounding 行为，保持现有提前返回语义。该切片只声明 M4/T1 runtime 根完成；M4/T2 SDK guard 见上一行，sound/importer/editor 更广迁移仍 pending。 |
| 2026-06-23 | M3 注册收编 | T2 D12 `runtime_plugin_exports!` trait-backed runtime 全量 rollout | plugins_12_runtime_export_macro_rollout_check_passed | `zircon_plugin_ai_runtime`、`zircon_plugin_hybrid_gi_runtime`、`zircon_plugin_navigation_runtime`、`zircon_plugin_particles_runtime`、`zircon_plugin_prefab_tools_runtime`、`zircon_plugin_rendering_runtime`、`zircon_plugin_solari_runtime`、`zircon_plugin_terrain_runtime`、`zircon_plugin_texture_runtime`、`zircon_plugin_tilemap_2d_runtime`、`zircon_plugin_virtual_geometry_runtime` 与 `zircon_plugin_zr_vm_language_runtime` 已加入前一批 animation/physics/net，全部删除 crate root 手写 `runtime_plugin()` / `package_manifest()` / `runtime_selection()` / `plugin_registration()` helper 块并改用 `zircon_plugin_sdk::runtime_plugin_exports!(...)`。各 crate 增加 workspace `zircon_plugin_sdk` runtime feature 依赖，`prefab_tools`、`terrain`、`tilemap_2d` 的自定义 `runtime_package_manifest()` 仍通过 `RuntimePlugin::package_manifest()` 被宏生成 helper 调用，行为未降级。全量扫描仅剩 `asset_importers/audio` 与 `asset_importers/texture` 的 legacy `package_manifest()`，它们不是 trait-backed D12 helper 块。验证：scoped rustfmt 通过；16-package offline `cargo check` 通过并刷新 `zircon_plugins/Cargo.lock`；同包 locked `cargo check` 通过（仅既有 `zircon_runtime` 与大插件 warning 噪声）；`cargo metadata --locked` 通过；SDK `runtime_plugin_exports` focused test 1/1 通过。M4 capability 单源与 M5 `RuntimePluginId` open/custom id 已由后续记录关闭。 |
| 2026-06-23 | M3 注册收编 | T2 D12 `runtime_plugin_exports!` + animation/physics/net 代表迁移 | plugins_12_runtime_export_macro_representative_check_passed | `zircon_plugin_sdk` 新增 `runtime_plugin_exports!` 宏，生成 `runtime_plugin()`、`package_manifest()`、`runtime_selection()` 与 `plugin_registration()` 四个标准 runtime crate helper，并通过 trait 方法投影，保留 `NetRuntimePlugin::package_manifest()` 这类自定义 manifest override。`zircon_plugin_animation_runtime`、`zircon_plugin_physics_runtime`、`zircon_plugin_net_runtime` 已删除 crate root 内手写转发块，改为一行宏调用；physics/net runtime crate 新增 workspace `zircon_plugin_sdk` 依赖，`zircon_plugins/Cargo.lock` 已经离线刷新并用 locked check 复验。验证：scoped rustfmt check 通过；同包离线 `cargo check` 通过并刷新 lock；同包 locked `cargo check` 通过（仅既有 `zircon_runtime`、net、physics warnings）；SDK `runtime_plugin_exports` focused test 1/1 通过。该切片关闭 D12 的 SDK helper 与三条评审证据路径代表迁移；剩余 trait-backed runtime helper rollout 已由上一行全量记录关闭，M4 capability 单源与 M5 `RuntimePluginId` open/custom id 已由后续记录关闭。 |
| 2026-06-23 | M3 注册收编 | T1 split importers 自由函数 → `RuntimePlugin` 入口 | plugins_12_split_importer_single_registration_entry_check_passed | `zircon_plugins/{gltf_importer,obj_importer,texture_importer,audio_importer,opus_importer,shader_wgsl_importer,ui_document_importer}/runtime/src/plugin.rs` 现为各 split importer 的 trait-backed 注册 owner，`lib.rs` 退为薄 façade 并只 re-export descriptor/manifest/selection/report helpers；旧 `texture_importer/runtime/src/registration.rs` 已删除，非 test 源码扫描不再出现 importer `pub fn register(...)`。各 split importer 的 `plugin_registration()` 统一由 `RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())` 生成，`runtime_selection()` 由 runtime descriptor projection 派生。`tools/plugin_structure_audits/registration.py` 已扩展 split importer 口径，`tools/audit_plugin_structure.py --json` 报告 `registration_conformance.m3_split_importer_gate_status = split-importer-single-entry-clean`、`split_importer_free_function_registration_sites = 0`、`split_importer_registration_owner_files = 0`，aggregate `m3_importer_gate_status = importer-single-entry-clean`。当时 M5 string-newtype 尚未落地，本切片仅给 `RuntimePluginId` 封闭 enum 补 `OpusImporter` 临时接线并在 loader 中 externalized 到 `zircon_plugins/opus_importer`；该过渡项已由后续 M5/T2 string-newtype 记录替换。验证：scoped rustfmt check、py_compile、plugin audit JSON、`cargo metadata --manifest-path zircon_plugins/Cargo.toml --format-version 1 --no-deps --locked` 与 split importer focused `cargo check --locked` 均通过（仅既有 `zircon_runtime` warnings）。D12 转发自由函数 helper/blanket、M4 capability 四源一致性与 M5 存量迁移仍 pending。 |
| 2026-06-23 | M3 注册收编 | T1 `asset_importers/*` family 自由函数 → `RuntimePlugin` 入口 | plugins_12_asset_importer_family_single_registration_entry_check_passed | `zircon_plugins/asset_importers/{data,model,shader}/runtime/src/plugin.rs` 现为注册 owner：各自新增 `*AssetImporterRuntimePlugin`，实现 `RuntimePlugin::descriptor` / `package_manifest` / `register`，并把 module registration 与 importer handler registration 收进 trait 入口；对应 `lib.rs` 退为薄 façade，不再暴露 `pub fn register(...)`。`asset_importers/model/runtime/src/registration.rs` 已删除，model/data/shader 的 `plugin_registration()` 改由 `RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())` 生成，`runtime_selection()` 改由 descriptor projection 派生。`tools/plugin_structure_audits/registration.py` 新增 M3/T1 审计字段，`tools/audit_plugin_structure.py --json` 报告 `registration_conformance.m3_t1_gate_status = family-single-entry-clean`、`asset_importer_family_free_function_registration_sites = 0`、`asset_importer_family_registration_owner_files = 0`，同时记录 data/model/shader 三个 trait entry file。当时运行时插件 id 仍采用 core 封闭 enum，本切片为 data/model/shader 增加 `AssetImporterData` / `AssetImporterModel` / `AssetImporterShader` 作为临时接线，并在 loader 中继续标记 implementation externalized；该过渡项已由后续 M5/T2 string-newtype 记录替换。验证：scoped rustfmt check 通过；`python -m py_compile tools/audit_plugin_structure.py plugin_structure_audits\__init__.py plugin_structure_audits\manifest_schema.py plugin_structure_audits\skeleton.py plugin_structure_audits\registration.py` 通过；`python tools/audit_plugin_structure.py --json` 保持 `missing_plugin_toml = 0`、`manifest_schema_violations = 0`、`sample_conformance_status = sample-clean`、`migration_debt_count = 35`，并新增 registration gate clean；`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_data_runtime -p zircon_plugin_asset_importer_model_runtime -p zircon_plugin_asset_importer_shader_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-importer-m3-0622 --message-format short --color never` 通过（仅既有 runtime warnings）。`cargo test ... zircon_plugin_asset_importer_model_runtime ... registration_contributes_stl_ply_and_dxf_importers` 首轮 904s 超时无结果；`--lib` 复跑被既有 `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs` 的 `MaterialCaptureSeed` / `MaterialRuntime::capture_seed` lib-test drift 阻断，不计通过。split importers（`gltf_importer` / `obj_importer` / `texture_importer` / `audio_importer` / `opus_importer` / `shader_wgsl_importer` / `ui_document_importer`）已由后续 M3/T1 split 记录关闭公开自由函数注册；D12 转发自由函数 helper/blanket、M4 capability 单源与 M5 存量迁移仍 pending。 |
| 2026-06-22 | M3 注册收编 | T2 `RuntimePluginRegistrationBuilder` + animation runtime 代表迁移 | plugins_12_registration_builder_animation_passed | `zircon_plugin_sdk::registration` 新增 runtime 注册 builder：`RuntimePluginRegistrationBuilder::module(...)` 统一 intern plugin module owner + `register_module(...)` 顺序，`RuntimePluginModuleRegistration::runtime_scene_system(...)` 继续封装 owner token、system id、stage、set/order/before/after constraint，并由 `.register()` 写入 `RuntimeExtensionRegistry`，插件作者不再直接传递 `PluginModuleId`。`zircon_plugin_animation_runtime` 作为代表插件改用该 builder：`AnimationRuntimePlugin::register` 不再手写 `intern_plugin_module -> register_module -> register_runtime_scene_system`，`runtime_system.rs` 只接收 SDK module registration handle 并声明 animation system set/order constraint。为接入 SDK，animation runtime 改用 workspace dependency `zircon_plugin_sdk`，`zircon_plugins/Cargo.lock` 已由 offline check 刷新后再以 `--locked` 验证。验证：scoped rustfmt 通过；`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sdk -p zircon_plugin_animation_runtime --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-registration-m3-0622 --message-format short --color never` 通过并刷新 lock；同包 `--locked` check 通过（仅既有 runtime warnings）；`cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-registration-m3-0622 --message-format short --color never runtime_registration_builder -- --test-threads=1 --nocapture` 通过 1/1；`cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_animation_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-registration-m3-0622 --message-format short --color never animation_registration_contributes_runtime_module -- --test-threads=1 --nocapture` 通过 1/1。该切片关闭 D8 的 SDK 封装和 animation 代表迁移；`asset_importers/*` family 与 split importer 公开注册自由函数已由后续 M3/T1 记录关闭，D12 转发自由函数宏/blanket、M4 capability 单源与 M5 存量迁移仍 pending。 |
| 2026-06-22 | M2 骨架 + SDK | T5 `plugin_sdk::test::TestRuntime::builder()` fixture | plugins_12_test_runtime_fixture_passed | `zircon_plugin_sdk` 新增 `plugin_sdk::test` runtime fixture owner，并通过 `lib.rs` 与 `prelude` 暴露 `TestRuntime`、`TestRuntimeBuilder`、`TestRuntimeBaseModule` 与 `TestRuntimeError`。`TestRuntimeBuilder` 默认注册并激活 foundation/asset/scene 基础模块，收集 runtime plugin registration reports，合并 `RuntimePluginCatalog::runtime_extensions()`，注册插件贡献的 runtime modules，安装 scene/world runtime extensions，并可按需关闭基础模块、world extensions、scene hooks 或插件模块激活。`TestRuntime` 提供 `handle()`、`runtime()`、`extension_report()`、`activated_modules()`、typed `resolve_manager(...)`、`create_default_level()`、`advance_time_by_seconds(...)` 与 `tick_level_seconds(...)`，把 D11 长跨插件测试里的 runtime/scene/fixed-step 样板收进 SDK fixture。自测使用 fake runtime plugin 验证基础模块和插件 module 会注册激活、manager 可解析、默认 level 能安装 runtime world extensions 并 tick。验证：`rustfmt --edition 2021 zircon_plugins\plugin_sdk\src\lib.rs zircon_plugins\plugin_sdk\src\prelude.rs zircon_plugins\plugin_sdk\src\test.rs` 通过；`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-test-runtime-m2-0622 --message-format short --color never` 通过（仅既有 `zircon_runtime` warnings）；`cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-test-runtime-m2-0622 --message-format short --color never test_runtime_builder -- --test-threads=1 --nocapture` 复跑通过 2/2；首次同命令 600s 超时但后续编译进程自然结束，未计通过。M3 注册收编、M4 capability 四源一致性与 M5 存量迁移仍 pending。 |
| 2026-06-22 | M2 骨架 + SDK | T4 `authoring_plugin!` editor 宏 + workspace dependency inheritance | plugins_12_editor_authoring_macro_workspace_deps_passed | `zircon_plugin_sdk` 新增 `editor` feature 与 `plugin_sdk::editor` owner：`EditorPluginDeclaration` 统一 editor descriptor/base package manifest/capability/asset/content root 投影，`authoring_plugin!` 宏生成 editor plugin struct、`Default`/`new`、descriptor、package_manifest、editor_capabilities、registration_report 与 `EditorPlugin::register_editor_extensions` 转发样板。`zircon_plugin_sdk_examples_editor` 主插件改用 `authoring_plugin!`，`plugin.rs` 不再手写主 plugin descriptor/trait impl/base manifest builder，只保留 fixture-specific extension 注册转发和两个子 fixture plugin。`zircon_plugins/Cargo.toml` 新增 `[workspace.dependencies]`，`plugin_sdk`、样例 editor crate 与 native fixture 改用 workspace dependency inheritance；`tools/plugin_structure_audits/skeleton.py` 新增 sample workspace-dependency guard，当前 `sample_workspace_dependency_status = sample-workspace-deps-clean`。验证：scoped rustfmt 通过；`python tools/audit_plugin_structure.py --json` 保持 `m1_gate_status = classified-and-clear`、`sample_conformance_status = sample-clean`、`sample_workspace_dependency_violation_count = 0`、`migration_debt_count = 35`；py_compile 通过；`cargo metadata --manifest-path zircon_plugins/Cargo.toml --format-version 1 --no-deps --locked` 通过；`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --features editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-editor-sdk-m2-0622 --message-format short --color never` 通过（仅既有 runtime/editor warnings）；`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-editor-sdk-m2-0622 --message-format short --color never` 通过；默认 SDK check 与 native fixture check 在 workspace dependency 调整后均通过。`cargo test -p zircon_plugin_sdk --features editor ... authoring_plugin_macro_generates_descriptor_manifest_and_registration` 与 `cargo test -p zircon_plugin_sdk_examples_editor` 均 300s 超时且未生成测试二进制，不计通过。M3 注册收编、M4 capability 四源一致性仍 pending。 |
| 2026-06-22 | M2 骨架 + SDK | T3 `plugin_sdk` native ABI helper + native fixture 收编 | plugins_12_native_sdk_helper_passed | `zircon_plugins/plugin_sdk` 新增 feature 分界：默认 `runtime` 继续暴露 manifest/runtime declaration builder，`native` 仅依赖 `zircon_runtime_interface`，不再拉起完整 `zircon_runtime`，避免 native cdylib 作者被 runtime crate 迁移噪声阻塞。新增 `plugin_sdk::native`：导出 ABI v3 author-side `repr(C)` 类型、native status constants、`NativePluginStatic<T>` 静态 Sync wrapper、`NativePluginEntryPointV3` required/denied capability gate、SDK-owned byte buffer/free callback、`bytes_from_slice`、`callback_status`、`catch_native_callback_panic`、`export_native_plugin_descriptor_v3!`、`export_native_plugin_entry_v3!` 与 stateless `native_command_plugin_v3!` helper macro。`zircon_plugin_native_dynamic_fixture_native` 现以 `default-features = false, features = ["native"]` 依赖 SDK，删除本地 ABI struct/type alias、owner_token/free、panic hook、capability-list parsing 与 `#[no_mangle]` export 函数，只保留 fixture 命令、state、asset import 与 host diagnostics 行为。`tools/plugin_structure_audits/skeleton.py` 的 native fixture 豁免理由更新为 native-only ABI fixture uses SDK native helper, and the M2/T3 placeholder is closed。验证：scoped rustfmt check 通过；`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --no-default-features --features native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-native-sdk-m2-0622 --message-format short --color never` 通过；`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --no-default-features --features native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-native-sdk-m2-0622 --message-format short --color never -- --test-threads=1` 通过 4/4；`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-native-fixture-sdk-m2-0622 --message-format short --color never` 通过；`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-sdk-default-m2-0622 --message-format short --color never` 通过（仅既有 `zircon_runtime` warnings）；`python tools/audit_plugin_structure.py --json` 保持 `m1_gate_status = classified-and-clear`、`manifest_schema_violations = 0`、`skeleton_conformance.sample_conformance_status = sample-clean`、`migration_debt_count = 35`。M3 注册收编、M4 capability 四源一致性仍 pending。 |
| 2026-06-22 | M2 骨架 + SDK | T2 骨架符合度 guard + 首个样例插件 | plugins_12_crate_skeleton_sample_conformance_passed | `zircon_plugins/plugin_sdk_examples/editor` 已改成首个 Plugins 12 骨架样例：`lib.rs` 只保留 module 声明与精选 re-export，`capability.rs` 单源声明 editor capabilities，`plugin.rs` 拥有 editor plugin descriptor、SDK manifest builder 投影与 registration report，`extensions.rs` 拥有 window/importer/inspector/UI template/component drawer 注册，`extension_ids.rs` 拥有扩展 id 常量，`tests.rs` 拥有样例行为断言；`editor/Cargo.toml` 接入 `zircon_plugin_sdk`，`Cargo.lock` 记录该 path 依赖。新增 `tools/plugin_structure_audits/skeleton.py` 并接入 `tools/audit_plugin_structure.py --json`，输出 `skeleton_conformance.sample_conformance_status = sample-clean`、`sample_expected_count = 1`、`sample_violation_count = 0`、`migration_debt_count = 35`、`migration_debt_details_truncated = true`，`plugin_skeleton_gate.m2_gate_status = sample-clean-migration-debt-present`。`zircon_first_party_runtime_catalog::tests::plugins_12_crate_skeleton_conformance` 消费该报告，锁定首个样例不回退。验证：`python tools/audit_plugin_structure.py --json` 关键字段通过；`python -m py_compile tools/audit_plugin_structure.py tools/plugin_structure_audits/__init__.py tools/plugin_structure_audits/manifest_schema.py tools/plugin_structure_audits/skeleton.py` 通过；scoped rustfmt check 通过；`cargo metadata --manifest-path zircon_plugins/Cargo.toml --format-version 1 --no-deps --locked` 通过；`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor -p zircon_first_party_runtime_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never` 通过（仅既有 warning）。为恢复该 check，`RuntimePluginDescriptor` 私有字段迁移的 3 个旧 consumer 已改用 getter：`core/runtime/diagnostics/devtools.rs`、`plugin/package_manifest/builtin_catalog.rs`、`plugin/runtime_profile/availability.rs`。`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never -- --test-threads=1` 1200s 超时，`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12_crate_skeleton_conformance --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never -- --test-threads=1 --nocapture` 900s 超时，均未产出测试结果且不计通过，残留进程已确认清理。M2/T4 editor macro/workspace dependency inheritance 与 M2/T5 test runtime fixture 已由后续记录关闭；M4 capability 四源一致性仍 pending。 |
| 2026-06-22 | M2 骨架 + SDK | T1 `plugin_sdk` builder crate baseline | plugins_12_plugin_sdk_builder_baseline_passed | 新增 workspace crate `zircon_plugins/plugin_sdk`，根 façade 仅导出 `manifest`、`runtime` 与 `prelude`；`PluginManifestBuilder` 固化 `sdk_api_version = "0.1.0"`、默认平台 `windows/linux/macos` 与默认 packaging `SourceTemplate + LibraryEmbed`，并产出 runtime-owned `PluginPackageManifest`；`PluginModuleBuilder` 统一 runtime/editor/native/vm module 声明，editor module 默认 `EditorHost`；`RuntimePluginDeclaration` 从同一声明投影 `RuntimePluginDescriptor` 与 `PluginPackageManifest`。`plugin_sdk` 作为 support crate 被 `plugin_structure_audits::manifest_schema` 跳过，不计入 37 个插件 manifest root。验证：scoped rustfmt check 通过；`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-sdk-m2-0622 --message-format short --color never` 通过；`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-sdk-m2-0622 --message-format short --color never -- --test-threads=1` 通过 3/3；`python tools/audit_plugin_structure.py --json` 保持 `m1_gate_status = classified-and-clear`、`missing_plugin_toml = 0`、`manifest_schema_violations = 0`。M2/T4 editor 宏/workspace dependency 继承与 M2/T5 test runtime fixture 已由后续记录关闭；M4 capability 四源一致性仍 pending。 |
| 2026-06-22 | M1 统一 manifest schema | feature-enabled runtime descriptor/static manifest parity | plugins_12_feature_descriptor_parity_guard_passed | `zircon_first_party_runtime_catalog::tests::plugins_12_feature_enabled_runtime_descriptor_manifest_parity` 作为 feature-gated guard 覆盖 `base-runtime-plugins`、`advanced-render-runtime-plugins`、`navigation-runtime-plugin` 与 `zr-vm-language-runtime-plugin` 链接的一方 runtime providers，并把 Rust descriptor 的 `package_manifest()` 投影同静态 generated `plugin.toml` 对齐。已修复 glTF/rendering/texture maturity 漂移、Hybrid GI/Virtual Geometry `rendering` category 漂移、Navigation Recast capability 漂移，并补 Hybrid GI `RenderMeshSnapshot` placeholder 字段与共享 `ViewportRenderFrame.extract = Arc<RenderFrameExtract>` 构造编译缺口。验证：feature parity exact test 1/1 通过；默认 `plugins_12` catalog guards 3/3 通过；`python tools/audit_plugin_structure.py --json` 输出 `m1_gate_status = classified-and-clear`、`missing_plugin_toml = 0`、`manifest_schema_violations = 0`、`expected_manifest_count = 37`；Navigation runtime registration test 1/1 通过；scoped rustfmt check 通过（仅既有 `zircon_runtime` warnings）。M2/T1 `plugin_sdk` baseline 已由后续记录关闭；M2/T2+ 骨架迁移与 M4 capability 四源一致性仍 pending。 |
| 2026-06-22 | M1 统一 manifest schema | T5 静态 `plugin.toml` 生成标记 + native manifest 单源嵌入（D-S7/D3） | plugins_12_static_manifest_generated_marker_static_passed_cargo_timeout | 30 个非 native 根 `plugin.toml` 已加 `# @generated from Rust descriptor package_manifest(); do not edit by hand.`；`native_dynamic_fixture/native/src/lib.rs` 删除内嵌 TOML 副本并改 `concat!(include_str!("../../plugin.toml"), "\0")`；`zircon_first_party_runtime_catalog::tests::plugins_12_static_plugin_manifest_is_generated` 新增生成头、runtime descriptor 子集 parity 与 native 单源 guard；`rustfmt --edition 2021 --config skip_children=true --check` 通过，静态生成头扫描 30/30 通过，scoped `git diff --check` 仅 LF/CRLF 提示；`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12_static_plugin_manifest_is_generated --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins12-ds7-0622 --message-format short --color never -- --test-threads=1` 1200s 超时且不计通过，残留进程已按目标目录清理。 |
| 2026-06-22 | M1 统一 manifest schema | T3/T5 缺失 importer manifest 覆盖 + 生成 guard 复跑 | plugins_12_missing_importer_manifests_guard_passed | 新增 `asset_importers/{audio,data,model,shader,texture}/plugin.toml` 与 `opus_importer/plugin.toml`，全部使用 `@generated from Rust descriptor package_manifest()` 头和统一必选段；`asset_importers/audio`、`asset_importers/texture` 的 `package_manifest()` 补 `supported_platforms` 与带 target/capability 的 runtime module，`asset_importers/model` 删除重复 runtime module 派生；`first_party_runtime_catalog` 的 `plugins_12_static_plugin_manifest_is_generated` 覆盖清单扩到 36 个非 native manifest，并修复 guard 的多行 TOML 数组解析。验证：scoped rustfmt 通过；静态生成头扫描 36/36 通过；`cargo metadata --manifest-path zircon_plugins/Cargo.toml --format-version 1 --no-deps --locked` 通过；`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_audio_runtime -p zircon_plugin_asset_importer_texture_runtime -p zircon_plugin_asset_importer_model_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins12-missing-manifests-0622 --message-format short --color never` 通过（仅既有 `zircon_runtime` warnings）；`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12_static_plugin_manifest_is_generated --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins12-missing-manifests-0622 --message-format short --color never -- --test-threads=1` 通过 1/1（仅既有 `zircon_runtime` warnings）；完整 `plugins_12_manifest_schema_uniform`、审计脚本与 capability 四源一致性仍 pending。 |
| 2026-06-22 | M1 统一 manifest schema | T4 必选字段 uniform guard + `supported_platforms` 收敛 | plugins_12_manifest_schema_uniform_supported_platforms_guard_passed | 20 个既有一方 manifest 与 `native_dynamic_fixture/plugin.toml` 补 `supported_platforms = ["windows", "linux", "macos"]`；`RuntimePluginDescriptor::package_manifest()` 默认投影同一平台集合，避免静态 descriptor 派生 manifest 漏字段；`plugins_12_manifest_schema_uniform` 新增并检查 36 个 generated 非 native manifest + 1 个 native 手写 manifest 的 `id/version/sdk_api_version/display_name/category/description/supported_targets/supported_platforms/capabilities/maturity/[[modules]]` 必选形状，静态 required-field scan 37/37 通过。验证：scoped rustfmt/check 通过；`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12_static_plugin_manifest_is_generated --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins12-missing-manifests-0622 --message-format short --color never -- --test-threads=1` 通过 1/1；`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12_manifest_schema_uniform --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins12-missing-manifests-0622 --message-format short --color never -- --test-threads=1` 通过 1/1（均仅既有 `zircon_runtime` warnings）。完整 `tools/plugin_structure_audits/*` 脚本化入口、全 feature descriptor parity 与 capability 四源一致性仍 pending。 |
| 2026-06-22 | M1 统一 manifest schema | T4 `plugin_structure_audits` 聚合入口 + workspace guard | plugins_12_manifest_structure_audit_guard_passed | 新增仓库根 `tools/audit_plugin_structure.py` 与 `tools/plugin_structure_audits/manifest_schema.py`，从 `zircon_plugins/Cargo.toml` workspace members 反推 37 个插件根（跳过 `editor_support` / `first_party_runtime_catalog`，`features/*` 归父插件，`asset_importers/<kind>` 归对应导入器），输出 `plugin_manifest_schema_uniform` JSON：`missing_plugin_toml = 0`、`manifest_schema_violations = 0`、`expected_manifest_count = 37`、`generated_manifest_count = 36`、`generated_manifest_header_violations = 0`、`m1_gate_status = classified-and-clear`，并在 `first_party_runtime_catalog` 新增 `plugins_12_manifest_schema_uniform_audit_report_is_clean` workspace guard 消费该报告。验证：`python tools/audit_plugin_structure.py --json` 通过；`python -m py_compile tools/audit_plugin_structure.py tools/plugin_structure_audits/__init__.py tools/plugin_structure_audits/manifest_schema.py` 通过；scoped rustfmt/check 通过；`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12_ --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins12-missing-manifests-0622 --message-format short --color never -- --test-threads=1` 通过 3/3（仅既有 `zircon_runtime` warnings）。全 feature descriptor parity 已由后续记录关闭；M2/T2+ 骨架迁移与 M4 capability 四源一致性仍 pending。 |
