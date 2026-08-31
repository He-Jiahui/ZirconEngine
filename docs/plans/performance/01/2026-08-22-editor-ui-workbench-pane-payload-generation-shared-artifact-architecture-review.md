---
title: Editor workbench pane payload generation and shared artifact performance review
date: 2026-08-22
module: zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders
priority: MVP-P0/P1 editor presentation path
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate docking and TraceInsights timing profiler
---

# Goal

Make each visible workbench pane consume one immutable, generation-owned projection instead of
rebuilding a wide owned payload whenever the retained host recomputes. Expensive profiling analysis
must run once per changed capture generation on a cancelable worker. Hierarchical and tabular panes
must materialize only visible rows. Hidden and unchanged panes must perform no projection work.

## Reviewed source

- folder: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders`
- Rust files: 12/12
- lines: 849
- bytes: 32,479
- joined current source-bytes SHA256:
  `41ce3526107b8ea22b96a064982499a9035a654c029ea39afd0f431358615ad1`
- joined pre-change source-bytes SHA256:
  `2845baa704066a87ee7fcf044ff6c2d98df08e4578f4e34188e9f03799ec7d5c`
- owning commit before review: `bee4c707b714738346b49bba15c59468b8bd9b39`

| File | Lines | SHA256 |
| --- | ---: | --- |
| `animation_graph.rs` | 21 | `5a7ee29c7b9ba8a0378152ece3988af21712dbf196fb577627cf30b80dbd8540` |
| `animation_sequence.rs` | 22 | `defde03881d5d61f41468edaff71e21b1103b2186f4e9418f4adba8c39a95655` |
| `build_export.rs` | 32 | `e2c8708efd92bf5fbaf95fd2bfe51afcfedb144ea975b8d4e4d7630473bcd99c` |
| `component_showcase.rs` | 8 | `5b0b7aaf52ef7f14acd5461e88f844ee883b7e7d3767975b3cf46028f5c695be` |
| `console.rs` | 13 | `a1d0bca70abb17a3322455ac4f36cea50bb7a4c24ab53dd5887ccf642004d263` |
| `generated_bottom.rs` | 8 | `26b483f2b526b9baa1a3c75765ff823cfa445857e3423a128016e23ac8387e61` |
| `hierarchy.rs` | 18 | `5ee4df93eb5ec5debf3345b292288145f36fc74ffe53c2f0ca1a877c0dade296` |
| `inspector.rs` | 115 | `626142b8090118f19b166668c3758044191b6b1d122743c23efbb2c3cb353611` |
| `mod.rs` | 45 | `43168459e65346495fe948db46a948058ef0bce25efa0ac94cd438f6d8870895` |
| `module_plugins.rs` | 50 | `d30682057bba3978ac64c8448b276d4af875f6d6937aacf2555b096b796ebb2c` |
| `performance_timeline.rs` | 186 | `d176f0364af7199092e1b225a3710005ead7718c5f69af08e7db6e5921aab835` |
| `runtime_diagnostics.rs` | 331 | `61092bddc7429f428e1378a7001439fc366a1f6766a0f97cbd80a7478c92a1b2` |

All 12 files were read in full. The review also followed payload visibility, retained-host pane
collection, `ModelRc`, scene entry ownership, module/plugin and build/export projection caches,
runtime profile snapshot ownership, and hotspot analysis. This is an architecture review of the
current source, not a claim based on file-local clone counting alone.

## Existing foundations to retain

`should_collect_runtime_diagnostics` and `should_collect_payload_for_kind` prevent hidden Runtime
Diagnostics, Performance Timeline, Module/Plugins and Build/Export panes from collecting their
sources. Console data already shares text, level and jump-sequence owners. Module/plugin and
build/export upstream caches already reuse stable source identities. These are correct demand gates.

They are undermined after collection because the builder boundary converts shared projections into
new wide owned payloads. Visibility alone does not make repeated visible-pane work acceptable.

## Findings by pane family

| Scope | Current behavior | Structural consequence |
| --- | --- | --- |
| `runtime_diagnostics.rs` | Deep-clones the complete `RuntimeDiagnosticsSnapshot` before reading it. | Frame, span, counter, render, physics and animation storage is copied even though the builder only formats a bounded diagnostic view. |
| `performance_timeline.rs` | Deep-clones the same snapshot and synchronously calls `analyze_hotspots` for every payload build. | S spans repeatedly clone four string keys, group in a hash map, retain duration vectors and frame sets, sort each group, then sort all groups on the UI caller. |
| `module_plugins.rs` | Clones the pane owner, then `row_data` clones every wide row before every field is converted to a new `String`. | Stable plugin generations still pay a row-wide intermediate copy and 26 final string copies per plugin. |
| `build_export.rs` | `row_data` clones every target before 10 text fields are copied again. | Stable build/export generations still rebuild the entire owned table. |
| `hierarchy.rs` | Clones every scene display name and queries ordered selection membership for every row. | An already shared scene generation is flattened into all owned rows; its generation is discarded, so unchanged refresh cannot be O(1). |
| `inspector.rs` | Rebuilds the full nested inspector/component/property payload and string markers. | Selection-stable frames have no shared typed property artifact and no visible-row budget. |
| animation builders | Clone the complete pane presentation and move fields into another owned payload. | A stable animation generation cannot be reused at the pane boundary. |
| `mod.rs` TemplateV2 | Clones the complete values map and component patch vector. | Template presentation remains proportional to all values and patches per build. |
| simple generated/showcase panes | Small constant payloads. | Keep simple; do not introduce a cache until measurements show a reason. |

For P profile spans partitioned into G hotspot groups, current hotspot work is at least
O(P + sum(m_i log m_i) + G log G) plus O(P) cloned key and duration storage. It is invoked by UI
payload frequency rather than profile generation. This is the highest-risk algorithm in this
folder because P can grow independently of visible row count, while the final pane displays at most
12 hotspot rows.

## M1 result

The low-risk source change preserves payload schemas and ordering while removing intermediate
ownership work:

- Runtime Diagnostics and Performance Timeline borrow an existing runtime snapshot. The normal
  `Some` path no longer deep-clones the snapshot and does not construct an empty default snapshot.
  This removes one full snapshot clone from each payload build; Performance Timeline still performs
  the structurally incorrect synchronous hotspot analysis.
- Module/Plugins borrows the pane owner and iterates borrowed rows. Per build this removes one pane
  owner clone and one 26-field `ModulePluginStatusViewData` clone per plugin before final payload
  string construction.
- Build/Export iterates borrowed rows. Per build this removes one 11-field
  `BuildExportTargetViewData` clone per target before final payload string construction.

The final owned payload strings remain. M1 is therefore an allocation/refcount stopgap, not the
generation-owned or virtualized target architecture.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp`
- `dev/UnrealEngine/Engine/Source/Developer/TraceInsights/Private/Insights/TimingProfiler/Widgets/STimersView.cpp`
- `dev/UnrealEngine/Engine/Source/Developer/TraceInsights/Private/Insights/TimingProfiler/Widgets/STimerTreeView.cpp`
- `dev/UnrealEngine/Engine/Source/Developer/TraceInsights/Private/Insights/TimingProfiler/ViewModels/StatsAggregator.h`
- `dev/UnrealEngine/Engine/Source/Developer/TraceInsights/Private/Insights/TimingProfiler/ViewModels/StatsAggregator.cpp`
- `dev/UnrealEngine/Engine/Source/Developer/TraceInsights/Private/Insights/TimingProfiler/ViewModels/TimerAggregator.cpp`

