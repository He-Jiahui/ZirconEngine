import AccountTreeOutlinedIcon from "@mui/icons-material/AccountTreeOutlined";
import ExtensionOutlinedIcon from "@mui/icons-material/ExtensionOutlined";
import FolderSpecialOutlinedIcon from "@mui/icons-material/FolderSpecialOutlined";
import OpenInNewOutlinedIcon from "@mui/icons-material/OpenInNewOutlined";
import TerminalOutlinedIcon from "@mui/icons-material/TerminalOutlined";
import WebAssetOutlinedIcon from "@mui/icons-material/WebAssetOutlined";
import { Box, Typography } from "@mui/material";
import { useMemo, useState } from "react";
import { EmptyStateBlock, HubList, HubPanel, HubTreeView, MetricCard, QuickActions, SourceEngineList, StatusBadge } from "../components/data";
import { HubStatusBanner } from "../components/feedback";
import { HubButton, HubCheckbox, HubSwitch, HubTabs } from "../components/inputs";
import { formatCountText } from "../text/counts";
import { projectTargetPayload, quickActionProjectTargetPayload } from "../tauri/projectTarget";
import { hubTokens } from "../theme/tokens";
import type { HubActionHandler, HubShellState } from "../types/hub";
import { HUB_ACTION } from "../types/hub";

export interface EditorPageProps {
  state: HubShellState;
  onAction: HubActionHandler;
}

