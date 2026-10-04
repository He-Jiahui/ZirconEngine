import AutoStoriesOutlinedIcon from "@mui/icons-material/AutoStoriesOutlined";
import CloudOutlinedIcon from "@mui/icons-material/CloudOutlined";
import ConstructionOutlinedIcon from "@mui/icons-material/ConstructionOutlined";
import ExtensionOutlinedIcon from "@mui/icons-material/ExtensionOutlined";
import FolderOutlinedIcon from "@mui/icons-material/FolderOutlined";
import GroupsOutlinedIcon from "@mui/icons-material/GroupsOutlined";
import Inventory2OutlinedIcon from "@mui/icons-material/Inventory2Outlined";
import KeyboardDoubleArrowLeftIcon from "@mui/icons-material/KeyboardDoubleArrowLeft";
import KeyboardDoubleArrowRightIcon from "@mui/icons-material/KeyboardDoubleArrowRight";
import SettingsOutlinedIcon from "@mui/icons-material/SettingsOutlined";
import WebAssetOutlinedIcon from "@mui/icons-material/WebAssetOutlined";
import { Box, ButtonBase, Drawer, List, ListItemButton, ListItemIcon, Tooltip, Typography, useMediaQuery } from "@mui/material";
import { useState } from "react";
import { hubTokens } from "../../theme/tokens";
import { admittedSourceEngineId } from "../../projections/sourceEngineChoices";
import type { HubActionHandler, HubPageId, HubShellText, HubSourceEngineSummary } from "../../types/hub";
import { HUB_ACTION } from "../../types/hub";

const navIcons: Record<HubPageId, typeof FolderOutlinedIcon> = {
  projects: FolderOutlinedIcon,
  editor: WebAssetOutlinedIcon,
  assets: Inventory2OutlinedIcon,
  builds: ConstructionOutlinedIcon,
  plugins: ExtensionOutlinedIcon,
  cloud: CloudOutlinedIcon,
  team: GroupsOutlinedIcon,
  learn: AutoStoriesOutlinedIcon,
  settings: SettingsOutlinedIcon,
};

export interface NavigationDrawerProps {
  activePage: string;
  text: HubShellText;
  engineVersion: string;
  sourceEngines: HubSourceEngineSummary[];
  activeSourceEngineId: string | null;
  onAction: HubActionHandler;
}

