import type { ReactNode } from "react";
import AddIcon from "@mui/icons-material/Add";
import { Box, Divider, Typography } from "@mui/material";
import { HubButton, HubIconButton } from "../inputs";
import { hubTokens } from "../../theme/tokens";

export function ButtonStatesPanel() {
  return (
    <Box
      sx={{
        display: "grid",
        gridTemplateColumns: "1.1fr 1.1fr 1fr 0.75fr",
        gap: 2.4,
        alignItems: "end",
        p: 2,
        borderTop: `1px solid ${hubTokens.colors.line}`,
        backgroundColor: "rgba(18,18,18,0.9)",
        "@media (max-width: 1100px)": {
          gridTemplateColumns: "1fr 1fr",
        },
      }}
    >
      <StateGroup title="Primary">
        <HubButton tone="primary">Default</HubButton>
        <HubButton tone="primary" sx={{ backgroundColor: "rgba(18,140,137,0.9)" }}>
          Hover
        </HubButton>
        <HubButton tone="primary" sx={{ backgroundColor: "rgba(13,105,103,0.9)" }}>
          Pressed
        </HubButton>
        <HubButton tone="primary" disabled>
          Disabled
        </HubButton>
      </StateGroup>
      <StateGroup title="Secondary">
        <HubButton>Default</HubButton>
        <HubButton sx={{ backgroundColor: "#292929" }}>Hover</HubButton>
        <HubButton sx={{ backgroundColor: "#1c1c1c" }}>Pressed</HubButton>
        <HubButton disabled>Disabled</HubButton>
      </StateGroup>
      <StateGroup title="Tertiary">
        <HubButton tone="tertiary">Default</HubButton>
        <HubButton tone="tertiary" sx={{ backgroundColor: "rgba(33,213,207,0.08)" }}>
          Hover
        </HubButton>
        <HubButton tone="tertiary" sx={{ color: "#12a7a2" }}>
          Pressed
        </HubButton>
        <HubButton tone="tertiary" disabled>
          Disabled
        </HubButton>
      </StateGroup>
      <StateGroup title="Icon">
        {[0, 1, 2, 3].map((index) => (
          <HubIconButton key={index} label={`Icon state ${index + 1}`} disabled={index === 3} selected={index === 0}>
            <AddIcon />
          </HubIconButton>
        ))}
      </StateGroup>
    </Box>
  );
}

function StateGroup({ title, children }: { title: string; children: ReactNode }) {
  return (
    <Box sx={{ minWidth: 0 }}>
      <Typography variant="caption" sx={{ display: "block", color: hubTokens.colors.textSoft, mb: 1 }}>
        {title}
      </Typography>
      <Box sx={{ display: "flex", alignItems: "center", gap: 1.2, minWidth: 0, overflow: "hidden" }}>
        {children}
      </Box>
      <Divider sx={{ display: { xs: "none", lg: "block" }, mt: 1.2, opacity: 0 }} />
    </Box>
  );
}
