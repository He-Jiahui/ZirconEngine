import type { ButtonProps, SxProps, Theme } from "@mui/material";
import { Button } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

// 色调表达入口层级和危险程度，不能替代调用方的禁用、确认或权限判断。
export type HubButtonTone = "primary" | "secondary" | "tertiary" | "danger";

// 调用方保留原按钮事件及表单属性，只把视觉变体收敛到公共入口样式；文案由页面本地化。
export interface HubButtonProps extends Omit<ButtonProps, "variant"> {
  tone?: HubButtonTone;
}

// 将入口层级映射到同一套视觉语义，删除确认等危险动作仍由业务调用链控制。
const toneStyles: Record<HubButtonTone, SxProps<Theme>> = {
  primary: {
    color: hubTokens.colors.textOnAccent,
    backgroundColor: hubTokens.colors.accentDim,
    borderColor: "rgba(45, 212, 207, 0.48)",
    "&:hover": {
      backgroundColor: "rgba(17, 127, 124, 0.92)",
      borderColor: "rgba(45, 212, 207, 0.68)",
    },
  },
  secondary: {
    color: hubTokens.colors.text,
    backgroundColor: "rgba(32,32,32,0.82)",
    borderColor: hubTokens.colors.lineStrong,
    "&:hover": {
      backgroundColor: hubTokens.colors.panelHover,
      borderColor: "rgba(255,255,255,0.22)",
    },
  },
  tertiary: {
    color: hubTokens.colors.accent,
    backgroundColor: "transparent",
    borderColor: "transparent",
    "&:hover": {
      backgroundColor: "rgba(33,213,207,0.08)",
      borderColor: "transparent",
    },
  },
  danger: {
    color: hubTokens.colors.dangerText,
    backgroundColor: "rgba(120,25,25,0.54)",
    borderColor: "rgba(245,111,102,0.48)",
    "&:hover": {
      backgroundColor: "rgba(142,30,29,0.72)",
    },
  },
};

// 为页面和对话框统一按钮外观；页面样式最后追加，以便只覆盖所在布局需要的尺寸和间距。
export function HubButton({ tone = "secondary", sx, ...props }: HubButtonProps) {
  return (
    <Button
      {...props}
      variant="contained"
      sx={[
        {
          border: "1px solid",
          px: 2.5,
          minWidth: 0,
        },
        toneStyles[tone],
        ...asSxArray(sx),
      ]}
    />
  );
}

// 保留调用方样式对象、函数及数组的原有顺序，避免包装器改变样式组合的优先级。
function asSxArray(sx?: SxProps<Theme>) {
  if (!sx) {
    return [];
  }
  return Array.isArray(sx) ? sx : [sx];
}
