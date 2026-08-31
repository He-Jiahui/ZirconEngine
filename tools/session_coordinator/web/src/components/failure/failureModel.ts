import type { FailureNode, JsonObject } from "../../api/contracts";

export function failureClass(node: JsonObject): "applicable" | "fixed" | "foreign" | "invalid" | "open" {
  const status = String(node.status ?? node.resolution ?? "open").toLowerCase();
  if (status.includes("invalid")) return "invalid";
  if (status.includes("fixed") || status.includes("resolved")) return "fixed";
  if (node.applicable === false || node.applicable === 0) return "foreign";
  if (node.applicable === true || node.applicable === 1) return "applicable";
  return "open";
}

export type FailureReviewState = "needs_review" | "verified";

export type FailureReviewItem = FailureNode & {
  reviewState: FailureReviewState;
};

export function failureReviewItems(nodes: FailureNode[]): FailureReviewItem[] {
  return nodes
    .map((node) => ({
      ...node,
      reviewState: node.status === "fixed" ? "verified" : "needs_review" as FailureReviewState,
    }))
    .sort((left, right) => {
      const leftRank = left.reviewState === "needs_review" ? 0 : 1;
      const rightRank = right.reviewState === "needs_review" ? 0 : 1;
      return leftRank - rightRank || left.priority - right.priority || left.node_id - right.node_id;
    });
}
