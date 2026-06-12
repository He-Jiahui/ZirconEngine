import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { fallbackShellState } from "../data/hubData";
import type { HubActionId, HubActionPayload, HubShellState } from "../types/hub";
import { assertHubShellState } from "./hubStateValidator";

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
    return assertHubShellState(await invoke<unknown>("hub_state"));
  } catch (error) {
    console.warn("Hub state validation failed; using fallback state.", error);
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

  return assertHubShellState(
    await invoke<unknown>("hub_action", {
      request: { actionId, targetId, payload },
    }),
  );
}

export async function subscribeHubStateChanged(onStateChanged: (state: HubShellState) => void): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return () => {};
  }

  return await listen<unknown>("hub-state-changed", (event) => {
    try {
      onStateChanged(assertHubShellState(event.payload));
    } catch (error) {
      console.warn("Ignored invalid hub-state-changed payload.", error);
    }
  });
}

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
