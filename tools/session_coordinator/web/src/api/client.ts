import type { CodexSessionsProjection, ContinuationProjection, ControlSnapshot, FailureHistoryProjection, FailureProjection, GitProjection, LogRange, ValidationHistoryProjection, ValidationProjection, WorkflowDetail } from "./contracts";
import { parseCodexSessions, parseContinuationProjection, parseEnvelope, parseFailureHistory, parseFailureProjection, parseGitProjection, parseLogRange, parseSnapshot, parseValidationHistory, parseValidationProjection, parseWorkflowDetail } from "./validation";

async function get<T>(path: string, parser: (value: unknown) => T, signal?: AbortSignal): Promise<T> {
  const response = await fetch(path, { credentials: "same-origin", headers: { Accept: "application/json" }, signal });
  const contentType = response.headers.get("content-type") ?? "";
  if (!contentType.startsWith("application/json")) throw new Error("控制服务返回了非 JSON 响应");
  const envelope = parseEnvelope(await response.json(), parser);
  if (!response.ok || !envelope.data) throw new Error("控制服务请求失败");
  return parser(envelope.data);
}

export const controlClient = {
  snapshot: (signal?: AbortSignal): Promise<ControlSnapshot> => get("/control/v1/snapshot", parseSnapshot, signal),
  failures: (signal?: AbortSignal): Promise<FailureProjection> => get("/control/v1/failures", parseFailureProjection, signal),
  failureHistory: (limit = 100, signal?: AbortSignal): Promise<FailureHistoryProjection> =>
    get(`/control/v1/failures/history?limit=${encodeURIComponent(limit)}`, parseFailureHistory, signal),
  git: (signal?: AbortSignal): Promise<GitProjection> => get("/control/v1/git", parseGitProjection, signal),
  codexSessions: (signal?: AbortSignal): Promise<CodexSessionsProjection> => get("/control/v1/codex-sessions", parseCodexSessions, signal),
  validation: (signal?: AbortSignal): Promise<ValidationProjection> => get("/control/v1/validation", parseValidationProjection, signal),
  validationHistory: (limit = 50, signal?: AbortSignal): Promise<ValidationHistoryProjection> =>
    get(`/control/v1/validation/history?limit=${encodeURIComponent(limit)}`, parseValidationHistory, signal),
  continuations: (signal?: AbortSignal): Promise<ContinuationProjection> => get("/control/v1/continuations", parseContinuationProjection, signal),
  workflow: (runId: string, signal?: AbortSignal): Promise<WorkflowDetail> =>
    get(`/control/v1/workflows/${encodeURIComponent(runId)}`, parseWorkflowDetail, signal),
  logs: (before?: number, signal?: AbortSignal): Promise<LogRange> => {
    const query = before ? `?limit=250&before=${before}` : "?limit=250";
    return get(`/control/v1/logs${query}`, parseLogRange, signal);
  },
};
