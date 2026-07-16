---
related_code:
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/metadata.rs
  - zircon_runtime/src/plugin/extension_registry/register/resource_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/runtime_scene_system_registration.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
implementation_files:
  - zircon_plugins/plugin_sdk/src/registration.rs
tests:
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/ai/runtime/src/tests/perception_runtime.rs
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --message-format short --color never
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
doc_type: module-detail
---

# Plugin SDK Runtime Registration

## Purpose

`RuntimePluginRegistrationBuilder` is the blessed owner-aware path for runtime plugin contributions. It interns one plugin module owner, then registers component metadata, scene resources, typed events, plugin options, bridge exports/imports, owner-revocation listeners, scene hooks, and ordered runtime scene systems without exposing owner-token plumbing to each first-party plugin.

## Registration Model

Call `RuntimePluginRegistrationBuilder::new(registry).module(module_name)` once from the plugin's sole `RuntimePlugin::register` owner. The returned module registration carries the interned owner through every supported contribution. `component(ComponentTypeDescriptor)` now covers component metadata through the same path as `resource`, `event`, `export_interface`, `import_interface`, and `runtime_scene_system`; plugins do not need to drop back to a raw `RuntimeExtensionRegistry::register_component` call.

Runtime scene systems use a builder for stage, system set, order, and explicit before/after constraints. The registration remains declarative until `register()`, after which `RuntimeExtensionRegistry::apply_to_world` installs the contributions into each runtime world. Bridge imports remain cloneable weak lifecycle bindings that are populated only after the merged catalog finalizes.

## Boundaries

The SDK hides owner propagation but does not own subsystem behavior, component storage, event payload semantics, or manager resolution. Component descriptors remain neutral metadata from `zircon_runtime::core::framework::scene`; typed component values stay in their plugin/runtime owner. Optional cross-plugin calls must use `import_interface`, not a concrete manager lookup.

## Test Coverage

The inline SDK test proves component, resource, event, interface-import, option, and ordered system contributions are all registered through one module owner. Plugins06 M4 adds a production consumer: AI registers its perception components/resources and imports `physics.query.v1` through this builder, with package and schedule assertions in `perception_runtime.rs`.

Windows managed plugin SDK job `839e57989b4842ccab52fb635cc71548` passed the current-source build, unit tests, and doctests. The same source was consumed by AI package job `14793e415ec1442c8de52545b1d59eed` and upward Runtime build `62e2f5f56994496e984a1f3eb4142d1b`; both exited successfully.
