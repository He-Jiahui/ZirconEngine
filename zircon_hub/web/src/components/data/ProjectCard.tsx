import MoreVertIcon from "@mui/icons-material/MoreVert";
import { Box, Card, CardActionArea, Chip, IconButton, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";
import type { HubProjectSummary } from "../../types/hub";
import { ProjectCover } from "./ProjectCover";

export interface ProjectCardProps {
  project: HubProjectSummary;
  selected?: boolean;
  onOpen?: (project: HubProjectSummary) => void;
}

export function ProjectCard({ project, selected = false, onOpen }: ProjectCardProps) {
  return (
    <Card
      sx={{
        height: 251,
        minWidth: 0,
        borderColor: selected ? "rgba(45,212,207,0.44)" : hubTokens.colors.lineStrong,
        transition: "border-color 140ms ease, transform 140ms ease",
        "&:hover": {
          borderColor: "rgba(45,212,207,0.4)",
          transform: "translateY(-1px)",
        },
      }}
    >
      <CardActionArea
        onClick={() => onOpen?.(project)}
        sx={{ height: "100%", p: 1.2, display: "flex", flexDirection: "column", alignItems: "stretch" }}
      >
        <Box sx={{ height: 112, borderRadius: "6px", position: "relative", overflow: "hidden" }}>
          <ProjectCover coverId={project.coverId} />
          <IconButton
            size="small"
            aria-label={`${project.name} menu`}
            sx={{
              position: "absolute",
              top: 6,
              right: 6,
              width: 30,
              height: 30,
              color: hubTokens.colors.textSoft,
              backgroundColor: "rgba(15,15,15,0.76)",
              "&:hover": { backgroundColor: "rgba(25,25,25,0.9)" },
            }}
          >
            <MoreVertIcon fontSize="small" />
          </IconButton>
        </Box>
        <Box sx={{ pt: 1.2, minWidth: 0 }}>
          <Typography variant="h6" noWrap>
            {project.name}
          </Typography>
          <Typography variant="body2" color="text.secondary" noWrap sx={{ mt: 0.4 }}>
            {project.path}
          </Typography>
          <Typography variant="body2" color="text.disabled" noWrap sx={{ mt: 0.4 }}>
            {project.modified}
          </Typography>
          <Box sx={{ display: "flex", gap: 0.8, mt: 1.1 }}>
            <Chip label={project.engineVersion} size="small" sx={chipSx("accent")} />
            <Chip label={project.platform} size="small" sx={chipSx("neutral")} />
          </Box>
        </Box>
      </CardActionArea>
    </Card>
  );
}

function chipSx(tone: "accent" | "neutral") {
  return {
    height: 24,
    color: tone === "accent" ? hubTokens.colors.accent : hubTokens.colors.textSoft,
    backgroundColor: tone === "accent" ? "rgba(11,112,109,0.42)" : "rgba(255,255,255,0.07)",
    border: `1px solid ${tone === "accent" ? "rgba(45,212,207,0.22)" : hubTokens.colors.line}`,
    borderRadius: "6px",
    "& .MuiChip-label": { px: 1 },
  };
}
