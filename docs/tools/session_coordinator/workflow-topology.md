---
related_code:
  - tools/session_coordinator/workflows/topology.py
  - tools/session_coordinator/workflows/plan_import.py
  - tools/session_coordinator/tests/test_workflow_topology.py
plan_sources:
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
  - docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-17-legacy-milestone-heading-closeout-deadlock.md
tests:
  - tools/session_coordinator/tests/test_workflow_topology.py
doc_type: module-detail
---

# Coordinator workflow topology

`workflows/topology.py` converts an immutable numbered plan definition into the
coordinator's canonical `M<n>` graph. It never writes the plan to make it fit
the workflow: the parser accepts a fenced `zircon-workflow` block when present,
or derives equivalent nodes from supported Markdown milestone headings.

## Legacy heading compatibility

Fallback parsing accepts all of the following presentation forms and normalizes
them to the same canonical node IDs:

- `## Milestone M1: Title`
- `### SH03-M1 Title`
- `### M1 Title`

This is intentionally a parser responsibility. Requiring a legacy plan to be
rewritten merely to close a scoped current-source change creates a workflow
deadlock: the plan has no selectable milestone, while generic finalization
requires a completed session that numbered plans cannot set directly.

The normal graph checks remain in force after parsing: duplicate IDs, missing
dependencies, cycles, oversized plans, and invalid child-output files continue
to be rejected. A parsed node still needs managed validation, a distinct review
session, exact manifest attribution, failure acceptance, and an atomic
coordinator commit.

## Verification

`WorkflowTopologyTests.test_fallback_imports_plain_numbered_milestone_headings`
proves the plain-heading form yields canonical nodes and preserves an `M2 -> M1`
dependency. The complete topology regression suite covers it alongside the
fenced and prefixed legacy forms.
