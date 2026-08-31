# Runtime 06 loaded native plugin owner split

## Scope

- Target: `zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs`.
- Baseline: clean 677-line tracked owner containing the public loaded-plugin facade, asset-import command adapter, dynamic-library generation pin, callback admission/diagnostics, behavior snapshot invocation, and lifecycle callback helpers.
- Priority sources: Runtime 06, the engine structure convention/review findings, Runtime plugin-interface review 99p, and its Runtime 58 predecessor.
- This slice changes ownership only. It does not change the native ABI, callback admission algorithm, atomic memory ordering, lifecycle transition policy, diagnostics mode, behavior call order, error text, or public `plugin::native` surface.

## Architecture review before optimization

The existing callback generation protocol is a retained foundation, not a local algorithm defect to rewrite. One `Arc<NativePluginStableLibrary>` pins a loaded library generation; the high activity bit closes admission for lifecycle transition while the remaining bits count active foreign callbacks. Behavior snapshots retain a passive generation owner and acquire a callback lease only immediately before foreign code invocation. Duration diagnostics are isolated in 64 cache-line-aligned shards.

The primary local Unreal references were `Runtime/Core/Public/Modules/ModuleManager.h`, `ModuleInterface.h`, and `Runtime/Projects/Public/Interfaces/IPluginManager.h`. Unreal separates plugin discovery/descriptor identity from module load/unload and module behavior lifecycle callbacks. Zircon should preserve that separation while retaining its stricter generation pin and callback admission protocol. The previous single Rust file mixed those distinct ownership domains even though the runtime behavior was already sound.

Runtime review 99p still identifies broader open work: a unified bridge/native call lease, holder census, safe-point proof, transactional generation publication, World binding generation, revisioned diagnostics, and retirement evidence. This split does not claim those P0/P1 items are closed; it creates replaceable owners for the retained native foundation.

## Implemented layout

| Owner | Responsibility | Current lines |
|---|---|---:|
| `loaded_native_plugin.rs` | Loaded generation facade, public reports/accessors, and lifecycle orchestration | 306 |
| `loaded_native_plugin/callback.rs` | Stable library generation, atomic callback admission, lease/drop, transition gate, and sharded diagnostics | 285 |
| `loaded_native_plugin/behavior.rs` | Generation-pinned behavior snapshot invocation and typed rejection/missing reports | 95 |
| `loaded_native_plugin/asset_import.rs` | `NativeAssetImportCommandHost` status projection | 37 |

`NativePluginCallbackDiagnostics` is still publicly projected by `loaded_native_plugin`, and all sibling-only lease, owner, snapshot, and lifecycle error types retain their original `loaded_native_plugin::*` paths through restricted re-exports.

## Preserved invariants

- All 60 original function definitions remain present; two private `new` constructors were added so child fields do not become visible across owners.
- All nine original struct/enum names remain present with zero difference.
- All 26 string literals match the baseline as a multiset with zero difference.
- Atomic ordering counts are unchanged: Acquire 4, Release 3, Relaxed 11, and AcqRel 1.
- Weak CAS, strong CAS, fetch-add/sub/max, transition-bit, callback-count mask, 64-shard diagnostic constant, measurement, admission, and lifecycle transition occurrence counts are unchanged.
- Callback lease drop still decrements activity before library generation ownership can retire.
- Lifecycle callbacks still execute only after transition admission closes ordinary callback leases, and their measurement path still bypasses ordinary callback counting exactly as before.
- The asset command adapter retains the same OK/error/denied/panic/unknown status mapping.

## Current evidence and status

- Scoped `rustfmt --edition 2021 --check` passed for all four Rust owners.
- Static migration comparison retained original functions `60/60`, types `9/9`, and string literals `26/26`; only the two private constructors are new.
- Root size changed from 677 to 306 lines; every owner is at or below 306 lines.
- The facade contains no atomic, thread-local shard, behavior snapshot declaration, or asset-host trait implementation.
- Production owners contain no new `allow`, `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` escape path.
- Scoped whitespace/conflict scans and tracked-root `git diff --check` passed; Git emitted only the repository's LF/CRLF checkout notice.
- Managed Cargo and native fixture validation were not requested while bypassing the shared validation blocker.
- Status is `runtime_06_loaded_native_plugin_owner_split_implemented_static_passed_managed_validation_deferred_atomic_protocol_unchanged`.

## Required structural and performance follow-up

Before optimizing callback admission or diagnostics, measure the full bridge/native path rather than only this atomic owner: diagnostics off/sampled/on, 1/100/1,000 providers, concurrent World calls, active-call reload, rejected transition, drain latency, holder lifetime, callback p50/p95/p99/max, atomic/cache-miss cost, DLL retirement latency, and power/CPU residency. The experiment must compare equivalent native work and lifecycle behavior against the retained implementation and a local Unreal module/plugin lifecycle workload; no result from a synthetic callback loop alone may establish product parity.

The next structural work remains Runtime 99p M2/M3: one call lease must pin table/provider/library generations, expose holders, close admission at a proved safe point, drain or reject with a deadline, and publish provider/binding/import/diagnostic generations transactionally. No latency, throughput, memory, energy, power, or Unreal-parity improvement is claimed in this record.
