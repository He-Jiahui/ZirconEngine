import type { IconButtonProps, SxProps, Theme } from "@mui/material";
import { IconButton, Tooltip } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

// 图标入口必须提供本地化可访问名称；提示可解释禁用原因，选中标记只控制外观，切换语义由调用方另行提供。
export interface HubIconButtonProps extends IconButtonProps {
  selected?: boolean;
  label: string;
  tooltip?: string;
}

// 共用图标按钮用于窗口和页内动作；外层提示容器使禁用入口仍可说明原因，事件与禁用状态由页面控制。
export function HubIconButton({ selected = false, label, tooltip, sx, ...props }: HubIconButtonProps) {
  return (
    <Tooltip title={tooltip ?? label}>
      <span style={{ display: "inline-flex" }}>
        <IconButton
          {...props}
          aria-label={label}
          sx={[
            {
              width: 50,
              height: 42,
              color: selected ? hubTokens.colors.textOnAccent : hubTokens.colors.textSoft,
              backgroundColor: selected ? "rgba(9,94,91,0.56)" : "rgba(31,31,31,0.72)",
              border: `1px solid ${selected ? "rgba(45,212,207,0.48)" : hubTokens.colors.lineStrong}`,
              "&:hover": {
                color: hubTokens.colors.text,
                backgroundColor: selected ? "rgba(11,112,109,0.68)" : hubTokens.colors.panelHover,
              },
              "&.Mui-disabled": {
                color: hubTokens.colors.textMuted,
                backgroundColor: "transparent",
                borderColor: "transparent",
              },
            },
            ...asSxArray(sx),
          ]}
        />
      </span>
    </Tooltip>
  );
}

// 按调用方原有顺序追加布局样式，兼容主题函数与样式数组而不覆盖按钮的可访问名称。
function asSxArray(sx?: SxProps<Theme>) {
  if (!sx) {
    return [];
  }
  return Array.isArray(sx) ? sx : [sx];
}
