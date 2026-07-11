import type { SessionProjection } from "../api/contracts";
import { Button } from "@mui/material";
import { BoundedTable } from "../components/BoundedTable";
import { StatusText } from "../components/StatusText";
import { HubPanel } from "../theme";

export function SessionsPage({ sessions }: { sessions: SessionProjection[] }) { return <HubPanel title="Session 状态"><BoundedTable rows={sessions} rowKey={(row) => row.sessionId} columns={[
  { key: "name", label: "会话", render: (row) => row.displayName ?? row.sessionId }, { key: "status", label: "枚举状态", render: (row) => <StatusText value={row.status} /> }, { key: "reason", label: "状态摘要", render: (row) => row.statusReason ?? "—" }, { key: "plan", label: "计划", render: (row) => row.planPath ?? "—" }, { key: "heartbeat", label: "最后心跳", render: (row) => row.lastHeartbeatAt }, { key: "actions", label: "操作", render: (row) => <Button component="a" href={`/ui/actions?session=${encodeURIComponent(row.sessionId)}`} size="small">受控操作</Button> },
]} /></HubPanel>; }
