import type { ActionRecord } from "../api/contracts";

const storageKey = "zircon-control.pending-actions.v1";

export interface PendingActionReference {
  actionId: string;
  kind: string;
}

export interface ActionPollingOptions {
  lookup: (actionId: string) => Promise<{ action: ActionRecord }>;
  onUpdate: (action: ActionRecord) => void;
  wait?: (milliseconds: number) => Promise<void>;
  intervalMs?: number;
  signal?: AbortSignal;
}

export function isActionTerminal(status: ActionRecord["status"]): boolean {
  return status !== "previewed" && status !== "executing";
}

export async function pollActionUntilTerminal(
  initial: ActionRecord,
  options: ActionPollingOptions,
): Promise<ActionRecord> {
  let current = initial;
  const wait = options.wait ?? delay;
  while (!isActionTerminal(current.status)) {
    if (options.signal?.aborted) throw abortError();
    await wait(options.intervalMs ?? 1_000);
    if (options.signal?.aborted) throw abortError();
    current = (await options.lookup(current.actionId)).action;
    options.onUpdate(current);
  }
  return current;
}

export function loadPendingActions(storage: Storage | null = browserStorage()): PendingActionReference[] {
  if (!storage) return [];
  try {
    const parsed = JSON.parse(storage.getItem(storageKey) ?? "[]") as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((item): item is PendingActionReference => Boolean(
      item && typeof item === "object"
      && typeof (item as PendingActionReference).actionId === "string"
      && typeof (item as PendingActionReference).kind === "string",
    ));
  } catch {
    return [];
  }
}

export function savePendingActions(actions: PendingActionReference[], storage: Storage | null = browserStorage()): void {
  if (!storage) return;
  try {
    storage.setItem(storageKey, JSON.stringify(actions));
  } catch {
    // Storage can be disabled by browser policy; live tracking still continues in memory.
  }
}

export function rememberPendingAction(action: ActionRecord): void {
  if (isActionTerminal(action.status)) return;
  const current = loadPendingActions().filter((item) => item.actionId !== action.actionId);
  savePendingActions([...current, { actionId: action.actionId, kind: action.kind }]);
}

export function forgetPendingAction(actionId: string): void {
  savePendingActions(loadPendingActions().filter((item) => item.actionId !== actionId));
}

function browserStorage(): Storage | null {
  return typeof window === "undefined" ? null : window.sessionStorage;
}

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, milliseconds));
}

function abortError(): Error {
  const error = new Error("动作状态跟踪已取消");
  error.name = "AbortError";
  return error;
}
