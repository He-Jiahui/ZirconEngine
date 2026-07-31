import { Alert, Button, Stack, Typography } from "@mui/material";
import type { ActionRecord } from "../../api/contracts";

const cancellableLifecycleKinds = new Set(["service.stop", "service.restart", "service.force_stop", "service.rollover"]);

export function ActionActivityList({ actions, trackingErrors, onCancelExecuting, cancelDisabled = false }: {
  actions: ActionRecord[];
  trackingErrors: Record<string, string>;
  onCancelExecuting?: (action: ActionRecord) => void;
  cancelDisabled?: boolean;
}) {
  const visibleIds = new Set(actions.map((action) => action.actionId));
  const unmatchedErrors = Object.entries(trackingErrors).filter(([actionId]) => !visibleIds.has(actionId));
  if (!actions.length && !unmatchedErrors.length) return <Typography color="text.secondary">尚无操作</Typography>;
  return <Stack spacing={1}>{actions.map((action) => {
    const issue = trackingErrors[action.actionId];
    const severity = action.status === "failed" || action.status === "state_changed" || issue ? "warning" : action.status === "succeeded" ? "success" : "info";
    const waitingForCargo = action.kind === "service.rollover" && action.status === "executing" && action.result?.waitingForCargo === true;
    return <Alert key={`${action.actionId}-${action.status}`} severity={severity}>
      <Typography component="div">{action.kind} · {action.status} · {action.actionId.slice(0, 12)}</Typography>
      <Typography variant="body2">执行者：{action.actor} · 原因：{action.reason ?? "尚未确认"}</Typography>
      {waitingForCargo && <Typography variant="body2">正在等待受管 Cargo 自然结束；不会排空或停止其他 Session。</Typography>}
      {action.errorCode && <Typography variant="body2">错误代码：{action.errorCode}</Typography>}
      {action.result && <Typography component="pre" variant="caption" className="json-evidence">{JSON.stringify(action.result, null, 2)}</Typography>}
      {issue && <Typography variant="body2">状态跟踪失败：{issue}</Typography>}
      {action.status === "executing" && cancellableLifecycleKinds.has(action.kind) && onCancelExecuting && <Button
        size="small"
        color="warning"
        disabled={cancelDisabled}
        onClick={() => onCancelExecuting(action)}
      >{action.kind === "service.rollover" ? "取消延后重启" : "取消排空并恢复服务"}</Button>}
    </Alert>;
  })}{unmatchedErrors.map(([actionId, issue]) => <Alert key={actionId} severity="warning">
    <Typography component="span">待处理动作 · {actionId.slice(0, 12)}</Typography>
    <Typography variant="body2">状态恢复失败：{issue}</Typography>
  </Alert>)}</Stack>;
}
