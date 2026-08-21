---
related_code:
  - zircon_editor/src/ui/binding
  - zircon_editor/src/ui/binding_dispatch/editor_event_normalization.rs
  - zircon_editor/src/ui/host/editor_event_control_requests.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/reflection/adapter.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch
  - zircon_runtime_interface/src/ui/binding
tests:
  - zircon_editor/src/tests/ui/binding
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandInfo.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/UICommandInfo.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandList.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/UICommandList.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/SlateApplication.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
doc_type: current-architecture-performance-review
status: static_complete_structural_cutover_required_dynamic_pending
created_at: 2026-08-19
---

# Editor UI binding compiled-intent generation architecture review

## Status

- Result: `static_complete / structural_cutover_required / dynamic_pending`.
- MVP priority: P0 for high-frequency viewport input and in-process binding dispatch; P1 for bounded
  external parsing and deletion of the unused parallel router.
- Accounting: retain `zircon_editor/src/ui/binding/**` in `pending.md`. Do not add it to `review.md`
  before the compiled-intent cutover, input-storm matrix and F4 product trace pass.
- Code disposition: no Rust source changed. The relevant Editor/Runtime UI scopes are leased by
  another session, and the required correction crosses binding, input, route and host ownership.

## Exact scope

| scope | files | physical lines | in-module tests | raw bytes | ordered path-and-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/ui/binding/**` | 41/41 | 1,902 | 3 | 75,422 | `6ac47f49fe0426628f9c062df5744cc7167a41e638faf286d447b41417d88afb` |
| external binding tests | 6/6 | 794 | 21 | 28,446 | supporting behavior coverage; not included in the production fingerprint |

The fingerprint is SHA256 over sorted normalized path, NUL, raw bytes, NUL. All 41 current Rust
files and the 21 external binding tests were read in full. Nine production files contain foreign
rustfmt import/re-export ordering changes, totalling 10 inserted and 10 deleted lines; the fingerprint
records the current worktree rather than pretending those files are clean.

## Module acceptance record

| module | files/lines | current-source performance verdict |
|---|---:|---|
| `animation` | 3 / 452 | Typed commands are useful, but every conversion allocates an owned symbol and cloned string arguments. The largest codec repeats the same generic-call encode/decode model. |
| `asset` | 3 / 94 | Small and typed; locator strings are still copied into and out of the generic ABI for in-process dispatch. |
| `core` | 13 / 529 | P0 owner of seven sequential deep-clone probes, typed-to-generic-to-typed rebinding and an unused second router. Native formatting is exposed as a normal method instead of an external-boundary operation. |
| `dock` | 3 / 156 | Stable typed variants exist, but slot, instance, page and preset strings are rebuilt through generic calls. Lifecycle retirement belongs to the compiled route generation. |
| `draft` | 3 / 72 | Dynamic argument injection performs the full roundtrip, then mutates the control ID after decoding. The field/control mapping must be compiled once. |
| `selection` | 3 / 39 | Compact payload; no independent algorithmic hotspot after the shared conversion path is removed. |
| `viewport` | 9 / 466 | Pointer move, scroll, resize and button edges are exposed in the same binding family as low-frequency toolbar commands, and binding/remote-route decoding reaches that family last. The native retained pointer path was separately verified to produce `EditorViewportEvent` directly, so it does not pay this decoder; its later event-to-command conversion is owned by the companion dispatch review. |
| `welcome` | 3 / 77 | Low frequency but still subject to the shared allocation path; no separate MVP hotspot. |
| root `mod.rs` | 1 / 17 | Re-export surface only. |

## Structural bottlenecks

### P0: payload decoding deep-clones the complete call up to seven times

`EditorUiBindingPayload::from_call()` asks `Animation`, `Selection`, `Asset`, `Welcome`, `Draft`,
`Dock` and finally `Viewport` to decode in sequence. Every probe receives `call.clone()`. A viewport
or custom call therefore deep-clones its symbol and recursive argument vector seven times before it is
accepted or returned. `MenuAction`, `EditorCommand`, `EditorOperation` and `InspectorFieldBatch` also
pay all seven failed probes before their direct match.

This is a dispatch algorithm defect, not a reason to reorder variants. Moving Viewport first would
only move the cost to another family and preserve O(families * payload bytes). The external parser
must classify the symbol once, then dispatch to exactly one decoder. In-process calls must not enter
this decoder at all.

### P0: dynamic arguments rebuild typed state through a disposable generic call

For every non-`EditorOperation` payload, `with_arguments()` performs these steps:

1. clone the event path and encode the existing typed payload into a new `UiEventBinding`;
2. allocate the old symbol/arguments, then immediately discard the old arguments;
3. install dynamic arguments and call `from_ui_binding()`;
4. run the seven-family probe chain and allocate typed string/nested-array fields again;
5. for inspector draft fields, rewrite the control ID after the roundtrip.

The path is used by retained asset, inspector, pane, viewport and welcome controls, builtin template
bindings and the Workbench template bridge. It means a registered immutable binding template is not
actually reused: every interaction reconstructs and reclassifies it. Static route arguments and
dynamic event values need separate storage in one compiled descriptor.

### P1: an alternate serialized route duplicates the typed pointer path

`ViewportCommand` combines `PointerMoved`, `Scrolled`, `Resized` and button edges with toolbar actions
such as projection, grid, snap and overlay selection. Reflection also publishes default commands for
pointer routes, so control/binding invocation can reach the serialized family. However, the native
retained-host pointer callback already maps `UiPointerRoute` directly to `EditorViewportEvent`; it
does not call `with_arguments()` or the seven-family decoder. This review therefore does not claim
that normal mouse movement currently pays native binding formatting or seven deep clones.

The correct cutover preserves that typed native lane and prevents remote/template routes from creating
a second input authority. Runtime12 owns edge-preserving typed accumulation; Editor08 owns
low-frequency compiled command identity. Pointer move/analog deltas may be coalesced within one frame
and one capture target, while press/release/cancel, resize barriers and route/capture changes remain
ordered. Neither lane should format or parse native binding text during normal operation.

### P1: native text formatting leaks into reflection, diagnostics and route registration

`native_binding()` first builds another owned `UiEventBinding`, then formats path, symbol and all
arguments. Production callers use it in unsupported-event errors, Workbench reflection, retained-host
automation diagnostics and route lookup/registration. Runtime-interface PERF-MVP-572 already records
the downstream native-string hash lookup and additional result clones.

The fix is one typed `UiRouteId`/command handle published with a compiled generation. Native text is a
bounded authoring, persistence, automation interchange or error-detail representation. Reflection
may expose it lazily on explicit demand; stable reflection and event dispatch must record zero native
format/escape bytes.

### P1: `EditorUiRouter` is a test-only parallel routing surface

The generic router stores `BTreeMap<UiEventPath, Vec<Box<handler>>>` and allocates a result vector on
dispatch. Production searches found only tests using it; the actual product uses runtime UI route IDs
and host dispatch. Keeping a second public router creates an attractive but incomplete path with no
owner lease, unregister, generation or authorization. Remove it after tests migrate to the product
gateway, unless a current production owner proves a distinct requirement.

### P1: tests reward roundtrip text but do not constrain the hot-path algorithm

The 21 external tests primarily assert native string equality, parse/format roundtrips, typed payload
selection and one headless `EditorUiRouter` dispatch. They do not count call clones, family probes,
native-format bytes, allocations, high-rate move/scroll traffic, payload-depth limits, route unload or
edge ordering. The present suite can stay green while the P0 cost scales with event rate and payload
size.

## Reference-engine evidence

- Unreal `UICommandInfo.h` represents command name and binding context with `FName`, publishes shared
  `FUICommandInfo`, and defines explicit unregister. `UICommandInfo.cpp` registers/removes that shared
  identity through `FInputBindingManager`; it does not parse display text per invocation.
- Unreal `UICommandList.h/.cpp` maps shared command identities to action delegates and provides paired
  map/unmap. Zircon should adopt stable identity, owner retirement and direct lookup, while retaining
  Rust types and avoiding Unreal's recursive parent/child command-list search as a global hot path.
- Unreal `SlateApplication.h:1788-1810` defines direct routing of move and wheel/gesture events to a
  widget path with structured `FPointerEvent` data. `SlateApplication.cpp:5761-6055` transforms the
  pointer event once, maintains capture/drag/user state and invokes `OnMouseMove`/`OnTouchMoved`
  directly. This supports separating high-frequency input from command serialization.

These sources establish responsibility and data-flow shape, not a claim that Zircon must reproduce
Unreal APIs or already has comparable timings. Same-hardware WPR/ETW, allocation and package-power
evidence remains mandatory.

## Required architecture cutover

1. Editor08 publishes an immutable `CompiledEditorBindingGeneration`. Each entry has stable typed
   identity, owner generation, authorization/provenance policy, event kind, payload schema and shared
   static arguments. Plugin unload atomically retires its handles and all derived routes.
2. Runtime09 resolves a descriptor to dense `UiRouteId` once per generation. Local dispatch carries
   the handle plus compact dynamic slots; it performs no symbol formatting, parsing, family probing or
   binding clone. Remote/native input validates budgets before allocation, classifies one symbol once
   and resolves to the same gateway.
3. Runtime12 and EditorUI01 retain the existing direct pointer-to-`EditorViewportEvent` route and make
   remote/template input use the same typed receipt authority. Coalescing is capture-, target- and
   barrier-aware; it never drops press/release/cancel or crosses geometry and focus generations. Only
   a resolved low-frequency action becomes an Editor command intent.
4. Template and Workbench generations bind immutable descriptors once. Per-event data patches typed
   dynamic slots or constructs the final domain event directly; it never encodes an old payload merely
   to discard its arguments.
5. Native binding text becomes an explicit bounded codec. Enforce bytes, argument count, recursive
   nodes/depth and string limits before recursive allocation. Reflection/automation request native
   detail lazily and cache it by binding generation when genuinely required.
6. Remove `EditorUiRouter` and string-key product routes after callers and tests use the unified
   invocation gateway. No compatibility facade or dual authority remains after cutover.

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| decode | seven families, routes `1/100/10k`, args `0/1/100`, nesting `max/max+1` | exactly one external family classification; in-process family probes/call clones/native parse `=0`; over-budget input rejected before recursive allocation |
| dynamic binding | template controls `1/100/10k`, static args `0/10/100`, dynamic slots `0/1/10` | old-payload encode/drop `=0`; stable descriptor clone `=0`; allocation and copied bytes proportional only to owned dynamic payload |
| input storm | native plus remote/template pointer/scroll `125/500/1000 Hz`, 1M mixed events, capture/focus/resize barriers | all routes converge on typed receipts; symbol/native String allocation `=0`; latest/delta coalescing bounded; every press/release/cancel and barrier preserves order; queue entries/bytes/age bounded |
| generation lifecycle | owners `1/100`, reload/unload `1/10k`, routes `1/100/10k` | registration once/generation; direct lookup near O(1); stale handle rejected; owner revoke removes command/route/template atomically; leaked route/handler `=0` |
| product | F0 startup and F4 editor, cold/warm/idle/storm, 31 runs | WPR/ETW CPU, waits, wakeups, allocations, RSS, input-to-effect p95 and package power with identical hardware/assets/settings; artifacts stay on D/E/F |

RenderDoc is not the primary tool for binding/input CPU ownership. It becomes required only if the
cutover changes viewport/UI submitted geometry, resource generations or visible output; then capture
draw/event/resource and pixel parity. WPR/ETW and allocator counters own this module's dynamic proof.

## Static gates executed

- Read 41/41 production Rust files and all 6/6 external binding test files; reproduced 75,422 bytes,
  1,902 production lines, 24 total relevant tests and fingerprint `6ac47f49...`.
- Confirmed seven sequential `UiBindingCall` clones, the typed-to-generic-to-typed `with_arguments()`
  roundtrip, Viewport binding's last decoder position and production native-string callers. Also
  confirmed the native retained pointer path bypasses this decoder through direct
  `UiPointerRoute -> EditorViewportEvent` mapping.
- Confirmed `EditorUiRouter` has test callers but no production caller in `zircon_editor/src`.
- Read the cited Unreal command/input primary sources and current Optimize08/Runtime UI 11A reports.
- `rustfmt --edition 2021 --check` passed for 41/41 production files. Scoped `git diff --check`,
  24/24 routed-path existence and
  `python -m tools.session_coordinator --repo-root . --json plan audit` passed. The fingerprint was
  recomputed after documentation edits and remains `6ac47f49...`.
- The Editor UI asset-binding performance contract passed 4/4 through its native `unittest` entry.
  The Runtime12 input structure audit test passed its audit collection but failed 0/1 at stale guard
  expectations: the audit now reports 20 runtime modules and 25 behavior anchors with every
  `missing_*` list empty and `risks=[]`, while the test still asserts 18 and 21. Runtime12 owns that
  guard update; this review does not edit its protected test.
- The documentation convention gate reports 0 violations owned by these two records. During this
  run the unrelated global baseline moved from 692 violations across 242 documents to 693 across 243
  of 2,711 documents; concurrent foreign documentation was not edited.
- Dynamic Cargo, allocation counters, F0/F4 launch, WPR/ETW, input-storm, package-power and RenderDoc
  parity evidence remain pending. This is not an accepted milestone, so no commit or WeCom
  notification is due.
