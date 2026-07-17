import { Stack, Typography } from "@mui/material";
import type { JsonObject, ValidationProjection } from "../api/contracts";
import { BoundedTable } from "../components/BoundedTable";
import { ValidationLaneTable } from "../components/validation/ValidationLaneTable";
import { ArtifactLifecycleSummary } from "../components/validation/ArtifactLifecycleSummary";
import { HubPanel } from "../theme";
const value = (row: JsonObject, key: string) => String(row[key] ?? "—");

export function reservationAge(createdAt: string, now = new Date()): string {
  const created = Date.parse(createdAt);
  if (!Number.isFinite(created)) return "等待时间未知";
  return `已等待 ${Math.max(0, Math.floor((now.getTime() - created) / 60_000))} 分钟`;
}

function laneQueueSummary(validation: ValidationProjection): string[] {
  const warmSummaries = (["cpu", "gpu"] as const).flatMap((lane) => {
    const rows = validation.cargoReservations.filter((reservation) => (
      reservation.laneScope === lane && reservation.executionMode !== "burst"
    ));
    if (rows.length === 0) return [];
    const active = rows.filter((reservation) => reservation.status !== "pending").length;
    const pending = rows.filter((reservation) => reservation.status === "pending").sort((left, right) => left.queuePosition - right.queuePosition);
    const label = lane === "cpu" ? "CPU 热缓存" : "GPU";
    return [`${label}：运行 ${active} · 排队 ${pending.length}${pending[0] ? ` · 下一个 ${pending[0].sessionId}` : ""}`];
  });
  const cpuBurst = validation.cpuBurst ?? { capacity: 1, active: 0, eligiblePending: 0 };
  return [...warmSummaries, `CPU 突发 WIP：${cpuBurst.active}/${cpuBurst.capacity} · 可隔离检查 ${cpuBurst.eligiblePending}`];
}

function reservationHealthDetail(status: "pending" | "leased" | "running", expiresAt: string): string {
  if (status === "running") return "作业健康检测中；预约到期不影响运行";
  return `到期 ${expiresAt}`;
}

function reservationModeDetail(executionMode: "warm" | "burst", burstEligible: boolean, status: "pending" | "leased" | "running"): string {
  if (executionMode === "burst") return "隔离突发";
  return burstEligible && status === "pending" ? "热缓存 · 可隔离检查" : "热缓存";
}

export function ValidationPage({ validation }: { validation: ValidationProjection }) {
  const laneSummaries = laneQueueSummary(validation);
  return <Stack spacing={2}>
    <HubPanel title="验证通道队列">
      <Stack spacing={0.5}>
        <Typography variant="body2" color="text.secondary">只排验证，不阻塞 Session 注册、文件工作或看板。</Typography>
        {laneSummaries.map((summary) => <Typography key={summary} variant="body2">{summary}</Typography>)}
        {validation.cargoReservations.length === 0 ? <Typography>没有活动预约；验证可立即申请。</Typography> : validation.cargoReservations.map((reservation) => {
          const lane = reservation.laneScope === "cpu" ? "CPU" : "GPU";
          const state = reservation.status === "running" ? "运行中" : reservation.status === "leased" ? "已预约" : "等待中";
          return <Typography key={reservation.reservationId}>{lane} #{reservation.queuePosition} · {reservationModeDetail(reservation.executionMode, reservation.burstEligible, reservation.status)} · {state} · {reservation.sessionId} · {reservationAge(reservation.createdAt)} · {reservationHealthDetail(reservation.status, reservation.expiresAt)}</Typography>;
        })}
      </Stack>
    </HubPanel>
    <HubPanel title="Cargo 实时通道">
      <ArtifactLifecycleSummary lifecycle={validation.artifactLifecycle} />
      <ValidationLaneTable jobs={validation.currentCargoTargets} />
    </HubPanel>
    <HubPanel title="临时验证副本">
      <Typography>构建后由服务清理，不形成 worktree。</Typography>
      <BoundedTable rows={validation.validationCopies} rowKey={(row) => value(row, "job_id")} columns={[
        { key: "id", label: "副本", render: (row) => value(row, "job_id") },
        { key: "session", label: "Session", render: (row) => value(row, "session_id") },
        { key: "path", label: "副本根目录", render: (row) => value(row, "job_root") },
        { key: "status", label: "状态", render: (row) => value(row, "status") },
        { key: "time", label: "创建时间", render: (row) => value(row, "created_at") },
      ]} />
    </HubPanel>
  </Stack>;
}
