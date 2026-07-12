import type { FinalizeRequestProjection } from "../../api/contracts";
import { BoundedTable } from "../BoundedTable";
import { StatusText } from "../StatusText";

const val = (row: FinalizeRequestProjection, key: keyof FinalizeRequestProjection) => String(row[key] ?? "—");
export function MilestoneCommitEvidence({ requests }: { requests: FinalizeRequestProjection[] }) {
  return <BoundedTable rows={requests} rowKey={(row) => val(row, "request_id")} columns={[
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
  ]} />;
}
