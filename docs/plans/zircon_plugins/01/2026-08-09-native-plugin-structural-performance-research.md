---
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_host_handle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/abi_decode
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/context_handles
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/bridge_scope
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/registration_policy
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/ecs_registration
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_callbacks.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/runtime_behavior.rs
  - zircon_runtime/src/plugin/bridge/import.rs
  - zircon_runtime/src/plugin/bridge/weak.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/diagnostics/store.rs
  - zircon_plugins/physics/runtime/src/runtime_system.rs
  - zircon_plugins/physics/runtime/src/diagnostics.rs
  - zircon_plugins/net/runtime/src/runtime_system.rs
implementation_files:
  - zircon_runtime/src/plugin/native_plugin_loader/benchmark_harness.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_host_handle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/abi_decode
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/context_handles
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/bridge_scope
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/registration_policy
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/ecs_registration
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/callback_lease.rs
plan_sources:
  - user: 2026-08-09 native plugin structural performance optimization request
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
tests:
  - zircon_runtime/tests/native_plugin_loader_contract.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/runtime_behavior.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/callback_lease.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_publication.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/abi_decode/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/context_handles/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/bridge_scope/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/registration_policy/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/ecs_registration/tests.rs
  - tools/tests/test_plugins_01_host_api_adapter_boundary.py
  - tools/tests/test_plugins_01_native_benchmark_harness_contract.py
  - docs/plans/zircon_plugins/01/failure-2026-07-17-bridge-import-stable-call-double-mutex.md
  - docs/plans/zircon_plugins/01/failure-2026-07-22-native-callback-per-call-lease-and-abi-copy.md
doc_type: milestone-detail
status: m0_accepted_benchmark_harness_static_complete_validation_pending
---

# Plugins01 Native Plugin Structural Performance Research

## Scope And Decision Gate

This record is the required pre-implementation design and measurement plan for the Plugins01
native-loader performance work. It does not claim a current-source performance result, a completed
failure return, or a product/MVP acceptance result.

The MVP baseline in `docs/plans/mvp/index.md` remains `in_progress`. The explicit user request
authorizes this research and the later lowest-layer repair, but does not authorize treating the
advanced plugin path as an MVP completion shortcut.

No optimization code may be written until a current-source Windows trace identifies the dominant
cost among the workloads below. Coordinator job `c9548a72e5c341359e0ce0525eae1d1f` was
terminalized `failed/124` and released after coordinator recovery; it produced no test output and
is not performance evidence.

## Current-Source Audit

The current implementation already removes two earlier hot-path faults:

| Area | Current mechanism | Audit conclusion |
| --- | --- | --- |
| Native callback call | `NativePluginBehaviorSnapshot` retains a generation `Arc`, then acquires an atomic in-flight lease immediately before the foreign callback. The loaded registry guard has already been dropped. | Do not reintroduce a callback mutex or copy a dynamic-library handle to simplify a new change. The current static review found no foreign callback while holding `loaded`. |
| Native host context lookup | `HostContextRegistry` uses a generation-encoded handle, immutable `ArcSwap` page directory, context `Arc` pin, and a second generation check. The writer mutex is confined to allocation, reuse, and retirement. | Do not replace the design with `RwLock` or a global handle map. Its contention status still needs ETW confirmation at production-shaped load. |
| Bridge import call | `BridgeImport`/`WeakBridge` publish generation/provider snapshots with `ArcSwap`; the documented stable path has no binding/provider mutex. | Preserve the in-flight `Arc` lifetime and disabled-generation semantics during any catalog consolidation. |
| Native system scheduling | Direct ABI V3 uses `NativeDynamicAccess` as the conservative main-thread fallback. ABI V4 validates version, layout, explicit stable component/resource access, affinity, and granted capability before `register_external_native_system` compiles real `SystemParamAccess`; the current V4 worker-safe regression proves that access is not conservative. | Do not "optimize" the scheduler by weakening V3 safety or duplicating the access authority. Profile only production-shaped V4 registrations if native scheduling is suspected. |

### Static Review Update (2026-08-10)

The current M5 hot-reload publication and rollback review is C0/I0/M0 at static scope. The
reviewed `lifecycle.rs` snapshot is
`1EC59252A032BC6741C4816A509F7E22E8A851928BCA32D9E92BFA778D8DE2DC`; its paired,
currently untracked external test snapshot `tests/hot_reload_publication.rs` is
`795869C3962C96ED0E0D771078B8596C58FEDCAD77F103E3B2975D750AB8339C` and must not be
absorbed by this session. The tests make the publication-failure contracts explicit: replacement
cleanup occurs before retained callback admission reopens, cleanup diagnostics remain visible, and
a retained generation remains closed when its restore fails. The bridge-publication path drops the
loaded registry guard before either plugin callback. This static review is paired with the
completed managed Windows Rust 1.94.1 core build and focused
`native_plugin_live_host::tests::hot_reload` gate for this same source snapshot. It is still not
performance evidence or M5 product acceptance: the subsequent Plan08 commandlet gate stopped
before its target filter on five external render/text compilation diagnostics, and no
release-profile measurement has been authorized while Frameworks06 M0 is open.

The remaining structural pressure is not a proven micro-hotspot but an ownership problem:

- `NativePluginLiveHost` owns the loaded registry plus bridge bindings, bridge generations,
  registration-replay generations, revisions, and separate build locks. These are related views of
  one plugin generation, but they are published and invalidated independently.
- `host_api_adapter.rs` is 1,161 lines and mixes ABI decoding, V3/V4 policy validation, ECS
  registration, stable context handles, bridge scopes, foreign-call containment, and lifecycle
  closeout. This is a boundary that must be decomposed before adding another optimization branch.
- `RuntimePluginCatalog` has its own generation/projection and a mutex-protected project-plan
  cache. The native host has a different lifecycle authority for DLL quiescence, rollback, and
  bridge/replay snapshots. A future product publication may compose validated snapshots from both
  domains, but it must not collapse their mutable storage, locks, or revision counters into one
  cache merely to give the composition a single name.
- Feature-enabled bootstrap has a separate, unmeasured structural candidate: the runtime-module
  assembly builds a `RuntimePluginCatalog` to derive the feature report, then
  `builtin_modules_for_config_with_runtime_plugin_and_feature_registrations` builds another catalog
  from the same registration reports to construct the bridge lifecycle state. The catalog constructor
  takes owned reports, collects both report vectors, and publishes a full initial projection, so this
  is two complete clone/projection passes for one startup selection rather than two cheap views. The
  availability helper's builtin descriptor catalog is a different data set and must not be merged
  into this candidate. Current-source review confirms that `feature_reports.rs` returns only the
  dependency report and registration references, not its catalog, before `builtin_modules.rs`
  constructs the retained bridge catalog from the same report slices. A release trace must first count both construction sites at 1/100/1000
  registrations and feature registrations; only a measured cost may justify one immutable bootstrap
  catalog or selection snapshot flowing from assembly to app and then being retained by the bridge
  lifecycle state. Its only production app caller is
  `EngineEntry::for_config_with_runtime_plugin_and_feature_registrations`, so it is classified as
  startup construction cost, never as a frame/tick hotspot.
