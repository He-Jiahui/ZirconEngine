import type { ActionActivityResponse, ActionCatalog, ActionRecord, ControlAuthSession, JsonObject } from "../api/contracts";

let csrfToken = "";

export class ControlActionError extends Error {
  constructor(public readonly code: string, message: string) { super(message); }
}

async function request<T>(method: string, path: string, body?: JsonObject, signal?: AbortSignal): Promise<T> {
  const headers: Record<string, string> = { Accept: "application/json" };
  if (body) headers["Content-Type"] = "application/json";
  if (method !== "GET" && csrfToken) headers["X-CSRF-Token"] = csrfToken;
  const response = await fetch(path, {
    method,
    credentials: "same-origin",
    headers,
    body: body ? JSON.stringify(body) : undefined,
    signal,
  });
  const envelope = await response.json() as { ok?: boolean; data?: T; error?: { code?: string; message?: string } };
  if (!response.ok || envelope.ok !== true || envelope.data === undefined) {
    throw new ControlActionError(envelope.error?.code ?? "control_request_failed", envelope.error?.message ?? "受控操作请求失败");
  }
  return envelope.data;
}

export const actionClient = {
  authSession: () => request<ControlAuthSession>("GET", "/control/v1/auth/session"),
  catalog: () => request<ActionCatalog>("GET", "/control/v1/actions/catalog"),
  activity: (limit = 50, signal?: AbortSignal) => request<ActionActivityResponse>("GET", `/control/v1/actions?limit=${encodeURIComponent(limit)}`, undefined, signal),
  elevate: async (grant: string): Promise<ControlAuthSession> => {
    const session = await request<ControlAuthSession & { csrfToken: string }>("POST", "/control/v1/auth/elevate", { grant });
    csrfToken = session.csrfToken;
    return session;
  },
  startLoopbackValidation: (parameters: JsonObject) => request<{ action: ActionRecord }>(
    "POST", "/control/v1/validation/queue/start", parameters,
  ),
  continueValidationQueue: () => request<{
    ticket: { ticketId: string; sessionId: string; planPath: string; status: string } | null;
    progress: Record<string, number>;
  }>("POST", "/control/v1/validation/queue/continue", {}),
  preview: (kind: string, parameters: JsonObject) => request<{ action: ActionRecord }>("POST", "/control/v1/actions/preview", { kind, parameters }),
  confirm: (actionId: string, phrase: string, reason: string) => request<{ action: ActionRecord }>("POST", `/control/v1/actions/${encodeURIComponent(actionId)}/confirm`, { phrase, reason }),
  cancel: (actionId: string, reason: string) => request<{ action: ActionRecord }>("POST", `/control/v1/actions/${encodeURIComponent(actionId)}/cancel`, { reason }),
  status: (actionId: string, signal?: AbortSignal) => request<{ action: ActionRecord }>("GET", `/control/v1/actions/${encodeURIComponent(actionId)}`, undefined, signal),
};
