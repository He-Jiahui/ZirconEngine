import CloudOutlinedIcon from "@mui/icons-material/CloudOutlined";
import FolderSpecialOutlinedIcon from "@mui/icons-material/FolderSpecialOutlined";
import Inventory2OutlinedIcon from "@mui/icons-material/Inventory2Outlined";
import LanOutlinedIcon from "@mui/icons-material/LanOutlined";
import PhoneIphoneOutlinedIcon from "@mui/icons-material/PhoneIphoneOutlined";
import StorageOutlinedIcon from "@mui/icons-material/StorageOutlined";
import { Box, Typography } from "@mui/material";
import { useMemo, useState } from "react";
import { EmptyStateBlock, HubList, HubPanel, HubTreeView, MetricCard, QuickActions, StatusBadge } from "../components/data";
import { HubStatusBanner } from "../components/feedback";
import { HubButton, HubCheckbox, HubSwitch, HubTabs } from "../components/inputs";
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
  const packageActions = useMemo(
    () => state.actionHistory.filter((action) => action.kind === "package-project"),
    [state.actionHistory],
  );
  const installActions = useMemo(
    () => state.actionHistory.filter((action) => action.kind === "install-project"),
    [state.actionHistory],
  );
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
      <Box sx={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 2, mb: 2.5 }}>
        <Box sx={{ minWidth: 0 }}>
          <Typography variant="h4">{state.pageTitle}</Typography>
          <Typography variant="body1" color="text.secondary" sx={{ mt: 0.9 }}>
            {state.pageSubtitle}
          </Typography>
        </Box>
        <Box sx={{ display: "flex", gap: 1.2, flexWrap: "wrap", justifyContent: "flex-end" }}>
          <HubButton startIcon={<Inventory2OutlinedIcon />} onClick={() => void onAction(HUB_ACTION.packageProject, undefined, workflowProjectTarget)}>
            {actionText.packageProject}
          </HubButton>
          <HubButton tone="primary" startIcon={<PhoneIphoneOutlinedIcon />} onClick={() => void onAction(HUB_ACTION.installDevice, undefined, workflowProjectTarget)}>
            {actionText.installToDevice}
          </HubButton>
        </Box>
      </Box>

      <Box sx={{ mb: 1.4 }}>
        <HubStatusBanner task={state.taskSummary} />
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
                  onSelect={(item) => void onAction(HUB_ACTION.openOutputFolder, item.id, { historyId: item.id })}
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
                    void onAction(HUB_ACTION.openOutputFolder, undefined, { outputDir: state.settings.defaultBuildOutputDir });
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
                  onSelect={(item) => void onAction(HUB_ACTION.openOutputFolder, item.id, { historyId: item.id })}
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
