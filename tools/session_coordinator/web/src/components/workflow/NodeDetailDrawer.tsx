import { Box, IconButton, Link, List, ListItem, ListItemText, Stack, Typography } from "@mui/material";
import { useEffect, useRef } from "react";
import type { FailureNode, LeaseProjection, WorkflowArtifact, WorkflowDetail, WorkflowNode } from "../../api/contracts";
import { AttemptTimeline } from "./AttemptTimeline";

export function NodeDetailDrawer({ node, planPath, edges, artifacts, leases, failures, onClose }: { node: WorkflowNode | null; planPath: string | null; edges: WorkflowDetail["edges"]; artifacts: WorkflowArtifact[]; leases: LeaseProjection[]; failures: FailureNode[]; onClose: () => void }) {
  const closeRef = useRef<HTMLButtonElement>(null);
  const previousFocus = useRef<HTMLElement | null>(null);
  useEffect(() => {
    if (node) {
      previousFocus.current = document.activeElement as HTMLElement | null;
      closeRef.current?.focus();
    } else {
      previousFocus.current?.focus();
      previousFocus.current = null;
    }
  }, [node]);
  const nodeArtifacts = node ? artifacts.filter((artifact) => artifact.nodeId === null || artifact.nodeId === node.nodeId) : [];
  const dependencies = node ? edges.filter((edge) => edge.toNodeId === node.nodeId) : [];
  const dependents = node ? edges.filter((edge) => edge.fromNodeId === node.nodeId) : [];
  const nodeLeases = node ? leases.filter((lease) => lease.session_id === node.ownerSessionId).slice(0, 20) : [];
  const applicableFailures = failures.filter((failure) => !planPath || failure.origin_plan === planPath || failure.fixing_plan === planPath);
  return <Box component="aside" aria-hidden={!node} onKeyDown={(event) => { if (event.key === "Escape") onClose(); }} sx={{ display: node ? "block" : "none", position: "fixed", inset: "73px 0 0 auto", zIndex: (theme) => theme.zIndex.modal, overflowY: "auto", bgcolor: "background.paper", borderLeft: 1, borderColor: "divider", boxShadow: 12 }}>
    {node && <Stack spacing={2} sx={{ width: "min(600px, 92vw)", p: 3 }} role="dialog" aria-label={`节点详情 ${node.title}`}>
      <Stack direction="row" sx={{ alignItems: "center" }}><Typography variant="h6" sx={{ flex: 1 }}>{node.title}</Typography><IconButton ref={closeRef} onClick={onClose} aria-label="关闭节点详情"><span aria-hidden="true">×</span></IconButton></Stack>
      <Typography>阶段：{node.stage}　状态：{node.state}</Typography>
      <Typography>执行会话：{node.ownerSessionId ?? "未分配"}</Typography>
      <Typography>所属计划：{planPath ?? "未关联"}</Typography>
      <Typography>阻塞/状态摘要：{node.statusReason ?? "无"}</Typography>
      <Typography>依赖：{dependencies.map((edge) => edge.fromNodeId).join("、") || "无"}</Typography>
      <Typography>下游：{dependents.map((edge) => edge.toNodeId).join("、") || "无"}</Typography>
      <Typography variant="h6">文件租约</Typography>
      <List dense>{nodeLeases.map((lease, index) => <ListItem key={String(lease.path_key ?? index)}><ListItemText primary={String(lease.display_path ?? lease.path_key ?? "未命名租约")} secondary={`到期 ${String(lease.expires_at ?? "未知")}`} /></ListItem>)}{!nodeLeases.length && <ListItem><ListItemText primary="当前节点 Session 没有活动租约" /></ListItem>}</List>
      <Typography variant="h6">Artifact 与计划证据</Typography>
      <List dense>{nodeArtifacts.map((artifact) => <ListItem key={artifact.artifactId}><ListItemText primary={<Link href={`/control/v1/artifacts/${encodeURIComponent(artifact.artifactId)}`}>{artifact.displayName}</Link>} secondary={`${artifact.kind} · ${artifact.byteCount ?? "未知"} bytes · ${artifact.contentHash ?? "无哈希"}`} /></ListItem>)}{!nodeArtifacts.length && <ListItem><ListItemText primary="当前节点没有 Artifact" /></ListItem>}</List>
      <Typography variant="h6">适用 Failure</Typography>
      <List dense>{applicableFailures.slice(0, 20).map((failure) => <ListItem key={failure.node_id}><ListItemText primary={failure.summary_slug} secondary={`${failure.status} · ${failure.artifact_path}`} /></ListItem>)}{!applicableFailures.length && <ListItem><ListItemText primary="没有适用 Failure" /></ListItem>}</List>
      <Typography variant="h6">执行证据与历史</Typography>
      <pre className="json-evidence">{JSON.stringify(node.currentAttempt?.evidence ?? {}, null, 2)}</pre>
      <AttemptTimeline attempts={node.attemptHistory} />
    </Stack>}
  </Box>;
}
