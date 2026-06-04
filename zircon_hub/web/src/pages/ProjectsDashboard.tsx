import AddIcon from "@mui/icons-material/Add";
import DownloadIcon from "@mui/icons-material/Download";
import FolderOutlinedIcon from "@mui/icons-material/FolderOutlined";
import FormatListBulletedIcon from "@mui/icons-material/FormatListBulleted";
import GridViewIcon from "@mui/icons-material/GridView";
import { Box, Typography } from "@mui/material";
import { useMemo, useState } from "react";
import { ButtonStatesPanel, HubPanel, ProjectCard, ProjectTable, QuickActions } from "../components/data";
import { HubButton, HubSearchField, HubSelect, HubToggle } from "../components/inputs";
import { dispatchHubAction } from "../tauri/hubApi";
import { hubTokens } from "../theme/tokens";
import type { HubProjectSummary, HubShellState } from "../types/hub";

export interface ProjectsDashboardProps {
  state: HubShellState;
}

export function ProjectsDashboard({ state }: ProjectsDashboardProps) {
  const [search, setSearch] = useState("");
  const [filter, setFilter] = useState("all");
  const [sort, setSort] = useState("last-modified");
  const [viewMode, setViewMode] = useState("grid");

  const visibleProjects = useMemo(() => {
    const query = search.trim().toLowerCase();
    if (!query) {
      return state.projects;
    }
    return state.projects.filter((project) => `${project.name} ${project.path}`.toLowerCase().includes(query));
  }, [search, state.projects]);

  const handleOpenProject = (project: HubProjectSummary) => {
    void dispatchHubAction("open-project", project.id);
  };

  return (
    <Box sx={{ height: "100%", minHeight: 0, display: "grid", gridTemplateRows: "1fr auto" }}>
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
            <Typography variant="h4">Projects</Typography>
            <Typography variant="body1" color="text.secondary" sx={{ mt: 0.9 }}>
              Manage your projects and start building worlds.
            </Typography>
          </Box>
          <Box sx={{ display: "flex", gap: 1.2, flexWrap: "wrap", justifyContent: "flex-end" }}>
            <HubButton startIcon={<DownloadIcon />} sx={{ minWidth: 172 }}>
              Import Project
            </HubButton>
            <HubButton tone="primary" startIcon={<AddIcon />} sx={{ minWidth: 208 }}>
              New Project
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
          <HubSearchField value={search} placeholder="Search projects..." onChange={setSearch} />
          <Box sx={{ minWidth: 0 }} />
          <HubSelect
            value={filter}
            minWidth={183}
            options={[
              { value: "all", label: "All Projects" },
              { value: "windows", label: "Windows" },
              { value: "linux", label: "Linux" },
            ]}
            onChange={setFilter}
          />
          <HubSelect
            value={sort}
            minWidth={190}
            options={[
              { value: "last-modified", label: "Last Modified" },
              { value: "name", label: "Name" },
              { value: "engine", label: "Engine Version" },
            ]}
            onChange={setSort}
          />
          <HubToggle
            value={viewMode}
            onChange={setViewMode}
            options={[
              { value: "grid", label: "Grid view", icon: <GridViewIcon /> },
              { value: "list", label: "List view", icon: <FormatListBulletedIcon /> },
            ]}
          />
        </Box>

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
          {visibleProjects.map((project) => (
            <ProjectCard key={project.id} project={project} selected={project.id === "elysium"} onOpen={handleOpenProject} />
          ))}
        </Box>

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
            title="Recent Projects"
            action={
              <HubButton startIcon={<FolderOutlinedIcon />} sx={{ height: 32, minWidth: 196 }}>
                View All Projects
              </HubButton>
            }
          >
            <ProjectTable projects={state.recentProjects} />
          </HubPanel>

          <HubPanel title="Quick Actions">
            <QuickActions actions={state.quickActions} onAction={(action) => void dispatchHubAction(action.id)} />
          </HubPanel>
        </Box>
      </Box>

      <Box
        sx={{
          borderTop: `1px solid ${hubTokens.colors.line}`,
          backgroundColor: "rgba(15,15,15,0.95)",
        }}
      >
        <Box sx={{ px: 2, pt: 1.3 }}>
          <Typography variant="h6">Button States</Typography>
        </Box>
        <ButtonStatesPanel />
      </Box>
    </Box>
  );
}