export function NavigationDrawer({
  activePage,
  text,
  engineVersion,
  sourceEngines,
  activeSourceEngineId,
  onAction,
}: NavigationDrawerProps) {
  const [collapsed, setCollapsed] = useState(false);
  const compactViewport = useMediaQuery("(max-width:980px)");
  const effectiveCollapsed = collapsed || compactViewport;
  const drawerWidth = effectiveCollapsed ? hubTokens.window.sidebarCollapsedWidth : hubTokens.window.sidebarWidth;
  const collapseLabel = effectiveCollapsed ? text.expand : text.collapse;
  const CollapseIcon = effectiveCollapsed ? KeyboardDoubleArrowRightIcon : KeyboardDoubleArrowLeftIcon;
  const activeEngineId = admittedSourceEngineId(sourceEngines, activeSourceEngineId);
  const activeEngine = sourceEngines.find((engine) => engine.id === activeEngineId);
  const statusColor = activeEngine ? hubTokens.colors.success : hubTokens.colors.warning;
  const statusLabel = activeEngine?.status ?? text.noSourceEngineRegistered;
  const engineLabel = activeEngine?.name ?? engineVersion;

  return (
    <Drawer
      data-testid="hub-navigation-drawer"
      variant="permanent"
      sx={{
        width: drawerWidth,
        flexShrink: 0,
        "& .MuiDrawer-paper": {
          position: "relative",
          width: drawerWidth,
          height: "100%",
          boxSizing: "border-box",
          backgroundImage: "none",
          backgroundColor: "rgba(16,16,16,0.96)",
          borderRight: `1px solid ${hubTokens.colors.line}`,
          overflow: "hidden",
          transition: "width 160ms ease",
        },
      }}
    >
      <Box sx={{ display: "flex", flexDirection: "column", height: "100%", p: 2, gap: 2 }}>
        <Box component="nav" aria-label={text.workspaceProfile}>
          <List id="hub-navigation-items" sx={{ display: "grid", gap: 0.8, p: 0 }}>
            {text.navItems.map(({ id, label }) => {
              const Icon = navIcons[id];
              const selected = activePage === id;
              const button = (
                <ListItemButton
                  key={id}
                  aria-label={effectiveCollapsed ? label : undefined}
                  aria-current={selected ? "page" : undefined}
                  selected={selected}
                  onClick={() => void onAction(HUB_ACTION.showPage, id)}
                  sx={{
                    height: 49,
                    borderRadius: `${hubTokens.radius.panel}px`,
                    color: selected ? hubTokens.colors.text : hubTokens.colors.textSoft,
                    border: `1px solid ${selected ? "rgba(45,212,207,0.34)" : "transparent"}`,
                    backgroundColor: selected ? "rgba(15,99,96,0.56)" : "transparent",
                    justifyContent: effectiveCollapsed ? "center" : "flex-start",
                    px: effectiveCollapsed ? 0 : 1.6,
                    "&.Mui-selected, &.Mui-selected:hover": {
                      backgroundColor: "rgba(15,99,96,0.64)",
                    },
                    "&:hover": {
                      backgroundColor: "rgba(255,255,255,0.045)",
                    },
                  }}
                >
                  <ListItemIcon sx={{ minWidth: effectiveCollapsed ? 0 : 40, color: "inherit", justifyContent: "center" }}>
                    <Icon />
                  </ListItemIcon>
                  <Typography
                    variant="body2"
                    sx={{
                      display: effectiveCollapsed ? "none" : "block",
                      fontWeight: selected ? 700 : 500,
                    }}
                  >
                    {label}
                  </Typography>
                </ListItemButton>
              );
              return effectiveCollapsed ? (
                <Tooltip key={id} title={label} placement="right">
                  {button}
                </Tooltip>
              ) : button;
            })}
          </List>
        </Box>

        <Box sx={{ flex: "1 1 auto" }} />

        <Box
          sx={{
            p: 1.5,
            borderRadius: `${hubTokens.radius.panel}px`,
            border: `1px solid ${hubTokens.colors.lineStrong}`,
            backgroundColor: "rgba(32,32,32,0.62)",
            display: effectiveCollapsed ? "none" : "block",
          }}
        >
          <Typography variant="caption" sx={{ color: hubTokens.colors.text, display: "flex", gap: 0.8, alignItems: "center" }}>
            <Box sx={{ width: 8, height: 8, borderRadius: hubTokens.radius.pill, backgroundColor: statusColor }} />
            {text.engineStatus}
          </Typography>
          <Typography variant="body2" sx={{ mt: 1.2, color: hubTokens.colors.textSoft }}>
            {engineLabel}
          </Typography>
          <Typography variant="caption" sx={{ color: statusColor }}>
            {statusLabel}
          </Typography>
          <Tooltip title={text.checkForUpdatesDetail}>
            <span style={{ display: "block" }}>
              <ButtonBase
                disabled
                sx={{
                  width: "100%",
                  height: 38,
                  mt: 1.4,
                  borderRadius: `${hubTokens.radius.compact}px`,
                  border: `1px solid ${hubTokens.colors.lineStrong}`,
                  color: hubTokens.colors.textMuted,
                  backgroundColor: "rgba(28,28,28,0.7)",
                  cursor: "not-allowed",
                  "&.Mui-disabled": {
                    color: hubTokens.colors.textMuted,
                    opacity: 0.62,
                  },
                }}
              >
                <Typography variant="caption">{text.checkForUpdates}</Typography>
              </ButtonBase>
            </span>
          </Tooltip>
        </Box>

        <ButtonBase
          aria-label={collapseLabel}
          aria-controls="hub-navigation-items"
          aria-expanded={!effectiveCollapsed}
          onClick={() => setCollapsed((current) => !current)}
          sx={{
            height: 42,
          justifyContent: effectiveCollapsed ? "center" : "flex-start",
            gap: 1,
          px: effectiveCollapsed ? 0 : 1,
          color: hubTokens.colors.textSoft,
          borderTop: `1px solid ${hubTokens.colors.line}`,
          display: compactViewport ? "none" : "flex",
          }}
        >
          <CollapseIcon fontSize="small" />
          <Typography variant="body2" sx={{ display: effectiveCollapsed ? "none" : "block" }}>
            {collapseLabel}
          </Typography>
        </ButtonBase>
      </Box>
    </Drawer>
  );
}
