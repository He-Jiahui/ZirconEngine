---
title: Editor floating-window projection generation receipt and spatial scale review
date: 2026-08-23
module: zircon_editor/src/ui/retained_host/floating_window_projection.rs
priority: MVP-P1 editor shell layout and tab drag
status: source_reviewed_m0_instrumentation_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate persistent window invalidation roots
---

# Goal

Publish one exact floating-window geometry receipt shared by shell layout, native-window sync,
pointer routing, main presentation and native presenters. Stable slow recomputes must not rebuild
the same hash maps, floating frame vector and `5F` pointer-node candidate set merely to rediscover
unchanged geometry. Do not invent a per-recompute epoch: the receipt must represent the real model,
shared-source, native-host and metrics inputs.

## Reviewed source

- owner Rust files: 1/1
- physical lines: 654
- bytes: 23,516
- current SHA256: `9c58825e14718cd2076e25c7268358666a1d7fc9d2effcd0903910cc2146ecf7`
- owning commit at review: `08094b9b9e17f6c80372e15c17b01204038b305b`
- post-M0 owner lines: 665
- post-M0 owner bytes: 23,910
- post-M0 owner SHA256: `2cebd670f41436c9f11e3eea29437360fb4d283e3d2efd13622bbd9faa2ea55d`

The owner file was read in full, including all tests. The review also traced full/metrics/scoped
recompute decisions, committed shell state, floating source-template frames, native host snapshots,
Workbench layout frames, shell-pointer topology/geometry patching, presentation consumers and the
existing shell-pointer and host-scene performance records. The shell-pointer owner remains covered
by its separate 8/8 review; this record does not double-count it as newly reviewed.

## Current algorithm

`recompute_if_dirty` returns immediately for an unchanged retained host, so this is not an idle-frame
O(F) problem. On a slow recompute, floating projection:

1. recomputes shared source layout;
2. visits F model windows to submit native bounds candidates;
3. snapshots H native hosts and indexes them into a new `HashMap`;
4. visits F windows again, clones each window key and optional tree id, and builds a second
   `HashMap<MainPageId, FloatingWindowProjectionFrames>`;
5. shell pointer compares F topology ids, allocates and resolves an F-entry floating-frame vector,
   probes up to `5F` retained nodes, then compares/patches another `5F` nodes.

Before main/native presentation consumers, a stable-topology slow recompute can therefore perform
up to `H + 14F` direct candidate visits plus fixed root work. This is a source operation model, not
a measured timing claim.

## Findings

### P0 foundation: the native-host index is the correct linear algorithm

The owner first indexes H native hosts and then performs F lookups. This is O(H + F), replacing the
O(H*F) nested lookup that a direct per-window scan would create. Keep the `HashMap`; the structural
problem is rebuilding it without an input receipt, not its lookup algorithm.

### P1: bundle identity is discarded after every slow recompute

`FloatingWindowProjectionBundle` is a value-owned `HashMap` with no generation or shared owner.
`CommittedShellState`, `BuiltinWorkbenchWindowLayoutFrames` and the bundle do not carry one coherent
geometry cursor, although the later host-contract presentation already has independent structure,
interaction, viewport and hit-test generations.

Consequently consumers can only compare topology/frames again. Adding a monotonically increasing
generation at bundle construction would be wrong: it would change on every slow recompute and make
reuse impossible while claiming that every identical value is new.

### P1: stable pointer patch allocates before it knows that geometry changed

`resolve_drag_hit_geometry` always collects an F-entry `Vec<Option<UiFrame>>`. `patch_drag_surface`
then probes and compares `9 + 5F` nodes. Surface rebuild and `ArcSwap` publication are changed-only,
which is correct, but the candidate construction and comparisons have already occurred.

### P1: source domains need independent receipts

The correct geometry key spans:

- ordered floating model identity, requested frames and window ids;
- floating source document/center frames and shell scale;
- native host window ids, bounds, host presence and surface tree ids;
- document-header and separator metrics;
- root size and componentized Workbench layout frames used by pointer clamping.

These domains change independently. One global host presentation generation is too broad; one local
hash computed from all values would repeat O(F) work. Mutation owners must publish receipts.

## Unreal source basis

Direct source read under `dev/UnrealEngine`:

- `SWindow` creates and retains one `FHittestGrid`, registers itself as the invalidation root, and
  retains persistent window geometry (`SWindow.cpp:2069-2080`, `2140-2149`).
- `FSlateInvalidationRoot::PaintInvalidationRoot` uses cached element data, enters the slow path only
  for explicit invalidation, uses the fast widget list when non-empty, and otherwise performs no
  widget update (`SlateInvalidationRoot.cpp:356-424`).
