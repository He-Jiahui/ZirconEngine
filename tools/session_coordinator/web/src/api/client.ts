import type { ControlSnapshot, LogRange, WorkflowDetail } from "./contracts";
import { parseEnvelope, parseLogRange, parseSnapshot, parseWorkflowDetail } from "./validation";

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
  workflow: (runId: string, signal?: AbortSignal): Promise<WorkflowDetail> =>
    get(`/control/v1/workflows/${encodeURIComponent(runId)}`, parseWorkflowDetail, signal),
  logs: (before?: number, signal?: AbortSignal): Promise<LogRange> => {
    const query = before ? `?limit=250&before=${before}` : "?limit=250";
    return get(`/control/v1/logs${query}`, parseLogRange, signal);
  },
};