Unreal's tab invocation first finds or reuses the live `SDockTab`; spawning occurs only when no live
or restorable tab exists (`TabManager.cpp:1711-1727`, `1766-1823`). The transferable invariant is
that a visible tab is a persistent presentation owner, not a request to rebuild all tab data.

TraceInsights binds timer data to `STreeView` through `TreeItemsSource`, `OnGetChildren` and
`OnGenerateRow` (`STimersView.cpp:503-509`; `STimerTreeView.cpp:189-195`). It does not prebuild a
flat widget row for every timer. Timer aggregation is an `FAsyncTask` with an explicit worker and
cancellation token (`StatsAggregator.h:28-49`, `StatsAggregator.cpp:117-184`). Completion publishes
results through one callback; a replacement request cancels or ignores the old generation.

The timer tree also throttles source-list checks according to timer count and services aggregation
completion from `Tick` (`STimersView.cpp:2752-2768`). Filtering/group updates explicitly request one
tree refresh and record their wall time (`1597-1725`). The relevant standard is persistent model,
cancelable generation work, bounded update cadence and on-demand rows. It is not blind background
threading of every UI conversion.

## Target architecture

1. Give each pane source a typed `{source_id, generation}` receipt and publish one immutable
   `Arc<PaneProjectionArtifact>` per changed generation. The retained host and native windows share
   that artifact; presentation refreshes compare receipts and do no work when unchanged.
