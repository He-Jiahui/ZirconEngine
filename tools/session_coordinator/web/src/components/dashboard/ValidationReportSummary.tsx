import { Box, Chip, Grid, Stack, Typography } from "@mui/material";
import { HubPanel } from "../../theme";
import type { AnalyticsValidationReport } from "./analyticsModel";

const statusDefinitions = [
  { key: "passed", label: "通过", color: "success.main" },
  { key: "failed", label: "失败", color: "error.main" },
  { key: "queued", label: "排队", color: "warning.main" },
  { key: "materializing", label: "物化", color: "info.main" },
  { key: "running", label: "运行", color: "primary.main" },
  { key: "snapshotStale", label: "快照过期", color: "text.disabled" },
] as const;

export function ValidationReportSummary({ report, loadError = null }: { report: AnalyticsValidationReport; loadError?: string | null }) {
  const health = reportHealth(report);
  const maxFailureCount = Math.max(1, ...report.failureReasons.map((reason) => reason.count));
  return <HubPanel title="验证报表摘要">
    <Stack spacing={2}>
      <Stack direction={{ xs: "column", md: "row" }} spacing={1.25} sx={{ alignItems: { md: "center" } }}>
        <Stack spacing={0.2} sx={{ flex: 1, minWidth: 0 }}>
          <Stack direction="row" spacing={0.8} sx={{ alignItems: "center" }}>
            <Box aria-hidden="true" sx={{ width: 8, height: 8, flex: "0 0 auto", bgcolor: loadError ? "error.main" : health.color, borderRadius: "50%" }} />
            <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>{loadError ? "验证历史加载失败" : health.label}</Typography>
          </Stack>
          <Typography variant="caption" color={loadError ? "error.main" : "text.secondary"}>{loadError ? `${loadError}；实时快照仍继续更新，历史报表会自动重试。` : report.loaded ? reportScope(report) : "正在读取验证历史；实时快照仍继续更新。"}</Typography>
        </Stack>
        <Stack direction="row" spacing={0.75} sx={{ flexWrap: "wrap" }}>
          <Chip size="small" variant="outlined" label={report.loaded ? `全库 ${formatInteger(report.total)} 条` : "全库计数载入中"} />
          <Chip size="small" color={report.sampleTruncated ? "warning" : "default"} variant="outlined" label={report.sampleTruncated ? `最近 ${report.sampleSize} 条明细 · 已截断` : `${report.sampleSize} 条明细`} />
          {report.eventDetailsTruncated > 0 && <Chip size="small" color="warning" variant="outlined" label={`${report.eventDetailsTruncated} 条事件明细截断`} />}
        </Stack>
      </Stack>

      <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", sm: "repeat(2, minmax(0, 1fr))", lg: "repeat(4, minmax(0, 1fr))" }, borderTop: 1, borderBottom: 1, borderColor: "divider", "& > *": { borderBottom: { xs: 1, lg: 0 }, borderColor: "divider" }, "& > *:last-child": { borderBottom: 0 } }}>
        <ReportMetric label="终态成功率" value={formatPercent(report.successRate)} detail={report.loaded ? `通过 ${formatInteger(report.statuses.passed)} · 失败 ${formatInteger(report.statuses.failed)}` : "等待终态计数"} tone={report.successRate !== null && report.successRate < 0.5 ? "error.main" : "success.main"} />
        <ReportMetric label="验证总量" value={formatInteger(report.total)} detail={report.loaded ? `终态 ${formatInteger(report.terminal)} · 未完成 ${formatInteger(report.backlog)}` : "等待全库计数"} />
        <ReportMetric label="近 24 小时" value={formatInteger(report.last24Hours.started)} detail={`通过 ${report.last24Hours.passed} · 失败 ${report.last24Hours.failed} · ${formatPercent(report.last24Hours.successRate)}`} tone={report.last24Hours.failed > 0 ? "error.main" : "primary.main"} />
        <ReportMetric label="快照过期" value={formatInteger(report.statuses.snapshotStale)} detail={report.newestUpdatedAt ? `最近更新 ${formatTimestamp(report.newestUpdatedAt)}` : "暂无更新时间"} tone={Number(report.statuses.snapshotStale) > 0 ? "warning.main" : "text.secondary"} />
      </Box>

      <Stack spacing={0.8}>
        <Stack direction="row" spacing={1} sx={{ alignItems: "baseline" }}><Typography variant="subtitle2" sx={{ flex: 1 }}>全库状态分布</Typography><Typography variant="caption" color="text.secondary">{formatInteger(report.total)} tickets</Typography></Stack>
        <Box role="img" aria-label={statusDefinitions.map((definition) => `${definition.label} ${formatInteger(report.statuses[definition.key])}`).join("，")} sx={{ display: "flex", width: "100%", height: 14, overflow: "hidden", bgcolor: "action.hover", borderRadius: 0.5 }}>
          {statusDefinitions.map((definition) => { const value = report.statuses[definition.key] ?? 0; return value > 0 && report.total ? <Box key={definition.key} sx={{ width: `${(value / report.total) * 100}%`, minWidth: 3, bgcolor: definition.color }} /> : null; })}
        </Box>
        <Stack direction="row" spacing={1.5} sx={{ flexWrap: "wrap" }}>{statusDefinitions.map((definition) => <Stack key={definition.key} direction="row" spacing={0.5} sx={{ alignItems: "center" }}><Box aria-hidden="true" sx={{ width: 8, height: 8, bgcolor: definition.color }} /><Typography variant="caption" color="text.secondary">{definition.label} {formatInteger(report.statuses[definition.key])}</Typography></Stack>)}</Stack>
      </Stack>

      <Grid container spacing={2}>
        <Grid size={{ xs: 12, lg: 5 }}>
          <Stack spacing={0.6}>
            <Typography variant="subtitle2">数据窗口</Typography>
            <DataLine label="明细数量" value={report.loaded ? `${report.sampleSize}${report.sampleTruncated ? "+" : ""}` : "载入中"} />
            <DataLine label="最早创建" value={report.oldestCreatedAt ? formatTimestamp(report.oldestCreatedAt) : "—"} />
            <DataLine label="最近更新" value={report.newestUpdatedAt ? formatTimestamp(report.newestUpdatedAt) : "—"} />
            <DataLine label="事件截断" value={`${report.eventDetailsTruncated} 条 ticket`} tone={report.eventDetailsTruncated ? "warning.main" : undefined} />
          </Stack>
        </Grid>
        <Grid size={{ xs: 12, lg: 7 }}>
          <Stack spacing={0.7}>
            <Stack direction="row" spacing={1} sx={{ alignItems: "baseline" }}><Typography variant="subtitle2" sx={{ flex: 1 }}>Failure code · 最近明细</Typography><Typography variant="caption" color="text.secondary">同一 ticket / code 计一次</Typography></Stack>
            {report.failureReasons.length === 0 ? <Typography variant="body2" color="text.secondary">{report.loaded && Number(report.statuses.failed) > 0 ? `最近 ${report.sampleSize} 条明细没有结构化 failure code。` : "当前明细窗口没有结构化失败原因。"}</Typography> : report.failureReasons.map((reason) => <Stack key={reason.code} spacing={0.25}>
              <Stack direction="row" spacing={1} sx={{ alignItems: "baseline", minWidth: 0 }}><Typography variant="body2" sx={{ flex: 1, minWidth: 0, fontFamily: "monospace", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }} title={reason.code}>{reason.code}</Typography><Typography variant="caption" color="text.secondary">{reason.phase ?? "phase 未知"}</Typography><Typography variant="body2" sx={{ width: 28, textAlign: "right", fontVariantNumeric: "tabular-nums" }}>{reason.count}</Typography></Stack>
              <Box sx={{ height: 5, bgcolor: "action.hover", overflow: "hidden" }}><Box sx={{ width: `${Math.max(4, reason.count / maxFailureCount * 100)}%`, height: "100%", bgcolor: "error.main" }} /></Box>
            </Stack>)}
            {report.unclassifiedFailures > 0 && <Typography variant="caption" color="warning.main">另有 {report.unclassifiedFailures} 个失败 ticket 未提供 errorCode。</Typography>}
          </Stack>
        </Grid>
      </Grid>
    </Stack>
  </HubPanel>;
}

