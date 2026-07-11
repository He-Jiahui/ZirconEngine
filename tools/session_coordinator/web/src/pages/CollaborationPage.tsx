import { Stack, Typography } from "@mui/material";
import type { CollaborationProjection, JsonObject } from "../api/contracts";
import { BoundedTable } from "../components/BoundedTable";
import { LeaseTable } from "../components/collaboration/LeaseTable";
import { HubPanel } from "../theme";
const val = (row: JsonObject, key: string) => String(row[key] ?? "—");
export function CollaborationPage({ collaboration }: { collaboration: CollaborationProjection }) { return <Stack spacing={2}><HubPanel title="稳定工作区基线"><Typography component="pre" className="json-evidence">{JSON.stringify(collaboration.baseline ?? { status: "uninitialized" }, null, 2)}</Typography></HubPanel><HubPanel title="文件/模块写入租约"><LeaseTable leases={collaboration.leases} /></HubPanel><HubPanel title="延迟 Patch 与冲突"><BoundedTable rows={collaboration.patches} rowKey={(row) => val(row, "patch_id")} columns={[
  { key: "id", label: "Patch", render: (row) => val(row, "patch_id") }, { key: "owner", label: "Session", render: (row) => val(row, "session_id") }, { key: "status", label: "状态/冲突", render: (row) => `${val(row, "status")} · ${val(row, "error_text")}` }, { key: "targets", label: "目标", render: (row) => JSON.stringify(row.targets ?? []) }, { key: "time", label: "创建时间", render: (row) => val(row, "created_at") },
]} /></HubPanel></Stack>; }
