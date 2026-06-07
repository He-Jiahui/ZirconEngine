import BuildOutlinedIcon from "@mui/icons-material/BuildOutlined";
import DownloadForOfflineOutlinedIcon from "@mui/icons-material/DownloadForOfflineOutlined";
import FolderSpecialOutlinedIcon from "@mui/icons-material/FolderSpecialOutlined";
import Inventory2OutlinedIcon from "@mui/icons-material/Inventory2Outlined";
import PhoneIphoneOutlinedIcon from "@mui/icons-material/PhoneIphoneOutlined";
import { Box, LinearProgress, Typography } from "@mui/material";
import { useMemo, useState } from "react";
import { EmptyStateBlock, HubList, HubPanel, HubTreeView, MetricCard, QuickActions, SourceEngineList, StatusBadge } from "../components/data";
import { HubStatusBanner } from "../components/feedback";
import { HubButton, HubTabs } from "../components/inputs";
import { formatCountText } from "../text/counts";
import { quickActionProjectTargetPayload, workflowProjectPath, workflowProjectTargetPayload, workflowTargetProject } from "../tauri/projectTarget";
import { hubTokens } from "../theme/tokens";
import type { HubActionHandler, HubActionHistoryItem, HubActionId, HubShellState } from "../types/hub";
import { HUB_ACTION } from "../types/hub";

export interface BuildsPageProps {
  state: HubShellState;
  onAction: HubActionHandler;
}

const buildActionKinds: HubActionHistoryItem["kind"][] = ["build-editor-runtime", "package-project", "install-project"];
const workflowActionIds: HubActionId[] = [HUB_ACTION.buildProject, HUB_ACTION.packageProject, HUB_ACTION.installDevice];

