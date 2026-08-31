import { Alert, Box, Button, Chip, List, ListItemButton, Stack, Typography } from "@mui/material";
import { useEffect, useState } from "react";
import type { FailureHistoryChain, FailureHistoryProjection, FailureNode, FailureProjection } from "../../api/contracts";
import { StatusText } from "../StatusText";
import { failureClass, failureReviewItems, type FailureReviewItem } from "./failureModel";

interface FailureGraphProps extends FailureProjection {
  history?: FailureHistoryProjection | null;
  historyLoading?: boolean;
  onLoadMoreHistory?: () => void;
}

export function FailureGraph({ nodes, diagnostics, history, historyLoading = false, onLoadMoreHistory }: FailureGraphProps) {
  const items = failureReviewWindow(nodes, history);
  const [selectedId, setSelectedId] = useState<number | null>(items[0]?.node_id ?? null);
  useEffect(() => {
    if (!items.some((item) => item.node_id === selectedId)) setSelectedId(items[0]?.node_id ?? null);
  }, [items, selectedId]);
  const selectedIndex = Math.max(0, items.findIndex((item) => item.node_id === selectedId));
  const selected = items[selectedIndex] ?? null;
  const openCount = items.filter((item) => item.reviewState === "needs_review").length;
  const verifiedCount = items.length - openCount;
  const selectedHistory = selected
    ? history?.chains.find((chain) => chain.lifecycleKey === selected.lifecycle_key) ?? null
    : null;
  return <Stack spacing={2} aria-label="Failure 审核队列">
    <Stack direction={{ xs: "column", sm: "row" }} spacing={1} sx={{ alignItems: { sm: "center" } }}>
      <Typography variant="body2" color="text.secondary" sx={{ flex: 1 }}>按优先级逐项审核。只有协调器写入的 fixed 回传和门禁证据会改变验证结论。</Typography>
      <Chip size="small" color={(history?.statusCounts.open ?? openCount) ? "warning" : "success"} label={`待审核 ${history?.statusCounts.open ?? openCount}`} />
      <Chip size="small" variant="outlined" label={`已修复 ${history?.statusCounts.fixed ?? verifiedCount}`} />
    </Stack>
    {!items.length && <Typography>当前没有 Failure 节点。</Typography>}
    {selected && <Box className="dashboard-flow-grid">
      <List component="nav" aria-label="Failure 审核项" sx={{ border: 1, borderColor: "divider", borderRadius: 1, p: 0, maxHeight: { xs: "42vh", lg: "64vh" }, overflow: "auto" }}>
        {items.map((item) => <FailureQueueItem key={item.node_id} item={item} selected={item.node_id === selected.node_id} onSelect={setSelectedId} />)}
      </List>
      <Stack spacing={1.5} sx={{ minWidth: 0 }}><FailureChainMap item={selected} history={selectedHistory} /><FailureReviewDetail item={selected} index={selectedIndex} total={items.length} diagnostics={diagnostics} history={selectedHistory} historyLoading={historyLoading} onStep={(offset) => setSelectedId(items[Math.min(items.length - 1, Math.max(0, selectedIndex + offset))]?.node_id ?? selected.node_id)} /></Stack>
    </Box>}
    {history?.truncated && onLoadMoreHistory && <Button variant="outlined" size="small" disabled={historyLoading} onClick={onLoadMoreHistory} sx={{ alignSelf: "flex-start" }}>{historyLoading ? "正在加载" : "加载更多 Failure 历史"}</Button>}
    <Stack spacing={1}>
      <Typography variant="subtitle2">图诊断</Typography>
      {diagnostics.slice(0, 100).map((diagnostic) => <Alert key={diagnostic.diagnosticId} severity="warning"><Typography variant="body2">{diagnostic.code}：{diagnostic.message}</Typography><Typography variant="caption">{diagnostic.paths.join("、") || "无关联路径"}</Typography></Alert>)}
      {!diagnostics.length && <Typography variant="body2" color="text.secondary">当前没有图诊断。</Typography>}
    </Stack>
  </Stack>;
}

