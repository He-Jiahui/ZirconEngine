import CloseIcon from "@mui/icons-material/Close";
import CropSquareIcon from "@mui/icons-material/CropSquare";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import HelpOutlinedIcon from "@mui/icons-material/HelpOutlined";
import MinimizeIcon from "@mui/icons-material/Minimize";
import NotificationsNoneIcon from "@mui/icons-material/NotificationsNone";
import SettingsIcon from "@mui/icons-material/Settings";
import StorageOutlinedIcon from "@mui/icons-material/StorageOutlined";
import { Avatar, Box, ButtonBase, Divider, Typography } from "@mui/material";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useState } from "react";
import { brandMark } from "../../data/hubData";
import { hubTokens } from "../../theme/tokens";
import type { HubActionHandler, HubShellState } from "../../types/hub";
import { HUB_ACTION } from "../../types/hub";
import { StatusBadge } from "../data";
import { HubIconButton } from "../inputs";
import { SourceEnginePopover, UserMenuPopover } from "../overlays";

export interface TopBarProps {
  state: HubShellState;
  onAction: HubActionHandler;
}

export function TopBar({ state, onAction }: TopBarProps) {
  const [engineAnchor, setEngineAnchor] = useState<HTMLElement | null>(null);
  const [userAnchor, setUserAnchor] = useState<HTMLElement | null>(null);
  const activeEngine =
    state.sourceEngines.find((engine) => engine.id === state.activeSourceEngineId) ??
    state.sourceEngines.find((engine) => engine.active);
  const engineLabel = activeEngine?.name ?? state.engineVersion;
  const userName = state.team.identityName || state.ui.common.notConfigured;
  const userInitials = initialsFromName(userName);
  const notificationDetail = comingSoonDetail(state, "notification-center");
  const signOutDetail = comingSoonDetail(state, "sign-out");
  const handleMinimize = () => runWindowAction((appWindow) => appWindow.minimize());
  const handleToggleMaximize = () => runWindowAction((appWindow) => appWindow.toggleMaximize());
  const handleClose = () => runWindowAction((appWindow) => appWindow.close());

  const handleUserAction = (actionId: string) => {
    if (actionId === "preferences") {
      void onAction(HUB_ACTION.showPage, "settings");
      return;
    }
    if (actionId === "documentation") {
      void onAction(HUB_ACTION.showPage, "learn");
      return;
    }
    if (actionId === "account") {
      void onAction(HUB_ACTION.showPage, "team");
      return;
    }
  };

  return (
    <Box
      component="header"
      sx={{
        height: hubTokens.window.topBarHeight,
        display: "grid",
        gridTemplateColumns: "222px minmax(0, 1fr) auto",
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
            {state.ui.shell.productCategory}
          </Typography>
        </Box>
      </Box>

      <Box sx={{ display: "flex", alignItems: "center", gap: 1.2, minWidth: 0, overflow: "hidden" }}>
        <ButtonBase
          onClick={(event) => setEngineAnchor(event.currentTarget)}
          sx={{
            width: 190,
            minWidth: 160,
            height: 38,
            display: "grid",
            gridTemplateColumns: "24px minmax(0, 1fr) 20px",
            alignItems: "center",
            gap: 0.8,
            px: 1.2,
            color: hubTokens.colors.text,
            border: `1px solid ${engineAnchor ? "rgba(45,212,207,0.48)" : hubTokens.colors.lineStrong}`,
            borderRadius: `${hubTokens.radius.compact}px`,
            backgroundColor: engineAnchor ? "rgba(18,82,80,0.38)" : "rgba(31,31,31,0.78)",
            textAlign: "left",
            "&:hover": {
              borderColor: "rgba(45,212,207,0.36)",
              backgroundColor: "rgba(38,38,38,0.86)",
            },
          }}
        >
          <StorageOutlinedIcon sx={{ color: hubTokens.colors.accent, fontSize: 20 }} />
          <Typography variant="body2" noWrap>
            {engineLabel}
          </Typography>
          <ExpandMoreIcon sx={{ color: hubTokens.colors.textSoft, fontSize: 18 }} />
        </ButtonBase>
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
          {state.demoMode ? <StatusBadge label={state.ui.shell.demoModeBadge} tone="warning" /> : null}
        </Box>
      </Box>

      <Box sx={{ display: "flex", alignItems: "center", gap: 1, pr: 1.4 }}>
        <Box sx={{ display: "flex", gap: 0.5, "@media (max-width: 1180px)": { display: "none" } }}>
          <HubIconButton label={state.ui.shell.notifications} tooltip={notificationDetail} disabled sx={topIconSx}>
            <NotificationsNoneIcon />
          </HubIconButton>
          <HubIconButton label={state.ui.shell.help} onClick={() => void onAction(HUB_ACTION.showPage, "learn")} sx={topIconSx}>
            <HelpOutlinedIcon />
          </HubIconButton>
          <HubIconButton label={state.ui.shell.settings} onClick={() => void onAction(HUB_ACTION.showPage, "settings")} sx={topIconSx}>
            <SettingsIcon />
          </HubIconButton>
        </Box>
        <Divider orientation="vertical" flexItem sx={{ mx: 0.7, borderColor: hubTokens.colors.line }} />
        <ButtonBase
          onClick={(event) => setUserAnchor(event.currentTarget)}
          sx={{
            height: 42,
            display: "flex",
            alignItems: "center",
            gap: 0.9,
            minWidth: 0,
            px: 0.6,
            borderRadius: `${hubTokens.radius.compact}px`,
            color: hubTokens.colors.text,
            border: `1px solid ${userAnchor ? "rgba(45,212,207,0.48)" : "transparent"}`,
            "&:hover": { backgroundColor: "rgba(255,255,255,0.045)" },
          }}
        >
          <Avatar sx={{ width: 36, height: 36, bgcolor: hubTokens.colors.avatar, fontSize: 14 }}>{userInitials}</Avatar>
          <Typography variant="body2" noWrap sx={{ maxWidth: 126, "@media (max-width: 1180px)": { display: "none" } }}>
            {userName}
          </Typography>
          <ExpandMoreIcon sx={{ fontSize: 18, color: hubTokens.colors.textSoft }} />
        </ButtonBase>
        <Divider orientation="vertical" flexItem sx={{ mx: 0.7, borderColor: hubTokens.colors.line }} />
        <Box sx={{ display: "flex", gap: 0.2 }}>
          <HubIconButton label={state.ui.shell.minimize} onClick={handleMinimize} sx={windowIconSx}>
            <MinimizeIcon fontSize="small" />
          </HubIconButton>
          <HubIconButton label={state.ui.shell.maximize} onClick={handleToggleMaximize} sx={windowIconSx}>
            <CropSquareIcon fontSize="small" />
          </HubIconButton>
          <HubIconButton label={state.ui.shell.close} onClick={handleClose} sx={windowIconSx}>
            <CloseIcon fontSize="small" />
          </HubIconButton>
        </Box>
      </Box>
      <SourceEnginePopover
        anchorEl={engineAnchor}
        open={Boolean(engineAnchor)}
        engines={state.sourceEngines}
        activeEngineId={state.activeSourceEngineId}
        settings={state.settings}
        text={state.ui.shell}
        onClose={() => setEngineAnchor(null)}
        onSelect={(engineId) => {
          setEngineAnchor(null);
          void onAction(HUB_ACTION.selectEngine, engineId);
        }}
        onManage={() => {
          setEngineAnchor(null);
          void onAction(HUB_ACTION.showPage, "settings");
        }}
      />
      <UserMenuPopover
        anchorEl={userAnchor}
        open={Boolean(userAnchor)}
        initials={userInitials}
        userName={userName}
        text={state.ui.shell}
        signOutDetail={signOutDetail}
        onClose={() => setUserAnchor(null)}
        onAction={handleUserAction}
      />
    </Box>
  );
}

function comingSoonDetail(state: HubShellState, id: string): string {
  return state.comingSoon.find((entry) => entry.id === id)?.detail ?? "";
}

function initialsFromName(name: string): string {
  const initials = name
    .trim()
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0])
    .join("")
    .toUpperCase();

  return initials || "ZH";
}

type TauriWindow = ReturnType<typeof getCurrentWindow>;

function runWindowAction(action: (appWindow: TauriWindow) => Promise<void>) {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
    return;
  }

  void action(getCurrentWindow());
}

const topIconSx = {
  width: 38,
  height: 38,
  backgroundColor: "transparent",
  borderColor: "transparent",
  color: hubTokens.colors.textSoft,
  "&.Mui-disabled": {
    color: hubTokens.colors.textMuted,
    backgroundColor: "transparent",
    borderColor: "transparent",
  },
};

const windowIconSx = {
  width: 36,
  height: 34,
  backgroundColor: "transparent",
  borderColor: "transparent",
  color: hubTokens.colors.textSoft,
};