2. Keep payload types typed and shared through the retained layer. Convert only visible cells to
   host strings. Delete parallel wide DTOs after all consumers move in one hard cutover.
3. Move hotspot aggregation to the runtime profiling product boundary. A changed profile generation
   schedules one cancelable task on the shared scheduler; hidden diagnostics schedules none. Publish
   a bounded immutable report with source generation and analysis generation.
4. Make hierarchy, inspector, module/plugin and build/export tables persistent indexed models.
   Sorting/filtering creates shared index vectors; visible rows V materialize on demand. Selection
   and expansion update narrow state, not all row payloads.
5. Preserve the existing visibility gate, but base it on a single coalesced visible-pane demand
   index. A pane becoming hidden cancels pending optional analysis and releases only its consumer.
6. Keep simple constant panes simple. Generation machinery belongs at wide or asynchronous source
   boundaries, not around eight-line static builders.

Complexity targets:

- hidden pane: zero source snapshot, projection, analysis and row materialization;
- unchanged visible generation: O(1) receipt comparison, zero payload/string rebuild;
- changed source generation: one O(N) typed projection shared by all consumers;
- profiling analysis: one cancelable O(P + sorting) worker per accepted profile generation;
- display: O(V) row/cell materialization and O(1) indexed row/selection access;
- retained/native consumers: shared owner clones only, no second full table conversion.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| payload builds by kind/source generation | unchanged generation = 0 |
| snapshot/row/string clone bytes | hidden and unchanged = 0 |
| hotspot requests/start/cancel/finish/stale publish | at most one accepted result per profile generation |
| hotspot worker/main-thread wall | aggregation main-thread wall approximately 0; publish bounded |
| materialized rows versus source rows | O(V), not O(N) |
| source/projection retained bytes and owners | one immutable artifact per live generation |
| presentation/full/native rebuild counts | pane change patches only its segment once |
| queue depth/age and cancellation latency | bounded, visible, and deadline-controlled |

Matrix: pane kinds; hidden/visible/floating/native; source rows 0/1/100/10,000; profile spans
0/12/1,000/100,000; groups 1/100/10,000; stable refreshes 1/1,000; update rates 1/30/120 Hz;
windows 1/16; rapid tab switch, reload, cancel and close. Report median/p95/max CPU, main-thread wall,
allocations/clone bytes, RSS, input-to-paint latency, worker utilization and package energy on one
source/executable fingerprint.

Use the current editor profiler and WPR/ETW with all targets and artifacts on D/E/F. RenderDoc is
only an acceptance tool for a current-source launchable GPU pane/viewport pixel and draw-call
comparison. It cannot prove removal of CPU payload clones or hotspot work.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add per-kind generation/build/row/string/analysis counters and capture a source-bound baseline. | current-source WPR and allocator evidence on D/E/F |
| M1 | Borrow runtime snapshots and table rows; remove provable intermediate owner/row clones without changing payload schemas. | RED-to-GREEN source contracts, focused Cargo and behavior parity |
| M2 | Publish generation-owned shared pane artifacts and patch retained/native consumers by receipt. | stable visible generation build/clone = 0 |
| M3 | Move profiling hotspot analysis to cancelable shared-scheduler generation work. | UI-thread aggregation wall approximately 0; stale publish = 0 |
| M4 | Virtualize hierarchy, inspector, plugin and build/export rows and delete flat compatibility DTO paths. | materialized rows proportional to V |
| M5 | Run F0/F4 interaction, WPR/power and RenderDoc visual acceptance. | quantified before/after and product parity |

## Validation state

- Full folder source review: passed, 12/12 files.
- Related visibility, source ownership, cache, profiling algorithm and Unreal reference code: read.
- M1 source implementation: complete. Its RED-to-GREEN static performance contract is 3/3;
  full-folder `rustfmt --check` and scoped `git diff --check` pass.
- M0 and M2-M5 implementation and dynamic performance acceptance: pending.
- Managed Cargo is pending while shared Cargo/rustc lanes are active.

The folder remains in `pending.md` until M0-M5 pass on one fingerprint. M1 allocation reductions
must not be recorded in `review.md` as end-to-end performance acceptance.
