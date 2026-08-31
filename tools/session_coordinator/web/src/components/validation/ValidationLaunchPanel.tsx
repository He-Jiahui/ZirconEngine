import { Alert, Button, Stack, Typography } from "@mui/material";
import { useState } from "react";
import { actionClient } from "../../actions/actionClient";
import { actionMutationBlockReason } from "../../actions/actionParameters";
import type { ServiceProjection } from "../../api/contracts";
import { useControlStore } from "../../state/controlStore";

export function ValidationLaunchPanel({ service }: { service: ServiceProjection }) {
  const { refresh } = useControlStore();
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const mutationBlocked = actionMutationBlockReason(service);

  const start = async () => {
    if (mutationBlocked) return;
    setBusy(true);
    setMessage(null);
    try {
      const response = await actionClient.continueValidationQueue();
      setMessage(response.ticket
        ? `已推进验证：${response.ticket.ticketId} · ${response.ticket.status}`
        : "当前没有待验证任务，队列将在新任务入队后自动继续。");
      refresh();
    } catch (issue) {
      setMessage(errorMessage(issue));
    } finally {
      setBusy(false);
    }
  };

  return <Stack spacing={1.5}>
    <Typography variant="body2" color="text.secondary">协调器按 FIFO 自动领取最早的待验证票据；已有验证在物化或运行时，只推进其下一阶段。</Typography>
    {mutationBlocked && <Alert severity="warning">{mutationBlocked}</Alert>}
    {message && <Alert severity={message.startsWith("已推进") ? "success" : "warning"}>{message}</Alert>}
    <Stack direction="row" sx={{ justifyContent: "flex-end" }}><Button variant="contained" onClick={start} disabled={busy || Boolean(mutationBlocked)}>{busy ? "正在推进" : "立即推进下一项验证"}</Button></Stack>
  </Stack>;
}

function errorMessage(issue: unknown): string {
  return issue instanceof Error ? issue.message : "验证请求失败";
}
