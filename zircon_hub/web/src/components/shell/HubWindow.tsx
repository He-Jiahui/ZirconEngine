import { Box } from "@mui/material";
import { ProjectsDashboard } from "../../pages/ProjectsDashboard";
import { hubTokens } from "../../theme/tokens";
import type { HubShellState } from "../../types/hub";
import { NavigationDrawer } from "./NavigationDrawer";
import { TopBar } from "./TopBar";

export interface HubWindowProps {
  state: HubShellState;
}

export function HubWindow({ state }: HubWindowProps) {
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
      <TopBar state={state} />
      <Box sx={{ display: "flex", height: `calc(100vh - ${hubTokens.window.topBarHeight}px)`, minHeight: 0 }}>
        <NavigationDrawer activePage={state.activePage} />
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
          <ProjectsDashboard state={state} />
        </Box>
      </Box>
    </Box>
  );
}
