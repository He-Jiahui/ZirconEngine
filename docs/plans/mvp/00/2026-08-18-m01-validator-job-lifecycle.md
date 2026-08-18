---
record_kind: milestone
status: completed
created_at: 2026-08-18
plan: docs/plans/mvp/00-current-source-baseline-recovery.md
milestone: M0.1
---

Plan: `docs/plans/mvp/00-current-source-baseline-recovery.md`

Milestone: `M0.1`

Status: completed

Files:

- `.codex/skills/zircon-dev/scripts/validate-matrix.ps1`
- `.codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1`
- `tools/tests/session-coordinator-smoke.Tests.ps1`

# M0.1 validator coordinator job lifecycle

## Scope Delivered

- Release a coordinator Cargo job when target resolution or a true pre-start failure aborts validation.
- Finish and then release a job when Cargo start may have reached the coordinator, including ambiguous start responses.
- Preserve the primary validation failure when best-effort finish or release cleanup also fails, including under `WarningPreference=Stop`.
- Keep dry-run validation on the release-only path so no real Cargo process is started.

## Performance Evidence

- Before the repair, the invalid-target reproduction left `1` coordinator job unreleased.
- A five-iteration dry-run benchmark released `5/5` jobs and left `0` active jobs.
- End-to-end control-plane latency was min `6.528 s`, p50 `7.073 s`, and p95/max `7.760 s`.
- The changed-contract Pester selection completed in `2.92 s` versus `247.35 s` for the full local suite, reducing repeated Pester gate time by `98.8%` while retaining the full-suite baseline result.
- These figures measure validator/coordinator control-plane lifecycle, not runtime hot-path throughput.

## Fresh Testing Evidence

- Local batch: Pester `106/106`, coordinator smoke `3/3`, and Python protocol tests `62/62` passed.
- The changed-contract Pester selection passed `14/14` locally in `2.92 s`.
- Managed batch `86865607c8f741c0b0ff680e4c21c0c4` passed with source manifest `85cc3178bc939753690dd189c37d484757f3d991242b62244ab2dc7b4c54bd83`.
- The managed batch passed `14` Pester, `3` smoke, and `62` Python cases in `95.47 s` with exit code `0`.

## Review

- Independent review: Critical `0`, Important `0`, Minor `0`.

## Residual Scope

- M0.2 Runtime 04 resolver validation and M0.3 Runtime-to-Editor-to-App compilation remain open; this record does not close the parent MVP 00 gate.
