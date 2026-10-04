import BuildIcon from "@mui/icons-material/Build";
import ChevronRightIcon from "@mui/icons-material/ChevronRight";
import Inventory2Icon from "@mui/icons-material/Inventory2";
import OpenInNewIcon from "@mui/icons-material/OpenInNew";
import PhoneIphoneIcon from "@mui/icons-material/PhoneIphone";
import { Box, ButtonBase, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";
import type { HubQuickAction } from "../../types/hub";

// 动作可用性由快照投影给出，页面回调负责携带明确目标并接入统一错误反馈；展示列表不选择项目。
export interface QuickActionsProps {
  actions: HubQuickAction[];
  onAction?: (action: HubQuickAction) => void;
}

// 图标仅区分入口类别，未知图标的视觉回退不改变动作身份或授权。
const actionIcons = {
  build: BuildIcon,
  device: PhoneIphoneIcon,
  package: Inventory2Icon,
  editor: OpenInNewIcon,
};

// 展示跨页面共用的快捷入口；可用标记只是当前快照的交互门禁，后端执行前仍须重验目标和环境。
export function QuickActions({ actions, onAction }: QuickActionsProps) {
  return (
    <Box sx={{ display: "grid", gap: 0.75 }}>
      {actions.map((action) => {
        const Icon = actionIcons[action.icon as keyof typeof actionIcons] ?? BuildIcon;
        return (
          <ButtonBase
            key={action.id}
            disabled={!action.enabled}
            onClick={() => {
              if (action.enabled) {
                onAction?.(action);
              }
            }}
            sx={{
              minWidth: 0,
              minHeight: 55,
              display: "grid",
              gridTemplateColumns: "36px minmax(0, 1fr) 24px",
              alignItems: "center",
              gap: 1.2,
              px: 1.2,
              py: 0.9,
              color: hubTokens.colors.text,
              border: `1px solid ${hubTokens.colors.lineStrong}`,
              borderRadius: `${hubTokens.radius.compact}px`,
              backgroundColor: "rgba(32,32,32,0.64)",
              textAlign: "left",
              "&:hover": {
                backgroundColor: "rgba(40,40,40,0.82)",
                borderColor: "rgba(45,212,207,0.26)",
              },
              "&.Mui-disabled": {
                opacity: 0.48,
                color: hubTokens.colors.textMuted,
                cursor: "not-allowed",
              },
            }}
          >
            <Icon sx={{ fontSize: 25, color: hubTokens.colors.textSoft }} />
            <Box sx={{ minWidth: 0 }}>
              <Typography variant="body2" noWrap sx={{ fontWeight: 700, color: hubTokens.colors.text }}>
                {action.title}
              </Typography>
              <Typography variant="caption" noWrap sx={{ display: "block", color: hubTokens.colors.textMuted }}>
                {action.detail}
              </Typography>
            </Box>
            <ChevronRightIcon sx={{ color: hubTokens.colors.textSoft }} />
          </ButtonBase>
        );
      })}
    </Box>
  );
}
