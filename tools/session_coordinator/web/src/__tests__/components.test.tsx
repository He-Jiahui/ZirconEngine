import test from "node:test";
import assert from "node:assert/strict";
import { JSDOM } from "jsdom";
import { act, useState } from "react";
import { createRoot } from "react-dom/client";
import { ThemeProvider } from "@mui/material/styles";
import type { AuditEvent, CargoJobProjection, CodexSessionsProjection, ControlSnapshot, FailureNode, FinalizeRequestProjection, WorkflowNode } from "../api/contracts";
import { StatusText } from "../components/StatusText";
import { FixedSizeList } from "../components/audit/fixedList";
import { NodeDetailDrawer } from "../components/workflow/NodeDetailDrawer";
import { LogViewer } from "../components/logs/LogViewer";
import { controlTheme } from "../theme";
import { ValidationLaneTable } from "../components/validation/ValidationLaneTable";
import { MilestoneCommitEvidence } from "../components/git/MilestoneCommitEvidence";
import { overviewMetrics } from "../pages/OverviewPage";
import { ArtifactLifecycleSummary, artifactLifecycleCounts } from "../components/validation/ArtifactLifecycleSummary";
import { SessionsPage } from "../pages/SessionsPage";

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

test("control tables render canonical Cargo and commit projection fields", async () => {
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  const request: FinalizeRequestProjection = { request_id: "request-1", session_id: "s", message: "m", paths: ["a.rs"], categories: { code: ["a.rs"] }, untracked: ["notes.md"], validation: [["cargo", "test", "-p", "crate"]], maintenance: 0, status: "committed", commit_sha: "0123456789abcdef", error_text: null, created_at: "now", completed_at: "now" };
  const cleanupError = '<img src=x onerror="globalThis.__zirconXss=true">cleanup denied';
  await act(async () => root.render(<ThemeProvider theme={controlTheme}><ValidationLaneTable jobs={[cargoJob({ cleanup_policy: "delete_on_release", cleanup_status: "failed", cleanup_error: cleanupError, reused_from_job_id: "previous-job-123456789", compatibility_key: "compatibility-123456789" })]} /><MilestoneCommitEvidence requests={[request]} /></ThemeProvider>));
  assert.match(host.textContent ?? "", /workspace/);
  assert.match(host.textContent ?? "", /running/);
  assert.match(host.textContent ?? "", /R:\/targets\/job-1/);
  assert.match(host.textContent ?? "", /用后即删/);
  assert.match(host.textContent ?? "", /清理失败/);
  assert.match(host.textContent ?? "", /compatibilit…/);
  assert.match(host.textContent ?? "", /previous-job…/);
  assert.match(host.textContent ?? "", /<img/);
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

test("overview counts only running Cargo targets in the real-time baseline", () => {
  const snapshot = { workflows: [], sessions: [], failures: { nodes: [] }, validation: { cargoJobs: [{ status: "running" }, { status: "running" }], currentCargoTargets: [{ status: "running" }, { status: "succeeded" }] } } as unknown as ControlSnapshot;
  assert.deepEqual(overviewMetrics(snapshot), [["工作流", 0], ["活动会话", 0], ["Failure", 0], ["运行验证", 1]]);
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
function cargoJob(overrides: Partial<CargoJobProjection> = {}): CargoJobProjection { return { job_id: "job-1", session_id: "s", lane_kind: "workspace", target_dir: "R:/targets/job-1", status: "running", dry_run: 0, pid: 12, command: ["cargo", "test"], exit_code: null, created_at: "2026-07-11T00:00:00Z", last_heartbeat_at: "2026-07-11T00:01:00Z", started_at: "2026-07-11T00:00:00Z", finished_at: "2026-07-11T00:01:00Z", released_at: null, reuse_key: "reuse", compatibility_key: "compatibility", reuse_profile: "{}", reused_from_job_id: null, cleanup_policy: "retained", cleanup_status: "retained", cleanup_error: null, ...overrides }; }
function mockFetch() { globalThis.fetch = async () => new Response(JSON.stringify({ ok: true, data: { events: [], truncated: false, nextBefore: null }, meta: { apiVersion: 1, correlationId: "test" } }), { status: 200, headers: { "Content-Type": "application/json" } }); }

function setupDom() {
  const dom = new JSDOM("<!doctype html><html><body></body></html>", { url: "http://127.0.0.1:4317/ui/" });
  Object.assign(globalThis, { window: dom.window, document: dom.window.document, HTMLElement: dom.window.HTMLElement, Element: dom.window.Element, Node: dom.window.Node, DocumentFragment: dom.window.DocumentFragment, MutationObserver: dom.window.MutationObserver, ShadowRoot: dom.window.ShadowRoot, getComputedStyle: dom.window.getComputedStyle, IS_REACT_ACT_ENVIRONMENT: true });
  Object.defineProperty(globalThis, "navigator", { configurable: true, value: dom.window.navigator });
  globalThis.requestAnimationFrame = (callback: FrameRequestCallback) => setTimeout(() => callback(Date.now()), 0) as unknown as number;
  globalThis.cancelAnimationFrame = (id: number) => clearTimeout(id);
  Object.defineProperty(window, "matchMedia", { value: () => ({ matches: false, addListener() {}, removeListener() {}, addEventListener() {}, removeEventListener() {}, dispatchEvent: () => false }) });
}
