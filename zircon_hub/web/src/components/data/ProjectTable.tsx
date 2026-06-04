import type { ReactNode } from "react";
import MoreVertIcon from "@mui/icons-material/MoreVert";
import { IconButton, Table, TableBody, TableCell, TableHead, TableRow, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";
import type { HubRecentProject } from "../../types/hub";
import { ProjectCover } from "./ProjectCover";

export interface ProjectTableProps {
  projects: HubRecentProject[];
}

export function ProjectTable({ projects }: ProjectTableProps) {
  return (
    <Table size="small" sx={{ tableLayout: "fixed" }}>
      <TableHead>
        <TableRow>
          <HeaderCell width="32%">Name</HeaderCell>
          <HeaderCell width="18%">Engine Version</HeaderCell>
          <HeaderCell width="16%">Last Modified</HeaderCell>
          <HeaderCell>Location</HeaderCell>
          <HeaderCell width={42} />
        </TableRow>
      </TableHead>
      <TableBody>
        {projects.map((project) => (
          <TableRow
            key={project.id}
            hover
            sx={{
              height: 36,
              "& td": {
                borderColor: "rgba(255,255,255,0.075)",
              },
            }}
          >
            <TableCell sx={{ color: hubTokens.colors.text, py: 0.45 }}>
              <Typography variant="body2" noWrap sx={{ display: "flex", alignItems: "center", gap: 1.2 }}>
                <ProjectCover coverId={project.coverId} size="thumb" />
                {project.name}
              </Typography>
            </TableCell>
            <BodyCell>{project.engineVersion}</BodyCell>
            <BodyCell>{project.modified}</BodyCell>
            <BodyCell>{project.location}</BodyCell>
            <TableCell align="right" sx={{ py: 0.45 }}>
              <IconButton aria-label={`${project.name} actions`} size="small" sx={{ color: hubTokens.colors.textSoft }}>
                <MoreVertIcon fontSize="small" />
              </IconButton>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
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
