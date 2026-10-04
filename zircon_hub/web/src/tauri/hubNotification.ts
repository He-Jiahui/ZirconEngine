import type { HubTaskSummary } from "../types/hub";

/**
 * Identifies a logical task message without treating progress or queue depth
 * updates as a new notification.
 */
export function taskNotificationKey(task: HubTaskSummary): string {
  return JSON.stringify([
    task.taskId,
    task.operation,
    task.tone,
    task.label,
    task.detail,
    task.recovery,
    task.running,
  ]);
}

export function shouldOpenTaskNotification(previousKey: string | null, task: HubTaskSummary): boolean {
  if (!isActionableTask(task)) {
    return false;
  }
  return previousKey !== taskNotificationKey(task);
}

export function shouldAutoHideTask(task: HubTaskSummary): boolean {
  return !task.running;
}

function isActionableTask(task: HubTaskSummary): boolean {
  return task.running || task.tone !== "neutral" || Boolean(task.recovery);
}
