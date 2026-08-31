import type { ControlEvent, ControlSnapshot } from "../api/contracts";

export interface ControlState {
  snapshot: ControlSnapshot | null;
  cursor: number;
  loading: boolean;
  connected: boolean;
  needsRefresh: boolean;
  error: string | null;
  retryNonce: number;
}

export type ControlAction =
  | { type: "loading" }
  | { type: "snapshot"; snapshot: ControlSnapshot }
  | { type: "event"; event: ControlEvent }
  | { type: "resync" }
  | { type: "connection"; connected: boolean }
  | { type: "error"; message: string | null };

export const initialControlState: ControlState = {
  snapshot: null, cursor: 0, loading: true, connected: false, needsRefresh: false, error: null, retryNonce: 0,
};

export function snapshotRetryDelay(attempt: number): number {
  const exponent = Math.max(0, Math.floor(Number.isFinite(attempt) ? attempt : 0));
  return Math.min(1_000 * (2 ** Math.min(exponent, 4)), 10_000);
}

export function controlReducer(state: ControlState, action: ControlAction): ControlState {
  switch (action.type) {
    case "loading": return { ...state, loading: true };
    case "snapshot": return { ...state, snapshot: action.snapshot, cursor: action.snapshot.eventCursor, loading: false, needsRefresh: false, error: null, retryNonce: 0 };
    case "event":
      if (action.event.id <= state.cursor) return state;
      if (action.event.id !== state.cursor + 1) return { ...state, needsRefresh: true, error: "事件序列存在缺口，正在重新同步" };
      return { ...state, cursor: action.event.id, needsRefresh: true };
    case "resync": return { ...state, needsRefresh: true };
    case "connection": return { ...state, connected: action.connected, error: action.connected ? null : state.error };
    case "error": return action.message === null
      ? { ...state, error: null }
      : { ...state, loading: false, needsRefresh: false, error: action.message, retryNonce: state.retryNonce + 1 };
  }
}
