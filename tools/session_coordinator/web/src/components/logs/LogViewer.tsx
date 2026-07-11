import { Checkbox, FormControlLabel, MenuItem, Stack, TextField, Typography } from "@mui/material";
import { useEffect, useMemo, useState } from "react";
import type { AuditEvent } from "../../api/contracts";
import { controlClient } from "../../api/client";
import { FixedSizeList } from "../audit/fixedList";
import { HubButton } from "../../theme";

export function LogViewer({ events }: { events: AuditEvent[] }) {
  const [query, setQuery] = useState("");
  const [paused, setPaused] = useState(false);
  const [level, setLevel] = useState("");
  const [session, setSession] = useState("");
  const [since, setSince] = useState("");
  const [until, setUntil] = useState("");
  const [rangeEvents, setRangeEvents] = useState<AuditEvent[]>([]);
  const [frozenEvents, setFrozenEvents] = useState<AuditEvent[]>([]);
  const [nextBefore, setNextBefore] = useState<number | null>(null);
  const [truncated, setTruncated] = useState(false);
  const [loading, setLoading] = useState(false);
  const merged = useMemo(() => mergeEvents(rangeEvents, events), [rangeEvents, events]);
  const source = paused ? frozenEvents : merged;
  const sessions = useMemo(() => [...new Set(merged.map((event) => event.sessionId).filter(Boolean))] as string[], [merged]);
  useEffect(() => { controlClient.logs().then((range) => { setRangeEvents(range.events); setNextBefore(range.nextBefore); setTruncated(range.truncated); }).catch(() => undefined); }, []);
  const filtered = useMemo(() => source.filter((event) => {
    const text = `${event.createdAt} ${event.type} ${event.sessionId ?? ""} ${JSON.stringify(event.payload)}`;
    const time = Date.parse(event.createdAt);
    return text.toLowerCase().includes(query.toLowerCase())
      && (!level || event.type.toLowerCase().includes(level.toLowerCase()))
      && (!session || event.sessionId === session)
      && (!since || time >= Date.parse(since))
      && (!until || time <= Date.parse(until));
  }), [source, query, level, session, since, until]);
  const togglePaused = (checked: boolean) => { if (checked) setFrozenEvents(merged); setPaused(checked); };
  const loadOlder = async () => { if (!nextBefore || loading) return; setLoading(true); try { const range = await controlClient.logs(nextBefore); setRangeEvents((current) => mergeEvents(range.events, current)); setNextBefore(range.nextBefore); setTruncated(range.truncated); } finally { setLoading(false); } };
  return <Stack spacing={2}>
    <Stack direction={{ xs: "column", sm: "row" }} spacing={2}>
      <TextField size="small" label="搜索日志" value={query} onChange={(event) => setQuery(event.target.value)} />
      <TextField size="small" label="级别/类型过滤" value={level} onChange={(event) => setLevel(event.target.value)} />
      <TextField select size="small" label="Session 流" value={session} onChange={(event) => setSession(event.target.value)} sx={{ minWidth: 150 }}><MenuItem value="">全部</MenuItem>{sessions.map((item) => <MenuItem key={item} value={item}>{item}</MenuItem>)}</TextField>
      <FormControlLabel control={<Checkbox checked={paused} onChange={(event) => togglePaused(event.target.checked)} />} label={paused ? "已暂停跟随" : "自动跟随"} />
    </Stack>
    <Stack direction={{ xs: "column", sm: "row" }} spacing={2}><TextField type="datetime-local" size="small" label="开始时间" slotProps={{ inputLabel: { shrink: true } }} value={since} onChange={(event) => setSince(event.target.value)} /><TextField type="datetime-local" size="small" label="结束时间" slotProps={{ inputLabel: { shrink: true } }} value={until} onChange={(event) => setUntil(event.target.value)} />{truncated && <HubButton onClick={loadOlder} disabled={loading}>{loading ? "加载中" : "加载更早记录"}</HubButton>}</Stack>
    <Typography variant="caption" role="status">纯文本虚拟列表，仅挂载可见行；当前 {filtered.length} 行。{truncated ? "更早记录已截断，可按范围继续加载。" : "已到达可用记录起点。"}{paused ? "新事件不会改变冻结视图。" : "当前跟随最新事件。"}</Typography>
    <FixedSizeList items={filtered} follow={!paused} rowKey={(event) => String(event.eventId)} label="协调器日志" render={(event) => <span title={formatEvent(event)}><time>{event.createdAt}</time> [{event.type}] {event.sessionId ?? "系统"} {JSON.stringify(event.payload)}</span>} />
  </Stack>;
}

function formatEvent(event: AuditEvent): string {
  return `${event.createdAt} [${event.type}] ${event.sessionId ?? "系统"} ${JSON.stringify(event.payload)}`;
}

function mergeEvents(...groups: AuditEvent[][]): AuditEvent[] {
  const unique = new Map<number, AuditEvent>();
  for (const group of groups) for (const event of group) unique.set(event.eventId, event);
  return [...unique.values()].sort((left, right) => left.eventId - right.eventId);
}
