import OpenInNewOutlinedIcon from "@mui/icons-material/OpenInNewOutlined";
import { Box, Card, CardActionArea, Chip, IconButton, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";
import type { HubProjectSummary } from "../../types/hub";
import { ProjectCover } from "./ProjectCover";

// 项目摘要及当前选择由页面提供；打开回调接收原摘要，供页面按稳定项目身份进入详情。
export interface ProjectCardProps {
  project: HubProjectSummary;
  selected?: boolean;
  openDetailsLabel: string;
  onOpen?: (project: HubProjectSummary) => void;
}

// 卡片和角落入口共用详情动作；此处只表达页面选择，实际打开编辑器仍由详情工作流发起。
export function ProjectCard({ project, selected = false, openDetailsLabel, onOpen }: ProjectCardProps) {
  return (
    <Card
      sx={{
        height: 251,
        minWidth: 0,
        position: "relative",
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
      <IconButton
        size="small"
        aria-label={`${openDetailsLabel}: ${project.name}`}
        onClick={(event) => {
          // 角落入口与整张卡片指向同一详情，须阻止一次手势同时触发两个入口。
          event.stopPropagation();
          onOpen?.(project);
        }}
        sx={{
          position: "absolute",
          top: 16,
          right: 16,
          width: 30,
          height: 30,
          color: hubTokens.colors.textSoft,
          backgroundColor: "rgba(15,15,15,0.76)",
          "&:hover": { backgroundColor: "rgba(25,25,25,0.9)" },
        }}
      >
        <OpenInNewOutlinedIcon fontSize="small" />
      </IconButton>
    </Card>
  );
}

// 引擎与平台都属于摘要元数据，只以轻量色调区分信息层级，不赋予可点击或状态判断语义。
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
