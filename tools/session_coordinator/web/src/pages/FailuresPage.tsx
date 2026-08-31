import { CircularProgress, Stack, Typography } from "@mui/material";
import { useEffect, useState } from "react";
import type { FailureHistoryProjection, FailureProjection } from "../api/contracts";
import { controlClient } from "../api/client";
import { FailureGraph } from "../components/failure/FailureGraph";
import { HubPanel } from "../theme";

export function FailuresPage({ fallback, refreshKey }: { fallback: FailureProjection; refreshKey: number }) {
  const [failures, setFailures] = useState<FailureProjection | null>(fallback.nodes.length > 0 || fallback.diagnostics.length > 0 ? fallback : null);
  const [history, setHistory] = useState<FailureHistoryProjection | null>(null);
  const [historyLimit, setHistoryLimit] = useState(100);
  const [historyLoading, setHistoryLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [historyError, setHistoryError] = useState<string | null>(null);
  useEffect(() => {
    const controller = new AbortController();
    controlClient.failures(controller.signal).then((next) => { setFailures(next); setError(null); }).catch((reason) => {
      if (!controller.signal.aborted) setError(String(reason));
    });
    return () => controller.abort();
  }, [refreshKey]);
  useEffect(() => {
    const controller = new AbortController();
    setHistoryLoading(true);
    controlClient.failureHistory(historyLimit, controller.signal).then((next) => {
      setHistory(next);
      setHistoryError(null);
    }).catch((reason) => {
      if (!controller.signal.aborted) setHistoryError(String(reason));
    }).finally(() => {
      if (!controller.signal.aborted) setHistoryLoading(false);
    });
    return () => controller.abort();
  }, [refreshKey, historyLimit]);
  return <HubPanel title="Failure 图级治理"><Stack spacing={1}>
    {error && !failures && <Typography role="alert">{error}</Typography>}
    {historyError && !history && <Typography role="alert">Failure 历史加载失败：{historyError}</Typography>}
    {!failures && !error && <CircularProgress aria-label="加载 Failure 图" size={24} />}
    {failures && <FailureGraph nodes={failures.nodes} diagnostics={failures.diagnostics} history={history} historyLoading={historyLoading} onLoadMoreHistory={() => setHistoryLimit((limit) => Math.min(200, limit + 50))} />}
  </Stack></HubPanel>;
}
