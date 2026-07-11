import type { AuditEvent } from "../api/contracts";
import { VirtualAuditList } from "../components/audit/VirtualAuditList";
import { HubPanel } from "../theme";
export function AuditPage({ events }: { events: AuditEvent[] }) { return <HubPanel title="只读审计轨迹"><VirtualAuditList events={events} /></HubPanel>; }
