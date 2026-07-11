import { MenuItem, Select, Stack, Typography } from "@mui/material";
import { useEffect, useState } from "react";
import type { CollaborationProjection, FailureProjection, WorkflowDetail, WorkflowSummary } from "../api/contracts";
import { controlClient } from "../api/client";
import { WorkflowPipeline } from "../components/workflow/WorkflowPipeline";
import { HubPanel } from "../theme";

export function WorkflowsPage({ workflows, collaboration, failures }: { workflows: WorkflowSummary[]; collaboration: CollaborationProjection; failures: FailureProjection }) {
  const [runId, setRunId] = useState(workflows[0]?.runId ?? "");
  const [detail, setDetail] = useState<WorkflowDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  const selectedUpdatedAt = workflows.find((workflow) => workflow.runId === runId)?.updatedAt;
  useEffect(() => {
    if (!workflows.length) { setRunId(""); setDetail(null); return; }
    if (!workflows.some((workflow) => workflow.runId === runId)) setRunId(workflows[0].runId);
  }, [runId, workflows]);
  useEffect(() => { if (!runId) return; const controller = new AbortController(); controlClient.workflow(runId, controller.signal).then((next) => { setDetail(next); setError(null); }).catch((reason) => { if (!controller.signal.aborted) setError(String(reason)); }); return () => controller.abort(); }, [runId, selectedUpdatedAt]);
  return <Stack spacing={2}><HubPanel title="工作流选择"><Select size="small" fullWidth value={runId} onChange={(event) => setRunId(event.target.value)} displayEmpty>{workflows.map((item) => <MenuItem value={item.runId} key={item.runId}>{item.workflowKey} · {item.sessionId} · {item.state}</MenuItem>)}</Select></HubPanel>{error && <Typography role="alert">{error}</Typography>}{detail && <HubPanel title={detail.workflowKey}><WorkflowPipeline detail={detail} leases={collaboration.leases} failures={failures.nodes} /></HubPanel>}{!workflows.length && <Typography>没有可显示的工作流。</Typography>}</Stack>;
}
