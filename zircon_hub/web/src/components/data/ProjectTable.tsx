import type { ReactNode } from "react";
import MoreVertIcon from "@mui/icons-material/MoreVert";
import { Box, IconButton, Table, TableBody, TableCell, TableHead, TableRow, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";
import type { HubRecentProject } from "../../types/hub";
import { ProjectCover } from "./ProjectCover";

export interface ProjectTableProps {
  projects: HubRecentProject[];
  selectedProjectId: string | null;
  labels: {
    name: string;
    engineVersion: string;
    lastModified: string;
    location: string;
    openDetails: string;
  };
  onSelect?: (project: HubRecentProject) => void;
  onOpenDetail?: (project: HubRecentProject) => void;
  onRowMenu?: (project: HubRecentProject, anchor: HTMLElement) => void;
}

export function ProjectTable({ projects, selectedProjectId, labels, onSelect, onOpenDetail, onRowMenu }: ProjectTableProps) {
  return (
    <Box sx={{ overflowX: "auto", minWidth: 0 }}>
      <Table size="small" sx={{ tableLayout: "fixed", minWidth: 560 }}>
        <TableHead>
          <TableRow>
            <HeaderCell width="32%">{labels.name}</HeaderCell>
            <HeaderCell width="18%">{labels.engineVersion}</HeaderCell>
            <HeaderCell width="16%">{labels.lastModified}</HeaderCell>
            <HeaderCell>{labels.location}</HeaderCell>
            <HeaderCell width={42} />
          </TableRow>
        </TableHead>
        <TableBody>
          {projects.map((project) => {
            const selected = project.id === selectedProjectId;
            return (
              <TableRow
                key={project.id}
                hover
                selected={selected}
                onClick={() => onSelect?.(project)}
                sx={{
                  height: 36,
                  cursor: onSelect ? "pointer" : "default",
                  "& td": {
                    borderColor: "rgba(255,255,255,0.075)",
                  },
                  "&.Mui-selected, &.Mui-selected:hover": {
                    backgroundColor: "rgba(18,82,80,0.32)",
                  },
                }}
              >
                <TableCell sx={{ color: hubTokens.colors.text, py: 0.45 }}>
                  <Typography component="div" variant="body2" noWrap sx={{ display: "flex", alignItems: "center", gap: 1.2 }}>
                    <ProjectCover coverId={project.coverId} size="thumb" />
                    {project.name}
                  </Typography>
                </TableCell>
                <BodyCell>{project.engineVersion}</BodyCell>
                <BodyCell>{project.modified}</BodyCell>
                <BodyCell>{project.location}</BodyCell>
                <TableCell align="right" sx={{ py: 0.45 }}>
                  <IconButton
                    aria-label={`${labels.openDetails}: ${project.name}`}
                    size="small"
                    onClick={(event) => {
                      event.stopPropagation();
                      if (onRowMenu) {
                        onRowMenu(project, event.currentTarget);
                        return;
                      }
                      onOpenDetail?.(project);
                    }}
                    sx={{ color: hubTokens.colors.textSoft }}
                  >
                    <MoreVertIcon fontSize="small" />
                  </IconButton>
                </TableCell>
              </TableRow>
            );
          })}
        </TableBody>
      </Table>
    </Box>
  );
}

function HeaderCell({ children, width }: { children?: ReactNode; width?: string | number }) {
  return (
    <TableCell width={width} sx={{ color: hubTokens.colors.textSoft, fontSize: 12, fontWeight: 500, py: 0.6 }}>
      {children}
    </TableCell>
  );
}

function BodyCell({ children }: { children: ReactNode }) {
  return (
    <TableCell sx={{ color: hubTokens.colors.textSoft, py: 0.45 }}>
      <Typography variant="body2" noWrap>
        {children}
      </Typography>
    </TableCell>
  );
}
