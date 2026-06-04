import type { PropsWithChildren, ReactNode } from "react";
import { Box, Card, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface HubPanelProps extends PropsWithChildren {
  title: string;
  action?: ReactNode;
}

export function HubPanel({ title, action, children }: HubPanelProps) {
  return (
    <Card
      component="section"
      sx={{
        p: 2,
        minWidth: 0,
        overflow: "hidden",
      }}
    >
      <Box sx={{ display: "flex", alignItems: "center", gap: 2, mb: 1.6 }}>
        <Typography variant="h6" sx={{ flex: "1 1 auto" }}>
          {title}
        </Typography>
        {action}
      </Box>
      <Box sx={{ color: hubTokens.colors.textSoft }}>{children}</Box>
    </Card>
  );
}
