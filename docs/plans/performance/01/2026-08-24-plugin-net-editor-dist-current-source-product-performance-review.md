---
title: Plugin Net Editor and Dist Current-Source Product Performance Review
date: 2026-08-24
status: static_complete_dynamic_pending
scope:
  - zircon_plugins/net/editor
  - zircon_plugins/net/dist
canonical_owners:
  - docs/plans/optimize/zircon_plugins/10-first-party-network-source-runtime-editor-dist-catalog-transport-rpc-replication-product-integration-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_editor/26-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Settings/LevelEditorPlaySettings.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Settings/LevelEditorPlayNetworkEmulationSettings.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/NetworkPredictionInsights/Source/NetworkPredictionInsights/Private/NetworkPredictionTraceModule.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/NetworkPredictionInsights/Source/NetworkPredictionInsights/Private/NetworkPredictionProvider.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/NetworkPredictionInsights/Source/NetworkPredictionInsights/Private/UI/SNPWindow.cpp
---

# Plugin Net Editor and Dist Current-Source Product Performance Review

## 1. Status and frozen scope

The Net Editor and NativeDynamic distribution projection completed E3 current-source static review over **7/7 Rust files** at revision `080fefe6acd449beded4497dee4a474b9e1f7383`:

| Module folder | Files | Physical / non-empty lines | Bytes | Tests / ignored | Final fingerprint |
|---|---:|---:|---:|---:|---|
| `zircon_plugins/net/editor` | 6/6 | 421 / 397 | 16,898 | 1 / 0 | `daf6d2cdd0b7e03f53c63cecff28aaf1555968109aeca09ba4c4f218728d61c3` |
| `zircon_plugins/net/dist` | 1/1 | 98 / 86 | 3,574 | 2 / 0 | `3f2f7fbbeb4e4d89da1b49a8b590637a41c38ea35c3a4682d09814a68e4cbd8f` |

Fingerprints hash sorted repository-relative path plus file bytes. Six of seven files pass standalone `rustfmt --check --edition 2021 --config skip_children=true`; the sole mismatch is existing import ordering in `editor/src/authoring.rs`, not a parse or behavior defect. Both folders are source-clean and pass `git diff --check`.

Managed Windows Cargo is unavailable in the current validation lane, so the three Rust tests were read but not executed. No current-source Editor/native product is launchable and WPR/RenderDoc tools are unavailable. Dynamic startup, UI, ABI, topology, trace, scale and power acceptance therefore remain pending. RenderDoc will apply only when a network workload changes a rendered Editor/game surface; it cannot establish CPU network scheduling cost.

## 2. Per-file review result

| Module | Reviewed files | Result |
|---|---|---|
| Editor declaration and registration | `lib.rs`, `plugin.rs`, `capability.rs` | Registers metadata only; it supplies no document, operation factory, diagnostic producer or lifecycle owner. |
| Editor authoring vocabulary | `authoring.rs` | Declares 2 views, 1 drawer/template, 6 commands, 3 inspectors, 1 asset/toolkit/template and a two-node graph palette. All five referenced source resources are absent. |
| Editor tests | `tests/mod.rs`, `tests/authoring_extensions.rs` | One test proves descriptors and payload-schema strings appear in a registry; it does not resolve resources or execute commands. |
| NativeDynamic projection | `dist/src/lib.rs` | Exports package/runtime registration metadata. Command/event manifests, state, invocation, save/restore, unload and host-ready bridge behavior are empty. |

The common `register_authoring_contribution_batch` implementation was followed across the package boundary. It iterates descriptors into `EditorExtensionRegistry`; it does not synthesize handlers, documents, UI resources or data sources. Exact repository searches found no implementation outside this package for `net.authoring`, `net.diagnostics` or the six Net operation IDs. The first-party Editor catalog routes only Navigation and Neural, so Net Editor is not selected by the ordinary product path.

## 3. Structural performance findings

### P0: the declared Editor product is unreachable and non-executable

`plugins://net/editor/authoring.zui`, the three inspector ZUI documents and `replication_schema.default.toml` do not exist. The six commands have no factory/handler, typed transaction or terminal receipt. Adding empty files or no-op handlers would only make registration tests greener while preserving a false product.

This is a performance blocker because no real authoring workload, save/compile path, diagnostics surface or multiplayer topology exists to profile. Startup time, main-thread cost, update cadence, retained memory and power cannot be compared with another engine from inert descriptors.

### P0: diagnostics has a view name but no observation pipeline

`net.diagnostics` has no consumer for Runtime Net diagnostics or trace events. There is no session/world/connection generation, sample timestamp, freshness, queue high-water, byte budget or dirty revision. The correct repair is not a new per-frame snapshot loop: Editor25 already owns the shared observation session, trace store, provider cadence and observer-effect controls.

