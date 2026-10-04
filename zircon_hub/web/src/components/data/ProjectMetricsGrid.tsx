import EventOutlinedIcon from "@mui/icons-material/EventOutlined";
import PushPinOutlinedIcon from "@mui/icons-material/PushPinOutlined";
import StorageOutlinedIcon from "@mui/icons-material/StorageOutlined";
import WarningAmberIcon from "@mui/icons-material/WarningAmber";
import { Box } from "@mui/material";
import type { HubProjectsText, HubProjectDetail, HubSourceEngineSummary } from "../../types/hub";
import { MetricCard } from "./MetricCard";

// 项目和已解析的引擎绑定来自同一详情快照；绑定缺失时保留项目自身版本信息与说明。
export interface ProjectMetricsGridProps {
  project: HubProjectDetail;
  boundEngine?: HubSourceEngineSummary;
  text: HubProjectsText;
}

// 概览用于解释当前项目上下文；路径存在只表示可定位，后续构建和交付仍需各自验证前提。
export function ProjectMetricsGrid({ project, boundEngine, text }: ProjectMetricsGridProps) {
  return (
    <Box
      sx={{
        display: "grid",
        gridTemplateColumns: "repeat(4, minmax(0, 1fr))",
        gap: 1.2,
        mb: 1.4,
        "@media (max-width: 1180px)": { gridTemplateColumns: "repeat(2, minmax(0, 1fr))" },
        "@media (max-width: 720px)": { gridTemplateColumns: "1fr" },
      }}
    >
      <MetricCard label={text.status} value={project.status} detail={project.exists ? text.ready : text.pathUnavailable} icon={<WarningAmberIcon />} tone={project.exists ? "success" : "warning"} />
      <MetricCard label={text.engine} value={project.engineVersion} detail={boundEngine?.status ?? text.projectBinding} icon={<StorageOutlinedIcon />} tone="accent" />
      <MetricCard label={text.lastModified} value={project.modified} detail={project.platform} icon={<EventOutlinedIcon />} />
      <MetricCard label={text.projectPin} value={project.pinned ? text.pinned : text.unpinned} detail={project.templateLabel} icon={<PushPinOutlinedIcon />} />
    </Box>
  );
}
