---
title: Plugin Runtime Diagnostics Current-Source Product Performance Review
date: 2026-08-24
status: static_complete_product_unavailable_dynamic_pending
scope:
  - zircon_plugins/runtime_diagnostics
canonical_owners:
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Developer/TraceInsights/Public/Insights/IUnrealInsightsModule.h
  - dev/UnrealEngine/Engine/Source/Developer/TraceInsights/Private/Insights/InsightsManager.cpp
  - dev/UnrealEngine/Engine/Source/Developer/TraceServices/Public/TraceServices/Model/AnalysisSession.h
  - dev/UnrealEngine/Engine/Source/Developer/TraceServices/Private/AnalysisService.cpp
  - dev/UnrealEngine/Engine/Source/Developer/TraceServices/Public/TraceServices/ModuleService.h
  - dev/UnrealEngine/Engine/Source/Developer/TraceServices/Private/ModuleService.cpp
---

# Plugin Runtime Diagnostics Current-Source Product Performance Review

## 1. Status and frozen scope

The Runtime Diagnostics plugin package completed E3 current-source static review over **6/6 Rust files** at revision `0a5f22c944d802b0677ebeee5fc3168361bbac5c`:

| Module folder | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| `zircon_plugins/runtime_diagnostics/editor` | 5/5 | 270 / 248 | 10,061 | 2 / 0 | included below |
| `zircon_plugins/runtime_diagnostics/dist` | 1/1 | 102 / 90 | 3,958 | 2 / 0 | included below |
| **Total** | **6/6** | **372 / 338** | **14,019** | **4 / 0** | `5c294fc43ba181c7061beb491aa54aea265a4d419214b2d868f6aa2698acda9c` |

The fingerprint is SHA-256 over sorted `repository-relative-path|sha256(file-bytes)` rows joined by LF. All six files pass `rustfmt --check --edition 2021 --config skip_children=true`; the package passes `git diff --check` and has no current worktree change. The package contains only these Rust files, two Cargo manifests and generated `plugin.toml`. It contains no `.zui` resource.

Managed Windows Cargo remains unavailable, so none of the four tests ran. There is no current-source executable carrying this plugin for WPR/ETW, and no rendered diagnostics workload for RenderDoc. RenderDoc would only qualify pixels/draw/GPU behavior after the real Editor pane consumes a real provider; it cannot establish this package's CPU observation contract.

The package is therefore statically reviewed but **not an available diagnostics product**. No source optimization was applied in this review.

## 2. Per-file review result

| Module | Reviewed files | Result |
|---|---|---|
| Declaration and IDs | `editor/src/{capability,extension_ids,lib}.rs` | Declares one experimental Editor capability, three packaging forms, a view, drawer and template ID. It declares no diagnostic source, stream, snapshot, query or lifetime contract. |
| Editor registration | `editor/src/plugin.rs` | Registers `editor.runtime_diagnostics` and a drawer/menu/command against `plugins://runtime_diagnostics/editor/authoring.zui`; the URI does not resolve anywhere in the package. |
| Editor tests | `editor/src/tests.rs` | Registers into a fresh empty registry and checks strings/manifests. It never combines with builtin Editor contributions, resolves the URI, reads diagnostics or exercises unload. |
| Native dist | `dist/src/lib.rs` | Editor-only, stateless ABI descriptor with empty command/event manifests, no bridge methods, no host-ready callback, no state and no unload callback. Its behavior string says views remain hosted by the Editor module. |

The real builtin product is already owned by `zircon_editor`: `runtime_diagnostics_view_descriptor.rs:10-21` registers the same `editor.runtime_diagnostics` ID and resolves `res://ui/editor/host/runtime_diagnostics_body.zui`; `template_documents.rs:39-40,152-153` and the asset catalog publish that resource. The plugin's standalone test bypasses this owner because `EditorPluginRegistrationReport::from_plugin` uses an empty registry. The registry itself rejects repeated view IDs through `register_view` and `insert_unique`.

## 3. Structural performance findings

### P1: a duplicate product authority makes activation fail or become order-dependent

The builtin Editor and optional plugin both claim `editor.runtime_diagnostics`. The plugin also synthesizes a drawer, menu item and `view.editor.runtime_diagnostics.open`, while the builtin command catalog already targets the same view. A real combined admission cannot preserve both authorities; a loader that fails late has already paid package discovery, manifest parsing, ABI load/materialization and rollback costs.

Editor25 must remain the only Runtime Diagnostics view owner. The plugin may contribute a typed provider/source lease to that product, or it must be removed from selectable profiles. It must never re-register the pane, drawer or product command.

### P1: every advertised packaging form points at a missing required resource

`plugin.rs:45-59` registers `plugins://runtime_diagnostics/editor/authoring.zui`, but the nine-file package has no such asset. Source-template, library-embed and native-dynamic availability are therefore false. The current tests only prove that the unresolved string was stored.

Do not add a placeholder ZUI. First select the unique product owner. If this package becomes a provider adapter, remove its view/template contribution. If it retains any private resource, source/static/native package graphs must resolve the same content hash before capability publication.

### P1: the plugin has no diagnostics data path

