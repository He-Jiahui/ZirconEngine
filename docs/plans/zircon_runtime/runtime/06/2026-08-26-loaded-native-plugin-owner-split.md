---
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin/asset_import.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin/behavior.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin/callback.rs
implementation_files:
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin/asset_import.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin/behavior.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin/callback.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/optimize/zircon_runtime/99p-runtime-plugin-interface-bridge-slot-generation-strong-weak-native-vm-lifecycle-diagnostics-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/58-runtime-plugin-interface-bridge-slot-generation-strong-weak-native-vm-lifecycle-diagnostics-product-integration-review.md
tests:
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/callback_lease.rs
  - rustfmt --edition 2021 --check
  - static function, type, string, and atomic-protocol migration comparison
  - git diff --check
doc_type: milestone-detail
---

# Runtime 06 loaded native plugin owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| M2/M3 | Loaded native generation folder-backed owner split | `runtime_06_loaded_native_plugin_owner_split_implemented_static_passed_managed_validation_deferred_atomic_protocol_unchanged` | 2026-08-26 | Root 677 -> 306 lines; callback/behavior/asset adapter owners 285/95/37 lines; original functions 60/60, types 9/9, strings 26/26, and all atomic-protocol counters retained. |

Completed:

- Kept `LoadedNativePlugin` identity, reports, public accessors, behavior facade, and lifecycle orchestration in the root owner.
- Moved stable-library generation ownership, callback lease/drop, transition admission, and 64-shard diagnostics into the callback owner.
- Moved generation-pinned behavior invocation and rejection/missing reports into the behavior owner.
- Moved the asset import command trait adapter and status projection into the adapter owner.
- Preserved all existing type paths with a public diagnostics projection and restricted sibling projections for internal lease/lifecycle types.
- Recorded the profiling requirements and Runtime 99p transaction/holder work that must precede algorithm claims.

## Review basis

Local Unreal separates plugin discovery/descriptor ownership from module load/unload and module behavior lifecycle callbacks. Runtime 99p confirms Zircon's transition-bit callback lease and generation owner are retained foundations, while the unified bridge/native call lease, holder census, safe-point transaction, World generation binding, and retirement evidence remain open. This slice aligns source ownership with those boundaries without changing the retained runtime protocol.

There is no compatibility module, duplicate implementation, public API expansion, ABI change, callback admission change, atomic-ordering change, lifecycle policy change, new allocation, or performance claim.

## Verification

- Scoped `rustfmt --edition 2021 --check` passed for all four Rust files.
- Static migration comparison retained all 60 original functions, all nine original types, and all 26 string literals; two private constructors were added for child encapsulation.
- Atomic counts remain Acquire 4, Release 3, Relaxed 11, AcqRel 1; CAS/fetch, transition-bit, callback-count, shard, measurement, admission, and transition counts have zero delta.
- The root no longer owns atomics/thread-local shards, behavior snapshot declarations, or the asset command trait implementation.
- All owners are at or below 306 lines; whitespace and conflict-marker scans passed.
- Tracked-root `git diff --check` passed with only the repository LF/CRLF checkout notice.
- Managed Cargo, native dynamic fixture, hot-reload, concurrency, profiler, and power validation were not run while bypassing the current shared validation blocker.

## Open scope

Runtime 06 and the complete runtime architecture remain `in_progress`. M2/M3 accepted closeout, Runtime 99p's three P0 and 64 P1 items, unified table/provider/library call lease, holder census, safe-point proof, transactional generation publication, World replay binding, drain/retirement evidence, managed validation, milestone commit, coordinator integration receipt, and WeCom publication are not completed by this slice.
