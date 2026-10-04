import CloudOutlinedIcon from "@mui/icons-material/CloudOutlined";
import FolderSpecialOutlinedIcon from "@mui/icons-material/FolderSpecialOutlined";
import Inventory2OutlinedIcon from "@mui/icons-material/Inventory2Outlined";
import LanOutlinedIcon from "@mui/icons-material/LanOutlined";
import PhoneIphoneOutlinedIcon from "@mui/icons-material/PhoneIphoneOutlined";
import StorageOutlinedIcon from "@mui/icons-material/StorageOutlined";
import { Alert, Box, Button, Typography } from "@mui/material";
import { useEffect, useMemo, useState } from "react";
import { EmptyStateBlock, HubList, HubPanel, HubTreeView, MetricCard, QuickActions, StatusBadge } from "../components/data";
import { HubStatusBanner } from "../components/feedback";
import { HubButton, HubCheckbox, HubSwitch, HubTabs } from "../components/inputs";
import { collectDeliveryActions } from "../projections/deliveryActions";
import { accountCopy } from "../account/copy";
import { useAccount } from "../account/context";
import { formatCountText } from "../text/counts";
import { quickActionProjectTargetPayload, workflowProjectPath, workflowProjectTargetPayload, workflowTargetProject } from "../tauri/projectTarget";
import { hubTokens } from "../theme/tokens";
import type { HubActionHandler, HubShellState } from "../types/hub";
import { HUB_ACTION } from "../types/hub";

export interface CloudPageProps {
  state: HubShellState;
  onAction: HubActionHandler;
}

