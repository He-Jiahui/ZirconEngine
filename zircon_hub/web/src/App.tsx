import { useEffect, useRef, useState } from "react";
import { HubSnackbar } from "./components/feedback";
import { HubWindow } from "./components/shell";
import { fallbackShellState } from "./data/hubData";
import { dispatchHubAction, loadHubState, subscribeHubStateChanged } from "./tauri/hubApi";
import type { HubActionHandler, HubShellState } from "./types/hub";

export function App() {
  const [state, setState] = useState<HubShellState>(fallbackShellState);
  const [snackbarOpen, setSnackbarOpen] = useState(false);
  const stateRef = useRef(state);

  useEffect(() => {
    stateRef.current = state;
  }, [state]);

  useEffect(() => {
    let cancelled = false;
    loadHubState().then((nextState) => {
      if (!cancelled) {
        setState(nextState);
      }
    });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    subscribeHubStateChanged((nextState) => {
      if (!cancelled) {
        setState(nextState);
      }
    })
      .then((cleanup) => {
        if (cancelled) {
          cleanup();
          return;
        }

        unlisten = cleanup;
      })
      .catch((error) => {
        if (cancelled) {
          return;
        }

        const shellText = stateRef.current.ui.shell;
        setState((current) => ({
          ...current,
          taskSummary: {
            label: shellText.liveUpdatesUnavailable,
            detail: shellText.liveUpdatesUnavailableDetail,
            tone: "warning",
            running: false,
            recovery: shellText.stateRefreshAfterCommand,
            operation: shellText.liveUpdatesUnavailable,
            progressPercent: 0,
          },
        }));
        console.warn(shellText.liveUpdatesUnavailable, error);
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    if (state.taskSummary.running || state.taskSummary.tone !== "neutral" || state.taskSummary.recovery) {
      setSnackbarOpen(true);
    }
  }, [state.taskSummary.recovery, state.taskSummary.running, state.taskSummary.tone]);

  const handleAction: HubActionHandler = async (actionId, targetId, payload) => {
    try {
      const nextState = await dispatchHubAction(actionId, targetId, payload);
      setState(nextState);
    } catch (error) {
      const shellText = stateRef.current.ui.shell;
      setState((current) => ({
        ...current,
        taskSummary: {
          label: shellText.actionFailed,
          detail: shellText.actionFailedDetail,
          tone: "error",
          running: false,
          recovery: shellText.checkActionTarget,
          operation: shellText.actionFailed,
          progressPercent: 0,
        },
        taskStatus: current.taskStatus.map((status) => (status.id === "error" ? { ...status, tone: "error" } : status)),
      }));
      console.error(shellText.actionFailed, error);
    }
  };

  return (
    <>
      <HubWindow state={state} onAction={handleAction} />
      <HubSnackbar task={state.taskSummary} open={snackbarOpen} onClose={() => setSnackbarOpen(false)} />
    </>
  );
}
