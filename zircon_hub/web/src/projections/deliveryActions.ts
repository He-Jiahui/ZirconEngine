import type { HubActionHistoryItem } from "../types/hub";

export interface DeliveryActionProjection {
  packageActions: HubActionHistoryItem[];
  installActions: HubActionHistoryItem[];
}

export function collectDeliveryActions(
  actions: readonly HubActionHistoryItem[],
): DeliveryActionProjection {
  const packageActions: HubActionHistoryItem[] = [];
  const installActions: HubActionHistoryItem[] = [];
  for (const action of actions) {
    if (action.kind === "package-project") {
      packageActions.push(action);
    } else if (action.kind === "install-project") {
      installActions.push(action);
    }
  }
  return { packageActions, installActions };
}
