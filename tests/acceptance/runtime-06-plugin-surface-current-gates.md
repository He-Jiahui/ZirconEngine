---
related_code:
  - zircon_runtime/src/plugin
  - zircon_runtime/src/script/vm/tests/lifecycle_failures.rs
  - zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle
plan_sources:
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
output_records:
  - docs/plans/zircon_runtime/runtime/06/2026-07-09-plugin-surface-and-lifecycle-output-records.md
status: owned_plugin_native_gates_accepted_cross_package_zrvm_gates_remain
---

# Runtime 06 Plugin Surface Current Gates

Date: 2026-07-11

## Current evidence

- `vm_lifecycle_fallback`: 5/5 passed.
- `vampire_project_session`: 1 passed and 10 correctly ignored because the
  default-feature lane does not provide `backend-zr-vm` and
  `ZR_VM_RUST_BINDING_LIB_DIR`.
- `plugin_surface_lifecycle` initially ran 3/5. Both failures were stale
  evidence routing: the guards read route-only parent/status aggregator files.
  They now read the numbered Runtime 06/15/Frameworks output records and the
  folder-backed status/date child rows; current-source standalone validation is
  4/4 passed (the package cargo-gate test was already green in the old binary).
- `native_plugin` reached the hot-reload failure-injection family but aborted
  after a deliberately failed callback poisoned the shared restored-payload
  mutex. Every callback/test access now recovers the poisoned guard through
  `PoisonError::into_inner`, preventing `extern "C"` from panicking across the
  non-unwinding FFI boundary. In the new package binary, the plugin-surface
  filter passes 5/5 and both the ordinary state snapshot test and the injected
  restore-failure rollback test pass 1/1.
- The three tests that share the restored-payload fixture now hold one explicit
  fixture lock for their full arrange/act/assert lifetime. The newly compiled
  `native_plugin` aggregate passes 139/139 in 135.15s, replacing the earlier
  138/139 race-dependent result.

## Decision

Runtime 06 remains `in_progress`. The owned static lifecycle, VM fallback, and
native-plugin aggregate gates are accepted. Full app/plugin-workspace and real
ZrVM gates are not yet claimed complete.
