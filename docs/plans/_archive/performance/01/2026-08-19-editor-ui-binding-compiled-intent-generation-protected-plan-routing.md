---
related_code:
  - zircon_editor/src/ui/binding
  - zircon_editor/src/ui/binding_dispatch
  - zircon_editor/src/ui/retained_host/callback_dispatch
  - zircon_runtime_interface/src/ui/binding
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandInfo.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/UICommandList.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
---

# Protected plan routing: Editor UI compiled binding intent generation

## Reason for routing

The main performance plan, `review.md`, `pending.md`, Optimize08/Runtime UI 11A and numbered owner
plans are protected or foreign dirty. Broad Runtime/Editor UI source is leased by the active `mvp00`
session. This record routes the current 41/41-file evidence without overwriting those owners. Evidence
source: `2026-08-19-editor-ui-binding-compiled-intent-generation-architecture-review.md`.

## Requested Performance01 updates

### PERF-MVP-572

Extend typed-route ownership through the Editor-side codec:

- `EditorUiBindingPayload::from_call()` deep-clones the recursive call once for each of seven command
  families; Viewport and fallback symbols pay all seven probes;
- non-operation `with_arguments()` encodes the old typed payload, discards its arguments, and decodes
  the replacement through the same seven-family chain;
- reflection, route registration and diagnostics still call `native_binding()` in production;
- 21 external tests emphasize native text roundtrips but do not constrain allocations or probe count.

Required target: `CompiledEditorBindingGeneration` plus dense `UiRouteId`, immutable shared static
arguments and typed dynamic slots. In-process invoke must record zero native formatting/parsing,
family probes and full binding clones. External codecs classify once and enforce bytes/args/nodes/depth
budgets before recursive allocation.

### PERF-MVP-297

Add the corrected Editor Viewport evidence. `PointerMoved`, `Scrolled`, `Resized` and button edges are
available through the same serialized `ViewportCommand` binding family and worst-position decoder as
low-frequency toolbar commands, including reflection default routes. The native retained pointer path
already maps `UiPointerRoute` directly to `EditorViewportEvent` and does not pay that decoder.
Runtime12/EditorUI01 must preserve this typed path, converge remote/template input on the same receipt
authority and use frame-local, capture-aware coalescing. Press/release/cancel, focus/capture changes
and resize/scale geometry barriers cannot be coalesced away or reordered.

Required matrix: 125/500/1000 Hz and 1M mixed events; named symbol/native String allocation zero;
bounded queue entries/bytes/age; complete edge/barrier parity; input-to-effect p95 and power trace.

### PERF-MVP-076 and PERF-MVP-099

Reflection and stable Workbench generations must consume compiled binding handles. They must not call
`as_ui_binding()` plus `native_binding()` for every menu/action during a stable refresh. Native detail
is generated lazily for explicit remote/diagnostic demand and cached by binding generation.

### New P0 child item: compiled Editor binding generation

Create one Editor08/Runtime09 child item that owns command identity, typed schema, static/dynamic
argument separation, owner lease, authorization/provenance, route publication and hard retirement.
This is the Editor-side prerequisite for PERF-MVP-572; it must reuse the invocation gateway and
Runtime UI route generation rather than create another router.

### Direct P1 cleanup after cutover

Delete `EditorUiRouter` after its tests migrate to the product gateway. It is currently a test-only
`BTreeMap<UiEventPath, Vec<handler>>` with allocating dispatch and no owner/unregister/generation
contract. Do not preserve it as a compatibility facade.

## Requested owner-plan updates

### Editor08

Own `CompiledEditorBindingGeneration`, stable command identity, schema, authorization/provenance and
owner retirement. Menu, palette, template, remote automation and UI actions resolve through the same
invocation gateway. Native binding text remains a bounded external representation.

### Runtime09

Own dense route publication, generation validation and external binding codec budgets. Stable local
invoke accepts a route handle plus typed dynamic slots and never formats/parses native text. Plugin
unload and template reload publish explicit retirement receipts.

### Runtime12 and EditorUI01

Own the high-frequency structured input lane, including the existing direct native
`UiPointerRoute -> EditorViewportEvent` path and convergence of remote/template input. Coalesce only
compatible motion/analog values inside a frame and capture target; preserve all edges and
geometry/focus barriers. Low-frequency resolved actions may cross into Editor08 command intent after
input routing, not before.

### EditorUI08

Compile template and Workbench control bindings once per generation. Dynamic callbacks patch typed
slots or create the final domain event directly; they must not encode the previous typed payload and
re-run generic decoding per event.

### Optimize08 and Runtime UI 11A

Optimize08 retains command authority, policy, context and plugin lifecycle. Runtime UI 11A retains
single UI service, product input dispatch and host receipt semantics. Performance01 supplies clone,
allocation, rate, latency and power gates; no second Editor-only runtime UI authority is introduced.

## Requested protected index state

- `pending.md`: add or retain one concise row for `zircon_editor/src/ui/binding/**` with 41/41 files,
  1,902 lines, 3 in-module tests, fingerprint `6ac47f49...`, and
  `static_complete / structural_cutover_required / dynamic_pending`.
- `review.md`: do not add the module. Require compiled generation/route cutover, typed input split,
  owner retirement, bounded external codec, scale counters, F0/F4, WPR/ETW and power evidence.
- Keep this routing record concise in protected indexes; detailed findings stay in the companion
  architecture review.

## Acceptance handoff

| owner | required proof |
|---|---|
| Editor08 | one compiled binding/command generation; static arguments shared; policy/provenance parity; atomic owner revoke |
| Runtime09 | route handle direct lookup; in-process native String/parse/family probe/full clone `=0`; bounded external codec |
| Runtime12 + EditorUI01 | 125/500/1000 Hz and 1M mixed input; bounded coalescing; complete edge/capture/focus/geometry ordering |
| EditorUI08 | stable template/Workbench generation callback allocation `=0` except owned dynamic data; no typed-generic-typed roundtrip |
| Performance01 | 31-run WPR/ETW, allocation, latency, RSS and package-power matrix on identical hardware/assets/settings; D/E/F artifacts |

RenderDoc is conditional: require it only when submitted UI/viewport geometry, resource generations or
visible output changes. CPU binding and input ownership is accepted with WPR/ETW plus allocator and
domain counters.
