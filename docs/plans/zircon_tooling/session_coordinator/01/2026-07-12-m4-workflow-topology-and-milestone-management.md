# M4 Workflow Topology and Milestone Management

Plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
Milestone: M4
Status: accepted
Files: ["tools/session_coordinator/migrations.py", "tools/session_coordinator/git_finalize.py", "tools/session_coordinator/notifications.py", "tools/session_coordinator/server.py", "tools/session_coordinator/workspace_copy.py", "tools/session_coordinator/control_plane/router.py", "tools/session_coordinator/control_plane/actions/catalog.py", "tools/session_coordinator/control_plane/actions/executor.py", "tools/session_coordinator/control_plane/actions/fingerprint.py", "tools/session_coordinator/control_plane/actions/models.py", "tools/session_coordinator/control_plane/actions/service.py", "tools/session_coordinator/workflows/__init__.py", "tools/session_coordinator/workflows/artifacts.py", "tools/session_coordinator/workflows/attempts.py", "tools/session_coordinator/workflows/gates.py", "tools/session_coordinator/workflows/milestones.py", "tools/session_coordinator/workflows/plan_import.py", "tools/session_coordinator/workflows/projections.py", "tools/session_coordinator/workflows/topology.py", "tools/session_coordinator/tests/test_action_catalog.py", "tools/session_coordinator/tests/test_action_concurrency.py", "tools/session_coordinator/tests/test_action_execution.py", "tools/session_coordinator/tests/test_action_fingerprint.py", "tools/session_coordinator/tests/test_notifications.py", "tools/session_coordinator/tests/test_workflow_attempts.py", "tools/session_coordinator/tests/test_workflow_commit.py", "tools/session_coordinator/tests/test_workflow_gates.py", "tools/session_coordinator/tests/test_workflow_projections.py", "tools/session_coordinator/tests/test_workflow_schema.py", "tools/session_coordinator/tests/test_workflow_topology.py", "tools/session_coordinator/web/src/App.tsx", "tools/session_coordinator/web/src/__tests__/contracts.test.ts", "tools/session_coordinator/web/src/pages/ActionsPage.tsx", "tools/session_coordinator/web/src/pages/WorkflowsPage.tsx", "tools/session_coordinator/web/src/api/contracts.ts", "tools/session_coordinator/web/src/api/validation.ts", "tools/session_coordinator/web/dist/index.html", "tools/session_coordinator/web/dist/assets/index-Bhkifx4O.js", "tools/session_coordinator/web/dist/assets/index-wUCUaAk1.js", "docs/cli-and-tooling/session-coordinator-milestone-workflows.md", "docs/plans/zircon_tooling/session_coordinator/01/2026-07-12-m4-workflow-topology-and-milestone-management.md"]

## Scope delivered

- Added monotonic schema 17 for immutable topology/evidence, schema 18 for
  milestone manifests/integrity, and schema 19 to upgrade already-deployed
  validation bindings without rewriting earlier migrations.
- Added bounded fenced/fallback topology parsing, immutable content versions,
  controlled content-only activation, and fail-closed structural activation
  after any non-Goal history exists.
- Added milestone-specific immutable path/hash manifests shared by validation,
  gate evaluation, live UI eligibility, and Git commit.
- Bound managed validation to an exact copied-source hash before launch and
  rechecked that source before importing its result, closing repository/copy
  time-of-check/time-of-use gaps.
- Separated reviewer and executor identity using registered Session IDs. The
  authenticated reviewer binding, durable review evidence, and target run owner
  must be distinct.
- Added exact structured plan-output validation, Failure-graph gates, strict
  manifest lease ownership, and Goal closeout auditing across write scope,
  leases, and attribution.
- Revalidated all gates under the Git mutex, preserved foreign staged state,
  committed only the milestone manifest, and completed baseline/finalizer state
  before publishing post-CAS workflow success.
- Made Goal attempt, lease release, Session/run completion, and event append one
  database transaction.
- Added at-most-once WeCom reservation/results with sanitized failures and no
  automatic retry or persisted webhook credential.
- Extended the control console with topology history, evidence timelines,
  reviewer/executor-aware controls, and live server-recomputed commit eligibility.

## Architecture decisions

- Schema 16 was already released by M3, so M4 appends schema 17 and 18 rather
  than rewriting an applied migration.
- Git compare-and-swap is the linearization point. Before it, failure restores
  the index and keeps HEAD unchanged. After it, recovery moves only forward and
  must finish baseline/finalizer state before milestone evidence succeeds.
- A milestone manifest is durable, immutable, node-scoped, and topology-scoped.
  Session-wide attribution cannot silently expand a different milestone commit.
- Structural plan edits never rewrite a graph with history. Content-only edits
  can activate a new immutable version because node identity remains unchanged.
- External notification has no idempotency key. A reserved or unknown delivery
  is therefore terminal and never retried automatically.

## Fresh testing evidence

- Focused Python architecture suites: 38 tests passed in 122.905 seconds.
- Critical-boundary Python suites: 25 tests passed in 118.859 seconds, including
  copied-source mutation rejection and post-CAS baseline forward recovery.
- Consolidated Python M4 regression set: 56 tests passed in 225.556 seconds.
- Critical-boundary Python rerun: 26 tests passed in 171.715 seconds.
- Web `npm run check`: TypeScript passed, 30 tests passed, production build
  succeeded, and two hashed assets were verified.

## Review

The first independent review found four Critical and eight Important findings;
all were repaired. The second review then found three Critical and eight
Important follow-ups covering identity namespaces, validation TOCTOU, post-CAS
completion, topology activation, manifest scope, output validation, closeout,
database integrity, notification failure behavior, and live UI eligibility.
All follow-ups were repaired and received focused regression coverage. The
fourth independent read-only review reported Critical 0 and Important 0; its
34 Python tests and 30 web tests passed, and its query-only audit confirmed the
real schema-v19 database, unique manifest binding, registered-Session review
constraint, and foreign-key integrity.
