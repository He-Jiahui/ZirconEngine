import type { AuditEvent } from "../api/contracts";
import { LogViewer } from "../components/logs/LogViewer";
import { HubPanel } from "../theme";
export function LogsPage({ events }: { events: AuditEvent[] }) { return <HubPanel title="协调器文本日志"><LogViewer events={events} /></HubPanel>; }
