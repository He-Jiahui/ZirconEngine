import type { HubActionHistoryItem } from "../types/hub";

// 交付页以两条历史视图复用同一动作快照，条目仍引用原记录并保留后端提供的顺序。
export interface DeliveryActionProjection {
  packageActions: HubActionHistoryItem[];
  installActions: HubActionHistoryItem[];
}

// 快照变化时一次投影打包及安装历史；这里匹配历史记录的种类，不能用启动命令名称替代。
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
