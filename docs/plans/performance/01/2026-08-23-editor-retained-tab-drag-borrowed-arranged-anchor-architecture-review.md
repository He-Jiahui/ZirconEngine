---
title: Editor retained tab drag borrowed arranged anchor performance review
date: 2026-08-23
module: zircon_editor/src/ui/retained_host/tab_drag.rs and tab_drag
priority: MVP-P0 editor tab drop release
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine Slate docking tab well
---

# Goal

Make final tab-drop insertion consume borrowed current-model identity first, then converge onto the
same committed arranged tab frames used by paint and native pointer routing. Remove release-frame
container construction and whole-row clones immediately. Do not add a local geometry cache or a
second text-width algorithm; the existing Workbench committed-frame owner must become the single
authority in the later structural cutover.

## Reviewed source

- owner Rust files: 9/9 (`tab_drag.rs` plus all 8 files under `tab_drag`)
- physical lines before M0: 1,484
- bytes before M0: 48,391
- joined current-file SHA256 manifest: `be0fe56f37abc21020a04d7853918e0cd1e0f6b10f703a48a0ba2f4cede02be2`
- owning commit at review: `08094b9b9e17f6c80372e15c17b01204038b305b`
- owner Rust files after M0: 9/9
- physical lines after M0: 1,525
- bytes after M0: 49,762
- joined post-M0 current-file SHA256 manifest:
  `95722f1291eb6b118d18d663d036b135e70fb4bd8ec20e4fb6c4b19c1db81a5c`

All owner files were read in full. The review also traced app release routing, shell-pointer typed
receipts, committed shell state, Workbench document model construction, chrome projection caches,
scene projection, actual `HostChromeTabData` frame generation, native tab routing and relevant
retained tab-drag tests. The 2026-07-17 static review remains directionally correct; this record
updates it with the current componentized-frame and host-generation architecture.

## Structural findings

### P0: one drop builds and clones two O(T) temporary collections

`strip_hit_box` materializes a `Vec<StripTabEntry>` for every target tab. Each row clones the view
instance, title and host; document hosts also clone the page/window id and workspace path. Precise
resolution then consumes that vector and allocates a second vector solely to remove the dragged
instance. Only one selected row is needed in the result.

The current algorithm is O(T), not quadratic, and it runs on final drop rather than every pointer
move. Its avoidable cost is nevertheless concentrated on the UI release frame: two O(T)
allocations, T identity/title/host clones, and T workspace-path clones for a document strip. M0
must retain ordering and midpoint behavior while iterating borrowed rows and materializing only the
selected result.

### P1: drop insertion reconstructs geometry instead of consuming arranged frames

After cloning rows, the resolver measures each surviving title until it reaches the pointer. This
reconstructs widths using `estimate_dock_tab_width` or `estimate_document_tab_width`, while the
committed host scene already owns actual `HostChromeTabData.frame` values produced from authored or
fallback template layout. Projection/paint and native pointer routing therefore have arranged
frames, but drop insertion uses a parallel title-based model.

This is both repeated O(K) Runtime Text measurement and a correctness risk when authored layout,
font generation, scale, clipping or fallback geometry differs from the estimator. The fix belongs
to the existing committed document-tab frame authority plan, not a cache inside `tab_drag`.

### P1: host lookup separately scans layout ownership

`drop_host_for_tab` scans active drawers, main-page document trees, exclusive pages and floating
workspaces to recover the dragged tab's current host. Document-path helpers are depth-first and
allocate only the selected path, but a release may still visit O(layout nodes) before target-strip
resolution performs its own O(T) work. M2 needs a generation-owned instance-to-host index shared
with the committed Workbench model. A new per-drop hash map would only move the allocation.

### P2: current target enumeration has intentional compatibility behavior

For a left or right group, one visible expanded stack enables the strip and the current resolver
then enumerates tabs from both slots. M0 must preserve that ordering and visibility behavior. Any
semantic change requires real-window drag tests and belongs with the single-frame-authority
cutover, not the allocation cleanup.

## Zircon and Unreal source basis

Direct Zircon evidence:

- `HostChromeTabData` stores typed `TabData` with its actual tab and close frames.
- `scene_projection.rs` creates document `tab_frames` from the same projected header nodes and tab
  model that paint/native routing consume.
- `projection_cache.rs` retains document and side tab models across identical source generations.
- native document-tab routing iterates those retained frames, while `strip_hitbox.rs` bypasses them
  and measures titles again.
- Workbench document model collection already carries `WorkspaceTarget` and `workspace_path`; these
  can be borrowed and cloned once only when a result is selected.

Direct Unreal source under `dev/UnrealEngine`:

- `SDockingTabWell::OnArrangeChildren` retains actual tab widgets and performs one ordered
  arrangement pass (`SDockingTabWell.cpp:100-160`).
