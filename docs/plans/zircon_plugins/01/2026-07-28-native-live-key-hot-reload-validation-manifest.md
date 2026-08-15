---
doc_type: milestone-validation-manifest
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/keys.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_publication.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/runtime_behavior.rs
tests:
  - native_hot_reload_owned_identity_reinserts_into_its_module_kind_partition
  - native_live_host_rollback_plan_restores_existing_plugin_when_reload_fails_before_unload
  - bulk_reload_reopens_the_old_generation_when_loaded_lock_reacquisition_fails
  - unload_reopens_the_retained_generation_when_loaded_lock_reacquisition_fails
  - hot_reload_reopens_the_retained_generation_when_loaded_lock_reacquisition_fails
  - hot_reload_reports_replacement_cleanup_failure_after_publication_fails
  - hot_reload_keeps_retained_generation_transition_active_when_rollback_restore_fails
  - hot_reload_keeps_retained_generation_transition_active_when_publication_rollback_restore_fails
  - cargo test --package zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 native_plugin_live_host::tests::hot_reload
  - Plan08 plugin-list commandlet current-source managed gate
---

# Native Live-Key Hot-Reload Validation Manifest

Plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
Milestone: M5
Failure: docs/plans/zircon_plugins/01/failure-2026-07-27-native-live-key-hot-reload-contract-drift.md
Status: resolving_failure
Files: ["docs/plans/zircon_plugins/01/failure-2026-07-27-native-live-key-hot-reload-contract-drift.md", "docs/plans/zircon_plugins/01/2026-07-28-native-live-key-hot-reload-validation-manifest.md", "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs", "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs", "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_publication.rs", "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/runtime_behavior.rs"]

The owned transition identity rebuilds the typed, module-kind-aware registry key at rollback and
successful reinsertion. The managed Windows Rust 1.94.1 `zircon_runtime` build and focused
`native_plugin_live_host::tests::hot_reload` gate have passed, and two independent current-source
reviews returned `C0/I0/M0` for the rollback-atomicity follow-up. The originating Plan08
commandlet gate was then attempted but stopped in shared `zircon_runtime` compilation on five
external render/text diagnostics before its target filter ran. This manifest remains open until a
fresh managed Plan08 commandlet run executes its target tests; it is not a failure return or
acceptance record. A 2026-08-10 static re-audit found all five historical paths replaced and their
current boundaries carrying the expected compile repairs, so the upward gate is source-ready for a
fresh managed retry. That audit is not Cargo evidence; an unrelated active `zircon_runtime`
Cargo/rustc process prevented the retry in this slice.
