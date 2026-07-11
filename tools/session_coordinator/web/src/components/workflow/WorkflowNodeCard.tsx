import { Card, CardActionArea, CardContent, Stack, Typography } from "@mui/material";
import type { WorkflowNode } from "../../api/contracts";
import { StatusText } from "../StatusText";

export function WorkflowNodeCard({ node, onOpen }: { node: WorkflowNode; onOpen: (node: WorkflowNode) => void }) {
  const attempt = node.currentAttempt;
  return <Card variant="outlined">
    <CardActionArea onClick={() => onOpen(node)} aria-label={`查看节点 ${node.title}`}>
      <CardContent>
        <Stack direction="row" spacing={1} sx={{ alignItems: "center" }}><StatusText value={node.state} /><Typography sx={{ fontWeight: 700 }}>{node.title}</Typography></Stack>
        <Typography variant="caption">会话：{node.ownerSessionId ?? "未分配"} · 尝试：{attempt?.attemptNumber ?? 0}</Typography>
        {node.statusReason && <Typography variant="body2">原因：{node.statusReason}</Typography>}
      </CardContent>
    </CardActionArea>
  </Card>;
}
