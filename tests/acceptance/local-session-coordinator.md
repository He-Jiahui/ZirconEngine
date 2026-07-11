---
related_code:
  - tools/session_coordinator/legacy.py
  - tools/session_coordinator/audit.py
  - tools/session_coordinator/cleanup.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/baselines.py
  - tools/session_coordinator/config.py
  - tools/session_coordinator/cli.py
  - tools/install-session-coordinator-task.ps1
implementation_files:
  - tools/session_coordinator/legacy.py
  - tools/session_coordinator/audit.py
  - tools/session_coordinator/cleanup.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/baselines.py
  - tools/session_coordinator/config.py
  - tools/session_coordinator/cli.py
  - tools/install-session-coordinator-task.ps1
plan_sources:
  - docs/superpowers/specs/2026-07-11-local-session-coordinator-design.md
  - docs/superpowers/plans/2026-07-11-local-session-coordinator.md
  - user: 2026-07-11 shared-main multi-Session coordination and service-managed intermediate versions
tests:
  - tools/session_coordinator/tests/test_legacy_migration.py
  - tools/session_coordinator/tests/test_retention.py
  - tools/session_coordinator/tests/test_rollout_audit.py
  - tools/session_coordinator/tests/test_baselines.py
  - tools/session_coordinator/tests/test_patches.py
  - tools/session_coordinator/tests/test_server.py
  - tools/tests/session-coordinator-smoke.Tests.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
doc_type: testing-guide
---

# Local Session Coordinator Acceptance

## Acceptance Boundary

The coordinator is accepted only when shared-`main` writes remain attributable, business intermediate versions stay outside Git, every Cargo target is drive-root managed, legacy Session state is migrated without content loss, and normal commits require an explicit finalize request. No acceptance command creates a worktree or branch.

All Git mutation, archive, retention, failure-return, validation-copy, and cleanup destructive tests run in generated temporary repositories or generated managed target roots. The real ZirconEngine rollout phase starts read-only, records exact counts/hashes, and applies only deterministic legacy mappings and the reviewed user-level startup cutover.

## Required Gates

```powershell
python -m unittest discover -s tools/session_coordinator/tests -p "test_*.py" -v
powershell -NoProfile -ExecutionPolicy Bypass -File tools/tests/session-coordinator-smoke.Tests.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
python .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/test_validate_plan_failure_handoffs.py
python .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py --repo-root E:/Git/ZirconEngine
python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:/Git/ZirconEngine
powershell -NoProfile -ExecutionPolicy Bypass -File tools/zircon-session.ps1 audit all -RepoRoot E:/Git/ZirconEngine -Json
git diff --check -- tools .codex/skills docs/cli-and-tooling tests/acceptance
```

## Evidence Matrix

| Requirement | Evidence owner | M6-T result |
|---|---|---|
| Enum-only Session import and unknown-status reason preservation | `test_legacy_migration.py` | PASS; 140 imported/service Sessions, 0 invalid enum values |
| 24-hour/no-reference archival with before/after SHA-256 | `test_legacy_migration.py`, legacy rollout smoke | PASS; 121 archived, 0 hash mismatches, 10 retained, 0 remaining candidates |
| Completed 14-day / archived 30-day retention and live reference protection | `test_retention.py` | PASS; focused recovery/retention drill 10/10 |
| Recursive formal/legacy plan and Failure diagnostics | `test_rollout_audit.py`, `audit all` | PASS; current read-only audit sees 130 formal plans, 126 legacy documents, 11 handoffs; 39 concurrent artifact diagnostics reported without mutation |
| Repo-local Cargo targets remain diagnostic-only | `test_rollout_audit.py` | PASS; one `.codex/targets` legacy root reported, no unsafe managed cleanup candidate |
| Two-tick startup health gate and reversible legacy disable | installer dry-run, rollback/path-scope smoke, ignored local cutover record | PASS; current-user startup backend, 2 cutover ticks, old task disabled and retained; preparing/idempotent/maintenance restore paths reviewed |
| Queued patch/finalize/cleanup/validation-copy restart recovery | coordinator unit and smoke suites | PASS; recovery drill 10/10; complete smoke five modes PASS |
| No Session-tag/checkpoint commit and no webhook material in Git | Git subject/staged-secret audit | PASS; scoped secret/subject audit and coordinator finalize tests |
| Full coordinator regression and independent review | Python discovery, five-mode PowerShell smoke, reviewer | PASS; 118/118 unit tests, five smoke modes plus 27 embedded Cargo/cleanup tests, no Critical/Important/Minor findings |

## Real-Repository Rollout Snapshot

The 2026-07-11 rollout ran on `main` against `E:/Git/ZirconEngine`. Two unchanged report-only scans produced the same SHA-256, `D24AC45A9488DC9815421833DBF0DACAACFA321C357381384B59586863A65FC9`. Import was replayed idempotently, archive preview returned zero further candidates, and four successful maintenance ticks were recorded in total (two cutover gates plus two rollout ticks).

The baseline remains deliberately `degraded`: reconciliation found 64 exact-content changes that had not yet been attributed by their independent business Sessions. The coordinator refused to create a new epoch or absorb those files. This is fail-closed evidence, not a coordinator-owned defect; each originating Session must register and attribute its own current hashes before the baseline can become healthy.

## Destructive Exclusions

- Never delete a path merely because its name resembles a target directory.
- Never archive a recent/live/referenced Session note.
- Never collect an object referenced by a retained snapshot or patch.
- Never disable a legacy cleanup task until two maintenance ticks and three surrounding health checks succeed.
- Never include runtime tokens, maintenance capabilities, webhook URLs, or machine-local cutover state in a commit.