Network observation must publish typed deltas from the canonical runtime network instance into Editor25. A visible view may query a bounded window only when provider revision or viewport range changes; a hidden view must perform zero presentation polling. Work per refresh must be `O(changed rows + visible samples)`, retained data `O(configured byte/time budget)`, and stale-generation publication zero.

### P0: no multiplayer test topology exists to make measurements representative

Net Editor does not own a Dedicated/Listen server plus 1..N client launch request, per-process ports/accounts/worlds, ready/join barriers, link emulation or teardown receipts. Single-process loopback timings would conceal process, serialization, scheduling, queue and transport costs and cannot be presented as engine-level evidence.

Editor26 and Editor07 must provide the topology. Capture identity must include BuildSet, role, process, world/session, connection generation, workload and effective network policy so WPR/ETW and trace evidence can be correlated rather than merged by display string.

### P1: NativeDynamic is a metadata shell, not a parity target

The dist entry exposes registration metadata with state schema 0 and empty command/event manifests. It has no invocation, state migration, unload or host-ready callback. A native build can therefore load a descriptor while providing none of the source product lifecycle. Benchmarking that shell would measure ABI entry overhead, not Net functionality.

Native/source parity must be generated from the same provider graph and verified for resources, command/event schemas, state, lifecycle and observation before any distribution performance claim.

### P2: descriptor construction is not the current bottleneck

The Editor builds small temporary vectors and strings during one registration pass. There is no evidence that this path repeats per frame or dominates startup. Replacing those allocations before the product path exists would optimize the wrong layer. Instrument registration count/wall/allocation after catalog closure; change it only if a current-source startup trace shows material cost.

## 4. Unreal evidence and adopted policy

Unreal is the primary structural reference:

- `LevelEditorPlaySettings.h:93-100,374-402,486-506` models standalone/listen/client roles, optional separate server, single/multi-process execution, client count, server port, fixed server/client rates and network emulation as one real launch configuration.
- `LevelEditorPlayNetworkEmulationSettings.h:17-68,103-106` gives packet policy explicit server/client targets and availability conditions instead of treating a UI toggle as proof of emulation.
- `NetworkPredictionTraceModule.cpp:15-20` installs a trace provider and analyzer into an analysis session.
- `NetworkPredictionProvider.h:32-49,59-65` records simulation tick, net receive/commit, fault, buffered input, user state, reconcile and configuration against trace/session identity.
- `SNPWindow.cpp:197-209` reads the analysis provider and rebuilds filtered presentation only when the provider data counter changes. This is the relevant anti-polling pattern for Zircon, not unconditional UI-frame snapshots.

Zircon should retain its descriptor vocabulary but connect it to durable documents, executable operations, a shared trace provider and a real test topology. It should not copy Unreal's implementation language or UI, and it must compare measurements only on the same machine, build class and declared workload.

## 5. Required optimization sequence

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Product truth | Keep Net Editor unavailable while resources/factories are missing; expose a typed reason. | Ordinary catalog reports unavailable instead of registering broken views; no empty placeholder acceptance. |
| M1 Catalog and resources | Generated runtime/editor/native provider closure and five versioned, resolvable product resources. | All five URIs resolve, compile and hash identically in source/export/native forms. |
| M2 Authoring lifecycle | Listener/route/replication documents plus six typed operations, undo/save/recovery/compiler and terminal receipts. | Create/edit/validate/compile/save/reopen/cancel tests execute real behavior with zero unowned mutation. |
| M3 Observation | Editor25 `NetworkObservationProvider` consumes bounded, generation-qualified runtime deltas. | Hidden idle polling is zero; refresh is revision-driven; retained samples/bytes and refresh work obey configured bounds. |
| M4 Multiplayer topology | Editor07/26 launches Dedicated/Listen plus 1..N clients with link policy and deterministic teardown. | Ready/join/stop receipts identify every process/world/connection; zero leaked process, task, socket or stale generation. |
| M5 Distribution parity | Source/export/native consume the same generated provider graph and lifecycle contract. | Golden capability/resource/schema/state/lifecycle/trace parity; descriptor-only native shells fail qualification. |
| M6 Dynamic acceptance | Current-source WPR/ETW, runtime trace and conditional RenderDoc captures over declared workloads. | Main-thread network wait is zero; no per-frame full snapshot; queues/bytes/tasks are bounded; P50/P95/P99, RSS, wakeups and joules/work unit are published with BuildSet and workload. |

Static file review is complete. Product and dynamic acceptance are pending, so these folders must not be promoted into protected `review.md`, and this record does not warrant a Git milestone commit or quantified WeCom notification.
