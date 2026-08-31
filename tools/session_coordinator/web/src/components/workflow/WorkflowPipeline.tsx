import { Box, Stack, Typography } from "@mui/material";
import { useState } from "react";
import type { FailureNode, LeaseProjection, WorkflowDetail, WorkflowNode } from "../../api/contracts";
import { NodeDetailDrawer } from "./NodeDetailDrawer";
import { nodesByStage, workflowStages } from "./graphLayout";
import { StageColumn } from "./StageColumn";
import { StageRail } from "../dashboard/StageRail";

const titles: Record<string, string> = { goal: "目标", preflight: "预检", implementation: "实施", validation: "验证", review: "审查", commit: "提交", notification: "通知" };
export function WorkflowPipeline({ detail, leases, failures }: { detail: WorkflowDetail; leases: LeaseProjection[]; failures: FailureNode[] }) {
  const [selected, setSelected] = useState<WorkflowNode | null>(null);
  const grouped = nodesByStage(detail.nodes);
  const rail = workflowStages.map((stage) => {
    const nodes = grouped.get(stage) ?? [];
    const failed = nodes.filter((node) => ["failed", "blocked"].includes(node.state)).length;
    const running = nodes.filter((node) => ["running", "leased"].includes(node.state)).length;
    return { label: titles[stage], state: failed ? "blocked" as const : running ? "active" as const : nodes.length && nodes.every((node) => ["succeeded", "skipped"].includes(node.state)) ? "done" as const : "queued" as const, detail: `${nodes.length} 节点${failed ? ` · ${failed} 失败` : running ? ` · ${running} 运行中` : ""}` };
  });
  return <><Stack spacing={1.5}><Typography variant="body2" color="text.secondary">阶段从左至右推进。每个节点保留不可变尝试历史；点击卡片可查看命令、产物、门禁和适用 Failure。</Typography><StageRail stages={rail} ariaLabel="工作流阶段" /><Box sx={{ display: "flex", gap: 1.5, overflowX: "auto", pb: 1, scrollSnapType: "x proximity" }}>{workflowStages.map((stage) => <StageColumn key={stage} title={titles[stage]} nodes={grouped.get(stage) ?? []} onOpen={setSelected} />)}</Box></Stack><NodeDetailDrawer node={selected} planPath={detail.planPath} edges={detail.edges} artifacts={detail.artifacts} leases={leases} failures={failures} gates={detail.gates} reviews={detail.reviews} notifications={detail.notifications} onClose={() => setSelected(null)} /></>;
}
