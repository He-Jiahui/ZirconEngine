import test from "node:test";
import assert from "node:assert/strict";
import { JSDOM } from "jsdom";
import { act, useState } from "react";
import { createRoot } from "react-dom/client";
import { ThemeProvider } from "@mui/material/styles";
import type { AuditEvent, CargoLaneProjection, CodexSessionsProjection, ControlSnapshot, FailureNode, FinalizeRequestProjection, WorkflowNode } from "../api/contracts";
import { StatusText } from "../components/StatusText";
import { FixedSizeList } from "../components/audit/fixedList";
import { NodeDetailDrawer } from "../components/workflow/NodeDetailDrawer";
import { LogViewer } from "../components/logs/LogViewer";
import { controlTheme } from "../theme";
import { ValidationLaneTable } from "../components/validation/ValidationLaneTable";
import { MilestoneCommitEvidence } from "../components/git/MilestoneCommitEvidence";
import { admissionSummary, cleanupDebtSummary, continuationGuidance, interventionGuidance, OverviewPage, overviewMetrics, resourceBlockers, resourceBlockerSummary, sessionLivenessSummary, syncHealthSummary, validationFlowHealth, validationFlowSummary, workBoard } from "../pages/OverviewPage";
import { ArtifactLifecycleSummary, artifactLifecycleCounts } from "../components/validation/ArtifactLifecycleSummary";
import { SessionsPage } from "../pages/SessionsPage";
import { ValidationPage } from "../pages/ValidationPage";

setupDom();

test("status remains readable without color", async () => {
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  await act(async () => root.render(<ThemeProvider theme={controlTheme}><StatusText value="failed" /></ThemeProvider>));
  assert.match(host.textContent ?? "", /failed/);
  await act(async () => root.unmount()); host.remove();
});

test("virtual list mounts only visible rows", async () => {
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  const items = Array.from({ length: 500 }, (_, index) => index);
  await act(async () => root.render(<FixedSizeList items={items} rowKey={String} render={String} label="test" />));
  assert.ok(host.querySelectorAll('[role="listitem"]').length < 40);
  assert.equal(host.querySelector('[role="list"]')?.getAttribute("aria-label"), "test，共 500 行");
  await act(async () => root.unmount()); host.remove();
});

test("log pause freezes incoming events and resume follows", async () => {
  mockFetch();
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  const first = [audit(1, "first")];
  await act(async () => { root.render(<ThemeProvider theme={controlTheme}><LogViewer events={first} /></ThemeProvider>); await Promise.resolve(); });
  const checkbox = host.querySelector('input[type="checkbox"]') as HTMLInputElement;
  await act(async () => checkbox.click());
  await act(async () => root.render(<ThemeProvider theme={controlTheme}><LogViewer events={[...first, audit(2, "second")]} /></ThemeProvider>));
  assert.doesNotMatch(host.textContent ?? "", /second/);
  await act(async () => checkbox.click());
  assert.match(host.textContent ?? "", /second/);
  await act(async () => root.unmount()); host.remove();
});

test("log payload is rendered as text and cannot execute markup", async () => {
  mockFetch();
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  const payload = '<img src=x onerror="globalThis.__zirconXss=true"><script>globalThis.__zirconXss=true</script>';
  await act(async () => { root.render(<ThemeProvider theme={controlTheme}><LogViewer events={[audit(3, payload)]} /></ThemeProvider>); await Promise.resolve(); });
  assert.equal(host.querySelector("script"), null);
  assert.equal(host.querySelector("img"), null);
  assert.match(host.textContent ?? "", /<script>/);
  assert.equal((globalThis as typeof globalThis & { __zirconXss?: boolean }).__zirconXss, undefined);
  await act(async () => root.unmount()); host.remove();
});

