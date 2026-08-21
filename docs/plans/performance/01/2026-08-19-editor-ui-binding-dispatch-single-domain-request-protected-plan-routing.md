---
related_code:
  - zircon_editor/src/ui/binding_dispatch
  - zircon_editor/src/ui/host/editor_event_execution
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/49-editor-event-runtime-envelope-listener-registry-journal-replay-snapshot-dirty-lifecycle-product-integration-review.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Public/PropertyHandle.h
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/PropertyHandleImpl.cpp
---

# Protected plan routing: Editor UI binding dispatch single domain request

## Reason for routing

The main performance plan, `pending.md`, `review.md`, Optimize05, Optimize49 and numbered owner plans
are protected or foreign dirty. Broad Editor/Runtime UI source is held by active sessions. This record
routes the current 35/35-file evidence without editing those owners or creating a parallel plan
authority. Evidence source:
`2026-08-19-editor-ui-binding-dispatch-single-domain-request-architecture-review.md`.

## Requested Performance01 updates

### PERF-MVP-572

Extend the compiled binding target through final Editor dispatch. A resolved invocation must construct
one domain request directly. Animation, Asset, Draft and Inspector currently clone into disposable
host DTOs before immediate conversion to final events; Draft and Inspector later reconstruct bindings
and dispatch them a second time.

Required counters: classification passes, final request constructions, intermediate host DTO count,
clone operations and copied bytes for `1/100/10k` requests and `1 B/1 KiB/1 MiB` payloads. Stable
in-process invoke requires intermediate DTOs, native codec work and duplicate parse all `=0`.

### PERF-MVP-297 and PERF-MVP-314

Record the corrected Viewport flow: native retained pointer ingress is already typed, but execution
still remaps every `EditorViewportEvent` to `ViewportCommand`; serialized binding takes the reverse
and forward path. Replace both with one canonical typed input intent accepted by Viewport state.
Coalesce compatible motion/analog input only within the same frame, capture target and geometry/focus
generation. Press/release/cancel and resize/focus/capture barriers remain ordered.

Required matrix: `125/500/1000 Hz`, 1M mixed events, bounded queue entries/bytes/age,
event-command remap zero, complete edge/barrier parity, input-to-damage p50/p95/p99 and package-power
trace.

### PERF-MVP-329, PERF-MVP-456, PERF-MVP-489 and PERF-MVP-567

Inspector binding dispatch must consume the same compiled reflection identity, immutable inspection
generation and single resolved adapter owned by these items. It must not add a second field-path cache
or schema authority. Batch admission transfers or borrows payload once; write/readback remains one
resolved-adapter operation.

### New P0 child under Optimize05: single typed property mutation request

Add current evidence that the Inspector chain clones the complete change batch around three adjacent
representations and reconstructs a binding during execution. The target is Optimize05's typed
`PropertyEditRequest` with exact changed paths, per-target before/after values and one transaction.
Move existing preflight, active-gizmo refusal, selection context, checkpoint, post-apply sync,
rollback and undo semantics into that one owner before deleting the wrappers.

Acceptance: fields `1/100/10k`, payload `1 B/1 KiB/1 MiB`, single/multi-select and all tested failure
points; post-admission batch deep clones zero; one transaction; world, selection, draft, history and
diagnostic parity.

### Optimize49 realtime and receipt split

Attach the Viewport evidence to E-EVT-P0-04/M2. Realtime move/scroll must not visit command registry,
transaction, journal or listener registry. Only semantic low-frequency commands produce execution
receipts. Document revision advances only on successful changed commits; realtime, failure and no-op
advance zero.

### Direct P1 cleanup after cutover

Delete `AnimationHostEvent`, `AssetHostEvent`, `DraftHostEvent`, `InspectorBindingBatch`,
`apply_selection_binding()` and `apply_viewport_binding()` when tests exercise the product domain path.
Do not retain compatibility re-exports or test-only facades.

## Requested owner-plan updates

### Editor01 and Optimize49

Own the boundary between realtime input, semantic command, presentation delta, audit envelope and
committed operation. Publish one typed receipt per admitted semantic request; do not derive receipts
from global journal snapshots.

### Editor05 and Optimize05

Own the single typed Inspector/Draft property mutation request, compiled path resolution, per-target
state, checkpoint, transaction, rollback and undo. UI dispatch cannot reconstruct bindings or
maintain its own property authority.

### Editor08

Own compiled low-frequency command identity and direct domain request construction. Animation, Asset
and Docking route IDs parse/validate once per generation; dispatch transfers dynamic ownership once.

### Runtime12 and EditorUI01

Own structured realtime input admission, typed barrier/coalescing policy and direct Viewport state
application. Preserve the existing native typed pointer ingress and converge serialized/remote input
without routing motion through the command control plane.

## Requested protected index state

- `pending.md`: add or retain one concise row for `zircon_editor/src/ui/binding_dispatch/**` with
  35/35 files, 1,232 lines, production fingerprint `4df9aa1d...`, and
  `static_complete / structural_cutover_required / dynamic_pending`.
- `review.md`: do not add the module. Require single-domain-request hard cutover, Inspector rollback
  parity, realtime input split, scale counters, current-source Cargo, F4 product trace, WPR/ETW and
  package-power evidence.
- Keep protected indexes module-level and concise; detailed evidence stays in the companion review.

## Acceptance handoff

| owner | required proof |
|---|---|
| Editor05 + Optimize05 | one typed property request; batch deep-clone bytes after admission `=0`; checkpoint/rollback/undo parity at all failure points |
| Editor01 + Optimize49 | realtime move command/journal/listener/transaction visits `=0`; semantic receipt and revision rules pass |
| Editor08 | disposable host DTOs `=0`; Animation/Asset/Docking identity parsed once per generation; stale/reload/error parity |
| Runtime12 + EditorUI01 | 125/500/1000 Hz and 1M input matrix; bounded coalescing; all edges and barriers ordered |
| Performance01 | 31-run WPR/ETW, allocation, latency, RSS and package-power matrix on identical hardware/assets/settings; D/E/F artifacts |

RenderDoc is conditional on changed Viewport draw/resource/overlay output. CPU dispatch ownership is
accepted with WPR/ETW, allocation and domain counters; visible rendering changes additionally require
RenderDoc and pixel parity.
