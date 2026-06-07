import ArrowBackIcon from "@mui/icons-material/ArrowBack";
import EventOutlinedIcon from "@mui/icons-material/EventOutlined";
import FolderOpenOutlinedIcon from "@mui/icons-material/FolderOpenOutlined";
import Inventory2OutlinedIcon from "@mui/icons-material/Inventory2Outlined";
import OpenInNewIcon from "@mui/icons-material/OpenInNew";
import PushPinOutlinedIcon from "@mui/icons-material/PushPinOutlined";
import StorageOutlinedIcon from "@mui/icons-material/StorageOutlined";
import WarningAmberIcon from "@mui/icons-material/WarningAmber";
import { Box, Typography } from "@mui/material";
import { useMemo, useState } from "react";
import {
  EmptyStateBlock,
  HubList,
  HubPanel,
  HubTreeView,
  MetricCard,
  ProjectCover,
  QuickActions,
  SourceEngineList,
  StatusBadge,
} from "../components/data";
import { HubStatusBanner } from "../components/feedback";
import { HubButton, HubTabs } from "../components/inputs";
import { projectTargetPayload, quickActionProjectTargetPayload } from "../tauri/projectTarget";
import { hubTokens } from "../theme/tokens";
import type { HubActionHandler, HubShellState, StatusTone } from "../types/hub";
import { HUB_ACTION } from "../types/hub";

export interface ProjectDetailPageProps {
  state: HubShellState;
  onAction: HubActionHandler;
}

