import AddIcon from "@mui/icons-material/Add";
import DownloadIcon from "@mui/icons-material/Download";
import FolderOutlinedIcon from "@mui/icons-material/FolderOutlined";
import FormatListBulletedIcon from "@mui/icons-material/FormatListBulleted";
import GridViewIcon from "@mui/icons-material/GridView";
import SearchOffIcon from "@mui/icons-material/SearchOff";
import { Box, Typography } from "@mui/material";
import { useEffect, useMemo, useState } from "react";
import { EmptyStateBlock, HubPanel, ProjectCard, ProjectTable, QuickActions } from "../components/data";
import { HubDialog } from "../components/overlays";
import { HubButton, HubComboBox, HubSearchField, HubSelect, HubTextField, HubToggle } from "../components/inputs";
import { quickActionProjectTargetPayload } from "../tauri/projectTarget";
import { hubTokens } from "../theme/tokens";
import type { HubActionHandler, HubProjectSummary, HubRecentProject, HubShellState } from "../types/hub";
import { HUB_ACTION } from "../types/hub";
import { ProjectBrowserPage } from "./ProjectBrowserPage";
import { ProjectDetailPage } from "./ProjectDetailPage";

export interface ProjectsDashboardProps {
  state: HubShellState;
  onAction: HubActionHandler;
}

