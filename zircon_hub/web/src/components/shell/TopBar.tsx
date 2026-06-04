import CloseIcon from "@mui/icons-material/Close";
import CropSquareIcon from "@mui/icons-material/CropSquare";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import HelpOutlinedIcon from "@mui/icons-material/HelpOutlined";
import MinimizeIcon from "@mui/icons-material/Minimize";
import NotificationsNoneIcon from "@mui/icons-material/NotificationsNone";
import SettingsIcon from "@mui/icons-material/Settings";
import { Avatar, Box, Divider, Typography } from "@mui/material";
import { brandMark } from "../../data/hubData";
import { hubTokens } from "../../theme/tokens";
import type { HubShellState } from "../../types/hub";
import { StatusBadge } from "../data";
import { HubIconButton, HubSelect } from "../inputs";

export interface TopBarProps {
  state: HubShellState;
}

export function TopBar({ state }: TopBarProps) {
  return (
    <Box
      component="header"
      sx={{
        height: hubTokens.window.topBarHeight,
        display: "grid",
        gridTemplateColumns: "222px minmax(250px, 1fr) auto",
        alignItems: "center",
        borderBottom: `1px solid ${hubTokens.colors.line}`,
        backgroundColor: "rgba(17,17,17,0.96)",
        "@media (max-width: 980px)": {
          gridTemplateColumns: "78px minmax(0, 1fr) auto",
        },
      }}
    >
      <Box sx={{ display: "flex", alignItems: "center", gap: 1.2, px: 3, minWidth: 0 }}>
        <Box component="img" src={brandMark} alt="" sx={{ width: 36, height: 36, flex: "0 0 auto" }} />
        <Box sx={{ minWidth: 0, "@media (max-width: 980px)": { display: "none" } }}>
          <Typography variant="h6" noWrap sx={{ textTransform: "uppercase", lineHeight: 1 }}>
            {state.productName}
          </Typography>
          <Typography variant="body2" noWrap color="text.secondary" sx={{ mt: 0.3 }}>
            Game Engine
          </Typography>
        </Box>
      </Box>

      <Box sx={{ display: "flex", alignItems: "center", gap: 1.2, minWidth: 0, overflow: "hidden" }}>
        <HubSelect
          value={state.engineVersion}
          minWidth={176}
          options={[{ value: state.engineVersion, label: state.engineVersion }]}
          onChange={() => undefined}
        />
        <Box
          sx={{
            display: "flex",
            gap: 1.2,
            minWidth: 0,
            overflow: "hidden",
            "@media (max-width: 1260px)": { display: "none" },
          }}
        >
          {state.taskStatus.map((status) => (
            <StatusBadge key={status.id} label={status.label} tone={status.tone} />
          ))}
        </Box>
      </Box>

      <Box sx={{ display: "flex", alignItems: "center", gap: 1, pr: 1.4 }}>
        <Box sx={{ display: "flex", gap: 0.5, "@media (max-width: 1180px)": { display: "none" } }}>
          <HubIconButton label="Notifications" sx={topIconSx}>
            <NotificationsNoneIcon />
          </HubIconButton>
          <HubIconButton label="Help" sx={topIconSx}>
            <HelpOutlinedIcon />
          </HubIconButton>
          <HubIconButton label="Settings" sx={topIconSx}>
            <SettingsIcon />
          </HubIconButton>
        </Box>
        <Divider orientation="vertical" flexItem sx={{ mx: 0.7, borderColor: hubTokens.colors.line }} />
        <Box sx={{ display: "flex", alignItems: "center", gap: 0.9, minWidth: 0 }}>
          <Avatar sx={{ width: 36, height: 36, bgcolor: "#4b4f52", fontSize: 14 }}>AD</Avatar>
          <Typography variant="body2" noWrap sx={{ maxWidth: 126, "@media (max-width: 1180px)": { display: "none" } }}>
            Alex Developer
          </Typography>
          <ExpandMoreIcon sx={{ fontSize: 18, color: hubTokens.colors.textSoft }} />
        </Box>
        <Divider orientation="vertical" flexItem sx={{ mx: 0.7, borderColor: hubTokens.colors.line }} />
        <Box sx={{ display: "flex", gap: 0.2 }}>
          <HubIconButton label="Minimize" sx={windowIconSx}>
            <MinimizeIcon fontSize="small" />
          </HubIconButton>
          <HubIconButton label="Maximize" sx={windowIconSx}>
            <CropSquareIcon fontSize="small" />
          </HubIconButton>
          <HubIconButton label="Close" sx={windowIconSx}>
            <CloseIcon fontSize="small" />
          </HubIconButton>
        </Box>
      </Box>
    </Box>
  );
}

const topIconSx = {
  width: 38,
  height: 38,
  backgroundColor: "transparent",
  borderColor: "transparent",
  color: hubTokens.colors.textSoft,
};

const windowIconSx = {
  width: 36,
  height: 34,
  backgroundColor: "transparent",
  borderColor: "transparent",
  color: hubTokens.colors.textSoft,
};
