import { List, ListItem, ListItemText } from "@mui/material";
import type { WorkflowAttempt } from "../../api/contracts";
import { StatusText } from "../StatusText";

export function AttemptTimeline({ attempts }: { attempts: WorkflowAttempt[] }) {
  return <List aria-label="不可变尝试历史">{attempts.map((attempt) => <ListItem key={attempt.attemptId} divider secondaryAction={<StatusText value={attempt.state} />}>
    <ListItemText primary={`第 ${attempt.attemptNumber} 次${attempt.accepted ? "（当前已接受）" : ""}`} secondary={`${attempt.startedAt ?? "未开始"} → ${attempt.completedAt ?? "未完成"}`} />
  </ListItem>)}</List>;
}