- `SDockingTabWell` separately retains typed tab children and current arranged geometry for drag/drop
  instead of reconstructing a detached window model.

The transferable rule is a persistent window/hit-test owner with explicit invalidation receipts.
Zircon should not copy Unreal's internal types, but its main window, native windows, shell pointer and
paint consumers must share one immutable geometry generation.

## Target architecture

1. M0 adds profile counters for bundle builds, H/F source rows, bounds-sync candidates, pointer
   geometry resolves, floating-frame candidates, node candidates, topology misses and unchanged
   geometry reuse. Counters must precede any cache implementation.
2. M1 makes floating model, native-window host registry and shared source-template layout publish
   independent immutable receipts at their mutation owners.
3. M2 constructs an `Arc`-owned floating geometry artifact keyed by those receipts plus metrics. Main
   presentation, native presenters and pointer routing share it.
4. M3 gives shell-pointer layout a composite committed cursor. Exact equality returns O(1) before
   constructing `DragHitGeometry`; topology change still performs a hard rebuild.
5. M4 evaluates the retained `9 + 5F` surface against a direct generation-owned spatial index at
   scale. Cut over only if measured CPU/allocation improves with route/z-order parity.

No unbounded cache, per-recompute content hash, fake epoch, legacy geometry facade or second native
host registry is allowed.

## Instrumentation and acceptance

Matrix: F `0/1/8/64/256/1000`; H `0/F/2F`; recompute
`idle/full/window-metrics/shell-content/workbench-projection`; change
`none/root/drawer/one-window/all-windows/topology/native-bounds/tree-id/metrics`; drag
`inactive/active`; input `10/125/500 Hz`.

Acceptance requires:

- idle frame: zero bundle builds and zero shell-pointer geometry resolves;
- M0 source counters reconcile with the operation model for each case;
- M2 unchanged slow recompute: zero H/F projection rows and zero bundle allocation;
- M3 exact cursor hit: zero floating-frame candidates and zero node candidates;
- changed one window: O(1) changed artifact work plus bounded publication, not O(F) cloning;
- topology change: one deliberate rebuild with exactly `9 + 5F` retained nodes;
- WPR/ETW reports main-thread p50/p95/max, CPU samples, allocation bytes/count, queue depth, OS
  window-call attempts and process energy estimate on one executable fingerprint;
- RenderDoc is used only if M4 changes rendered overlay/window output, to prove draw/pixel parity.

All executable, target, trace, allocation, power and RenderDoc artifacts must stay on D:, E: or F:.
No guessed latency or energy target is accepted.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add source-bound counters without changing behavior. | focused RED/GREEN and profile schema evidence |
| M1 | Publish source-domain receipts from mutation owners. | exact invalidation tests for every key domain |
| M2 | Share one immutable floating geometry artifact. | unchanged projection rows/builds = 0 |
| M3 | Add composite pointer-layout cursor and O(1) exact reuse. | candidates = 0 on exact hit |
| M4 | Compare retained surface with spatial index; hard-cut only with evidence. | F-scale WPR/allocation/route parity |
| M5 | Current-source real-window, power and any required RenderDoc acceptance. | quantified before/after artifact set |

## Validation state

- Owner review: complete, 1/1 current Rust file.
- Related recompute, pointer, native-host, presentation and Unreal sources: traced and mapped.
- Architecture report: recorded before instrumentation or optimization.
- M0 instrumentation: applied after this report was created. The profile stream now records bundle
  builds, native-host rows H, window rows F, bounds-sync candidates F, pointer geometry resolves,
  floating-frame candidates F, node candidates `9 + 5F`, topology misses and unchanged geometry
  reuse. These counters report operation cardinality only; they are not timing, allocation or power
  evidence.
- Focused static contract:
  `tools/tests/test_editor_floating_window_projection_spatial_scale_performance_contract.py`, 88
  lines, 3,351 bytes, SHA256
  `d79072cacf31f561e44b3d89909069418bbfa2ba40f118018647d748d311ed37`; RED 0/4 to GREEN 4/4.
- Broad current-worktree performance contracts: GREEN 265/265. Retained-host performance contracts:
  GREEN 82/82. Changed Rust files passed Rustfmt and scoped `git diff --check`.
- Managed Cargo is not executable because the current coordinator session is archived. WPR, power,
  real-window and RenderDoc evidence remain pending.
- The owner remains in `pending.md`; it must not enter `review.md` on static evidence alone.
