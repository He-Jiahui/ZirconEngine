---
title: Editor popup, dialog and binding-route generation performance review
date: 2026-08-22
module: zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/{binding_actions,showcase_actions,dialog,popup_actions,popup_frame}/**
priority: MVP-P0 editor menus, dialogs and direct input dispatch
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate direct delegates and SMenuAnchor popup ownership
---

# Goal

Compile each node's input routes once, retain popup content by open/content generation, and place an
open popup through one work-area-aware geometry owner. Stable nodes must not rescan bindings or
rebuild menu/action/drop DTOs, and popup placement must not be split across generic host conversion
without viewport collision policy.

## Reviewed source

- Rust files: 27/27
- lines: 1,163
- bytes: 37,555
- joined raw source-bytes SHA256:
  `b158e28aaa567409f7ae7010fd27b65f703c5661366e9def2c67e69c76f8c461`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

| Folder | Files | Lines | Bytes |
| --- | ---: | ---: | ---: |
| `binding_actions/**` | 3 | 73 | 2,358 |
| `dialog/**` | 3 | 99 | 3,064 |
| `popup_actions/**` | 7 | 293 | 10,113 |
| `popup_frame/**` | 6 | 337 | 9,227 |
| `showcase_actions/**` | 8 | 361 | 12,793 |

### Per-file fingerprints

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `binding_actions/mod.rs` | 7 | 170 | `2bcb0776bc8156e0b880f211be1af652075768ac4fd1908dd64e540df6c04145` |
| `binding_actions/path.rs` | 26 | 860 | `6919891225ce7ef8b0bef4f50dab11370d603661b4f8e54c39c3139f54e0af72` |
| `binding_actions/primary.rs` | 40 | 1,328 | `4218fd2b07c66c11888371b19cd0c9453ebb9fb9c72b536d701771f892b9ffd0` |
| `dialog/actions.rs` | 81 | 2,473 | `27d720c88e2633f83610b8d979a811c5ffae938af52d7f20530f9d194de56ea8` |
| `dialog/message.rs` | 13 | 449 | `b5f9ade4af50932e37d72b334a883e4b1c3809d1a5abd75c72d97eada58a5120` |
| `dialog/mod.rs` | 5 | 142 | `c9f45e6e759becfce7f3285663cfe19858a26c9da5d6226c0cf633e76a961ec8` |
| `popup_actions/action_buttons.rs` | 25 | 915 | `63ed342a520b1efdff7b42d3b0b72eb1740a1748e59dd3c5155a60c6760be0ee` |
| `popup_actions/action_ids.rs` | 67 | 2,651 | `dcb81ba47e20cc08766583ed12dfdc0be7b6aeca4d119a056a8f7df39c449c00` |
| `popup_actions/drag_payloads.rs` | 17 | 482 | `5dec0db1b194c19f3a5b82d498fe5daf822aa50130b835c364e6bb6459e0e277` |
| `popup_actions/menu.rs` | 26 | 772 | `1390e517ba5d586a945570bc815700ee86445f0a93e25026c71da59cf30de6fe` |
| `popup_actions/mod.rs` | 77 | 2,457 | `a103dd5610e63f48a73a668f73242a1b060d16801874364474f793804cb1068f` |
| `popup_actions/model.rs` | 21 | 1,029 | `01e48fafdb13ba7865251407b1dbc15510a2cd3c063972c09118255610fc575a` |
| `popup_actions/popup_state.rs` | 60 | 1,807 | `836b2747f9f6be5cab897c564946b3ed199bcd36fdd131c97f6d86fc3667f39c` |
| `popup_frame/attributes.rs` | 12 | 312 | `ead0f790e017ab68f7e83bb120ab766970000bdf94dcf436a2d5fc45ca7e6360` |
| `popup_frame/mod.rs` | 106 | 3,397 | `45ead218090504ca3f76c2bab6494fe772bd8b9f030f36f28a3ff92d305e759f` |
| `popup_frame/origin.rs` | 44 | 1,155 | `6ea21dc717690e3d25d96b26d62902f6deadd0f9270b530b16896711823b0cfb` |
| `popup_frame/overlay.rs` | 30 | 780 | `a8af893afc24223198e3f4adc84fef5503e0de49c7f18543e2d7345299816cfd` |
| `popup_frame/placement.rs` | 64 | 1,847 | `f7f09e4e8df65ae4b56e8688e8beaf62b967be13487d968e93990090016833d2` |
| `popup_frame/tests.rs` | 81 | 1,736 | `6d62f854fe76b372f32d268350b096eda0617979efd0749b54e813689ab0edfe` |
| `showcase_actions/action_buttons.rs` | 53 | 1,862 | `34fa19388daa3f0f443c3adcb33fc1d408572bb4dd3781f800921307829e4e4f` |
| `showcase_actions/binding_ids.rs` | 64 | 2,126 | `dca552199a660abf1203db6898d63986259809ccf5fdbd36dd9b0e04ac34c424` |
| `showcase_actions/commit_action.rs` | 22 | 845 | `1e6a57bda4f57503370ab44ccfd564a35937fa023e5ce2f05fa288a2406a920c` |
| `showcase_actions/drag_actions.rs` | 40 | 1,534 | `4739456f5ee70f96dc421f3da450afdb956a459a290d99fc70e4273d127c7ffd` |
| `showcase_actions/edit_action.rs` | 27 | 1,091 | `1c04b045f731674bacd7eb8e657a44109fa3a3525efff8626fbb411fe93ea8e0` |
| `showcase_actions/mod.rs` | 18 | 605 | `854c86dadfcc5252519bb8b7db59809c2a0798e9881434af8553a9d400897a51` |
| `showcase_actions/primary_action.rs` | 71 | 2,671 | `a97e90fd792343b03bbfe2bdb9ead3c90927e0ef7ad82b4af1d9bf7984b384ca` |
| `showcase_actions/tests.rs` | 66 | 2,059 | `fbc8102038857626ac4f042d4209ac7a596a7f6246b87c7077158d09e9660f9c` |

Supporting source read: `host_template_node.rs`, `template_node_data/interaction.rs`, component
descriptor/drop policy definitions and the Unreal files listed below.

## Correct foundations to retain

1. Popup positioning is constant-time and returns immediately while closed, for non-overlay roles,
   or without a complete anchor. No worker thread belongs in this UI geometry operation.
2. Dialog roles are explicitly separated from generic showcase action buttons, and primary action
   fallback order is deterministic.
3. Popup-open state may be overridden by drag-overlay state, preserving drag/drop interaction.
4. Placement parsing is bounded and allocation-free after the placement string is obtained.

## Structural findings

### P0: one node repeatedly scans the same binding slice

`host_template_node` invokes the broad popup/action projector for every projected component. Primary
Click action, Click binding, Submit fallback and Change fallback each run a separate linear search.
Showcase primary, drag begin/update/end, commit, edit and up to four action-button suffixes can add
more full searches. Exact traversals vary by control and lazy fallback, but the design is
`O(K * B)` per changed node, where `B` is bindings and `K` is requested route kinds/suffixes.

The target is not a per-node `HashMap`: small binding lists would pay allocation and hashing cost.
Bindings should be compiled once into a stable route record at template/runtime projection, with
first-by-event and known showcase routes stored directly. Consumers then read exact route slots in
O(1), and a route-generation change rebuilds the record once.

### P0: broad flat popup/action DTO work runs for every component

Every generic node projects menu rows, popup state/frame, action IDs/buttons and accepted drag
payloads before producing the wide host DTO. Most nodes do not own all these feature categories.
Empty paths are individually bounded but multiply across the complete workbench tree and every full
reprojection. This is the same flat all-feature DTO architecture identified in the typed component
generation review; M2 must use component capability/category generation, not add more conditions to
one combined function.

### P1: binding-path normalization has two duplicate allocating implementations

Primary routes and showcase routes independently implement camel/path normalization. The primary
path creates one `String` per segment, clones each again while trimming, collects a `Vec`, and joins
into another `String`. Showcase suffix conversion creates a temporary snake-case string and then a
second formatted result. M1 can hard-cut both owners to one single-output-buffer normalizer without
changing route identity.

For `S` non-empty path segments, primary normalization changes from approximately `2S` segment
string allocations plus a segment `Vec` and joined output to one final `String`. Showcase IDs change
from a snake temporary/trim copy/formatted output to one final `String`.

### P1: closed popup content and actions lack a retained generation boundary

Closed geometry is cheap, but menu strings, structured menu rows, dialog/showcase actions and drop
payload text are still materialized whenever the node is fully projected. Menu titles exist in both
legacy and structured form. A popup content generation should retain shared typed rows while closed,
materialize native content on open, and update only when content/route/open receipts change.

### P1: placement has no work-area collision/flip authority

Current placement computes anchor/origin offsets but has no viewport/work-area input, fit, flip or
clamp contract. This is primarily a correctness/ownership fault, but it can also cause avoidable
large damage or repeated corrective layout when content extends outside the window. Placement must
be owned once after desired size and current work area are known; adding scattered clamps in painter
or host conversion would create divergent geometry authorities.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Input/SButton.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Input/SMenuAnchor.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Input/SMenuAnchor.cpp`

`SButton` stores direct `FOnClicked`/pressed/released/hover delegates on the widget and executes the
bound callback; it does not rescan a generic binding list during every presentation projection.
`SMenuAnchor` owns open state, content, placement and a `FitInWindow` policy. On close it replaces
the popup child with a null widget so hidden content is excluded from prepass and hierarchy queries,
and it reports the open-state transition through a direct delegate.

The transferable invariants are compiled/direct event routes, one popup lifecycle owner, closed
content removed from expensive hierarchy work, and one fit-aware placement result. Zircon must keep
its retained IDs and data-driven templates; copying Unreal's window/menu stack is not the goal.

## Target architecture

1. Template/runtime compilation publishes `CompiledNodeRoutes` with first event routes and known
   component/showcase slots. Node projection reads slots; no generic binding scan remains.
2. `PopupPresentationGeneration` owns open/content/route/anchor/work-area/scale receipts and shared
   typed menu/dialog/action/drop rows.
3. Closed stable popups retain data identity but do not construct native content or participate in
   prepass/hit/accessibility trees. Opening consumes the retained generation once.
4. One placement service receives anchor, desired size, placement/origins, offsets, scale and work
   area, then returns fitted/flipped geometry plus the chosen placement receipt.
5. Paint, hit testing, accessibility and input dispatch consume the same popup generation and route
   IDs; no duplicate legacy label/action authority survives hard cutover.

## Instrumentation and acceptance

Measure nodes `N=100/1k/10k`, bindings `B=0/1/8/64`, route path sizes `16 B/128 B/2 KiB`, menu
rows `0/1/16/256`, closed/open and 1% route/content/anchor changes at 30/60/120 Hz.

| Evidence | Acceptance target |
| --- | --- |
| binding visits and route normalization bytes/allocations | compile once per route generation; presentation visits = 0 |
| popup/menu/action/drop rows built and bytes copied | stable closed/open generation = 0 rebuilt |
| placement calls and chosen flip/clamp | one per changed geometry receipt; fully inside work area where fit is possible |
| host/prepass/hit/accessibility visits | closed popup native content = 0 |
| process CPU/allocation/RSS/input latency/power | before/after on one current-source executable fingerprint |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add binding visits, normalization allocation/bytes, row-build and placement counters; capture matrix. | attributable baseline |
| M1 | Hard-cut duplicate route normalizers to one single-output-buffer implementation. | focused RED-to-GREEN contract and route parity |
| M2 | Compile stable typed node routes and capability-specific popup generations. | presentation binding scans = 0 |
| M3 | Retain content by open/content receipt and remove closed native hierarchy work. | stable/closed row/prepass work = 0 |
| M4 | Add one work-area-aware fit/flip placement authority and remove legacy row/action owners. | geometry and interaction parity |
| M5 | Run managed scale, input, WPR/power and RenderDoc popup/dialog parity matrix. | quantified acceptance |

## M1 implementation result

`binding_actions` now owns one preallocated output-buffer normalizer and exposes an append form to
the sibling showcase route owner. `showcase_actions` preserves its exact prefix contract but deletes
its private camel/path implementation. The implementation preserves empty input, repeated path
separators, all-punctuation normalized segments and the exact empty showcase suffix result
`ui_component_showcase.`.

Structural allocation change per normalized route with `S` non-empty path segments:

| Work | Before | After |
| --- | ---: | ---: |
| segment `String` results | `S` | 0 |
| trim-result `String` copies | up to `S` | 0 |
| segment `Vec` | 1 | 0 |
| joined/formatted final `String` | 1 | 1 preallocated output |
| normalization implementations | 2 | 1 |

Showcase suffix conversion similarly removes its temporary snake string and formatted result in
favor of the one final preallocated action ID. M1 does not reduce binding-slice traversals; compiled
route generation remains M2.

Post-M1 scope:

- Rust files: 27/27
- lines: 1,181
- bytes: 38,261
- joined raw source-bytes SHA256:
  `bd0b028e2f0544c8ee0aed5c1c6562b3b4393d600d469fd15b16d11cb25729b6`
- unchanged owner files: 23 retain the pre-M1 fingerprints above

| Changed file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `binding_actions/mod.rs` | 8 | 252 | `97b4a0cf90092574e9cb5e876598e153924e1ed657c067437fa3fba163cccdfc` |
| `binding_actions/path.rs` | 55 | 1,765 | `b411093061c47d82f8bade3d22fec868ede0400e2414bf166e5cc1fcc3a284a6` |
| `showcase_actions/binding_ids.rs` | 43 | 1,593 | `eba3fe581332fea2b35484c1b08a7d1cc5f36d77f51098efcb08b3d30260195f` |
| `showcase_actions/tests.rs` | 75 | 2,311 | `0b70972f18d5ec9c8867876b41733cb7b4aa5c16541ee87cc4a618249d71e391` |

Focused contract: `tools/tests/test_editor_popup_binding_route_normalization_contract.py`, 38
lines, 1,316 bytes, SHA256
`b27b55775a968918f3d8207d3cd34c5162fd9ab4e5bb9d5b42983c70a376c271`.

## Validation state

- Full owner source review: passed, 27/27 Rust files.
- Host conversion, descriptor/drop policy and Unreal direct delegate/menu anchor sources: read.
- M1 focused contract: RED 2/2 before the change, GREEN 2/2 after the change and after `rustfmt`.
- Current owned performance-contract set: GREEN 40/40.
- `rustfmt --check` for 27/27 owner files and scoped `git diff --check`: passed.
- Rust route-identity regressions for multiple separators, punctuation-only segments and empty
  showcase suffix are present but not claimed passing until managed Cargo is executable.
- Managed Rust behavior tests and M0 plus M2-M5 remain pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; the focused command
  was rejected before Cargo launch with `cargo_session_not_executable`.
- WPR and RenderDoc remain pending a current-source launchable editor.

The module remains in `pending.md` until M0-M5 pass on one source/executable fingerprint.
