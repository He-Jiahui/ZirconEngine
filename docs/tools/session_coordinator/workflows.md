---
related_code:
  - tools/session_coordinator/workflows/__init__.py
  - tools/session_coordinator/workflows/models.py
  - tools/session_coordinator/workflows/store.py
  - tools/session_coordinator/workflows/projections.py
  - tools/session_coordinator/models.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/sessions.py
implementation_files:
  - tools/session_coordinator/workflows/__init__.py
  - tools/session_coordinator/workflows/models.py
  - tools/session_coordinator/workflows/store.py
  - tools/session_coordinator/workflows/projections.py
  - tools/session_coordinator/models.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/sessions.py
plan_sources:
  - docs/superpowers/specs/2026-07-11-workflow-control-center-and-tray-design.md
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
tests:
  - tools/session_coordinator/tests/test_workflow_schema.py
  - tools/session_coordinator/tests/test_workflow_store.py
  - tools/session_coordinator/tests/test_workflow_projections.py
doc_type: module-detail
---

# Session Coordinator Workflow Read Model

## Responsibility

`workflows` converts coordinator-owned Session state into a stable graph-shaped read model for the control center. M1 intentionally models one workflow run per Session with one Goal node. Later milestones may add milestone, slice, validation, Failure, finalize and notification nodes without changing the identity or attempt-history rules established here.

## Persisted Model

Schema version 14 adds workflow runs, nodes, edges, attempts, artifacts and diagnostics. It also adds the web ticket/session tables used by the control plane. Enum values are constrained in SQLite so free-form workflow state cannot leak into projections.

The core enums are:

- workflow state: `registered`, `active`, `waiting_dependency`, `waiting_lease`, `resolving_failure`, `waiting_validation`, `waiting_review`, `finalizing`, `succeeded`, `failed`, `cancelled`, `stale`, `archived`;
- node state: `pending`, `ready`, `running`, `waiting_external`, `failed`, `succeeded`, `cancelled`, `skipped`;
- node kind: `goal`, `milestone`, `slice`, `validation`, `review`, `commit`, `notification`, `closeout`;
- supervision state: `starting`, `healthy`, `degraded`, `draining`, `stopping`, `offline`, `recovering`, `read_only`, `identity_mismatch`, `fatal_integrity_error`.
- artifact kind: `log`, `report`, `screenshot`, `manifest`, `plan_record`, `failure_handoff`, `fixed_handoff`, `commit`, `other`.

## Identity and Attempt Rules

- `(session_id, workflow_key)` identifies the stable workflow run for a Session.
- A node has a stable key inside its run; M1 uses the Goal key.
- Attempts are immutable and monotonically numbered per node. Database triggers reject direct update/delete operations.
- Accepting a new attempt changes the node's current-attempt pointer; it does not delete or rewrite earlier attempts.
- Projections return the current accepted attempt separately from complete ordered history.
- Artifacts and diagnostics attach to attempts, preserving evidence ownership when retries occur.
- Composite foreign keys prevent edges, diagnostics and artifacts from pointing at a node or attempt owned by another workflow run.

## Session Mapping

Registering a Session ensures its workflow and Goal node exist. Session writes invoke the workflow hook inside the same SQLite transaction, so registration, heartbeat, status changes and maintenance-driven stale/archive transitions cannot commit an unmatched projection. Heartbeat and status changes resynchronize the fallback projection:

- active execution states project to running;
- waiting lease/validation states project to waiting;
- resolving a routed Failure projects to failed or attention-required state while preserving the Session's typed status;
- completed projects to succeeded;
- cancelled projects to cancelled;
- registered, stale and archived states remain explicitly non-running rather than inventing a new Session status.

The workflow model is a derived control-center view. It must not replace `SessionStatus` as the lifecycle authority and must not add side effects to read operations. Once a node has an accepted attempt, that latest accepted attempt remains the node's current state; Session synchronization updates run metadata but does not overwrite the accepted node state.

## Projection Contract

Summary projections are bounded and suitable for the dashboard list. Detail projections add ordered nodes, edges and attempt history. The current attempt is selected only from accepted attempts; a newer rejected or incomplete attempt remains visible in history without becoming current.

## Extension Constraints

Future graph expansion must keep topology acyclic, preserve stable keys, use coordinator services for mutations and append evidence instead of overwriting it. Failure nodes must reference the canonical `failure-*` or moved `fixed-*` artifact governed by the numbered child plan; the workflow database is an index and status projection, not a second canonical Markdown copy.

## Verification

Schema tests validate migration and enum constraints. Store tests validate stable run identity and append-only attempt behavior. Projection tests validate current-attempt selection and preserved history. Server/Session regressions verify automatic synchronization at register, heartbeat and status transitions.