test("drawer associates failures and exposes attempt, gate, review, and notification evidence", async () => {
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  function Harness() { const [selected, setSelected] = useState<WorkflowNode | null>(null); return <ThemeProvider theme={controlTheme}><button id="opener" onClick={() => setSelected(node)}>打开</button><NodeDetailDrawer node={selected} planPath="docs/plans/x/01.md" edges={[]} artifacts={[]} leases={[]} failures={[failure]} gates={[gate]} reviews={[review]} notifications={[notification]} onClose={() => setSelected(null)} /></ThemeProvider>; }
  await act(async () => root.render(<Harness />));
  const opener = host.querySelector("#opener") as HTMLButtonElement; opener.focus();
  await act(async () => opener.click());
  const close = document.querySelector('button[aria-label="关闭节点详情"]') as HTMLButtonElement;
  assert.equal(document.activeElement, close);
  assert.match(document.body.textContent ?? "", /architecture-fix/);
  assert.match(document.body.textContent ?? "", /failure-2026-07-11-architecture-fix\.md/);
  assert.match(document.body.textContent ?? "", /npm test/);
  assert.match(document.body.textContent ?? "", /退出码：0/);
  assert.match(document.body.textContent ?? "", /validation\.accepted/);
  assert.match(document.body.textContent ?? "", /reviewer-a/);
  assert.match(document.body.textContent ?? "", /企业微信.*succeeded/);
  await act(async () => close.click());
  assert.equal(document.activeElement, opener);
  await act(async () => root.unmount()); host.remove();
});

test("control tables render safe validation-lane and commit projection fields", async () => {
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  const request: FinalizeRequestProjection = { request_id: "request-1", session_id: "s", message: "m", paths: ["a.rs"], categories: { code: ["a.rs"] }, untracked: ["notes.md"], validation: [["cargo", "test", "-p", "crate"]], maintenance: 0, status: "committed", commit_sha: "0123456789abcdef", error_text: null, created_at: "now", completed_at: "now" };
  const lane = {
    ...cargoLane({ cleanup_policy: "delete_on_release", cleanup_status: "failed" }),
    process_observation: "observed",
    target_dir: "R:/targets/private-job-1",
    command: ["cargo", "--lane-private"],
    cleanup_error: '<img src=x onerror="globalThis.__zirconXss=true">cleanup denied',
  } as unknown as CargoLaneProjection;
  await act(async () => root.render(<ThemeProvider theme={controlTheme}><ValidationLaneTable jobs={[lane]} /><MilestoneCommitEvidence requests={[request]} /></ThemeProvider>));
  assert.match(host.textContent ?? "", /workspace/);
  assert.match(host.textContent ?? "", /running/);
  assert.match(host.textContent ?? "", /用后即删/);
  assert.match(host.textContent ?? "", /清理失败/);
  assert.match(host.textContent ?? "", /进程已观察；心跳慢不会中断/);
  assert.doesNotMatch(host.textContent ?? "", /private-job-1|lane-private|cleanup denied|<img/);
  assert.equal(host.querySelector("img"), null);
  assert.match(host.textContent ?? "", /0123456789ab/);
  assert.match(host.textContent ?? "", /m/);
  assert.match(host.textContent ?? "", /code 1/);
  assert.match(host.textContent ?? "", /cargo test -p crate/);
  assert.match(host.textContent ?? "", /cargo test/);
  assert.match(host.textContent ?? "", /1m 0s/);
  await act(async () => root.unmount()); host.remove();
});

test("artifact lifecycle summary uses the live unique-target projection", async () => {
  const lifecycle = {
    reusablePools: 5,
    ephemeralTargets: 4,
    pendingCleanup: 4,
    failedCleanup: 1,
  };
  assert.deepEqual(artifactLifecycleCounts(lifecycle), lifecycle);
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  await act(async () => root.render(<ThemeProvider theme={controlTheme}><ArtifactLifecycleSummary lifecycle={lifecycle} /></ThemeProvider>));
  for (const label of ["可复用池 5", "用后即删 4", "待清理 4", "清理失败 1"])
    assert.ok(host.querySelector(`[aria-label="${label}"]`));
  assert.match(host.textContent ?? "", /当前存在的唯一 Cargo 目录/);
  await act(async () => root.unmount()); host.remove();
});

