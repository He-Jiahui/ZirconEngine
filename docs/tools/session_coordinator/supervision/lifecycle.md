---
related_code:
  - tools/session_coordinator/supervision/lifecycle.py
  - tools/session_coordinator/supervision/service.py
  - tools/session_coordinator/supervision/models.py
  - tools/session_coordinator/control_plane/actions/catalog.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/actions/service.py
  - tools/session_coordinator/migrations.py
implementation_files:
  - tools/session_coordinator/supervision/lifecycle.py
  - tools/session_coordinator/supervision/service.py
  - tools/session_coordinator/supervision/models.py
plan_sources:
  - user: 2026-07-17 normal task admission must not be blocked by coordinator draining
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
tests:
  - tools/session_coordinator/tests/test_supervision_actions.py::SupervisionActionTests::test_rollover_preserves_admission_and_unstarted_work_for_successor
  - tools/session_coordinator/tests/test_supervision_actions.py::SupervisionActionTests::test_rollover_rejects_a_live_managed_cargo_tree_without_draining
  - tools/session_coordinator/tests/test_action_catalog.py::ActionCatalogTests::test_lifecycle_parameters_are_service_scoped_and_bounded
doc_type: workflow-detail
---

# Session Coordinator Lifecycle

## Responsibility

The lifecycle service controls deliberate daemon handoffs. Ordinary stop and restart operations retain their conservative global-shutdown semantics. `service.rollover` is the narrow deployment path for loading coordinator code while work admission remains normal.

## Admission-Preserving Rollover

1. A Maintainer previews and confirms the closed-catalog `service.rollover` action.
2. The service reads managed Cargo process-tree evidence. A non-empty live PID list rejects the action with `lifecycle_rollover_live_cargo`; no process is terminated, released, or reclassified.
3. When no managed tree is live, the coordinator persists an `awaiting_restart` intent and records an audit event. Leased but unstarted jobs remain unchanged, including their command fingerprint, reservation binding, compatibility payload, target directory, and FIFO position.
4. The old fixed-port listener exits. The normal single-instance successor publishes the same loopback descriptor endpoint and resolves the existing intent to the successor instance ID.

Rollover does not transition supervision to `draining`, does not set `maintenance_hold`, and does not synthesize a global task-admission gate. Its only temporary boundary is the normal single-listener handoff.

## Recovery Rule

Startup recovery treats only an explicit `service.rollover` intent in `awaiting_restart` as a valid cross-instance action. Other executing actions from an older daemon remain interrupted and fail closed. This keeps the exception bounded to durable lifecycle handoff evidence rather than weakening general action identity checks.

## Validation

The focused regression suite proves both sides of the contract: an empty live-process window preserves an unstarted lease and remains healthy through successor reconciliation; a real managed PID tree produces a recorded failure without drain, hold, process termination, or lease mutation.
