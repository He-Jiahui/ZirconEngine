---
status: proposed
created_at: 2026-08-25
owner_plan: docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
related_findings:
  - PERF-MVP-318
  - PERF-MVP-319
  - config-manager-synchronous-full-file-rewrite
related_code:
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/foundation/runtime/config_manager.rs
  - zircon_runtime/src/foundation/runtime/config_manager/state.rs
  - zircon_runtime/src/foundation/runtime/config_manager/worker.rs
  - zircon_runtime/src/foundation/runtime/config_manager/commit_fence.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigCacheIni.h
---

# Runtime02 Config Authority Transaction Performance Plan

## Decision

Do not optimize `ConfigStore` by replacing its mutex, adding a per-caller cache, or
moving another JSON clone. The current bottleneck is architectural: one raw in-memory
map is used both as the runtime read surface and as the snapshot source for a separate
persistence manager, while callers may bypass that manager entirely. The required hard
cut is a single versioned configuration authority with distinct durable and session
capabilities.

This is a research and implementation plan, not a performance acceptance record. No
current-source CPU, allocation, lock-wait, I/O latency, power, or engine-comparison
claim is made here. Windows-native Cargo and WPR evidence must be collected by the
coordinator before the implementation phase is admitted.

## Current Source Evidence

| Area | Current behavior | Consequence |
|---|---|---|
| `core/runtime/config_store.rs` | A single `Mutex<HashMap<String, Arc<Value>>>` owns raw JSON values. Typed reads borrow the `Arc<Value>` and deserialize on every load; `load_value` and `snapshot_values` return owned JSON. | PERF-MVP-318 removed the old typed-read deep clone, but did not create a generation, typed cache, schema, or transaction boundary. Full snapshot and raw read paths still clone JSON. |
| `core/runtime/handle/events.rs` and `runtime.rs` | `store_config_value`, `load_config_value`, `snapshot_config_values`, `store_config`, and `load_config` remain raw public Core APIs. | Any subsystem can mutate the persistence source without an explicit scope, validation, revision, durability decision, or change receipt. |
| `foundation/runtime/config_manager.rs` | `DefaultConfigManager` loads the same Core map and its worker snapshots that map. It already has dirty generations, trailing debounce, one named worker, atomic replace, report counters, bounded flush, and commit fencing. | The worker is valuable infrastructure to retain, but it cannot tell whether a raw Core write was durable, session-only, validated, or part of a multi-key operation. |
| `foundation/runtime/config_path.rs` | The manager has one user-local `config.json` target. | Project, user, command-line/environment, and session precedence are not modeled; the current target cannot be used as an implicit policy for every configuration key. |
| Production callers | Animation playback settings directly read/write the raw Core map. Dynamic runtime construction writes the render profile into that map, and the render bridge reads it back. | Animation changes can bypass persistence; a dynamic-session render choice can leak into shared runtime state. These are three behavior owners and five direct raw accesses, a small enough migration surface for a hard cut. |

The existing failure record confirms that the persistence worker already covers
same-value retry after write failure, burst coalescing, atomic replacement, bounded
shutdown, and late-writer fencing. Reimplementing that worker would add risk without
resolving the authority split.

## Unreal Reference Boundary

`ConfigCacheIni.h` deprecates direct mutable section access specifically because the
engine cannot reliably track and save unobserved mutations. It directs mutation through
set/add/remove APIs, carries a change tracker, distinguishes cache files and hierarchy
loading, exposes explicit flush/unload behavior, and makes asynchronous load completion
observable. Zircon should absorb those ownership principles, not Unreal's global C++
singleton or text-INI representation:

- all production changes enter an auditable mutation API;
- layers and persistence targets are explicit rather than inferred from call order;
- readers observe a sealed generation, and background durability is separated from
  runtime mutation;
- session-specific state is never silently written to the user configuration target.

## Target Architecture

```text
ConfigRegistry + typed ConfigKey<T>
  -> ConfigTransaction at observed revision
  -> validate/migrate each affected durable or session layer
  -> publish immutable ConfigGeneration with revision and source report
  -> durable target delta to existing ConfigPersistenceWorker
  -> atomic file publication and persistence receipt

Runtime consumers
  -> ConfigReadHandle / generation-qualified typed binding
  -> durable user/project value OR session override, never raw global JSON
```

### Authority and data model

1. Add `ConfigKey<T>` descriptors owned by `core/runtime/config`, with stable key id,
   schema and key version, scope, persistence target, default, validator, migration,
   restart/live-apply policy, and defining module owner. A key is registered during
   module activation; duplicate id or incompatible schema fails activation.
2. Replace the raw map's public mutation surface with a `ConfigAuthority`. It publishes
   an immutable `ConfigGeneration { revision, layers, effective_values, report }` and
   exposes a short-lived read handle. Readers capture one generation before decoding so
   they cannot combine values from unrelated revisions.
3. Use explicit precedence: built-in defaults, engine/project, user/editor,
   command-line/environment, session override. Session overrides have a separate type
   and never enqueue a durable write. Unknown, deprecated, invalid, and migrated input
   is retained in a load report rather than silently becoming a default.
