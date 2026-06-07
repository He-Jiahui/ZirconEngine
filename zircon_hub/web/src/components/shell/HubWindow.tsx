import { Box } from "@mui/material";
import { BuildsPage } from "../../pages/BuildsPage";
import { CatalogPage } from "../../pages/CatalogPage";
import { CloudPage } from "../../pages/CloudPage";
import { EditorPage } from "../../pages/EditorPage";
import { ProjectsDashboard } from "../../pages/ProjectsDashboard";
import { SettingsPage } from "../../pages/SettingsPage";
import { TeamPage } from "../../pages/TeamPage";
import { WorkspacePage } from "../../pages/WorkspacePage";
import { hubTokens } from "../../theme/tokens";
import type { HubActionHandler, HubShellState } from "../../types/hub";
import { NavigationDrawer } from "./NavigationDrawer";
import { TopBar } from "./TopBar";

export interface HubWindowProps {
  state: HubShellState;
  onAction: HubActionHandler;
}

export function HubWindow({ state, onAction }: HubWindowProps) {
  return (
    <Box
      sx={{
        width: "100vw",
        height: "100vh",
        minWidth: 0,
        minHeight: 0,
        overflow: "hidden",
        color: hubTokens.colors.text,
        background:
          "radial-gradient(circle at 30% 18%, rgba(38,86,82,0.13), transparent 30%), linear-gradient(180deg, #161616 0%, #111111 100%)",
        border: `1px solid ${hubTokens.colors.lineStrong}`,
        borderRadius: "10px",
      }}
    >
      <TopBar state={state} onAction={onAction} />
      <Box sx={{ display: "flex", height: `calc(100vh - ${hubTokens.window.topBarHeight}px)`, minHeight: 0 }}>
        <NavigationDrawer activePage={state.activePage} text={state.ui.shell} engineVersion={state.engineVersion} onAction={onAction} />
        <Box
          component="main"
          sx={{
            flex: "1 1 auto",
            minWidth: 0,
            minHeight: 0,
            overflow: "hidden",
            backgroundColor: "rgba(17,17,17,0.55)",
          }}
        >
          {state.activePage === "projects" ? (
            <ProjectsDashboard state={state} onAction={onAction} />
          ) : state.activePage === "editor" ? (
            <EditorPage state={state} onAction={onAction} />
          ) : state.activePage === "builds" ? (
            <BuildsPage state={state} onAction={onAction} />
          ) : state.activePage === "cloud" ? (
            <CloudPage state={state} onAction={onAction} />
          ) : state.activePage === "assets" || state.activePage === "plugins" || state.activePage === "learn" ? (
            <CatalogPage state={state} onAction={onAction} />
          ) : state.activePage === "team" ? (
            <TeamPage state={state} onAction={onAction} />
          ) : state.activePage === "settings" ? (
            <SettingsPage state={state} onAction={onAction} />
          ) : (
            <WorkspacePage state={state} onAction={onAction} />
          )}
        </Box>
      </Box>
    </Box>
  );
}
