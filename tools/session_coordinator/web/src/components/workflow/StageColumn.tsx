import { Stack, Typography } from "@mui/material";
import type { WorkflowNode } from "../../api/contracts";
import { WorkflowNodeCard } from "./WorkflowNodeCard";

export function StageColumn({ title, nodes, onOpen }: { title: string; nodes: WorkflowNode[]; onOpen: (node: WorkflowNode) => void }) {
  return <Stack component="section" spacing={1} sx={{ minWidth: 220 }} aria-label={`${title} 阶段`}>
    <Typography variant="h6">{title} <small>({nodes.length})</small></Typography>
    {nodes.map((node) => <WorkflowNodeCard key={node.nodeId} node={node} onOpen={onOpen} />)}
    {!nodes.length && <Typography variant="body2">此阶段暂无节点</Typography>}
  </Stack>;
}
