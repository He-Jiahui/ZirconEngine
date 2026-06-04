import { invoke } from "@tauri-apps/api/core";
import { fallbackShellState } from "../data/hubData";
import type { HubShellState } from "../types/hub";

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

export async function dispatchHubAction(actionId: string, targetId?: string): Promise<HubShellState> {
  if (!isTauriRuntime()) {
    return fallbackShellState;
  }

  try {
    return await invoke<HubShellState>("hub_action", {
      request: { actionId, targetId },
    });
  } catch {
    return fallbackShellState;
  }
}

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
