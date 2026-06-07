import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { fallbackShellState } from "../data/hubData";
import type { HubActionId, HubActionPayload, HubShellState } from "../types/hub";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export async function loadHubState(): Promise<HubShellState> {
  if (!isTauriRuntime()) {
    return fallbackShellState;
  }

  try {
    return await invoke<HubShellState>("hub_state");
  } catch {
    return fallbackShellState;
  }
}

export async function dispatchHubAction<TActionId extends HubActionId>(
  actionId: TActionId,
  targetId?: string,
  payload?: HubActionPayload<TActionId>,
): Promise<HubShellState> {
  if (!isTauriRuntime()) {
    return fallbackShellState;
  }

  return await invoke<HubShellState>("hub_action", {
    request: { actionId, targetId, payload },
  });
}

export async function subscribeHubStateChanged(onStateChanged: (state: HubShellState) => void): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return () => {};
  }

  return await listen<HubShellState>("hub-state-changed", (event) => {
    onStateChanged(event.payload);
  });
}

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
