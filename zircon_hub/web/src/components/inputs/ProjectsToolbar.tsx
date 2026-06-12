import FormatListBulletedIcon from "@mui/icons-material/FormatListBulleted";
import GridViewIcon from "@mui/icons-material/GridView";
import { Box } from "@mui/material";
import type { HubProjectsText } from "../../types/hub";
import { HubSearchField } from "./HubSearchField";
import { HubSelect } from "./HubSelect";
import { HubToggle } from "./HubToggle";

export interface ProjectsToolbarProps {
  search: string;
  filter: string;
  sort: string;
  viewMode: string;
  text: HubProjectsText;
  onSearch: (value: string) => void;
  onFilter: (value: string) => void;
  onSort: (value: string) => void;
  onViewMode: (value: string) => void;
}

export function ProjectsToolbar({ search, filter, sort, viewMode, text, onSearch, onFilter, onSort, onViewMode }: ProjectsToolbarProps) {
  return (
    <Box
      sx={{
        display: "grid",
        gridTemplateColumns: "minmax(260px, 307px) 1fr auto auto auto",
        alignItems: "center",
        gap: 1.2,
        mt: 2,
        "@media (max-width: 1180px)": {
          gridTemplateColumns: "minmax(240px, 1fr) auto auto",
        },
        "@media (max-width: 760px)": {
          gridTemplateColumns: "1fr",
        },
      }}
    >
      <HubSearchField value={search} placeholder={text.searchPlaceholder} onChange={onSearch} />
      <Box sx={{ minWidth: 0 }} />
      <HubSelect
        value={filter}
        minWidth={183}
        options={[
          { value: "all", label: text.filterAll },
          { value: "existing", label: text.filterExisting },
          { value: "missing", label: text.filterMissing },
        ]}
        onChange={onFilter}
      />
      <HubSelect
        value={sort}
        minWidth={190}
        options={[
          { value: "last-modified", label: text.sortLastModified },
          { value: "name", label: text.sortName },
        ]}
        onChange={onSort}
      />
      <HubToggle
        value={viewMode}
        onChange={onViewMode}
        options={[
          { value: "grid", label: text.gridView, icon: <GridViewIcon /> },
          { value: "list", label: text.listView, icon: <FormatListBulletedIcon /> },
        ]}
      />
    </Box>
  );
}
