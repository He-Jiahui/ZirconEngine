import assert from "node:assert/strict";
import test from "node:test";
import { renderToStaticMarkup } from "react-dom/server";
import { canUseAction } from "../actions/catalog";
import { actionClient } from "../actions/actionClient";
import { actionMutationBlockReason, buildActionParameters } from "../actions/actionParameters";
import { pollActionUntilTerminal } from "../actions/actionTracking";
import type { ActionRecord, ServiceProjection } from "../api/contracts";
import { ActionActivityList } from "../components/actions/ActionActivityList";
import { ImpactDiff } from "../components/actions/ActionDialog";
import { RiskSummary } from "../components/actions/RiskSummary";

test("role ordering and disabled red actions are explicit", () => {
  assert.equal(canUseAction("operator", "operator", true), true);
  assert.equal(canUseAction("observer", "operator", true), false);
  assert.equal(canUseAction("maintainer", "committer", false), false);
});

test("risk summary renders impact without HTML interpretation", () => {
  const action: ActionRecord = {
    actionId: "action-a", kind: "lease.claim_own_scope", risk: "yellow", requiredRole: "operator", actor: "cli", boundSessionId: "session-a",
    parameters: { sessionId: "session-a" }, impact: ["<script>alert(1)</script>"], warnings: ["必须重新预览"], stateFingerprint: "0123456789abcdef",
    status: "previewed", createdAt: "now", expiresAt: "later", reason: null, result: null, errorCode: null, confirmationPhrase: "CONFIRM LEASE.CLAIM_OWN_SCOPE",
  };
  const html = renderToStaticMarkup(<RiskSummary action={action} />);
  assert.match(html, /受控变更/);
  assert.match(html, /&lt;script&gt;alert\(1\)&lt;\/script&gt;/);
  assert.doesNotMatch(html, /<script>/);
});

test("state change renders the fresh impact diff without executing", () => {
  const previous: ActionRecord = {
    actionId: "action-old", kind: "patch.process", risk: "yellow", requiredRole: "operator", actor: "cli", boundSessionId: "session-a",
    parameters: { sessionId: "session-a" }, impact: ["保留影响", "旧影响"], warnings: [], stateFingerprint: "aaaaaaaaaaaaaaaa",
    status: "state_changed", createdAt: "now", expiresAt: "later", reason: "test", result: null, errorCode: "action_state_changed", confirmationPhrase: "CONFIRM PATCH.PROCESS",
  };
  const fresh = { ...previous, actionId: "action-new", status: "previewed" as const, impact: ["保留影响", "新影响"], stateFingerprint: "bbbbbbbbbbbbbbbb", errorCode: null };

  const html = renderToStaticMarkup(<ImpactDiff comparison={{ previous, fresh }} />);

  assert.match(html, /新增影响：新影响/);
  assert.match(html, /移除影响：旧影响/);
  assert.match(html, /尚未执行/);
  assert.match(html, /aaaaaaaaaaaa.*bbbbbbbbbbbb/);
});

test("lifecycle preview parameters contain only the bounded service timeout", () => {
  const parameters = buildActionParameters("service.restart", {
    sessionId: "session-a",
    template: "web-check",
    jobId: "",
    runId: "run-a",
    milestoneId: "M6",
    lifecycleTimeoutSeconds: 90,
    review: null,
  });

  assert.deepEqual(parameters, { timeoutSeconds: 90 });
  assert.throws(
    () => buildActionParameters("service.stop", {
      sessionId: "session-a",
      template: "web-check",
      jobId: "",
      runId: "run-a",
      milestoneId: "M6",
      lifecycleTimeoutSeconds: 0,
      review: null,
    }),
    /1–300/,
  );
});

test("executing action tracking reaches and reports the terminal record", async () => {
  const executing = actionRecord({ actionId: "action-running", kind: "service.restart", status: "executing" });
  const succeeded = actionRecord({ actionId: "action-running", kind: "service.restart", status: "succeeded", result: { state: "healthy" } });
  const responses = [executing, succeeded];
  const updates: string[] = [];

  const terminal = await pollActionUntilTerminal(executing, {
    lookup: async () => ({ action: responses.shift() ?? succeeded }),
    wait: async () => undefined,
    onUpdate: (action) => updates.push(action.status),
  });

  assert.equal(terminal.status, "succeeded");
  assert.deepEqual(updates, ["executing", "succeeded"]);
});

test("action activity displays tracking errors beside the affected action", () => {
  const html = renderToStaticMarkup(<ActionActivityList
    actions={[]}
    trackingErrors={{ "action-running": "服务重启后需要重新打开控制台以恢复查询" }}
  />);

  assert.match(html, /action-runni/);
  assert.match(html, /服务重启后需要重新打开控制台以恢复查询/);
});

test("executing lifecycle activity exposes an explicit drain cancellation control", () => {
  const html = renderToStaticMarkup(<ActionActivityList
    actions={[actionRecord({ kind: "service.stop", status: "executing" })]}
    trackingErrors={{}}
    onCancelExecuting={() => undefined}
  />);

  assert.match(html, /取消排空并恢复服务/);
});

test("persistent action activity loads safe audit fields", async () => {
  const originalFetch = globalThis.fetch;
  let requested = "";
  globalThis.fetch = async (input) => {
    requested = String(input);
    return new Response(JSON.stringify({ ok: true, data: { actions: [actionRecord({ actor: "operator-a", reason: "发布前重启", result: { state: "healthy" }, errorCode: "none" })], truncated: false, limit: 50 }, meta: { apiVersion: 1, correlationId: "test" } }), { status: 200, headers: { "Content-Type": "application/json" } });
  };
  try {
    const activity = await actionClient.activity();
    assert.equal(requested, "/control/v1/actions?limit=50");
    assert.equal(activity.actions[0]?.actor, "operator-a");
    const html = renderToStaticMarkup(<ActionActivityList actions={activity.actions} trackingErrors={{}} />);
    assert.match(html, /operator-a/);
    assert.match(html, /发布前重启/);
    assert.match(html, /healthy/);
    assert.match(html, /none/);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("unsafe service projection disables mutation before preview", () => {
  const service = (mode: string, state: string): ServiceProjection => ({
    status: "ok", branch: "main", mode, baseline: "healthy", instanceId: "i", startedAt: "now", controlApiVersions: [1],
    supervision: { state, busy: false, blockers: [] },
  });
  assert.match(actionMutationBlockReason(service("read_only", "read_only")) ?? "", /只读/);
  assert.match(actionMutationBlockReason(service("read_write", "identity_mismatch")) ?? "", /身份/);
  assert.match(actionMutationBlockReason(service("read_write", "fatal_integrity_error")) ?? "", /完整性/);
  assert.equal(actionMutationBlockReason(service("read_write", "healthy")), null);
});

function actionRecord(overrides: Partial<ActionRecord> = {}): ActionRecord {
  return {
    actionId: "action-a", kind: "lease.claim_own_scope", risk: "yellow", requiredRole: "operator", actor: "cli", boundSessionId: "session-a",
    parameters: { sessionId: "session-a" }, impact: [], warnings: [], stateFingerprint: "0123456789abcdef",
    status: "previewed", createdAt: "now", expiresAt: "later", reason: null, result: null, errorCode: null, confirmationPhrase: null,
    ...overrides,
  };
}
