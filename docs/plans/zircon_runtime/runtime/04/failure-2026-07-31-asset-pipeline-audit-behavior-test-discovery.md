---
handoff_kind: failure
status: fixed
created_at: 2026-07-31
summary_slug: asset-pipeline-audit-behavior-test-discovery
origin_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_runtime/runtime/04
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool/diagnostics.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool/single_flight.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool/task_pool.rs
tests:
  - python tools/tests/test_runtime_asset_pipeline_audit.py
---

# Runtime04: asset pipeline audit omits child behavior tests

## Failure evidence

`python tools/tests/test_runtime_asset_pipeline_audit.py` fails its current
child-guard audit. It reports these six missing behavior anchors even though
their Rust tests exist:

- `worker_pool_default_budgets_are_hard_limits`
- `worker_pool_bounded_queue_rejects_overflow_with_explicit_error`
- `concurrent_requests_for_same_asset_share_one_immutable_payload_owner`
- `worker_pool_diagnostics_track_in_flight_and_failure_counts`
- `worker_pool_frame_sampler_records_per_job_completion_deltas`
- `project_asset_manager_uses_the_injected_runtime_io_pool`

## Lowest shared root cause

`asset_pipeline_boundary.py` reads only
`zircon_runtime/src/asset/tests/pipeline/worker_pool.rs` for behavior anchors.
That parent module declares `diagnostics`, `single_flight`, and `task_pool`, but
does not contain their test bodies. The updated anchor inventory consequently
cannot see the behavior tests held by those child modules, while the plan and
mirror documents still claim `missing_behavior_test_anchors = []`.

## Required forward repair

- In `asset_pipeline_source_inventory.py`, append these three files to
  `RUNTIME_04_GUARD_FILES` and change `EXPECTED_GUARD_FILE_COUNT` from `17` to
  `20`:
  `zircon_runtime/src/asset/tests/pipeline/worker_pool/diagnostics.rs`,
  `zircon_runtime/src/asset/tests/pipeline/worker_pool/single_flight.rs`, and
  `zircon_runtime/src/asset/tests/pipeline/worker_pool/task_pool.rs`.
- In `asset_pipeline_boundary.py`, add the same three explicit paths to the
  `behavior_test_sources` tuple. `guard_sources` then obtains them through the
  inventory; the separate behavior tuple is required because the parent module
  contains declarations only, not test bodies.
- Keep the six existing behavioral tests as the authority; do not add duplicate
  wrappers, weaken the audit, or revert the current anchor names.
- Run the declared Python audit and update only the affected Runtime04 status
  records after its real result is known.

## Current state

Fixed. A 2026-07-31 direct rerun of `python
tools/tests/test_runtime_asset_pipeline_audit.py` executed 2 tests and failed
exactly 1: `test_current_child_guard_owners_close_the_runtime_04_audit`. Its
only assertion failure was the same six-item
`missing_behavior_test_anchors` list above; the second test passed. This is
current static failure evidence, not Cargo evidence.

The Runtime04 artifact session recorded this source-bound failure without
editing the foreign dirty anchor inventory or test file. The two owning audit
scripts require a separately acknowledged source scope before the forward
repair can be applied; no unacknowledged script mutation has been made.

The forward repair acquired the two audit-script paths under the Runtime04
owner session and added the three existing worker-pool child test modules to
both `RUNTIME_04_GUARD_FILES` and `behavior_test_sources`. The same current
source had also split the seven production worker-pool policy anchors into
`worker_pool/options.rs` and `worker_pool/completion.rs`; both real owners are
now explicit in the source inventory and worker source aggregation. Expected
counts therefore move from 22 to 24 production files and from 17 to 20 guard
files without removing or weakening an anchor.

The coordinator applied the protected `asset_pipeline_boundary.py` changes as
patch 70 (`98300d8138c042d5811691cabbedb65b`) and patch 72
(`9559f717f82a468c941d9ba6bb4d9841`). The final direct command
`python tools/tests/test_runtime_asset_pipeline_audit.py` executed 2 tests:
2 passed, 0 failed. A direct audit detail check reported `source=24`,
`guard=20`, `worker_missing=0`, `behavior_missing=0`, and `risks=0`; scoped
`git diff --check` also passed. The scoped second review reported
`0 Critical / 0 Important / 0 Minor`.
