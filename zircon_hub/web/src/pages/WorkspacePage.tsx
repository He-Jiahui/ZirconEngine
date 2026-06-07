import AccountTreeOutlinedIcon from "@mui/icons-material/AccountTreeOutlined";
import FolderOutlinedIcon from "@mui/icons-material/FolderOutlined";
import InsightsOutlinedIcon from "@mui/icons-material/InsightsOutlined";
import SettingsOutlinedIcon from "@mui/icons-material/SettingsOutlined";
import TuneOutlinedIcon from "@mui/icons-material/TuneOutlined";
import { Box, Typography } from "@mui/material";
import { useMemo, useState } from "react";
import { HubList, HubPanel, HubTreeView, MetricCard, QuickActions, SourceEngineList } from "../components/data";
import { HubStatusBanner } from "../components/feedback";
import { HubButton, HubCheckbox, HubSwitch, HubTabs } from "../components/inputs";
import { formatCountText } from "../text/counts";
import { quickActionProjectTargetPayload } from "../tauri/projectTarget";
import { hubTokens } from "../theme/tokens";
import type { HubActionHandler, HubShellState } from "../types/hub";
import { HUB_ACTION } from "../types/hub";

export interface WorkspacePageProps {
  state: HubShellState;
  onAction: HubActionHandler;
}

export function WorkspacePage({ state, onAction }: WorkspacePageProps) {
  const [tab, setTab] = useState("overview");
  const common = state.ui.common;
  const labels = state.settings.text.labels;
  const settingsText = state.settings.text;
  const activePageLabel = state.ui.shell.navItems.find((item) => item.id === state.activePage)?.label ?? state.pageTitle;
  const quickActionProjectTarget = quickActionProjectTargetPayload(state.selectedProject);
  const settingsRows = useMemo(
    () => [
      { id: "project-dir", title: labels.defaultProjectDir, detail: state.settings.defaultProjectDir },
      { id: "source-dir", title: labels.defaultSourceDir, detail: state.settings.defaultSourceDir },
      { id: "build-output", title: labels.defaultBuildOutputDir, detail: state.settings.defaultBuildOutputDir },
      { id: "device-install", title: labels.defaultDeviceInstallDir, detail: state.settings.defaultDeviceInstallDir },
      { id: "build-profile", title: labels.buildProfile, detail: state.settings.buildProfileDetail },
    ],
    [
      labels.buildProfile,
      labels.defaultBuildOutputDir,
      labels.defaultDeviceInstallDir,
      labels.defaultProjectDir,
      labels.defaultSourceDir,
      state.settings.defaultBuildOutputDir,
      state.settings.defaultDeviceInstallDir,
      state.settings.defaultProjectDir,
      state.settings.defaultSourceDir,
      state.settings.buildProfileDetail,
    ],
  );
  const sourceTree = useMemo(
    () => [
      {
        id: "workspace",
        label: state.pageTitle,
        detail: activePageLabel,
        children: [
          {
            id: "source-engines",
            label: common.sourceEngines,
            detail: formatCountText(common.entryCountTemplate, state.sourceEngines.length),
            children: state.sourceEngines.map((engine) => ({
              id: engine.id,
              label: engine.name,
              detail: engine.status,
            })),
          },
          {
            id: "paths",
            label: settingsText.configurationPathsPanel,
            children: [
              { id: "source", label: state.ui.shell.source, detail: state.settings.defaultSourceDir },
              { id: "output", label: common.output, detail: state.settings.defaultBuildOutputDir },
              { id: "device", label: state.ui.cloud.deviceInstall, detail: state.settings.defaultDeviceInstallDir },
            ],
          },
        ],
      },
    ],
    [
      common.entryCountTemplate,
      common.output,
      common.sourceEngines,
      activePageLabel,
      settingsText.configurationPathsPanel,
      state.pageTitle,
      state.settings.defaultBuildOutputDir,
      state.settings.defaultDeviceInstallDir,
      state.settings.defaultSourceDir,
      state.sourceEngines,
      state.ui.cloud.deviceInstall,
      state.ui.shell.source,
    ],
  );

  return (
    <Box
      sx={{
        height: "100%",
        minHeight: 0,
        overflow: "auto",
        px: `${hubTokens.window.pagePaddingX}px`,
        py: `${hubTokens.window.pagePaddingY}px`,
        "@media (max-width: 980px)": {
          px: 2,
          py: 2,
        },
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
          <HubButton startIcon={<FolderOutlinedIcon />} onClick={() => void onAction(HUB_ACTION.showPage, "projects")}>
            {state.ui.projects.title}
          </HubButton>
          <HubButton tone="primary" startIcon={<SettingsOutlinedIcon />} onClick={() => void onAction(HUB_ACTION.showPage, "settings")}>
            {state.ui.shell.settings}
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
        <MetricCard label={state.ui.shell.workspaceProfile} value={state.pageTitle} detail={activePageLabel} icon={<InsightsOutlinedIcon />} tone="accent" />
        <MetricCard
          label={common.sourceEngines}
          value={`${state.sourceEngines.length}`}
          detail={state.engineVersion}
          icon={<AccountTreeOutlinedIcon />}
          tone="success"
        />
        <MetricCard label={labels.buildProfile} value={state.settings.buildProfileLabel} detail={state.settings.jobsLabel} icon={<TuneOutlinedIcon />} />
      </Box>

      <Box sx={{ mb: 1.4 }}>
        <HubTabs
          value={tab}
          onChange={setTab}
          options={[
            { value: "overview", label: common.overview },
            { value: "settings", label: state.ui.shell.settings },
            { value: "activity", label: common.activity },
          ]}
        />
      </Box>

      <Box
        sx={{
          display: "grid",
          gridTemplateColumns: "minmax(0, 1fr) minmax(330px, 0.58fr)",
          gap: 1.4,
          "@media (max-width: 1180px)": {
            gridTemplateColumns: "1fr",
          },
        }}
      >
        {tab === "overview" ? (
          <>
            <HubPanel title={common.sourceEngines}>
              <SourceEngineList engines={state.sourceEngines} emptyLabel={state.ui.shell.noSourceEngineRegistered} onSelect={(engine) => void onAction(HUB_ACTION.selectEngine, engine.id)} />
            </HubPanel>
            <HubPanel title={settingsText.heading}>
              <HubList items={settingsRows} />
            </HubPanel>
            <HubPanel title={common.quickActions}>
              <QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />
            </HubPanel>
          </>
        ) : null}

        {tab === "settings" ? (
          <>
            <HubPanel title={settingsText.advancedConfigurationPanel}>
              <Box sx={{ display: "grid", gap: 1 }}>
                <HubSwitch checked={state.settings.buildProfile === "release"} label={labels.releaseBuild} detail={state.settings.buildProfileLabel} disabled />
                <HubCheckbox checked={state.settings.language === "Chinese"} label={labels.localizedUi} detail={state.settings.languageLabel} disabled />
                <HubCheckbox checked={state.sourceEngines.length > 0} label={state.ui.editor.sourceEngineRegistered} detail={state.engineVersion} disabled />
              </Box>
            </HubPanel>
            <HubPanel title={settingsText.configurationPathsPanel}>
              <HubList items={settingsRows} />
            </HubPanel>
          </>
        ) : null}

        {tab === "activity" ? (
          <>
            <HubPanel title={state.ui.editor.workspaceTree}>
              <HubTreeView nodes={sourceTree} defaultExpanded={["workspace", "source-engines", "paths"]} />
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