- The single-catalog algorithm is not the target: existing scale guards cover 1/100/10,000 source
  rows and require one candidate projection/diagnostic build with each registration and feature
  registration indexed once. Preserve that linear, generation-owned index path. If measurement
  selects this candidate, remove only the cross-boundary duplicate construction while retaining the
  one-catalog generation and diagnostic semantics.
- `RuntimePluginCatalogProjection::build` already records a catalog-generation metric with
  `build_elapsed_ns`, indexed-entry count, and indexed-string bytes while it builds feature,
  bridge, registration, and dependency indexes. M1 should read those existing per-catalog metrics
  from the retained bridge catalog alongside process-level ETW data; it must not insert a separate
  per-registration timer or global counter into the measured bootstrap path. The metric explains a
  catalog construction, but cannot by itself establish end-to-end startup cost or select M2.
- `RuntimePluginCatalog::project_plan_for` also holds its project-plan cache mutex across a cold
  manifest/feature/extension build. The cache has only three target keys and current production
  callers are bootstrap, dynamic-session linking, and editor export/enablement routes. Treat it as
  an editor/startup contention candidate only: first trace cold-miss lock wait and build time. A
  single-flight, build-outside-lock redesign is prohibited unless that trace makes it the top cost.
- Native discovery has an unmeasured dev/editor structural gap. The authority caches an unchanged
  root generation and coalesces concurrent tickets, but `refresh_manifest` and `remove_path`
  discard the notified path and submit a bounded full-root refresh. Thus an unchanged lookup does
  no filesystem walk, while every accepted watcher mutation can still enumerate the full tree.
  Trace a one-path change and a watcher burst at 1/1,000/10,000 manifests before adding an
  incremental map: any repair must preserve canonical-root identity, deterministic duplicate
  diagnostics, last-good publication, depth/symlink policy, and latest-wins ticket coalescing.
  Do not add a second watcher cache or a per-editor private discovery index.
- `BridgeDiagnostics::record_enabled_call` and `record_not_enabled_call` are compiled as shared
  relaxed `AtomicU64::fetch_add` operations only with `debug_assertions`. A normal `cargo test`
  benchmark can therefore measure debug diagnostic cache-line contention rather than the release
  bridge path, where those counters compile out.
- Fixed-step diagnostic publication is an unmeasured shared-call-path candidate. The Physics
  runtime takes `Instant` and unconditionally publishes `physics.step.duration_ms` through the
  generic `CoreHandle::record_diagnostic` on every fixed step, including the missing-manager
  path. Network poll ingress also publishes up to six metrics through that same API. The generic
  path takes the global diagnostic-store mutex, converts static paths/units/tags into owned
  strings, and updates the `BTreeMap` and history; core time and render diagnostics instead use
  `record_static` to reuse already-known metadata. Preserve the Physics M6 diagnostic contract:
  M1 must only add this workload when its release ETW trace can compare the generic publication
  stack with the existing product-shaped fixed-step/poll workload. If that trace makes the mutex,
  allocator, or generic diagnostic recording the top cost, route a lower-layer handoff for an
  allocation-free static-metadata CoreHandle API that preserves the dynamic diagnostic path. Do
  not add a plugin-local cache or disable release diagnostics before that evidence exists.
- The public native-loader export currently makes a Rust DLL host appear as a product plugin
  runtime. The architecture roadmap instead defines stable host handles and replaceable plugin
  instances as the public semantics, with VM plugins as the final product boundary.

The first three rows are static facts, not throughput evidence. Existing historic benchmark values
in failure records remain diagnostic only until they are reproduced from a fresh immutable source
snapshot.

## Reference Evidence

The dominant reference is Unreal Engine, with Bevy and Fyrox used to pressure-test Rust lifecycle
and dynamic-library assumptions.

| Reference | Evidence consulted | Adopted principle |
| --- | --- | --- |
| Unreal Engine | `dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h` and `Private/Modules/ModuleManager.cpp` | Module lookup/load/unload has one manager authority. Existing loaded modules may be read concurrently, but module load is a controlled lifecycle operation; `GetOrLoadModule` explicitly rejects an unloaded module load from a non-game thread. `ModuleManager.cpp` also puts module-load work under CPU profiling scopes. |
| Unreal Engine | `dev/UnrealEngine/Engine/Source/Developer/HotReload/Public/IHotReload.h` and `Private/HotReload.cpp` | Hot reload is a separate orchestration concern. It queries a module manager's loaded status and coordinates rebinding; it is not a second module registry. |
| Bevy | `dev/bevy/crates/bevy_app/src/plugin.rs` and `app.rs` | The externally visible lifecycle is ordered `build -> ready -> finish -> cleanup`; a plugin execution path should consume an already-published lifecycle state rather than mutate registration caches on demand. |
| Fyrox | `dev/Fyrox/fyrox-impl/src/plugin/mod.rs` and `dylib.rs` | Dynamic-plugin reload is an engine-owned lifecycle: it observes a reload flag, unloads its current state, copies and loads a replacement library, then fills/registers it. This is a linkage/backend concern, not a reason to expose raw dynamic-library ownership as the product plugin contract. |

Zircon deliberately diverges from Unreal's raw module-interface pointers: all callable plugin
state remains behind Rust `Arc` generation owners and typed host handles. This preserves memory
safety and is required for reload quiescence.

## Target Architecture

The selected direction is an immutable product-generation envelope published by the runtime plugin
control plane. It is a composition boundary, not a forced merge of every subsystem's mutable
state. This is a design target, not an implementation claim.

```text
CoreRuntime / RuntimePluginCatalog (catalog/product-plan authority)
  -> PluginRuntimeGeneration (immutable product envelope; version tuple)
       -> dense PluginSlot table
       -> manifests, capability plan, diagnostics projection
       -> Arc<validated bridge binding snapshot>
       -> Arc<validated registration replay plan>
       -> Arc<PluginBackendGeneration> (native backend revision or VM generation)
            -> NativePluginBackend (internal development/ABI backend)
            -> VmPluginBackend (final product backend)

Mutation path: stage -> validate -> initialize -> atomic publish -> retire old generation
Call path: PluginSlot -> immutable generation -> dense command/method slot -> bounded host output
```

Required invariants:

1. `RuntimePluginCatalog` is the only authority that publishes a product plugin generation. The
   published envelope holds validated immutable `Arc` snapshots and a version tuple. It does not
   replace the native host's DLL/reload state, bridge/replay cache ownership, or their internal
   revision rules with a shared mutex or global revision counter.
2. `NativePluginLiveHost` becomes an internal backend adapter. It may keep ABI V3/V4 and DLL
   quiescence for development packages, but editor commands and runtime consumers must target a
   typed plugin slot/handle rather than a concrete native live host.
3. A reload builds and validates a replacement backend snapshot off the call path, then publishes
   exactly one validated product-generation tuple. It retains the previous product and backend
   snapshots through in-flight `Arc` owners; failed replacement publication leaves the previous
   public generation intact.
4. Stable command, bridge, and capability calls resolve a prevalidated dense slot from an immutable
   generation. They must not parse manifests, perform name-tree lookup, allocate diagnostic
   snapshots, or acquire a mutation mutex.
