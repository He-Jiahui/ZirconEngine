import { Box, Button, Chip, List, ListItemButton, Stack, Typography } from "@mui/material";
import { useEffect, useState } from "react";
import type { ValidationHistoryEvent, ValidationHistoryProjection, ValidationHistoryTicket } from "../../api/contracts";
import { StatusText } from "../StatusText";
import { StageRail } from "../dashboard/StageRail";

export function ValidationHistory({ history, loading, onLoadMore }: { history: ValidationHistoryProjection; loading: boolean; onLoadMore: () => void }) {
  const tickets = history.tickets;
  const [selectedId, setSelectedId] = useState<string | null>(tickets[0]?.ticketId ?? null);
  useEffect(() => {
    if (!tickets.some((ticket) => ticket.ticketId === selectedId)) setSelectedId(tickets[0]?.ticketId ?? null);
  }, [tickets, selectedId]);
  const selected = tickets.find((ticket) => ticket.ticketId === selectedId) ?? tickets[0] ?? null;
  const counts = history.statusCounts;
  return <Stack spacing={1.5} aria-label="验证历史记录">
    <Stack direction={{ xs: "column", md: "row" }} spacing={1} sx={{ alignItems: { md: "center" } }}>
      <Typography variant="body2" color="text.secondary" sx={{ flex: 1 }}>每个 ticket 使用不可变源码快照，时间线记录提交、物化、运行和终态。</Typography>
      <Chip size="small" color="success" variant="outlined" label={`通过 ${counts.passed}`} />
      <Chip size="small" color="error" variant="outlined" label={`失败 ${counts.failed}`} />
      <Chip size="small" color="warning" variant="outlined" label={`快照失效 ${counts.snapshot_stale}`} />
      <Chip size="small" variant="outlined" label={`进行中 ${counts.queued + counts.materializing + counts.running}`} />
    </Stack>
    {!tickets.length && <Typography color="text.secondary">尚无验证历史。</Typography>}
    {selected && <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", lg: "minmax(300px, 0.85fr) minmax(0, 1.65fr)" }, gap: 2, minWidth: 0 }}>
      <List component="nav" aria-label="验证历史 ticket" sx={{ border: 1, borderColor: "divider", borderRadius: 1, p: 0, maxHeight: { xs: "42vh", lg: "68vh" }, overflow: "auto" }}>
        {tickets.map((ticket) => <ListItemButton key={ticket.ticketId} selected={ticket.ticketId === selected.ticketId} onClick={() => setSelectedId(ticket.ticketId)} sx={{ alignItems: "flex-start", gap: 1, py: 1.25 }}>
          <StatusText value={ticket.status} />
          <Stack spacing={0.25} sx={{ minWidth: 0 }}>
            <Typography variant="body2" sx={{ fontWeight: 600, overflowWrap: "anywhere" }}>{ticket.sessionId}</Typography>
            <Typography variant="caption" color="text.secondary">{ticketDuration(ticket)} · {formatHistoryTime(ticket.updatedAt)}</Typography>
          </Stack>
        </ListItemButton>)}
      </List>
      <ValidationHistoryDetail ticket={selected} />
    </Box>}
    {history.truncated && <Button variant="outlined" size="small" disabled={loading} onClick={onLoadMore} sx={{ alignSelf: "flex-start" }}>{loading ? "正在加载" : "加载更多历史"}</Button>}
  </Stack>;
}

function ValidationHistoryDetail({ ticket }: { ticket: ValidationHistoryTicket }) {
  return <Stack component="article" spacing={2} sx={{ minWidth: 0, border: 1, borderColor: "divider", borderRadius: 1, p: { xs: 1.5, md: 2.5 } }}>
    <Stack direction={{ xs: "column", sm: "row" }} spacing={1} sx={{ alignItems: { sm: "center" } }}>
      <Stack spacing={0.25} sx={{ flex: 1, minWidth: 0 }}>
        <Typography variant="overline" color="text.secondary">Validation ticket</Typography>
        <Typography variant="h6" sx={{ overflowWrap: "anywhere" }}>{ticket.ticketId}</Typography>
      </Stack>
      <StatusText value={ticket.status} />
    </Stack>
    <StageRail ariaLabel="ticket 生命周期" stages={ticketStageRail(ticket)} />
    <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", sm: "repeat(2, minmax(0, 1fr))" }, gap: 1 }}>
      <HistoryFact label="责任 Session" value={ticket.sessionId} />
      <HistoryFact label="总耗时" value={ticketDuration(ticket)} />
      <HistoryFact label="计划" value={ticket.planPath} />
      <HistoryFact label="源码快照" value={ticket.sourceManifestHash} />
    </Box>
    <Stack spacing={0.5}>
      <Typography variant="subtitle2">验证命令</Typography>
      <Typography component="code" variant="body2" sx={{ overflowWrap: "anywhere", fontFamily: "monospace" }}>{ticket.command.join(" ") || "命令不可用"}{ticket.commandTruncated ? " ..." : ""}</Typography>
    </Stack>
    <Stack spacing={1}>
      <Typography variant="subtitle2">执行时间线</Typography>
      {ticket.events.map((event) => <ValidationTimelineEvent key={event.eventId} event={event} />)}
      {ticket.eventsTruncated && <Typography variant="caption" color="text.secondary">较早事件已按单 ticket 64 条上限截断。</Typography>}
    </Stack>
  </Stack>;
}

