import type { CargoLaneProjection } from "../../api/contracts";
import { BoundedTable } from "../BoundedTable";
import { StatusText } from "../StatusText";

const value = (row: CargoLaneProjection, key: keyof CargoLaneProjection) => String(row[key] ?? "—");
const policyLabel = (policy: CargoLaneProjection["cleanup_policy"]) => policy === "retained" ? "可复用" : "用后即删";
const cleanupLabel: Record<CargoLaneProjection["cleanup_status"], string> = {
  retained: "保留中",
  pending: "待清理",
  deleted: "已清理",
  failed: "清理失败",
};
const processObservationLabel: Record<CargoLaneProjection["process_observation"], string> = {
  not_applicable: "—",
  awaiting_observation: "等待进程观察",
  observed: "进程已观察；心跳慢不会中断",
  reconciling: "进程退出待收束",
};
function duration(row: CargoLaneProjection) {
  if (!row.started_at || !row.finished_at) return "运行中/未记录";
  const milliseconds = Date.parse(row.finished_at) - Date.parse(row.started_at);
  if (!Number.isFinite(milliseconds) || milliseconds < 0) return "时间无效";
  const seconds = Math.round(milliseconds / 1000);
  return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
}
export function ValidationLaneTable({ jobs }: { jobs: CargoLaneProjection[] }) {
  return <BoundedTable rows={jobs} rowKey={(row) => value(row, "job_id")} columns={[
    { key: "session", label: "Session", render: (row) => value(row, "session_id") },
    { key: "lane", label: "验证通道", render: (row) => value(row, "lane_kind") },
    { key: "state", label: "状态", render: (row) => <StatusText value={value(row, "status")} /> },
    { key: "process", label: "进程观察", render: (row) => processObservationLabel[row.process_observation] },
    { key: "duration", label: "持续时间", render: duration },
    { key: "policy", label: "产物策略", render: (row) => policyLabel(row.cleanup_policy) },
    { key: "cleanup", label: "清理状态", render: (row) => <StatusText value={cleanupLabel[row.cleanup_status]} /> },
    { key: "time", label: "创建时间", render: (row) => value(row, "created_at") },
  ]} />;
}
