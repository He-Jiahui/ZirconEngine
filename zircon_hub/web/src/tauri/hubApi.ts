import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { fallbackShellState } from "../data/hubData";
import type { HubActionId, HubActionPayload, HubShellState } from "../types/hub";
import { assertHubShellState } from "./hubStateValidator";
import { HubStateLoadError } from "./hubBootstrap";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

let fallbackStateRevision = 0n;

export async function loadHubState(): Promise<HubShellState> {
  if (!isTauriRuntime()) {
    return fallbackShellState;
  }

  let payload: unknown;
  try {
    payload = await invoke<unknown>("hub_state");
  } catch (error) {
    throw new HubStateLoadError("backend-unavailable", error);
  }

  try {
    return assertHubShellState(payload);
  } catch (error) {
    throw new HubStateLoadError("protocol-mismatch", error);
  }
}

export async function dispatchHubAction<TActionId extends HubActionId>(
  actionId: TActionId,
  targetId?: string,
  payload?: HubActionPayload<TActionId>,
): Promise<HubShellState> {
  if (!isTauriRuntime()) {
    fallbackStateRevision += 1n;
    const stateRevision = fallbackStateRevision.toString();
    if (actionId === "show-page" && targetId) {
      const page = fallbackShellState.ui.shell.navItems.find((item) => item.id === targetId);
      if (page) {
        return { ...fallbackShellState, activePage: targetId, pageTitle: page.label, stateRevision };
      }
    }
    return { ...fallbackShellState, stateRevision };
  }

  return assertHubShellState(
    await invoke<unknown>("hub_action", {
      request: { actionId, targetId, payload },
    }),
  );
}

export async function subscribeHubStateChanged(
  onStateChanged: (state: HubShellState) => void,
  onProtocolMismatch?: (error: unknown) => void,
  isProvablyStaleEvent?: (payload: unknown) => boolean,
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return () => {};
  }

  return await listen<unknown>("hub-state-changed", (event) => {
    try {
      if (isProvablyStaleEvent?.(event.payload)) {
        return;
      }
      onStateChanged(assertHubShellState(event.payload));
    } catch (error) {
      onProtocolMismatch?.(error);
    }
  });
}

export function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
