---
title: Editor native pointer button reply and capture performance review
date: 2026-08-23
module: zircon_editor retained-host native_pointer button_dispatch
priority: MVP-P0 editor click latency, capture release and redraw authority
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine Slate retained widget path and FReply routing
---

# Goal

Make one button fact follow one retained route and return one typed reply. Captured release must not
acquire an unrelated presentation generation, drag detection and activation must not run as two
parallel callback traversals, and event consumption must be independent from redraw/frame-update
requests.

## Reviewed source

- reviewed pre-M0 Rust files: 105/105
- pre-M0 lines: 2,292
- pre-M0 bytes: 80,964
- pre-M0 source-only SHA256 over lexicographically sorted owner files:
  `04950173b88f801cd12f9e6348fd922fe0d29f81e055bc65e0f4486cd26840f9`
- post-M0 Rust files: 104/104; the forwarding-only `entry/sequence/steps/release.rs` was removed
- post-M0 lines: 2,285
- post-M0 bytes: 80,779
- post-M0 source-only SHA256 over lexicographically sorted owner files:
  `c8627496a87d084a6476fff73b7e841ae7dd4bee4f4a121c4f0cf71172818504`
- current after routing M0 integration: 104 files, 2,289 lines, 80,930 bytes, SHA256
  `33937b40171dac3a850ec2f3bd5e0d8ff11728c61713eb89504256ad793a6330`
- owning commit at review: `7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`

Owner scope is `native_pointer/button_dispatch.rs + button_dispatch/**`. All files were read in
full. The generation/state accessors, drag/resize capture, template activation, hierarchy/welcome/
asset callback wiring and handlers, pointer-state writeback, routing and redraw contracts were
traced directly. `primary_press.rs` already contained a local generation-routing change at review
start; it is preserved as current source, not attributed to this owner M0.

The obsolete 2026-07-17 report described a deep-cloned presentation snapshot. Current source now
uses `HostPresentationGeneration` with retained `Arc` members, so that claim is closed. A generation
read still borrows host state, clones nine retained handles/options and increments a counter; it is
not free and remains unnecessary for captured release.

## Correct foundations to retain

1. Release capture precedes overlays and normal body routes once dispatch steps begin.
2. Popup, menu, Workbench popup, chrome, Workbench node and pane ordering is explicit.
3. Viewport body input forwards the event without inventing a viewport redraw; text focus damage is
   carried separately.
4. One retained presentation generation is reused by overlays and Workbench hit testing.
5. Passive pane press can consume the event while returning idle when focus did not change.

## Structural findings

### P0: captured release acquires an unused presentation generation

`dispatch_native_pointer_button` calls `button_dispatch_input`, which obtains a complete
`HostPresentationGeneration`, before `finish_primary_capture_if_released`. Resize/tab-drag release
then returns immediately without using that generation. M0 normalizes the button and finishes
capture before the generation read. The click performance timer remains outside this work so the
trace still measures the complete input fact.

### P0: callback ownership is erased into `bool`/`()`, then damage is guessed

Hierarchy and asset targets return only whether a target kind matched. Their callbacks can update
interaction state, arm/clear drag payloads, apply runtime effects or request later structural work,
but `pane_button_fallback_damage` cannot observe the reply. It therefore redraws the complete pane
on every release and requests press frame update with guessed pane damage/full-frame fallback.

Comparing only interaction generation is unsafe: runtime effects can require a later frame update
without synchronously changing that generation. M1 must return a typed reply containing
`consumed`, `changed`, `frame_update`, capture/drag intent, owner id and exact damage. Until then,
M0 does not suppress callback redraw based on a partial generation heuristic.

### P0: one primary press traverses event and click callback stacks

Hierarchy, asset content and asset reference primary press first emit a generic pointer event and
then a clicked callback. The event branch prepares retained pointer targets and arms drag payloads;
the click branch prepares/synchronizes the target again and dispatches activation/selection. This
duplicates committed-layout/focus/target/bridge work for one physical fact and makes ordering an
implicit convention.

M1 merges these into one route reply that can request drag detection and activation together.

### P0: no-change branches still allocate or redraw

- viewport toolbar click converts a borrowed stable control id to a new `String` only for damage
  lookup;
- template pane release clones a wide `TemplateNodePointerHit` before rejecting non-primary-press;
- a click in overflow popup padding redraws the popup although state did not change;
- close-prompt release/secondary input allocates an action String before returning idle.

M0 removes these four costs without changing callback order or visual state. Closed action/control
identities move to typed ids in M1.

### P1: the dispatch pipeline is fragmented without a reply boundary

The 2,292-line owner is split across 105 files, including nested `entry/sequence/steps`,
`body_routes`, `target/entry/sequence/run` and `result_targets` forwarding layers. File count alone
is not a runtime measurement and optimized builds may inline these calls, but the current shape
duplicates input structs and `Option<FrameRect>` clones while hiding the missing event-reply
contract. M2 collapses forwarding-only layers only after the typed route/reply boundary exists.

### P1: chrome damage is predicted before the callback runs

