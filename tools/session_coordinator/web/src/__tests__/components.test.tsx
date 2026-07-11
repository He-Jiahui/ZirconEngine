import test from "node:test";
import assert from "node:assert/strict";
import { JSDOM } from "jsdom";
import { act, useState } from "react";
import { createRoot } from "react-dom/client";
import { ThemeProvider } from "@mui/material/styles";
import type { AuditEvent, ControlSnapshot, FailureNode, FinalizeRequestProjection, WorkflowNode } from "../api/contracts";
import { StatusText } from "../components/StatusText";
import { FixedSizeList } from "../components/audit/fixedList";
import { NodeDetailDrawer } from "../components/workflow/NodeDetailDrawer";
import { LogViewer } from "../components/logs/LogViewer";
import { controlTheme } from "../theme";
import { ValidationLaneTable } from "../components/validation/ValidationLaneTable";
import { MilestoneCommitEvidence } from "../components/git/MilestoneCommitEvidence";
import { overviewMetrics } from "../pages/OverviewPage";

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

test("drawer associates failures by origin or fixing plan and returns focus", async () => {
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  function Harness() { const [selected, setSelected] = useState<WorkflowNode | null>(null); return <ThemeProvider theme={controlTheme}><button id="opener" onClick={() => setSelected(node)}>打开</button><NodeDetailDrawer node={selected} planPath="docs/plans/x/01.md" edges={[]} artifacts={[]} leases={[]} failures={[failure]} onClose={() => setSelected(null)} /></ThemeProvider>; }
  await act(async () => root.render(<Harness />));
  const opener = host.querySelector("#opener") as HTMLButtonElement; opener.focus();
  await act(async () => opener.click());
  const close = document.querySelector('button[aria-label="关闭节点详情"]') as HTMLButtonElement;
  assert.equal(document.activeElement, close);
  assert.match(document.body.textContent ?? "", /architecture-fix/);
  assert.match(document.body.textContent ?? "", /failure-2026-07-11-architecture-fix\.md/);
  await act(async () => close.click());
  assert.equal(document.activeElement, opener);
  await act(async () => root.unmount()); host.remove();
});

test("control tables render canonical Cargo and commit projection fields", async () => {
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  const request: FinalizeRequestProjection = { request_id: "request-1", session_id: "s", message: "m", paths: [], categories: {}, untracked: [], validation: [], maintenance: 0, status: "committed", commit_sha: "0123456789abcdef", error_text: null, created_at: "now", completed_at: "now" };
  await act(async () => root.render(<ThemeProvider theme={controlTheme}><ValidationLaneTable jobs={[{ job_id: "job-1", session_id: "s", lane_kind: "workspace", target_dir: "R:/targets/job-1", status: "running", dry_run: 0, pid: 12, command: ["cargo", "test"], exit_code: null, created_at: "now", last_heartbeat_at: "now", started_at: "now", finished_at: null, released_at: null }]} /><MilestoneCommitEvidence requests={[request]} /></ThemeProvider>));
  assert.match(host.textContent ?? "", /workspace/);
  assert.match(host.textContent ?? "", /running/);
  assert.match(host.textContent ?? "", /R:\/targets\/job-1/);
  assert.match(host.textContent ?? "", /0123456789ab/);
  await act(async () => root.unmount()); host.remove();
});

test("overview counts running Cargo jobs from canonical status", () => {
  const snapshot = { workflows: [], sessions: [], failures: { nodes: [] }, validation: { cargoJobs: [{ status: "running" }, { status: "succeeded" }] } } as unknown as ControlSnapshot;
  assert.deepEqual(overviewMetrics(snapshot), [["工作流", 0], ["活动会话", 0], ["Failure", 0], ["运行验证", 1]]);
});

const node: WorkflowNode = { nodeId: "n", nodeKey: "n", kind: "goal", title: "节点", stage: "implementation", state: "running", ownerSessionId: "s", statusReason: null, currentAttempt: null, attemptHistory: [] };
const failure: FailureNode = { node_id: 1, lifecycle_key: "life", artifact_path: "docs/plans/y/02/failure-2026-07-11-architecture-fix.md", kind: "failure", status: "open", created_at: "now", resolved_at: null, summary_slug: "architecture-fix", origin_plan: "docs/plans/x/01.md", fixing_plan: "docs/plans/y/02.md", origin_child_dir: "01", fixing_child_dir: "02", priority: 1, imported_at: "now" };
function audit(eventId: number, message: string): AuditEvent { return { eventId, sessionId: "s", type: "info", payload: { message }, createdAt: "2026-07-11T00:00:00Z" }; }
function mockFetch() { globalThis.fetch = async () => new Response(JSON.stringify({ ok: true, data: { events: [], truncated: false, nextBefore: null }, meta: { apiVersion: 1, correlationId: "test" } }), { status: 200, headers: { "Content-Type": "application/json" } }); }

function setupDom() {
  const dom = new JSDOM("<!doctype html><html><body></body></html>", { url: "http://127.0.0.1:4317/ui/" });
  Object.assign(globalThis, { window: dom.window, document: dom.window.document, HTMLElement: dom.window.HTMLElement, Element: dom.window.Element, Node: dom.window.Node, DocumentFragment: dom.window.DocumentFragment, MutationObserver: dom.window.MutationObserver, ShadowRoot: dom.window.ShadowRoot, getComputedStyle: dom.window.getComputedStyle, IS_REACT_ACT_ENVIRONMENT: true });
  Object.defineProperty(globalThis, "navigator", { configurable: true, value: dom.window.navigator });
  globalThis.requestAnimationFrame = (callback: FrameRequestCallback) => setTimeout(() => callback(Date.now()), 0) as unknown as number;
  globalThis.cancelAnimationFrame = (id: number) => clearTimeout(id);
  Object.defineProperty(window, "matchMedia", { value: () => ({ matches: false, addListener() {}, removeListener() {}, addEventListener() {}, removeEventListener() {}, dispatchEvent: () => false }) });
}
