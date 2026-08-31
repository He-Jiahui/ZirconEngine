---
title: Editor template typed node popup and activation performance review
date: 2026-08-23
module: zircon_editor retained-host template geometry/component/input/activation/popup foundations
priority: MVP-P0 editor paint dispatch and input generation
status: source_reviewed_structural_cut_pending_dynamic_pending
reference_engine: Unreal Engine Slate typed widget paint/input and centralized popup placement
---

# Goal

Classify a retained template node once when its presentation generation is built, then use the same
typed descriptor for painting, hit results, input activation and popup layout. Stable frames must not
probe a long string predicate chain per visible node, and keyboard/pointer popup handling must not
reconstruct row geometry or navigation models per event.

## Reviewed source

- owner Rust files: 34/34
- lines: 1,333
- bytes: 46,421
- source-only SHA256 over lexicographically sorted owner files:
  `33ae5f4851fde651b98319c26df8792ad1cd481ec1b442e5f91e6b35bd9d071a`
- owning commit at review: `0d70d1ac6499abcf56c3f6c3ef43cb3a7502a249`

| Owner group | Files | Lines | Bytes |
| --- | ---: | ---: | ---: |
| `frame_geometry.rs + frame_geometry/**` | 5/5 | 87 | 2,667 |
| `template_geometry.rs + tests + template_geometry/**` | 4/4 | 112 | 3,162 |
| `template_component_family.rs + tests + template_component_family/**` | 8/8 | 320 | 12,388 |
| `template_input_semantics.rs + tests + template_input_semantics/**` | 4/4 | 95 | 3,479 |
| `template_activation_semantics.rs + tests + template_activation_semantics/**` | 6/6 | 373 | 13,764 |
| `template_popup_layout.rs + tests + template_popup_layout/**` | 7/7 | 346 | 10,961 |

All owner files were read in full. Direct consumers were inspected through template command dispatch,
surface hit construction/results, native pointer activation, popup dismissal and native keyboard popup
target discovery. `TemplatePaneNodeData` generation shape was inspected as the upstream contract. The
cited Unreal widget, input and popup sources were read directly. Supporting files are not counted as
owner coverage.

## Correct foundations to retain

1. Frame containment, visibility, union and template-frame projection are allocation-free O(1)
   helpers. Template bounds is a single borrowed row pass and does not clone node models.
2. Popup positioning is centralized and O(1): it chooses the larger available vertical side, clamps
   height and horizontal bounds and preserves a shared anchor-gap token. Paint, hit and keyboard paths
   call the same geometry helpers, which protects parity.
3. Uniform popup rows derive a constant row height and are compatible with the indexed two-candidate
   boundary policy reviewed in `surface_hit_test/**`.
4. Hit results already carry `TemplateComponentFamily`; input focus and primary activation reuse that
   typed value rather than reclassifying the hit from control id.
5. Disabled input is rejected before activation, typed table-row identity is dispatched before generic
   actions, and popup option/menu routes cannot accidentally inherit text-input focus.

## Structural findings

### P0: flat string DTO forces sequential paint probing instead of one typed dispatch

`TemplatePaneNodeData` has 163 public fields in one 179-line structure. Component identity is spread
across `component_role`, host `role`, category/layout, variants and control-id conventions. The shared
classifier tries declared role, host role, category/layout and then workbench control-id prefixes or
substrings.

For every visible node, `push_specialized_template_node_commands` sequentially tries five primary
painters, the dropdown painter and as many as 22 secondary painters before fallback. Individual
painters then repeat their own string identity checks; at least 11 current family predicates and 10
workbench-visual-language calls feed that chain. A late or fallback node can therefore execute close
to 28 painter predicates, with repeated prefix/substring and component-family classification, every
paint generation.

M1 compiles one `TemplateNodeDescriptor` when the host presentation generation is built. It contains a
typed paint opcode/component kind, visual language, input role, popup role and stable action/binding
identity. Paint uses one enum match/direct function table entry; it does not scan painters or classify
strings. This is an architectural replacement, not a reordering of the current predicate chain.

### P0: popup layout/navigation is recomputed separately for paint, hit and every keyboard event

The geometry helpers are shared, but their results are not retained. Paint iterates rows and computes
frames; surface hit indexing computes rows again; native keyboard target creation recomputes popup and
row frames and materializes a new `Vec<PopupKeyboardRow>` with cloned ids/text on each event.
`active_popup_keyboard_target` first checks that the generation hit index has popup rows, then ignores
those row indices and reverse-scans the full workbench node model using owned `row_data` until it finds
the active popup.

M2 publishes `HostPopupLayoutArtifact` per open popup: popup frame, visible row range, row extent,
typed row identities/actions, focus/select state and generation. Paint, hit, dismissal and keyboard
navigation borrow this artifact. Stable key repeat visits the active popup/row set only and allocates
nothing after warm-up. The native keyboard owner must hard-cut its full-node discovery and per-event
row vector, not add another cache beside the generation artifact.

### P1: component and activation semantics remain string protocols after projection

Workbench classification uses broad rules such as substring `workbench`, `IconButton`, `Segmented`
and `Button`; activation matches dispatch-kind strings and reparses asset dispatch source once to pick
the route and again to produce the asset activation. These rules are deterministic but distributed:
adding a component/action requires paint, hit and activation owners to keep string conventions in
sync.

M1 validates strings once at projection/compile time and stores typed enums. Unknown or ambiguous
identity becomes a generation diagnostic with source node id, rather than silently reaching a generic
fallback on every frame or click. String ids remain serialization/debug data, not hot-path dispatch.

### P1: template bounds and family are recomputed by multiple generation artifacts