test("validation page shows queue order without treating it as Session admission", async () => {
  const validation = {
    cargoJobs: [], currentCargoTargets: [], validationCopies: [],
    artifactLifecycle: { reusablePools: 0, ephemeralTargets: 0, pendingCleanup: 0, failedCleanup: 0 },
    cpuBurst: { capacity: 1, active: 1, eligiblePending: 1 },
    cargoReservations: [
      { reservationId: "cpu-running", sessionId: "owner", laneScope: "cpu", executionMode: "warm", burstEligible: false, status: "running", queuePosition: 1, createdAt: "now", expiresAt: "later" },
      { reservationId: "cpu-next", sessionId: "next", laneScope: "cpu", executionMode: "warm", burstEligible: true, status: "pending", queuePosition: 2, createdAt: "now", expiresAt: "later" },
      { reservationId: "cpu-burst", sessionId: "burst-owner", laneScope: "cpu", executionMode: "burst", burstEligible: true, status: "running", queuePosition: 1, createdAt: "now", expiresAt: "later" },
    ],
  } as unknown as ControlSnapshot["validation"];
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  await act(async () => root.render(<ThemeProvider theme={controlTheme}><ValidationPage validation={validation} /></ThemeProvider>));
  assert.match(host.textContent ?? "", /验证通道队列/);
  assert.match(host.textContent ?? "", /CPU #1 · 热缓存/);
  assert.match(host.textContent ?? "", /CPU #2 · 热缓存/);
  assert.match(host.textContent ?? "", /CPU #1 · 隔离突发/);
  assert.match(host.textContent ?? "", /可隔离检查/);
  assert.match(host.textContent ?? "", /CPU 突发 WIP：1\/1 · 可隔离检查 1/);
  assert.match(host.textContent ?? "", /只排验证，不阻塞 Session/);
  assert.match(host.textContent ?? "", /作业健康检测中；预约到期不影响运行/);
  assert.doesNotMatch(host.textContent ?? "", /owner · 等待时间未知 · 到期 later/);
  assert.match(host.textContent ?? "", /next · 等待时间未知 · 到期 later/);
  await act(async () => root.unmount()); host.remove();
});

test("validation flow health exposes lane WIP, queue head, and bounded waiting age", async () => {
  const validation = {
    cargoJobs: [], currentCargoTargets: [], validationCopies: [],
    artifactLifecycle: { reusablePools: 0, ephemeralTargets: 0, pendingCleanup: 0, failedCleanup: 0 },
    cpuBurst: { capacity: 1, active: 0, eligiblePending: 2 },
    cargoReservations: [
      { reservationId: "cpu-running", sessionId: "owner", laneScope: "cpu", executionMode: "warm", burstEligible: false, status: "running", queuePosition: 1, createdAt: "2026-07-16T15:50:00Z", expiresAt: "later" },
      { reservationId: "cpu-next", sessionId: "next", laneScope: "cpu", executionMode: "warm", burstEligible: true, status: "pending", queuePosition: 2, createdAt: "2026-07-16T15:51:00Z", expiresAt: "later" },
      { reservationId: "cpu-later", sessionId: "later", laneScope: "cpu", executionMode: "warm", burstEligible: true, status: "pending", queuePosition: 3, createdAt: "2026-07-16T15:55:00Z", expiresAt: "later" },
    ],
  } as unknown as ControlSnapshot["validation"];
  const snapshot = {
    workflows: [], sessions: [], failures: { nodes: [], diagnostics: [] }, validation,
    service: { mode: "read_write" },
  } as unknown as ControlSnapshot;
  const lanes = validationFlowHealth(snapshot, new Date("2026-07-16T16:00:00Z"));
  assert.deepEqual(lanes, [{ laneScope: "cpu", activeCount: 1, queuedCount: 2, nextSessionId: "next", oldestQueuedMinutes: 9 }]);
  assert.equal(validationFlowSummary(lanes[0]), "CPU 热缓存：运行 1 · 排队 2 · 下一个 next · 最久等待 9 分钟");

  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  await act(async () => root.render(<ThemeProvider theme={controlTheme}><OverviewPage snapshot={snapshot} /><ValidationPage validation={validation} /></ThemeProvider>));
  assert.match(host.textContent ?? "", /验证流速/);
  assert.match(host.textContent ?? "", /下一个 next/);
  assert.match(host.textContent ?? "", /CPU 热缓存：运行 1 · 排队 2/);
  assert.match(host.textContent ?? "", /CPU 突发 WIP：0\/1 · 可隔离检查 2/);
  assert.match(host.textContent ?? "", /只排验证，不阻塞 Session/);
  await act(async () => root.unmount()); host.remove();
});

test("overview counts only running Cargo targets in the real-time baseline", () => {
  const snapshot = { workflows: [], sessions: [], failures: { nodes: [] }, validation: { cargoJobs: [{ status: "running" }, { status: "running" }], currentCargoTargets: [{ status: "running" }, { status: "succeeded" }] } } as unknown as ControlSnapshot;
  assert.deepEqual(overviewMetrics(snapshot), [
    ["工作流", 0], ["活动会话", 0], ["Failure", 0], ["运行验证", 1],
    ["同步状态", "未采样"], ["资源阻塞", 1],
  ]);
});

test("overview prioritizes the latest quiet sync over a noisy historical trend", async () => {
  const snapshot = {
    workflows: [], sessions: [], failures: { nodes: [] },
    validation: { currentCargoTargets: [] },
    codexSessions: {
      lastRun: {
        runId: "latest", trigger: "periodic", status: "succeeded", scannedCount: 245,
        changedCount: 0, diagnosticCount: 0, unavailableCount: 0, durationMs: 294,
        errorCode: null, createdAt: "now", completedAt: "now",
      },
    },
    experience: {
      sync: { runs: 12, quietRuns: 9, visibleChanges: 3, averageDurationMs: 25 },
      blockers: [{ kind: "cargo", ownerSessionId: "session-a", laneKind: "test", status: "running", createdAt: "now" }],
    },
  } as unknown as ControlSnapshot;
  const metrics = overviewMetrics(snapshot);
  assert.deepEqual(metrics.slice(-2), [["同步状态", "安静"], ["资源阻塞", 1]]);
  assert.deepEqual(syncHealthSummary(snapshot), {
    headline: "安静",
    detail: "最近一次安静同步：扫描 245 项，用时 294ms；24 小时趋势：9/12 安静同步，3 项可见变更。",
  });
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  await act(async () => root.render(<ThemeProvider theme={controlTheme}><OverviewPage snapshot={snapshot} /></ThemeProvider>));
  assert.match(host.textContent ?? "", /协调器同步/);
  assert.match(host.textContent ?? "", /最近一次安静同步：扫描 245 项，用时 294ms/);
  await act(async () => root.unmount()); host.remove();
});

test("overview distinguishes a local validation wait from Session admission", () => {
  const snapshot = {
    service: { mode: "read_write", supervision: { busy: true } },
    experience: {
      sync: { runs: 0, quietRuns: 0, visibleChanges: 0, averageDurationMs: 0 },
      blockers: [{ kind: "cargo", ownerSessionId: "session-a", laneKind: "test", status: "running", createdAt: "now" }],
    },
  } as unknown as ControlSnapshot;

  assert.deepEqual(admissionSummary(snapshot), {
    title: "Session 准入开放",
    detail: "1 条独占验证通道正在使用；仅等待该通道，其他 Session 不排空、不暂停。",
  });
  assert.equal(
    sessionLivenessSummary({ service: { sessionTtlSeconds: 3600 } } as unknown as ControlSnapshot),
    "业务 Session 活跃窗口 60 分钟；资源租约和预约 TTL 仍独立回收。",
  );
});

test("overview turns a local validation wait into one same-plan continuation and primary return", async () => {
  const snapshot = {
    service: { mode: "read_write" },
    workflows: [], sessions: [], failures: { nodes: [] }, validation: { currentCargoTargets: [] },
    experience: {
      sync: { runs: 0, quietRuns: 0, visibleChanges: 0, averageDurationMs: 0 },
      blockers: [],
      continuations: [{
        sessionId: "waiting-owner", planPath: "docs/plans/tooling/01-workflow.md", waitKind: "validation",
        candidate: { milestone: "M1", title: "Write the remaining module documentation." },
        scopeClaimRequired: true, returnToPrimary: true,
      }],
    },
  } as unknown as ControlSnapshot;

  assert.deepEqual(continuationGuidance(snapshot), [{
    sessionId: "waiting-owner", waitKind: "validation", milestone: "M1",
    title: "Write the remaining module documentation.", returnToPrimary: true,
  }]);

  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  await act(async () => root.render(<ThemeProvider theme={controlTheme}><OverviewPage snapshot={snapshot} /></ThemeProvider>));
  assert.match(host.textContent ?? "", /不要等待/);
  assert.match(host.textContent ?? "", /Write the remaining module documentation/);
  assert.match(host.textContent ?? "", /先领取作用域/);
  assert.match(host.textContent ?? "", /完成后优先回到主任务/);
  await act(async () => root.unmount()); host.remove();
});

test("overview exposes cleanup debt without closing Session admission", () => {
  const snapshot = {
    service: { mode: "read_write" },
    validation: {
      currentCargoTargets: [],
      artifactLifecycle: {
        reusablePools: 2,
        ephemeralTargets: 3,
        pendingCleanup: 4,
        failedCleanup: 1,
      },
    },
  } as unknown as ControlSnapshot;

  assert.deepEqual(cleanupDebtSummary(snapshot), {
    title: "构建产物回收需处理",
    detail: "2 个可复用池、3 个临时产物；4 个待清理、1 个清理失败。请在验证详情处理，Session 准入保持开放。",
  });
  assert.equal(admissionSummary(snapshot).title, "Session 准入开放");
});

test("overview gives a bounded elapsed wait for an occupied validation lane", () => {
  assert.equal(resourceBlockerSummary({
    kind: "cargo", ownerSessionId: "session-a", laneKind: "test", status: "running", createdAt: "2026-07-16T16:00:00Z",
  }, new Date("2026-07-16T16:04:00Z")), "test 通道由 session-a 运行中（已运行 4 分钟）");
});

test("overview prefers live Cargo targets over a stale experience summary", () => {
  const snapshot = {
    validation: { currentCargoTargets: [{ job_id: "live", session_id: "session-live", lane_kind: "test", status: "running", created_at: "now" }] },
    experience: {
      sync: { runs: 0, quietRuns: 0, visibleChanges: 0, averageDurationMs: 0 },
      blockers: [{ kind: "cargo", ownerSessionId: "session-stale", laneKind: "workspace", status: "running", createdAt: "old" }],
    },
  } as unknown as ControlSnapshot;

  assert.deepEqual(resourceBlockers(snapshot), [{
    kind: "cargo", ownerSessionId: "session-live", laneKind: "test", status: "running", createdAt: "now",
  }]);
});

test("overview converts open failures into one plan-level intervention recommendation", () => {
  const snapshot = {
    failures: { nodes: [
      { ...failure, node_id: 3, summary_slug: "first", priority: 0, created_at: "2026-07-11", fixing_plan: "docs/plans/editor-a.md" },
      { ...failure, node_id: 4, summary_slug: "same-plan", priority: 0, created_at: "2026-07-12", fixing_plan: "docs/plans/editor-a.md" },
      { ...failure, node_id: 5, summary_slug: "later", priority: 0, created_at: "2026-07-13", fixing_plan: "docs/plans/editor-b.md" },
    ] },
  } as unknown as ControlSnapshot;

  assert.deepEqual(interventionGuidance(snapshot), {
    failureCount: 3,
    planCount: 2,
    next: { summary: "first", fixingPlan: "docs/plans/editor-a.md" },
  });
});

test("overview groups bounded work into actionable operator lanes", () => {
  const ready = Array.from({ length: 9 }, (_, index) => ({
    sessionId: `ready-${index}`, displayName: index === 0 ? null : `Ready ${index}`,
    planPath: null, status: "active", statusReason: null, baseHead: null,
    baselineEpoch: null, writeScope: [], updatedAt: "now", lastHeartbeatAt: "now",
  }));
  const snapshot = {
    sessions: [
      ...ready,
      { sessionId: "wait-validation", displayName: "Validation", planPath: "docs/plan.md", status: "waiting_validation", statusReason: "GPU lane busy", baseHead: null, baselineEpoch: null, writeScope: [], updatedAt: "now", lastHeartbeatAt: "now" },
      { sessionId: "attention-stale", displayName: "Stale", planPath: null, status: "stale", statusReason: null, baseHead: null, baselineEpoch: null, writeScope: [], updatedAt: "now", lastHeartbeatAt: "now" },
    ],
    failures: { nodes: [
      { ...failure, node_id: 2, summary_slug: "open-failure", status: "open" },
      { ...failure, node_id: 3, summary_slug: "fixed-failure", status: "fixed" },
    ] },
  } as unknown as ControlSnapshot;

  const board = workBoard(snapshot);
  assert.deepEqual(board.map((lane) => [lane.key, lane.cards.length, lane.overflowCount]), [
    ["ready", 8, 1], ["waiting", 1, 0], ["attention", 1, 0], ["intervention", 1, 0],
  ]);
  assert.equal(board[0].cards[0].title, "ready-0");
  assert.equal(board[1].cards[0].detail, "GPU lane busy");
  assert.equal(board[3].cards[0].title, "open-failure");
});

test("overview renders the four operator work-board lanes", async () => {
  const snapshot = {
    workflows: [],
    sessions: [{ sessionId: "ready", displayName: "Ready session", planPath: null, status: "active", statusReason: null, baseHead: null, baselineEpoch: null, writeScope: [], updatedAt: "now", lastHeartbeatAt: "now" }],
    failures: { nodes: [{ ...failure, node_id: 4, summary_slug: "intervention-needed", status: "open" }] },
    validation: { currentCargoTargets: [] },
  } as unknown as ControlSnapshot;
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  await act(async () => root.render(<ThemeProvider theme={controlTheme}><OverviewPage snapshot={snapshot} /></ThemeProvider>));
  for (const label of ["可继续", "等待资源", "需关注", "需介入", "Ready session", "intervention-needed"])
    assert.match(host.textContent ?? "", new RegExp(label));
  await act(async () => root.unmount()); host.remove();
});

test("Sessions page separates business authority from text-only Codex presence", async () => {
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  const diagnostic = '<img src=x onerror="globalThis.__zirconXss=true">safe-code';
  const codex: CodexSessionsProjection = {
    rows: [{ threadId: "thread-12345678901234567890", sourceLocation: "active", state: "active", originator: "Codex Desktop", cliVersion: "0.test", threadSource: "user", lastEvent: "task_started", lastTurnId: "turn-one", boundSessionId: "business-one", diagnosticCode: diagnostic, firstSeenAt: "now", lastActivityAt: "now", lastSyncedAt: "now" }],
    total: 1, truncated: false, stateCounts: { active: 1, idle: 0, archived: 0, unavailable: 0 }, sourceCounts: { active: 1, archived: 0, missing: 0 }, queueDepth: 2, lastSuccessfulAt: "now", lastTerminalCode: "succeeded", lastRun: null,
  };
  await act(async () => root.render(<ThemeProvider theme={controlTheme}><SessionsPage sessions={[]} codexSessions={codex} /></ThemeProvider>));
  assert.match(host.textContent ?? "", /业务 Session（计划与写入权威）/);
  assert.match(host.textContent ?? "", /Codex 来源 Session（只读存在性）/);
  assert.match(host.textContent ?? "", /队列 2/);
  assert.match(host.textContent ?? "", /thread-12345678…/);
  assert.equal(host.querySelector("[title='thread-12345678901234567890']")?.textContent, "thread-12345678…");
  assert.match(host.textContent ?? "", /<img/);
  assert.equal(host.querySelector("img"), null);
  assert.equal((globalThis as typeof globalThis & { __zirconXss?: boolean }).__zirconXss, undefined);
  await act(async () => root.unmount()); host.remove();
});

const node: WorkflowNode = { nodeId: "n", nodeKey: "n", kind: "goal", title: "节点", stage: "implementation", state: "running", ownerSessionId: "s", statusReason: null, currentAttempt: { attemptId: "attempt-1", attemptNumber: 1, state: "succeeded", accepted: true, evidence: { command: ["npm", "test"], exitCode: 0, durationMs: 60000 }, startedAt: "2026-07-11T00:00:00Z", completedAt: "2026-07-11T00:01:00Z" }, attemptHistory: [] };
const failure: FailureNode = { node_id: 1, lifecycle_key: "life", artifact_path: "docs/plans/y/02/failure-2026-07-11-architecture-fix.md", kind: "failure", status: "open", created_at: "now", resolved_at: null, summary_slug: "architecture-fix", origin_plan: "docs/plans/x/01.md", fixing_plan: "docs/plans/y/02.md", origin_child_dir: "01", fixing_child_dir: "02", priority: 1, imported_at: "now" };
const gate = { evidenceId: "gate-1", topologyVersionId: "topology-1", nodeId: "n", attemptId: "attempt-1", kind: "validation", decision: "accepted", code: "validation.accepted", inputFingerprint: "fingerprint", blockingNodeIds: [], applicableFailureIds: [], requiredEvidence: [], createdAt: "now" };
const review = { reviewId: "review-1", topologyVersionId: "topology-1", nodeId: "n", attemptId: "attempt-1", reviewer: "reviewer-a", executor: "executor-a", verdict: "accepted", criticalCount: 0, importantCount: 0, summary: "通过", createdAt: "now" };
const notification = { attemptId: "notification-1", commitSha: "0123456789abcdef", channel: "wecom", status: "succeeded", attemptedAt: "now", completedAt: "now", exitCode: 0, providerErrcode: null, sanitizedError: null, retryAllowed: false };
function audit(eventId: number, message: string): AuditEvent { return { eventId, sessionId: "s", type: "info", payload: { message }, createdAt: "2026-07-11T00:00:00Z" }; }
function cargoLane(overrides: Partial<CargoLaneProjection> = {}): CargoLaneProjection { return { job_id: "job-1", session_id: "s", lane_kind: "workspace", status: "running", created_at: "2026-07-11T00:00:00Z", started_at: "2026-07-11T00:00:00Z", finished_at: "2026-07-11T00:01:00Z", released_at: null, cleanup_policy: "retained", cleanup_status: "retained", process_observation: "not_applicable", ...overrides }; }
function mockFetch() { globalThis.fetch = async () => new Response(JSON.stringify({ ok: true, data: { events: [], truncated: false, nextBefore: null }, meta: { apiVersion: 1, correlationId: "test" } }), { status: 200, headers: { "Content-Type": "application/json" } }); }

function setupDom() {
  const dom = new JSDOM("<!doctype html><html><body></body></html>", { url: "http://127.0.0.1:4317/ui/" });
  Object.assign(globalThis, { window: dom.window, document: dom.window.document, HTMLElement: dom.window.HTMLElement, Element: dom.window.Element, Node: dom.window.Node, DocumentFragment: dom.window.DocumentFragment, MutationObserver: dom.window.MutationObserver, ShadowRoot: dom.window.ShadowRoot, getComputedStyle: dom.window.getComputedStyle, IS_REACT_ACT_ENVIRONMENT: true });
  Object.defineProperty(globalThis, "navigator", { configurable: true, value: dom.window.navigator });
  globalThis.requestAnimationFrame = (callback: FrameRequestCallback) => setTimeout(() => callback(Date.now()), 0) as unknown as number;
  globalThis.cancelAnimationFrame = (id: number) => clearTimeout(id);
  Object.defineProperty(window, "matchMedia", { value: () => ({ matches: false, addListener() {}, removeListener() {}, addEventListener() {}, removeEventListener() {}, dispatchEvent: () => false }) });
}