export function BuildsPage({ state, onAction }: BuildsPageProps) {
  const [tab, setTab] = useState("workflow");
  const project = state.selectedProject;
  const workflowProjectTarget = workflowProjectTargetPayload(state);
  const workflowProject = workflowTargetProject(state);
  const quickActionProjectTarget = quickActionProjectTargetPayload(project);
  const common = state.ui.common;
  const text = state.ui.builds;
  const buildHistory = useMemo(
    () => state.actionHistory.filter((action) => buildActionKinds.includes(action.kind)),
    [state.actionHistory],
  );
  const latestAction = buildHistory[0];
  const workflowRows = [
    {
      id: HUB_ACTION.buildProject,
      title: text.buildProject,
      detail: state.settings.buildWorkflowDetail,
      meta: state.settings.defaultBuildOutputDir,
      icon: <BuildOutlinedIcon fontSize="small" />,
    },
    {
      id: HUB_ACTION.packageProject,
      title: text.packageProject,
      detail: text.packageDetail,
      meta: workflowProject?.name ?? common.noProjectSelected,
      icon: <Inventory2OutlinedIcon fontSize="small" />,
    },
    {
      id: HUB_ACTION.installDevice,
      title: text.installToDevice,
      detail: text.installDetail,
      meta: state.settings.defaultDeviceInstallDir,
      icon: <PhoneIphoneOutlinedIcon fontSize="small" />,
    },
  ];
  const buildTree = useMemo(
    () => [
      {
        id: "builds",
        label: state.pageTitle,
        detail: workflowProject?.name ?? common.noSelectedProject,
        children: [
          { id: "profile", label: text.profile, detail: state.settings.buildProfileLabel },
          { id: "jobs", label: text.jobs, detail: state.settings.jobsLabel },
          { id: "output", label: common.output, detail: state.settings.defaultBuildOutputDir },
          { id: "device", label: text.deviceInstall, detail: state.settings.defaultDeviceInstallDir },
          {
            id: "history",
            label: common.history,
            detail: formatCountText(common.actionCountTemplate, buildHistory.length),
            children: buildHistory.map((action) => ({
              id: action.id,
              label: action.action,
              detail: action.status,
            })),
          },
        ],
      },
    ],
    [buildHistory, common, state.pageTitle, state.settings, text, workflowProject],
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
          <HubButton startIcon={<BuildOutlinedIcon />} onClick={() => void onAction(HUB_ACTION.buildProject, undefined, workflowProjectTarget)}>
            {text.buildButton}
          </HubButton>
          <HubButton tone="primary" startIcon={<Inventory2OutlinedIcon />} onClick={() => void onAction(HUB_ACTION.packageProject, undefined, workflowProjectTarget)}>
            {text.packageButton}
          </HubButton>
          <HubButton startIcon={<PhoneIphoneOutlinedIcon />} onClick={() => void onAction(HUB_ACTION.installDevice, undefined, workflowProjectTarget)}>
            {text.installButton}
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
        <MetricCard label={text.buildProfile} value={state.settings.buildProfileLabel} detail={state.settings.jobsLabel} icon={<BuildOutlinedIcon />} tone="accent" />
        <MetricCard label={text.outputRoot} value={common.configured} detail={state.settings.defaultBuildOutputDir} icon={<FolderSpecialOutlinedIcon />} />
        <MetricCard
          label={text.recentWorkflows}
          value={`${buildHistory.length}`}
          detail={latestAction?.status ?? text.noBuildHistory}
          icon={<DownloadForOfflineOutlinedIcon />}
          tone={latestAction ? metricTone(latestAction.tone) : "neutral"}
        />
      </Box>

      <Box sx={{ mb: 1.4 }}>
        <HubTabs
          value={tab}
          onChange={setTab}
          options={[
            { value: "workflow", label: common.workflow },
            { value: "history", label: common.history },
            { value: "outputs", label: common.outputs },
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
        {tab === "workflow" ? (
          <>
            <HubPanel title={text.buildWorkflow}>
              <Box sx={{ display: "grid", gap: 1.2 }}>
                <LinearProgress
                  variant="determinate"
                  value={state.taskSummary.progressPercent}
                  sx={{
                    height: 7,
                    borderRadius: 999,
                    backgroundColor: "rgba(255,255,255,0.08)",
                    "& .MuiLinearProgress-bar": { backgroundColor: hubTokens.colors.accent },
                  }}
                />
                <HubList
                  items={workflowRows}
                  onSelect={(item) => {
                    const actionId = workflowActionIds.find((id) => id === item.id);
                    if (actionId) {
                      void onAction(actionId, undefined, workflowProjectTarget);
                    }
                  }}
                />
              </Box>
            </HubPanel>
            <HubPanel title={common.selectedProject}>
              {workflowProject ? (
                <HubList
                  items={[
                    { id: "project", title: workflowProject.name, detail: workflowProjectPath(workflowProject), meta: "status" in workflowProject ? workflowProject.status : workflowProject.modified },
                    { id: "engine", title: common.engine, detail: workflowProject.engineVersion, meta: "platform" in workflowProject ? workflowProject.platform : undefined },
                    { id: "output", title: common.output, detail: state.settings.defaultBuildOutputDir },
                  ]}
                />
              ) : (
                <EmptyStateBlock title={common.noProjectSelected} detail={text.noProjectSelectedDetail} />
              )}
            </HubPanel>
            <HubPanel title={common.sourceEngines}>
              <SourceEngineList engines={state.sourceEngines} emptyLabel={state.ui.shell.noSourceEngineRegistered} onSelect={(engine) => void onAction(HUB_ACTION.selectEngine, engine.id)} />
            </HubPanel>
          </>
        ) : null}

        {tab === "history" ? (
          <>
            <HubPanel title={text.buildHistory}>
              {buildHistory.length > 0 ? (
                <HubList
                  items={buildHistory.map(historyRow)}
                  onSelect={(item) => void onAction(HUB_ACTION.openOutputFolder, item.id, { historyId: item.id })}
                />
              ) : (
                <EmptyStateBlock title={text.noBuildHistory} detail={text.noBuildHistoryDetail} />
              )}
            </HubPanel>
            <HubPanel title={text.latestWorkflow}>
              {latestAction ? (
                <BuildActionDetail
                  action={latestAction}
                  text={text}
                  onOpenOutput={() => void onAction(HUB_ACTION.openOutputFolder, latestAction.id, { historyId: latestAction.id })}
                />
              ) : (
                <EmptyStateBlock title={text.noWorkflowSelected} detail={text.noWorkflowSelectedDetail} />
              )}
            </HubPanel>
          </>
        ) : null}

        {tab === "outputs" ? (
          <>
            <HubPanel title={text.outputTree}>
              <HubTreeView nodes={buildTree} defaultExpanded={["builds", "history"]} />
            </HubPanel>
            <HubPanel title={text.outputFolders}>
              <HubList
                items={[
                  {
                    id: "build-output-root",
                    title: text.outputRoot,
                    detail: state.settings.defaultBuildOutputDir,
                    icon: <FolderSpecialOutlinedIcon fontSize="small" />,
                  },
                  {
                    id: "device-install-root",
                    title: text.deviceInstall,
                    detail: state.settings.defaultDeviceInstallDir,
                    icon: <PhoneIphoneOutlinedIcon fontSize="small" />,
                  },
                ]}
                onSelect={(item) => {
                  const outputDir =
                    item.id === "build-output-root"
                      ? state.settings.defaultBuildOutputDir
                      : state.settings.defaultDeviceInstallDir;
                  void onAction(HUB_ACTION.openOutputFolder, undefined, { outputDir });
                }}
              />
            </HubPanel>
            <HubPanel title={common.quickActions}>
              <QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />
            </HubPanel>
          </>
        ) : null}
      </Box>
    </Box>
  );
}

function historyRow(action: HubActionHistoryItem) {
  return {
    id: action.id,
    title: action.action,
    detail: action.detail,
    meta: action.finished,
    icon: action.kind === "install-project" ? <PhoneIphoneOutlinedIcon fontSize="small" /> : <BuildOutlinedIcon fontSize="small" />,
    disabled: !action.outputDir,
  };
}

function metricTone(tone: HubActionHistoryItem["tone"]) {
  return tone === "running" ? "accent" : tone;
}

function BuildActionDetail({
  action,
  text,
  onOpenOutput,
}: {
  action: HubActionHistoryItem;
  text: HubShellState["ui"]["builds"];
  onOpenOutput: () => void;
}) {
  return (
    <Box sx={{ display: "grid", gap: 1.1 }}>
      <Box sx={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 1.2 }}>
        <Typography variant="body2" noWrap sx={{ fontWeight: 700, color: hubTokens.colors.text }}>
          {action.action}
        </Typography>
        <StatusBadge label={action.status} tone={action.tone} />
      </Box>
      <HubButton startIcon={<FolderSpecialOutlinedIcon />} disabled={!action.outputDir} onClick={onOpenOutput}>
        {text.openOutput}
      </HubButton>
      <HubList items={action.detailRows} />
    </Box>
  );
}
