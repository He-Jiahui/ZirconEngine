---
title: Editor paint-recording tri-state damage and retained-range performance review
date: 2026-08-23
module: zircon_editor retained-host host_contract/paint_recording
priority: MVP-P0 editor invalidation recording
status: source_reviewed_architecture_fix_pending_dynamic_pending
reference_engine: Unreal Engine Slate invalidation root fast and slow paint paths
---

# Goal

Make recording distinguish an explicit full rebuild, accepted region patch and empty/no-op damage.
Clipping a requested patch to nothing must do zero recording/presentation work, and an accepted patch
must traverse only retained scene owners/ranges that intersect its damage set.

## Reviewed source

- Rust files: 3/3
- lines: 77
- bytes: 2,502
- joined UTF-8 path, NUL and raw-source-bytes SHA256:
  `5765bdbc11ff3b65098d252cd11060f5d176c8044388dec51d11c268b362646b`
- owning commit at review: `7762880fd1d8db3d3872888ba8377910177574af`

Scope: `zircon_editor/src/ui/retained_host/host_contract/paint_recording.rs` and
`paint_recording/**`.

Supporting production paths traced/read: workbench command dispatch, chrome extraction/layer
classification, stream full/patch constructors, Runtime draw-list damage, GPU/Softbuffer presenter
classification, CPU region repaint and redraw queue/retry contracts.

## Correct foundations to retain

1. Zero-size surfaces return before frame construction or scene traversal.
2. Accepted damage is clipped to surface bounds and becomes the recording frame's active paint clip.
   The clear command is restricted to the accepted region.
3. Recording-only frames avoid full RGBA framebuffer allocation, and primitive painters reject
   commands outside the active clip.
4. Full and patch streams already carry different layer policy and presenter diagnostics once a valid
   patch survives this boundary.

## Structural findings

### P0: explicit empty damage is converted into a full rebuild

The input uses `Option<&FrameRect>` where `None` means full. `clip_damage_to_frame` also returns `None`
when an explicit region is invalid or disjoint. `record_host_frame_commands` loses the input provenance
and interprets that result as full: it clears frame bounds and traverses the workbench. Extraction then
derives `full_rebuild = clipped_damage.is_none()`, so the misclassification propagates into static
layer selection and the stream constructor.

For any off-window patch the required useful pixel area and command count are zero; current work can
be a complete window command rebuild. This is not a finite area-amplification ratio and cannot be
fixed safely by inventing a zero-area `FrameRect`, because GPU/Softbuffer currently have no Empty stream
state.

### P0: an accepted region still enters the complete workbench root dispatcher

After setting the clip, recording unconditionally calls `draw_workbench_presentation_commands`.
Primitive rejection and recently added scene-layer/dock damage gates save some leaf work, but the root
route, componentized/host choice and any ungated owner resolution still execute. The July static review
already recorded this as PERF-MVP-151; the current source retains the same root boundary.

M3 connects a bounded damage-region set to retained root/chrome/dock/overlay ranges and spatial pane/
node indices. A clip is a correctness boundary, not a traversal scheduler.

### P0: full/region provenance is inferred after a lossy geometry operation

`ChromeCommandExtraction` stores only `Option<FrameRect>`, and `ChromeCommandStream` stores a damage
Option plus a separate bool. Full/patch/empty are therefore parallel conventions rather than one typed
state. Retry, redraw and frame geometry have adjacent lossy Option/union behavior documented by the
redraw and frame-geometry reviews.

M1 introduces one exhaustive `Full | Regions | Empty` contract at redraw acceptance and preserves it
through recording, extraction, command stream, retry and presentation. Only an explicit full request,
bootstrap or recovery transition may produce Full; region clipping can produce Regions or Empty only.

### P1: recording defines another private intersection/visibility implementation

`damage.rs` duplicates finite intersection and positive-size visibility instead of consuming the
typed geometry contract. Its `> 0.0` threshold also differs from paint-side visibility semantics found
in the frame/paint geometry review. M2 moves it to the shared logical/paintable/pixel damage-region
contract, preventing another source of empty/full escalation.

