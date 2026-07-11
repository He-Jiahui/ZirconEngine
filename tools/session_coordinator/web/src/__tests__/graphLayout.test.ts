import test from "node:test";
import assert from "node:assert/strict";
import { currentAttempt, nodesByStage } from "../components/workflow/graphLayout";
import type { WorkflowNode } from "../api/contracts";

const node = { nodeId: "n", nodeKey: "n", kind: "goal", title: "N", stage: "validation", state: "succeeded", ownerSessionId: "s", statusReason: null,
  currentAttempt: null, attemptHistory: [
    { attemptId: "1", attemptNumber: 1, state: "failed", accepted: false, evidence: {}, startedAt: null, completedAt: null },
    { attemptId: "2", attemptNumber: 2, state: "failed", accepted: false, evidence: {}, startedAt: null, completedAt: null },
    { attemptId: "3", attemptNumber: 3, state: "succeeded", accepted: true, evidence: {}, startedAt: null, completedAt: null },
  ] } satisfies WorkflowNode;
test("workflow layout places nodes by stage", () => assert.equal(nodesByStage([node]).get("validation")?.length, 1));
test("workflow layout exposes the default goal stage", () => assert.ok(nodesByStage([{ ...node, stage: "goal" }]).get("goal")?.length));
test("workflow layout uses latest accepted attempt", () => assert.equal(currentAttempt(node)?.state, "succeeded"));
