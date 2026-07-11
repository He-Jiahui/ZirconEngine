import { Box } from "@mui/material";
import { useState } from "react";
import type { FailureNode, LeaseProjection, WorkflowDetail, WorkflowNode } from "../../api/contracts";
import { NodeDetailDrawer } from "./NodeDetailDrawer";
import { nodesByStage, workflowStages } from "./graphLayout";
import { StageColumn } from "./StageColumn";

const titles: Record<string, string> = { goal: "目标", preflight: "预检", implementation: "实施", validation: "验证", review: "审查", commit: "提交", notification: "通知" };
export function WorkflowPipeline({ detail, leases, failures }: { detail: WorkflowDetail; leases: LeaseProjection[]; failures: FailureNode[] }) {
  const [selected, setSelected] = useState<WorkflowNode | null>(null);
  const grouped = nodesByStage(detail.nodes);
  return <><Box sx={{ display: "flex", gap: 2, overflow: "auto", pb: 1 }}>{workflowStages.map((stage) => <StageColumn key={stage} title={titles[stage]} nodes={grouped.get(stage) ?? []} onOpen={setSelected} />)}</Box><NodeDetailDrawer node={selected} planPath={detail.planPath} edges={detail.edges} artifacts={detail.artifacts} leases={leases} failures={failures} onClose={() => setSelected(null)} /></>;
}
