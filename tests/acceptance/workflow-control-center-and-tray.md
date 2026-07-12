---
related_code:
  - tools/session_coordinator/control_plane
  - tools/session_coordinator/workflows
  - tools/session_coordinator/supervision
  - tools/session_coordinator/soak.py
  - tools/session_coordinator/web
  - tools/session_tray
implementation_files:
  - tools/session_coordinator
  - tools/session_tray
plan_sources:
  - docs/superpowers/specs/2026-07-11-workflow-control-center-and-tray-design.md
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
tests:
  - tools/session_coordinator/tests
  - tools/session_coordinator/web/src/__tests__
  - tools/tests/workflow-control-center-smoke.Tests.ps1
  - tools/tests/workflow-control-center-soak.ps1
doc_type: testing-guide
---

# Workflow Control Center and Tray Acceptance

## Purpose

This matrix proves the local Jenkins-style web console and Windows tray remain one control system. SQLite, workflow transitions, file/Cargo ownership, Git finalize, and lifecycle intents belong to the coordinator. The browser, Hub entry, and tray are authenticated clients and never become alternative persistence or mutation authorities.

## Requirement Matrix

| Requirement | Authoritative evidence | Acceptance condition |
|---|---|---|
| Jenkins-style workflow visualization | production web `npm run check`, read-only smoke, 1280×800 and 1568×1003 screenshots | Workflow/Node/Attempt, Sessions, Failures, leases, Cargo, Git, artifacts, logs and audit render from one snapshot plus ordered SSE |
| Controlled browser operations | action auth/catalog/fingerprint/execution/concurrency suites and controlled-action smoke | every mutation is typed, permissioned, previewed, confirmed, state-revalidated and audited; persistent identity-scoped Activity resumes executing-action tracking after refresh; read-only/fatal states disable mutation; reversible lifecycle drains expose cancellation; arbitrary shell/path/SQL input is impossible |
| Windows tray service management | Rust unit tests, real HTTP force-stop handshake test, production NSIS build, current-user install/upgrade/uninstall evidence and `-TrayLifecycle` smoke | verified repository/process/runtime identity gates open/drain/resume/stop/restart; Confirm/Cancel are serialized, Resume atomically cancels a reversible drain or reconciles a terminal-action orphan, lifecycle activation failure/startup recovery releases orphaned intents, and schema v26 safely audits historical multi-active repair before enforcing one active reversible intent; explicit Restart crosses only an unprotected stopping guard; stopping/terminal/offline proof commits atomically, while force-stop keeps transport alive through durable proof and Action-ID acknowledgement before a retrying graceful shutdown or second identity-gated termination check; timer scheduling and shutdown callback faults preserve terminal proof and a server-owned retry path; structured startup query exposes coordinator and tray separately; installed tray launches with explicit repository identity; in-place upgrade and residue-free uninstall succeed; tray exit leaves daemon alive; stale PID cannot be killed |
| Build artifact disk control | Cargo/cleanup/schema tests | compatible targets are reused with one writer; incomplete compatibility becomes ephemeral and is deleted immediately after safe release, with persisted retry |
| Security boundary | control security/auth/artifact matrix | malicious Host/Origin/referrer, ticket replay, CSRF, traversal, enumeration and oversized/range inputs fail closed without credential disclosure |
| Release-scale performance | `test_control_load.py` | exact 200/100/5,000/100,000/10,000/8-client/500MB shape; health <100ms, snapshot <800ms, list <300ms, event <500ms, preview <1s at P95 |
| Recovery | `test_control_recovery.py`, supervision suites and tray recovery tests | v13 data survives upgrade; failed migration rolls back; restart invalidates old runtime and preserves events; tray recovery journal survives restart, corrupt state fails closed, service health/audit projects the same recovery counters, and circuit/explicit-stop rules hold |
| Long-running stability | `workflow-control-center-soak.ps1 -Hours 24` | scheduled samples, disconnects, maintenance and one controlled restart complete with event continuity and bounded RSS/handle growth |

## Full Acceptance Command

Run only after all M6 implementation slices are present. Cargo commands must use the exact target directory returned by the coordinator lease.

```powershell
python -m unittest discover -s tools/session_coordinator/tests -v
npm --prefix tools/session_coordinator/web run check

$lease = (.\tools\zircon-session.ps1 cargo acquire test `
  --session-id workflow-control-center-20260711-1915 -Json | ConvertFrom-Json)
$env:CARGO_TARGET_DIR = $lease.job.target_dir
try {
  cargo test --manifest-path tools/session_tray/Cargo.toml --locked
  cargo build --manifest-path tools/session_tray/Cargo.toml --locked
  powershell -NoProfile -ExecutionPolicy Bypass `
    -File tools/tests/workflow-control-center-smoke.Tests.ps1 -Full
} finally {
  .\tools\zircon-session.ps1 cargo release $lease.job.job_id `
    --session-id workflow-control-center-20260711-1915 -Json
}

powershell -NoProfile -ExecutionPolicy Bypass `
  -File tools/tests/workflow-control-center-soak.ps1 -Hours 24
```

## Failure Interpretation

Repair the lowest shared layer first. A snapshot/UI failure does not justify a frontend fallback if schema, projection, or event ordering is wrong. A tray lifecycle failure does not justify direct process termination if descriptor, identity, supervision, or controlled-action evidence is missing. A Cargo cleanup failure must preserve the target and retry; it must never widen deletion roots.

No success claim may use a short soak as evidence for the 24-hour gate. Any change to daemon lifecycle, SSE, maintenance scheduling, or resource retention after a soak requires the full soak to be repeated.
