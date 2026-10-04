import CloseOutlinedIcon from "@mui/icons-material/CloseOutlined";
import RefreshOutlinedIcon from "@mui/icons-material/RefreshOutlined";
import SearchOutlinedIcon from "@mui/icons-material/SearchOutlined";
import { Alert, Box, Button, Typography } from "@mui/material";
import { useAccount } from "./context";
import type { AccountCopy } from "./copy";
import type { OperationSummary } from "./protocol";

export function operationNoticeDetails(operation: OperationSummary, copy: AccountCopy) {
  const cloud = operation.action === "cloud-commit";
  const packageTargetMigration = operation.action === "package-install" && operation.status === "unknown" && !operation.targetMode;
  const title = cloud ? copy.cloudOperation : operation.action === "create-organization" ? copy.createOrganization : operation.action === "catalog-license" ? copy.catalogLicense : operation.action === "package-install" ? copy.packageInstall : copy.organizationChange;
  const message = packageTargetMigration ? copy.error("account_package_target_migration_required") : cloud
    ? operation.status === "unknown" ? copy.cloudUnknown
      : operation.status === "committed" ? copy.cloudCommitted
        : operation.error === "account_cloud_conflict" ? copy.cloudConflict
          : operation.error === "account_operation_id_conflict" ? copy.cloudIdConflict : copy.failed
    : operation.status === "unknown" ? copy.unknown : operation.status === "failed" ? copy.failed : copy.committed;
  return {
    title,
    message,
    canCheck: operation.status === "unknown" && !packageTargetMigration,
    canRetry: !cloud && !packageTargetMigration && (operation.status === "unknown" || (operation.status === "failed" && operation.action === "create-organization")),
    canAcknowledge: operation.status !== "unknown",
    acknowledgeLabel: copy.dismiss,
  };
}

export function OperationNotice({ copy }: { copy: AccountCopy }) {
  const { mutation, controller, snapshot } = useAccount();
  const operations = snapshot.operations;
  const busy = mutation?.status === "running" || Boolean(snapshot.operationsError);
  const local = mutation && !operations.some(operation => operation.operationId === mutation.operationId) ? mutation : null;
  if (!mutation && operations.length === 0 && !snapshot.operationsError) return null;
  return <Box sx={{ mb: 1.5 }}>
    {snapshot.operationsError ? <Alert severity="error" sx={{ mb: 1 }}>{copy.error(snapshot.operationsError)}<Button size="small" startIcon={<RefreshOutlinedIcon />} disabled={mutation?.status === "running"} onClick={() => void controller.reloadOperations()}>{copy.retry}</Button></Alert> : null}
    {local ? <Alert data-testid={`account-local-operation-${local.operationId}`} severity={local.status === "failed" ? "error" : "info"} sx={{ mb: 1 }}>
      {local.sourceAction === "cloud-commit" ? <Typography variant="body2" sx={{ fontWeight: 700 }}>{copy.cloudOperation}</Typography> : null}
      {local.status === "running" ? copy.loading : local.status === "unknown" ? local.sourceAction === "cloud-commit" ? copy.cloudUnknown : copy.unknown : copy.failed}
      {local.error && local.error !== "account_service_outcome_unknown" ? <Typography variant="body2">{copy.error(local.error)}</Typography> : null}
      {local.status === "unknown" ? <Box sx={{ display: "flex", gap: 1, flexWrap: "wrap", mt: 0.5 }}>
        <Button size="small" startIcon={<SearchOutlinedIcon />} disabled={busy} onClick={() => void controller.checkMutation(local.operationId)}>{copy.checkResult}</Button>
        {local.canRetry ? <Button size="small" startIcon={<RefreshOutlinedIcon />} disabled={busy} onClick={() => void controller.retryMutation(local.operationId)}>{copy.retry}</Button> : null}
      </Box> : local.status === "failed" ? <Button size="small" startIcon={<CloseOutlinedIcon />} onClick={controller.dismissFailedMutation}>{copy.dismiss}</Button> : null}
    </Alert> : null}
    {operations.map(operation => {
      const details = operationNoticeDetails(operation, copy);
      return <Alert key={operation.operationId} data-testid={`account-operation-${operation.operationId}`} severity={operation.status === "failed" ? "error" : operation.status === "committed" ? "success" : "info"} sx={{ mb: 1, "& .MuiAlert-message": { minWidth: 0 } }}>
      <Typography variant="body2" sx={{ fontWeight: 700 }}>{details.title}</Typography>
      <Typography variant="body2">{mutation?.operationId === operation.operationId && mutation.status === "running" ? copy.loading : details.message}</Typography>
      <Typography variant="caption" sx={{ display: "block", overflowWrap: "anywhere" }}>{operation.operationId}</Typography>
      <Box sx={{ display: "flex", gap: 1, flexWrap: "wrap", mt: 0.5 }}>
        {details.canCheck ? <Button size="small" startIcon={<SearchOutlinedIcon />} disabled={busy} onClick={() => void controller.checkMutation(operation.operationId)}>{copy.checkResult}</Button> : null}
        {details.canRetry ? <Button size="small" startIcon={<RefreshOutlinedIcon />} disabled={busy} onClick={() => void controller.retryMutation(operation.operationId)}>{copy.retry}</Button> : null}
        {details.canAcknowledge ? <Button size="small" startIcon={<CloseOutlinedIcon />} disabled={busy} onClick={() => void controller.acknowledgeOperation(operation.operationId)}>{details.acknowledgeLabel}</Button> : null}
      </Box>
    </Alert>})}
  </Box>;
}