5. The V3/V4 ABI host adapter is split by responsibility before new behavior is added:
   `abi_decode`, `registration_policy`, `context_handles`, `bridge_scope`, and `ecs_registration`.
   The root adapter only wires those domains and the panic boundary.

This reuses the existing stable-handle, generation, bounded-output, and callback-lease concepts;
it does not add a compatibility facade, a second registry, raw pointers, or type-name special
cases.

## Discovery Refresh Candidate

Unreal's `IDirectoryWatcher` carries a batched `FFileChangeData` with explicit `Added`,
`Modified`, `Removed`, and `RescanRequired` actions. Its Windows implementation translates normal
`ReadDirectoryChangesW` records into path-specific actions and emits `RescanRequired` only when
the watcher cannot recover an exact delta. Zircon should keep the same distinction, while retaining
Rust `Arc` snapshot ownership rather than adopting Unreal's raw module pointers.

If release ETW selects dev discovery refresh as the top shared cost, the authority owns one
generation-scoped immutable manifest index inside its published snapshot. A refresh ticket accepts
one coalesced batch of canonical manifest-path actions: add/modify reparses only that path, remove
deletes only that path, and a watcher-overflow or invalidated root is the sole full-rescan action.
The refresh service remains the only lifecycle authority: it merges a burst into one candidate,
validates duplicate diagnostics and canonical ordering, then atomically publishes one next
generation or retains the prior snapshot. There must be no editor-local map and no second watcher
cache.

The RED gate is path-count and lifecycle based: a one-file mutation among 1,000 and 10,000
manifests reads/parses exactly one changed manifest; a coalesced burst publishes at most one
generation; removal and duplicate ordering match a cold full scan; overflow takes the explicit
full-rescan route; and failed delta parsing preserves the last-good snapshot. The matching ETW
workload compares cold full scan, one-path delta, and burst refresh from one immutable release
source identity. It must report filesystem enumeration/read attribution separately from snapshot
materialization, so removing reads is not misreported as eliminating all publication cost.

### Current-Source Discovery Update (2026-08-10)

Frameworks04 currently owns the native discovery implementation and has implemented the proposed
single authority-owned manifest index: watcher work is kept on the active/pending refresh ticket,
same-path actions use latest-wins coalescing, explicit refresh parses only the canonical manifest,
removal updates the immutable index, and invalid/overflow paths fall back to a bounded root scan.
The static regressions cover a 1,024-manifest one-path refresh, removal, lexical-root aliasing,
failed incremental parsing retaining the prior generation, and the fallback scan. Scoped Rust
1.94.1 formatting and patch checks are clean.

This is neither a current-source Cargo result nor performance evidence. The Frameworks04 source
owner is still `waiting_validation`, so Plugins01 did not transfer or modify those files. The
review also found one follow-up before the handoff can close: after a manifest that previously
emitted a collector diagnostic is repaired, a successful incremental publication must remove the
obsolete diagnostic instead of inheriting it from the old snapshot. The required regression must
assert both the repaired candidate and absence of the stale diagnostic. Release/profile ETW work
remains unstarted until a managed, source-bound validation and benchmark launch is available.

The complexity claim must be split accordingly. For a batch of `k` canonical path changes over
`N` indexed manifests, event normalization and last-action-wins coalescing target
`O(k log k)`, while manifest I/O and parsing target `O(k)`. A naive
`Arc::make_mut(BTreeMap<...>)`, whole-map clone, or full candidate/diagnostic resort still performs
`O(N)` publication work while the old generation is retained. Such an implementation may prove
that filesystem traversal disappeared, but it cannot claim an end-to-end incremental algorithm or
that the bottleneck is gone. M1 must therefore report collection and publication materialization
as separate phases. If materialization dominates after I/O removal, M2 must use bounded structural
sharing (for example, immutable pages with copy-on-write only for touched pages) and update
duplicate diagnostics only for plugin ids affected by the batch. It must not add a second mutable
cache, an unbounded per-path log, or a persistent-tree dependency before the release trace proves
that the remaining `O(N)` publication phase is material.

This follows the re-audited Unreal boundary rather than copying its storage implementation:
`IDirectoryWatcher` delivers a batch of `FFileChangeData` actions, Windows maps recoverable
`ReadDirectoryChangesW` records to Added/Modified/Removed, and emits `RescanRequired` when the
precise delta was lost. Zircon keeps that action distinction but retains its stronger immutable
snapshot, last-good publication, and cancellation contracts.

The current refresh service imposes three implementation constraints. Its state key is
`(NativePluginDiscoveryRoot, NativePluginDiscoveryRefreshInput)`, with `RootScan` and
`LoadManifest` as separate published generations; a path delta must therefore not become a third
path-bearing input, or each changed path would split the root's publication history. Keep that
selection key stable, and carry a coalesced `RefreshWork` batch only on the active/pending ticket:
the latest action per canonical path wins and an overflow/invalid-root action dominates as full
scan. `NativePluginDiscoverySnapshot` must own the one immutable canonical-path index beside its
candidate and diagnostic projections; the collector receives the prior last-good snapshot and
returns a new candidate snapshot, never mutating a published `Arc`. Finally, do not derive
assertions from the formatted `input_identity`: add structured refresh-work counters for test and
evidence output (directories enumerated, entries inspected, manifests read, manifests parsed, and
published generations), while keeping the release hot path free of per-entry clocks or an
unbounded event log.

## Benchmark Fixture Qualification

The existing ignored tests are correctness fixtures, not yet release-equivalent benchmark
executables:

| Fixture | What it establishes | Why it cannot be the accepted baseline unchanged |
| --- | --- | --- |
| `callback_lease.rs::native_callback_atomic_lease_{1,2,16,64}_thread_benchmark` | Same-plugin admission is atomic at the four thread shapes and diagnostics can be disabled. | The former multi-shape fixture was not comparable. It is now one separately filterable ignored case per shape, with managed metadata bound before warm-up and one outer interval. |
| `registration_replay.rs::native_registration_replay_{1,100,1000}_systems_{1,100}_methods_benchmark` | Registration replay builds one bridge scope and one method lookup for each generated shape. | The former cold-only nested-loop fixture was not comparable. It is now six isolated cases with a warm-up replay and source/profile metadata. |
| `runtime_behavior.rs::native_runtime_broadcast_{1,8,32}_plugin_benchmark` | Runtime command dispatch visits the sorted live-host path and reports post-run callback and loaded-lock diagnostics. | The former multi-shape fixture allocated and asserted inside its short timed loop. It is now one warmed ignored case per shape; correctness checks and JSON serialization occur after the core interval. |
| `context_registry.rs::native_host_context_lookup_{1,16}_thread_benchmark` | Stable handles avoid the writer lock at one and sixteen threads. | The former fixture timed every lookup and retained 1M values. It now has an aggregate 1M-lookup throughput interval plus a separate 8,192-sample post-measurement latency phase that reports sampling ratio and observer time. |
| `plugin::bridge` and `runtime_plugin_catalog` | The stable paths use `ArcSwap` and cached project plans. | No ignored benchmark currently supplies the required stable-call or catalog-publication workload. |

