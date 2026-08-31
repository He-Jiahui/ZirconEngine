---
title: Editor native popup dismiss generation stack performance review
date: 2026-08-23
module: zircon_editor retained-host native_popup_dismiss
priority: MVP-P0 editor popup pointer latency and coherent input generation
status: source_reviewed_m0_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate retained menu stack and focus-path dismissal
---

# Goal

Make outside-press popup dismissal a bounded query against the same committed presentation generation
already used by pointer dispatch. It must not clone a second interaction snapshot, scan all workbench
nodes, clone the wide template-node DTO, or recompute an independently versioned popup owner on every
primary press.

## Reviewed source

- owner Rust files: 3/3
- lines: 120
- bytes: 4,597
- source-only SHA256 over lexicographically sorted owner files:
  `699181b68c0035a6cc148bcecc9efc379f506bb63f361830bb2b93265edec722`
- post-M0 owner files/lines/bytes/SHA256: 3 / 125 / 4,730 /
  `168a5fc00e53bc81b808d859dcddf5cb2c889efa601683e62adb7fdac7dc0247`
- owning commit at review: `ced40579ae881b30ea9a606914f7af7073eb421a`

| Owner group | Files | Lines | Bytes |
| --- | ---: | ---: | ---: |
| `native_popup_dismiss.rs` | 1/1 | 4 | 137 |
| `native_popup_dismiss/**` | 2/2 | 116 | 4,460 |

All owner files were read in full. The primary-press overlay chain, presentation-generation accessors,
popup hit membership, popup layout, cancellation callback and damage union were inspected as direct
boundaries. The 2026-07-17 combined keyboard/popup record covered this owner at 112 lines; this record
supersedes that stale coverage for the current tree.

## Correct foundations to retain

1. Overlay dispatch gives page overflow and top-level menus precedence before workbench popup
   dismissal, then top-level chrome. This ordering is an input correctness constraint.
2. Dismiss containment distinguishes the trigger frame and popup frame, while damage is their union.
3. Closed, disabled, identity-less and row-less nodes do not produce a dismiss target.
4. Cancellation goes through the surface-control callback and clears transient hover, with bounded
   extra-damage union.
5. The primary-press caller already owns one immutable `HostPresentationGeneration`; its hit index
   records open, enabled popup rows with geometry/order membership in the same generation.

## Structural findings

### P0: the path discards its coherent generation and clones interaction state again

`dispatch_primary_press_overlays` holds `HostPresentationGeneration` but passes only `structure` into
the dismiss helper. The helper then calls `UiHostWindow::get_pane_interaction_state`, borrowing window
state again and cloning `HostPaneInteractionStateData`. Structure, hit membership and interaction are
therefore not guaranteed to be the single generation already selected for this input dispatch.

M0 now passes the generation through, borrows its structure and interaction snapshots, and consumes
its popup row slice. This removes the extra state borrow/clone and restores one-generation authority.

### P0: every eligible primary press scans N nodes and deep-clones candidates

Target discovery reverse-scans every workbench node and calls `row_data`; the current template-node
DTO has 163 public fields. The committed hit index's popup candidates are ignored, just as they were
in the pre-M0 keyboard path. Cost and clone volume scale with unrelated controls rather than open
popups.

M0 now changes discovery to O(P) borrowed probes over current-generation popup candidates. M1
publishes the top popup/dismiss geometry directly, removing stable-event discovery and geometry
reconstruction.

### P1: dismiss and keyboard reconstruct different active-popup policies

Dismiss returns only hovered/focused/selected candidates and has no fallback; keyboard keeps a
fallback. Both reconstruct popup frames independently. Paint and hit testing have their own popup row
and geometry paths. These consumers can disagree when interaction identity is absent or transient.

M1 defines a typed active-popup stack/artifact with explicit top-popup policy, trigger frame, popup
frame, damage frame and cancel binding. Keyboard, hit, dismiss and paint borrow projections from the
same arranged generation. No consumer-local cache or raw-node fallback survives the cutover.

### P1: no dismissal complexity or rebuild evidence exists

