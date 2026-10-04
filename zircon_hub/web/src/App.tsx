import RefreshOutlinedIcon from "@mui/icons-material/RefreshOutlined";
import { Alert, Box, Button, CircularProgress, Typography } from "@mui/material";
import { useCallback, useEffect, useRef, useState } from "react";
import type { Dispatch, MutableRefObject, SetStateAction } from "react";
import { HubErrorBoundary, HubSnackbar } from "./components/feedback";
import { HubWindow } from "./components/shell";
import { fallbackShellState } from "./data/hubData";
import { canDispatchHubAction, resolveHubBootstrap, type HubBootstrapOutcome } from "./tauri/hubBootstrap";
import { dispatchHubAction, isTauriRuntime, loadHubState, subscribeHubStateChanged } from "./tauri/hubApi";
import { shouldOpenTaskNotification, taskNotificationKey } from "./tauri/hubNotification";
import {
  HubBootstrapRetryBudget,
  HubStateChronology,
  acceptHubState,
  type HubStateSource,
} from "./tauri/hubStateChronology";
import { hubTokens } from "./theme/tokens";
import type { WindowActionFailureHandler } from "./tauri/windowActionScheduler";
import type { HubActionHandler, HubShellState } from "./types/hub";
import { AccountProvider } from "./account";

