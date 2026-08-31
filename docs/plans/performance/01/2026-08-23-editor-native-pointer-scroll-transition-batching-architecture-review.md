---
title: Editor native pointer scroll transition batching performance review
date: 2026-08-23
module: zircon_editor retained-host native_pointer scroll_dispatch
priority: MVP-P0 editor scrolling latency, redraw and state publication
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine Slate retained wheel-path routing and handled replies
---

# Goal

Route each wheel sample once, separate event consumption from visual mutation, and publish one typed
scroll transition per input fact. Boundary scroll and passive/unhandled targets must not repaint,
asset surfaces must not clone/publish a wide interaction state field by field, and exact changed
viewport/row regions must replace unconditional full-pane damage.

## Reviewed source

- owner Rust files: 19/19
- pre-M0 lines: 431
- pre-M0 bytes: 14,421
- pre-M0 source-only SHA256 over lexicographically sorted owner files:
  `4e5cb0bd68fe510e736e25a60da3efd4913fc0ea60dc37c2611629b86db953bb`
- post-M0 lines: 438
- post-M0 bytes: 14,687
- post-M0 source-only SHA256 over lexicographically sorted owner files:
  `c9902971a4a62607656b388d0ce46746866e688bcb7ba37b72726f38fd593cd7`
- current after routing M0 interaction integration: 19 files, 440 lines, 14,766 bytes, SHA256
  `5c815e861fcb2f2a500a4145dba21741b9ab9deea6ea0974a72c4ee8859473d8`
- owning commit at review: `7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`

Owner scope is `native_pointer/scroll_dispatch.rs + scroll_dispatch/**`. All files were read in full.
Direct callback wiring and app handlers were traced through hierarchy, welcome, console, inspector,
asset details, asset tree/content/reference bridges, interaction writeback and redraw. The current
owner had no local diff at review start.

## Correct foundations to retain

1. One committed presentation generation supplies overflow, menu and pane routing. Popup overflow
   consumes wheel events at its boundary so covered document content cannot scroll through it.
2. Menu/overflow precede pane routing; pane target dispatch is a closed enum and short-circuits after
   the matching native/asset handler.
3. Asset content already rejects equal pointer state before UI writeback. Viewport scroll delegates to
   the viewport input owner and does not request native host repaint by default.
4. Scroll bridges maintain bounded offsets and visible-row state rather than rebuilding all list rows
   in this dispatch layer.

## Structural findings

### P0: consumed or routed scroll unconditionally repaints

Page overflow returns its popup frame even when offset and hovered row are unchanged at a boundary.
Menu scroll always returns its damage frame without testing whether the callback changed menu state.
Pane scroll clones the pane frame and returns it for handled, passive and final unhandled branches;
the passive classifier performs no callback yet still repaints the full pane.

M0 compares interaction generation around menu/pane callbacks, returns idle for unchanged overflow,
and makes passive/unhandled pane routes consumed-idle. M1 replaces generation rediscovery with a typed
reply carrying `consumed`, `changed`, reason and owner ids.

### P0: one asset scroll can publish eight wide interaction states

`apply_asset_pointer_state_to_ui` writes tree/content/references/used-by hover and scroll fields using
up to eight individual setters. Each setter uses `update_pane_interaction`, which clones the complete
26-field state including four Strings, compares and may allocate a new Arc/generation. One wheel fact
can therefore create a chain of intermediate generations and repeated wide copies.

M1 introduces one typed `AssetSurfaceInteractionPatch` and applies all changed fields atomically.
The same batch contract covers hierarchy/welcome multi-field state and produces one transition
receipt for damage.

### P0: route identities allocate Strings per sample

Asset tree/content scroll clones one mode String; reference scroll clones mode and list-kind Strings.
These values encode closed activity/browser and references/used-by categories. Route construction may
already own the same strings before callback cloning.

M1 replaces them with typed compact ids in route/dispatch/callback boundaries. Strings are generated
only for diagnostics or persistence. This hard cut is shared with pointer move and routing plans.

### P0: damage is pane-sized rather than row/viewport-sized

Every non-viewport handled pane scroll returns the complete pane frame even when only a clipped list
viewport changed. No receipt identifies old/new visible range, scrollbar thumb, hover row or text
viewport. Exact damage needs the scroll surface's applied offset/range receipt and shared multi-region
transport; a local smaller rectangle would be unsupported guesswork.

