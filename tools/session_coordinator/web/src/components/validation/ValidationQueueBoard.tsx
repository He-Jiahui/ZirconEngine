import { Box, Chip, Stack, Typography } from "@mui/material";
import type { CargoReservationProjection, CpuBurstProjection } from "../../api/contracts";
import { StatusText } from "../StatusText";
import { validationQueueLanes } from "./validationQueueModel";

function queueAge(createdAt: string, now = Date.now()): string {
  const timestamp = Date.parse(createdAt);
  if (!Number.isFinite(timestamp)) return "等待时间未知";
  const minutes = Math.max(0, Math.floor((now - timestamp) / 60_000));
  return `${minutes} 分钟`;
}

function modeLabel(reservation: CargoReservationProjection): string {
  if (reservation.executionMode === "burst") return "隔离突发";
  return reservation.burstEligible && reservation.status === "pending" ? "热缓存 · 可隔离检查" : "热缓存";
}

function healthDetail(reservation: CargoReservationProjection): string {
  if (reservation.status === "running") return "作业健康检测中；预约到期不影响运行";
  if (reservation.status === "leased") return `预启动租约：到期 ${reservation.expiresAt} 将自动释放给下一项`;
  return "活跃 Session 保持队位；失联才释放";
}

export function ValidationQueueBoard({ reservations, cpuBurst }: { reservations: CargoReservationProjection[]; cpuBurst: CpuBurstProjection }) {
  const lanes = validationQueueLanes(reservations);
  return <Stack spacing={1.5} aria-label="构建与验证队列">
    <Stack direction={{ xs: "column", sm: "row" }} spacing={1} sx={{ alignItems: { sm: "center" } }}>
      <Typography variant="body2" color="text.secondary" sx={{ flex: 1 }}>只排验证，不阻塞 Session 注册、文件工作或 failure 修复。</Typography>
      <Chip size="small" label={`CPU 突发 WIP：${cpuBurst.active}/${cpuBurst.capacity} · 可隔离检查 ${cpuBurst.eligiblePending}`} color={cpuBurst.active ? "warning" : "default"} />
    </Stack>
    <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", lg: "repeat(2, minmax(0, 1fr))" }, gap: 1.5 }}>
      {lanes.map((lane) => <Box component="section" key={lane.scope} aria-label={`${lane.label} 队列`} sx={{ minWidth: 0, border: 1, borderColor: "divider", borderRadius: 1, overflow: "hidden" }}>
        <Stack direction="row" spacing={1} sx={{ px: 1.5, py: 1, alignItems: "center", borderBottom: 1, borderColor: "divider", bgcolor: "action.hover" }}>
          <Typography variant="subtitle2" sx={{ flex: 1 }}>{lane.label}</Typography>
          <Typography variant="caption" color="text.secondary">运行 {lane.running} · 预约 {lane.leased} · 排队 {lane.pending}</Typography>
        </Stack>
        {!lane.items.length && <Typography variant="body2" color="text.secondary" sx={{ p: 1.5 }}>当前通道空闲，可立即申请验证。</Typography>}
        {lane.items.map((reservation) => <Stack key={reservation.reservationId} direction="row" spacing={1} sx={{ alignItems: "center", px: 1.5, py: 1.25, borderBottom: 1, borderColor: "divider", "&:last-child": { borderBottom: 0 } }}>
          <Typography variant="caption" sx={{ width: 28, fontVariantNumeric: "tabular-nums" }}>#{reservation.queuePosition}</Typography>
          <StatusText value={reservation.status === "pending" ? "排队" : reservation.status === "leased" ? "预启动" : "运行中"} />
          <Stack spacing={0} sx={{ minWidth: 0, flex: 1 }}>
            <Typography variant="body2" noWrap title={reservation.sessionId}>{reservation.sessionId}</Typography>
            <Typography variant="caption" color="text.secondary">{lane.scope.toUpperCase()} #{reservation.queuePosition} · {modeLabel(reservation)} · {reservation.status === "running" ? "运行中" : reservation.status === "leased" ? "预启动" : "等待中"} · {reservation.sessionId} · {queueAge(reservation.createdAt)} · {healthDetail(reservation)}</Typography>
          </Stack>
        </Stack>)}
      </Box>)}
    </Box>
  </Stack>;
}
