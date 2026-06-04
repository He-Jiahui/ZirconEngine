import AutoStoriesOutlinedIcon from "@mui/icons-material/AutoStoriesOutlined";
import CloudOutlinedIcon from "@mui/icons-material/CloudOutlined";
import ConstructionOutlinedIcon from "@mui/icons-material/ConstructionOutlined";
import ExtensionOutlinedIcon from "@mui/icons-material/ExtensionOutlined";
import FolderOutlinedIcon from "@mui/icons-material/FolderOutlined";
import GroupsOutlinedIcon from "@mui/icons-material/GroupsOutlined";
import Inventory2OutlinedIcon from "@mui/icons-material/Inventory2Outlined";
import KeyboardDoubleArrowLeftIcon from "@mui/icons-material/KeyboardDoubleArrowLeft";
import SettingsOutlinedIcon from "@mui/icons-material/SettingsOutlined";
import WebAssetOutlinedIcon from "@mui/icons-material/WebAssetOutlined";
import { Box, ButtonBase, Drawer, List, ListItemButton, ListItemIcon, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

const navItems = [
  { id: "projects", label: "Projects", Icon: FolderOutlinedIcon },
  { id: "editor", label: "Editor", Icon: WebAssetOutlinedIcon },
  { id: "assets", label: "Assets", Icon: Inventory2OutlinedIcon },
  { id: "builds", label: "Builds", Icon: ConstructionOutlinedIcon },
  { id: "plugins", label: "Plugins", Icon: ExtensionOutlinedIcon },
  { id: "cloud", label: "Cloud", Icon: CloudOutlinedIcon },
  { id: "team", label: "Team", Icon: GroupsOutlinedIcon },
  { id: "learn", label: "Learn", Icon: AutoStoriesOutlinedIcon },
  { id: "settings", label: "Settings", Icon: SettingsOutlinedIcon },
];

export interface NavigationDrawerProps {
  activePage: string;
}

export function NavigationDrawer({ activePage }: NavigationDrawerProps) {
  return (
    <Drawer
      variant="permanent"
      sx={{
        width: hubTokens.window.sidebarWidth,
        flexShrink: 0,
        "& .MuiDrawer-paper": {
          position: "relative",
          width: hubTokens.window.sidebarWidth,
          height: "100%",
          boxSizing: "border-box",
          backgroundImage: "none",
          backgroundColor: "rgba(16,16,16,0.96)",
          borderRight: `1px solid ${hubTokens.colors.line}`,
          overflow: "hidden",
          "@media (max-width: 980px)": {
            width: hubTokens.window.sidebarCollapsedWidth,
          },
        },
      }}
    >
      <Box sx={{ display: "flex", flexDirection: "column", height: "100%", p: 2, gap: 2 }}>
        <List sx={{ display: "grid", gap: 0.8, p: 0 }}>
          {navItems.map(({ id, label, Icon }) => {
            const selected = activePage === id;
            return (
              <ListItemButton
                key={id}
                selected={selected}
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
            "@media (max-width: 980px)": { display: "none" },
          }}
        >
          <Typography variant="caption" sx={{ color: hubTokens.colors.text, display: "flex", gap: 0.8, alignItems: "center" }}>
            <Box sx={{ width: 8, height: 8, borderRadius: 999, backgroundColor: hubTokens.colors.success }} />
            Engine Status
          </Typography>
          <Typography variant="body2" sx={{ mt: 1.2, color: hubTokens.colors.textSoft }}>
            Zircon Engine 1.8.2
          </Typography>
          <Typography variant="caption" sx={{ color: hubTokens.colors.success }}>
            Up to date
          </Typography>
          <ButtonBase
            sx={{
              width: "100%",
              height: 38,
              mt: 1.4,
              borderRadius: `${hubTokens.radius.compact}px`,
              border: `1px solid ${hubTokens.colors.lineStrong}`,
              color: hubTokens.colors.textSoft,
              backgroundColor: "rgba(28,28,28,0.7)",
            }}
          >
            <Typography variant="caption">Check for Updates</Typography>
          </ButtonBase>
        </Box>

        <ButtonBase
          sx={{
            height: 42,
            justifyContent: "flex-start",
            gap: 1,
            px: 1,
            color: hubTokens.colors.textSoft,
            borderTop: `1px solid ${hubTokens.colors.line}`,
          }}
        >
          <KeyboardDoubleArrowLeftIcon fontSize="small" />
          <Typography variant="body2" sx={{ "@media (max-width: 980px)": { display: "none" } }}>
            Collapse
          </Typography>
        </ButtonBase>
      </Box>
    </Drawer>
  );
}
