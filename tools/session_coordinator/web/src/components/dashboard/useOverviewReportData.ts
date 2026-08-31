import { useEffect, useState } from "react";
import type { AuditEvent, FailureHistoryProjection, ValidationHistoryProjection } from "../../api/contracts";
import { controlClient } from "../../api/client";

export const OVERVIEW_REPORT_REFRESH_MS = 60_000;

export function useOverviewReportData() {
  const [validationHistory, setValidationHistory] = useState<ValidationHistoryProjection | null>(null);
  const [failureHistory, setFailureHistory] = useState<FailureHistoryProjection | null>(null);
  const [auditEvents, setAuditEvents] = useState<AuditEvent[]>([]);
  const [validationHistoryError, setValidationHistoryError] = useState<string | null>(null);

  useEffect(() => {
    let controller: AbortController | null = null;
    const refresh = () => {
      controller?.abort();
      controller = new AbortController();
      const signal = controller.signal;
      controlClient.validationHistory(200, signal)
        .then((history) => { setValidationHistory(history); setValidationHistoryError(null); })
        .catch((error) => {
          if (!signal.aborted) setValidationHistoryError(error instanceof Error ? error.message : "验证历史加载失败");
        });
      controlClient.failureHistory(200, signal).then(setFailureHistory).catch(() => undefined);
      controlClient.logs(undefined, signal).then((logs) => setAuditEvents(logs.events)).catch(() => undefined);
    };

    refresh();
    const timer = window.setInterval(refresh, OVERVIEW_REPORT_REFRESH_MS);
    return () => {
      window.clearInterval(timer);
      controller?.abort();
    };
  }, []);

  return { validationHistory, failureHistory, auditEvents, validationHistoryError };
}