function FailureChainMap({ item, history }: { item: FailureReviewItem; history: FailureHistoryChain | null }) {
  const verified = item.reviewState === "verified";
  const events = history?.events ?? [];
  return <Box className="dashboard-band" sx={{ p: 1.5 }} aria-label="Failure 关系链"><Stack spacing={1}><Stack direction="row" spacing={1} sx={{ alignItems: "center" }}><Typography variant="subtitle2" sx={{ flex: 1 }}>Failure 关系链</Typography><Chip size="small" color={verified ? "success" : "warning"} label={verified ? "已修复" : "待验证"} /></Stack><Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", sm: "repeat(4, minmax(0, 1fr))" }, gap: 1, alignItems: "stretch" }}>
    <FailureChainNode label="来源计划" value={item.origin_plan || "未标记"} tone="primary.main" />
    <FailureChainNode label="Failure" value={item.summary_slug} tone="error.main" />
    <FailureChainNode label="修复计划" value={item.fixing_plan || "未分配"} tone="warning.main" />
    <FailureChainNode label="验证结论" value={verified ? "fixed 回传" : "等待 fixed"} tone={verified ? "success.main" : "text.secondary"} />
  </Box><Typography variant="caption" color="text.secondary">生命周期 {events.length ? `${events.length} 个事件：${events.map((event) => event.kind === "fixed" ? "修复" : "增加").join(" → ")}` : "暂无历史事件"}；选择左侧 Failure 查看证据和逐项审核。</Typography></Stack></Box>;
}

function FailureChainNode({ label, value, tone }: { label: string; value: string; tone: string }) {
  return <Stack spacing={0.35} sx={{ minWidth: 0, p: 1, borderLeft: 3, borderColor: tone, bgcolor: "action.hover" }}><Typography variant="caption" color="text.secondary">{label}</Typography><Typography variant="body2" sx={{ fontWeight: 700, overflowWrap: "anywhere" }}>{value}</Typography></Stack>;
}

export function failureReviewWindow(nodes: FailureNode[], history?: FailureHistoryProjection | null): FailureReviewItem[] {
  const reviewItems = failureReviewItems(nodes);
  if (!history) return reviewItems.slice(0, 200);
  const itemByLifecycle = new Map(reviewItems.map((item) => [item.lifecycle_key, item]));
  const historyItems = history.chains.map((chain) => itemByLifecycle.get(chain.lifecycleKey)).filter((item): item is FailureReviewItem => item !== undefined);
  return historyItems.length > 0 ? historyItems : reviewItems.slice(0, 200);
}

function FailureQueueItem({ item, selected, onSelect }: { item: FailureReviewItem; selected: boolean; onSelect: (nodeId: number) => void }) {
  const state = failureClass(item);
  return <ListItemButton selected={selected} onClick={() => onSelect(item.node_id)} sx={{ alignItems: "flex-start", gap: 1, py: 1.25 }}>
    <StatusText value={state} />
    <Stack spacing={0.25} sx={{ minWidth: 0 }}>
      <Typography variant="body2" sx={{ fontWeight: selected ? 700 : 600, overflowWrap: "anywhere" }}>{item.summary_slug}</Typography>
      <Typography variant="caption" color="text.secondary">P{item.priority} · {item.reviewState === "verified" ? "已验证归档" : "待逐项审核"}</Typography>
    </Stack>
  </ListItemButton>;
}

