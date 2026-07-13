# Cargo Artifact Lifecycle Visualization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Cargo reuse, ephemeral cleanup, pending deletion, and cleanup failures directly observable on the existing Jenkins-style validation page without adding browser mutation authority.

**Architecture:** Extend the existing strict `CargoJobProjection` contract with fields already present in the coordinator snapshot, derive bounded summary counts entirely from the current snapshot, and render those summaries plus lifecycle columns in the existing validation components. No database, API, route, cleanup algorithm, or action-catalog changes are allowed.

**Tech Stack:** React 19, TypeScript, MUI, Node test runner, Vite, Python coordinator snapshot contracts.

---

## File Structure

- Modify `tools/session_coordinator/web/src/api/contracts.ts`: declare the persisted Cargo lifecycle fields and exact enums.
- Modify `tools/session_coordinator/web/src/api/validation.ts`: require lifecycle fields and reject unknown policies/statuses.
- Create `tools/session_coordinator/web/src/components/validation/ArtifactLifecycleSummary.tsx`: compute and render the four bounded summary metrics.
- Modify `tools/session_coordinator/web/src/components/validation/ValidationLaneTable.tsx`: render Session and lifecycle details using text-safe formatters.
- Modify `tools/session_coordinator/web/src/pages/ValidationPage.tsx`: compose the summary above the existing Cargo table and keep validation-copy behavior unchanged.
- Modify `tools/session_coordinator/web/src/__tests__/contracts.test.ts`: cover required fields and enum rejection.
- Modify `tools/session_coordinator/web/src/__tests__/components.test.tsx`: cover summary de-duplication and table text.
- Modify `docs/cli-and-tooling/workflow-control-center.md`: document the snapshot-bounded lifecycle visualization and no-manual-delete boundary.
- Modify `docs/plans/zircon_tooling/session_coordinator/01/2026-07-12-m6-security-load-release-acceptance.md`: append slice and testing-stage evidence.

## Milestone CAV-M1: Read-Only Cargo Lifecycle Visibility

### Implementation slices

- [x] **CAV-M1.1 Extend the strict Cargo projection contract and its contract fixtures.**

  Add the following exact properties to `CargoJobProjection`:

  ```ts
  reuse_key: string | null;
  compatibility_key: string | null;
  reuse_profile: string | null;
  reused_from_job_id: string | null;
  cleanup_policy: "retained" | "delete_on_release";
  cleanup_status: "retained" | "pending" | "deleted" | "failed";
  cleanup_error: string | null;
  ```

  In `validation.ts`, parse the nullable strings and validate both enums using the same explicit-set pattern as existing Cargo lane/status validation. Update the persisted-producer fixture in `contracts.test.ts` with a retained job, then add two malformed snapshots whose policy or cleanup status is unknown and assert that `parseSnapshot` throws.

- [x] **CAV-M1.2 Add a focused summary component.**

  Create `ArtifactLifecycleSummary.tsx` with a pure exported helper:

  ```ts
  export interface ArtifactLifecycleCounts {
    reusablePools: number;
    ephemeralJobs: number;
    pendingCleanup: number;
    failedCleanup: number;
  }

  export function artifactLifecycleCounts(jobs: CargoJobProjection[]): ArtifactLifecycleCounts
  ```

  `reusablePools` is the size of the set of non-null `compatibility_key` values on retained jobs. The other counters count `delete_on_release`, `pending`, and `failed` rows respectively. Render four MUI summary cells labelled `可复用池`, `用后即删`, `待清理`, and `清理失败`, with numeric text and a sentence stating that the values describe the current bounded coordinator snapshot rather than a disk scan.

- [x] **CAV-M1.3 Extend the Cargo table without creating a new page or action.**

  Add columns for `Session`, `产物策略`, `兼容键`, `复用来源`, `清理状态`, and `清理错误`. Map policy and cleanup enums to the approved Chinese labels, truncate compatibility/job identifiers to the first 12 characters plus an ellipsis when longer, and preserve the full value in the element `title`. Render cleanup status through `StatusText`; render errors as JSX text only and `—` when null. Keep existing lane, job state, target, PID, exit code, and creation time columns.

- [x] **CAV-M1.4 Compose the summary and document the authority boundary.**

  Place `ArtifactLifecycleSummary` above `ValidationLaneTable` in the existing `Cargo 验证租约` panel. Update the operator guide to state that the UI is a read-only snapshot projection, counts pools by compatibility identity, and never deletes paths or changes retention policy. Append one CAV-M1 implementation row to the M6 child record after these slices are complete.

### Testing stage CAV-M1-T

- [x] **Run the focused frontend contract and component tests, then the complete production frontend gate.**

  Run from the shared main checkout without creating a worktree:

  ```powershell
  npm --prefix tools/session_coordinator/web test
  npm --prefix tools/session_coordinator/web run check
  ```

  Expected: all Node tests pass; TypeScript accepts the expanded projection; production build succeeds; the recursive dist verifier reports only reachable content-hashed assets and no credential/webhook material.

- [x] **Run the focused backend snapshot contract and final diff checks.**

  ```powershell
  python -m unittest tools.session_coordinator.tests.test_control_snapshot tools.session_coordinator.tests.test_control_http -q
  git diff --check -- tools/session_coordinator/web docs/cli-and-tooling/workflow-control-center.md docs/plans/zircon_tooling/session_coordinator/01
  ```

  Expected: the snapshot and HTTP suites pass without API/schema changes; diff check reports no whitespace errors. If an upper-layer failure occurs, correct the lowest contract/validation layer first and rerun upward.

- [x] **Record acceptance without committing before M6 finishes.**

  Append a `CAV-M1-T` row with exact test counts and production asset results to `docs/plans/zircon_tooling/session_coordinator/01/2026-07-12-m6-security-load-release-acceptance.md`. Keep the files in the active M6 change set; the user-required commit boundary remains the accepted M6 milestone after the independent review and 24-hour soak.

## Acceptance Evidence

- Unknown lifecycle enum values fail closed in runtime contract tests.
- One compatibility pool with multiple historical jobs is counted once.
- Ephemeral, pending, failed, reused-from, and cleanup-error values are readable without relying on color.
- Cleanup error text is not interpreted as markup.
- No new action, route, database, API, disk scan, or deletion path exists.
- Full production web gate and focused backend snapshot/HTTP gates pass.

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| CAV-M1 | CAV-M1.1–M1.4 | `completed` | 2026-07-12 | 严格 Cargo 生命周期契约、四项有界摘要、生命周期表格和只读权限边界全部实现。 |
| CAV-M1 | CAV-M1-T | `completed` | 2026-07-12 | 前端 33/33、27 个哈希生产资源，后端全量 287/287 分模块通过；真实 807MB 状态库诊断暴露快照放大问题后，补充事件载荷上限、去重事件和内部清单摘要投影，未增加浏览器写权限。 |
