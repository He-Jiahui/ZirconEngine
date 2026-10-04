---
title: Editor native keyboard generation popup navigation performance review
date: 2026-08-23
module: zircon_editor retained-host native_keyboard
priority: MVP-P0 editor popup input latency
status: source_reviewed_m0_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate focus-path keyboard routing and retained menu navigation
---

# Goal

Move popup keyboard routing from event-time presentation discovery and row reconstruction to a
generation-owned navigation artifact. A stable open popup must resolve its target and current row
without scanning all workbench nodes, cloning the wide template-node DTO, or allocating and copying
every visible row for every key and text event.

## Reviewed source

- owner Rust files: 13/13
- lines: 1,035
- bytes: 39,136
- source-only SHA256 over lexicographically sorted owner files:
  `dec7a200f63ecad63cd02cc1a0bd09a078c99f8d91bbacebbf36f7867a0d81f4`
- post-M0 owner files/lines/bytes/SHA256: 13 / 1,059 / 39,735 /
  `35b78e7414b32b6eb3b8e0aedf87c8ab476d4837d0650af9f24e0fe53eb5a692`

| Owner group | Files | Lines | Bytes |
| --- | ---: | ---: | ---: |
| `native_keyboard.rs` | 1/1 | 12 | 419 |
| `native_keyboard/*.rs` | 4/4 | 118 | 4,464 |
| `native_keyboard/dispatch/**` | 1/1 | 141 | 5,696 |
| `native_keyboard/target/**` | 7/7 | 764 | 28,557 |

All owner files were read in full. Presentation generation, hit-index popup membership, popup
geometry/layout, interaction state, callback dispatch and page-overflow consumers were inspected as
supporting boundaries. The 2026-07-17 record covered 13 files but only 584 lines; this record
supersedes its owner coverage for the current tree.

## Correct foundations to retain

1. Keyboard commands are a closed enum and navigation uses index arithmetic after a target exists.
2. Command-palette navigation respects its 12-row visible window and requests only the needed next,
   previous or terminal window; it does not materialize the full command catalog.
3. Popup selection preserves interaction identity before focused/selected fallback, and disabled
   menu/options rows are excluded.
4. The committed `HostWorkbenchHitIndex` already records open, enabled popup node rows. Its rebind
   membership includes control identity, geometry, z-order, `popup_open`, `disabled` and dispatchability,
   so current-generation candidates are authoritative for keyboard discovery.
5. Dispatch emits bounded redraw regions and retains exact accept/cancel/page-window behavior.

## Structural findings

### P0: every key and text event rediscovers the active popup from presentation data

Both command and text dispatch call `active_popup_keyboard_target_for_ui`. Page overflow is tested
first; ordinary popups then consult only `has_popup_rows` and ignore the indexed rows, reverse-scan
all `workbench_window_nodes`, and call cloning `row_data` for each candidate. `TemplatePaneNodeData`
is a 163-field presentation DTO, so the scan performs deep reference/string/model clones before it
can reject most nodes.

M0 now reuses the same-generation popup row slice and borrowed node access. This changes ordinary
target discovery from O(N) node probes plus wide node clones to O(P) borrowed probes, where P is the
number of open eligible popup nodes. M1 removes discovery from stable events entirely by publishing
the active popup navigation owner in the presentation generation.

### P0: visible popup rows are reconstructed and copied on every event

For every key and text event, option and menu builders allocate a fresh `Vec<PopupKeyboardRow>`,
clone action/value/identity/search strings, recompute row frames and then clone the selected row.
Page overflow repeats the same pattern and additionally formats every page index. This work scales
with visible rows and input repeat frequency even when neither structure nor popup window changed.

M1 introduces `HostPopupNavigationArtifact`, built or partially rebound with the arranged generation.
It owns popup kind, control/action identity, ordered enabled row descriptors, row frames, active row,
visible window metadata and normalized searchable text. Events borrow the artifact and update only an
index/interaction delta. M1 must hard-delete ordinary per-event row-vector construction rather than
cache a second independently versioned copy.

### P1: prefix search allocates lowercase strings per candidate

The query is normalized once, but every candidate field calls `to_lowercase()` before `starts_with`.
One row may probe search text, value and identity, producing up to three temporary strings. M0 now
uses a lazy Unicode lowercase iterator that stops at the prefix boundary and allocates no candidate
string. A Rust regression preserves Unicode lowercase expansion and empty-prefix semantics. M1
precomputes normalized search keys in the navigation artifact and reports candidate probes.

### P1: selection and geometry repeat work already known by the arranged generation

Selection can scan rows three times: interaction identity, focused, then selected. Popup geometry and
row frames are recomputed independently by keyboard, painting and hit testing. The page-overflow path
also clones tab metadata even though its hidden-row projection already exists in presentation data.

M1 unifies popup row identity/frame/eligibility under one arranged artifact and stores the active row
index. Paint, hit, dismiss and keyboard consumers share one structure/hit generation; no consumer may
silently reconstruct a current popup from raw nodes.

### P1: the hot path has no workload or allocation evidence

