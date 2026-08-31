---
title: Editor redraw coalescing and damage-queue performance review
date: 2026-08-23
module: zircon_editor retained-host redraw request and dispatch result
priority: MVP-P0 editor event-loop redraw scheduling and presentation damage
status: source_reviewed_architecture_pending_dynamic
reference_engine: Unreal Engine Slate invalidation root and cached element ownership
---

# Goal

Coalesce high-frequency editor input without creating a native redraw storm and without converting
multiple small dirty regions into one large bounding repaint. Frame-update, presentation, retry and
profiling semantics must survive a retained multi-region damage representation end to end.

## Reviewed source

- Rust files: 7/7
- lines: 474
- bytes: 13,382
- joined normalized UTF-8 path, NUL and raw-source-bytes SHA256:
  `f52ff84d04a49d90801f6771b41dd800c4656b2b387c56390014869b1175fbac`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

Scope: `redraw.rs`, `redraw/**` and path-owned `redraw_tests.rs`.

Supporting production paths traced/read: host external-redraw state, event-loop pending redraw,
surface-present retry, input outcome tracking, native scheduling, presenter dispatch and damage-aware
GPU/softbuffer presentation.

## Correct foundations to retain

1. `queue_redraw` asks winit for a native redraw only on the empty-to-pending transition. Additional
   input/external requests merge into pending state without posting one OS redraw per event.
2. Frame-update-only work is distinct from visual presentation. Merge preserves a required frame
   update when combined with visual damage, and full damage dominates a region.
3. External redraw state and event-loop pending state are bounded single values, not unbounded channels.
   Existing queued/coalesced/drained counters establish an initial observation point.
4. Retryable surface failure is kept outside the normal native redraw queue and uses bounded exponential
   backoff from 8ms to 250ms. A successful present resets backoff; retry does not consume the measured
   input batch.
5. Native resize configures/presents the latest surface without rerunning retained reflow for every
   intermediate size.

## Structural findings

### P0: region information collapses at every redraw ownership boundary

`HostRedrawRequest::Region` owns one `FrameRect`. Pointer dispatch merges old/new effects with bounding
union; the host external-redraw slot merges again; the event-loop pending slot merges again; deferred
surface retry merges with the next redraw; finally `present_redraw` accepts `Option<FrameRect>`.

The event queue is bounded, but useful spatial information is destroyed before command extraction.
The 4,050x opposite-corner 1080p example from the geometry review therefore applies at every coalescing
stage. M1 replaces the Region payload with the canonical retained `DamageRegionSet`, and M2 carries it
through all external/pending/retry/presenter boundaries without a last-mile bounding union.

### P0: coalescing counters hide spatial amplification

`external_redraw_coalesced_count` records that requests merged, while `RedrawRegion` counts each region
constructor. Neither records input/output region count, useful area, union area or full promotion. More
coalescing can therefore look beneficial while it increases paint area by orders of magnitude.

M0 records useful/clipped/presented area, region count, overlap/merge decisions, amplification and
promotion reason at external, pending and presenter boundaries.

### P0: presenter retry reconstructs the lossy request

On retryable surface failure, `present_redraw` converts the single optional damage rectangle back into
a new `HostRedrawRequest`. Multi-region migration must preserve the exact region set and source generation
through retry; otherwise a rare surface failure permanently promotes the next successful present to a
larger repaint or full frame.

### P1: one scenario label represents a merged input batch

Merge selects the latest scenario only when the newer request requires a frame update; otherwise an
earlier scenario can label multiple later visual regions. Runtime cost is unchanged, but WPR/latency
attribution can point to the wrong source and defeat optimization decisions. M3 records per-source region
counts and defines deterministic batch attribution separately from frame-update ownership.

### P1: invalid frame-update regions escalate to full presentation

`request_frame_update_region` creates a region-with-update; if geometry rejects it, the caller queues a
full frame. This is safe for unknown visual consequences but a zero/non-finite frame can turn a narrow
update request into full-window work. M0 records this promotion reason; M3 distinguishes frame-update-only,
unknown damage and explicitly full visual invalidation rather than inferring from invalid geometry.

