import type { ReactNode } from "react";
import { Box, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface EmptyStateBlockProps {
  title: string;
  detail: string;
  icon?: ReactNode;
}

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
