import { Box, Stack, Typography } from "@mui/material";
import type { WorkflowNode } from "../../api/contracts";
import { WorkflowNodeCard } from "./WorkflowNodeCard";

export function StageColumn({ title, nodes, onOpen }: { title: string; nodes: WorkflowNode[]; onOpen: (node: WorkflowNode) => void }) {
  const running = nodes.filter((node) => ["running", "leased"].includes(node.state)).length;
  const failed = nodes.filter((node) => ["failed", "blocked"].includes(node.state)).length;
  const completed = nodes.filter((node) => ["succeeded", "skipped"].includes(node.state)).length;
  const borderColor = failed ? "error.main" : running ? "warning.main" : completed ? "success.main" : "divider";
  return <Stack component="section" spacing={1} sx={{ minWidth: 236, maxWidth: 280, flex: "0 0 236px", scrollSnapAlign: "start", borderTop: 3, borderColor, pt: 0.75 }} aria-label={`${title} 阶段`}>
    <Stack direction="row" spacing={1} sx={{ alignItems: "baseline" }}><Typography variant="subtitle1" sx={{ fontWeight: 700, flex: 1 }}>{title}</Typography><Typography variant="caption" color="text.secondary">{completed}/{nodes.length} 完成</Typography></Stack>
    {(running > 0 || failed > 0) && <Box sx={{ px: 1, py: 0.75, bgcolor: failed ? "error.dark" : "warning.dark", borderRadius: 1 }}><Typography variant="caption">{failed ? `${failed} 项需处理` : `${running} 项运行中`}</Typography></Box>}
    {nodes.map((node) => <WorkflowNodeCard key={node.nodeId} node={node} onOpen={onOpen} />)}
    {!nodes.length && <Typography variant="body2" color="text.secondary">此阶段暂无节点</Typography>}
  </Stack>;
}