### P1: scroll callbacks re-run preparation on every sample

Detail scroll handlers re-check committed layout, focus source window and surface size, then may sync
layout before handling the wheel. Asset handlers prepare pointer targets and clone string modes.
Stable size/layout should be generation-gated and preparation should resolve to a retained scroll
owner at route time. M2 measures and moves only proven stable work; ordered wheel input remains on the
UI owner rather than an unbounded worker queue.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp`

`ProcessMouseWheelOrGestureEvent` rejects zero wheel delta, locates one retained widget path and routes
through capture or the widgets under the pointer. `RouteMouseWheelOrGestureEvent` bubbles until a
handler returns a handled `FReply`; event handling does not itself imply invalidating every widget in
the path. The transferable invariants are one route path, explicit handled/mutated separation and
widget-owned scroll state. Zircon should retain its hit grid and typed surfaces, not copy Slate widget
classes or infer timing targets from source.

## Target architecture

1. Scroll routing returns a compact typed owner and a reply with consumed/changed/reason fields.
2. Each scroll surface applies one bounded offset/hover/visible-range patch and publishes one
   interaction generation only when values change.
3. Closed asset surface/list identities are typed through route, callback and state patch.
4. Damage projection uses exact clipped viewport, row and scrollbar owner frames in the shared region
   set. Boundary scroll can be consumed with zero redraw.
5. Stable target/layout generations reuse prepared bridges; preparation/rebuild reasons are measured.

## Instrumentation and acceptance

Matrix: wheel `0/small/large/precision`; samples `1/1K/1M @125/500/1000 Hz`; list rows
`0/1/100/10K/1M`; route `overflow/menu/native/asset/viewport/passive/none`; offset
`middle/min/max`; backend `GPU/softbuffer/snapshot`; scale `1x/1.5x/2x/4K`.

| Evidence | Acceptance |
| --- | --- |
| route/callback/reply consumed/changed counts | one route and reply/sample; boundary changed=0 |
| state clone/String allocation/publication counts | one atomic patch on change; stable zero |
| bridge preparation/layout sync/rebuild counts | stable generation zero rebuild |
| old/new visible range and damage regions/area | clipped changed owners, no unconditional pane frame |
| CPU p50/p95/p99, allocation, context switches, power | exact source/workload fingerprint |

WPR owns CPU/scheduling/power evidence. RenderDoc is used only for eventual GPU scissor/draw/pixel
parity. All artifacts remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Gate menu/pane redraw by interaction generation; make unchanged overflow/passive/unhandled idle. | applied; focused RED 0/4 to GREEN 4/4 |
| M1 | Add typed scroll replies, asset identities and atomic interaction patches. | one publication/change; stable/boundary zero allocation |
| M2 | Retain prepared scroll owners by generation and emit exact visible-range damage. | no stable preparation/full-pane damage |
| M3 | Hard-cut String callback ids, bool-only handlers and unconditional pane-frame APIs. | one scroll authority |
| M4 | Run scale/WPR/power and RenderDoc parity matrices. | quantified acceptance and closeout |

## Validation state

- Owner source review: passed, 19/19 current Rust files.
- Direct callback/app bridge, interaction writeback and redraw boundaries: read and mapped.
- Unreal wheel path routing source: read and mapped.
- M0 implementation: applied. Unchanged menu/pane callbacks, overflow boundary samples and
  passive/unhandled pane routes now return consumed-idle without scheduling unchanged pixels.
- Focused static contract:
  `tools/tests/test_editor_native_pointer_scroll_transition_performance_contract.py`, 60 lines,
  2,488 bytes, SHA256
  `93dd35bac00f45ef7f01ba476f84c62edf6b1b666f7ea1c1f3054cd775ccb9d9`; RED 0/4, GREEN 4/4.
- Adjacent move-generation, drag-session, damage-borrow and page-overflow contracts: GREEN 11/11.
- Routing M0 now supplies the captured generation's split interaction state to scroll pane routing;
  the current owner fingerprint is recorded above and combined adjacent contracts remain GREEN.
- `rustfmt` and scoped `git diff --check`: passed.
- Managed Rust tests, M1-M4, current-source launch, WPR and RenderDoc remain pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; raw Cargo is not an
  allowed bypass.
- This owner remains in `pending.md` until M0-M4 pass on one source/executable fingerprint.
