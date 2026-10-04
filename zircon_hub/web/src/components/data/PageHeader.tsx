import { Box, Typography } from "@mui/material";
import type { ReactNode } from "react";

interface PageHeaderProps {
  title: string;
  subtitle: string;
  actions?: ReactNode;
}

export function PageHeader({ title, subtitle, actions }: PageHeaderProps) {
  return (
    <Box
      data-testid="hub-page-header"
      sx={{
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "space-between",
        gap: 2,
        mb: 2.3,
        "& .hub-page-header-title": {
          minWidth: 0,
        },
        "& .hub-page-header-actions": {
          display: "flex",
          gap: 1.2,
          flexWrap: "wrap",
          justifyContent: "flex-end",
          minWidth: 0,
        },
        "@media (max-width: 980px)": {
          display: "grid",
          gridTemplateColumns: "minmax(0, 1fr)",
          gap: 1.4,
          "& .hub-page-header-actions": {
            width: "100%",
            justifyContent: "flex-start",
            "& > *": {
              minWidth: 0,
              flex: "1 1 160px",
              maxWidth: "100%",
            },
          },
        },
        "@media (max-width: 760px)": {
          "& h4": { fontSize: 24, lineHeight: 1.4 },
        },
      }}
    >
      <Box className="hub-page-header-title">
        <Typography variant="h4">{title}</Typography>
        <Typography variant="body1" color="text.secondary" sx={{ mt: 0.9 }}>
          {subtitle}
        </Typography>
      </Box>
      {actions ? <Box className="hub-page-header-actions">{actions}</Box> : null}
    </Box>
  );
}