export function ProjectsDashboard({ state, onAction }: ProjectsDashboardProps) {
  const text = state.ui.projects;
  const actionText = state.ui.actions;
  const tableLabels = {
    name: text.tableName,
    engineVersion: text.tableEngineVersion,
    lastModified: text.tableLastModified,
    location: text.tableLocation,
    openDetails: text.openProjectDetailsLabel,
  };
  const [search, setSearch] = useState(state.searchQuery);
  const [filter, setFilter] = useState(state.projectFilter);
  const [sort, setSort] = useState(state.projectSort);
  const [viewMode, setViewMode] = useState(state.projectViewMode);
  const [projectName, setProjectName] = useState("");
  const [projectLocation, setProjectLocation] = useState(state.settings.defaultProjectDir);
  const [template, setTemplate] = useState("renderable-empty");
  const [engineId, setEngineId] = useState(state.activeSourceEngineId ?? state.sourceEngines[0]?.id ?? "");
  const quickActionProjectTarget = quickActionProjectTargetPayload(state.selectedProject);

  useEffect(() => {
    setSearch(state.searchQuery);
    setFilter(state.projectFilter);
    setSort(state.projectSort);
    setViewMode(state.projectViewMode);
  }, [state.projectFilter, state.projectSort, state.projectViewMode, state.searchQuery]);

  useEffect(() => {
    setProjectLocation(state.settings.defaultProjectDir);
  }, [state.settings.defaultProjectDir]);

  useEffect(() => {
    setEngineId((currentEngineId) => {
      if (state.sourceEngines.some((engine) => engine.id === currentEngineId)) {
        return currentEngineId;
      }
      return state.activeSourceEngineId ?? state.sourceEngines[0]?.id ?? "";
    });
  }, [state.activeSourceEngineId, state.sourceEngines]);

  useEffect(() => {
    if (state.projectTemplates.some((projectTemplate) => projectTemplate.id === template && projectTemplate.enabled)) {
      return;
    }
    const firstEnabled = state.projectTemplates.find((projectTemplate) => projectTemplate.enabled);
    if (firstEnabled) {
      setTemplate(firstEnabled.id);
    }
  }, [state.projectTemplates, template]);

  const visibleProjects = useMemo(() => {
    const query = search.trim().toLowerCase();
    if (!query) {
      return state.projects;
    }
    return state.projects.filter((project) => `${project.name} ${project.path}`.toLowerCase().includes(query));
  }, [search, state.projects]);
  const dashboardProjects = useMemo(() => visibleProjects.slice(0, 4), [visibleProjects]);
  const tableProjects = state.browserProjects.length > 0 ? state.browserProjects : state.recentProjects;

  const handleOpenProject = (project: HubProjectSummary) => {
    void onAction(HUB_ACTION.openProjectDetail, project.id);
  };
  const selectedTemplate = state.projectTemplates.find((projectTemplate) => projectTemplate.id === template);
  const createDisabled = projectName.trim().length === 0 || projectLocation.trim().length === 0 || !selectedTemplate?.enabled;
  const createProject = () => {
    if (createDisabled) {
      return;
    }
    void onAction(HUB_ACTION.createProject, undefined, {
      name: projectName,
      location: projectLocation,
      template,
      engineId: engineId || null,
    });
  };

  const visibleRows = useMemo<HubRecentProject[]>(() => {
    const query = search.trim().toLowerCase();
    if (!query) {
      return tableProjects;
    }
    return tableProjects.filter((project) => `${project.name} ${project.location}`.toLowerCase().includes(query));
  }, [search, tableProjects]);

  if (state.projectSubpage === "project-browser") {
    return <ProjectBrowserPage state={state} onAction={onAction} />;
  }

  if (state.projectSubpage === "project-detail") {
    return <ProjectDetailPage state={state} onAction={onAction} />;
  }

  return (
    <Box sx={{ height: "100%", minHeight: 0, display: "grid" }}>
      <Box
        sx={{
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
            <Typography variant="h4">{text.title}</Typography>
            <Typography variant="body1" color="text.secondary" sx={{ mt: 0.9 }}>
              {state.pageSubtitle}
            </Typography>
          </Box>
          <Box sx={{ display: "flex", gap: 1.2, flexWrap: "wrap", justifyContent: "flex-end" }}>
            <HubButton startIcon={<DownloadIcon />} sx={{ minWidth: 172 }} onClick={() => void onAction(HUB_ACTION.importProject)}>
              {actionText.importProject}
            </HubButton>
            <HubButton tone="primary" startIcon={<AddIcon />} sx={{ minWidth: 208 }} onClick={() => void onAction(HUB_ACTION.newProject)}>
              {actionText.newProject}
            </HubButton>
          </Box>
        </Box>

        <Box
          sx={{
            display: "grid",
            gridTemplateColumns: "minmax(260px, 307px) 1fr auto auto auto",
            alignItems: "center",
            gap: 1.2,
            mt: 2,
            "@media (max-width: 1180px)": {
              gridTemplateColumns: "minmax(240px, 1fr) auto auto",
            },
            "@media (max-width: 760px)": {
              gridTemplateColumns: "1fr",
            },
          }}
        >
          <HubSearchField
            value={search}
            placeholder={text.searchPlaceholder}
            onChange={(value) => {
              setSearch(value);
              void onAction(HUB_ACTION.searchProjects, undefined, { query: value });
            }}
          />
          <Box sx={{ minWidth: 0 }} />
          <HubSelect
            value={filter}
            minWidth={183}
            options={[
              { value: "all", label: text.filterAll },
              { value: "existing", label: text.filterExisting },
              { value: "missing", label: text.filterMissing },
            ]}
            onChange={(value) => {
              setFilter(value);
              void onAction(HUB_ACTION.setProjectFilter, value);
            }}
          />
          <HubSelect
            value={sort}
            minWidth={190}
            options={[
              { value: "last-modified", label: text.sortLastModified },
              { value: "name", label: text.sortName },
            ]}
            onChange={(value) => {
              setSort(value);
              void onAction(HUB_ACTION.setProjectSort, value);
            }}
          />
          <HubToggle
            value={viewMode}
            onChange={(value) => {
              setViewMode(value);
              void onAction(HUB_ACTION.setProjectViewMode, value);
            }}
            options={[
              { value: "grid", label: text.gridView, icon: <GridViewIcon /> },
              { value: "list", label: text.listView, icon: <FormatListBulletedIcon /> },
            ]}
          />
        </Box>

        {(viewMode === "list" ? visibleRows.length === 0 : visibleProjects.length === 0) ? (
          <Box sx={{ mt: 2.3 }}>
            <EmptyStateBlock
              title={text.noProjectsFound}
              detail={text.searchFiltersEmpty}
              icon={<SearchOffIcon />}
            />
          </Box>
        ) : viewMode === "list" ? (
          <Box sx={{ mt: 2.3 }}>
            <HubPanel title={text.projectBrowser}>
              <ProjectTable
                projects={visibleRows}
                selectedProjectId={state.selectedProjectId}
                labels={tableLabels}
                onSelect={(project) => void onAction(HUB_ACTION.selectProject, project.id)}
                onOpenDetail={(project) => void onAction(HUB_ACTION.openProjectDetail, project.id)}
              />
            </HubPanel>
          </Box>
        ) : (
          <Box
            sx={{
              display: "grid",
              gridTemplateColumns: "repeat(4, minmax(220px, 296px))",
              gap: 2,
              mt: 2.3,
              "@media (max-width: 1360px)": {
                gridTemplateColumns: "repeat(3, minmax(220px, 1fr))",
              },
              "@media (max-width: 1080px)": {
                gridTemplateColumns: "repeat(2, minmax(220px, 1fr))",
              },
              "@media (max-width: 760px)": {
                gridTemplateColumns: "1fr",
              },
            }}
          >
            {dashboardProjects.map((project) => (
              <ProjectCard
                key={project.id}
                project={project}
                selected={project.id === state.selectedProjectId}
                openDetailsLabel={text.openProjectDetailsLabel}
                onOpen={handleOpenProject}
              />
            ))}
          </Box>
        )}

        <Box
          sx={{
            display: "grid",
            gridTemplateColumns: "minmax(0, 1fr) minmax(330px, 0.58fr)",
            gap: 1.4,
            mt: 2.6,
            "@media (max-width: 1180px)": {
              gridTemplateColumns: "1fr",
            },
          }}
        >
          <HubPanel
            title={text.recentProjects}
            action={
              <HubButton
                startIcon={<FolderOutlinedIcon />}
                sx={{ height: 32, minWidth: 196 }}
                onClick={() => void onAction(HUB_ACTION.viewAllProjects)}
              >
                {actionText.viewAllProjects}
              </HubButton>
            }
          >
            <ProjectTable
              projects={state.recentProjects}
              selectedProjectId={state.selectedProjectId}
              labels={tableLabels}
              onSelect={(project) => void onAction(HUB_ACTION.selectProject, project.id)}
              onOpenDetail={(project) => void onAction(HUB_ACTION.openProjectDetail, project.id)}
            />
          </HubPanel>

          <HubPanel title={text.quickActions}>
            <QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />
          </HubPanel>
        </Box>
      </Box>

      <HubDialog
        open={state.projectSubpage === "new-project"}
        title={text.newProjectDialog}
        onClose={() => void onAction(HUB_ACTION.viewAllProjects)}
        actions={
          <>
            <HubButton onClick={() => void onAction(HUB_ACTION.viewAllProjects)}>{actionText.close}</HubButton>
            <HubButton tone="primary" disabled={createDisabled} onClick={createProject}>
              {actionText.createProject}
            </HubButton>
          </>
        }
      >
        <Box sx={{ display: "grid", gap: 1.4, pt: 0.5 }}>
          <HubTextField label={text.projectName} value={projectName} onChange={(event) => setProjectName(event.target.value)} />
          <HubTextField label={text.location} value={projectLocation} onChange={(event) => setProjectLocation(event.target.value)} />
          <HubComboBox
            value={engineId}
            minWidth={0}
            placeholder={text.sourceEngine}
            options={state.sourceEngines.map((engine) => ({
              value: engine.id,
              label: engine.name,
              detail: engine.sourcePath,
            }))}
            onChange={setEngineId}
          />
          <HubComboBox
            value={template}
            minWidth={0}
            placeholder={text.template}
            options={state.projectTemplates.map((projectTemplate) => ({
              value: projectTemplate.id,
              label: projectTemplate.optionLabel,
              detail: projectTemplate.disabledReason ?? projectTemplate.description,
              disabled: !projectTemplate.enabled,
            }))}
            onChange={setTemplate}
          />
        </Box>
      </HubDialog>
    </Box>
  );
}