function ticketStageRail(ticket: ValidationHistoryTicket) {
  const rank: Record<string, number> = { queued: 0, materializing: 1, running: 2, passed: 3, failed: 3, snapshot_stale: 3 };
  const current = rank[ticket.status] ?? 0;
  const state = (stage: number): "done" | "active" | "queued" | "blocked" => ticket.status === "failed" && stage === 3 ? "blocked" : current > stage ? "done" : current === stage ? "active" : "queued";
  return [
    { label: "排队", state: state(0), detail: "ticket 已登记" },
    { label: "物化", state: state(1), detail: "创建不可变源码副本" },
    { label: "运行", state: state(2), detail: "执行验证命令" },
    { label: "结论", state: state(3), detail: ticket.status === "snapshot_stale" ? "源码快照失效" : ticket.status === "failed" ? "失败，需修复" : ticket.status === "passed" ? "通过" : "等待终态" },
  ];
}

function ValidationTimelineEvent({ event }: { event: ValidationHistoryEvent }) {
  const facts = [
    event.phase ? `阶段 ${event.phase}` : null,
    event.errorCode ? `错误 ${event.errorCode}` : null,
    event.exitCode !== null ? `退出码 ${event.exitCode}` : null,
    event.jobId ? `Job ${event.jobId}` : null,
    event.runId ? `Run ${event.runId}` : null,
  ].filter(Boolean).join(" · ");
  return <Stack spacing={0.25} sx={{ borderLeft: 2, borderColor: event.toStatus === "failed" ? "error.main" : event.toStatus === "passed" ? "success.main" : "divider", pl: 1.25 }}>
    <Typography variant="body2" sx={{ fontWeight: 600 }}>{validationEventTitle(event)}</Typography>
    <Typography variant="caption" color="text.secondary">{formatHistoryTime(event.createdAt)}{facts ? ` · ${facts}` : ""}</Typography>
  </Stack>;
}

function validationEventTitle(event: ValidationHistoryEvent): string {
  if (event.toStatus) return event.fromStatus ? `${event.fromStatus} → ${event.toStatus}` : `已进入 ${event.toStatus}`;
  if (event.type === "validation.ticket_copy_linked") return "验证副本已关联";
  if (event.type === "validation.ticket_run_linked") return "验证运行已关联";
  return event.type;
}

function HistoryFact({ label, value }: { label: string; value: string }) {
  return <Stack spacing={0.25} sx={{ minWidth: 0, borderLeft: 2, borderColor: "divider", pl: 1 }}><Typography variant="caption" color="text.secondary">{label}</Typography><Typography variant="body2" sx={{ overflowWrap: "anywhere" }}>{value}</Typography></Stack>;
}

function formatHistoryTime(value: string): string {
  const parsed = Date.parse(value);
  return Number.isFinite(parsed) ? new Date(parsed).toLocaleString() : value;
}

export function ticketDuration(ticket: ValidationHistoryTicket, now = new Date()): string {
  const started = Date.parse(ticket.createdAt);
  const terminal = ["passed", "failed", "snapshot_stale"].includes(ticket.status);
  const finished = terminal ? Date.parse(ticket.updatedAt) : now.getTime();
  if (!Number.isFinite(started) || !Number.isFinite(finished)) return "耗时未知";
  const seconds = Math.max(0, Math.floor((finished - started) / 1000));
  if (seconds < 60) return `${seconds} 秒`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes} 分 ${seconds % 60} 秒`;
  return `${Math.floor(minutes / 60)} 小时 ${minutes % 60} 分`;
}
