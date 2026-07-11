import type { WorkflowNode } from "../../api/contracts";

export const workflowStages = ["goal", "preflight", "implementation", "validation", "review", "commit", "notification"] as const;

export function nodesByStage(nodes: WorkflowNode[]): Map<string, WorkflowNode[]> {
  const stages = new Map<string, WorkflowNode[]>(workflowStages.map((stage) => [stage, []]));
  for (const node of nodes) {
    const key = node.stage.toLowerCase();
    const bucket = stages.get(key) ?? [];
    bucket.push(node);
    stages.set(key, bucket);
  }
  return stages;
}

export function currentAttempt(nodes: WorkflowNode): WorkflowNode["currentAttempt"] {
  const accepted = nodes.attemptHistory.filter((attempt) => attempt.accepted);
  return accepted.sort((a, b) => b.attemptNumber - a.attemptNumber)[0] ?? null;
}
