---
related_code:
  - tools/session_coordinator/workspace_copy.py
implementation_files:
  - tools/session_coordinator/workspace_copy.py
tests:
  - tools/session_coordinator/tests/test_workspace_copy.py
doc_type: milestone-detail
---

# Validation-copy baseline archive stream drain

Plan: `docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`

Milestone: M6.9

Status: implementation_validated_inherited_commit_dependency_pending

## Problem and root cause

Frameworks01 exact-manifest materialization repeatedly failed before Cargo with
`validation_copy_dependency_archive_failed` at `baseline_archive`. The same pinned
HEAD succeeds when `git archive` writes directly to a file. A stream reproducer
iterated all 29,384 tar members, then observed exit code 141 with empty stderr.

Python `tarfile` stops after the tar end markers. A large `git archive` can still be
writing record padding at that point. Closing its stdout pipe before consuming the
remaining bytes causes SIGPIPE, and the coordinator incorrectly classifies the
complete archive as a dependency failure.

## Implementation

- Replace the manual stdout close, stderr read and wait sequence with
  `Popen.communicate()` after streamed extraction. This drains both remaining pipes
  without introducing a stderr back-pressure deadlock and then waits for the real
  process exit.
- Add a focused process-protocol regression whose valid tar has trailing record
  padding. The fake git process returns 141 when stdout is closed with unread bytes
  and returns 0 only after the caller drains the stream.
- Keep extraction failures authoritative when post-kill `communicate()` itself fails;
  cleanup remains best-effort only on an already-failing extraction path. Normal
  completion still requires successful pipe drain and the real git exit status.
- Preserve all archive path filtering, traversal checks, symlink handling, pinned
  HEAD selection, extracted-path accounting and nonzero-exit diagnostics.

## TDD and validation status

- RED: the focused regression reached `_extract_baseline_manifest`, extracted the
  file, then raised `Could not materialize the pinned validation baseline` because
  the simulated process returned 141.
- GREEN: the new regression and the existing large-manifest/single-archive test pass
  together, 2 passed / 0 failed.
- Complete `tools.session_coordinator.tests.test_workspace_copy` passes: 41 passed /
  0 failed in 231.923 seconds. Independent re-review and live-service rollover remain
  pending.

## Ownership and integration boundary

The two Python files already contained the preserved, attributed output of stale
Session `coordinator01-validation-copy-terminal-repair-r2-20260728` before this
milestone. Its baseline hashes were
`workspace_copy.py=5d04aafccdb85903ac09fc45b58d6f211f997eb5759fdc52319556b345d56a5f`
and
`test_workspace_copy.py=f22d3b347396ffa2dac3a7a257af603728a0de372555fcda8b8bc2d5be93e279`.
This milestone preserves those blobs and records only the bounded stream-drain
successor change. It must not form a clean-HEAD commit until the inherited
validation-copy dependency scope is attested or committed with its original owner
evidence.

## Completion table

| Item | Status | Evidence |
|---|---|---|
| Archive stream root-cause reproducer | complete | 29,384 members; exit 141; empty stderr |
| Focused TDD regression | complete | RED then GREEN; 2/2 focused tests pass |
| Complete workspace-copy module | complete | 41 passed / 0 failed / 231.923s |
| Independent review | in progress | initial C0/I1/M1; Minor fixed, inherited clean-HEAD I1 retained |
| Coordinator rollover and Frameworks01 retry | pending | must wait for active Cargo to release naturally |
| Managed commit | pending | inherited clean-HEAD dependency closure required |
