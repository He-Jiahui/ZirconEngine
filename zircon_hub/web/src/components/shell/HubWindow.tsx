import { Alert, Box, Typography } from "@mui/material";
import { ProjectsDashboard } from "../../pages/ProjectsDashboard";
import { hubTokens } from "../../theme/tokens";
import type { WindowActionFailureHandler } from "../../tauri/windowActionScheduler";
import { lazy, Suspense, type ComponentType, type LazyExoticComponent } from "react";
import type { HubActionHandler, HubPageId, HubShellState } from "../../types/hub";
import { NavigationDrawer } from "./NavigationDrawer";
import { TopBar } from "./TopBar";

export interface HubWindowProps {
  state: HubShellState;
  onAction: HubActionHandler;
  onWindowActionFailure: WindowActionFailureHandler;
}

type HubPageComponent = ComponentType<HubWindowProps>;
type LazyHubPage = LazyExoticComponent<HubPageComponent>;
type HubPageRoute = HubPageComponent | LazyHubPage;

const buildsPage = lazy(() => import("../../pages/BuildsPage").then(({ BuildsPage }) => ({ default: BuildsPage })));
const catalogPage = lazy(() => import("../../pages/CatalogPage").then(({ CatalogPage }) => ({ default: CatalogPage })));
const cloudPage = lazy(() => import("../../pages/CloudPage").then(({ CloudPage }) => ({ default: CloudPage })));
const editorPage = lazy(() => import("../../pages/EditorPage").then(({ EditorPage }) => ({ default: EditorPage })));
const settingsPage = lazy(() => import("../../pages/SettingsPage").then(({ SettingsPage }) => ({ default: SettingsPage })));
const teamPage = lazy(() => import("../../pages/TeamPage").then(({ TeamPage }) => ({ default: TeamPage })));
const workspacePage = lazy(() => import("../../pages/WorkspacePage").then(({ WorkspacePage }) => ({ default: WorkspacePage })));

const pageRoutes: Record<HubPageId, HubPageRoute> = {
  projects: ProjectsDashboard,
  editor: editorPage,
  assets: catalogPage,
  builds: buildsPage,
  plugins: catalogPage,
  cloud: cloudPage,
  team: teamPage,
  learn: catalogPage,
  settings: settingsPage,
};

export function HubWindow({ state, onAction, onWindowActionFailure }: HubWindowProps) {
  const activeRoute = toHubPageId(state.activePage);
  const PageComponent = activeRoute ? pageRoutes[activeRoute] : workspacePage;

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
      <TopBar state={state} onAction={onAction} onWindowActionFailure={onWindowActionFailure} />
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
            display: "flex",
            flexDirection: "column",
            flex: "1 1 auto",
            minWidth: 0,
            minHeight: 0,
            overflow: "hidden",
            backgroundColor: "rgba(17,17,17,0.55)",
          }}
        >
          {state.windowCloseSaveError ? (
            <Alert
              severity="error"
              variant="filled"
              role="alert"
              tabIndex={0}
              sx={{ flexShrink: 0, mx: 2, mt: 1, maxHeight: "30vh", overflowY: "auto", overflowWrap: "anywhere", "& .MuiAlert-message": { overflow: "visible" } }}
            >
              <Box sx={{ display: "flex", flexDirection: "column", gap: 0.5, textAlign: "start" }}>
                <Typography variant="subtitle2">{state.windowCloseSaveError.label}</Typography>
                <Typography variant="body2">{state.windowCloseSaveError.detail}</Typography>
                {state.windowCloseSaveError.recovery ? <Typography variant="caption">{state.windowCloseSaveError.recovery}</Typography> : null}
              </Box>
            </Alert>
          ) : null}
          <Box sx={{ flex: "1 1 auto", minWidth: 0, minHeight: 0, overflow: "hidden" }}>
            <Suspense fallback={<PageLoading label={state.pageTitle} />}>
              <PageComponent key={activeRoute ?? state.activePage} state={state} onAction={onAction} onWindowActionFailure={onWindowActionFailure} />
            </Suspense>
          </Box>
        </Box>
      </Box>
    </Box>
  );
}

function PageLoading({ label }: { label: string }) {
  return (
    <Box
      role="status"
      aria-label={label}
      sx={{
        height: "100%",
        display: "grid",
        placeItems: "center",
        color: hubTokens.colors.textSoft,
      }}
    >
      {label}
    </Box>
  );
}

function toHubPageId(activePage: string): HubPageId | undefined {
  return activePage in pageRoutes ? (activePage as HubPageId) : undefined;
}