export function EditorPage({ state, onAction }: EditorPageProps) {
  const [tab, setTab] = useState("overview");
  const project = state.selectedProject;
  const projectTarget = projectTargetPayload(project);
  const quickActionProjectTarget = quickActionProjectTargetPayload(project);
  const common = state.ui.common;
  const text = state.ui.editor;
  const actionText = state.ui.actions;
  const activeSourceEngine = state.sourceEngines.find((engine) => engine.active) ?? state.sourceEngines[0];
  const sourceBuildHistory = useMemo(
    () => activeSourceEngine?.buildHistory ?? [],
    [activeSourceEngine],
  );
  const pluginComingSoonRows = useMemo(
    () => state.comingSoon.filter((entry) => entry.category === "plugins"),
    [state.comingSoon],
  );
  const editorPlugins = useMemo(
    () => state.plugins.filter((plugin) => plugin.editorScoped),
    [state.plugins],
  );
  const editorActivity = useMemo(
    () =>
      state.actionHistory.filter(
        (action) => action.kind === "open-editor" || action.kind === "build-editor-runtime",
      ),
    [state.actionHistory],
  );
  const editorTree = useMemo(
    () => [
      {
        id: "editor-workspace",
        label: text.editorWorkspace,
        detail: project?.name ?? common.noSelectedProject,
        children: [
          {
            id: "project",
            label: text.selectedProject,
            detail: project?.path ?? common.noProjectSelected,
          },
          {
            id: "source-engines",
            label: text.sourceEngines,
            detail: formatCountText(common.entryCountTemplate, state.sourceEngines.length),
            children: state.sourceEngines.map((engine) => ({
              id: engine.id,
              label: engine.name,
              detail: engine.status,
            })),
          },
          {
            id: "editor-plugins",
            label: text.editorPlugins,
            detail: formatCountText(common.availableCountTemplate, editorPlugins.length),
            children: editorPlugins.map((plugin) => ({
              id: plugin.id,
              label: plugin.displayName,
              detail: plugin.category,
            })),
          },
        ],
      },
    ],
    [common, editorPlugins, project, state.sourceEngines, text],
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
          <HubButton startIcon={<AccountTreeOutlinedIcon />} onClick={() => void onAction(HUB_ACTION.showPage, "projects")}>
            {state.ui.projects.title}
          </HubButton>
          <HubButton
            startIcon={<FolderSpecialOutlinedIcon />}
            disabled={!activeSourceEngine || activeSourceEngine.outputPath === common.notConfigured}
            onClick={() => void onAction(HUB_ACTION.openOutputFolder, undefined, { outputDir: activeSourceEngine?.outputPath })}
          >
            {state.ui.builds.openOutput}
          </HubButton>
          <HubButton tone="primary" startIcon={<OpenInNewOutlinedIcon />} onClick={() => void onAction(HUB_ACTION.openEditor, undefined, projectTarget)}>
            {actionText.openEditor}
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
        <MetricCard
          label={common.selectedProject}
          value={project?.name ?? common.none}
          detail={project?.status ?? text.chooseProject}
          icon={<WebAssetOutlinedIcon />}
          tone={project?.exists ? "success" : "warning"}
        />
        <MetricCard label={common.sourceEngines} value={`${state.sourceEngines.length}`} detail={state.engineVersion} icon={<AccountTreeOutlinedIcon />} tone="accent" />
        <MetricCard label={text.editorPlugins} value={`${editorPlugins.length}`} detail={text.editorPackagingScope} icon={<ExtensionOutlinedIcon />} />
      </Box>

      <Box sx={{ mb: 1.4 }}>
        <HubTabs
          value={tab}
          onChange={setTab}
          options={[
            { value: "overview", label: common.overview },
            { value: "plugins", label: common.plugins },
            { value: "activity", label: common.activity },
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
        {tab === "overview" ? (
          <>
            <HubPanel title={text.launchTarget}>
              {project ? (
                <Box sx={{ display: "grid", gap: 1.1 }}>
                  <Box sx={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 1.2 }}>
                    <Typography variant="body2" noWrap sx={{ fontWeight: 700, color: hubTokens.colors.text }}>
                      {project.name}
                    </Typography>
                    <StatusBadge label={project.status} tone={project.exists ? "success" : "warning"} />
                  </Box>
                  <HubList
                    items={[
                      { id: "path", title: common.path, detail: project.path },
                      { id: "engine", title: common.engine, detail: project.engineVersion },
                      { id: "template", title: common.template, detail: project.templateLabel },
                    ]}
                  />
                </Box>
              ) : (
                <EmptyStateBlock title={text.noProjectSelectedTitle} detail={text.noProjectSelectedDetail} icon={<WebAssetOutlinedIcon />} />
              )}
            </HubPanel>
            <HubPanel title={common.sourceEngines}>
              <SourceEngineList engines={state.sourceEngines} emptyLabel={state.ui.shell.noSourceEngineRegistered} onSelect={(engine) => void onAction(HUB_ACTION.selectEngine, engine.id)} />
            </HubPanel>
            <HubPanel title={common.quickActions}>
              <QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />
            </HubPanel>
          </>
        ) : null}

        {tab === "plugins" ? (
          <>
            <HubPanel title={text.editorPluginScope}>
              {editorPlugins.length > 0 ? (
                <HubList
                  items={editorPlugins.map((plugin) => ({
                    id: plugin.id,
                    title: plugin.displayName,
                    detail: plugin.description,
                    meta: plugin.maturity,
                    icon: <ExtensionOutlinedIcon fontSize="small" />,
                  }))}
                />
              ) : (
                <EmptyStateBlock title={text.noEditorPluginsTitle} detail={text.noEditorPluginsDetail} />
              )}
            </HubPanel>
            <HubPanel title={text.workspaceTree}>
              <HubTreeView nodes={editorTree} defaultExpanded={["editor-workspace", "source-engines", "editor-plugins"]} />
            </HubPanel>
            {pluginComingSoonRows.length > 0 ? (
              <HubPanel title={text.pluginComingSoonPanel}>
                <HubList
                  items={pluginComingSoonRows.map((entry) => ({
                    id: entry.id,
                    title: entry.title,
                    detail: entry.detail,
                    meta: entry.meta,
                    disabled: entry.disabled,
                    icon: <ExtensionOutlinedIcon fontSize="small" />,
                  }))}
                />
              </HubPanel>
            ) : null}
          </>
        ) : null}

        {tab === "activity" ? (
          <>
            <HubPanel title={text.editorActivity}>
              {editorActivity.length > 0 ? (
                <HubList
                  items={editorActivity.map((action) => ({
                    id: action.id,
                    title: action.action,
                    detail: action.detail,
                    meta: action.finished,
                    icon: <TerminalOutlinedIcon fontSize="small" />,
                  }))}
                />
              ) : (
                <EmptyStateBlock title={text.noEditorActivityTitle} detail={text.noEditorActivityDetail} />
              )}
            </HubPanel>
            <HubPanel title={text.sourceBuildHistory}>
              {sourceBuildHistory.length > 0 ? (
                <HubList
                  items={sourceBuildHistory.map((record) => ({
                    id: record.id,
                    title: record.detail,
                    detail: record.outputDir,
                    secondaryDetail: record.secondaryDetail,
                    meta: record.finished,
                    icon: <TerminalOutlinedIcon fontSize="small" />,
                    disabled: !record.outputDir,
                  }))}
                  onSelect={(item) => {
                    const record = sourceBuildHistory.find((history) => history.id === item.id);
                    void onAction(HUB_ACTION.openOutputFolder, item.id, { outputDir: record?.outputDir });
                  }}
                />
              ) : (
                <EmptyStateBlock title={state.ui.builds.noBuildHistory} detail={state.ui.builds.noBuildHistoryDetail} />
              )}
            </HubPanel>
            <HubPanel title={text.launchReadiness}>
              <Box sx={{ display: "grid", gap: 1 }}>
                <HubSwitch checked={Boolean(project?.exists)} label={text.projectAvailable} detail={project?.path ?? common.noProjectSelected} disabled />
                <HubCheckbox checked={state.sourceEngines.length > 0} label={text.sourceEngineRegistered} detail={state.engineVersion} disabled />
                <HubCheckbox checked={editorPlugins.length > 0} label={text.editorPluginScopeStatus} detail={formatCountText(common.availableCountTemplate, editorPlugins.length)} disabled />
              </Box>
            </HubPanel>
          </>
        ) : null}
      </Box>
    </Box>
  );
}
