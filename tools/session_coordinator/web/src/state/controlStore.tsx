import { createContext, useCallback, useContext, useEffect, useMemo, useReducer, useRef, type PropsWithChildren } from "react";
import { controlClient } from "../api/client";
import { openControlEvents } from "../api/events";
import { createResyncDebouncer } from "./refreshDebouncer";
import { controlReducer, initialControlState, snapshotRetryDelay, type ControlState } from "./reducer";

interface ControlContextValue extends ControlState { refresh: () => void }
const ControlContext = createContext<ControlContextValue | null>(null);

export function ControlStoreProvider({ children }: PropsWithChildren) {
  const [state, dispatch] = useReducer(controlReducer, initialControlState);
  const refresh = useCallback(() => dispatch({ type: "resync" }), []);
  const resyncDebouncer = useMemo(() => createResyncDebouncer(refresh), [refresh]);
  const retryAttempt = useRef(0);

  useEffect(() => () => resyncDebouncer.cancel(), [resyncDebouncer]);

  useEffect(() => {
    if (!state.loading && !state.needsRefresh) return;
    const controller = new AbortController();
    dispatch({ type: "loading" });
    controlClient.snapshot(controller.signal)
      .then((snapshot) => dispatch({ type: "snapshot", snapshot }))
      .catch((error) => {
        if (!controller.signal.aborted) dispatch({ type: "error", message: error instanceof Error ? error.message : "快照加载失败" });
      });
    return () => controller.abort();
  }, [state.needsRefresh]);

  useEffect(() => {
    if (!state.error) {
      retryAttempt.current = 0;
      return;
    }
    if (state.needsRefresh) return;
    const timer = window.setTimeout(refresh, snapshotRetryDelay(retryAttempt.current));
    retryAttempt.current += 1;
    return () => window.clearTimeout(timer);
  }, [state.error, state.needsRefresh, state.retryNonce, refresh]);

  useEffect(() => {
    if (!state.snapshot || state.needsRefresh) return;
    return openControlEvents(state.cursor, {
      onEvent: () => resyncDebouncer.schedule(),
      onResync: () => resyncDebouncer.flush(),
      onConnection: (connected) => dispatch({ type: "connection", connected }),
      onError: (message) => dispatch({ type: "error", message }),
    });
  }, [state.snapshot, state.cursor, state.needsRefresh, resyncDebouncer]);

  const value = useMemo(() => ({ ...state, refresh }), [state, refresh]);
  return <ControlContext.Provider value={value}>{children}</ControlContext.Provider>;
}

export function useControlStore(): ControlContextValue {
  const value = useContext(ControlContext);
  if (!value) throw new Error("useControlStore 必须在 ControlStoreProvider 内使用");
  return value;
}