There are no counters for keyboard target requests, indexed popup candidates, node visits, wide-node
clones, row builds, row/frame/string copies, search probes, allocations, window requests or dispatch
latency. Without these, a fast 12-row command-palette case can hide repeated work at key-repeat rate.
M1 adds reason-coded counters and ETW spans before the structural cutover is accepted.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/MenuStack.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/MultiBox/MultiBox.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/MultiBox/SMenuEntryBlock.cpp`

`FSlateApplication::ProcessKeyDownEvent` obtains the user's retained focus path and tunnels/bubbles
the key along that path (`SlateApplication.cpp:5096-5134`). It does not scan the whole window tree to
rediscover an open menu on every key. The menu stack provides a focusable owner that forwards menu
keys and dismisses on Escape (`MenuStack.cpp:195-228`). `SMultiBoxWidget::OnKeyDown` advances the
existing menu focus for Up/Down (`MultiBox.cpp:1887-1901`), while a concrete menu entry delegates
those keys to its owning multibox (`SMenuEntryBlock.cpp:1454-1469`).

The transferable rule is retained focus/navigation ownership tied to the arranged widget/menu
lifetime. Zircon should keep its data-oriented generation model, but the active popup and rows must
be generation artifacts, not event-time reconstructions from the full presentation DTO.

## Target architecture

1. `HostPresentationGeneration` publishes one `HostPopupNavigationArtifact` for the active page or
   workbench popup, versioned with structure, hit membership and popup-window projection.
2. The artifact contains typed popup kind, control/action identity, popup frame, enabled row
   descriptors, row frames, normalized search keys, active row and virtualized window metadata.
3. Keyboard dispatch borrows the artifact. Navigation is O(1); text search is O(V) prefix probes over
   the visible window without heap allocation; V stays virtualization-bounded.
4. Interaction updates change the active row/index without rebuilding stable row descriptors.
5. Paint, hit, dismiss and keyboard share the same arranged popup artifact or explicit projections
   from it. Current-generation operation has no full-node scan and no independent geometry rebuild.
6. Full build, partial rebind, slow fallback and window request have separate reason-coded telemetry.

## Instrumentation and acceptance

| Evidence | Acceptance |
| --- | --- |
| target requests/index candidates/node visits | stable ordinary popup visits P indexed candidates, never N nodes |
| node/row/string/frame clones and allocations | zero event-time wide-node clone; zero stable navigation row rebuild after M1 |
| row artifact full builds/rebinds/reasons | only structure/window changes rebuild rows |
| key command CPU p50/p95/p99 | slope independent of total presentation nodes |
| text probes/bytes/allocations | O(V) prefix probes; zero candidate lowercase allocation |
| window requests and visible rows | command palette remains bounded to configured visible window |
| interaction generation updates | active-row moves do not rebuild structure artifact |
| correctness | page/options/menu, disabled, hover/focus/selected, cancel/accept and boundary parity |

Matrix: presentation nodes `1/100/1K/10K`; simultaneous popup candidates `1/2/10`; visible rows
`1/12/100/1K`; key repeat `1/10/30/60 Hz`; query `ASCII/Unicode/no-match/late-match`; popup
`page/options/menu/command-palette`; update `none/hover/selection/window/structure`; scale
`1x/1.5x/2x/4K`.

WPR owns CPU, allocation, context-switch and power evidence. RenderDoc is not a keyboard acceptance
tool; it is used only after a current-source GPU presenter is launchable and only for draw/resource/
pixel parity of any shared popup-artifact cutover. All artifacts remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Reuse generation popup candidates, borrow node rows and remove candidate lowercase allocation. | applied; static contracts GREEN, managed Rust/dynamic pending |
| M1 | Publish shared typed popup navigation artifact with telemetry; delete stable per-event row/geometry rebuilds. | zero stable-event artifact rebuild and row allocation |
| M2 | Share artifact projections with paint/hit/dismiss and optimize interaction-only active-index updates. | one generation authority; no normal raw-node fallback |
| M3 | Run scale/key-repeat/Unicode/WPR/power/UI parity and GPU popup parity where applicable. | quantified acceptance and milestone closeout |

## Validation state

- Owner source review: passed, 13/13 current Rust files.
- Generation, hit membership, geometry/layout, interaction and dispatch boundaries: read and mapped.
- Unreal focus-path and menu navigation sources: read and mapped.
- M0 static performance contract moved RED 0/3 to GREEN 3/3. The adjacent popup binding, surface,
  hit-index, presentation-generation, preview-index and paged-keyboard set passes 18/18.
- The changed Rust files pass independent `rustfmt --check`; scoped `git diff --check` passes with
  line-ending warnings only.
- Performance-contract discovery passes 136/142. The six unrelated failures remain the known two
  missing test-support files, missing `available_slots`, preview resize `.roots.clone()`, UI-asset
  root helper `.roots.clone()` and Runtime 07 source/telemetry/owner-gate document drift.
- Managed Rust tests, current-source launch, WPR and RenderDoc are pending because the managed Cargo
  Session is terminal `archived` with `cargo_session_not_executable`. No raw Cargo bypass is allowed.
- M0 dynamic acceptance and M1-M3 remain pending; this owner stays out of `review.md` until dynamic
  acceptance.
