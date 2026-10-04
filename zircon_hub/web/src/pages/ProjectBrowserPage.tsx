import AddIcon from "@mui/icons-material/Add";
import DashboardCustomizeOutlinedIcon from "@mui/icons-material/DashboardCustomizeOutlined";
import FormatListBulletedIcon from "@mui/icons-material/FormatListBulleted";
import GridViewIcon from "@mui/icons-material/GridView";
import SearchOffIcon from "@mui/icons-material/SearchOff";
import { Box, Typography } from "@mui/material";
import { useEffect, useMemo, useState } from "react";
import { EmptyStateBlock, HubPanel, PageHeader, ProjectTable, QuickActions, SourceEngineList } from "../components/data";
import { HubStatusBanner } from "../components/feedback";
import { HubButton, HubSearchField, HubSelect, HubToggle } from "../components/inputs";
import { useDebouncedProjectSearch } from "../projects/debouncedProjectSearch";
import { buildSearchIndex, filterSearchIndex } from "../projects/searchIndex";
import { quickActionProjectTargetPayload } from "../tauri/projectTarget";
import { hubTokens } from "../theme/tokens";
import type { HubActionHandler, HubRecentProject, HubShellState } from "../types/hub";
import { HUB_ACTION } from "../types/hub";

export interface ProjectBrowserPageProps {
  state: HubShellState;
  onAction: HubActionHandler;
}

export function ProjectBrowserPage({ state, onAction }: ProjectBrowserPageProps) {
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

  const browserProjects = state.browserProjects.length > 0 ? state.browserProjects : state.recentProjects;
  const searchIndex = useMemo(
    () => buildSearchIndex(browserProjects, (project) => `${project.name} ${project.location}`),
    [browserProjects],
  );
  const visibleRows = useMemo(() => {
    return filterSearchIndex(browserProjects, searchIndex, search);
  }, [browserProjects, search, searchIndex]);

  const openDetail = (project: HubRecentProject) => {
    void onAction(HUB_ACTION.openProjectDetail, project.id);
  };

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
      <PageHeader title={text.browserTitle} subtitle={state.pageSubtitle} actions={<>
          <HubButton startIcon={<DashboardCustomizeOutlinedIcon />} onClick={() => void onAction(HUB_ACTION.showProjectSubpage, "dashboard")}>
            {actionText.dashboard}
          </HubButton>
          <HubButton tone="primary" startIcon={<AddIcon />} onClick={() => void onAction(HUB_ACTION.newProject)}>
            {actionText.newProject}
          </HubButton>
        </>} />

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
          width: "100%",
          minWidth: 0,
          gridTemplateColumns: "minmax(280px, 420px) 1fr auto auto auto",
          alignItems: "center",
          gap: 1.2,
          mb: 1.4,
          "@media (max-width: 1180px)": {
            gridTemplateColumns: "minmax(240px, 1fr) auto auto",
          },
          "@media (max-width: 760px)": {
            gridTemplateColumns: "minmax(0, 1fr)",
            "& > *": { minWidth: 0, maxWidth: "100%" },
          },
        }}
      >
        <HubSearchField
          value={search}
          placeholder={text.searchPlaceholder}
          onChange={(value) => {
            setSearch(value);
            dispatchProjectSearch(value);
          }}
        />
        <Box sx={{ minWidth: 0 }} />
        <HubSelect
          value={filter}
          label={text.filterLabel}
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
          label={text.sortLabel}
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
          ariaLabel={text.browserTitle}
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

      <Box
        sx={{
          display: "grid",
          gridTemplateColumns: "minmax(0, 1fr) minmax(320px, 0.42fr)",
          gap: 1.4,
          "@media (max-width: 1180px)": {
            gridTemplateColumns: "1fr",
          },
        }}
      >
        <HubPanel title={text.allProjects}>
          {visibleRows.length === 0 ? (
            <EmptyStateBlock title={text.noProjectsFound} detail={text.noRecentProjectMatches} icon={<SearchOffIcon />} />
          ) : (
            <ProjectTable
              projects={visibleRows}
              selectedProjectId={state.selectedProjectId}
              labels={tableLabels}
              onSelect={(project) => void onAction(HUB_ACTION.selectProject, project.id)}
              onOpenDetail={openDetail}
            />
          )}
        </HubPanel>

        <Box sx={{ display: "grid", gap: 1.4, alignContent: "start" }}>
          <HubPanel title={text.quickActions}>
            <QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />
          </HubPanel>
          <HubPanel title={text.sourceEngines}>
            <SourceEngineList engines={state.sourceEngines} emptyLabel={state.ui.shell.noSourceEngineRegistered} onSelect={(engine) => void onAction(HUB_ACTION.selectEngine, engine.id)} />
          </HubPanel>
        </Box>
      </Box>
    </Box>
  );
}
