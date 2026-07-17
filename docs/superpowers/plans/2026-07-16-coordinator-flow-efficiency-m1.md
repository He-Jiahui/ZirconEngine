# Coordinator Flow Efficiency M1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove synchronizer-driven coordination noise and expose a compact, authoritative experience projection for the control console.

**Architecture:** `CodexSessionStore` remains the only persistence writer for discovered Codex state, but it will distinguish metadata refreshes from user-visible changes. `ControlSnapshotService` will project bounded, aggregate experience metrics from coordinator history so the web client can render current pressure without scanning raw audit events.

**Tech Stack:** Python 3 standard library, SQLite WAL, `unittest`, React 19, TypeScript, Vite and MUI.

---

## File structure

- `tools/session_coordinator/codex_sync/store.py` — classify meaningful versus metadata-only discovery changes and suppress timeline noise.
- `tools/session_coordinator/control_plane/snapshot.py` — add a bounded `experience` projection to snapshots.
- `tools/session_coordinator/control_plane/contracts.py` — document/control the projection schema if a server contract is centralized there.
- `tools/session_coordinator/tests/test_codex_store.py` — cover no-op and meaningful sync event behavior.
- `tools/session_coordinator/tests/test_control_snapshot.py` — cover rolling experience projection and bounded blocker data.
- `tools/session_coordinator/web/src/api/contracts.ts` — add the typed experience projection.
- `tools/session_coordinator/web/src/pages/OverviewPage.tsx` — render compact pressure/flow signals; M3 will expand this into the full work board.
- `tools/session_coordinator/web/src/__tests__/contracts.test.ts` and `tools/session_coordinator/web/src/__tests__/components.test.tsx` — assert projection compatibility and operator-visible labels.
- `docs/cli-and-tooling/local-session-coordinator.md` — update the control-plane and operator semantics.

## M1 — Event and state stabilization

### Implementation slices

- [x] Add focused store tests that reconcile the same discovered Session twice with only `last_synced_at` and source revision changing. Assert one `codex.session.discovered` event, zero `codex.session.updated` events on the second scan, and a retained sync-run row with `changed_count == 0`.
- [x] Add a store test that changes a visible field (`state`, `bound_session_id`, `diagnostic_code`, `last_event`, or `last_turn_id`) and asserts exactly one meaningful Session event with the new state.
- [x] Refactor `CodexSessionStore.reconcile` so metadata-only refreshes update the session row but do not increment visible change count or insert a timeline event. Keep source revision for discovery correctness and keep `codex_sync_runs` for operational telemetry.
- [x] Add snapshot tests with mixed sync runs and an active Cargo job; assert rolling visible-change/quiet-run metrics and a bounded, sanitized current resource blocker projection.
- [x] Implement `experience` in `ControlSnapshotService` using one read transaction and fixed query limits. Project only sanitized identifiers, owner, lane kind, status, and creation time; never project raw command lines or credential-bearing payloads.
- [x] Add typed client contracts and a compact Overview panel that shows no-op sync suppression and actionable resource blocker count. Preserve existing detailed pages as the source for drill-down.
- [x] Update the coordinator operator document with the distinction between metadata sync, visible Session state change, and hard resource blocking.

### Testing stage

- [x] Run `python -m unittest tools.session_coordinator.tests.test_codex_store tools.session_coordinator.tests.test_control_snapshot -v` and correct failures from the store/projection layer before changing UI behavior.
- [x] Run the web package's focused contract/component tests through its existing test script; verify the bundle sees the new optional projection without a client fallback error.
- [x] Run `python -m compileall -q tools/session_coordinator`, `npm run check`, and `git diff --check`.
- [ ] Load the source into the live daemon only after the foreign active Cargo job reaches a safe boundary, then compare a real quiet synchronization cycle. This must not restart or interrupt unrelated work.

### Acceptance evidence

- [ ] A live quiet sync creates no user-visible Session update event (source regression coverage is complete; live daemon reload is deliberately deferred while foreign Cargo is active).
- [x] The snapshot reports bounded experience data and a precise current blocker when a Cargo lane is owned.
- [x] The Overview renders the new pressure signals while existing route pages remain usable.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
| --- | --- | --- | --- | --- |
| M1 | 同步事件降噪、`experience` 投影与概览压力信号 | 源码与回归已完成；等待安全的实时加载窗口 | 2026-07-16 | Python focused 13/13；`npm run check`（类型检查、43/43、构建、27 assets）；`git diff --check`；live status `read_write` 且 `maintenanceHold=false` |
