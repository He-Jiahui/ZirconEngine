import type { AuditEvent } from "../api/contracts";
import { CircularProgress, Stack, Typography } from "@mui/material";
import { useEffect, useState } from "react";
import { controlClient } from "../api/client";
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

export function AuditPage({ fallback, refreshKey }: { fallback: AuditEvent[]; refreshKey: number }) {
  const [events, setEvents] = useState<AuditEvent[] | null>(fallback.length > 0 ? fallback : null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    const controller = new AbortController();
    controlClient.logs(undefined, controller.signal).then((range) => { setEvents(range.events); setError(null); }).catch((reason) => {
      if (!controller.signal.aborted) setError(String(reason));
    });
    return () => controller.abort();
  }, [refreshKey]);
  return <HubPanel title="只读审计轨迹"><Stack spacing={1}>
    {error && !events && <Typography role="alert">{error}</Typography>}
    {!events && !error && <CircularProgress aria-label="加载审计轨迹" size={24} />}
    {events && <VirtualAuditList events={coalesceAuditEvents(events)} />}
  </Stack></HubPanel>;
}
