import Inventory2OutlinedIcon from "@mui/icons-material/Inventory2Outlined";
import { Box, Typography } from "@mui/material";
import type { HubActionHandler, HubProjectDetail, HubProjectsText, HubQuickAction, ProjectTargetPayload } from "../../types/hub";
import { HUB_ACTION } from "../../types/hub";
import { HubButton } from "../inputs";
import { HubPanel } from "./HubPanel";
import { QuickActions } from "./QuickActions";
import { SourceEngineList } from "./SourceEngineList";
import { StatusBadge } from "./StatusBadge";

// 详情页须把所展示项目的显式目标与快捷动作可用目标分别传入；管理按钮不应仅凭全局选择推断操作对象。
export interface ProjectDetailSidebarProps {
  project: HubProjectDetail;
  projectTarget?: ProjectTargetPayload;
  quickActionProjectTarget?: ProjectTargetPayload;
  quickActions: HubQuickAction[];
  sourceEngines: Parameters<typeof SourceEngineList>[0]["engines"];
  text: HubProjectsText;
  actionText: {
    packageProject: string;
    installToDevice: string;
    cancelDelete: string;
    confirmDelete: string;
    unpinProject: string;
    pinProject: string;
    removeFromHub: string;
    requestDelete: string;
  };
  emptyEngineLabel: string;
  onAction: HubActionHandler;
}

// 聚合项目交付、源码选择和两阶段删除入口；这里只发送用户意图，目标校验、失败反馈及持久化归统一动作链。
export function ProjectDetailSidebar({
  project,
  projectTarget,
  quickActionProjectTarget,
  quickActions,
  sourceEngines,
  text,
  actionText,
  emptyEngineLabel,
  onAction,
}: ProjectDetailSidebarProps) {
  // 待删除标记属于后端确认流程；渲染确认入口不会执行删除，也不能以本地状态替代后端再次核对目标。
  // BUG: [CR-HUBWEB-0002] 当所展示项目不存在时交付按钮仍可点击，动作却会在目标解析或根目录校验中被拒绝；证据：详情页保留缺失项目，云交付页同操作已禁用。
  return (
    <Box sx={{ display: "grid", gap: 1.4, alignContent: "start" }}>
      <HubPanel title={text.quickActions}>
        <QuickActions actions={quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />
      </HubPanel>
      <HubPanel title={text.sourceEngines}>
        <SourceEngineList engines={sourceEngines} emptyLabel={emptyEngineLabel} onSelect={(engine) => void onAction(HUB_ACTION.selectEngine, engine.id)} />
      </HubPanel>
      <HubPanel title={text.package}>
        <Box sx={{ display: "grid", gap: 1 }}>
          <HubButton startIcon={<Inventory2OutlinedIcon />} onClick={() => void onAction(HUB_ACTION.packageProject, undefined, projectTarget)}>
            {actionText.packageProject}
          </HubButton>
          <HubButton onClick={() => void onAction(HUB_ACTION.installDevice, undefined, projectTarget)}>
            {actionText.installToDevice}
          </HubButton>
        </Box>
      </HubPanel>
      <HubPanel title={text.projectManagement}>
        <Box sx={{ display: "grid", gap: 1 }}>
          {project.pendingDelete ? (
            <>
              <StatusBadge label={text.deleteRequested} tone="warning" />
              <Typography variant="caption" color="text.secondary">
                {text.deleteRequestedDetail}
              </Typography>
              <HubButton onClick={() => void onAction(HUB_ACTION.cancelDelete, undefined, projectTarget)}>{actionText.cancelDelete}</HubButton>
              <HubButton tone="danger" onClick={() => void onAction(HUB_ACTION.confirmDelete, undefined, projectTarget)}>
                {actionText.confirmDelete}
              </HubButton>
            </>
          ) : (
            <>
              <HubButton onClick={() => void onAction(project.pinned ? HUB_ACTION.unpinProject : HUB_ACTION.pinProject, undefined, projectTarget)}>
                {project.pinned ? actionText.unpinProject : actionText.pinProject}
              </HubButton>
              <HubButton onClick={() => void onAction(HUB_ACTION.removeFromHub, undefined, projectTarget)}>{actionText.removeFromHub}</HubButton>
              <HubButton tone="danger" onClick={() => void onAction(HUB_ACTION.requestDelete, undefined, projectTarget)}>
                {actionText.requestDelete}
              </HubButton>
            </>
          )}
        </Box>
      </HubPanel>
    </Box>
  );
}