export function App() {
  const [bootstrap, setBootstrap] = useState<HubBootstrapOutcome>(() =>
    isTauriRuntime() ? { status: "booting" } : { status: "ready", state: fallbackShellState },
  );
  const [snackbarOpen, setSnackbarOpen] = useState(false);
  const lastTaskNotificationKeyRef = useRef<string | null>(null);
  const stateRef = useRef(fallbackShellState);
  const stateGenerationRef = useRef(0);
  const actionSequenceRef = useRef(0);
  const bootstrapSequenceRef = useRef(0);
  const bootstrapRetryBudgetRef = useRef(new HubBootstrapRetryBudget());
  const chronologyRef = useRef(
    new HubStateChronology(isTauriRuntime() ? undefined : fallbackShellState),
  );
  const state = bootstrap.status === "ready" ? bootstrap.state : null;
  const readyStateRef = useRef<HubBootstrapOutcome>(bootstrap);

  function applyHubState(nextState: HubShellState, source: HubStateSource): boolean {
    const acceptedState = acceptHubState(chronologyRef.current, nextState, source);
    if (!acceptedState) {
      return false;
    }
    stateGenerationRef.current += 1;
    stateRef.current = acceptedState;
    const nextBootstrap = { status: "ready" as const, state: acceptedState };
    readyStateRef.current = nextBootstrap;
    setBootstrap(nextBootstrap);
    return true;
  }

  const reloadHubState = useCallback(async (automaticRetry = false) => {
    if (!automaticRetry) {
      bootstrapRetryBudgetRef.current.reset();
    }
    const bootstrapSequence = bootstrapSequenceRef.current + 1;
    bootstrapSequenceRef.current = bootstrapSequence;
    const readyBeforeReload = readyStateRef.current;
    chronologyRef.current.beginBootstrap();
    readyStateRef.current = { status: "booting" };
    setBootstrap({ status: "booting" });
    const outcome = await resolveHubBootstrap(loadHubState);
    if (
      bootstrapSequence !== bootstrapSequenceRef.current
    ) {
      return;
    }
    if (outcome.status === "ready") {
      if (!applyHubState(outcome.state, "bootstrap")) {
        if (chronologyRef.current.hasPendingBootstrap()) {
          // A stale response left a newer epoch staged. Keep the transaction
          // boundary active and request that epoch's authority. Bound the
          // automatic recovery so a permanently stale backend cannot create a
          // tight request loop.
          if (bootstrapRetryBudgetRef.current.tryAcquire()) {
            void reloadHubState(true);
            return;
          }
          chronologyRef.current.quarantine();
          const nextBootstrap = {
            status: "protocol-mismatch" as const,
            error: new Error("Hub backend epoch could not be reconciled"),
          };
          readyStateRef.current = nextBootstrap;
          setBootstrap(nextBootstrap);
          return;
        }
        if (!canDispatchHubAction(readyStateRef.current)) {
          readyStateRef.current = readyBeforeReload;
          setBootstrap(readyBeforeReload);
        }
      }
      bootstrapRetryBudgetRef.current.reset();
      return;
    }
    if (outcome.status === "protocol-mismatch") {
      chronologyRef.current.quarantine();
    } else {
      chronologyRef.current.cancelBootstrap();
    }
    readyStateRef.current = outcome;
    setBootstrap(outcome);
  }, []);

  useEffect(() => {
    if (isTauriRuntime()) {
      void reloadHubState();
    }
  }, [reloadHubState]);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    if (!isTauriRuntime()) {
      return;
    }

    subscribeHubStateChanged(
      (nextState) => {
        if (!cancelled) {
          applyHubState(nextState, "event");
        }
      },
      (error) => {
        if (!cancelled) {
          // A protocol mismatch terminates the active bootstrap transaction.
          // Invalidate its sequence so a late hub_state response cannot
          // restore a ready shell after recovery has been shown.
          chronologyRef.current.quarantine();
          bootstrapSequenceRef.current += 1;
          const nextBootstrap = { status: "protocol-mismatch" as const, error };
          readyStateRef.current = nextBootstrap;
          setBootstrap(nextBootstrap);
        }
      },
      (payload) => chronologyRef.current.isProvablyStaleEvent(payload),
    )
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
        setBootstrap((current) => {
          if (current.status !== "ready") {
            return { status: "backend-unavailable", error };
          }
          stateGenerationRef.current += 1;
          const nextState = {
            ...current.state,
            taskSummary: {
              label: shellText.liveUpdatesUnavailable,
              detail: shellText.liveUpdatesUnavailableDetail,
              tone: "warning" as const,
              running: false,
              cancellable: false,
              recovery: shellText.stateRefreshAfterCommand,
              operation: shellText.liveUpdatesUnavailable,
              progressPercent: 0,
              taskId: current.state.taskSummary.taskId,
              queued: current.state.taskSummary.queued,
            },
          };
          stateRef.current = nextState;
          const nextBootstrap = { status: "ready" as const, state: nextState };
          readyStateRef.current = nextBootstrap;
          return nextBootstrap;
        });
        console.warn(shellText.liveUpdatesUnavailable, error);
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    if (!state) {
      lastTaskNotificationKeyRef.current = null;
      setSnackbarOpen(false);
      return;
    }

    const nextTaskKey = taskNotificationKey(state.taskSummary);
    if (shouldOpenTaskNotification(lastTaskNotificationKeyRef.current, state.taskSummary)) {
      lastTaskNotificationKeyRef.current = nextTaskKey;
      setSnackbarOpen(true);
      return;
    }

    if (!state.taskSummary.running && state.taskSummary.tone === "neutral" && !state.taskSummary.recovery) {
      lastTaskNotificationKeyRef.current = null;
      setSnackbarOpen(false);
    }
  }, [state]);

  const handleAction: HubActionHandler = async (actionId, targetId, payload) => {
    if (!canDispatchHubAction(readyStateRef.current)) {
      return;
    }
    const actionSequence = actionSequenceRef.current + 1;
    actionSequenceRef.current = actionSequence;
    const stateGenerationAtDispatch = stateGenerationRef.current;

    try {
      const nextState = await dispatchHubAction(actionId, targetId, payload);
      applyHubState(nextState, "invoke");
    } catch (error) {
      if (actionSequence !== actionSequenceRef.current || stateGenerationRef.current !== stateGenerationAtDispatch) {
        return;
      }

      const shellText = stateRef.current.ui.shell;
      stateGenerationRef.current += 1;
      updateReadyState(setBootstrap, (current) => ({
        ...current,
        taskSummary: failedTaskSummary(current, shellText),
        taskStatus: current.taskStatus.map((status) => (status.id === "error" ? { ...status, tone: "error" } : status)),
      }), stateRef);
      console.error(shellText.actionFailed, error);
    }
  };

  const windowActionGeneration = stateGenerationRef.current;
  const windowActionEpoch = state?.backendEpoch;
  const windowActionRevision = state?.stateRevision;
  const handleWindowActionFailure: WindowActionFailureHandler = (action, error) => {
    if (
      !canDispatchHubAction(readyStateRef.current) ||
      stateGenerationRef.current !== windowActionGeneration ||
      stateRef.current.backendEpoch !== windowActionEpoch ||
      stateRef.current.stateRevision !== windowActionRevision
    ) {
      return;
    }
    const shellText = stateRef.current.ui.shell;
    stateGenerationRef.current += 1;
    updateReadyState(setBootstrap, (current) => ({
      ...current,
      taskSummary: failedTaskSummary(current, shellText),
      taskStatus: current.taskStatus.map((status) => (status.id === "error" ? { ...status, tone: "error" } : status)),
    }), stateRef);
    console.error(`${shellText.actionFailed}: ${action}`, error);
  };

  if (!state) {
    return <HubBootstrapSurface bootstrap={bootstrap} onRetry={() => void reloadHubState()} />;
  }

  return (
    <>
      <HubErrorBoundary shellText={state.ui.shell} onReset={() => void reloadHubState()}>
        <AccountProvider key={state.backendEpoch} backendEpoch={state.backendEpoch}>
          <HubWindow state={state} onAction={handleAction} onWindowActionFailure={handleWindowActionFailure} />
        </AccountProvider>
      </HubErrorBoundary>
      <HubSnackbar task={state.taskSummary} open={snackbarOpen} onClose={() => setSnackbarOpen(false)} />
    </>
  );
}