export function ProjectDetailPage({ state, onAction }: ProjectDetailPageProps) {
  const text = state.ui.projects;
  const actionText = state.ui.actions;
  const [tab, setTab] = useState("overview");
  const project = state.selectedProject ?? null;
  const projectTarget = projectTargetPayload(project);
  const quickActionProjectTarget = quickActionProjectTargetPayload(project);
  const statusTone: StatusTone = project?.exists ? "success" : "warning";
  const boundEngine = project?.engineId
    ? state.sourceEngines.find((engine) => engine.id === project.engineId)
    : state.sourceEngines.find((engine) => engine.active);

  const detailRows = useMemo(
    () =>
      project
        ? [
            { id: "path", title: text.location, detail: project.path, icon: <FolderOpenOutlinedIcon fontSize="small" /> },
            { id: "engine", title: text.sourceEngine, detail: boundEngine?.name ?? project.engineVersion, icon: <StorageOutlinedIcon fontSize="small" /> },
            { id: "template", title: text.template, detail: project.templateLabel, meta: project.pinned ? text.pinned : undefined },
            { id: "platform", title: text.platform, detail: project.platform },
            { id: "project-id", title: text.projectId, detail: project.id },
          ]
        : [],
    [boundEngine?.name, project, text],
  );

  const projectTree = useMemo(
    () =>
      project
        ? [
            {
              id: "project-root",
              label: project.name,
              detail: project.path,
              children: [
                { id: "source-engine", label: text.sourceEngine, detail: boundEngine?.name ?? project.engineVersion },
                { id: "content-root", label: text.content, detail: project.exists ? text.available : text.missing },
                { id: "build-output", label: text.buildOutput, detail: state.settings.defaultBuildOutputDir },
                { id: "device-output", label: text.deviceInstall, detail: state.settings.defaultDeviceInstallDir },
              ],
            },
          ]
        : [],
    [boundEngine?.name, project, state.settings.defaultBuildOutputDir, state.settings.defaultDeviceInstallDir, text],
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
      <Box sx={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 2, mb: 2.3 }}>
        <Box sx={{ minWidth: 0 }}>
          <Typography variant="h4">{project?.name ?? text.detailTitle}</Typography>
          <Typography variant="body1" color="text.secondary" sx={{ mt: 0.9 }}>
            {project?.path ?? state.pageSubtitle}
          </Typography>
        </Box>
        <Box sx={{ display: "flex", gap: 1.2, flexWrap: "wrap", justifyContent: "flex-end" }}>
          <HubButton startIcon={<ArrowBackIcon />} onClick={() => void onAction(HUB_ACTION.viewAllProjects)}>
            {actionText.browser}
          </HubButton>
          <HubButton tone="primary" startIcon={<OpenInNewIcon />} onClick={() => void onAction(HUB_ACTION.openEditor, undefined, projectTarget)}>
            {actionText.openEditor}
          </HubButton>
        </Box>
      </Box>

      <Box sx={{ mb: 1.4 }}>
        <HubStatusBanner task={state.taskSummary} />
      </Box>

      {!project ? (
        <EmptyStateBlock title={text.noProjectSelected} detail={text.chooseProjectFromBrowser} icon={<WarningAmberIcon />} />
      ) : (
        <>
          <Box
            sx={{
              display: "grid",
              gridTemplateColumns: "repeat(4, minmax(0, 1fr))",
              gap: 1.2,
              mb: 1.4,
              "@media (max-width: 1180px)": { gridTemplateColumns: "repeat(2, minmax(0, 1fr))" },
              "@media (max-width: 720px)": { gridTemplateColumns: "1fr" },
            }}
          >
            <MetricCard label={text.status} value={project.status} detail={project.exists ? text.ready : text.pathUnavailable} icon={<WarningAmberIcon />} tone={project.exists ? "success" : "warning"} />
            <MetricCard label={text.engine} value={project.engineVersion} detail={boundEngine?.status ?? text.projectBinding} icon={<StorageOutlinedIcon />} tone="accent" />
            <MetricCard label={text.lastModified} value={project.modified} detail={project.platform} icon={<EventOutlinedIcon />} />
            <MetricCard label={text.projectPin} value={project.pinned ? text.pinned : text.unpinned} detail={project.templateLabel} icon={<PushPinOutlinedIcon />} />
          </Box>

          <Box sx={{ mb: 1.4 }}>
            <HubTabs
              value={tab}
              onChange={setTab}
              options={[
                { value: "overview", label: text.overview },
                { value: "files", label: text.files },
                { value: "actions", label: text.actions },
              ]}
            />
          </Box>

          <Box
            sx={{
              display: "grid",
              gridTemplateColumns: "minmax(0, 1fr) minmax(330px, 0.4fr)",
              gap: 1.4,
              "@media (max-width: 1180px)": {
                gridTemplateColumns: "1fr",
              },
            }}
          >
            {tab === "overview" ? (
              <HubPanel title={text.projectOverview} action={<StatusBadge label={project.status} tone={statusTone} />}>
                <Box
                  sx={{
                    display: "grid",
                    gridTemplateColumns: "minmax(220px, 0.36fr) minmax(0, 1fr)",
                    gap: 1.4,
                    alignItems: "stretch",
                    "@media (max-width: 760px)": { gridTemplateColumns: "1fr" },
                  }}
                >
                  <Box sx={{ minHeight: 216, borderRadius: `${hubTokens.radius.card}px`, overflow: "hidden" }}>
                    <ProjectCover coverId={project.coverId} />
                  </Box>
                  <HubList items={detailRows} />
                </Box>
              </HubPanel>
            ) : null}

            {tab === "files" ? (
              <HubPanel title={text.projectTree}>
                <HubTreeView nodes={projectTree} defaultExpanded={["project-root"]} />
              </HubPanel>
            ) : null}

            {tab === "actions" ? (
              <HubPanel title={text.projectActions}>
                <QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />
              </HubPanel>
            ) : null}

            <Box sx={{ display: "grid", gap: 1.4, alignContent: "start" }}>
              <HubPanel title={text.quickActions}>
                <QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />
              </HubPanel>
              <HubPanel title={text.sourceEngines}>
                <SourceEngineList engines={state.sourceEngines} emptyLabel={state.ui.shell.noSourceEngineRegistered} onSelect={(engine) => void onAction(HUB_ACTION.selectEngine, engine.id)} />
              </HubPanel>
              <HubPanel title={text.package}>
                <Box sx={{ display: "grid", gap: 1 }}>
                  <HubButton startIcon={<Inventory2OutlinedIcon />} onClick={() => void onAction(HUB_ACTION.packageProject, undefined, projectTarget)}>
                    {actionText.packageProject}
                  </HubButton>
                  <HubButton onClick={() => void onAction(HUB_ACTION.installDevice, undefined, projectTarget)}>
                    {actionText.installToDevice}
                  </HubButton>
                </Box>
              </HubPanel>
              <HubPanel title={text.projectManagement}>
                <Box sx={{ display: "grid", gap: 1 }}>
                  {project.pendingDelete ? (
                    <>
                      <StatusBadge label={text.deleteRequested} tone="warning" />
                      <Typography variant="caption" color="text.secondary">
                        {text.deleteRequestedDetail}
                      </Typography>
                      <HubButton onClick={() => void onAction(HUB_ACTION.cancelDelete, undefined, projectTarget)}>{actionText.cancelDelete}</HubButton>
                      <HubButton tone="danger" onClick={() => void onAction(HUB_ACTION.confirmDelete, undefined, projectTarget)}>
                        {actionText.confirmDelete}
                      </HubButton>
                    </>
                  ) : (
                    <>
                      <HubButton onClick={() => void onAction(project.pinned ? HUB_ACTION.unpinProject : HUB_ACTION.pinProject, undefined, projectTarget)}>
                        {project.pinned ? actionText.unpinProject : actionText.pinProject}
                      </HubButton>
                      <HubButton onClick={() => void onAction(HUB_ACTION.removeFromHub, undefined, projectTarget)}>{actionText.removeFromHub}</HubButton>
                      <HubButton tone="danger" onClick={() => void onAction(HUB_ACTION.requestDelete, undefined, projectTarget)}>
                        {actionText.requestDelete}
                      </HubButton>
                    </>
                  )}
                </Box>
              </HubPanel>
            </Box>
          </Box>
        </>
      )}
    </Box>
  );
}