M1 must add one narrowly scoped runner that reuses these fixture constructors and behavioral
assertions but separates warm-up, core timing, bounded latency sampling, result serialization, and
ETW process identity. It must not add a second plugin runtime, nor should it change a stable-path
algorithm before a trace selects that path. The runner must collect at most a bounded sample after
the core measurement; per-iteration clocks and unbounded latency vectors are disallowed in the
measured section.

The executable contract is one shared test-only harness module with one separately filterable
ignored case per workload shape, so each managed invocation starts a fresh lib-test process. It
must not run every shape sequentially in one process or add a product benchmark binary. The named
cases cover callback-lease threads 1/2/16/64, registration replay systems 1/100/1,000 with method
counts 1/100, runtime broadcast plugins 1/8/32, and stable context lookup threads 1/16. Each case
performs a fixed warm-up, takes one outer clock around the allocation-free core loop, performs
correctness assertions after that clock, and only then runs an optional bounded latency sample.
It emits one machine-readable JSON record containing a schema id, workload id, shape, selected
Cargo profile, `cfg!(debug_assertions)`, warm-up and measured operation counts, elapsed nanoseconds,
throughput, and the existing lock/context counters. Immutable manifest identity is joined from the
coordinator receipt; the test must not invent or read a shared-worktree identity. This makes one
ETW process correspond to one workload shape and prevents report allocation, sorting, logging, or
multi-shape cache state from contaminating the core interval.

That native-host runner is not evidence for the feature-bootstrap candidate. Its release trace
must instead reuse `EntryRunner::module_selection_report_with_runtime_plugin_and_feature_registrations`
with generated 1/100/1000 registration and non-empty feature-registration inputs. Existing
feature-aware app fixtures pass an empty feature iterator, so the trace fixture must add the
missing input shape without constructing a second runtime. It records one whole-selection elapsed
time per process plus ETW stacks, never a per-row timer, and compares only runs from the same
machine, profile, manifest, and source commit.

## M4 Adapter Boundary Map

The recorded M4 baseline had a 1,161-line `host_api_adapter.rs`. Its former `context_registry`
child isolated some state but also owned dense bridge dispatch. The current source has completed
the five-domain zero-behavior split below and hard-cut both legacy aggregate files; managed
current-source validation and the Editor concrete-host boundary remain open.

| Module | Current source responsibility | Boundary rule |
| --- | --- | --- |
| `abi_decode` | `NativeHostApiAdapterError`, UTF-8/slice reads, ABI version/size checks, stage and V4 access decoding. | Returns typed validation data/errors only. It cannot access a registry, load a context handle, or invoke a foreign callback. |
| `registration_policy` | `NativeHostApiV4RegistrationPolicy`, V4 scope construction, runtime-module ownership validation, and API-table assembly. `NativeHostApiV3RegistrationScope` is retirement-only public-surface debt, not a module to preserve. | Builds host-owned V4 policy and registration scopes. It may request a context-handle registration but cannot implement registry mutation callbacks or introduce a V3 compatibility surface. |
| `context_handles` | Generational handle encoding, immutable `ArcSwap` directory, registration closeout pins, and registration-context lookup. | Keeps the allocation/retirement writer lock private. Stable lookup remains lock-free and keeps an `Arc` pin through the complete ABI call. |
| `bridge_scope` | `NativeHostBridgeCallScope`, bridge context lookup, bridge FFI entry, and the existing dense bridge method table currently co-located in `context_registry`. | Holds the library-generation owner for foreign bridge callbacks. It must not borrow a registration context or acquire the loaded-plugin mutex. |
| `ecs_registration` | System/component FFI entries, V4 system registration, `NativeDynamicAccess`, and the non-bridge host entries that share the registration-context admission boundary. Existing V3 handling moves only as needed for the Runtime06 deletion, never as a preserved API. | Is the only module allowed to turn validated ABI descriptors into `RuntimeExtensionRegistry` mutations. All extern entries retain the panic guard at their ABI boundary. |

Current-source connection review is complete, with a material scope constraint: the active native
loader calls `NativePluginHostFunctionTableV2/V3` from `native_plugin_abi.rs`, which currently
provides capability negotiation plus entry-time log and diagnostic capture. The interface's
`ZrPluginEntryFnV3/V4` and the adapter's V3/V4 registration scope construction have no production
loader consumer; their registrations are exercised by adapter tests only. The live host does use
`NativeHostBridgeCallScope` during registration replay. Therefore M4 is a zero-behavior boundary
move, not a claim that the adapter's registration API is already an active native-plugin contract.

Runtime06 is the ABI authority for the required hard cut: descriptor/entry and the current
plugin-to-host function table are V3, behavior callbacks and the runtime-interface host API are
V4, and `NativeHostApiV3RegistrationScope` is retirement-only public-surface debt. M4 must not
extend, preserve, or create compatibility re-exports for that V3 scope. Its deletion remains with
the Runtime06 owner so this plan does not absorb a foreign source lease. A later ABI-integration
slice must explicitly wire the current V3 entry contract to the selected V4 host API, define the
capability and lifecycle contract, and add an end-to-end loader test before V4 registration can
become product behavior. It must not silently substitute the test-only API for the current
host-function table.

This is a required end state, not a current-source completion claim. `native_plugin_abi.rs` still
probes the V2 descriptor symbol and dispatches `call_native_plugin_entry_v2`; V2 ABI DTOs, aliases,
fixture support, and a fallback test remain in the runtime and plugin workspace. Runtime06's own
output record classifies that production hard cut as pending. Consequently neither this M4 review
nor any profiling result may report a V3-only loader until the Runtime06 owner removes the full
V2 surface and reruns its inventory and managed validation gates.

The narrower M5-T4 behavior-table contract is already satisfied in current source. The V2 entry
path constructs `NativePluginBehavior` only through `from_abi_v2_metadata`: it records stateless
metadata but sets command, save-state, restore-state, and unload callbacks to `None`. Its callback
snapshot is also tagged `legacy_v2_metadata`, and every state/unload dispatch returns the explicit
legacy-metadata rejection before any function pointer can be invoked. This confirms that the
remaining V2 descriptor/entry fallback is metadata-only; it is not a callable V2 behavior path.
The broader Runtime06 V2 descriptor/entry hard cut above nevertheless remains pending and is not
absorbed into Plugins01.

The root `host_api_adapter` module should contain only private module declarations, narrow
re-exports needed by `native_plugin_loader::mod`, and the common FFI panic boundary. Tests move
with their owning domain; no behavior is added during this move. In particular, the bridge table
does not belong in `context_handles` merely because both use stable handles, and registration
closeout must remain testable independently of bridge-library lifetime.

#### M4 Migration Sequence

1. Introduce the five folder-backed private modules and move only typed ABI decoding/errors into
   `abi_decode`; the root remains a declaration/re-export shell and retains the shared FFI panic
   boundary. No callback table, handle allocation, or registry mutation moves in this step.
2. Move generation encoding, `ArcSwap` directory publication, pins, closeout state, and the tagged
   raw-handle routing enum to `context_handles`. Registration and bridge handles must continue to
   share the same tagged namespace: splitting their raw slot/generation allocators would let a
   cross-domain handle resolve as a valid but wrong context.
