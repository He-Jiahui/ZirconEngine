import { invoke } from "@tauri-apps/api/core";
import { isTauriRuntime } from "../tauri/hubApi";
import { assertSnapshot, unavailableSnapshot, type AccountAction, type AccountSnapshot } from "./protocol";

export async function loadAccountState(backendEpoch: string): Promise<AccountSnapshot> {
  if (!isTauriRuntime()) return unavailableSnapshot(backendEpoch);
  const response: unknown = await invoke("account_state");
  assertSnapshot(response);
  return response;
}

export async function dispatchAccountAction(request: AccountAction): Promise<AccountSnapshot> {
  if (!isTauriRuntime()) throw new Error("account_not_configured");
  const response: unknown = await invoke("account_action", { request });
  assertSnapshot(response);
  return response;
}
