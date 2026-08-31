---
title: Editor Workbench document tabs committed frame authority performance review
date: 2026-08-22
module: zircon_editor/src/ui/workbench/document_tabs
priority: MVP-P1
status: source_reviewed_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate
---

# Goal

Keep document-tab sizing a constant-time policy function, but remove repeated title measurement and
parallel geometry reconstruction from projection, pointer and drag/drop consumers. One committed tab
layout must own visible frames, close frames and insertion boundaries for a model/font/layout
generation.

This record is deliberately module-scoped. It does not claim that document-tab projection, text
measurement or drag/drop is accepted until current-source dynamic evidence exists.

## Reviewed source

Current-source fingerprint:

- folder: `zircon_editor/src/ui/workbench/document_tabs`
- Rust files: 2/2
- source lines: 87
- bytes: 3,345
- joined UTF-8 SHA256: `12a13a5041eca51f0db00658774deb360efe7fd873230814b5c1863f109a3c3d`
- `metrics.rs`: `35f67ff93aae56a91ecad6b01a9d7a288a64be3c2ac667f93f6bd2af8b0785d4`
- `mod.rs`: `6f6ad5d0da2a71c6818217823130e7513f309ea3edfe5e6b4630450b84c58a7f`
- owning commit before this review: `08094b9b9e17f6c80372e15c17b01204038b305b`

Both files were read in full. The relevant production call chain was also read:

- `chrome_template_projection/dock_header.rs`
- `chrome_template_projection/dock_header/side.rs`
- `retained_host/tab_drag/tab_width.rs`
- `retained_host/tab_drag/strip_hitbox.rs`
- `retained_host/document_tab_pointer/{constants,helper}.rs`
- `retained_host/app/workspace_docking/drag_drop/route.rs`

## Current result

### Source module

`document_tab_preferred_width_from_title_width` is `O(1)`, allocation-free, clamps negative and
non-finite input, selects closeable/non-closeable minimums, and caps the result. `document_tab_close_x`
is also `O(1)` and allocation-free. Four local tests cover typography role, readable closeable width,
finite/min/max policy and close-frame arithmetic. No direct source edit is justified here.

`rustfmt --edition 2021 --check` passes for 2/2 files. This is static source acceptance only.

### P1 structural finding: tab geometry has several producers

The policy owner is shared, but resolved geometry is not:

1. `fallback_dock_header_nodes` measures every title, computes every width and allocates projected
   nodes whenever that fallback is rebuilt.
2. Side headers have a bounded projection cache, but the main/bottom/floating fallback path does not
   expose the same committed layout artifact.
3. `strip_hitbox` clones tab identity/title/host into a temporary vector and measures every surviving
   title again while resolving a precise drop.
4. Document pointer routing consumes shared constants and later measured frames, but it does not
   consume one immutable layout result also used by projection and drop insertion.

The local algorithms are linear where a tab strip must be laid out, but the same `O(T)` work can be
paid by multiple owners. A stable strip with unchanged titles/font/layout should pay zero text
measurements and zero tab-node rebuilds. This must be measured before implementation; it is not yet a
proven frame-time bottleneck.

### P2 bounded work

Precise insertion resolution currently runs on drop routing rather than every pointer move, so its
repeated measurement is lower priority than steady-state chrome reconstruction. Tab counts are also
human-scale. Do not introduce a complex cache into `metrics.rs`; solve ownership at the committed
layout/projection boundary.

## Unreal source basis

Primary reference files were read directly under `dev/UnrealEngine`:

- `Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabWell.cpp`
- `Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp`
- `Engine/Source/Runtime/SlateCore/Public/Widgets/SWidget.h`

Relevant behavior:

- `SDockingTabWell::OnArrangeChildren` owns tab ordering and computes one child-size policy before its
  linear arrangement loop (`100-155`). Paint calls `ArrangeChildren` and paints those arranged
  children rather than reconstructing a second tab-width model.
- `SDockingTabWell::ComputeDesiredSize` reads each child's `GetDesiredSize` (`268-295`).
- `SWidget::CacheDesiredSize` computes and stores desired size (`973-980`), while `GetDesiredSize`
  returns that stored value (`726-728`). The public contract explicitly says descendants are computed
  and cached before parents (`SWidget.h:755-774`).
