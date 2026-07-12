import type { CargoJobProjection } from "../../api/contracts";
import { BoundedTable } from "../BoundedTable";
import { StatusText } from "../StatusText";

const value = (row: CargoJobProjection, key: keyof CargoJobProjection) => String(row[key] ?? "—");
const policyLabel = (policy: CargoJobProjection["cleanup_policy"]) => policy === "retained" ? "可复用" : "用后即删";
const cleanupLabel: Record<CargoJobProjection["cleanup_status"], string> = {
  retained: "保留中",
  pending: "待清理",
  deleted: "已清理",
  failed: "清理失败",
};
function shortIdentity(identity: string | null) {
  if (!identity) return "—";
  const text = identity.length > 12 ? `${identity.slice(0, 12)}…` : identity;
  return <span title={identity}>{text}</span>;
}
function duration(row: CargoJobProjection) {
  if (!row.started_at || !row.finished_at) return "运行中/未记录";
  const milliseconds = Date.parse(row.finished_at) - Date.parse(row.started_at);
  if (!Number.isFinite(milliseconds) || milliseconds < 0) return "时间无效";
  const seconds = Math.round(milliseconds / 1000);
  return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
}
export function ValidationLaneTable({ jobs }: { jobs: CargoJobProjection[] }) {
  return <BoundedTable rows={jobs} rowKey={(row) => value(row, "job_id")} columns={[
    { key: "session", label: "Session", render: (row) => value(row, "session_id") },
    { key: "lane", label: "验证通道", render: (row) => value(row, "lane_kind") },
    { key: "state", label: "状态", render: (row) => <StatusText value={value(row, "status")} /> },
    { key: "policy", label: "产物策略", render: (row) => policyLabel(row.cleanup_policy) },
    { key: "compatibility", label: "兼容键", render: (row) => shortIdentity(row.compatibility_key) },
    { key: "reused", label: "复用来源", render: (row) => shortIdentity(row.reused_from_job_id) },
    { key: "cleanup", label: "清理状态", render: (row) => <StatusText value={cleanupLabel[row.cleanup_status]} /> },
    { key: "cleanup-error", label: "清理错误", render: (row) => value(row, "cleanup_error") },
    { key: "target", label: "Cargo 目标根", render: (row) => value(row, "target_dir") },
    { key: "command", label: "托管命令", render: (row) => row.command.join(" ") },
    { key: "pid", label: "PID", render: (row) => value(row, "pid") },
    { key: "exit", label: "退出码", render: (row) => value(row, "exit_code") },
    { key: "duration", label: "持续时间", render: duration },
    { key: "heartbeat", label: "最后心跳", render: (row) => row.last_heartbeat_at },
    { key: "profile", label: "复用配置", render: (row) => row.reuse_profile ?? "—" },
    { key: "time", label: "创建时间", render: (row) => value(row, "created_at") },
  ]} />;
}
