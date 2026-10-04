---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-28-runtime-ui-surface-input-publication-authority.md
  - docs/plans/astra/performance/01-bounded-hotpaths.md
---

# Runtime UI Input Routing Contract Alignment

## Confirmed Cause

The Runtime UI input-routing hot path was extracted from
`dynamic_api/session/runtime_ui.rs` into the private
`runtime_ui/input_routing.rs` owner. The production parent still mounts that
child and owns surface state plus publication calls, but the pressure-model
source contract still required routing anchors in the old parent file. The
contract therefore failed even though the unrouted mouse-motion rejection,
focused and navigation direct routing, published pointer query, typed rejected
pointer path, and reverse fallback fanout remain present in the child owner.

## Plan Completion List

| Batch | Work | Status | Validation Evidence |
|---|---|---|---|
| RUI-01 | Rebind Runtime UI input-publication pressure contracts to the parent state/publication owner and the private input-routing owner | implemented_pending_validation | The initial Runtime/Editor static batch found this one owner-drift failure after 94 of 95 tests passed. After the owner correction, the same 95-test Runtime/Editor batch passed, along with Python syntax and scoped diff checks. A single coordinator static ticket must still establish managed current evidence. No Rust production route, Cargo command, or release benchmark is claimed by this record. |

## Validation Boundary

This is a test-contract repair required by the Runtime module split. It does
not add coordinator behavior, build tooling, or a new performance model. The
existing model remains deterministic algorithm pressure only; Windows managed
Cargo and the real release p50/p95/p99 product workload remain pending.

## Coordinator Dispatch Log

The focused Runtime UI plus Editor03 static submission request
`astra-runtime-ui-editor03-static-contract-20260910-r1` was rejected before a
ticket was issued with `Validation overlay paths require current Session
attribution`. Its exact manifest correctly included the unowned Runtime routing
overlay, so this session did not claim or attribute another session's source
just to make the ticket admissible. No ticket status was queried. A later
combined submission must be made by the source owner or after a legitimate
cross-session handoff.