- `SDockingTabWell::BringTabToFront` avoids the foreground broadcast when old and new tabs are equal
  (`666-707`).

The applicable pattern is not Unreal's exact pixel constants. It is one tab-well owner, cached child
desired sizes and event-driven invalidation. Zircon should preserve its own Runtime Text shaping and
componentized Workbench contract while adopting that ownership model.

## Target architecture

Introduce an immutable `DocumentTabStripLayout` (name may follow the existing local type vocabulary)
owned by the Workbench chrome/layout generation. It contains stable tab identity plus visible tab,
label, close and insertion-boundary frames. Its key must include:

- committed document-tab topology/content revision;
- available strip size and scale;
- Runtime Text metrics generation;
- style/density generation affecting chrome.

Projection, pointer routing and tab-drop insertion must consume the same artifact. Only a key change
may run title measurement and the linear arrangement pass. `metrics.rs` remains the pure sizing-policy
owner and must not acquire model, font, renderer, pointer or global cache responsibility.

Complexity target:

- invalidated strip layout: `O(T + glyph_measurement)` once per generation;
- unchanged projection: `O(1)` cache hit, zero title measures, zero tab-node rebuilds;
- pointer/drop lookup: consume committed frames; no title clone or title measurement;
- memory: one bounded current layout per live tab surface, no unbounded title-key cache.

## Instrumentation and acceptance

Add counters/timings at the actual owners before changing the architecture:

| Counter | Required interpretation |
| --- | --- |
| document-tab layout builds | one per invalidated generation, zero during unchanged idle frames |
| document-tab title measures | exactly `T` on invalidation, zero on unchanged projection and drop |
| document-tab projected node builds | zero when model/layout/font generations are unchanged |
| document-tab frame consumers | projection, pointer and drop report the same layout generation |
| allocations/bytes | zero incremental layout allocations during unchanged idle projection |

Scenario matrix on one current-source fingerprint:

1. cold editor launch with the default MVP layout;
2. 1, 8 and 32 document tabs at 640x420, 900x620 and 1260x780;
3. 300 unchanged frames;
4. activate, close and reorder one tab;
5. resize the window through the three widths;
6. change text scale/font generation once.

Acceptance requires counter proofs plus Windows WPR/ETW CPU and allocation samples before/after on the
same build profile and scenario. Report median and p95 main-thread cost, measurement/build counts,
allocated bytes, process CPU time and energy estimate. No guessed millisecond or power target is
allowed. RenderDoc is not a CPU/tab-layout profiler; use it only after a launchable current-source
renderer exists to confirm that the single-authority cutover preserves draw count/pixel output and
does not add GPU work.

All traces, builds and temporary outputs must be placed on `D:`, `E:` or `F:`. Repository-owned final
evidence may be kept under `docs/tests/editor`.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add layout-build/title-measure/node-build counters and capture baseline. | Current-source WPR plus counter artifact |
| M1 | Define the immutable committed strip-layout artifact and invalidation key. | Unit tests for key changes and unchanged hits |
| M2 | Make chrome projection, pointer and drop consume the artifact; remove duplicate measurement/clones. | Source ownership scan and focused behavior tests |
| M3 | Run the scenario matrix and compare baseline/optimized CPU, allocation and energy data. | Counts meet the table; median/p95 reported |
| M4 | Real-window visual/interaction QA; RenderDoc only for renderer parity when applicable. | Screenshots, pointer/drop tests and optional capture |

## Validation state

- Full source review: passed, 2/2 files.
- Static formatting: passed, 2/2 files.
- Existing local unit coverage identified: 4 tests.
- Managed Cargo execution: pending. The shared coordinator had active Cargo lanes after startup
  validation request `2a2414d3921442df829708286bc36267` failed to reconcile; no test result is
  inferred from that environment condition.
- Dynamic CPU/allocation/power evidence: pending.
- Real-window and renderer parity: pending.

The folder stays in `docs/plans/performance/pending.md` until M0-M4 pass on one current-source
fingerprint. It must not enter `review.md` based only on the pure-helper review.