4. `ConfigTransaction` takes an observed revision, validates every requested key before
   publication, and publishes all resulting values or none. It returns a revision and
   durability receipt. Conflicting writers receive an explicit conflict and retry from a
   new read handle; no caller performs read-modify-write through `Value` directly.
5. Retain `ConfigPersistenceWorker`, `ConfigCommitFence`, and atomic-file owner. Change
   its snapshot input from `Fn() -> HashMap<String, Value>` to a sealed, target-specific
   durable generation. The worker must only serialize a committed generation and report
   the revision it persisted; it must not build its own authority snapshot.
6. Provide generation-qualified typed bindings for long-lived managers. The binding
   only deserializes when its key's effective revision changes, so frame-time callers do
   not repeat `serde_json` decoding. A read handle may still decode a cold key, but it
   must do so outside the authority lock.

### Planned consumer hard cut

| Owner | Replacement | Persistence policy |
|---|---|---|
| `DefaultAnimationManager` | A typed `AnimationPlaybackSettings` key and binding. `store_playback_settings` commits through the config transaction. | Durable user/editor or project policy declared by the key; restart behavior is explicit. |
| Dynamic runtime session construction and render bridge | A `RuntimeSessionConfig` override passed through dynamic session construction to the render bridge. | Session-only. It does not touch user/project persistence or shared Core config. |
| `CoreHandle` and `CoreRuntime` raw config methods | Migrate the two owners above, then remove production raw write/read/snapshot APIs. Raw JSON remains only inside file/ABI adapters. | No compatibility facade, dual write, or fallback path. |

The public `ConfigManager` service evolves only after the typed authority exists. Its
replacement surface returns typed results and receipts; it does not acquire a second
store beside `ConfigAuthority`.

## Delivery Order

1. **Evidence harness first.** Add failing source tests for layer precedence, invalid
   schema, migration, multi-key rollback, revision conflict, session non-persistence,
   generation-consistent reads, and raw Core API absence. Add a scenario harness that
   emits stable operation counters without treating unit-test timing as a benchmark.
2. **Authority kernel.** Introduce the registry, descriptors, immutable generation,
   transaction validation, and load report under `core/runtime/config`. Keep the current
   worker unchanged except for a temporary adapter owned by the new authority.
3. **Durable integration.** Feed target-specific sealed generations to the existing
   worker, preserve its debounce/atomic/fence behavior, and add revision-specific
   persistence receipts and shutdown tests.
4. **Consumer migration.** Move animation to the durable typed key and dynamic render
   configuration to a session override. Delete raw Core configuration methods and the
   `HashMap<String, Value>` snapshot adapter in the same change set.
5. **Performance closure.** Compare source-bound before/after runs, publish raw samples
   and environment metadata, then decide whether the generation publication structure
   itself needs further work. Do not claim a comparison with Unreal, Unity, or another
   engine without equivalent workloads and measurements.

## Measurement Gate

The coordinator must run Windows-native, source-bound measurements after the evidence
harness exists. Store every temporary workload file, ETL trace, raw sample, and report
under `E:\Git\ZirconEngine\.codex\test-tmp\config-authority-*`; do not create
artifacts on `C:`.

| Scenario | Cardinalities | Required measures |
|---|---|---|
| Typed reads | 1 KiB, 1 MiB, 100 MiB values; 1, 100, 10k keys; read:write 1:1 and 100:1 | deserialize count, cloned JSON bytes, allocation count/bytes, authority lock hold/wait, p50/p95/p99 CPU time |
| Transactions | 1 and 1k-key commits; valid, invalid, and revision conflict | validation/publish latency, retries, partial-publication count (must be zero), retained generations/bytes |
| Persistence | 1 and 1k update burst; successful write, injected write/replace failure, shutdown flush | caller filesystem time, worker queue depth, coalescing ratio, writes, serialized bytes, p95/max flush time, old/new-file recovery |
| Runtime ownership | animation settings update and dynamic pipelined render session | frame p95/p99 perturbation, session override leak count (must be zero), restart roundtrip, durable receipt revision |

For each scenario, collect a WPR CPU and file-I/O trace plus the harness counters, CPU
model, Windows build, power plan, driver versions, source fingerprint, sample order, and
noise interval. No current executable or WPR trace was available in this session, so the
baseline is explicitly **not collected**. The current 64-sample flush report is useful
diagnostics, not a replacement for this controlled measurement.

## Admission and Non-goals

- Do not change lock types, introduce a global interner, or cache deserialized values by
  ad hoc string key before the authority and generation contract exists.
- Do not move user config to project config, or persist dynamic-session choices.
- Do not run Cargo, build a benchmark, or generate profiler artifacts locally while the
  coordinator owns the validation lane.
- This plan does not modify diagnostics, trace capture, or GPU profiling; those remain
  owned by Runtime03 and Render17.
- The implementation may start only after the evidence harness and coordinator baseline
  have been accepted. Until then, this document is the actionable prerequisite and the
  existing persistence worker remains the production implementation.