- `ComputeDraggedTabOffset` and `ComputeChildDropIndex` use the tab well's current geometry and
  retained dragged-tab state (`346-410`).
- `OnDragOver` updates the retained drag offset, and `OnDrop` computes one insertion index and opens
  the tab through the owning stack (`545-603`).

The transferable rule is one tab-well owner with retained typed children and current arranged
geometry. Zircon should keep its own variable-width Runtime Text policy, but projection, pointer and
drop must consume one committed arrangement rather than independently measuring titles.

## Target architecture

1. M0 keeps current geometry behavior but makes strip rows borrowed references. It performs no
   target-row vector allocation and clones only the selected host/path/identity.
2. M1 exposes a generation-stable arranged tab-strip artifact from the Workbench host scene. Drop
   insertion consumes its tab frames/boundaries, so drop performs zero title measurements.
3. M2 adds a generation-owned instance-to-host index beside the committed Workbench model. Drag
   start/release host lookup becomes O(1) without a per-event map build.
4. M3 hard-cuts the production title-width reconstruction only after all authored/fallback,
   document/drawer/floating and scale consumers use the same artifact.

No `tab_drag`-local unbounded cache, duplicate glyph algorithm, compatibility facade or legacy
geometry path is allowed.

## Instrumentation and acceptance

Matrix: target tabs `1/8/32/100/1000`; workspace depth `0/4/16`; target
`left/right/bottom/document/floating`; result `before/after/gap/end/no-target/same-host`; geometry
`authored/fallback/scaled/clipped`; input `single release/125 Hz drag/500 Hz storm`.

Acceptance requires:

- M0 target-row vector allocations per release: `2 -> 0`;
- M0 whole-row identity/title/host clones: `T -> 0`;
- selected result remains one host/path materialization and one target-id clone;
- M1 drop title measurements: `K -> 0`, consuming the committed frame generation;
- M2 current-host layout visits: O(layout nodes) -> O(1) index lookup;
- exact insertion parity for before/after/gap/end and dragged-id filtering;
- WPR/ETW reports release p50/p95, main-thread CPU samples, allocations/bytes, queue depth and
  process energy estimate on the same current-source executable before/after;
- optional RenderDoc is used only to prove draw/pixel parity after a real frame-authority cutover,
  not as a CPU/drop profiler.

All executables, Cargo targets, traces, allocator logs, power evidence and RenderDoc captures must
remain on D:, E: or F:. No guessed timing or power comparison is accepted.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Borrow target rows; remove both temporary vectors and whole-row clones. | focused RED/GREEN, Rustfmt, focused behavior tests when Cargo is available |
| M1 | Consume committed arranged tab frames and insertion boundaries. | zero drop title measures; authored/fallback/scale parity |
| M2 | Add committed generation-owned instance-to-host index. | O(1) lookup counters and layout-depth matrix |
| M3 | WPR/allocation/power and real-window acceptance; RenderDoc only if rendering changes. | quantified same-build before/after evidence |

## Validation state

- Owner review: complete, 9/9 current Rust files.
- Related model/projection/native-routing/tests and Unreal tab-well sources: read and mapped.
- Architecture report: recorded before implementation.
- M0 implementation: applied after this report was created. `TabStripHitBox` now owns geometry
  only; left/right/bottom/document target rows are borrowed iterators; the dragged row is skipped
  before width measurement; only the selected host/path/identity is materialized. Existing
  left/right dual-slot order and before/after/gap/end behavior were preserved.
- Exact source-operation delta: temporary target-row vectors per release `2 -> 0`; whole-row
  identity/title/host clones `T -> 0`; document workspace-path clones `T -> 1 selected path`;
  selected target-id clones remain one. Runtime Text measurements remain O(K) and are explicitly
  pending M1 rather than claimed solved.
- Focused static contract:
  `tools/tests/test_editor_retained_tab_drag_borrowed_arranged_anchor_performance_contract.py`, 81
  lines, 2,903 bytes, SHA256
  `55ba8641b8c3ab911346fae051ad6907d132d88b86513777089bfc8161b40110`; RED 0/6 to GREEN 6/6.
- Retained-host performance contracts: GREEN 82/82. Broad current-worktree performance contracts:
  GREEN 261/261. Rustfmt for all 9 owner files and scoped `git diff --check` passed.
- Three focused Rust behavior tests were added for gap, strip-end and dragged-only filtering. They
  have not executed. The managed Cargo retry did not enter compilation because current
  `CODEX_SESSION_ID` `019ffe1c-46d5-7933-97cb-65996b76f552` is archived and the coordinator rejects
  creating or renewing Cargo work for that owner. Raw Cargo and a fabricated session identity are
  not accepted bypasses.
- WPR, allocation, power, real-window and RenderDoc evidence: pending.
- The owner remains in `pending.md`; it must not enter `review.md` on static evidence alone.
