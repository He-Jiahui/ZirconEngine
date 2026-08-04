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

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-plugins-dx-and-structure-framework",
  "goal": "统一插件 manifest、骨架、注册入口、capability 单源与存量硬切，并保留可审计的里程碑提交证据。",
  "milestones": [
    {"id": "M1", "title": "统一 manifest schema", "depends_on": []},
    {"id": "M2", "title": "骨架与 SDK", "depends_on": ["M1"]},
    {"id": "M3", "title": "注册收编", "depends_on": ["M2"]},
    {"id": "M4", "title": "capability 单源与 editor/runtime 对称", "depends_on": []},
    {"id": "M5", "title": "存量硬切", "depends_on": ["M2", "M3", "M4"]}
  ]
}
```

<!-- Workflow topology is maintained independently from milestone output records. M4 is an independently committable late-adoption slice because M1–M3 predate coordinator workflow evidence; the task-level dependency remains authoritative in §4. -->

## 2. 2026-06 初始缺口基线与后续状态（保留历史路径证据）

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

当前状态（2026-08-01）：S1-S6 是已关闭的历史基线，不再表示当前缺口。asset importer 已生成 `plugin.toml` 并由 `runtime/src/plugin.rs` 的 `RuntimePlugin` 实现负责注册；animation capability 已收敛到 `runtime/src/capability.rs`；结构审计与 §6 checklist 记录 `missing_plugin_toml = 0`、`manifest_schema_violations = 0`、`free_function_registration_sites = 0`、`native_crate_name_collisions = 0`。S7-S12 各行包含后续状态，其中已关闭项同样不得被读作待办。真正未关闭范围以 §9 的 open failure 和跨计划联动为准。

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

- [x] 旧自由函数注册路径已删除，调用方走 `impl RuntimePlugin::register`（2026-07-10 全插件 runtime 根审计新增 `free_function_registration_sites`，当前为 0；support crate `plugin_sdk` 不误计入插件根）
- [x] 缺失 `plugin.toml` 已补齐且过 schema 校验（2026-07-15 当前真实审计 `missing_plugin_toml = 0`、`manifest_schema_violations = 0`；`first_party_editor_catalog` 是 workspace support catalog，不是可分发插件根，已由审计分类排除，未用伪造 manifest 消除告警）
- [x] capability 常量为 `capability.rs` 单源，四源一致（2026-06-23 已完成 15 个 trait-backed first-party runtime 根的 M4/T1 首批 guard，并完成 `PluginFeatureBundleBuilder` + editor `mirrors_runtime(...)` 的 M4/T2 SDK guard；2026-06-28 D5 editor authoring macro consumer guard 已让 animation/physics/net editor plugin 使用 `zircon_plugin_sdk::authoring_plugin!` 生成 struct/Default/declaration/EditorPlugin impl，状态锚 `d5_editor_authoring_macro_consumers_static_passed_cargo_deferred`，守卫 `review_d5_editor_authoring_plugins_use_sdk_macro`；同日 D9 editor/runtime mirror consumer guard 已让 animation/physics/net editor plugin 通过 SDK macro 的 `mirrors_runtime_manifest:` / `EditorPluginDeclaration::mirrors_runtime_manifest` mirror runtime manifest，审计输出 `editor_runtime_mirror_violations = 0`、`d9_editor_runtime_mirror_gate_status = editor-runtime-mirror-clean`；同日 M5/T1 importer capability-owner、runtime-only skeleton owner、editor-only skeleton owner、authoring runtime/editor skeleton owner、particles/physics/texture skeleton owner、editor_build_export_desktop skeleton owner、sound runtime/editor feature skeleton owner、timeline_sequence editor skeleton owner 与最终 8 根 owner rollout 已把 skeleton migration debt 从 35 降至 0；`migration_debt_roots = []`）
- [x] native 双 crate_name 已显式区分 runtime / editor（2026-07-10 新增 `native_crate_name_collisions`；共享 cdylib 仅在 runtime/editor `kind` 明确且互异时合法，当前为 0）
- [x] 无兼容 re-export / 双轨；删除清单写进提交说明（2026-07-10 `registration_compatibility_shim_sites = 0`、`m3_hard_cut_gate_status = registration-hard-cut-clean`；本切片删除清单为空并已写入产出记录）

## 7. 完成定义

`m1_gate_status = classified-and-clear`、`plugin_skeleton_gate.m2_gate_status = sample-clean-migration-debt-clear`、M5 收口后 `migration_debt_count = 0`、各 violation 字段 = 0、`exempt` 仅含登记豁免项、镜像守卫绿；`cargo test --manifest-path zircon_plugins/Cargo.toml --workspace`、`cargo build --manifest-path zircon_plugins/Cargo.toml --workspace`、`cargo fmt --all --check`、`tools/audit_plugin_structure.py --json` 无 risk。

## 8. 联动

- M1 manifest schema 与 Plugins 01 §3 的 `PluginPackageManifest` 字段对齐，扩 schema 校验不破坏 01 已落 `declared_system_anchors_are_registered`；capability 四源 guard 是 01 已提"四源一致性"的机器化落地。
- plugins index §2 依赖图：01 定稿 → 12 schema/骨架 → 02–11 照用；本计划列为波次零前置。
- M5 与各插件能力计划（02 Sound … 11 调用桥）同窗口：插件被实质改动时同一变更迁骨架，避免一次性大爆炸冻结全工作区。

## 9. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`12/2026-07-09-plugin-dx-and-structure-framework-output-records.md`](12/2026-07-09-plugin-dx-and-structure-framework-output-records.md)
- 当前源码与计划收敛：[`12/2026-08-01-current-source-plan-convergence.md`](12/2026-08-01-current-source-plan-convergence.md)
- 当前收口记录：[`12/2026-07-15-plugin-runtime-event-consumer-output-records.md`](12/2026-07-15-plugin-runtime-event-consumer-output-records.md)（event generation 精确里程碑已提交为 `663537c7`；linked runtime module 受管测试 3/3，app Navigation 产品门禁 job `188a8df88f10431c8240845ad440dd05` 为 1/1。代码语义复核 P0/P1 为 0，但 exact 16-file manifest 复核为 P0=0、P1=3、P2=1：Editor09 fixed lifecycle 尚未落地、共享 interface blob 依赖 Editor03 operation owner、event-mirror 的 module/World/test wiring 未完整纳入；必须按 owner 有序提交后重建 manifest，因此计划状态保持 `in_progress`）
- fixed 已修复：[plugin-editor-runtime-mirror-consumer-wiring](05/fixed-2026-07-15-plugin-editor-runtime-mirror-consumer-wiring.md)
- fixed 已修复：[plugin-mirror-v1-runtime-fallback](../zircon_editor/editor/03/fixed-2026-07-15-plugin-mirror-v1-runtime-fallback.md)
- fixed 已修复：[plugin-structure-audit-report-fixture-drift](../zircon_editor/editor/09/fixed-2026-07-15-plugin-structure-audit-report-fixture-drift.md)
- fixed 已修复：[repeated-milestone-slice-manifest-selection-conflict](12/fixed-2026-08-04-repeated-milestone-slice-manifest-selection-conflict.md)
- 2026-07-22 event mirror生命周期性能交接：公开connected subscription直接drop不会回减World reader/callback count，Navigation等按需producer可因此永久保持debug capture；Plugins12统一direct token、dynamic session、World destroy与plugin reload的generational owner/reclaim状态机。open `待修复`：[`12/failure-2026-07-22-runtime-event-mirror-drop-lifecycle.md`](12/failure-2026-07-22-runtime-event-mirror-drop-lifecycle.md)，见PERF-MVP-455。
- 2026-07-22 asset importer generation交接：Plugins12按PERF-MVP-503为descriptor/capability/plugin load-unload提供稳定generation与delta，Runtime04据此维护extension/full-suffix/id/plugin索引；同一plugin transaction一次规范化matcher，卸载只移除owned slots。禁止插件各自维护第二套registry或让Editor通过全descriptor clone发现能力；见Runtime04 `asset-importer-generation-index` open failure。
- 2026-07-22 importer VG request policy交接：glTF、OBJ与model runtime importer当前对所有非蒙皮primitive无条件同步调用`cook_virtual_geometry_from_mesh`。Plugins12按PERF-MVP-509把VG请求收敛为project/profile/plugin capability驱动的typed policy，feature-off必须0 cook；插件只提交Runtime04 content+config ticket并消费共享artifact，不自行缓存、开线程或重复cook。runtime内置路径使用同一policy，reload/unload精确失效。
- 2026-07-22 asset-type bulk generation补充：PERF-MVP-562要求Plugins12按一次plugin/catalog generation向Editor09提交transactional contribution batch；同一asset type的templates/commands聚合后只允许一次validate/merge/sort/publish，reload/unload只发布一个successor generation。禁止逐descriptor调用`apply_contribution`造成O(N²) `Vec`搬移与千次consumer cache失效。
