import type { ReactNode } from "react";
import ChevronRightIcon from "@mui/icons-material/ChevronRight";
import { Box } from "@mui/material";
import { hubTokens } from "../../theme/tokens";
import { HubIconButton } from "../inputs";

export interface ProjectCardRailProps {
  children: ReactNode;
  hasMore: boolean;
  moreLabel: string;
  onMore: () => void;
}

export function ProjectCardRail({ children, hasMore, moreLabel, onMore }: ProjectCardRailProps) {
  return (
    <Box sx={{ position: "relative" }}>
      <Box
        sx={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fill, minmax(clamp(220px, 22vw, 296px), 1fr))",
          gap: 2,
        }}
      >
        {children}
      </Box>
      {hasMore ? (
        <>
          <Box
            aria-hidden
            sx={{
              pointerEvents: "none",
              position: "absolute",
              top: 0,
              right: 0,
              width: 96,
              height: "100%",
              background: "linear-gradient(90deg, rgba(17,18,18,0), rgba(17,18,18,0.94))",
            }}
          />
          <HubIconButton
            label={moreLabel}
            onClick={onMore}
            sx={{
              position: "absolute",
              top: "50%",
              right: 8,
              transform: "translateY(-50%)",
              width: 38,
              height: 38,
              backgroundColor: hubTokens.colors.panel,
            }}
          >
            <ChevronRightIcon />
          </HubIconButton>
        </>
      ) : null}
    </Box>
  );
}