### P2: fixed-size damage clones are not the bottleneck

`NativePointerDispatchResult::damage_region` and event-loop presentation clone `FrameRect`. These are
16-byte fixed-size values with no heap allocation. Region representation and area amplification take
priority; a borrowed query can be retained for API clarity but is not a standalone performance milestone.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElements.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/SlateRect.h`

Unreal coalesces work by retaining invalid widget proxies/cached element lists within an invalidation
root. A window paint still has one element-list owner, but dirty widgets/ranges remain distinct until
the invalidation pass decides what to rebuild. Zircon's transferable constraint is bounded event
scheduling plus retained spatial/range invalid owners, not one OS event per widget and not one bounding
rectangle for the entire input batch.

## Target architecture

1. `HostRedrawRequest` keeps orthogonal `frame_update`, scenario/batch attribution and visual damage
   state (`none`, bounded regions or full), instead of encoding all combinations as a lossy enum.
2. External redraw, event-loop pending state and surface retry each own the same bounded region-set type.
3. One empty-to-pending transition schedules winit redraw; subsequent input only updates retained state.
4. Presentation consumes clipped regions plus affected scene/range generations. Retry stores the exact
   unsubmitted request and success consumes it once.
5. Profiling reports merge/promotion decisions and useful/presented area per input batch/source.

## Instrumentation and acceptance

Matrix: events `1/1k/1M`, rate `125/500/1000Hz`, regions `1/2/8/64`, placement
`overlap/adjacent/opposite`, frame update `none/every/1%`, surface retry `0/1/5`, backend
`GPU/softbuffer/snapshot`, source `hover/menu/drag/resize/plugin`.

| Evidence | Acceptance |
| --- | --- |
| OS redraw requests vs input requests | one request per empty-to-pending transition |
| external/pending/retry region counts and bytes | bounded; exact regions retained |
| useful/clipped/presented area and amplification | no default bounding-space repaint |
| frame-update and full-promotion reason | explicit, no invalid-geometry inference |
| input batch/scenario attribution | deterministic source-aware counters |
| CPU/RSS/latency/context switches/power | same executable/workload before and after |
| RenderDoc draw/batch/scissor/GPU and pixel parity | accepted current backend build |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add scheduling, region/area/amplification, promotion, retry and source counters; capture. | attributable baseline |
| M1 | Change redraw visual state to bounded `DamageRegionSet` while retaining O(1) native scheduling. | pending requests retain disjoint regions |
| M2 | Propagate exact regions through external state, event loop, retry and presenter/range extraction. | no last-mile bounding union |
| M3 | Separate update/unknown/full semantics and batch scenario attribution; hard-cut old queries. | explicit promotion and attribution |
| M4 | Run managed event-storm/WPR/power and RenderDoc/scissor/pixel matrix. | quantified accepted milestone |

## Validation state

- Full direct owner review: passed, 7/7 Rust files including path-owned tests.
- External/event-loop/retry/presenter and input-outcome consumers: traced/read.
- Relevant Unreal invalidation, cached-element, rect and hit-grid sources: read and mapped.
- Existing tests cover region/full/frame-update merge, latest update scenario, one native schedule per
  pending batch and bounded retry behavior; multi-region behavior does not exist yet.
- No Rust change applied: presenter and retry still accept one rectangle, so a local enum wrapper would
  collapse before submission and create a false optimization.
- Current owned editor performance-contract set remains GREEN 79/79; broad set remains 106/111 with the
  five unchanged known failures documented by adjacent reports.
- Managed Rust, WPR and RenderDoc validation remain pending because the managed Cargo Session is
  terminal `archived` with `cargo_session_not_executable`; no elapsed-time, GPU or power claim exists.

This module remains in `pending.md` until M0-M4 pass on one source/executable/workload fingerprint.
