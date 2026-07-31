import { FixedSizeList } from "./fixedList";
import type { AuditEvent } from "../../api/contracts";

export type DisplayAuditEvent = AuditEvent & { repeatCount?: number };

export function VirtualAuditList({ events }: { events: DisplayAuditEvent[] }) {
  return <FixedSizeList items={events} rowKey={(event) => String(event.eventId)} render={(event) => <><time>{event.createdAt}</time>　<strong>{event.type}</strong>　{event.sessionId ?? "系统"}{(event.repeatCount ?? 1) > 1 ? <>　<span>相同等待已合并 {event.repeatCount} 次</span></> : null}</>} label="审计事件" />;
}
