import { useEffect, useRef, useState } from "react";
import { HubErrorBoundary, HubSnackbar } from "./components/feedback";
import { HubWindow } from "./components/shell";
import { fallbackShellState } from "./data/hubData";
import { dispatchHubAction, loadHubState, subscribeHubStateChanged } from "./tauri/hubApi";
import type { WindowActionFailureHandler } from "./tauri/windowActionScheduler";
import type { HubActionHandler, HubShellState } from "./types/hub";

export function App() {
  const [state, setState] = useState<HubShellState>(fallbackShellState);
  const [snackbarOpen, setSnackbarOpen] = useState(false);
  const stateRef = useRef(state);
  const stateGenerationRef = useRef(0);
  const actionSequenceRef = useRef(0);

  function applyHubState(nextState: HubShellState) {
    stateGenerationRef.current += 1;
    setState(() => nextState);
  }

  async function reloadHubState() {
    const nextState = await loadHubState();
    applyHubState(nextState);
  }

  useEffect(() => {
    stateRef.current = state;
  }, [state]);

  useEffect(() => {
    let cancelled = false;
    const actionSequenceAtLoad = actionSequenceRef.current;
    const stateGenerationAtLoad = stateGenerationRef.current;
    loadHubState().then((nextState) => {
      if (
        !cancelled &&
        actionSequenceRef.current === actionSequenceAtLoad &&
        stateGenerationRef.current === stateGenerationAtLoad
      ) {
        applyHubState(nextState);
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
        applyHubState(nextState);
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
        stateGenerationRef.current += 1;
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
            taskId: current.taskSummary.taskId,
            queued: current.taskSummary.queued,
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
    const actionSequence = actionSequenceRef.current + 1;
    actionSequenceRef.current = actionSequence;
    const stateGenerationAtDispatch = stateGenerationRef.current;

    try {
      const nextState = await dispatchHubAction(actionId, targetId, payload);
      if (actionSequence === actionSequenceRef.current && stateGenerationRef.current === stateGenerationAtDispatch) {
        applyHubState(nextState);
      }
    } catch (error) {
      if (actionSequence !== actionSequenceRef.current || stateGenerationRef.current !== stateGenerationAtDispatch) {
        return;
      }

      const shellText = stateRef.current.ui.shell;
      stateGenerationRef.current += 1;
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
          taskId: current.taskSummary.taskId,
          queued: current.taskSummary.queued,
        },
        taskStatus: current.taskStatus.map((status) => (status.id === "error" ? { ...status, tone: "error" } : status)),
      }));
      console.error(shellText.actionFailed, error);
    }
  };

  const handleWindowActionFailure: WindowActionFailureHandler = (action, error) => {
    const shellText = stateRef.current.ui.shell;
    stateGenerationRef.current += 1;
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
        taskId: current.taskSummary.taskId,
        queued: current.taskSummary.queued,
      },
      taskStatus: current.taskStatus.map((status) => (status.id === "error" ? { ...status, tone: "error" } : status)),
    }));
    console.error(`${shellText.actionFailed}: ${action}`, error);
  };

  return (
    <>
      <HubErrorBoundary shellText={state.ui.shell} onReset={() => void reloadHubState()}>
        <HubWindow state={state} onAction={handleAction} onWindowActionFailure={handleWindowActionFailure} />
      </HubErrorBoundary>
      <HubSnackbar task={state.taskSummary} open={snackbarOpen} onClose={() => setSnackbarOpen(false)} />
    </>
  );
}
