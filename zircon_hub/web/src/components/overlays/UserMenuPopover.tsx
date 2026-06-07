import AccountCircleOutlinedIcon from "@mui/icons-material/AccountCircleOutlined";
import AutoStoriesOutlinedIcon from "@mui/icons-material/AutoStoriesOutlined";
import LogoutOutlinedIcon from "@mui/icons-material/LogoutOutlined";
import SettingsOutlinedIcon from "@mui/icons-material/SettingsOutlined";
import { Avatar, Box, ButtonBase, Divider, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";
import type { HubShellText } from "../../types/hub";
import { HubPopover } from "./HubPopover";

export interface UserMenuPopoverProps {
  anchorEl: HTMLElement | null;
  open: boolean;
  userName: string;
  initials: string;
  text: HubShellText;
  onClose: () => void;
  onAction: (actionId: string) => void;
}

export function UserMenuPopover({ anchorEl, open, userName, initials, text, onClose, onAction }: UserMenuPopoverProps) {
  const menuItems = [
    { id: "account", label: text.userAccount, detail: text.userAccountDetail, Icon: AccountCircleOutlinedIcon },
    { id: "preferences", label: text.preferences, detail: text.preferencesDetail, Icon: SettingsOutlinedIcon },
    { id: "documentation", label: text.documentation, detail: text.documentationDetail, Icon: AutoStoriesOutlinedIcon },
    { id: "sign-out", label: text.signOut, detail: text.signOutDetail, Icon: LogoutOutlinedIcon, danger: true, disabled: true },
  ];

  return (
    <HubPopover anchorEl={anchorEl} open={open} width={284} align="right" onClose={onClose}>
      <Box sx={{ display: "grid", gridTemplateColumns: "42px minmax(0, 1fr)", alignItems: "center", gap: 1.1, px: 1, py: 0.8 }}>
        <Avatar sx={{ width: 38, height: 38, bgcolor: "#4b4f52", fontSize: 14 }}>{initials}</Avatar>
        <Box sx={{ minWidth: 0 }}>
          <Typography variant="body2" noWrap sx={{ fontWeight: 700 }}>
            {userName}
          </Typography>
          <Typography variant="caption" noWrap sx={{ display: "block", color: hubTokens.colors.textMuted }}>
            {text.workspaceProfile}
          </Typography>
        </Box>
      </Box>

      <Divider sx={{ my: 0.8, borderColor: hubTokens.colors.line }} />

      <Box sx={{ display: "grid", gap: 0.45 }}>
        {menuItems.map(({ id, label, detail, Icon, danger, disabled }) => {
          const isDisabled = Boolean(disabled);

          return (
            <ButtonBase
              key={id}
              disabled={isDisabled}
              onClick={() => {
                if (isDisabled) {
                  return;
                }
                onAction(id);
                onClose();
              }}
              sx={{
                width: "100%",
                minHeight: 54,
                display: "grid",
                gridTemplateColumns: "34px minmax(0, 1fr)",
                alignItems: "center",
                gap: 1,
                px: 1,
                py: 0.8,
                borderRadius: `${hubTokens.radius.compact}px`,
                color: isDisabled ? hubTokens.colors.textMuted : danger ? hubTokens.colors.error : hubTokens.colors.text,
                textAlign: "left",
                "&:hover": {
                  backgroundColor: isDisabled ? "transparent" : danger ? "rgba(105,31,29,0.24)" : "rgba(255,255,255,0.055)",
                },
                "&.Mui-disabled": {
                  color: hubTokens.colors.textMuted,
                  cursor: "not-allowed",
                  opacity: 0.62,
                },
              }}
            >
              <Icon fontSize="small" />
              <Box sx={{ minWidth: 0 }}>
                <Typography variant="body2" noWrap sx={{ fontWeight: 700, color: "inherit" }}>
                  {label}
                </Typography>
                <Typography
                  variant="caption"
                  noWrap
                  sx={{
                    display: "block",
                    color: isDisabled ? hubTokens.colors.textMuted : danger ? "rgba(255,216,213,0.72)" : hubTokens.colors.textMuted,
                  }}
                >
                  {detail}
                </Typography>
              </Box>
            </ButtonBase>
          );
        })}
      </Box>
    </HubPopover>
  );
}