### P1: no counter exposes escalation or root traversal amplification

There are no counters for requested damage kind, accepted kind, clipped-empty requests, full
escalations, root owners visited, ranges reused/rebuilt or commands rejected by leaf clip. Without
these, a low patch command count can hide high traversal and string/resource preparation.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElements.h`

Unreal's invalidation root enters the slow path only from explicit cache/slow-path state. Otherwise it
calls `PaintFastPath` only when the retained widget path list is non-empty; cached element data remains
attached to the window/root. The result separately reports whether widgets were repainted. Widget
proxies and sorted update lists identify work before paint rather than relying on a primitive clip to
discover irrelevance at the leaf.

The transferable rules are explicit no-work state, retained update ownership, fast-path selection
before traversal and cached draw ranges. Zircon should retain its generation-safe values and bounded
damage set; it must not reproduce Unreal pointer/widget-list internals.

## Target architecture

1. One typed damage decision is `Full`, `Regions(DamageRegionSet)` or `Empty`, with documented legal
   transitions and no parallel bool/Option convention.
2. Clipping Full remains Full; clipping Regions returns bounded Regions or Empty; Empty remains Empty.
   Presentation skips Empty without command allocation, atlas lookup or backend submit.
3. Recording receives retained root/chrome/dock/overlay ranges and a spatial owner index. Regions patch
   only intersecting dirty generations; stable ranges are reused in z order.
4. Full is restricted to bootstrap, surface loss/recovery, explicit global invalidation or damage-set
   overflow policy with an attributed reason.
5. Shared frame geometry owns logical, paintable and pixel damage semantics for redraw, recording,
   replay and presentation.

## Instrumentation and acceptance

Matrix: damage `full/inside/partially clipped/disjoint/invalid/empty/2..N disjoint`, surface
`0/1/1080p/4K`, scene `Welcome/default/10k hierarchy/128 plugin panes`, backend
`GPU/softbuffer/snapshot`, state `steady/resize/recovery`.

| Evidence | Acceptance |
| --- | --- |
| requested/accepted damage kind and transition reason | disjoint/invalid region never becomes Full |
| root/owner/node visits and reused/rebuilt ranges | region work proportional to intersecting dirty owners |
| recorded/extracted/runtime commands and allocations | Empty exactly zero; stable ranges reused |
| leaf clip rejections after preparation | approaches zero after spatial/range routing |
| CPU/RSS/frame latency/context switches/WPR power | same executable/workload before and after |
| RenderDoc draws/scissors/uploads/GPU and pixel parity | accepted current backend build |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add transition/escalation/owner/range/command counters and capture baseline. | attributable recording cost |
| M1 | Hard-cut bool/Option conventions to `Full | Regions | Empty` through presenter. | empty request: zero work; no accidental Full |
| M2 | Adopt bounded `DamageRegionSet` and shared geometry semantics. | disjoint regions retained within budget |
| M3 | Route regions through spatial owners and retained command ranges. | traversal proportional to dirty intersections |
| M4 | Run managed scale, WPR/power, GPU/Softbuffer and RenderDoc/pixel matrix. | quantified accepted milestone |

## Validation state

- Full direct owner review: passed, 3/3 Rust files.
- Workbench dispatch, extraction, stream model, Runtime conversion and GPU/Softbuffer consumers:
  traced/read.
- Relevant Unreal invalidation fast/slow path and retained element sources: read and mapped above.
- No partial implementation was applied because the current ABI cannot express Empty without changing
  every downstream owner; a local sentinel would preserve the structural bug.
- Managed Rust, WPR and RenderDoc validation remain pending because the managed Cargo Session is
  terminal `archived` with `cargo_session_not_executable`; no elapsed-time, GPU or power claim exists.

This module remains in `pending.md` until M0-M4 pass on one source/executable/workload fingerprint.
