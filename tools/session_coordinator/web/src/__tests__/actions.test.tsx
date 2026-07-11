import assert from "node:assert/strict";
import test from "node:test";
import { renderToStaticMarkup } from "react-dom/server";
import { canUseAction } from "../actions/catalog";
import type { ActionRecord } from "../api/contracts";
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
