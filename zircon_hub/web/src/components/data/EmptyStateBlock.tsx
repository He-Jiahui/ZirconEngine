import type { ReactNode } from "react";
import { Box, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

// 页面须先区分无结果、未选择等状态，再提供相应本地化说明；此契约不携带加载或恢复动作。
export interface EmptyStateBlockProps {
  title: string;
  detail: string;
  icon?: ReactNode;
}

// 共用空状态占位保持页面信息层级，具体为空的原因由调用页决定，不能从图标推断业务状态。
export function EmptyStateBlock({ title, detail, icon }: EmptyStateBlockProps) {
  return (
    <Box
      sx={{
        minHeight: 148,
        display: "grid",
        placeItems: "center",
        gap: 0.9,
        p: 2,
        color: hubTokens.colors.textSoft,
        border: `1px dashed ${hubTokens.colors.lineStrong}`,
        borderRadius: `${hubTokens.radius.panel}px`,
        backgroundColor: "rgba(28,28,28,0.42)",
        textAlign: "center",
      }}
    >
      {icon ? <Box sx={{ color: hubTokens.colors.accent }}>{icon}</Box> : null}
      <Box sx={{ minWidth: 0 }}>
        <Typography variant="body2" sx={{ color: hubTokens.colors.text, fontWeight: 700 }}>
          {title}
        </Typography>
        <Typography variant="caption" sx={{ color: hubTokens.colors.textMuted }}>
          {detail}
        </Typography>
      </Box>
    </Box>
  );
}