There is no Runtime entry, diagnostics producer, snapshot bridge, command/event callback, source identity, clock, generation, freshness, sampling cadence, backpressure, privacy policy or terminal receipt. The native descriptor explicitly exports zero bridge methods and no host-ready/unload behavior. Loading it cannot change the data shown in the builtin pane.

The provider boundary must be explicit: `DiagnosticProviderLease { provider_id, source_id, activation_generation, schema_version, clock_domain, cadence, budget, privacy, health }`. Publication is atomic with capability admission; unload revokes the lease, fences stale callbacks and releases buffers/tasks. A descriptor row or successful ABI entry is not provider readiness.

### P1: current tests erase the integration failures that matter

The two Editor tests prove local descriptor contents in a new registry. The two dist tests prove ABI strings and zero bridge methods. There is no RED test for duplicate builtin view admission, required resource resolution, catalog/profile selection, source/static/native parity, provider snapshot flow, backpressure, reload or unload cleanup.

M0 tests must materialize the builtin Editor registry first and assert that the current package cannot be advertised as available. Later tests must activate a real provider and prove one diagnostics pane, one source-qualified stream and complete lease cleanup across source/static/native forms.

### P1: the declaration shell is not the recurring hot path

This 14,019-byte Rust scope has no poll/tick/query loop and no production diagnostics algorithm to micro-optimize. The measurable recurring costs live in the Editor25 observation pipeline and Runtime03 recorder/store: collection cadence, cross-process snapshots, cardinality, copying, synchronization, query caching and presentation invalidation. Treating this package as a profiler would direct optimization at startup declarations while leaving the actual observer overhead untouched.

This package should expose provider activation/health counters only. Editor25 owns immutable generation snapshots/deltas and demand-driven presentation; Runtime03 owns bounded collection and export. WPR/ETW qualification must attribute observer CPU, allocation, wakeups and IPC bytes by provider/source rather than to the view descriptor.

## 4. Unreal evidence and adopted policy

Unreal's current source supports the responsibility boundary rather than a parallel-view plugin design:

- `TraceServices/Private/AnalysisService.cpp:318-359` creates one analysis session, installs typed providers/analyzers under an edit scope, invokes enabled modules, then starts analysis. Optional modules extend the shared session rather than declaring a second Insights product.
- `TraceServices/Private/ModuleService.cpp:106-114` calls `OnAnalysisBegin(Session)` for enabled modules. `AnalysisSession.h:126-175` separates analyzer/provider registration, typed read/edit access and RAII read/edit scopes.
- `TraceInsights/Private/Insights/InsightsManager.cpp:465-528` owns stop/reset/session-change notification. Lines 300-368 and 1013-1019 read providers under `FAnalysisSessionReadScope`; UI does not call producer internals directly.
- `IUnrealInsightsModule.h:266-356` exposes the current analysis session and a central major-tab configuration registry. It gives extensions a controlled integration point without allowing multiple owners for the same product surface.

Zircon should adopt the same separation of concerns, not copy Unreal types or Slate: Runtime producers -> source-qualified observation session -> typed provider leases -> immutable bounded snapshots/query cache -> one Editor25 presentation owner. Provider work runs at explicit cadence/budget; the UI reads a committed generation and never performs producer FFI, locking or collection during layout/paint.

## 5. Required optimization sequence

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Product truth and authority | Keep the plugin unavailable; add combined builtin/plugin admission and resource-resolution RED tests; choose Editor25 as the single pane owner. | Exactly one `editor.runtime_diagnostics` authority; missing resource cannot publish capability; no placeholder UI or alternate ID. |
| M1 Provider lease contract | Replace view/drawer/template contribution with typed source-qualified provider registration, generation fencing, cadence/budget/health and unload revocation. | Plugin activation yields a provider receipt, not a second view; stale callbacks and buffers disappear after unload/reload. |
| M2 Packaging and catalog parity | Carry executable provider callbacks and required resources consistently across source template, library embed and native dynamic; profile selection closes dependencies. | All forms expose the same provider schema/content hashes and fail closed when callbacks/resources are absent. |
| M3 Observation session and presentation | Runtime03 producers publish bounded snapshots/deltas; Editor25 owns session/source selection, query cache and demand-driven invalidation. | UI layout/paint performs no collection/FFI; generation/freshness/loss/health are visible and bounded. |
| M4 Dynamic qualification | Launch a current-source Editor plus local/child Runtime, activate/reload/unload providers and run idle, busy, disconnect and flood workloads. | Publish BuildSet-bound P50/P95/P99 snapshot/query/presentation latency, observer CPU, RSS/allocation, IPC bytes, wakeups and joules; no duplicate pane, stale source or unbounded queue remains. |

## 6. Direct-fix decision and dynamic status

Adding the missing file, renaming the view or wiring another local collector would be simple code but the wrong structural optimization. Each action would preserve a second diagnostics product or couple presentation to a producer before the provider/session authority exists. This review therefore makes no source change and routes implementation to Editor25, Runtime03, Editor50 and Plugins01/08.

Static review is complete. Cargo, real catalog activation, provider data flow, reload/unload, WPR/ETW and power acceptance remain pending. No Git milestone commit or quantified WeCom notification is warranted.
