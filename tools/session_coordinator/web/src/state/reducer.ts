import type { ControlEvent, ControlSnapshot } from "../api/contracts";

export interface ControlState {
  snapshot: ControlSnapshot | null;
  cursor: number;
  loading: boolean;
  connected: boolean;
  needsRefresh: boolean;
  error: string | null;
}

export type ControlAction =
  | { type: "loading" }
  | { type: "snapshot"; snapshot: ControlSnapshot }
  | { type: "event"; event: ControlEvent }
  | { type: "resync" }
  | { type: "connection"; connected: boolean }
  | { type: "error"; message: string | null };

export const initialControlState: ControlState = {
  snapshot: null, cursor: 0, loading: true, connected: false, needsRefresh: false, error: null,
};

export function controlReducer(state: ControlState, action: ControlAction): ControlState {
  switch (action.type) {
    case "loading": return { ...state, loading: true };
    case "snapshot": return { ...state, snapshot: action.snapshot, cursor: action.snapshot.eventCursor, loading: false, needsRefresh: false, error: null };
    case "event":
      if (action.event.id <= state.cursor) return state;
      if (action.event.id !== state.cursor + 1) return { ...state, needsRefresh: true, error: "事件序列存在缺口，正在重新同步" };
      return { ...state, cursor: action.event.id, needsRefresh: true };
    case "resync": return { ...state, needsRefresh: true };
    case "connection": return { ...state, connected: action.connected, error: action.connected ? null : state.error };
    case "error": return { ...state, error: action.message };
  }
}