function ReportMetric({ label, value, detail, tone = "text.primary" }: { label: string; value: string; detail: string; tone?: string }) {
  return <Stack spacing={0.3} sx={{ minWidth: 0, p: 1.5 }}><Typography variant="caption" color="text.secondary">{label}</Typography><Typography variant="h4" color={tone} sx={{ lineHeight: 1, fontVariantNumeric: "tabular-nums" }}>{value}</Typography><Typography variant="caption" color="text.secondary" sx={{ overflowWrap: "anywhere" }}>{detail}</Typography></Stack>;
}

function DataLine({ label, value, tone }: { label: string; value: string; tone?: string }) {
  return <Stack direction="row" spacing={1} sx={{ alignItems: "baseline" }}><Typography variant="caption" color="text.secondary" sx={{ width: 72 }}>{label}</Typography><Typography variant="body2" color={tone} sx={{ minWidth: 0, overflowWrap: "anywhere", fontVariantNumeric: "tabular-nums" }}>{value}</Typography></Stack>;
}

function reportHealth(report: AnalyticsValidationReport): { label: string; color: string } {
  if (!report.loaded) return { label: "历史报表载入中", color: "text.disabled" };
  if (report.terminal === 0) return { label: "尚无终态结果", color: "warning.main" };
  if (report.successRate !== null && report.successRate < 0.5) return { label: "失败结果占多数", color: "error.main" };
  if (report.backlog && report.backlog > 0) return { label: "验证持续处理中", color: "primary.main" };
  return { label: "验证结果已结算", color: "success.main" };
}

function reportScope(report: AnalyticsValidationReport): string {
  const scope = report.sampleTruncated ? `全库状态计数 + 最近 ${report.sampleSize} 条明细` : `${report.sampleSize} 条完整明细`;
  return `${scope}；成功率仅使用通过与失败终态。`;
}

function formatInteger(value: number | null): string { return value === null ? "—" : new Intl.NumberFormat("zh-CN").format(value); }
function formatPercent(value: number | null): string { return value === null ? "—" : `${(value * 100).toFixed(1)}%`; }
function formatTimestamp(value: string): string { const parsed = new Date(value); return Number.isNaN(parsed.getTime()) ? "—" : new Intl.DateTimeFormat("zh-CN", { month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", hour12: false }).format(parsed); }
