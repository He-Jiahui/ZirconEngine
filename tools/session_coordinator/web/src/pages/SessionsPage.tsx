import type { CodexSessionsProjection, SessionProjection } from "../api/contracts";
import { Button, Stack, Typography } from "@mui/material";
import { useEffect, useState } from "react";
import { controlClient } from "../api/client";
import { BoundedTable } from "../components/BoundedTable";
import { StatusText } from "../components/StatusText";
import { HubPanel } from "../theme";

export function SessionsPage({ sessions, codexSessions, refreshKey = 0 }: { sessions: SessionProjection[]; codexSessions: CodexSessionsProjection; refreshKey?: number }) {
  const [detail, setDetail] = useState(codexSessions);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    const controller = new AbortController();
    controlClient.codexSessions(controller.signal).then((next) => { setDetail(next); setError(null); }).catch((reason) => {
      if (!controller.signal.aborted) setError(String(reason));
    });
    return () => controller.abort();
  }, [refreshKey]);
  const visibleCodex = detail.rows.length > 0 || codexSessions.rows.length === 0 ? detail : codexSessions;
  return <Stack spacing={2}><HubPanel title="业务 Session（计划与写入权威）"><BoundedTable rows={sessions} rowKey={(row) => row.sessionId} columns={[
  { key: "name", label: "会话", render: (row) => row.displayName ?? row.sessionId }, { key: "status", label: "枚举状态", render: (row) => <StatusText value={row.status} /> }, { key: "effectiveWait", label: "当前等待", render: effectiveWaitLabel }, { key: "reason", label: "状态摘要", render: (row) => row.statusReason ?? "—" }, { key: "plan", label: "计划", render: (row) => row.planPath ?? "—" }, { key: "heartbeat", label: "最后心跳", render: (row) => row.lastHeartbeatAt }, { key: "actions", label: "操作", render: (row) => <Button component="a" href={`/ui/actions?session=${encodeURIComponent(row.sessionId)}`} size="small">受控操作</Button> },
]} /></HubPanel><HubPanel title="Codex 来源 Session（只读存在性）"><Stack spacing={1}>{error && visibleCodex.rows.length === 0 && <Typography role="alert">{error}</Typography>}<Typography variant="body2">总数 {visibleCodex.total} · 队列 {visibleCodex.queueDepth} · 活动 {visibleCodex.stateCounts.active} · 空闲 {visibleCodex.stateCounts.idle} · 归档 {visibleCodex.stateCounts.archived} · 不可用 {visibleCodex.stateCounts.unavailable}{visibleCodex.truncated ? " · 仅显示前 1000 行" : ""}</Typography><Typography variant="caption">最近成功 {visibleCodex.lastSuccessfulAt ?? "—"} · 最近终态 {visibleCodex.lastTerminalCode ?? "—"}</Typography><BoundedTable rows={visibleCodex.rows} rowKey={(row) => row.threadId} columns={[
  { key: "thread", label: "Codex Thread", render: (row) => <span title={row.threadId}>{shortId(row.threadId)}</span> },
  { key: "state", label: "来源状态", render: (row) => <StatusText value={row.state} /> },
  { key: "location", label: "位置", render: (row) => row.sourceLocation },
  { key: "event", label: "最近事件", render: (row) => row.lastEvent },
  { key: "activity", label: "最近活动", render: (row) => row.lastActivityAt },
  { key: "sync", label: "最近同步", render: (row) => row.lastSyncedAt },
  { key: "origin", label: "来源 / CLI", render: (row) => [row.originator, row.cliVersion, row.threadSource].filter(Boolean).join(" / ") || "—" },
  { key: "binding", label: "精确业务绑定", render: (row) => row.boundSessionId ?? "未绑定" },
  { key: "diagnostic", label: "诊断代码", render: (row) => row.diagnosticCode ?? "—" },
]} /></Stack></HubPanel></Stack>;
}

function shortId(value: string): string { return value.length > 18 ? `${value.slice(0, 15)}…` : value; }

function effectiveWaitLabel(row: SessionProjection): string {
  if (row.waitKind === "validation") return "验证已排队（仅占用验证资源）";
  if (row.waitKind === "external") return "外部条件等待（不占用验证队列）";
  if (row.waitKind === "lease") return "文件作用域等待";
  return "—";
}
