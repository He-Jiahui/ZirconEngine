# Window resize frame-cadence geometry publication review

Date: 2026-08-28

Status: static implementation candidate and deterministic pressure evidence. The trailing gate is
removed from current source; managed lower tests and product latency/CPU/RSS/GPU remain pending.

## Outcome

The current source now has one frame-bound resize path:

1. a size event updates committed host metrics, retains only the latest pending presenter extent,
   and merges an interactive frame request; and
2. `RedrawRequested` configures that latest extent, runs the interactive retained-frame commit,
   and presents the resulting normal generation.

The former 80 ms timer, pending-reflow flag, redraw suppression, stretched-snapshot present branch,
and trailing scheduler have been deleted from the production path. Multiple native events before a
redraw overwrite the pending extent and merge into the existing `HostRedrawRequest`, so native redraw
is scheduled only on the empty-to-pending transition.

This closes the known interaction freeze in the event scheduler. It does not yet prove that the
downstream interactive commit is geometry-only: affected-node layout/hit work, semantic generation
stability, stale-geometry presentation, and product timing still require dynamic evidence.

## Current-source state machine

The production chain is now:

- `host_contract/window/event_loop/events/resize.rs` rejects duplicate metrics, updates the host
  window state, retains the latest physical presenter size, and queues a resize request with
  `into_interactive_frame_update()`.
- `host_contract/window/event_loop/redraw.rs` consumes the merged redraw, configures the latest
  pending presenter extent first, invokes `request_interactive_frame_update`, then calls the normal
  present path.
- `host_contract/window/event_loop/lifecycle.rs` has no resize deadline or trailing scheduler.
- `app/host_lifecycle/shell_metrics.rs` reads the already committed host metrics without a pending
  resize gate.
- `host_contract/window/event_loop/redraw/present.rs` has no resize-only stale snapshot branch.
- `app/host_lifecycle/recompute.rs:72-150` classifies `WindowMetrics`, but still continues through
  floating projection, viewport/pointer synchronization, presentation application, native
  presenters and pointer surfaces. Retained pane payloads can be reused, but the transaction is
  not geometry-only.
- `host_contract/globals/state.rs:171-188` replaces the monolithic
  `Arc<HostWindowPresentationData>` and advances `presentation_structure_generation`.

Runtime UI below this Editor boundary already has the necessary local resize shape.
`ui/layout/pass/incremental.rs` detects `pure_root_resize`, reuses clean descendant measurement and
arranges only roots plus parent-size-dependent descendants. The following arranged/hit/render patch
uses the combined affected-node set. The existing focused contract models 10,000 nodes and 200 resize
steps as 400 incremental arrange probes and 600 nodes per post-layout domain instead of two million
full probes/patches per domain. The missing work is therefore not another Runtime leaf cache: Editor
must publish current window metrics into that incremental authority at frame cadence without first
rebuilding semantic pane/presentation products.

The current product counter gate correctly requires ordinary resize to have zero presentation,
shell, host-scene and pane-projection rebuilds and zero structure-generation advances. However,
the latency pipeline currently accepts an input-to-damage/present sample even when the presented
surface extent and retained geometry generation do not match. It can therefore report a short
submission latency for a visually stale frame.

## Unreal reference contract

The checked-in Unreal source uses a different authority boundary:

- `SlateApplication.cpp:7060-7080` handles a size event by updating `SWindow`'s cached size,
  requesting renderer resize and drawing the window in that same size-change path when eligible.
- `SWindow.cpp:1057-1070` changes the cached size and invalidates the root child order immediately.
- `SlateApplication.cpp:7122-7125` flushes pending renderer commands during interactive resizing so
  commands for an old viewport size do not remain queued.
- `SlateInvalidationRoot.cpp:356-424` owns cached element data and chooses slow or fast paint from
  explicit invalidation state. Desktop-position-only changes use the separate screen-position
  shift path at lines 342-380.

Unreal does not prove a portable millisecond threshold for Zircon. The transferable contract is
that current window size, widget invalidation and the draw being presented belong to the same
frame authority. It does not intentionally present old widget geometry for an arbitrary trailing
debounce interval.

## Required algorithm

The event ingress is now latest-value and frame-bound. The remaining atomic geometry publisher
target is:

