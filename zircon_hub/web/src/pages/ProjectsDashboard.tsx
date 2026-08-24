import AddIcon from "@mui/icons-material/Add";
import DownloadIcon from "@mui/icons-material/Download";
import FolderOutlinedIcon from "@mui/icons-material/FolderOutlined";
import SearchOffIcon from "@mui/icons-material/SearchOff";
import { Box, Typography } from "@mui/material";
import { useEffect, useMemo, useState } from "react";
import { EmptyStateBlock, HubPanel, ProjectCard, ProjectCardRail, ProjectTable, QuickActions } from "../components/data";
import { CreateProjectDialog, HubMenu, type HubMenuItem } from "../components/overlays";
import { HubStatusBanner } from "../components/feedback";
import { HubButton, ProjectsToolbar } from "../components/inputs";
import { useDebouncedProjectSearch } from "../projects/debouncedProjectSearch";
import { buildSearchIndex, filterSearchIndex } from "../projects/searchIndex";
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
  const [rowMenu, setRowMenu] = useState<{ anchor: HTMLElement; project: HubRecentProject } | null>(null);
  const dispatchProjectSearch = useDebouncedProjectSearch((query) => {
    void onAction(HUB_ACTION.searchProjects, undefined, { query });
  });
  const quickActionProjectTarget = quickActionProjectTargetPayload(state.selectedProject);

  useEffect(() => {
    setSearch(state.searchQuery);
    setFilter(state.projectFilter);
    setSort(state.projectSort);
    setViewMode(state.projectViewMode);
  }, [state.projectFilter, state.projectSort, state.projectViewMode, state.searchQuery]);

  const projectSearchIndex = useMemo(
    () => buildSearchIndex(state.projects, (project) => `${project.name} ${project.path}`),
    [state.projects],
  );
  const visibleProjects = useMemo(
    () => filterSearchIndex(state.projects, projectSearchIndex, search),
    [projectSearchIndex, search, state.projects],
  );
  const dashboardProjects = useMemo(() => visibleProjects.slice(0, 4), [visibleProjects]);
  const tableProjects = state.browserProjects.length > 0 ? state.browserProjects : state.recentProjects;
  const tableSearchIndex = useMemo(
    () => buildSearchIndex(tableProjects, (project) => `${project.name} ${project.location}`),
    [tableProjects],
  );

  const handleOpenProject = (project: HubProjectSummary) => {
    void onAction(HUB_ACTION.openProjectDetail, project.id);
  };

  const rowMenuItems = (project: HubRecentProject): HubMenuItem[] => [
    { id: HUB_ACTION.openProjectDetail, label: text.openProjectDetailsLabel },
    project.pinned
      ? { id: HUB_ACTION.unpinProject, label: actionText.unpinProject }
      : { id: HUB_ACTION.pinProject, label: actionText.pinProject },
    { id: HUB_ACTION.requestDelete, label: actionText.requestDelete },
  ];

  const handleRowMenuSelect = (project: HubRecentProject, itemId: string) => {
    if (itemId === HUB_ACTION.openProjectDetail) {
      void onAction(HUB_ACTION.openProjectDetail, project.id);
      return;
    }
    if (itemId === HUB_ACTION.pinProject || itemId === HUB_ACTION.unpinProject) {
      void onAction(itemId, undefined, { projectId: project.id });
      return;
    }
    void onAction(HUB_ACTION.requestDelete, undefined, { projectId: project.id });
  };

  const visibleRows = useMemo<HubRecentProject[]>(
    () => filterSearchIndex(tableProjects, tableSearchIndex, search),
    [search, tableProjects, tableSearchIndex],
  );

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

        <Box sx={{ mb: 1.4 }}>
          <HubStatusBanner task={state.taskSummary} />
        </Box>

        <ProjectsToolbar
          search={search}
          filter={filter}
          sort={sort}
          viewMode={viewMode}
          text={text}
          onSearch={(value) => {
            setSearch(value);
            dispatchProjectSearch(value);
          }}
          onFilter={(value) => {
            setFilter(value);
            void onAction(HUB_ACTION.setProjectFilter, value);
          }}
          onSort={(value) => {
            setSort(value);
            void onAction(HUB_ACTION.setProjectSort, value);
          }}
          onViewMode={(value) => {
            setViewMode(value);
            void onAction(HUB_ACTION.setProjectViewMode, value);
          }}
        />

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
                onRowMenu={(project, anchor) => setRowMenu({ anchor, project })}
              />
            </HubPanel>
          </Box>
        ) : (
          <Box sx={{ mt: 2.3 }}>
            <ProjectCardRail
              moreLabel={actionText.viewAllProjects}
              hasMore={visibleProjects.length > dashboardProjects.length}
              onMore={() => void onAction(HUB_ACTION.viewAllProjects)}
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
            </ProjectCardRail>
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
              onRowMenu={(project, anchor) => setRowMenu({ anchor, project })}
            />
          </HubPanel>

          <HubPanel title={text.quickActions}>
            <QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />
          </HubPanel>
        </Box>
      </Box>

      <CreateProjectDialog
        open={state.projectSubpage === "new-project"}
        templates={state.projectTemplates}
        sourceEngines={state.sourceEngines}
        activeSourceEngineId={state.activeSourceEngineId}
        defaultProjectDir={state.settings.defaultProjectDir}
        text={text}
        actionText={actionText}
        onClose={() => void onAction(HUB_ACTION.viewAllProjects)}
        onCreate={(payload) => void onAction(HUB_ACTION.createProject, undefined, payload)}
      />
      {rowMenu ? (
        <HubMenu
          anchorEl={rowMenu.anchor}
          open
          items={rowMenuItems(rowMenu.project)}
          onClose={() => setRowMenu(null)}
          onSelect={(itemId) => handleRowMenuSelect(rowMenu.project, itemId)}
        />
      ) : null}
    </Box>
  );
}
