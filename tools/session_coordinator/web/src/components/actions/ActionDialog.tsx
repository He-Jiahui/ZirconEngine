import { Alert, Button, Dialog, DialogActions, DialogContent, DialogTitle, Stack, TextField, Typography } from "@mui/material";
import { useEffect, useState } from "react";
import type { ActionRecord } from "../../api/contracts";
import { RiskSummary } from "./RiskSummary";

export function ActionDialog({ action, comparison, busy, error, onConfirm, onCancel, onClose }: {
  action: ActionRecord | null;
  comparison?: { previous: ActionRecord; fresh: ActionRecord } | null;
  busy: boolean;
  error: string | null;
  onConfirm: (phrase: string, reason: string) => void;
  onCancel: (reason: string) => void;
  onClose: () => void;
}) {
  const [phrase, setPhrase] = useState(""); const [reason, setReason] = useState("");
  useEffect(() => { setPhrase(""); setReason(""); }, [action?.actionId]);
  return <Dialog open={Boolean(action)} onClose={busy ? undefined : onClose} fullWidth maxWidth="md" aria-labelledby="action-dialog-title">
    {action && <><DialogTitle id="action-dialog-title">确认受控操作：{action.kind}</DialogTitle><DialogContent><Stack spacing={2} sx={{ pt: 1 }}>
      <ImpactDiff comparison={comparison ?? null} />
      <RiskSummary action={action} />
      <Typography variant="body2">预览有效期至 {action.expiresAt}。状态发生变化后必须重新预览，界面不会自动重试。</Typography>
      {error && <Alert severity="warning">{error}</Alert>}
      <TextField label="操作原因" value={reason} onChange={(event) => setReason(event.target.value)} required multiline minRows={2} />
      <TextField label="确认短语" value={phrase} onChange={(event) => setPhrase(event.target.value)} required helperText={action.confirmationPhrase ? `请输入：${action.confirmationPhrase}` : "该动作仅支持预览"} />
    </Stack></DialogContent><DialogActions>
      <Button onClick={() => onCancel(reason)} disabled={busy || !reason.trim()}>取消此预览</Button>
      <Button onClick={onClose} disabled={busy}>关闭</Button>
      <Button variant="contained" color={action.risk === "red" ? "error" : "primary"} onClick={() => onConfirm(phrase, reason)} disabled={busy || !reason.trim() || phrase !== action.confirmationPhrase}>确认执行</Button>
    </DialogActions></>}
  </Dialog>;
}

export function ImpactDiff({ comparison }: { comparison: { previous: ActionRecord; fresh: ActionRecord } | null }) {
  if (!comparison) return null;
  const previousImpact = new Set(comparison.previous.impact);
  const freshImpact = new Set(comparison.fresh.impact);
  const added = comparison.fresh.impact.filter((item) => !previousImpact.has(item));
  const removed = comparison.previous.impact.filter((item) => !freshImpact.has(item));
  return <Alert severity="warning"><Stack spacing={0.5}>
    <Typography variant="subtitle2">状态已变化，以下是新旧预览差异（尚未执行）</Typography>
    <Typography variant="body2">新增影响：{added.length ? added.join("；") : "无"}</Typography>
    <Typography variant="body2">移除影响：{removed.length ? removed.join("；") : "无"}</Typography>
    <Typography variant="caption">状态指纹：{comparison.previous.stateFingerprint.slice(0, 12)} → {comparison.fresh.stateFingerprint.slice(0, 12)}</Typography>
  </Stack></Alert>;
}