```rust
struct PendingWindowMetrics {
    input_sequence: u64,
    metrics: UiWindowMetrics,
    received_at: Instant,
}

struct HostGeometryGeneration {
    generation: u64,
    source_input_sequence: u64,
    client_size: UiSize,
    raster_scale: f32,
    layout_frames: Arc<HostLayoutFrames>,
    hit_index: Arc<HostWorkbenchHitIndex>,
    damage: HostDamageRegion,
}
```

On each presentation cadence boundary:

1. Consume only the newest pending metrics. Record older pending sequences as typed coalesced
   outcomes rather than pretending each was presented.
2. Capture the committed semantic structure generation and previous geometry generation.
3. Recompute only affected layout roots. A root resize may legitimately affect all layout nodes,
   but it must not visit pane payloads, menu strings, semantic template nodes or other `S`-sized
   presentation data.
4. Compare the mounted-control/native-surface/responsive-breakpoint signature with the committed
   structure. A real topology change returns a typed structural fallback.
5. Patch changed hit entries/cells and form bounded old/new damage from the same frame deltas.
6. Validate finite geometry and expected generations, then atomically publish geometry, hit index,
   damage and source input sequence.
7. Present only a command generation whose geometry client size matches the configured surface
   extent. If preparation misses the current cadence slot, retain the latest request for the next
   slot instead of presenting a newly sized surface with silently stale geometry.

If another size event arrives while preparing, it replaces the pending value and runs on the next
cadence slot. There is never more than one geometry preparation per surface per frame.

## Complexity and pressure evidence

Let `S` be semantic nodes/payload rows, `L` total layout nodes, `H` total hit entries,
`L_aff/H_aff` affected rows and `D` bounded damage regions.

- Rejected trailing final reflow: `O(S + L + H + D)` once after the trailing interval.
- Target per cadence commit: `O(L_aff + H_aff + D)` and `O(L_aff + D)` transient memory.
- Target over an interaction: multiply the local cost by cadence commits, not raw native events.

The deterministic model is `tools/editor_window_resize_reflow_pressure.py`. Schema v2 labels the
trailing path as a rejected baseline rather than a current implementation and binds the artifact to
current Zircon sources, checked-in Unreal sources, and HEAD
`b2e76ff33cc298ad76f7b801a1d06d1e2faa046d`. The artifact is
`E:\zircon-profiles\editor-window-resize-reflow-pressure-20260829-r2.json`, SHA-256
`CA7ED75C587695928681706A98FE931970A07F4ECE6C975FDD1C4E236A561510`.

For the existing profile default, 24 generated steps plus restoration produce 25 events at 40 ms:

| Metric | Rejected trailing baseline | Frame-cadence geometry target |
| --- | ---: | ---: |
| retained geometry commits | 1 | 25 |
| coalesced native events | 24 | 0 |
| geometry mismatch window | 1,040 ms / 63 frame budgets | at most 13.333 ms in the model |
| full-frame redraw requests | 26 | not modeled; extent changes may require full pixels |
| semantic projection visits | 10,000 | 0 |
| modeled operation units | 30,008 | 3,400 |

For 2,000 events at 4 ms, the rejected mismatch window becomes 8,076 ms/485 frame budgets. The
cadence target commits 481 generations, coalesces 1,519 events and performs 65,416 modeled local
operation units. This exceeds the rejected single final reflow's 30,008 units. That is an explicit
tradeoff: an interface that refuses to update for eight seconds can minimize total work while
remaining unusable. Acceptance must jointly bound freshness and per-commit work.

Operation units are row/region visits supplied to the model, not CPU cycles. The 10,000/64/8
parameters are a stated pressure scenario, not measured current tree cardinalities. The model
excludes platform scheduling, allocator, CPU/RSS, surface fill, GPU submission, responsive
fallback and real layout dependency propagation.

## Missing observability

Add source-bound fields/counters before accepting implementation:

- `window_metrics_received_count`, duplicate count and typed coalesced sequence ranges;
- `geometry_prepare_count`, commit count, no-op count and fallback count by reason;
- source input sequence, client size and raster scale on each geometry generation;
- configured surface extent and geometry extent on each present;
- `stale_geometry_present_count` for any present where those extents/generations disagree;
- `resize_input_to_matching_geometry_us`, with p50/p95/p99/max;
- layout roots/nodes visited, changed frame rows, hit entries/cells patched and full rebuilds;
- semantic presentation, shell, host-scene, pane-projection and structure-generation counters;
- command fragment reuse, full command rebuild, text/SVG cache misses and upload bytes;
- damaged/presented pixels, resize CPU duration, allocation count/bytes and process RSS.

Count conservation is mandatory:

```text
non_duplicate_resize_inputs
  = typed_coalesced_inputs
  + matching_geometry_presented_inputs
  + typed_fallback_inputs
```

The existing `input_to_damage_us` and `damage_to_submit_us` remain useful, but cannot substitute
for input-to-matching-geometry evidence.

The read-only executable gate is `tools/ui-window-resize-geometry-evidence.ps1`. It rejects
missing counters instead of treating an absent stream as zero, validates both input and geometry
transaction conservation, and computes nearest-rank summaries for
`ui.window_resize.input_to_matching_geometry_us` and
`ui.window_resize.geometry_prepare_us`. Its current budgets are 16.67 ms p95 / 33.34 ms max for
matching geometry and 8 ms p95 for geometry preparation. The gate is intentionally standalone
until the shared capture owner is stable; passing synthetic contract tests is not product evidence.

## Test and product acceptance plan

Static source tests now verify the absence of the timer/gate, latest-extent overwrite, interactive
frame request, presenter configuration before frame commit, frame commit before normal present, and
the Unreal cached-size/renderer-resize/draw order. They pass 21/21 across the focused resize,
interactive-frame, and structure suites; Rust tests are written but have not been run.
The full static performance-contract discovery passes 1,578/1,599. Its 16 failures and five errors
are in shared presenter-signature, resource-module, plugin-reflection, Runtime text/streaming, and
other foreign contracts; none is a resize scheduling or frame-bound publication failure.

Remaining dynamic plan:

1. Lower test: several metrics arrive before one cadence slot; only the latest is prepared, older
   sequences are coalesced, and one complete geometry/hit/damage generation publishes.
2. Lower test: a second event arrives during preparation; it remains pending for the next slot and
   cannot mutate the generation being committed.
3. Lower test: ordinary size change preserves semantic `Arc` identity and structure generation;
   responsive breakpoint change returns its typed fallback.
4. Lower test: old-only hit regions reject, new-only regions hit and rendering/hit testing read the
   same generation.
5. Lower test: desktop move remains the separate `O(1)` transform path; DPI invalidates only
   scale-keyed geometry/text/SVG/raster products.
6. Product profile: existing 25-step/40 ms sequence, 200 steps at 8 ms and 2,000 steps at 4 ms.
   Use the same PID for warm-up, measured run and quiescence; run at least three measured samples.
7. Product oracle: after every sequence, compare screenshot pixels, frame rectangles and boundary
   hit probes with a forced full rebuild at the same final metrics.

Ordinary resize acceptance gates:

- `stale_geometry_present_count == 0`;
- matching-geometry latency p95 at or below 16.67 ms and max at or below 33.34 ms on the reference
  machine;
- geometry prepare/commit CPU p95 at or below 8 ms;
- zero semantic presentation/shell/host-scene/pane rebuilds and zero structure-generation change;
- zero full hit-index and full command-set rebuilds;
- committed generations no greater than presentation cadence slots plus one final restoration;
- no text/SVG decode or GPU upload for unchanged content/scale keys;
- no unexplained monotonic RSS growth; post-quiescence growth below 5% in the 2,000-step run;
- count conservation and final visual/hit parity both pass.

Client-size changes can require painting the full new surface. Full pixel damage is not itself a
failure. The failures are semantic reconstruction, mismatched geometry presentation, repeated
resource work and unbounded CPU/memory growth.

## Implementation ownership

This slice changes only the resize scheduling/publication boundary: it removes the timer-owned state
and makes the existing interactive frame commit authoritative at redraw. Shared recompute,
presentation, renderer, and profiler owners are not absorbed. The next implementation step is to
prove and, where necessary, hard-cut the semantic/geometry authority split below that boundary,
followed by managed lower tests and current-source product evidence. The old timer-only architecture
remains explicitly rejected.