3. Move `DenseBridgeMethodTable`, `NativeHostBridgeCallScope`, bridge lookup, and bridge extern
   callback to `bridge_scope`. It retains the library-generation owner and must acquire the context
   pin before dense dispatch, without borrowing registration state or the live-host loaded lock.
4. Move V4 ownership/policy and table assembly to `registration_policy`, then move the V4
   system/component extern callbacks and registry mutations to `ecs_registration`. The Runtime06
   owner removes the V3 scope and its re-exports as one hard cut; M4 must not leave a shim or alias
   behind to make the intermediate folder move compile.
5. Split tests by those ownership domains. Preserve stale-handle, reuse/wrap, in-flight pin,
   closeout-waits-for-pins, opaque bridge slot, disabled/missing bridge, V4 authorization, and FFI
   panic cases. Run the managed current-source gate only after the full move; test relocation alone
   is not evidence that production loader entry wiring exists.

M4 design review status: complete. The adapter responsibility split is statically complete and
preserves raw-handle domain discrimination, registration closeout pinning, bridge generation
ownership, the live bridge public path, V4 API semantics, and the existing callback results. It
does not extend the V3 registration-scope export or add a compatibility path. Acceptance still
requires managed current-source validation; the split does not depend on, or authorize,
profiling-driven optimization changes.

The recorded M4 baseline is explicit: `host_api_adapter.rs` was 1,161 lines, and production Editor
source contained 42 direct `NativePluginLiveHost` / `NativePluginLoader` references across 14
files. The 2026-08-13 hard cut introduces cloneable strong/weak `NativePluginHostHandle` types,
migrates app and Editor production consumers to that typed boundary plus runtime-owned
discovery/load functions, and removes both concrete backend types from the `plugin::native`
facade. The current scan finds zero concrete backend references in app/Editor production code and
zero old `native_plugin_live_host` names in Editor source. Review caught that the first handle
version proxied only common paths and therefore narrowed the formal API. A focused RED contract
proved the gap; the typed handle now preserves all 35 public live-host methods, while runtime-owned
functions preserve all 11 loader discovery, incremental refresh/remove, and load entries without
re-exporting either concrete backend. This closes the static ownership boundary; M4 remains
unaccepted until the managed current-source compilation and behavior gates are GREEN.

## Profiling Protocol

All measurement artifacts must be written below
`E:\Git\ZirconEngine\artifacts\profiles\plugins01\<utc-timestamp>\`; no artifact may be
created under `C:`. Cargo and test binaries must remain coordinator-managed on an approved `D:`,
`E:`, or `F:` target root.

Static host identity observed at `2026-08-10T11:28:30+08:00` is AMD Ryzen 7 5800H
(8 cores/16 logical processors, reported maximum clock 3,201 MHz), Windows 11 Pro x64
`10.0.26200` build `26200`, about 39.9 GiB visible memory, and active power scheme
`85d583c5-cf2e-4197-80fd-3789a227a72c` (`Balanced`). WPR
`10.0.26100.8875` is the installed Windows system tool and WPA `11.7.383.39833` is installed under
`D:`. Reading those installed tools does not authorize a `C:` output: every trace, exported table,
JSON record, log, and report remains under the `E:` artifact root above. This snapshot is profiler
readiness metadata only because a foreign Cargo job was active. Every accepted run must record the
same fields again, plus AC/battery state and thermal/frequency stability, and reject a comparison
when the power scheme, source manifest, selected profile, or machine identity differs.

For each workload, take ten warm runs from the same immutable source manifest. Capture both a
release-profile product trace and, only when needed for counter diagnostics, a separate test/debug
trace. Every summary must print the profile and `cfg!(debug_assertions)` state; debug results may
diagnose synchronization but cannot be used as release throughput or power evidence. Capture a CPU
trace and a separate power trace only if the Windows Energy Estimation Engine provides valid data:

```powershell
$artifactRoot = 'E:\Git\ZirconEngine\artifacts\profiles\plugins01\<utc-timestamp>'
New-Item -ItemType Directory -Force $artifactRoot
wpr -start GeneralProfile -filemode
# Run exactly one coordinator-managed benchmark process from the frozen source manifest.
wpr -stop "$artifactRoot\cpu.etl"

