import test from "node:test";
import assert from "node:assert/strict";
import type { FailureNode } from "../api/contracts";
import { failureReviewWindow } from "../components/failure/FailureGraph";
import { failureClass, failureReviewItems } from "../components/failure/failureModel";
test("failure classification keeps foreign diagnostics non-applicable", () => assert.equal(failureClass({ status: "open", applicable: false }), "foreign"));
test("failure classification shows fixed nodes", () => assert.equal(failureClass({ status: "fixed" }), "fixed"));

test("failure review queue puts open high-priority failures before verified history", () => {
  const items = failureReviewItems([
    failure(3, "fixed", 0, "verified-history"),
    failure(2, "open", 2, "normal-open"),
    failure(1, "open", 0, "urgent-open"),
  ]);

  assert.deepEqual(items.map((item) => item.node_id), [1, 2, 3]);
  assert.equal(items[0]?.reviewState, "needs_review");
  assert.equal(items[2]?.reviewState, "verified");
});

test("failure review window follows bounded history and retains fixed chains", () => {
  const nodes = [failure(1, "open", 0, "urgent-open"), failure(2, "open", 2, "normal-open"), failure(3, "fixed", 0, "fixed-history")];
  const items = failureReviewWindow(nodes, {
    chains: [nodes[0]!, nodes[2]!].map((node) => ({
      lifecycleKey: node.lifecycle_key,
      summarySlug: node.summary_slug,
      status: node.status,
      priority: node.priority,
      originPlan: node.origin_plan,
      fixingPlan: node.fixing_plan,
      artifactPath: node.artifact_path,
      createdAt: node.created_at,
      resolvedAt: node.resolved_at,
      events: [{ kind: node.status === "fixed" ? "fixed" as const : "added" as const, createdAt: node.created_at, artifactPath: node.artifact_path }],
    })),
    statusCounts: { open: 2, fixed: 1 },
    truncated: true,
  });
  assert.deepEqual(items.map((item) => item.node_id), [1, 3]);
});

function failure(node_id: number, status: FailureNode["status"], priority: number, summary_slug: string): FailureNode {
  return { node_id, status, priority, summary_slug, lifecycle_key: `lifecycle-${node_id}`, artifact_path: `docs/failure-${node_id}.md`, kind: status === "fixed" ? "fixed" : "failure", created_at: "now", resolved_at: status === "fixed" ? "now" : null, origin_plan: "docs/origin.md", fixing_plan: "docs/fixing.md", origin_child_dir: "origin", fixing_child_dir: "fixing", imported_at: "now" };
}