Chrome routing computes resize/tab/rail damage before invoking a void callback. It cannot express
unchanged, actual new bounds, popup side effects or multiple disjoint regions. M2 moves damage into
the typed transition receipt and the shared multi-region invalidation queue.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp`
  - `ProcessMouseButtonDownEvent` routes captured input directly to the captor path; otherwise it
    locates one retained `FWidgetPath` and calls `RoutePointerDownEvent`.
  - `RoutePointerDownEvent` tunnels preview once, bubbles only while unhandled, and returns one
    `FReply`.
  - `ProcessMouseButtonUpEvent` passes an empty path so a hit path is built only when capture did not
    handle release.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/ReplyBase.h`
  - the reply explicitly preserves handled state and handler ownership.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/Reply.h`
  - one handled reply carries capture/release, focus, drag detection/drop and throttling intent.

The applicable model is not to copy Slate types mechanically. Zircon needs the same ownership
property: one retained route, one event reply, and explicit effects. Handled input must not imply
paint invalidation; damage remains owned by state/paint invalidation.

## Target architecture

1. Normalize input and resolve active capture before acquiring a presentation generation.
2. Resolve one typed retained route from one generation; no Workbench-then-pane rediscovery.
3. Return `ButtonDispatchReply { consumed, changed, frame_update, capture, drag, focus, damage,
   owner, reason }` from every target.
4. Combine drag detection and activation in the same target transition.
5. Publish one atomic interaction/structural patch and enqueue exact multi-region damage only for
   changed pixels.
6. Remove bool/void callback routes, String identities and conservative pane/full-frame fallbacks
   in a hard cutover.

## Instrumentation and acceptance

Matrix: state `press/release`; button `primary/secondary/middle`; capture
`none/resize/tab-drag`; target `none/popup/menu/chrome/workbench/template/native/asset/viewport/
passive`; rows `0/1/100/10K/1M`; backend `GPU/softbuffer/snapshot`; scale `1x/1.5x/2x/4K`.

Record per fact: generation reads, route/hit builds, callback count, target preparations, String
bytes allocated, interaction/structure publications, frame updates, redraw requests, damage area,
main-thread CPU and input-to-present latency. Acceptance requires:

- captured release generation reads = 0 and unrelated route/hit/callback work = 0;
- one route build and one typed callback reply per uncaptured fact;
- unchanged/passive/padding facts redraw = 0 and frame update = 0;
- one publication per visual change;
- no full-pane/full-frame fallback for a known target;
- no allocation growth with row count and no closed-id String allocation;
- p95 input dispatch below 0.20 ms at 10K rows and below 0.35 ms at 1M rows on the recorded host;
- idle/no-change energy trace has no input-caused present burst.

WPR owns CPU/scheduling/power evidence. RenderDoc is used only for eventual draw/scissor/pixel
parity after a current-source executable exists. All artifacts remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Capture-before-generation; borrowed toolbar id; early template rejection; idle popup padding; close-prompt gate. | applied; focused RED 0/5 to GREEN 5/5 |
| M1 | Add one typed button route/reply and merge drag plus activation callbacks. | one route/reply/fact; exact effects |
| M2 | Move damage to transition receipts; collapse forwarding-only layers; add multi-region invalidation. | no guessed pane/full-frame fallback |
| M3 | Hard-cut bool/void callbacks and String route/action ids. | one button authority, zero compatibility shims |
| M4 | Run scale/WPR/power and RenderDoc parity matrices. | quantified acceptance and closeout |

## Validation state

- Owner source review: passed, 105/105 pre-M0 and 104/104 post-M0 Rust files.
- Direct generation, capture, route, callback/app handler, state writeback and redraw boundaries:
  read and mapped.
- Unreal pointer routing and reply sources: read and mapped.
- M0 implementation: applied. Captured release now precedes generation acquisition; toolbar damage
  borrows its id; non-primary template input rejects before wide-hit clone; overflow padding emits
  only real focus damage; non-primary close-prompt input rejects before action allocation.
- Focused static contract:
  `tools/tests/test_editor_native_pointer_button_reply_performance_contract.py`, 69 lines, 2,590
  bytes, SHA256 `403a0c6d3664e31572cbe17a75b37b132716d7bb9b19a0b650d2b0261cd0a724`;
  RED 0/5, GREEN 5/5.
- Adjacent pointer/overflow/workbench/viewport contracts: GREEN 37/37.
- Routing M0 now passes the captured generation's interaction state into button pane routing; the
  current owner fingerprint is recorded above and the combined adjacent set remains GREEN.
- Full performance-contract discovery: 168/174. The six failures are unchanged external owner
  gaps: two missing historical test sources, missing `available_slots`, two UI-root clone contracts
  and Runtime 07 evidence drift.
- `rustfmt` and scoped `git diff --check`: passed.
- Managed Rust tests, M1-M4, current-source launch, WPR and RenderDoc remain pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; raw Cargo is not an
  allowed bypass.
- This owner remains in `pending.md` until M0-M4 pass on one source/executable fingerprint.
