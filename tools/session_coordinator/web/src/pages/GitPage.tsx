import { CircularProgress, Stack, Typography } from "@mui/material";
import { useEffect, useState } from "react";
import type { GitProjection, SessionProjection, WorkflowSummary } from "../api/contracts";
import { controlClient } from "../api/client";
import { MilestoneCommitEvidence } from "../components/git/MilestoneCommitEvidence";
import { HubPanel } from "../theme";

export function GitPage({ fallback, refreshKey, sessions = [], workflows = [] }: { fallback: GitProjection; refreshKey: number; sessions?: SessionProjection[]; workflows?: WorkflowSummary[] }) {
  const [git, setGit] = useState<GitProjection | null>(fallback.finalizeRequests.length > 0 ? fallback : null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    const controller = new AbortController();
    controlClient.git(controller.signal).then((next) => { setGit(next); setError(null); }).catch((reason) => {
      if (!controller.signal.aborted) setError(String(reason));
    });
    return () => controller.abort();
  }, [refreshKey]);
  return <Stack spacing={2} className="dashboard-page"><HubPanel title="里程碑最终提交证据"><Stack spacing={1}>
    {error && !git && <Typography role="alert">{error}</Typography>}
    {!git && !error && <CircularProgress aria-label="加载 Git 提交证据" size={24} />}
    {git && <MilestoneCommitEvidence requests={git.finalizeRequests} sessions={sessions} workflows={workflows} />}
  </Stack></HubPanel></Stack>;
}
