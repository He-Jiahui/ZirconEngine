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
import type { ComponentType } from "react";
import type { HubActionHandler, HubPageId, HubShellState } from "../../types/hub";
import { NavigationDrawer } from "./NavigationDrawer";
import { TopBar } from "./TopBar";

export interface HubWindowProps {
  state: HubShellState;
  onAction: HubActionHandler;
}

type HubPageComponent = ComponentType<HubWindowProps>;

const pageRoutes: Record<HubPageId, HubPageComponent> = {
  projects: ProjectsDashboard,
  editor: EditorPage,
  assets: CatalogPage,
  builds: BuildsPage,
  plugins: CatalogPage,
  cloud: CloudPage,
  team: TeamPage,
  learn: CatalogPage,
  settings: SettingsPage,
};

export function HubWindow({ state, onAction }: HubWindowProps) {
  const activeRoute = toHubPageId(state.activePage);
  const PageComponent = activeRoute ? pageRoutes[activeRoute] : WorkspacePage;

  return (
    <Box
      sx={{
        width: "100vw",
        height: "100vh",
        minWidth: 0,
        minHeight: 0,
        overflow: "hidden",
        color: hubTokens.colors.text,
        background: hubTokens.gradients.window,
        border: `1px solid ${hubTokens.colors.lineStrong}`,
        borderRadius: "10px",
      }}
    >
      <TopBar state={state} onAction={onAction} />
      <Box sx={{ display: "flex", height: `calc(100vh - ${hubTokens.window.topBarHeight}px)`, minHeight: 0 }}>
        <NavigationDrawer
          activePage={state.activePage}
          text={state.ui.shell}
          engineVersion={state.engineVersion}
          sourceEngines={state.sourceEngines}
          activeSourceEngineId={state.activeSourceEngineId}
          onAction={onAction}
        />
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
          <PageComponent state={state} onAction={onAction} />
        </Box>
      </Box>
    </Box>
  );
}

function toHubPageId(activePage: string): HubPageId | undefined {
  return activePage in pageRoutes ? (activePage as HubPageId) : undefined;
}
