import type { FinalizeRequestProjection } from "../../api/contracts";
import { BoundedTable } from "../BoundedTable";
import { StatusText } from "../StatusText";

const val = (row: FinalizeRequestProjection, key: keyof FinalizeRequestProjection) => String(row[key] ?? "—");
export function MilestoneCommitEvidence({ requests }: { requests: FinalizeRequestProjection[] }) {
  return <BoundedTable rows={requests} rowKey={(row) => val(row, "request_id")} columns={[
    { key: "id", label: "请求", render: (row) => <code>{val(row, "request_id").slice(0, 12)}</code> },
    { key: "session", label: "会话", render: (row) => val(row, "session_id") },
    { key: "status", label: "状态", render: (row) => <StatusText value={val(row, "status")} /> },
    { key: "commit", label: "Commit", render: (row) => <code>{val(row, "commit_sha").slice(0, 12)}</code> },
    { key: "time", label: "时间", render: (row) => val(row, "created_at") },
  ]} />;
}
