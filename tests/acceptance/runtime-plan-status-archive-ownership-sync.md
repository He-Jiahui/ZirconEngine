# Runtime plan-status numbered-archive ownership acceptance

## Scope

Runtime 05 plan-status/index and Runtime 15 test-owner maintenance only. This slice does not change runtime, plugin, render, editor, UI, text, asset, scene, or scripting production behavior.

## Baseline problem

- The Python boundary still required concrete five-column output rows in all 15 parent plans and historical slice anchors in `runtime/index.md`.
- The Rust plan-status suite repeated the same pre-migration assumption in Cargo-gate, closeout, recent-static, status-output, and subplan-status guards.
- The current index had lost its allowed aggregate 15-plan map, 17-problem map, and seven-row known-backlog table while concrete evidence had already moved into numbered archives.
- Three existing folder-backed guard owners and two new archive/routing helpers were absent from the audit/review inventory.

## Invariants

- Parent plans keep frontmatter, a current status overview, and links to numbered output archives; they do not duplicate historical concrete slice rows.
- Concrete five-column records are discovered from `runtime/01/` through `runtime/15/`.
- `runtime/index.md` owns only aggregate routing: 15 subplans, 17 problem rows, seven known backlog gaps, dependencies, lifecycle status, and remaining gates.
- Historical status-output anchors are accepted from numbered archives; current parent/index summaries remain concise.
- Runtime 05 remains `in_progress` until the full `scene::` Cargo gate closes.

## Evidence

- `python -m unittest tools.tests.test_runtime_plan_status_archive_ownership -v`: passed 1/1.
- Python byte-code compilation for the plan-status audit sources and regression test: passed.
- Direct `runtime_plan_status_boundary_audit`: support owners 84/84, subplans 15/15, index rows 15/15, problems 17/17, backlog 7/7, all missing-anchor/gap sets empty, `risks = []`.
- Standalone Rust plan-status harness compiled without warnings.
- Focused Rust index-table guards: passed 18/18.
- Full standalone Rust plan-status suite: passed 48/48.
- Runtime plan-directory output-record audit: zero violations; the repository-wide audit still reports unrelated active editor/render/text/frameworks/priority-doc migrations.
- Runtime 15 review-guard row-data structure suite after owner-inventory migration: passed 73/73.
- Package/workspace Cargo was not used as completion evidence because the shared lib-test baseline remains owned by active out-of-scope work.

## Decision

The Runtime 05 plan-status numbered-archive ownership migration is accepted as a static architecture-maintenance slice. No runtime subplan is promoted to `completed`; all declared package/startup gates remain authoritative.
