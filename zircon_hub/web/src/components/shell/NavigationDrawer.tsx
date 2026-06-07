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
import { Box, ButtonBase, Drawer, List, ListItemButton, ListItemIcon, Tooltip, Typography } from "@mui/material";
import { useState } from "react";
import { hubTokens } from "../../theme/tokens";
import type { HubActionHandler, HubPageId, HubShellText } from "../../types/hub";
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
  onAction: HubActionHandler;
}

export function NavigationDrawer({ activePage, text, engineVersion, onAction }: NavigationDrawerProps) {
  const [collapsed, setCollapsed] = useState(false);
  const drawerWidth = collapsed ? hubTokens.window.sidebarCollapsedWidth : hubTokens.window.sidebarWidth;
  const collapseLabel = collapsed ? text.expand : text.collapse;
  const CollapseIcon = collapsed ? KeyboardDoubleArrowRightIcon : KeyboardDoubleArrowLeftIcon;

  return (
    <Drawer
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
          "@media (max-width: 980px)": {
            width: hubTokens.window.sidebarCollapsedWidth,
          },
        },
      }}
    >
      <Box sx={{ display: "flex", flexDirection: "column", height: "100%", p: 2, gap: 2 }}>
        <List sx={{ display: "grid", gap: 0.8, p: 0 }}>
          {text.navItems.map(({ id, label }) => {
            const Icon = navIcons[id];
            const selected = activePage === id;
            return (
              <ListItemButton
                key={id}
                selected={selected}
                onClick={() => void onAction(HUB_ACTION.showPage, id)}
                sx={{
                  height: 49,
                  borderRadius: `${hubTokens.radius.panel}px`,
                  color: selected ? hubTokens.colors.text : hubTokens.colors.textSoft,
                  border: `1px solid ${selected ? "rgba(45,212,207,0.34)" : "transparent"}`,
                  backgroundColor: selected ? "rgba(15,99,96,0.56)" : "transparent",
                  "&.Mui-selected, &.Mui-selected:hover": {
                    backgroundColor: "rgba(15,99,96,0.64)",
                  },
                  "&:hover": {
                    backgroundColor: "rgba(255,255,255,0.045)",
                  },
                }}
              >
                <ListItemIcon sx={{ minWidth: 40, color: "inherit" }}>
                  <Icon />
                </ListItemIcon>
                <Typography
                  variant="body2"
                  sx={{
                    display: collapsed ? "none" : "block",
                    fontWeight: selected ? 700 : 500,
                    "@media (max-width: 980px)": { display: "none" },
                  }}
                >
                  {label}
                </Typography>
              </ListItemButton>
            );
          })}
        </List>

        <Box sx={{ flex: "1 1 auto" }} />

        <Box
          sx={{
            p: 1.5,
            borderRadius: `${hubTokens.radius.panel}px`,
            border: `1px solid ${hubTokens.colors.lineStrong}`,
            backgroundColor: "rgba(32,32,32,0.62)",
            display: collapsed ? "none" : "block",
            "@media (max-width: 980px)": { display: "none" },
          }}
        >
          <Typography variant="caption" sx={{ color: hubTokens.colors.text, display: "flex", gap: 0.8, alignItems: "center" }}>
            <Box sx={{ width: 8, height: 8, borderRadius: 999, backgroundColor: hubTokens.colors.success }} />
            {text.engineStatus}
          </Typography>
          <Typography variant="body2" sx={{ mt: 1.2, color: hubTokens.colors.textSoft }}>
            {engineVersion}
          </Typography>
          <Typography variant="caption" sx={{ color: hubTokens.colors.success }}>
            {text.upToDate}
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
          onClick={() => setCollapsed((current) => !current)}
          sx={{
            height: 42,
            justifyContent: collapsed ? "center" : "flex-start",
            gap: 1,
            px: collapsed ? 0 : 1,
            color: hubTokens.colors.textSoft,
            borderTop: `1px solid ${hubTokens.colors.line}`,
          }}
        >
          <CollapseIcon fontSize="small" />
          <Typography variant="body2" sx={{ display: collapsed ? "none" : "block", "@media (max-width: 980px)": { display: "none" } }}>
            {collapseLabel}
          </Typography>
        </ButtonBase>
      </Box>
    </Drawer>
  );
}
