import type { PropsWithChildren, ReactNode } from "react";
import { Box, Card, Typography } from "@mui/material";
import { useId } from "react";
import { hubTokens } from "../../theme/tokens";

export interface HubPanelProps extends PropsWithChildren {
  title: string;
  action?: ReactNode;
}

export function HubPanel({ title, action, children }: HubPanelProps) {
  const headingId = useId();

  return (
    <Card
      component="section"
      aria-labelledby={headingId}
      sx={{
        p: 2,
        minWidth: 0,
        overflow: "hidden",
      }}
    >
      <Box sx={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: 2, mb: 1.6 }}>
        <Typography id={headingId} variant="h6" sx={{ flex: "1 1 auto", minWidth: 0, overflowWrap: "anywhere" }}>
          {title}
        </Typography>
        {action ? <Box sx={{ minWidth: 0, maxWidth: "100%", "& .MuiButton-root": { maxWidth: "100%", minWidth: 0, whiteSpace: "normal", height: "auto", minHeight: "2rem" } }}>{action}</Box> : null}
      </Box>
      <Box sx={{ color: hubTokens.colors.textSoft }}>{children}</Box>
    </Card>
  );
}