There are no counters for outside-press requests, indexed candidates, node visits/clones, popup-frame
builds, contains probes, dismiss reason, damage area or input latency. M1 adds these counters plus ETW
spans so P95/P99 and power evidence can be tied to exact popup and tree scale.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/MenuStack.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp`

Unreal stores active menus in `FMenuStack`: push inserts a menu and content mapping, and dismiss works
from a known stack index in reverse child-first order (`MenuStack.cpp:597-695`). Focus changes locate
the menu through the retained focus path/content map, dismiss descendants, or dismiss the stack when
focus leaves it (`MenuStack.cpp:771-839`). Window activation similarly resolves against the retained
stack (`MenuStack.cpp:865-881`). `FSlateApplication` owns this stack and delegates global dismissal;
it does not rediscover menus by walking and copying the full widget presentation for every event.

The transferable rule is an authoritative retained popup/menu stack attached to the UI generation.
Zircon's data-oriented implementation can use compact indices rather than widget pointers, but
dismissal must start from the known active popup owner and its arranged frames.

## Target architecture

1. `HostPresentationGeneration` owns a typed active-popup stack or shared popup artifact, ordered by
   actual popup/z precedence and versioned with structure, interaction and hit membership.
2. Each entry retains stable control/cancel identity, trigger frame, popup frame, damage frame and
   containment rules. Ordinary outside press reads only the top dismissible entry.
3. Primary-press overlay routing passes one generation through every overlay consumer; no consumer
   re-reads/clones window state during the event.
4. Keyboard, hit, dismiss and paint use shared artifact projections and one explicit fallback policy.
5. Build/rebind/fallback/dismiss reasons and query cost are observable; current-generation operation
   has no raw-node scan.

## Instrumentation and acceptance

| Evidence | Acceptance |
| --- | --- |
| outside-press requests/index candidates/node visits | O(P) at M0; O(1) top-popup query after M1 |
| state/node clones and state borrows | zero extra interaction snapshot clone; zero wide-node clone |
| popup artifact builds/rebinds/reasons | no rebuild for stable outside press |
| contains probes/dismiss reason | bounded and attributable to one top popup |
| damage area/region count | exact trigger-popup union plus caller extra damage |
| pointer CPU p50/p95/p99 | slope independent of unrelated node count |
| correctness | overlay precedence, inside/outside, disabled, hover/focus/selected and callback parity |

Matrix: presentation nodes `1/100/1K/10K`; popup depth `0/1/2/10`; press `trigger/popup/gap/outside`;
identity `hover/focus/selected/none`; update `none/interaction/geometry/close`; scale
`1x/1.5x/2x/4K`; input repeat `1/10/30/60 Hz`.

WPR owns CPU, allocation, context-switch and power evidence. RenderDoc is relevant only after a
current-source GPU presenter is launchable and only for popup draw/resource/pixel parity after M1;
it is not a pointer-query profiler. All artifacts remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Pass one generation through dismiss, reuse popup candidates and borrow nodes. | applied; static contracts GREEN, managed Rust/dynamic pending |
| M1 | Publish typed top-popup/dismiss artifact with telemetry and delete reconstructed geometry/policy. | O(1) stable query; zero artifact rebuild |
| M2 | Converge keyboard/hit/dismiss/paint popup ownership and nested stack policy. | one generation authority; no normal raw-node fallback |
| M3 | Run scale/repeat/WPR/power/UI parity and GPU popup parity where applicable. | quantified acceptance and milestone closeout |

## Validation state

- Owner source review: passed, 3/3 current Rust files.
- Primary-press, generation, hit-index, layout, callback and damage boundaries: read and mapped.
- Unreal retained stack and focus/window dismissal sources: read and mapped.
- M0 static performance contract moved RED 0/3 to GREEN 3/3. Together with keyboard, hit-index,
  presentation-generation and popup-binding contracts, the focused set passes 14/14.
- The three changed Rust boundaries pass independent `rustfmt --check`; scoped `git diff --check`
  passes with line-ending warnings only. `native_pointer/button_dispatch/primary_press.rs` is the
  changed caller boundary and is not counted in this owner's 3-file fingerprint.
- Performance-contract discovery passes 139/145. The six unrelated failures remain the known two
  missing test-support files, missing `available_slots`, preview resize `.roots.clone()`, UI-asset
  root helper `.roots.clone()` and Runtime 07 source/telemetry/owner-gate document drift.
- Managed Rust tests, current-source launch, WPR and RenderDoc remain pending because the managed
  Cargo Session is terminal `archived` with `cargo_session_not_executable`. No raw Cargo bypass is
  allowed.
- M0 dynamic acceptance and M1-M3 remain pending; this owner stays out of `review.md` until dynamic
  acceptance.