function updateReadyState(
  setBootstrap: Dispatch<SetStateAction<HubBootstrapOutcome>>,
  update: (state: HubShellState) => HubShellState,
  stateRef: MutableRefObject<HubShellState>,
) {
  setBootstrap((current) => {
    if (current.status !== "ready") {
      return current;
    }
    const state = update(current.state);
    stateRef.current = state;
    return { status: "ready", state };
  });
}

function failedTaskSummary(state: HubShellState, shellText: HubShellState["ui"]["shell"]) {
  return {
    label: shellText.actionFailed,
    detail: shellText.actionFailedDetail,
    tone: "error" as const,
    running: false,
    cancellable: false,
    recovery: shellText.checkActionTarget,
    operation: shellText.actionFailed,
    progressPercent: 0,
    taskId: state.taskSummary.taskId,
    queued: state.taskSummary.queued,
  };
}

function HubBootstrapSurface({ bootstrap, onRetry }: { bootstrap: HubBootstrapOutcome; onRetry: () => void }) {
  const loading = bootstrap.status === "booting";
  const protocolMismatch = bootstrap.status === "protocol-mismatch";
  return (
    <Box
      data-testid="hub-bootstrap-surface"
      sx={{
        width: "100vw",
        height: "100vh",
        display: "grid",
        placeItems: "center",
        p: 3,
        boxSizing: "border-box",
        color: hubTokens.colors.text,
        background: hubTokens.gradients.window,
      }}
    >
      {loading ? (
        <Box role="status" sx={{ display: "grid", justifyItems: "center", gap: 1.5 }}>
          <CircularProgress size={30} />
          <Typography>Loading Hub state...</Typography>
        </Box>
      ) : (
        <Alert
          severity="error"
          action={
            <Button color="inherit" startIcon={<RefreshOutlinedIcon />} onClick={onRetry}>
              Retry
            </Button>
          }
          sx={{ width: "min(620px, 100%)" }}
        >
          <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
            {protocolMismatch ? "Hub protocol mismatch" : "Hub backend unavailable"}
          </Typography>
          <Typography variant="body2">
            {protocolMismatch
              ? "The backend returned state this version of Hub cannot safely use."
              : "Hub could not load authoritative state. Retry after the backend is available."}
          </Typography>
        </Alert>
      )}
    </Box>
  );
}
