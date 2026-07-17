# Coordinator Flow Efficiency M3 Implementation Plan

**Goal:** Turn the existing control snapshot into a compact operator work board, without adding service-side gates or changing lifecycle/Cargo behavior.

## Design

The Overview derives four bounded lanes exclusively from the current snapshot:

| Lane | Source | Meaning |
| --- | --- | --- |
| 可继续 | `active`/`registered` Sessions | Normal work can proceed now. |
| 等待资源 | `waiting_lease`/`waiting_validation`/`finalizing` Sessions | The Session has a narrow, observable wait; it is not a global stop. |
| 需关注 | `resolving_failure`/`stale` Sessions | Recoverable health or preflight action is needed. |
| 需介入 | open Failure nodes | An unresolved recorded Failure requires owner action. |

Each lane caps rendered cards and includes a concise reason, while the existing resource panel keeps the current Cargo owner/lane visible. Detailed Session, Validation and Failure routes remain the authoritative drill-down surfaces.

## Test-first slices

- [x] Add a pure Overview projection test covering lane classification, concise fallback labels, open-Failure filtering, and the per-lane render bound.
- [x] Render four labeled work-board lanes from the existing snapshot without modifying server contracts.
- [x] Keep quiet-sync and resource-ownership metrics visible beside the board.
- [x] Run `npm run check` and `git diff --check`.

## Acceptance

- [x] An operator can see runnable, narrow-waiting, attention and intervention work without inspecting an event stream.
- [x] A resource owner never implies global Session admission is closed.
- [x] Existing Overview metrics and detailed routes remain compatible with legacy snapshots.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
| --- | --- | --- | --- | --- |
| M3 | 客户端四列工作板与压力信号 | 已完成 | 2026-07-17 | Overview 投影/渲染回归；`npm run check` 通过类型检查、45/45、构建与 27 assets；live snapshot `read_write`、UI 200 |