function FailureReviewDetail({ item, index, total, diagnostics, history, historyLoading, onStep }: { item: FailureReviewItem; index: number; total: number; diagnostics: FailureProjection["diagnostics"]; history: FailureHistoryChain | null; historyLoading: boolean; onStep: (offset: number) => void }) {
  const linkedDiagnostics = diagnostics.filter((diagnostic) => diagnostic.paths.some((path) => path === item.artifact_path));
  const verified = item.reviewState === "verified";
  return <Stack component="article" spacing={2} sx={{ minWidth: 0, border: 1, borderColor: "divider", borderRadius: 1, p: { xs: 1.5, md: 2.5 } }}>
    <Stack direction={{ xs: "column", sm: "row" }} spacing={1} sx={{ alignItems: { sm: "center" } }}>
      <Stack spacing={0.25} sx={{ flex: 1, minWidth: 0 }}>
        <Typography variant="overline" color="text.secondary">Failure 审核 #{index + 1}/{total}</Typography>
        <Typography variant="h6" sx={{ overflowWrap: "anywhere" }}>{item.summary_slug}</Typography>
      </Stack>
      <StatusText value={verified ? "已验证" : "待验证"} />
    </Stack>
    <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", sm: "repeat(2, minmax(0, 1fr))" }, gap: 1 }}>
      <ReviewFact label="来源计划" value={item.origin_plan} />
      <ReviewFact label="修复责任" value={item.fixing_plan} />
      <ReviewFact label="Failure 记录" value={item.artifact_path} />
      <ReviewFact label="验证结论" value={verified ? `已于 ${item.resolved_at ?? "协调器记录的时间"} fixed 回传` : "等待修复计划提交 fixed 回传与上游门禁"} />
    </Box>
    <Alert severity={verified ? "success" : "info"}>{verified ? "此项已由协调器的 failure/fixed 生命周期记录验证。" : "先核对修复责任、回传 artifact 和关联门禁；完成后由协调器记录验证结果。"}</Alert>
    <Stack spacing={1} aria-label="Failure 增加和修复历史">
      <Typography variant="subtitle2">增加和修复历史</Typography>
      {historyLoading && !history && <Typography variant="body2" color="text.secondary">正在加载生命周期历史...</Typography>}
      {!historyLoading && !history && <Typography variant="body2" color="text.secondary">当前历史窗口中没有此 Failure 的生命周期记录。</Typography>}
      {history?.events.map((event) => <Stack key={`${event.kind}:${event.createdAt}`} spacing={0.25} sx={{ borderLeft: 2, borderColor: event.kind === "fixed" ? "success.main" : "warning.main", pl: 1.25 }}>
        <Typography variant="body2" sx={{ fontWeight: 600 }}>{event.kind === "fixed" ? "修复记录已增加" : "Failure 记录已增加"}</Typography>
        <Typography variant="caption" color="text.secondary">{formatHistoryTime(event.createdAt)} · {event.artifactPath}</Typography>
      </Stack>)}
    </Stack>
    <Stack direction="row" spacing={1} sx={{ justifyContent: "space-between" }}>
      <Button size="small" variant="outlined" disabled={index === 0} onClick={() => onStep(-1)}>上一项</Button>
      <Button size="small" variant="contained" disabled={index === total - 1} onClick={() => onStep(1)}>下一项</Button>
    </Stack>
    {linkedDiagnostics.length > 0 && <Stack spacing={0.5}><Typography variant="subtitle2">关联图诊断</Typography>{linkedDiagnostics.map((diagnostic) => <Typography key={diagnostic.diagnosticId} variant="body2">{diagnostic.code}：{diagnostic.message}</Typography>)}</Stack>}
  </Stack>;
}

function formatHistoryTime(value: string): string {
  const parsed = Date.parse(value);
  return Number.isFinite(parsed) ? new Date(parsed).toLocaleString() : value;
}

function ReviewFact({ label, value }: { label: string; value: string }) {
  return <Stack spacing={0.25} sx={{ minWidth: 0, borderLeft: 2, borderColor: "divider", pl: 1 }}><Typography variant="caption" color="text.secondary">{label}</Typography><Typography variant="body2" sx={{ overflowWrap: "anywhere" }}>{value}</Typography></Stack>;
}
