import type { AuditEvent } from "../api/contracts";
import { CircularProgress, Stack, Typography } from "@mui/material";
import { useEffect, useState } from "react";
import { controlClient } from "../api/client";
import { LogViewer } from "../components/logs/LogViewer";
import { HubPanel } from "../theme";
export function LogsPage({ fallback, refreshKey }: { fallback: AuditEvent[]; refreshKey: number }) {
  const [events, setEvents] = useState<AuditEvent[] | null>(fallback.length > 0 ? fallback : null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    const controller = new AbortController();
    controlClient.logs(undefined, controller.signal).then((range) => { setEvents(range.events); setError(null); }).catch((reason) => {
      if (!controller.signal.aborted) setError(String(reason));
    });
    return () => controller.abort();
  }, [refreshKey]);
  return <HubPanel title="协调器文本日志"><Stack spacing={1}>
    {error && !events && <Typography role="alert">{error}</Typography>}
    {!events && !error && <CircularProgress aria-label="加载协调器日志" size={24} />}
    {events && <LogViewer events={events} />}
  </Stack></HubPanel>;
}
