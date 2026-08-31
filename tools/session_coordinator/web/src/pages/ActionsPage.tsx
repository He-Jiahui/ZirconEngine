import { Alert, Button, Chip, FormControl, InputLabel, MenuItem, Select, Stack, TextField, Typography } from "@mui/material";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { actionClient, ControlActionError } from "../actions/actionClient";
import { actionMutationBlockReason, buildActionParameters, isLifecycleAction } from "../actions/actionParameters";
import { canUseAction } from "../actions/catalog";
import { forgetPendingAction, loadPendingActions, pollActionUntilTerminal, rememberPendingAction } from "../actions/actionTracking";
import type { ActionCatalog, ActionRecord, ControlAuthSession, ServiceProjection, SessionProjection, WorkflowDetail, WorkflowSummary } from "../api/contracts";
import { controlClient } from "../api/client";
import { ActionActivityList } from "../components/actions/ActionActivityList";
import { ActionDialog } from "../components/actions/ActionDialog";
import { HubPanel } from "../theme";

export function ActionsPage({ service, sessions, workflows, auth, onAuthChange }: { service: ServiceProjection; sessions: SessionProjection[]; workflows: WorkflowSummary[]; auth: ControlAuthSession | null; onAuthChange: (session: ControlAuthSession) => void }) {
  const [catalog, setCatalog] = useState<ActionCatalog | null>(null); const [grant, setGrant] = useState(""); const [sessionId, setSessionId] = useState(() => new URLSearchParams(window.location.search).get("session") ?? "");
  const [template, setTemplate] = useState("coordinator-actions"); const [jobId, setJobId] = useState(""); const [lifecycleTimeout, setLifecycleTimeout] = useState("120"); const [preview, setPreview] = useState<ActionRecord | null>(null); const [history, setHistory] = useState<ActionRecord[]>([]); const [trackingErrors, setTrackingErrors] = useState<Record<string, string>>({}); const [busy, setBusy] = useState(false); const [error, setError] = useState<string | null>(null);
  const [reviewSummary, setReviewSummary] = useState(""); const [criticalCount, setCriticalCount] = useState("0"); const [importantCount, setImportantCount] = useState("0");
  const [comparison, setComparison] = useState<{ previous: ActionRecord; fresh: ActionRecord } | null>(null);
  const candidateRuns = workflows.filter((workflow) => workflow.sessionId === sessionId);
  const [runId, setRunId] = useState(""); const [workflow, setWorkflow] = useState<WorkflowDetail | null>(null); const [milestoneId, setMilestoneId] = useState("");
  const [workflowRefresh, setWorkflowRefresh] = useState(0);
  const trackers = useRef(new Map<string, AbortController>());
  const selectedRunUpdatedAt = candidateRuns.find((run) => run.runId === runId)?.updatedAt;
  const selectedMilestone = workflow?.nodes.find((node) => node.kind === "milestone" && node.nodeKey === milestoneId);
  useEffect(() => { actionClient.catalog().then(setCatalog).catch((issue) => setError(String(issue))); }, []);
  useEffect(() => { if (!sessionId) setSessionId(auth?.boundSessionId ?? sessions[0]?.sessionId ?? ""); }, [auth?.boundSessionId, sessionId, sessions]);
  useEffect(() => { const next = candidateRuns.some((run) => run.runId === runId) ? runId : (candidateRuns[0]?.runId ?? ""); if (next !== runId) setRunId(next); }, [candidateRuns, runId]);
  useEffect(() => { if (!runId) { setWorkflow(null); return; } const controller = new AbortController(); controlClient.workflow(runId, controller.signal).then((value) => { setWorkflow(value); const ids = value.nodes.filter((node) => node.kind === "milestone").map((node) => node.nodeKey); setMilestoneId((current) => ids.includes(current) ? current : (ids[0] ?? "")); }).catch((issue) => { if (!controller.signal.aborted) setError(message(issue)); }); return () => controller.abort(); }, [runId, selectedRunUpdatedAt, workflowRefresh]);
  const role = auth?.role ?? "observer"; const specs = useMemo(() => catalog?.actions ?? [], [catalog]);
  const reviewMode = reviewSummary.trim().length > 0;
  const reviewScopeAllowed = Boolean(auth?.boundSessionId) && auth?.boundSessionId !== sessionId;
  const parsedLifecycleTimeout = Number(lifecycleTimeout);
  const lifecycleTimeoutValid = Number.isInteger(parsedLifecycleTimeout) && parsedLifecycleTimeout >= 1 && parsedLifecycleTimeout <= 300;
  const mutationBlockReason = actionMutationBlockReason(service);
  const upsertHistory = useCallback((action: ActionRecord) => setHistory((items) => [action, ...items.filter((item) => item.actionId !== action.actionId)]), []);
  const trackAction = useCallback((initial: ActionRecord) => {
    if (initial.status !== "executing" || trackers.current.has(initial.actionId)) return;
    rememberPendingAction(initial);
    const controller = new AbortController();
    trackers.current.set(initial.actionId, controller);
    setTrackingErrors((items) => { const next = { ...items }; delete next[initial.actionId]; return next; });
    void pollActionUntilTerminal(initial, {
      lookup: (actionId) => actionClient.status(actionId, controller.signal),
      signal: controller.signal,
      onUpdate: upsertHistory,
    }).then((terminal) => {
      upsertHistory(terminal);
      forgetPendingAction(terminal.actionId);
    }).catch((issue) => {
      if (!controller.signal.aborted) setTrackingErrors((items) => ({ ...items, [initial.actionId]: message(issue) }));
    }).finally(() => trackers.current.delete(initial.actionId));
  }, [upsertHistory]);
  useEffect(() => {
    const controller = new AbortController();
    for (const reference of loadPendingActions()) {
      actionClient.status(reference.actionId, controller.signal).then(({ action }) => {
        upsertHistory(action);
        if (action.status === "executing") trackAction(action); else forgetPendingAction(action.actionId);
      }).catch((issue) => {
        if (!controller.signal.aborted) setTrackingErrors((items) => ({ ...items, [reference.actionId]: message(issue) }));
      });
    }
    return () => controller.abort();
  }, [trackAction, upsertHistory]);
  useEffect(() => {
    if (!auth || auth.actor === "loopback-viewer") return;
    const controller = new AbortController();
    actionClient.activity(50, controller.signal).then((activity) => {
      setHistory(activity.actions);
      for (const action of activity.actions) if (action.status === "executing") trackAction(action);
    }).catch((issue) => {
      if (!controller.signal.aborted) setError(`操作活动加载失败：${message(issue)}`);
    });
    return () => controller.abort();
  }, [auth?.actor, auth?.role, auth?.boundSessionId, trackAction]);
  useEffect(() => () => { for (const controller of trackers.current.values()) controller.abort(); trackers.current.clear(); }, []);
  const elevate = async () => { setBusy(true); setError(null); try { const next = await actionClient.elevate(grant.trim()); onAuthChange(next); setGrant(""); } catch (issue) { setError(message(issue)); } finally { setBusy(false); } };
  const startPreview = async (kind: string) => { if (mutationBlockReason) { setError(mutationBlockReason); return; } setBusy(true); setError(null); setComparison(null); try { const parameters = buildActionParameters(kind, { sessionId, template, jobId, runId, milestoneId, lifecycleTimeoutSeconds: parsedLifecycleTimeout, review: kind === "topology.refresh" && reviewMode ? { reviewerSessionId: auth?.boundSessionId ?? "", executorSessionId: sessionId, criticalCount: Number(criticalCount), importantCount: Number(importantCount), summary: reviewSummary.trim() } : null }); const response = await actionClient.preview(kind, parameters); setPreview(response.action); upsertHistory(response.action); } catch (issue) { setError(message(issue)); } finally { setBusy(false); } };
  const confirm = async (phrase: string, reason: string) => { if (!preview) return; const previous = preview; setBusy(true); setError(null); try { const response = await actionClient.confirm(previous.actionId, phrase, reason); upsertHistory(response.action); setPreview(null); setComparison(null); setWorkflowRefresh((value) => value + 1); trackAction(response.action); } catch (issue) { if (issue instanceof ControlActionError && issue.code === "action_state_changed") { try { const refreshed = await actionClient.preview(previous.kind, previous.parameters); setPreview(refreshed.action); setComparison({ previous, fresh: refreshed.action }); upsertHistory(refreshed.action); setError("状态已变化：已生成并显示新旧预览差异，操作尚未执行。"); } catch (refreshIssue) { setError(`状态已变化，但新预览失败：${message(refreshIssue)}`); } } else { setError(message(issue)); } } finally { setBusy(false); } };
  const cancel = async (reason: string) => { if (!preview) return; setBusy(true); try { const response = await actionClient.cancel(preview.actionId, reason); upsertHistory(response.action); setPreview(null); setComparison(null); } catch (issue) { setError(message(issue)); } finally { setBusy(false); } };
  const cancelExecuting = async (action: ActionRecord) => { setBusy(true); setError(null); try { const response = await actionClient.cancel(action.actionId, "网页操作员取消尚未进入停止阶段的生命周期"); trackers.current.get(action.actionId)?.abort(); trackers.current.delete(action.actionId); forgetPendingAction(action.actionId); upsertHistory(response.action); setWorkflowRefresh((value) => value + 1); } catch (issue) { setError(message(issue)); } finally { setBusy(false); } };
  return <Stack spacing={2}>
    <HubPanel title="短期权限提升"><Stack spacing={1}><Typography variant="body2">在本机运行 `zircon-session control elevate --role operator --session-id &lt;id&gt;`，把一次性授权粘贴到这里。浏览器不能自行签发授权。</Typography><Stack direction={{ xs: "column", md: "row" }} spacing={1}><TextField fullWidth label="一次性提升授权" value={grant} onChange={(event) => setGrant(event.target.value)} type="password" autoComplete="off" /><Button variant="contained" onClick={elevate} disabled={busy || !grant.trim()}>提升权限</Button></Stack><Typography variant="body2">当前角色：<Chip size="small" label={role} /> {auth?.boundSessionId ? `绑定 ${auth.boundSessionId}` : "未绑定 Session"}</Typography></Stack></HubPanel>
    {error && <Alert severity="warning">{error}</Alert>}
    {mutationBlockReason && <Alert severity="error">{mutationBlockReason} 当前仍可查看活动历史和服务证据。</Alert>}
    <fieldset disabled={Boolean(mutationBlockReason)} style={{ border: 0, margin: 0, padding: 0, minWidth: 0 }}>
    <HubPanel title="Action Catalog"><Stack spacing={2}><FormControl fullWidth><InputLabel id="action-session-label">目标 Session</InputLabel><Select labelId="action-session-label" label="目标 Session" value={sessionId} onChange={(event) => setSessionId(event.target.value)}>{sessions.map((session) => <MenuItem key={session.sessionId} value={session.sessionId}>{session.displayName ?? session.sessionId}</MenuItem>)}</Select></FormControl><Stack direction={{ xs: "column", md: "row" }} spacing={1}><FormControl sx={{ minWidth: 260 }}><InputLabel id="workflow-run-label">工作流</InputLabel><Select labelId="workflow-run-label" label="工作流" value={runId} onChange={(event) => setRunId(event.target.value)}>{candidateRuns.map((run) => <MenuItem key={run.runId} value={run.runId}>{run.workflowKey}</MenuItem>)}</Select></FormControl><FormControl sx={{ minWidth: 180 }}><InputLabel id="milestone-label">里程碑</InputLabel><Select labelId="milestone-label" label="里程碑" value={milestoneId} onChange={(event) => setMilestoneId(event.target.value)}>{workflow?.nodes.filter((node) => node.kind === "milestone").map((node) => <MenuItem key={node.nodeId} value={node.nodeKey}>{node.nodeKey} · {node.state}</MenuItem>)}</Select></FormControl><FormControl sx={{ minWidth: 220 }}><InputLabel id="validation-template-label">验证模板</InputLabel><Select labelId="validation-template-label" label="验证模板" value={template} onChange={(event) => setTemplate(event.target.value)}><MenuItem value="coordinator-actions">协调器 Action 套件</MenuItem><MenuItem value="web-check">网页完整检查</MenuItem></Select></FormControl><TextField label="取消验证 Job ID" value={jobId} onChange={(event) => setJobId(event.target.value)} /><TextField label="服务操作等待秒数" type="number" value={lifecycleTimeout} onChange={(event) => setLifecycleTimeout(event.target.value)} error={!lifecycleTimeoutValid} helperText={lifecycleTimeoutValid ? "1–300 秒" : "请输入 1–300 的整数"} slotProps={{ htmlInput: { min: 1, max: 300, step: 1 } }} /></Stack>{selectedMilestone?.commitEligibility && <Alert severity={selectedMilestone.commitEligibility.eligible ? "success" : "info"}>提交门禁：{selectedMilestone.commitEligibility.code}{selectedMilestone.commitEligibility.missing.length ? `；缺少 ${selectedMilestone.commitEligibility.missing.join("、")}` : ""}{selectedMilestone.commitEligibility.rejected.length ? `；未通过 ${selectedMilestone.commitEligibility.rejected.join("、")}` : ""}</Alert>}<Stack direction="row" useFlexGap sx={{ flexWrap: "wrap", gap: 1 }}>{specs.map((spec) => { const sessionScopeAllowed = spec.kind === "topology.refresh" && reviewMode ? reviewScopeAllowed : auth?.boundSessionId === sessionId; const enabled = canUseAction(role, spec.requiredRole, spec.enabled) && (!spec.sessionBound || sessionScopeAllowed); const workflowMissing = ((spec.kind === "milestone.commit" || spec.kind === "validation.start") && (!runId || !milestoneId)) || (spec.kind === "topology.refresh" && reviewMode && (!runId || !milestoneId)) || (spec.kind === "session.complete" && !runId); const gateBlocked = spec.kind === "milestone.commit" && !selectedMilestone?.commitEligibility?.eligible; return <Button key={spec.kind} variant="outlined" color={spec.risk === "red" ? "error" : "primary"} disabled={busy || !enabled || workflowMissing || gateBlocked || (spec.kind === "validation.cancel" && !jobId) || (isLifecycleAction(spec.kind) && !lifecycleTimeoutValid)} onClick={() => startPreview(spec.kind)}>{spec.title} · {spec.risk}{!spec.enabled ? "（待后续里程碑）" : ""}</Button>; })}</Stack></Stack></HubPanel>
    </fieldset>
    <HubPanel title="独立评审导入"><Stack spacing={1}><Typography variant="body2">评审权限必须绑定到评审者 Session，且该 Session 必须不同于上方目标执行 Session；点击“刷新计划拓扑/导入独立评审”后，服务会把两种身份及评审证据绑定到当前计划、HEAD、基线和提交清单指纹。</Typography><TextField label="评审摘要（留空则只刷新拓扑）" value={reviewSummary} onChange={(event) => setReviewSummary(event.target.value)} multiline minRows={2} /><Stack direction={{ xs: "column", sm: "row" }} spacing={1}><TextField label="Critical 数量" type="number" value={criticalCount} onChange={(event) => setCriticalCount(event.target.value)} slotProps={{ htmlInput: { min: 0 } }} /><TextField label="Important 数量" type="number" value={importantCount} onChange={(event) => setImportantCount(event.target.value)} slotProps={{ htmlInput: { min: 0 } }} /></Stack></Stack></HubPanel>
    <HubPanel title="本页操作历史"><ActionActivityList actions={history} trackingErrors={trackingErrors} onCancelExecuting={cancelExecuting} cancelDisabled={busy || Boolean(mutationBlockReason)} /></HubPanel>
    <ActionDialog action={preview} comparison={comparison} busy={busy} error={error} onConfirm={confirm} onCancel={cancel} onClose={() => { setPreview(null); setComparison(null); }} />
  </Stack>;
}

function message(issue: unknown): string { return issue instanceof Error ? issue.message : "受控操作失败"; }
