import type { CodexSessionsProjection, SessionProjection } from "../api/contracts";
import { Button, Stack, Typography } from "@mui/material";
import { BoundedTable } from "../components/BoundedTable";
import { StatusText } from "../components/StatusText";
import { HubPanel } from "../theme";

export function SessionsPage({ sessions, codexSessions }: { sessions: SessionProjection[]; codexSessions: CodexSessionsProjection }) { return <Stack spacing={2}><HubPanel title="业务 Session（计划与写入权威）"><BoundedTable rows={sessions} rowKey={(row) => row.sessionId} columns={[
  { key: "name", label: "会话", render: (row) => row.displayName ?? row.sessionId }, { key: "status", label: "枚举状态", render: (row) => <StatusText value={row.status} /> }, { key: "reason", label: "状态摘要", render: (row) => row.statusReason ?? "—" }, { key: "plan", label: "计划", render: (row) => row.planPath ?? "—" }, { key: "heartbeat", label: "最后心跳", render: (row) => row.lastHeartbeatAt }, { key: "actions", label: "操作", render: (row) => <Button component="a" href={`/ui/actions?session=${encodeURIComponent(row.sessionId)}`} size="small">受控操作</Button> },
]} /></HubPanel><HubPanel title="Codex 来源 Session（只读存在性）"><Stack spacing={1}><Typography variant="body2">总数 {codexSessions.total} · 队列 {codexSessions.queueDepth} · 活动 {codexSessions.stateCounts.active} · 空闲 {codexSessions.stateCounts.idle} · 归档 {codexSessions.stateCounts.archived} · 不可用 {codexSessions.stateCounts.unavailable}{codexSessions.truncated ? " · 仅显示前 1000 行" : ""}</Typography><Typography variant="caption">最近成功 {codexSessions.lastSuccessfulAt ?? "—"} · 最近终态 {codexSessions.lastTerminalCode ?? "—"}</Typography><BoundedTable rows={codexSessions.rows} rowKey={(row) => row.threadId} columns={[
  { key: "thread", label: "Codex Thread", render: (row) => <span title={row.threadId}>{shortId(row.threadId)}</span> },
  { key: "state", label: "来源状态", render: (row) => <StatusText value={row.state} /> },
  { key: "location", label: "位置", render: (row) => row.sourceLocation },
  { key: "event", label: "最近事件", render: (row) => row.lastEvent },
  { key: "activity", label: "最近活动", render: (row) => row.lastActivityAt },
  { key: "sync", label: "最近同步", render: (row) => row.lastSyncedAt },
  { key: "origin", label: "来源 / CLI", render: (row) => [row.originator, row.cliVersion, row.threadSource].filter(Boolean).join(" / ") || "—" },
  { key: "binding", label: "精确业务绑定", render: (row) => row.boundSessionId ?? "未绑定" },
  { key: "diagnostic", label: "诊断代码", render: (row) => row.diagnosticCode ?? "—" },
]} /></Stack></HubPanel></Stack>; }

function shortId(value: string): string { return value.length > 18 ? `${value.slice(0, 15)}…` : value; }