Workbench hit-index construction and each paint-index construction separately call
`template_nodes_bounds`. Surface dispatchability computes component family, and surface-node metadata
can compute it again for nodes without an explicit role; later hit construction computes it again.
Each individual pass is linear and borrowed, but the same immutable model pays repeated scans and
classification because it lacks one arranged generation descriptor.

M1 stores bounds and the typed descriptor beside each model generation. Hit, paint and surface
projections borrow them and report cache/build/reuse counters. No process-global cache keyed only by
address is allowed; identity and invalidation belong to the presentation generation.

### P2: local helper changes would entrench the wrong boundary

The reviewed geometry and popup arithmetic are already constant-time and test-covered. Changing row
height, clamp rules, string match order or returning more borrowed strings would not remove the P0
work, and could break paint/hit/keyboard parity without current-source dynamic tests. No isolated M0
code change is accepted for this owner. Instrumentation and typed-generation cutover precede
optimization claims.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SWidget.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Input/SButton.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Layout/LayoutUtils.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Input/SMenuAnchor.cpp`

Slate's non-virtual `SWidget::Paint` establishes clipping, persistent state, hit-grid membership and
instrumentation, then invokes the concrete widget's virtual `OnPaint` once
(`SWidget.cpp:1480-1658`; `SWidget.h:1765-1788`). `SButton::OnPaint` and mouse handlers implement the
typed button behavior directly (`SButton.cpp:171-192,351-428`). Input enters virtual widget handlers
and returns `FReply`; it is not redispatched by testing a list of component-name strings
(`SWidget.h:381-415`; `SWidget.cpp:450-488`).

Popup fitting is centralized in `ComputePopupFitInRect`, including orientation flip and work-area
clamp (`LayoutUtils.cpp:61-154`). `SMenuAnchor` owns the popup/menu lifetime, prepasses content to get
one desired size and asks `CalculatePopupWindowPosition` for placement before publishing the window
(`SMenuAnchor.cpp:535-586`). The transferable rule is typed per-widget behavior plus one retained popup
owner/layout artifact, not Unreal's class hierarchy or exact dimensions.

## Target architecture

1. Projection validates string metadata and emits immutable `TemplateNodeDescriptor` values keyed by
   model generation and row/node id.
2. Descriptor fields include `TemplatePaintOpcode`, `TemplateComponentFamily`, visual language,
   `TemplateInputRole`, typed activation route and `TemplatePopupRole`.
3. Painting performs one typed dispatch per selected row. Specialized painters receive the descriptor
   and never rediscover node kind from strings.
4. Surface construction, hit receipts and activation borrow the same descriptor; hit receipts carry a
   compact typed route payload, not a second partial interpretation.
5. Open popups publish one `HostPopupLayoutArtifact` shared by paint, hit, dismissal and keyboard
   navigation. Current-generation event paths never scan all nodes or rebuild row vectors.
6. Unknown descriptors and slow fallback are explicit counters/reason codes and fail plan acceptance
   if used by MVP editor controls.

## Instrumentation and acceptance

| Evidence | Acceptance |
| --- | --- |
| descriptor build/reuse CPU/bytes | once per changed model generation; stable generation zero |
| painter predicates per painted node | exactly one typed dispatch; no sequential probe chain |
| component/visual/input/activation string classifications | zero in paint/hit/event hot paths |
| unknown/fallback descriptor count | zero for MVP engine/editor/plugin controls |
| popup layout builds/reuses | once per popup generation; stable frame/key repeat zero builds |
| keyboard node/row visits and allocations | active popup visible/eligible rows only; zero warm allocation |
| paint/hit/keyboard popup geometry | exact shared artifact identity and row parity |
| bounds/family scans | one arranged-model build, then borrowed by all projections |

Matrix: nodes `1/100/1K/10K`; component kind `early/late/fallback/unknown`; pane/plugin models
`1/10/100`; popup rows `0/1/20/1K/10K`; input `pointer/key/key-repeat`; popup placement
`below/above/clamped/offscreen/projected`; state `disabled/focused/selected/separator`; update
`semantic/layout/model`; damage `single/multi/full`; scale `1x/1.5x/2x/4K`.

WPR owns CPU, allocation, context-switch and power evidence. RenderDoc is needed only for GPU
draw/resource/pixel parity after a current-source presenter is launchable. Artifacts stay on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add descriptor/painter-probe/popup-build/keyboard-visit allocation telemetry without changing semantics. | source and workload fingerprints; no false performance claim |
| M1 | Compile typed node descriptors and replace sequential painter/string dispatch. | one typed dispatch; zero hot-path classification |
| M2 | Publish shared popup layout/navigation artifact and cut over paint/hit/dismiss/keyboard. | zero stable recomputation/allocation; exact parity |
| M3 | Hard-delete workbench control-id family fallback and current-generation popup full scans; run scale/WPR/UI matrix. | MVP fallback count zero and dynamic acceptance |

## Validation state

- Owner source review: passed, 34/34 Rust files.
- Paint dispatch, surface hit, pointer activation, popup dismissal, keyboard discovery and the 163-field
  upstream node DTO: read and mapped as supporting consumers/contracts.
- Unreal typed paint/input and centralized popup placement: read and mapped.
- Existing local unit sources cover frame visibility/union, declared and fallback component families,
  text-input focus exclusions, activation routes, table identity and popup flip/clamp/row parity. They
  are source evidence only in this session.
- No local code change was made: the defensible optimization boundary is generation compilation and
  shared popup ownership, not an isolated helper micro-change.
- Managed Rust tests, current-source launch, WPR and RenderDoc remain pending because the managed Cargo
  Session is terminal `archived` with `cargo_session_not_executable`. No raw Cargo bypass or dynamic
  latency/power claim is permitted.
- M0-M3 remain pending; all six owner groups stay in `pending.md` until dynamic acceptance.
