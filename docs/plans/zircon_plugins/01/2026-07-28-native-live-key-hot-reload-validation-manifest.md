---
doc_type: milestone-validation-manifest
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/keys.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs
tests:
  - native_hot_reload_owned_identity_reinserts_into_its_module_kind_partition
  - native_live_host_rollback_plan_restores_existing_plugin_when_reload_fails_before_unload
  - Plan08 plugin-list commandlet current-source managed gate
---

# Native Live-Key Hot-Reload Validation Manifest

Plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
Failure: docs/plans/zircon_plugins/01/failure-2026-07-27-native-live-key-hot-reload-contract-drift.md
Status: in_progress
Files: ["docs/plans/zircon_plugins/01/failure-2026-07-27-native-live-key-hot-reload-contract-drift.md", "docs/plans/zircon_plugins/01/2026-07-28-native-live-key-hot-reload-validation-manifest.md", "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/keys.rs", "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs", "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs", "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs", "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs"]

The owned transition identity must rebuild the typed, module-kind-aware registry key at rollback
and successful reinsertion. Run the two runtime regressions before the Plan08 commandlet gate.
This manifest binds only managed current-source validation; it is not a failure return or acceptance record.
