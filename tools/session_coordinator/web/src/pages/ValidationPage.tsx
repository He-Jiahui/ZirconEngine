import { Stack, Typography } from "@mui/material";
import type { JsonObject, ValidationProjection } from "../api/contracts";
import { BoundedTable } from "../components/BoundedTable";
import { ValidationLaneTable } from "../components/validation/ValidationLaneTable";
import { ArtifactLifecycleSummary } from "../components/validation/ArtifactLifecycleSummary";
import { HubPanel } from "../theme";
const value = (row: JsonObject, key: string) => String(row[key] ?? "—");
export function ValidationPage({ validation }: { validation: ValidationProjection }) { return <Stack spacing={2}><HubPanel title="Cargo 验证租约"><ArtifactLifecycleSummary jobs={validation.cargoJobs} /><ValidationLaneTable jobs={validation.cargoJobs} /></HubPanel><HubPanel title="临时验证副本"><Typography>构建后由服务清理，不形成 worktree。</Typography><BoundedTable rows={validation.validationCopies} rowKey={(row) => value(row, "job_id")} columns={[
  { key: "id", label: "副本", render: (row) => value(row, "job_id") }, { key: "session", label: "Session", render: (row) => value(row, "session_id") }, { key: "path", label: "副本根目录", render: (row) => value(row, "job_root") }, { key: "status", label: "状态", render: (row) => value(row, "status") }, { key: "time", label: "创建时间", render: (row) => value(row, "created_at") },
]} /></HubPanel></Stack>; }