export function CloudPage({ state, onAction }: CloudPageProps) {
  const [tab, setTab] = useState("packages");
  const [discardConfirmationKey, setDiscardConfirmationKey] = useState<string | null>(null);
  const account = useAccount();
  const accountState = account.snapshot;
  const accountText = accountCopy(state.settings.language);
  const project = state.selectedProject;
  const workflowProjectTarget = workflowProjectTargetPayload(state);
  const workflowProject = workflowTargetProject(state);
  const quickActionProjectTarget = quickActionProjectTargetPayload(project);
  const common = state.ui.common;
  const text = state.ui.cloud;
  const actionText = state.ui.actions;
  const reservedServices = useMemo(
    () => state.comingSoon.filter((entry) => entry.category === "local-delivery"),
    [state.comingSoon],
  );
  const { packageActions, installActions } = useMemo(
    () => collectDeliveryActions(state.actionHistory),
    [state.actionHistory],
  );
  const selectedProjectPath = workflowProject ? workflowProjectPath(workflowProject) : null;
  const binding = account.cloudBinding;
  const cloudSyncScopeMatchesBinding = !account.cloudSyncScope || Boolean(binding
    && account.cloudSyncScope.organizationId === binding.organizationId
    && account.cloudSyncScope.projectId === binding.projectId
    && account.cloudSyncScope.localProjectGuid === binding.localProjectGuid);
  const activeCloudSync = account.cloudSync?.status === "uploading" || account.cloudSync?.status === "downloading"
    || account.cloudSync?.status === "applying" || account.cloudSync?.status === "discarding";
  const visibleCloudSync = cloudSyncScopeMatchesBinding || activeCloudSync ? account.cloudSync : null;
  const cloudSyncContextChanged = Boolean(account.cloudSync && !cloudSyncScopeMatchesBinding && !activeCloudSync);
  const bindingMatches = Boolean(binding && binding.organizationId === account.selectedOrganization
    && binding.projectId && account.projects.loaded && account.projects.items.some(item => item.id === binding.projectId));
  const cloudHead = account.cloudHead;
  const cloudOperation = accountState.operations.find(operation => operation.action === "cloud-commit"
    && (operation.status === "unknown" || operation.status === "failed"));
  const unknownCloudSync = account.cloudSync?.status === "unknown" ? account.cloudSync : null;
  const duplicateCloudOperation = Boolean(cloudOperation && (
    (visibleCloudSync && "operationId" in visibleCloudSync && visibleCloudSync.operationId === cloudOperation.operationId)
      || unknownCloudSync?.operationId === cloudOperation.operationId
  ));
  const cloudBusy = account.mutation?.status === "running"
    || account.cloudSync?.status === "uploading"
    || account.cloudSync?.status === "downloading"
    || account.cloudSync?.status === "applying"
    || account.cloudSync?.status === "discarding"
    || Boolean(accountState.operationsError);
  const canOperateCloud = accountState.account.status === "signed-in" && bindingMatches && Boolean(cloudHead?.loaded)
    && !cloudHead?.loading && !cloudHead?.error && !cloudBusy && !cloudOperation;
  const cloudStagePending = visibleCloudSync?.status === "staged" || visibleCloudSync?.status === "conflict"
    || visibleCloudSync?.status === "download-failed" || visibleCloudSync?.status === "apply-failed"
    || visibleCloudSync?.status === "discard-failed";
  const stageForApply = visibleCloudSync?.status === "staged"
    ? { stageId: visibleCloudSync.stage.stageId, revision: visibleCloudSync.stage.revision }
    : visibleCloudSync?.status === "conflict"
      ? { stageId: visibleCloudSync.result.stageId, revision: visibleCloudSync.result.revision }
      : visibleCloudSync?.status === "apply-failed"
        ? { stageId: visibleCloudSync.stageId, revision: visibleCloudSync.revision }
        : null;
  const failedCloudSync = visibleCloudSync?.status === "failed" ? visibleCloudSync : null;
  const retryStage = visibleCloudSync?.status === "download-failed" || visibleCloudSync?.status === "discard-failed" ? visibleCloudSync : null;
  const discardStageRevision = visibleCloudSync?.status === "staged" ? visibleCloudSync.stage.revision
    : visibleCloudSync?.status === "conflict" ? visibleCloudSync.result.revision
      : visibleCloudSync?.status === "download-failed" || visibleCloudSync?.status === "apply-failed" || visibleCloudSync?.status === "discard-failed"
        ? visibleCloudSync.revision : null;
  const cloudRevision = cloudHead?.snapshot?.revision;
  const cloudSyncFailureError = failedCloudSync?.error
    ?? (visibleCloudSync?.status === "download-failed" || visibleCloudSync?.status === "apply-failed" || visibleCloudSync?.status === "discard-failed" ? visibleCloudSync.error : null);
  const canResumeCloudOperation = !account.cloudSyncScope || (cloudSyncScopeMatchesBinding
    && (!cloudOperation || unknownCloudSync?.operationId === cloudOperation.operationId));
  const cloudStageStale = Boolean(cloudHead?.loaded && !cloudHead.loading && (
    (stageForApply !== null && stageForApply.revision !== cloudRevision)
      || (retryStage !== null && retryStage.revision !== cloudRevision)
  ));
  const canStartCloudUpload = canOperateCloud && !cloudStagePending;
  const canStartCloudDownload = canOperateCloud && (!cloudStagePending || cloudStageStale);
  const canApplyStage = canOperateCloud && !cloudStageStale;
  const canDiscardStage = canOperateCloud && discardStageRevision !== null;
  const discardStageKey = discardStageRevision && binding
    ? `${binding.localProjectGuid}:${binding.organizationId}:${binding.projectId}:${discardStageRevision}` : null;

  useEffect(() => {
    if (accountState.account.status === "signed-in" && !account.cloudBindingLoading && !account.cloudBinding && !account.cloudBindingError) {
      void account.controller.loadCloudBinding();
    }
  }, [account.controller, accountState.account.generation, accountState.account.status, selectedProjectPath,
    account.cloudBinding, account.cloudBindingError, account.cloudBindingLoading]);

  useEffect(() => {
    if (binding && account.organizations.loaded && account.organizations.items.some(item => item.id === binding.organizationId)
      && account.selectedOrganization !== binding.organizationId) {
      void account.controller.selectOrganization(binding.organizationId);
    }
  }, [account.controller, account.organizations.items, account.organizations.loaded, account.selectedOrganization, binding]);

  useEffect(() => {
    if (bindingMatches && binding && account.selectedOrganization === binding.organizationId && account.projects.loaded
      && (!cloudHead || cloudHead.organization !== binding.organizationId || cloudHead.project !== binding.projectId)) {
      void account.controller.loadCloudHead(binding.projectId);
    }
  }, [account.controller, account.projects.loaded, account.selectedOrganization, binding, bindingMatches, cloudHead]);
  const outputTree = useMemo(
    () => [
      {
        id: "cloud",
        label: text.localDeliveryTree,
        detail: text.localDeliveryTreeDetail,
        children: [
          { id: "package-root", label: text.packageOutput, detail: state.settings.defaultBuildOutputDir },
          { id: "device-root", label: text.deviceInstall, detail: state.settings.defaultDeviceInstallDir },
          {
            id: "services",
            label: text.serviceSlots,
            detail: formatCountText(common.reservedCountTemplate, reservedServices.length),
            children: reservedServices.map((entry) => ({
              id: entry.id,
              label: entry.title,
              detail: entry.meta,
            })),
          },
        ],
      },
    ],
    [common.reservedCountTemplate, reservedServices, state.settings.defaultBuildOutputDir, state.settings.defaultDeviceInstallDir, text],
  );

  return (
    <Box
      sx={{
        height: "100%",
        minHeight: 0,
        overflow: "auto",
        px: `${hubTokens.window.pagePaddingX}px`,
        py: `${hubTokens.window.pagePaddingY}px`,
        "@media (max-width: 980px)": { px: 2, py: 2 },
      }}
    >
      <PageHeader title={state.pageTitle} subtitle={state.pageSubtitle} actions={<>
          <HubButton disabled={!workflowProjectTarget} startIcon={<Inventory2OutlinedIcon />} onClick={() => void onAction(HUB_ACTION.packageProject, undefined, workflowProjectTarget)}>
            {actionText.packageProject}
          </HubButton>
          <HubButton disabled={!workflowProjectTarget} tone="primary" startIcon={<PhoneIphoneOutlinedIcon />} onClick={() => void onAction(HUB_ACTION.installDevice, undefined, workflowProjectTarget)}>
            {actionText.installToDevice}
          </HubButton>
        </>} />

      <HubPanel title={accountText.cloudSyncTitle}>
        {accountState.account.status === "unavailable" ? (
          <Alert severity="info">{accountText.unavailable}</Alert>
        ) : accountState.account.status !== "signed-in" ? (
          <Box sx={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: 1 }}>
            <Typography variant="body2" color="text.secondary" sx={{ flex: "1 1 220px" }}>{accountText.signedOut}</Typography>
            <Button variant="contained" disabled={!accountState.account.configured || Boolean(account.pending)} onClick={() => void account.controller.authenticate("sign-in")}>{accountText.signIn}</Button>
          </Box>
        ) : account.cloudBindingLoading ? (
          <Typography variant="body2" color="text.secondary">{accountText.loading}</Typography>
        ) : account.cloudBindingError ? (
          <Alert severity="error" action={<Button color="inherit" size="small" onClick={() => void account.controller.loadCloudBinding()}>{accountText.retry}</Button>}>
            {accountText.error(account.cloudBindingError)}
          </Alert>
        ) : !binding ? (
          <Typography variant="body2" color="text.secondary">{accountText.cloudLinkFromTeam}</Typography>
        ) : (
          <Box sx={{ display: "flex", flexDirection: "column", gap: 1.1, minWidth: 0 }}>
            <Box sx={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: 1 }}>
              <Typography variant="body2" sx={{ flex: "1 1 220px", overflowWrap: "anywhere" }}>
                {account.organizations.items.find(item => item.id === binding.organizationId)?.name ?? binding.organizationId}
                {" / "}
                {account.projects.items.find(item => item.id === binding.projectId)?.name ?? binding.projectId}
              </Typography>
              <Typography variant="caption" color="text.secondary">
                {accountText.cloudRevision}: {cloudHead?.snapshot?.revision ?? "0"}
              </Typography>
            </Box>
            {!bindingMatches ? <Alert severity="info">{accountText.cloudSyncUnavailable}</Alert> : null}
            {cloudHead?.error ? <Alert severity="error" action={<Button size="small" color="inherit" onClick={() => void account.controller.loadCloudHead(binding.projectId)}>{accountText.retry}</Button>}>{accountText.error(cloudHead.error)}</Alert> : null}
            {account.cloudBinding && cloudHead?.loaded && !cloudHead.snapshot ? <Typography variant="body2" color="text.secondary">{accountText.cloudNoRemoteSnapshot}</Typography> : null}
            {cloudSyncContextChanged ? <Alert severity="warning">{accountText.cloudSyncProjectChanged}</Alert> : null}
            {visibleCloudSync?.status === "uploading" ? <Alert severity="info">{accountText.cloudUploading}</Alert> : null}
            {visibleCloudSync?.status === "downloading" ? <Alert severity="info">{accountText.cloudDownloading}</Alert> : null}
            {visibleCloudSync?.status === "applying" ? <Alert severity="info">{accountText.cloudApplying}</Alert> : null}
            {visibleCloudSync?.status === "discarding" ? <Alert severity="info">{accountText.cloudDiscardStage}…</Alert> : null}
            {visibleCloudSync?.status === "staged" ? <Alert severity="info">{accountText.cloudStageReady}</Alert> : null}
            {cloudStageStale ? <Alert severity="warning">{accountText.cloudStageStale}</Alert> : null}
            {visibleCloudSync?.status === "applied" ? <Alert severity="success">{accountText.cloudApplyComplete}</Alert> : null}
            {visibleCloudSync?.status === "discarded" ? <Alert severity="success">{accountText.cloudStageDiscarded}</Alert> : null}
            {visibleCloudSync?.status === "conflict" ? (
              <Alert severity="warning">
                {accountText.cloudLocalConflicts(visibleCloudSync.result.conflictCount)}
                {visibleCloudSync.result.paths.length ? <Box component="ul" sx={{ my: 0.5, pl: 2.5, overflowWrap: "anywhere" }}>{visibleCloudSync.result.paths.map(path => <li key={path}>{path}</li>)}</Box> : null}
              </Alert>
            ) : null}
            {discardStageRevision ? (
              <Alert
                severity="info"
                action={discardConfirmationKey === discardStageKey ? (
                  <Box sx={{ display: "flex", flexWrap: "wrap", gap: 0.5 }}>
                    <Button size="small" color="inherit" disabled={!canDiscardStage} onClick={async () => {
                      if (await account.controller.discardCloudDownload(discardStageRevision)) setDiscardConfirmationKey(null);
                    }}>{accountText.cloudConfirmDiscard}</Button>
                    <Button size="small" color="inherit" onClick={() => setDiscardConfirmationKey(null)}>{accountText.cloudCancelDiscard}</Button>
                  </Box>
                ) : <Button size="small" color="inherit" disabled={!canDiscardStage} onClick={() => setDiscardConfirmationKey(discardStageKey)}>{accountText.cloudDiscardStage}</Button>}
              >
                {accountText.cloudDiscardHelp}
              </Alert>
            ) : null}
            {cloudSyncFailureError ? (
              <Alert severity="error" action={failedCloudSync && cloudOperation?.operationId === failedCloudSync.operationId
                ? <Button color="inherit" size="small" onClick={() => void account.controller.acknowledgeOperation(failedCloudSync.operationId)}>{accountText.dismiss}</Button>
                : undefined}>
                {failedCloudSync?.error === "account_cloud_conflict" ? accountText.cloudConflict
                  : failedCloudSync?.error === "account_operation_id_conflict" ? accountText.cloudIdConflict
                    : accountText.error(cloudSyncFailureError)}
              </Alert>
            ) : null}
            {visibleCloudSync?.status === "committed" ? (
              <Alert severity="success">{accountText.cloudCommitted}{visibleCloudSync.revision ? ` ${accountText.cloudRevision}: ${visibleCloudSync.revision}` : ""}</Alert>
            ) : null}
            {unknownCloudSync ? (
              <Alert severity="warning">
                <Typography variant="body2">{accountText.cloudUnknown}</Typography>
                <Typography variant="caption" sx={{ overflowWrap: "anywhere" }}>{accountText.cloudOperationId}: {unknownCloudSync.operationId}</Typography>
                <Box sx={{ display: "flex", flexWrap: "wrap", gap: 1, mt: 0.5 }}>
                  <Button size="small" disabled={cloudBusy} onClick={() => void account.controller.checkMutation(unknownCloudSync.operationId)}>{accountText.checkResult}</Button>
                  <Button size="small" disabled={cloudBusy || !bindingMatches || !canResumeCloudOperation} onClick={() => void account.controller.retryCloudSnapshot(unknownCloudSync.operationId)}>{accountText.cloudRetrySnapshot}</Button>
                </Box>
              </Alert>
            ) : null}
            {cloudOperation && !duplicateCloudOperation ? (
              <Alert severity={cloudOperation.status === "unknown" ? "warning" : "error"}>
                <Typography variant="body2">{cloudOperation.status === "unknown" ? accountText.cloudUnknown : cloudOperation.error === "account_cloud_conflict" ? accountText.cloudConflict : accountText.cloudIdConflict}</Typography>
                <Typography variant="caption" sx={{ overflowWrap: "anywhere" }}>{accountText.cloudOperationId}: {cloudOperation.operationId}</Typography>
                <Box sx={{ display: "flex", flexWrap: "wrap", gap: 1, mt: 0.5 }}>
                  {cloudOperation.status === "unknown" ? <Button size="small" disabled={cloudBusy} onClick={() => void account.controller.checkMutation(cloudOperation.operationId)}>{accountText.checkResult}</Button> : null}
                  {cloudOperation.status === "unknown" ? <Button size="small" disabled={cloudBusy || !bindingMatches || !canResumeCloudOperation} onClick={() => void account.controller.retryCloudSnapshot(cloudOperation.operationId)}>{accountText.cloudRetrySnapshot}</Button> : null}
                  {cloudOperation.status !== "unknown" ? <Button size="small" disabled={cloudBusy} onClick={() => void account.controller.acknowledgeOperation(cloudOperation.operationId)}>{accountText.dismiss}</Button> : null}
                </Box>
              </Alert>
            ) : null}
            <Box sx={{ display: "flex", flexWrap: "wrap", gap: 1 }}>
              <HubButton tone="primary" disabled={!canStartCloudUpload} onClick={() => void account.controller.pushCloudSnapshot()}>{accountText.cloudPush}</HubButton>
              {cloudRevision ? <HubButton disabled={!canStartCloudDownload} onClick={() => void account.controller.stageCloudDownload(cloudRevision)}>{accountText.cloudDownload}</HubButton> : null}
              {retryStage ? <HubButton disabled={!canOperateCloud || cloudStageStale} onClick={() => void account.controller.stageCloudDownload(retryStage.revision)}>{accountText.cloudRetryStage}</HubButton> : null}
              {stageForApply ? <HubButton disabled={!canApplyStage} onClick={() => void account.controller.applyCloudDownload(stageForApply.stageId, stageForApply.revision)}>{visibleCloudSync?.status === "apply-failed" ? accountText.cloudRetryApply : accountText.cloudApply}</HubButton> : null}
              {bindingMatches && !account.cloudBindingLoading ? <HubButton disabled={cloudBusy || Boolean(cloudOperation)} onClick={() => void account.controller.loadCloudHead(binding.projectId)}>{accountText.cloudRefreshHead}</HubButton> : null}
            </Box>
          </Box>
        )}
      </HubPanel>

      <Box sx={{ mb: 1.4 }}>
        <HubStatusBanner
          task={state.taskSummary}
          cancelLabel={state.ui.common.cancelTask}
          onCancel={() => void onAction(HUB_ACTION.cancelBackgroundTask, String(state.taskSummary.taskId))}
        />
      </Box>

      <Box
        sx={{
          display: "grid",
          gridTemplateColumns: "repeat(3, minmax(0, 1fr))",
          gap: 1.2,
          mb: 1.4,
          "@media (max-width: 980px)": { gridTemplateColumns: "1fr" },
        }}
      >
        <MetricCard label={text.packageRoot} value={common.local} detail={state.settings.defaultBuildOutputDir} icon={<FolderSpecialOutlinedIcon />} tone="accent" />
        <MetricCard label={text.deviceInstall} value={common.configured} detail={state.settings.defaultDeviceInstallDir} icon={<PhoneIphoneOutlinedIcon />} tone="success" />
        <MetricCard label={text.serviceSlots} value={`${reservedServices.length}`} detail={text.reservedLocalServices} icon={<CloudOutlinedIcon />} />
      </Box>

      <Box sx={{ mb: 1.4 }}>
        <HubTabs
          value={tab}
          onChange={setTab}
          options={[
            { value: "packages", label: common.packages },
            { value: "installs", label: common.installs },
            { value: "services", label: common.services },
          ]}
        />
      </Box>

      <Box
        sx={{
          display: "grid",
          gridTemplateColumns: "minmax(0, 1fr) minmax(330px, 0.55fr)",
          gap: 1.4,
          "@media (max-width: 1180px)": { gridTemplateColumns: "1fr" },
        }}
      >
        {tab === "packages" ? (
          <>
            <HubPanel title={text.packageOutputs}>
              {packageActions.length > 0 ? (
                <HubList
                  items={packageActions.map((action) => ({
                    id: action.id,
                    title: action.target,
                    detail: action.detail,
                    secondaryDetail: action.outputDir ?? common.noOutputDirectory,
                    meta: action.finished,
                    icon: <Inventory2OutlinedIcon fontSize="small" />,
                    disabled: !action.outputDir,
                  }))}
                  onSelect={(item) => void onAction(HUB_ACTION.openOutputFolder, undefined, { receiptId: item.id })}
                />
              ) : (
                <EmptyStateBlock title={text.noPackagesRecorded} detail={text.noPackagesRecordedDetail} />
              )}
            </HubPanel>
            <HubPanel title={text.packageTarget}>
              <HubList
                items={[
                  { id: "project", title: common.project, detail: workflowProject?.name ?? common.noProjectSelected },
                  { id: "project-path", title: common.path, detail: workflowProject ? workflowProjectPath(workflowProject) : common.noProjectSelected },
                  { id: "output", title: state.ui.builds.outputRoot, detail: state.settings.defaultBuildOutputDir },
                  { id: "profile", title: state.ui.builds.buildProfile, detail: state.settings.buildProfileDetail },
                ]}
                onSelect={(item) => {
                  if (item.id === "output") {
                    void onAction(HUB_ACTION.openOutputFolder, undefined, { capability: "default-build-output" });
                  }
                }}
              />
            </HubPanel>
            <HubPanel title={common.quickActions}>
              <QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />
            </HubPanel>
          </>
        ) : null}

        {tab === "installs" ? (
          <>
            <HubPanel title={text.deviceInstalls}>
              {installActions.length > 0 ? (
                <HubList
                  items={installActions.map((action) => ({
                    id: action.id,
                    title: action.target,
                    detail: action.detail,
                    secondaryDetail: action.outputDir ?? common.noOutputDirectory,
                    meta: action.finished,
                    icon: <PhoneIphoneOutlinedIcon fontSize="small" />,
                    disabled: !action.outputDir,
                  }))}
                  onSelect={(item) => void onAction(HUB_ACTION.openOutputFolder, undefined, { receiptId: item.id })}
                />
              ) : (
                <EmptyStateBlock title={text.noInstallsRecorded} detail={text.noInstallsRecordedDetail} />
              )}
            </HubPanel>
            <HubPanel title={text.installReadiness}>
              <Box sx={{ display: "grid", gap: 1 }}>
                <HubSwitch checked={Boolean(workflowProject && (!("exists" in workflowProject) || workflowProject.exists))} label={state.ui.editor.projectAvailable} detail={workflowProject ? workflowProjectPath(workflowProject) : common.noProjectSelected} disabled />
                <HubCheckbox checked={state.settings.defaultDeviceInstallDir !== common.notConfigured} label={text.deviceInstallFolder} detail={state.settings.defaultDeviceInstallDir} disabled />
                <HubCheckbox checked={packageActions.length > 0} label={text.packageHistory} detail={formatCountText(text.packageActionCountTemplate, packageActions.length)} disabled />
              </Box>
            </HubPanel>
          </>
        ) : null}

        {tab === "services" ? (
          <>
            <HubPanel title={text.reservedServices}>
              <HubList
                items={reservedServices.map((entry) => ({
                  id: entry.id,
                  title: entry.title,
                  detail: entry.detail,
                  meta: entry.meta,
                  icon: <LanOutlinedIcon fontSize="small" />,
                  disabled: entry.disabled,
                }))}
              />
            </HubPanel>
            <HubPanel title={text.localDeliveryTree}>
              <HubTreeView nodes={outputTree} defaultExpanded={["cloud", "services"]} />
            </HubPanel>
            <HubPanel title={text.currentStatus}>
              <Box sx={{ display: "grid", gap: 1.1 }}>
                <Box sx={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 1.2 }}>
                  <Typography variant="body2" noWrap sx={{ color: hubTokens.colors.text, fontWeight: 700 }}>
                    {text.localPackageHandoff}
                  </Typography>
                  <StatusBadge label={state.taskSummary.label} tone={state.taskSummary.tone} />
                </Box>
                <HubList
                  items={[
                    { id: "operation", title: common.operation, detail: state.taskSummary.operation, icon: <StorageOutlinedIcon fontSize="small" /> },
                    { id: "detail", title: common.detail, detail: state.taskSummary.detail },
                  ]}
                />
              </Box>
            </HubPanel>
          </>
        ) : null}
      </Box>
    </Box>
  );
}
import { PageHeader } from "../components/data/PageHeader";
