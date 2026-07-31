import type { AuditEvent } from "../api/contracts";
import { type DisplayAuditEvent, VirtualAuditList } from "../components/audit/VirtualAuditList";
import { HubPanel } from "../theme";

function rolloverWaitKey(event: AuditEvent): string | null {
  if (event.type !== "lifecycle.rollover_waiting_for_cargo") return null;
  return JSON.stringify(event.payload);
}

export function coalesceAuditEvents(events: AuditEvent[]): DisplayAuditEvent[] {
  const compact: DisplayAuditEvent[] = [];
  let previousKey: string | null = null;
  for (const event of events) {
    const key = rolloverWaitKey(event);
    const previous = compact.at(-1);
    if (key !== null && key === previousKey && previous) {
      previous.repeatCount = (previous.repeatCount ?? 1) + 1;
      continue;
    }
    compact.push({ ...event, repeatCount: 1 });
    previousKey = key;
  }
  return compact;
}

export function AuditPage({ events }: { events: AuditEvent[] }) {
  return <HubPanel title="只读审计轨迹"><VirtualAuditList events={coalesceAuditEvents(events)} /></HubPanel>;
}
