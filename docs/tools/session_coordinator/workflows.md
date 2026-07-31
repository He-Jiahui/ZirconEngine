---
related_code:
  - tools/session_coordinator/workflows/__init__.py
  - tools/session_coordinator/workflows/plan_import.py
  - tools/session_coordinator/workflows/models.py
  - tools/session_coordinator/workflows/store.py
  - tools/session_coordinator/workflows/projections.py
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/workflows/failure_closeouts.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/actions/models.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/models.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/sessions.py
implementation_files:
  - tools/session_coordinator/workflows/__init__.py
  - tools/session_coordinator/workflows/plan_import.py
  - tools/session_coordinator/workflows/models.py
  - tools/session_coordinator/workflows/store.py
  - tools/session_coordinator/workflows/projections.py
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/workflows/failure_closeouts.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/actions/models.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/git_finalize.py
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
  - tools/session_coordinator/tests/test_workflow_topology.py
  - tools/session_coordinator/tests/test_workflow_commit.py
  - tools/session_coordinator/tests/test_failure_closeout.py
  - tools/session_coordinator/tests/test_action_execution.py
  - tools/session_coordinator/tests/test_milestone_cli.py
  - tools/session_coordinator/tests/test_failures.py
  - tools/session_coordinator/tests/test_milestone_failure_scope.py
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

## Current-Source Attestation Selection

An immutable historical milestone record and a fresh current-source attestation may
legitimately declare the same plan and milestone. The coordinator must not ask an
executor to rewrite, move, or falsify the historical record merely to make a new
run selectable.

When more than one child-plan record declares the requested milestone, manifest
binding selects exactly one record whose path is both dirty relative to `HEAD` and
attributed to the executing Session. Its declared files must satisfy the same
attribution check. Zero or multiple such records remain a hard
`milestone_manifest_record_ambiguous` rejection with the complete and attributed
candidate lists. This keeps historical accepted evidence immutable while ensuring a
new topology/run can bind only its executor-owned current-source proof.

## Fixed-Return Manifest Failure Scope

An already completed cross-plan return may need its own immutable manifest
committed while the fixing plan is still responsible for unrelated open
Failures. For that narrow case, the manifest must include a canonical `fixed-*`
artifact whose `fixing_plan` is the executing plan and whose destination is in
the origin child directory. The Failure selector then evaluates only the
manifest's exact origin workflow-node keys, retaining legacy node-less origin
Failures as plan-wide blockers.

This is not a general fixing-plan waiver. A normal milestone manifest, a
manifest with an unrelated fixed artifact, or an explicit/Goal finalize still
includes every open Failure assigned to the fixing plan. The same selector is
used when computing gate fingerprints, refreshing `failure_audit`, and twice
inside the Git mutex before the scoped commit, so a newly opened applicable
Failure invalidates the commit rather than being bypassed.

### Future-milestone Failure deferral

An executor may defer one open Failure from the current milestone only to a
strictly reachable successor in the same active plan. The durable record binds the
Session, plan path, semantic topology hash, source milestone, target milestone and
exact lifecycle key; it is not transferable to another run owner. Reverse, unrelated
or foreign-Session requests fail before a record is written. A semantic dependency
change immediately makes the old deferral inapplicable, while the ordinary
prepare/commit/Goal-closeout gates retain their stronger content-hash check and reject
any plan-text drift after import.

The `runtime14-rust-focused` validation template is a closed action/CLI choice. It
runs only `cargo +1.94.1 test -p zircon_runtime --lib
runtime_14_module_family_mirror_docs_match_structure_audit_counts --locked --jobs 1
-- --nocapture --test-threads=1`. Unlike Python and Web templates, this Rust template
uses the server-owned Cargo metadata closure planner: the immutable copy includes the
root manifests/toolchain plus every workspace member and recursively reachable local
path package required to reload the workspace, including `zircon_runtime_interface`
and `zircon_reflect_derive`. Registry/Git packages remain Cargo-managed. Source-null
path packages outside the repository are fail-closed by default; this closed template
may discover only sibling Git repositories, pin their HEAD, derive the job-root mount
and archive the manifest-referenced package roots without reading dirty worktree bytes.
Cargo metadata is decoded as UTF-8 independently of the Windows process locale. The
exact template identity is stored beside legacy validation bindings without rebuilding
an older database table.

### Atomic combined Failure closeout

When one accumulated full-blob delivery resolves multiple overlapping lifecycles, the
closeout accepts a sorted, deduplicated target set only when every target has the same
fixing plan, a canonical fixed artifact and exactly one return receipt. Supplemental
docs, tests and output records require a typed `failure_closeout_delivery` record bound
to the exact lifecycle-key set and explicit delivery paths; arbitrary snapshot extras
are rejected. Paths owned by any preserved open Failure remain forbidden. Prepare and
finalize revalidate all targets, returns, validation evidence and preserved lifecycles
under the Git mutex, so the union is committed once or not at all.

## Extension Constraints

Future graph expansion must keep topology acyclic, preserve stable keys, use coordinator services for mutations and append evidence instead of overwriting it. Failure nodes must reference the canonical `failure-*` or moved `fixed-*` artifact governed by the numbered child plan; the workflow database is an index and status projection, not a second canonical Markdown copy.

### Protected Plan Revisions

`TopologyImporter` stores every numbered-plan source revision as an immutable topology version. A normal structural refresh is rejected once a run has attempts, reviews, manifests, validation bindings, commit intents, or artifacts: changing an accepted node, dependency, title, slice, workflow identity, or goal would make historic evidence ambiguous.

There is one deliberately narrow continuation path for a plan that has already accepted earlier milestones: the new topology may append one or more previously unknown milestone IDs while preserving the entire existing milestone map, every old dependency, all slices, workflow identity, goal, plan identity, and source kind. No in-flight milestone attempt may exist. The importer then retains existing node IDs, evidence, accepted states and edges, inserts only the new pending nodes and their incoming dependency edges, and activates the candidate version. This permits a new delivery milestone such as M5 after M1/M2 have completed while leaving unresolved M3/M4 pending. It must never reconcile, re-label, or fabricate historical evidence.

## Verification

Schema tests validate migration and enum constraints. Store tests validate stable run identity and append-only attempt behavior. `test_workflow_topology.py` verifies that a progressed graph rejects arbitrary structural replacement, while an append-only milestone candidate preserves accepted nodes and leaves both earlier pending nodes and the new milestone pending. Projection tests validate current-attempt selection and preserved history. Server/Session regressions verify automatic synchronization at register, heartbeat and status transitions.
