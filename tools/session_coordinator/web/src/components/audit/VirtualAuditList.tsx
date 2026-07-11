import { FixedSizeList } from "./fixedList";
import type { AuditEvent } from "../../api/contracts";

export function VirtualAuditList({ events }: { events: AuditEvent[] }) {
  return <FixedSizeList items={events} rowKey={(event) => String(event.eventId)} render={(event) => <><time>{event.createdAt}</time>　<strong>{event.type}</strong>　{event.sessionId ?? "系统"}</>} label="审计事件" />;
}
