# Acceptance And Evidence

## Progressive Disclosure Index

- Read this file when preparing the validation flow or deciding whether a change can be accepted.
- If you still need to choose tools, go to `../wsl-tool-selection/index.md`.
- Coverage breadth is governed by `docs/plans/milestone-validation-policy.md`: representative coverage plus targeted variants for the risks the milestone touches, not exhaustive enumeration.

## Validation Workflow

1. Re-state the scope.
   - Name the changed feature, bug, or milestone and the affected layers.
   - When WSL is used, state the Linux-specific reason, toolchain, architecture, profile, features, terminal command result and physical target below `/mnt/d/cargo-targets`, `/mnt/e/cargo-targets`, or `/mnt/f/cargo-targets`.

2. Establish the baseline.
   - Record the pre-change failure and any known repository-level baseline failures from `zircon-dev`.
   - Separate "already broken" from "introduced or fixed now" with evidence.

3. Assemble the focused test batch.
   - The focused unit/subsystem tests for the changed behavior.
   - Boundary or failure-path cases only where the milestone changes that risk surface.
   - Tool-assisted runs (sanitizer, `gdb`, `valgrind`, `heaptrack`) only when the failure mode requires them.

4. Execute from lower to upper layers.
   - Run focused lower-layer tests first, then parent-layer, then integration where in scope.
   - If a step fails, debug from the lowest shared layer and re-run only the affected focused batch (`../../support-first-regression-testing/guide.md`).

5. Acceptance.
   - A change is not accepted if a test failure is ignored or silently deferred.
   - A change is not accepted if the evidence trail cannot explain why the result is trustworthy.
   - Untested boundary/failure paths outside the milestone's changed risk surface are follow-up scope, not acceptance blockers.

## Evidence Record

Record evidence in the authorized numbered child output under `../../write-plan-output-records/guide.md`; update a plan-definition table only during explicitly authorized maintenance. Keep one canonical evidence owner; do not create per-feature acceptance documents under `tests/acceptance/` (existing files there are historical archive).

A milestone evidence row states: changed scope, commands actually run, result summary, failures repaired, and any deferred external checks. It must not claim a broad pass from a narrow run.

## Reporting

- State the plan record location.
- State the test batches and tool runs that contributed to the conclusion.
- State every failure encountered and how it was resolved, or why it remains a blocker.
