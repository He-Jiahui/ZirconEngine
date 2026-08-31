import { Box, Chip, Grid, Stack, Typography } from "@mui/material";
import type { FinalizeRequestProjection, SessionProjection, WorkflowSummary } from "../../api/contracts";
import { BoundedTable } from "../BoundedTable";
import { StatusText } from "../StatusText";

const val = (row: FinalizeRequestProjection, key: keyof FinalizeRequestProjection) => String(row[key] ?? "—");
export function MilestoneCommitEvidence({ requests, sessions = [], workflows = [] }: { requests: FinalizeRequestProjection[]; sessions?: SessionProjection[]; workflows?: WorkflowSummary[] }) {
  return <Stack spacing={2} aria-label="里程碑提交看板">
    <Grid container spacing={1.25}>{requests.map((request) => {
      const session = sessions.find((item) => item.sessionId === request.session_id);
      const workflow = workflows.find((item) => item.sessionId === request.session_id);
      const state = request.status === "committed" ? "done" : request.status === "failed" ? "blocked" : request.status === "finalizing" ? "active" : "queued";
      const progress = state === "done" ? 100 : state === "active" ? 66 : state === "blocked" ? 35 : 15;
      return <Grid key={request.request_id} size={{ xs: 12, md: 6 }}><Box sx={{ p: 1.5, border: 1, borderColor: state === "blocked" ? "error.main" : "divider", borderRadius: 1, bgcolor: "background.paper" }}><Stack spacing={1}><Stack direction="row" spacing={1} sx={{ alignItems: "center" }}><Typography variant="subtitle2" sx={{ flex: 1, overflowWrap: "anywhere" }}>{request.message || "未命名里程碑"}</Typography><Chip size="small" color={state === "done" ? "success" : state === "blocked" ? "error" : state === "active" ? "primary" : "warning"} label={request.status} /></Stack><Stack direction="row" spacing={1} sx={{ alignItems: "center" }}><Box sx={{ flex: 1, height: 7, bgcolor: "action.hover", borderRadius: 999, overflow: "hidden" }}><Box sx={{ width: `${progress}%`, height: "100%", bgcolor: state === "blocked" ? "error.main" : state === "done" ? "success.main" : "primary.main" }} /></Box><Typography variant="caption">{progress}%</Typography></Stack><Typography variant="caption" color="text.secondary" sx={{ overflowWrap: "anywhere" }}>计划 {session?.planPath ?? workflow?.planPath ?? "未关联计划"} · Session {request.session_id} · {request.paths.length} 个路径</Typography><Typography variant="caption" color="text.secondary">验证证据 {request.validation.length || 0} 项{request.commit_sha ? ` · ${request.commit_sha.slice(0, 12)}` : ""}{request.error_text ? ` · ${request.error_text}` : ""}</Typography></Stack></Box></Grid>;
    })}{!requests.length && <Grid size={12}><Typography color="text.secondary">尚无里程碑提交请求。</Typography></Grid>}</Grid>
    <Typography variant="caption" color="text.secondary">下方为完整提交证据；上方按计划 → 验证 → Commit 显示当前准备度。</Typography>
    <BoundedTable rows={requests} rowKey={(row) => val(row, "request_id")} columns={[
    { key: "id", label: "请求", render: (row) => <code>{val(row, "request_id").slice(0, 12)}</code> },
    { key: "session", label: "会话", render: (row) => val(row, "session_id") },
    { key: "status", label: "状态", render: (row) => <StatusText value={val(row, "status")} /> },
    { key: "message", label: "提交信息", render: (row) => row.message },
    { key: "scope", label: "归属范围", render: (row) => `${row.paths.length} 个路径 · ${Object.entries(row.categories).map(([kind, paths]) => `${kind} ${paths.length}`).join(" · ") || "未分类"}` },
    { key: "untracked", label: "未跟踪", render: (row) => row.untracked.join("、") || "无" },
    { key: "validation", label: "验证证据", render: (row) => row.validation.map((command) => command.join(" ")).join("；") || "无" },
    { key: "maintenance", label: "维护模式", render: (row) => row.maintenance ? "是" : "否" },
    { key: "commit", label: "Commit", render: (row) => <code>{val(row, "commit_sha").slice(0, 12)}</code> },
    { key: "error", label: "错误", render: (row) => row.error_text ?? "无" },
    { key: "time", label: "时间", render: (row) => val(row, "created_at") },
    { key: "completed", label: "完成时间", render: (row) => row.completed_at ?? "—" },
  ]} />
  </Stack>;
}