wpr -start Power -filemode
# Repeat the same frozen workload after the CPU trace has stopped.
wpr -stop "$artifactRoot\power.etl"
```

Cargo's `test` profile inherits `dev`, while release disables `debug-assertions`; see the official
[Cargo profile reference](https://doc.rust-lang.org/cargo/reference/profiles.html). The current
validator exposes `development`, `release`, and the workspace-defined
`[profile.profiling]` as `inherits = "release"`, `debug = true`, and `strip = false`. Release remains
the authority for throughput and process-power comparisons; the symbolized CPU-stack attribution
run must use the managed `profiling` profile from the same immutable source and workload shape.
The validator must therefore carry all three allow-listed identities through compatibility,
command construction, and artifact publication: development uses no profile flag and `debug`,
release uses exactly one `--release` and `release`, and profiling uses exactly one
`--profile profiling` and `profiling`. Raw `cargo` is not an acceptable bypass, and an undocumented
environment override is not durable benchmark provenance.

The two profile-contract gaps found by Plugins01 are now repaired in current source. Omitting the
CLI parameter keeps compatibility JSON, every compiling builder, and artifact publication on the
development/debug default. `profiling` is allow-listed through compatibility, build/test,
export/profile-contract builders, and artifact publication as exactly `--profile profiling` plus
the `profiling` directory. Release-only traces still cannot substitute for symbolized function-level
attribution. Positive CLI dry-run acceptance remains pending. The artifact-governance rejection
cleared; a foreign `zircon_editor --profile profiling --no-default-features` compile was active from
2026-08-10 13:50 +08 and ended naturally, but the next minimal default DryRun still received
`cargo_cpu_lane_reserved` for Runtime08 reservation `5227c713f4b04f129dc1cac7a3055c2e`.
No Cargo process was started by Plugins01. Raw Cargo, parallel validation, and FIFO bypass remain
prohibited.

Benchmark-validity prerequisite: the ignored callback-lease benchmark takes one elapsed time
around its 1M operations and is suitable for a release throughput measurement. The host-context
workload now separates aggregate throughput from sparse latency observation: its 1M lookup
interval has no per-lookup timing or growing allocation, and the later latency sample is capped at
8,192 observations. It reports the sampling ratio and observer elapsed through
`BenchmarkMeasurement`; these modes must not be compared as one number. The static harness
contract guards the split. This removes the measurement-harness defect, but supplies neither a
current-source dynamic result nor authorization to change the registry algorithm before ETW
identifies it as dominant. The managed validator still requires `-IgnoredTests` with a narrow
`-TestFilter`.

The stable-bridge regression has a different limitation. It samples only the first 2,048 calls per
thread with per-call `Instant` timing, then folds those samples into aggregate throughput while its
serialized control has no matching observer. This keeps the structural lock-free regression useful
and makes its throughput comparison conservative, but its p95/p99 are neither a warm steady-state
latency distribution nor a fair control comparison. M1 must run the release throughput workload
without per-call sampling, then collect a separately warmed, explicitly sampled latency workload
whose observer cost is reported. ETW stack attribution remains the authority for selecting a bridge
optimization; no test-print latency percentile is sufficient on its own.

The ETL analysis must filter to the benchmark process and report CPU sample attribution, context
switches, synchronization waits, committed/working-set deltas, and call-stack share for the
following workloads:

| Workload | Current test/fixture basis | Required scale |
| --- | --- | --- |
| Stable bridge call | bridge import stable-call regression | 1 and 16 threads, 1M calls per thread |
| Native callback lease | `tests/callback_lease.rs` ignored benchmark | 1/2/16/64 threads, 1M total calls per shape |
| Host context lookup | `context_registry.rs` ignored benchmark only after its aggregate-throughput/sparse-latency split | 1 and 16 threads, 1M lookups |
| Dev discovery refresh | discovery authority generation regressions | one changed manifest and watcher burst at 1/1,000/10,000 manifests |
| Catalog publication | project-plan/catalog batch regression | 1/100/10k registrations/features/modules |
| Reload lifetime | live-host rollback/replacement tests | 100 replacements with in-flight call, rollback, and unload-failure branches |

For every workload, record median, p95, p99, throughput, peak working set, allocation/copy counts
from local counters, generation-publish count, and ETW top-stack percentage. A result is rejected
when its source manifest differs from the corresponding implementation commit.

Power may be reported only as hardware- and scenario-specific ETW energy data. There is no valid
claim that Zircon is "close to another engine" without the same machine, power plan, GPU workload,
plugin fixture, warm-up policy, and measurement interval. If the machine cannot attribute energy
to the process, the report must say `energy_attribution_unsupported`, not infer watts from CPU time.

The immutable source identity for M1 is the coordinator materialized validation-copy
`inputManifestHash`, joined with that copy's terminal evidence. `validate-matrix` currently retains
only a Cargo job id and target directory after acquire/start; it does not pass that identity to its
child process. Before a benchmark can claim to print a source manifest, its owner must use a
coordinator-issued identity contract: either inject the materialized identity at launch or join the
benchmark output to its job/copy receipt in the final report. The benchmark must never substitute
`git rev-parse`, a shared-worktree hash, or a newly invented manifest for the managed copy identity.

Coordinator01 now has an open canonical failure for this launch-contract gap and has added a named,
allow-listed benchmark template that derives the two child-only values from a materialized cargo
copy. Its first source review nevertheless found a P1 identity-domain mismatch: the executor passes
the full cargo-copy `inputManifestHash` to `MilestoneService.bind_validation`, whose stale gate
still expects the milestone's scoped manifest hash. A Cargo closure contains more than the milestone
paths, so a real request can reject as `validation_copy_manifest_stale` before Cargo starts. The
repair must preserve the scoped hash for the milestone stale gate while separately persisting the
full immutable copy hash, profile, case, root PID, and terminal evidence for benchmark/ETW identity.
The same review found that the synchronous `WorkspaceCopyService.run` path still inherits
`ZR_BENCHMARK_*` values while the new asynchronous start path removes them; both ordinary paths
must share the same sanitization before this contract is accepted.
Plugins01 must not set `ZR_BENCHMARK_SOURCE_MANIFEST` or `ZR_BENCHMARK_CARGO_PROFILE` in a shell,
consume the foreign copy, or start a run until Coordinator01 proves that dual-identity contract.

The 2026-08-10 coordinator public job record also exposes a separate terminal-recovery defect:
external job `932da703b7c7451997118ffbf57d517a` remains `running` with `exit_code: null`, but has
`live_process_pids: []` and a populated `process_tree_exited_at` equal to its recorded start time.
Its `source_copy_job_id` is null, so it cannot establish M1 provenance. This stale foreign job
continues to hold FIFO admission; Plugins01 must not finish, release, retry, or otherwise mutate it.
Any independently observed Cargo process cannot be attributed to that record and is not Plugins01
compile or benchmark evidence. Coordinator terminal recovery must resolve the record before M1
requests a legitimate managed launch.

## Review And Tooling Evidence (2026-08-10)

The current hot-reload rollback implementation received two independent read-only reviews. The
final recheck found `C0/I0/M0`: retained-generation callback admission is reopened only after a
successful restore, failed publication unloads the replacement and retains cleanup diagnostics,
and the bridge-publication recovery drops its live-host lock before plugin callbacks. Current-source
format verification covered `lifecycle.rs`, `tests.rs`, `tests/runtime_behavior.rs`, and
`tests/hot_reload_publication.rs`; the scoped `git diff --check` was also clean apart from the
repository's existing CRLF conversion notices. This is correctness evidence only, not a performance
measurement or a completed plugin milestone.

The M0 profile boundary was repaired under exact coordinator leases. The validator now accepts
`development`, `release`, and `profiling`; puts the selected profile in the compatibility document;
maps release to exactly one `--release`; maps profiling to exactly one `--profile profiling`; keeps
clean profile-free; and publishes from `debug`, `release`, or `profiling`. The omitted-parameter
test exercises the development identity, flag-free builders, and debug publication without passing
`-CargoProfile`, then proves those outputs equal an explicit `development` selection across the
same compatibility, build/test/export/profile-contract, and publication boundaries. Public validator documentation records the three modes and reserves profiling for
symbolized CPU/ETW attribution. A post-review RED exposed that `zircon_plugins/Cargo.toml` lacked
the selected custom profile even though the validator could render it for that subworkspace. The
plugin workspace now defines the same `inherits = "release"`, `debug = true`, `strip = false`
contract as the root workspace, and a two-workspace contract changed from `0 passed / 1 failed` to
`1 passed / 0 failed`. The refreshed profile-bound pure Pester batch is `19 passed / 0 failed`; it
covers compatibility, both workspace manifests, every command builder, hashing/publication,
unknown-profile early rejection, and documentation. PowerShell parsing is `2/2`, and scoped
`git diff --check` passes. After the nested-workspace command repair, validator SHA-256 is
`7ADD8DAF3B0CF9B4A42D579393AEA69B280E4CD0D64D432EC9528CF4713A7009`; test SHA-256 is
`7A9AED3C394B16692BC0871A10F109D1BFE48D40F5B3E20D8F9B74273A66834B`; plugin-workspace manifest
SHA-256 is `AEE2DF65DEEBCA512E12E469E0A771316C5F1FDA79438FA158F21FF544B13264`.

Fresh M0 acceptance selected the five enclosing Pester groups for compatibility identity, workspace
profile parity, publication, invalid-profile rejection, and real CLI dry-run parsing. It completed
`17 passed / 0 failed` in 152.74 seconds. The positive CLI cases obtained managed drive-root lanes
and rendered development, profiling, and explicit `zircon_plugins/Cargo.toml` commands. The latter
now changes Cargo's working directory to `zircon_plugins` and invokes `Cargo.toml` without duplicating
the workspace prefix. This closes the prior coordinator-admission evidence gap. The profile repair's
independent review remains `C0/I0/M0`; the Frameworks06 failure was returned before any benchmark
baseline, ETW trace, energy measurement, or optimization was started.

M1 now has a shared test-only harness at
`native_plugin_loader/benchmark_harness.rs`. Callback lease (1/2/16/64 threads), registration
replay (1/100/1,000 systems x 1/100 methods), runtime broadcast (1/8/32 plugins), and context
lookup (1/16 threads) each have a separately filterable ignored case, so every managed invocation
starts a fresh lib-test process. Before any fixture allocation or timing, each case binds a
64-digit coordinator source-manifest hash, an explicit `release` or `profiling` marker, and
`cfg!(debug_assertions)`; development/debug runs fail before timing. Core paths use one outer
interval, perform no per-iteration timing or result serialization, and assert afterwards. Context
lookup additionally takes an independent, warmed, bounded 8,192-sample latency pass only after
the aggregate interval, reporting p50/p95/p99, sample ratio, and observer time. The static
contract was RED before the harness existed and is now `11 passed / 0 failed` in the current pure
Python contract suite. Pre-measurement review added three RED-to-GREEN guards: every worker must
reach a readiness barrier before the outer interval begins; threaded throughput must stop its outer
interval after every core loop completes but before worker joins; and context lookup must report
the measured writer-acquisition delta rather than the fixture's cumulative counter. The
shared atomic completion gate uses one pre-created token per worker and owner-thread `park/unpark`,
so token drop wakes the owner even during unwinding without allocating in the measured interval;
join teardown remains outside that interval. The existing guards also prove that post-sample sorting
is charged to the latency observer interval and that nearest-rank p95/p99 tail reporting cannot
silently exclude the slowest sample. Benchmark JSON schema `zircon.native.benchmark/2` now declares
`latency_percentile_algorithm=nearest_rank` for both sampled and null-sample records, preventing
reports calculated by different percentile rules from being compared as one series. Scoped Rust
1.94.1 `rustfmt --check` and explicit trailing-whitespace checks pass. A current-source managed Cargo compile and
profile-bound ignored execution remain pending on the Coordinator01 source-copy-bound benchmark
identity failure: a named template now exists, but its scoped/full manifest binding must be repaired
and accepted before it can launch this harness. No
throughput number from this harness is accepted yet.

### Open Failure implementation review

All 17 open Plugins01 failure records were re-read against current source. Sixteen have their named
production mechanism present and remain open for current-source managed validation, performance or
product evidence, upward return, or commit rather than for a newly reproduced implementation gap:
bridge snapshot, host-context page directory, load-report projection, callback global-lock removal,
per-call callback lease/admission, registration replay generation, conservative access plan, package validation projection, compiled
catalog project plan, availability generation, ref-counted plugin ids, bounded event drain, plugin
workspace lock materialization, typed live-key hot reload, discovery compile boundary, and
per-World callback factories are all still represented in production source. This count is not an
acceptance claim; several records retain explicit dynamic or broad evidence requirements.

The one reproduced production algorithm gap in the original static audit was
`failure-2026-07-17-native-plugin-discovery-recursive-rescan.md`: its then-current
`discover/authority.rs::refresh_manifest` and `remove_path` discarded notification paths and forced
a `RootScan`. Frameworks04 now owns the corresponding current-source repair, which uses one
authority-owned manifest index with latest-action-wins path batches, single-manifest
add/modify/remove, last-good publication, and full rescan only for overflow or invalidated roots.
Plugins01 did not edit that active ownership boundary. The repair still requires its owner to add
the stale-diagnostic regression identified above and to obtain managed current-source validation.

### Native Discovery End-to-End Complexity Correction (2026-08-11)

The current path-scoped collector work is necessary but does not yet make a watcher mutation
end-to-end incremental. `NativePluginDiscoverySnapshot::from_incremental_payload` clones the
entire `NativePluginDiscoveryManifestIndex`; the index's `project()` then walks every path,
rebuilds the duplicate-id selection map, and clones every selected candidate. Finally,
`NativePluginDiscoveryAuthority::report_from_snapshot` calls `to_vec()` for both candidates and
diagnostics on every warm `discover()` projection. Therefore a batch of `k` changed manifests can
perform O(k) file I/O and parsing while still performing O(N) publication work and O(N) report
materialization for N indexed manifests. A subtree removal additionally uses `BTreeMap::retain`,
which is O(N) by construction.

This is not a license to add a second cache or to replace the public load report with shared
mutable state. Unreal's `FModuleManager::FindModulePaths` populates one manager-owned
`ModulePathsCache` before later name lookups, and Bevy's `AssetServer` separates read-only info
queries from write-side publication. The corresponding Zircon direction, if measurements select
this cost, is one discovery authority with two explicit boundaries: an immutable generation view
for status/read-only consumers, and one owned `NativePluginLoadReport` materialization only for
an actual dynamic-load/export consumer. Any structural-sharing index must retain canonical-path
ordering, duplicate diagnostics, last-good publication, and latest-wins batches; `Arc::make_mut`
over the whole `BTreeMap` alone is insufficient because it retains the O(N) copy.

The managed release/profiling study must record separate counters and CPU stacks for: manifest
enumeration/stat/read/parse, index clone/update, candidate/diagnostic projection, and report
materialization. It compares cold scan, 1-path change, path burst, subtree removal, and unchanged
status lookup at 1, 1,000, and 10,000 manifests from one immutable manifest. M2 may select a
structural-sharing publication change only when these traces show publication or report
materialization dominates; otherwise the authority remains unchanged and the measured lower-layer
cost is routed to its owner.

### Runtime Profile Availability Architecture Recheck (2026-08-10)

The availability failure's current source already has the appropriate single-generation ownership
model. `assembly_presets.rs` constructs only the selected profile for `for_id`; availability
projection indexes descriptors and provider membership once; and `generation.rs` provides compact
category/runtime-id indexes while materializing owned reason strings only at the export or
diagnostic boundary. The existing 1/100/1,000 contracts assert linear membership and selection
steps, first-selection ordering, required-OR merging, row sharing for `missing_required`, and
serialized-report parity. The startup assembly test asserts one availability projection.

The reference review supports retaining that boundary rather than adding another cache:
Unreal `ModuleManager.cpp::GetOrLoadModule` allows concurrent reads only for a previously loaded
module and keeps loading under its controlled game-thread lifecycle; Bevy's `Plugin` contract
orders `build`, `ready`, `finish`, and `cleanup`; Fyrox's `DynamicPlugin` reload path makes
reload and registration one engine-owned operation. Zircon deliberately diverges by retaining
Rust-owned immutable availability rows and typed runtime IDs rather than raw module pointers.
The correct follow-up is a source-bound bootstrap measurement proving construction remains outside
the frame path, not a second global or editor-owned availability cache. Current source has no
newly reproduced structural defect in this failure; its fresh focused/broad managed validation
and product startup trace remain pending.

## Implementation Gates

| Gate | Status | Exit evidence |
| --- | --- | --- |
| Static ownership and reference audit | `complete` | This record; current code and Unreal/Bevy/Fyrox sources were read. |
| M4 native adapter and typed upper boundary | `static_complete / managed_dynamic_pending` | `host_api_adapter` is split into the five planned private domains; the root is a declaration/re-export shell. App and Editor production references to `NativePluginLiveHost` / `NativePluginLoader` are zero. `NativePluginHostHandle` projects all 35 live-host methods, runtime-owned functions project all 11 loader methods, and the boundary contract discovers both backend method sets and verifies exact delegation. Static contracts are `13/13`; the plan's 25 named acceptance entries are present `25/25`; scoped Rust `1.94.1` formatting and diff checks are clean. This is not managed compile or behavioral acceptance. |
| Windows profiler discovery | `complete` | `wpr.exe` 10.0.26100.8875 and `wpa.exe` 11.7.383.0 are available; WPR lists CPU and Power profiles. No trace was started and this is not timing or energy evidence. |
| Current-source benchmark executable | `harness_implemented / static_contract_12_green / rust_focused_pending` | The shared test-only harness emits source/profile/debug markers and schema `/2` nearest-rank percentile metadata for every isolated workload shape; dynamic metadata and counter keys now use a JSON string boundary so emitted records remain parseable. Every workload binds metadata before fixture creation or timing, a readiness barrier excludes worker startup, an allocation-free atomic completion gate stops threaded timing before worker join teardown, and context lookup reports writer acquisitions as a measured-interval delta while separating aggregate throughput from bounded post-measurement latency sampling. A released managed release/profiling execution is still required. |
| Immutable source identity binding | `pending_managed_contract` | Benchmark output is joined to the materialized validation-copy `inputManifestHash` and terminal evidence; no shared-worktree fallback is permitted. |
| Managed benchmark-profile support | `m0_accepted / failure_returned` | Omitted development, explicit release, and symbol-preserving profiling are consistent across compatibility, builders, root/plugin workspace manifests, publication, and docs. Pure focused contracts are `19/19`; fresh managed CLI acceptance is `17/17`; independent review is `C0/I0/M0`. |
| Baseline ETW traces | `pending` | `cpu.etl`, optional `power.etl`, source manifest, and ten-run summary under `E:`. |
| TDD design tests | `pending` | RED tests for one-generation publish, old-generation lifetime, no stable mutation lock, and backend-independence. |
| Measured M2 lowest-layer optimization | `pending` | Select and implement at most one measured top cost. `PluginRuntimeGeneration` remains conditional on cross-view publication being the measured cost; completed M4 adapter decomposition is tracked separately above and does not authorize an unmeasured M2 optimization. |
| Post-change profile | `pending` | Same workload, machine, power plan, and manifest discipline; top bottleneck is either removed or explicitly re-routed. |
| Failure return and milestone commit | `pending` | Focused + upward gates, independent review, `failure return`, managed commit SHA. |
| WeCom result | `pending` | Concise validated metric summary sent only after the commit and immutable evidence exist. |

## Dependency-Ordered Execution

The architecture must advance from shared measurement support toward plugin orchestration. A
later row may not substitute for evidence required by an earlier row.

| Milestone | Owner and scope | Testing stage and promotion gate |
| --- | --- | --- |
| M0 | `Frameworks06`: provide one explicit allow-listed `CargoProfile` contract for `development`, `release`, and the workspace's symbol-preserving `profiling` profile, defaulting to existing development behavior. The selected profile must participate in coordinator compatibility/reuse identity and every Cargo command builder that compiles code, including export/profile-contract paths. Artifact publication must select `debug`, `release`, or `profiling` from the same identity; target-wide `cargo clean` remains profile-free. | Focused PowerShell contracts prove the omitted parameter and explicit development both remain flag-free and use `debug` across compatibility, every builder, and publication; release adds exactly one `--release`; profiling adds exactly one `--profile profiling`; all three lanes are distinct; invalid profiles fail before Cargo; and each publication resolves its matching directory. Return the cross-plan failure before Plugins01 reserves a baseline. |
| M1 | Plugins01: add a minimal benchmark harness only after M0. It must print workload shape, source manifest, selected profile, and `cfg!(debug_assertions)` before timing. Reuse existing ignored fixture contracts rather than inventing a second plugin runtime, while separating core timing from clocks, unbounded latency storage, and output serialization. | Run release throughput and power cases plus a separately managed profiling CPU-stack case per workload shape from the same immutable source, then repeat each accepted measurement ten times. Reject a run with a missing profile marker, nonzero exit, changed manifest, invalid process energy attribution, missing CPU symbols, or per-iteration timing/allocation in the measured section. |
| M2 | Plugins01: choose only the measured top shared cost. Candidate changes are one catalog-generation publication boundary, bridge slot compaction, a context-handle lookup repair, changed-manifest discovery refresh, or a lower-layer static-diagnostic publication API; the static audit does not select among them. | Start with RED ownership/lifetime/concurrency tests for the selected boundary. The testing stage covers lower-layer tests, normal live-host paths, failed publication/rollback, and the original benchmark workload. |
| M3 | Plugins01: if and only if M2 identifies cross-view publication as the cost, consolidate native bridge, replay, and catalog snapshots into `PluginRuntimeGeneration`; preserve `Arc` retirement and callback admission invariants. | Validate all generation transition failure paths and rerun the matched profile/ETW workload. A lower layer failure returns through its owner before editor or product paths are changed. |
| M4 | Plugins01: split the native host adapter into the five named responsibility modules while keeping native loading an internal development/ABI backend. | Compile and behavioral validation must show no public/editor consumer relies on a concrete native live host. Re-run ABI V3/V4, context-handle, registration, bridge, and lifecycle tests before closing the architecture milestone. |

## Next Slice

### 2026-08-11 architecture and harness review

- The current replay generation remains the correct lowest shared authority: it owns the parsed
  registration inputs, dense bridge slots, validated access plan, and generation-owned bridge call
  scope. Each registered system retains only those compact values. No new cache, global callback
  owner, or cross-generation publication layer is justified before measurement.
- Reference review used Unreal's `ModuleManager` load/unload authority, Bevy's explicit
  `build`/`finish`/`cleanup` plugin lifecycle, and Godot's versioned C extension initialization
  and deinitialization levels. Zircon deliberately differs by retaining an `Arc`-owned native
  generation through in-flight callbacks and rollback; that divergence is required by the native
  ABI lifetime contract and must not be optimized away.
- `python tools/tests/test_plugins_01_native_benchmark_harness_contract.py` passes `12/12`.
  This confirms only source binding, release/profiling markers, worker readiness before timed
  start, worker completion before timed teardown, allocation-free completion signaling,
  interval-delta counters, bounded post-interval latency observation, and JSON encoding. It is not
  a release measurement, CPU stack, ETW trace, power result, or implementation-selection result.
- M1 therefore remains measurement-only: acquire a coordinator-owned current-source validation
  job, run each listed workload ten times under one immutable manifest, and require matched ETW
  CPU stacks plus process energy attribution before choosing exactly one M2 repair.

M0 is accepted and the M1 harness is static-complete. The next slice is one fresh managed
current-source compile, then release throughput/power and profiling CPU-stack runs for each
isolated workload shape. Bind all runs to one coordinator materialized source manifest, repeat
accepted measurements ten times, and choose exactly one implementation slice from the measured
top stack. Until those measurements exist, this plan remains
`m0_accepted_benchmark_harness_static_complete_validation_pending`.
