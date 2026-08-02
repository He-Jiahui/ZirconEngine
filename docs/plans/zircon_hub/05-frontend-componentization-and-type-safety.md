---
related_code:
  - zircon_hub/web/src/App.tsx
  - zircon_hub/web/src/components/shell/HubWindow.tsx
  - zircon_hub/web/src/components/shell/TopBar.tsx
  - zircon_hub/web/src/components/feedback/HubErrorBoundary.tsx
  - zircon_hub/web/src/pages/ProjectsDashboard.tsx
  - zircon_hub/web/src/pages/ProjectBrowserPage.tsx
  - zircon_hub/web/src/pages/ProjectDetailPage.tsx
  - zircon_hub/web/src/pages/SettingsPage.tsx
  - zircon_hub/web/src/tauri/hubApi.ts
  - zircon_hub/web/src/tauri/hubStateValidator.ts
  - zircon_hub/web/src/types/hub.ts
  - zircon_hub/web/src/data/hubData.ts
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/view_model/ui_text.rs
  - zircon_hub/tests/ui_shell_composition_contract.rs
  - zircon_hub/tests/ui_project_layout_contract.rs
  - zircon_hub/tests/project_workflow_contract.rs
  - zircon_hub/tests/project_page_copy_contract.rs
  - zircon_hub/tests/project_source_engine_contract.rs
  - zircon_hub/tests/ui_panel_slot_contract.rs
  - zircon_hub/tests/ui_overlay_primitives_contract.rs
  - zircon_hub/tests/ui_metric_section_contract.rs
  - zircon_hub/tests/ui_inputs_contract.rs
  - zircon_hub/tests/ui_input_navigation_api_contract.rs
  - zircon_hub/tests/ui_navigation_contract.rs
  - zircon_hub/tests/ui_data_display_contract.rs
  - zircon_hub/tests/ui_data_container_primitives_contract.rs
  - zircon_hub/tests/tauri_react_shell_contract.rs
  - zircon_hub/tests/ui_foundation_contract.rs
plan_sources:
  - docs/plans/zircon_hub/index.md
  - docs/plans/zircon_hub/01-action-dispatch-and-typed-payload.md
  - docs/zircon_hub/ui/responsive-component-system.md
status: in_progress
---

# 05 前端组件化与类型安全

> 2026-08-01 实仓复核：路由表、根级 ErrorBoundary、运行时 DTO 护栏、demo-mode 徽标，以及 Projects/Settings/Project Detail 的主要组件拆分均已落地。`web/src` 中与 TypeScript owner 重复的 4 个 CommonJS `.js` 编译产物已删除并由 `.gitignore` 禁止回流；本文状态改为 `in_progress`，剩余双重可空类型收敛与受管 frontend/Hub gate 尚未验收。

> 2026-06-12 实仓复核注记：本文撰写期间，工作树中的并行进程已经落地了本计划的一部分目标——HubWindow 路由表、根级 HubErrorBoundary、`assertHubShellState` 运行时护栏、`hubData.ts` 收缩（995 行 → 661 行，空项目骨架 + `demoMode: true`）。下文「现状与证据」已按当前实仓修订（修订点见「风险与协调」末节），M1 与 M3 切片 1/2 转为盘点验收口径，本计划剩余的实施重心是 **M2 组件拆分**、**M3 的 demoMode 徽标** 与 **`?: X | null` 双重可空清理**。

## 现状与证据

- 页面路由：`HubWindow.tsx:23-37` 已是 `const pageRoutes: Record<HubPageId, HubPageComponent>` 查表渲染（assets/plugins/learn 三键同值 `CatalogPage`），`toHubPageId`（73-75 行）未匹配时回落 `WorkspacePage`。原「7 分支三元链」描述已失效（并行进程已落地，详见风险注记）。
- 大块内联 JSX（仍然成立，M2 工作对象）：`ProjectsDashboard.tsx`（332 行）内联搜索/筛选/排序/视图切换工具栏（148-208 行）与完整的新建项目对话框表单（289-329 行，含 38-41 行表单 state 与 51-72 行三个同步 effect）；`SettingsPage.tsx`（312 行）单文件无分节组件（overview/toolchain/paths/advanced 四个 tab 分节在 159-245 行，右栏 health/activeEngine 在 248-266 行，`SettingsPathField` 局部组件在 272-289 行）；`ProjectDetailPage.tsx`（237 行）指标网格（119-133 行）与右侧栏（188-231 行）内联。
- 类型安全：`hubApi.ts` 已接入 `assertHubShellState`（20、36、50 行），初始加载校验失败落 fallback（21-24 行）、事件载荷无效则忽略并维持当前状态（hubApi.ts:48-54）；护栏实现在 `web/src/tauri/hubStateValidator.ts`（69 行，顶层 9 个字符串字段 + 12 个数组字段 + 6 个对象字段）。`types/hub.ts`（776 行）仍多处 `?: string | null` 可选与可空双重含义混用（如 `HubTaskSummary.recovery:14`、`HubProjectDetail.engineId/templateId:49-50`、`HubShellState.selectedProjectId/activeSourceEngineId/selectedProject/settingsDraft:757-774`）——M3 切片 3 工作对象。
- 错误处理：根级 `HubErrorBoundary` 已存在（`web/src/components/feedback/HubErrorBoundary.tsx`，59 行，类组件 + `getDerivedStateFromError`），`App.tsx:138-140` 挂载并以 `onReset={() => void reloadHubState()}` 重载状态；action 失败仍仅把 taskSummary 置 error（`App.tsx:110-133`，契约口径正确：不回退 demo 数据）。
- fallback mock：`hubData.ts` 已收缩为 661 行最小骨架（`projects: []`、`sourceEngines: []`、`demoMode: true`，保留完整 ui 文案树与 settings.health 7 行集合，237-294 行），演示项目与硬编码相对时间已删除。**残余缺口**：`demoMode` 标记无任何 UI 呈现（`TopBar.tsx` 全文无 demoMode 引用），Rust 侧 `HubViewModel`（view_model.rs:43-75）与 `HubShellText`（ui_text.rs:23-63）均无 demo_mode 字段/文案 key。

## 目标

1. 页面路由表化：`HubWindow` 改为 `Record<HubPageId, ComponentType<HubPageProps>>` 查表渲染，保持 HubWindow 的唯一 router 地位与契约断言。【已落地，M1 转验收】
2. 页面瘦身为"状态投影 + 组合"：抽出 `ProjectsToolbar`（搜索/筛选/排序/视图切换）、`CreateProjectDialog`、`ProjectDetailSidebar`、`ProjectMetricsGrid`、`SettingsSection`（按 tab/分节）等业务组件，落位遵守现有 barrel 结构（`components/data|inputs|overlays` 家族）；pages 只剩数据投影与布局组合，对齐 `ui_global_rules_contract` 的"pages 仅组合"规则。
3. DTO 运行时护栏（零新依赖）：手写一个轻量 `assertHubShellState(value): HubShellState`——只校验顶层关键字段存在与类型（activePage、ui、recentProjects 为数组等），失败抛带字段名的错误并走 fallback；不引 zod。【已落地，M3 切片 1 转验收】
4. ErrorBoundary：根级一个边界组件，渲染异常时显示本地化错误卡片 +"重新加载"按钮（重新 `loadHubState`）。【已落地；实现文案走 `state.ui.shell` 既有四 key（actionFailed/actionFailedDetail/checkActionTarget/stateRefreshAfterCommand）而非原计划的 `ui.common`，零新增 key，以实仓为准】
5. fallback mock 治理：`hubData.ts` 收缩为最小骨架状态（空项目列表 + 完整 ui 文案树 + `demoMode: true` 标记），演示项目数据移出生产 bundle（仅截图/契约流程经 seeded config 注入，对齐 03 计划 M2）；硬编码相对时间随演示数据一并退出。【收缩已落地；剩余 demoMode 徽标呈现，归 M3 切片 2】

## 非目标

- 不引入 React Router / 状态库 / 表单库 / i18n 库 / zod（依赖克制是索引 §3 的全局约束）。
- 不做代码分割与虚拟滚动（窗口固定尺寸桌面应用，列表量级小，收益不成立）。
- 不改 `hubApi.ts` 的三函数公共面（`loadHubState` / `dispatchHubAction` / `subscribeHubStateChanged`，契约锁定）。
- 不改 `ProjectBrowserPage.tsx` 的内联工具栏（与 dashboard 工具栏形似但契约面独立，且 06 计划不触碰该页；如后续复用 `ProjectsToolbar` 另起切片）。
- 不触碰 06 计划 M3 已认领的落点：dashboard 卡片网格分支（将由 `ProjectCardRail` 收编）、`ProjectTable` 的 `onRowMenu` 扩展、`NavigationDrawer` props 扩展。

## 里程碑

> 通用约定：下文行号为 2026-06-12 实仓基线，行号漂移时以引用的组件/函数名定位。前端 `npm run typecheck` / `npm run build` 一律在 `zircon_hub/` 目录（`package.json` 所在目录，`scripts.typecheck = "tsc -b"`、`scripts.build = "tsc -b && vite build"`）下执行；Rust 契约用 `cargo test -p zircon_hub --test <契约文件名> --locked` 单测精跑。

### M1 路由表与 ErrorBoundary【已落地——转盘点验收】

切片：
1. `HubWindow` 查表渲染（assets/plugins/learn 共用 CatalogPage 的映射在表中显式三键同值）；fallback workspace 页保持。【已落地】
2. 根级 `HubErrorBoundary` 组件 + 本地化错误卡片；挂载点为 `App.tsx`（实仓如此；原计划写 main.tsx，以实仓为准，契约已锁 App.tsx import 行）。【已落地】
3. 同变更刷新 `ui_shell_composition_contract` / `ui_page_surface_coverage_contract` 对 HubWindow 源文本的断言。【已落地——契约已是路由表断言】

测试阶段：
- `npm run typecheck && npm run build`；`cargo test -p zircon_hub ui_shell --locked`。

#### M1 现状核对清单（验收即关闭，无需编码）

逐项确认以下实仓事实仍然成立，全部成立则 M1 关闭：

1. 路由表：`HubWindow.tsx:21-33` 为
   ```tsx
   type HubPageComponent = ComponentType<HubWindowProps>;

   const pageRoutes: Record<HubPageId, HubPageComponent> = {
     projects: ProjectsDashboard,
     editor: EditorPage,
     assets: CatalogPage,
     builds: BuildsPage,
     plugins: CatalogPage,
     cloud: CloudPage,
     team: TeamPage,
     learn: CatalogPage,
     settings: SettingsPage,
   };
   ```
   35-37 行 `const PageComponent = activeRoute ? pageRoutes[activeRoute] : WorkspacePage;`，73-75 行 `toHubPageId` 用 `activePage in pageRoutes` 收窄。
2. ErrorBoundary：`HubErrorBoundary.tsx:16-25` 类组件实现 `getDerivedStateFromError` / `componentDidCatch`；44-52 行重置按钮 `this.setState({ error: null }); this.props.onReset();`，按钮文案 `shellText.stateRefreshAfterCommand`；`App.tsx:138-140` 挂载 `<HubErrorBoundary shellText={state.ui.shell} onReset={() => void reloadHubState()}>`，`App.tsx:20-23` 定义 `reloadHubState`。
3. barrel：`components/feedback/index.ts:1` 有 `export * from "./HubErrorBoundary";`。
4. 契约已锁路由表与挂载（抽查原文，应全部命中）：
   - `ui_shell_composition_contract.rs:72` `"import { HubErrorBoundary, HubSnackbar } from \"./components/feedback\";"`；139-141 行 `"type HubPageComponent = ComponentType<HubWindowProps>;"` / `"const pageRoutes: Record<HubPageId, HubPageComponent> = {"` / `"const PageComponent = activeRoute ? pageRoutes[activeRoute] : WorkspacePage;"`。
   - `ui_global_rules_contract.rs:330-333`、`ui_navigation_contract.rs:144-152`、`ui_shell_navigation_contract.rs:146-149`、`ui_shell_page_contract.rs:71-82`、`ui_input_navigation_api_contract.rs:374-377` 同样锁 `pageRoutes` 查表两行。
   - `tauri_react_shell_contract.rs:389-405` 逐键锁 `"projects: ProjectsDashboard,"` … `"settings: SettingsPage,"` 与 `"<PageComponent state={state} onAction={onAction} />"`。
   - `ui_page_surface_coverage_contract.rs:45` 读取 HubWindow 源（覆盖面断言）。

#### M1 验收命令

```bash
cd zircon_hub && npm run typecheck && npm run build
cargo test -p zircon_hub --test ui_shell_composition_contract --locked
cargo test -p zircon_hub --test ui_page_surface_coverage_contract --locked
cargo test -p zircon_hub --test ui_global_rules_contract --locked
```

### M2 页面组件拆分

切片：
1. Projects 族：抽 `ProjectsToolbar`、`CreateProjectDialog`（payload 形状对齐 01 计划 M2 的扁平 payload——`CreateProjectPayload` 已落仓于 `types/hub.ts:659-664`）、`ProjectDetailSidebar`、`ProjectMetricsGrid`；放入 `components/inputs` / `components/overlays` / `components/data` 并更新 barrel。
2. `SettingsPage` 按分节抽 `SettingsSection` 复合组件（路径/工具链/构建/语言四节 + Health 面板），接入 04 计划 M2 的 discard/restore 按钮【两按钮已在页面 112-117 行落地，拆分时保持在页头不动】。
3. 拆分只移代码不改行为：props 全部显式类型，无新增本地业务文案。
4. 同变更刷新 `ui_project_layout_contract`、`ui_data_container_primitives_contract` 等对组件清单/barrel 的断言。

测试阶段：
- `npm run typecheck && npm run build`；`cargo test -p zircon_hub --locked`（全量 ui 契约）。

#### M2 拆分总原则（消除设计决策点）

- **action 分发字符串尽量留在页面**：工具栏/对话框用回调 props（`onSearch`/`onCreate` 等），`void onAction(HUB_ACTION.xxx, ...)` 原文留在页面，契约零改动；侧栏/Settings 分节动作数量多（8+），改为把 `onAction: HubActionHandler` 与同名局部标识符（`projectTarget`、`updateDraft`、`browseFolder`）作为 props 传入，**保持源文本逐字不变**，契约只需把 `read_crate_file` 落点从页面换成组件文件。
- **硬切换**：每个切片在同一变更内新建组件、替换页面调用点、删除内联块、刷新契约断言，不留双轨。
- **收尾复核**：每个切片提交前，对被移动的每段源文本在 `zircon_hub/tests/` 下跑 `rg -l "<原文片段>" zircon_hub/tests`，确认所有引用处都已改指新文件。

#### M2 目标代码形状

（S1）`ProjectsToolbar` —— 新建 `web/src/components/inputs/ProjectsToolbar.tsx`。现状为 `ProjectsDashboard.tsx:148-208` 的内联 grid 行（`gridTemplateColumns: "minmax(260px, 307px) 1fr auto auto auto"` + 1180/760 两档媒体查询），改为受控展示组件，选项数组与文案 key 随组件走，分发回调留在页面：

```tsx
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
        "@media (max-width: 1180px)": { gridTemplateColumns: "minmax(240px, 1fr) auto auto" },
        "@media (max-width: 760px)": { gridTemplateColumns: "1fr" },
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
```

页面调用点（替换 148-208 行；`setSearch` 等本地 state 与 `void onAction(...)` 原文保留在页面，对应契约断言不动）：

```tsx
<ProjectsToolbar
  search={search}
  filter={filter}
  sort={sort}
  viewMode={viewMode}
  text={text}
  onSearch={(value) => {
    setSearch(value);
    void onAction(HUB_ACTION.searchProjects, undefined, { query: value });
  }}
  onFilter={(value) => {
    setFilter(value);
    void onAction(HUB_ACTION.setProjectFilter, value);
  }}
  onSort={(value) => {
    setSort(value);
    void onAction(HUB_ACTION.setProjectSort, value);
  }}
  onViewMode={(value) => {
    setViewMode(value);
    void onAction(HUB_ACTION.setProjectViewMode, value);
  }}
/>
```

（S2）`CreateProjectDialog` —— 新建 `web/src/components/overlays/CreateProjectDialog.tsx`。现状为 `ProjectsDashboard.tsx:289-329` 的内联 `<HubDialog>` 表单 + 页面持有的表单 state（38-41 行 `projectName/projectLocation/template/engineId` 四个 useState）与三个同步 effect（51-72 行）。表单 state 与 effect 整体移入对话框组件，页面只投影 props：

```tsx
import { Box } from "@mui/material";
import { useEffect, useState } from "react";
import type {
  CreateProjectPayload,
  HubActionText,
  HubProjectsText,
  HubProjectTemplate,
  HubSourceEngineSummary,
} from "../../types/hub";
import { HubButton, HubComboBox, HubTextField } from "../inputs";
import { HubDialog } from "./HubDialog";

export interface CreateProjectDialogProps {
  open: boolean;
  templates: HubProjectTemplate[];
  sourceEngines: HubSourceEngineSummary[];
  activeSourceEngineId: string | null | undefined;
  defaultProjectDir: string;
  text: HubProjectsText;
  actionText: HubActionText;
  onClose: () => void;
  onCreate: (payload: CreateProjectPayload) => void;
}

export function CreateProjectDialog({ open, templates, sourceEngines, activeSourceEngineId, defaultProjectDir, text, actionText, onClose, onCreate }: CreateProjectDialogProps) {
  const [projectName, setProjectName] = useState("");
  const [projectLocation, setProjectLocation] = useState(defaultProjectDir);
  const [template, setTemplate] = useState("renderable-empty");
  const [engineId, setEngineId] = useState(activeSourceEngineId ?? sourceEngines[0]?.id ?? "");

  // 三个同步 effect 自页面 51-72 行原样迁移，依赖项改为 props：
  // defaultProjectDir → setProjectLocation；sourceEngines/activeSourceEngineId → setEngineId 守卫；
  // templates → 首个 enabled 模板兜底。

  const selectedTemplate = templates.find((projectTemplate) => projectTemplate.id === template);
  const createDisabled = projectName.trim().length === 0 || projectLocation.trim().length === 0 || !selectedTemplate?.enabled;
  const createProject = () => {
    if (createDisabled) {
      return;
    }
    onCreate({ name: projectName, location: projectLocation, template, engineId: engineId || null });
  };

  return (
    <HubDialog
      open={open}
      title={text.newProjectDialog}
      onClose={onClose}
      actions={
        <>
          <HubButton onClick={onClose}>{actionText.close}</HubButton>
          <HubButton tone="primary" disabled={createDisabled} onClick={createProject}>
            {actionText.createProject}
          </HubButton>
        </>
      }
    >
      {/* 表单四控件自页面 302-328 行原样迁移：
          HubTextField label={text.projectName} / label={text.location}
          HubComboBox placeholder={text.sourceEngine} options={sourceEngines.map((engine) => ({ ... detail: engine.sourcePath }))}
          HubComboBox placeholder={text.template} options={templates.map((projectTemplate) => ({ ... }))} */}
    </HubDialog>
  );
}
```

页面调用点（替换 289-329 行；`open={state.projectSubpage === "new-project"}` 与 `onClose={() => void onAction(HUB_ACTION.viewAllProjects)}` 原文留在页面，保住对应契约断言）：

```tsx
<CreateProjectDialog
  open={state.projectSubpage === "new-project"}
  templates={state.projectTemplates}
  sourceEngines={state.sourceEngines}
  activeSourceEngineId={state.activeSourceEngineId}
  defaultProjectDir={state.settings.defaultProjectDir}
  text={text}
  actionText={actionText}
  onClose={() => void onAction(HUB_ACTION.viewAllProjects)}
  onCreate={(payload) => void onAction(HUB_ACTION.createProject, undefined, payload)}
/>
```

同切片从页面删除：38-41 行四个表单 useState、51-72 行三个表单 effect、87-99 行 `selectedTemplate/createDisabled/createProject`，以及不再使用的 `HubComboBox/HubTextField` import。

（S3）`ProjectMetricsGrid` —— 新建 `web/src/components/data/ProjectMetricsGrid.tsx`。现状为 `ProjectDetailPage.tsx:119-133` 的四卡网格（`repeat(4, minmax(0, 1fr))` + 1180/720 两档媒体查询）。`boundEngine` 计算留在页面，组件收 `engineDetail` 字符串，四个 `MetricCard ...` 行源文本逐字保留：

```tsx
import EventOutlinedIcon from "@mui/icons-material/EventOutlined";
import PushPinOutlinedIcon from "@mui/icons-material/PushPinOutlined";
import StorageOutlinedIcon from "@mui/icons-material/StorageOutlined";
import WarningAmberIcon from "@mui/icons-material/WarningAmber";
import { Box } from "@mui/material";
import type { HubProjectDetail, HubProjectsText } from "../../types/hub";
import { MetricCard } from "./MetricCard";

export interface ProjectMetricsGridProps {
  project: HubProjectDetail;
  engineDetail: string;
  text: HubProjectsText;
}

export function ProjectMetricsGrid({ project, engineDetail, text }: ProjectMetricsGridProps) {
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
      <MetricCard label={text.engine} value={project.engineVersion} detail={engineDetail} icon={<StorageOutlinedIcon />} tone="accent" />
      <MetricCard label={text.lastModified} value={project.modified} detail={project.platform} icon={<EventOutlinedIcon />} />
      <MetricCard label={text.projectPin} value={project.pinned ? text.pinned : text.unpinned} detail={project.templateLabel} icon={<PushPinOutlinedIcon />} />
    </Box>
  );
}
```

页面调用点：`<ProjectMetricsGrid project={project} engineDetail={boundEngine?.status ?? text.projectBinding} text={text} />`。

（S4）`ProjectDetailSidebar` —— 新建 `web/src/components/data/ProjectDetailSidebar.tsx`。现状为 `ProjectDetailPage.tsx:188-231` 的右栏（quickActions / sourceEngines / package / projectManagement 四个 HubPanel，含 pendingDelete 分支）。props 沿用页面同名标识符，使 `void onAction(HUB_ACTION.packageProject, undefined, projectTarget)` 等 8 条分发源文本**逐字不变**：

```tsx
import Inventory2OutlinedIcon from "@mui/icons-material/Inventory2Outlined";
import { Box, Typography } from "@mui/material";
import type {
  HubActionHandler,
  HubActionText,
  HubProjectDetail,
  HubProjectsText,
  HubQuickAction,
  HubSourceEngineSummary,
  ProjectTargetPayload,
} from "../../types/hub";
import { HUB_ACTION } from "../../types/hub";
import { HubButton } from "../inputs";
import { HubPanel } from "./HubPanel";
import { QuickActions } from "./QuickActions";
import { SourceEngineList } from "./SourceEngineList";
import { StatusBadge } from "./StatusBadge";

export interface ProjectDetailSidebarProps {
  project: HubProjectDetail;
  quickActions: HubQuickAction[];
  sourceEngines: HubSourceEngineSummary[];
  projectTarget: ProjectTargetPayload | undefined;
  quickActionProjectTarget: ProjectTargetPayload | undefined;
  noEngineLabel: string;
  text: HubProjectsText;
  actionText: HubActionText;
  onAction: HubActionHandler;
}

export function ProjectDetailSidebar({ project, quickActions, sourceEngines, projectTarget, quickActionProjectTarget, noEngineLabel, text, actionText, onAction }: ProjectDetailSidebarProps) {
  return (
    <Box sx={{ display: "grid", gap: 1.4, alignContent: "start" }}>
      {/* 四个 HubPanel 自页面 189-230 行原样迁移；仅两处改写：
          emptyLabel={state.ui.shell.noSourceEngineRegistered} → emptyLabel={noEngineLabel}
          actions={state.quickActions} → actions={quickActions}
          其余 onAction/projectTarget/quickActionProjectTarget 源文本逐字保留 */}
    </Box>
  );
}
```

页面调用点（替换 188-231 行）：

```tsx
<ProjectDetailSidebar
  project={project}
  quickActions={state.quickActions}
  sourceEngines={state.sourceEngines}
  projectTarget={projectTarget}
  quickActionProjectTarget={quickActionProjectTarget}
  noEngineLabel={state.ui.shell.noSourceEngineRegistered}
  text={text}
  actionText={actionText}
  onAction={onAction}
/>
```

注意：页面 157-186 行的三个 tab 面板（projectOverview / projectTree / projectActions，含 `<QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />` 整行）**留在页面**——`project_quick_actions_contract.rs:305-313` 锁该整行在 `ProjectDetailPage.tsx`，不需刷新。

（S5）`SettingsSection` —— 新建 `web/src/components/data/SettingsSection.tsx`，单文件单组件，`section` 判别式覆盖四个 tab 分节 + 右栏：现状 `SettingsPage.tsx:159-245`（overview/toolchain/paths/advanced）与 248-266（health + activeSourceEngine 右栏）。`SettingsDraft` 类型（现页面 21-33 行）、`MetricTone`/`metricToneFromStatus`（19、310-312 行）与 `SettingsPathField`（272-289 行）一并移入该文件并导出需要的部分；`updateDraft`/`browseFolder`/`onAction` 以同名 props 传入，使节内 `onChange={(value) => updateDraft("buildProfile", value)}`、`browseFolder("defaultProjectDir", draft.defaultProjectDir)`、`void onAction(HUB_ACTION.selectEngine, engine.id)` 源文本逐字不变：

```tsx
import type {
  HubActionHandler,
  HubSettingsFolderField,
  HubSettingsHealthSummary,
  HubSettingsSummary,
  HubSettingsText,
  HubShellState,
  HubSourceEngineSummary,
  StatusTone,
} from "../../types/hub";
import type { HubListItem } from "./HubList";
import type { HubTreeNode } from "./HubTreeView";

export type SettingsDraft = Pick<
  HubSettingsSummary,
  | "pythonPath" | "cargoPath" | "rustupPath"
  | "defaultProjectDir" | "defaultSourceDir" | "defaultBuildOutputDir" | "defaultDeviceInstallDir"
  | "buildProfile" | "jobs" | "language"
>;

export type SettingsSectionKind = "overview" | "toolchain" | "paths" | "advanced" | "health-sidebar";

export interface SettingsSectionProps {
  section: SettingsSectionKind;
  draft: SettingsDraft;
  settingsText: HubSettingsText;
  health: HubSettingsHealthSummary;
  healthRows: HubListItem[];
  pathTree: HubTreeNode[];
  sourceEngines: HubSourceEngineSummary[];
  noEngineLabel: string;
  browseLabel: string;
  buildProfileLabel: string;
  languageLabel: string;
  healthTone: StatusTone;
  updateDraft: <Key extends keyof SettingsDraft>(key: Key, value: SettingsDraft[Key]) => void;
  browseFolder: (field: HubSettingsFolderField, initialDir: string) => void;
  onAction: HubActionHandler;
}

export function SettingsSection(props: SettingsSectionProps) {
  switch (props.section) {
    case "overview":      // 页面 159-189 行：buildDefaultsPanel + configurationPathsPanel 两个 HubPanel
    case "toolchain":     // 页面 191-200 行：sourceEnginesPanel（pythonPath/cargoPath/rustupPath + SourceEngineList）
    case "paths":         // 页面 202-235 行：pathDefaultsPanel（四个 SettingsPathField）
    case "advanced":      // 页面 237-245 行：advancedConfigurationPanel（两 HubCheckbox + HubTreeView）
    case "health-sidebar": // 页面 248-266 行：configurationHealthPanel（completeness + LinearProgress + HubList）+ activeSourceEnginePanel
      // 各分支 JSX 自页面对应行原样迁移，state.* 引用替换为同义 props
  }
}
```

页面瘦身后（SettingsPage 保留）：页头按钮区 101-122 行（save/discard/restore/projects，分发原文不动）、指标行 128-142 行（四个 MetricCard）、`HubTabs` 行 144-146、draft state 与 `updateDraft`/`saveDraft`/`browseFolder` 函数（75-85 行，分发原文不动）、`settingsDraftState`/`settingsDraftFromState` 帮助函数（291-308 行）、两栏 grid 容器（148-157 行）；四个 tab 门控改为 `{tab === "overview" ? <SettingsSection section="overview" {...sectionProps} /> : null}` 形式，右栏为 `<SettingsSection section="health-sidebar" {...sectionProps} />`。

#### M2 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/web/src/components/inputs/ProjectsToolbar.tsx` | 新建 | 搜索/筛选/排序/视图切换工具栏（受控 + 回调） |
| `zircon_hub/web/src/components/inputs/index.ts` | 修改 | 追加 `export * from "./ProjectsToolbar";`（按字母序插在 HubToggle 前后均可，保持现有排序风格则插于 `HubToggle` 之后） |
| `zircon_hub/web/src/components/overlays/CreateProjectDialog.tsx` | 新建 | 新建项目对话框（表单 state + 三个同步 effect + 扁平 CreateProjectPayload 装配） |
| `zircon_hub/web/src/components/overlays/index.ts` | 修改 | 追加 `export * from "./CreateProjectDialog";` |
| `zircon_hub/web/src/components/data/ProjectMetricsGrid.tsx` | 新建 | 项目详情四卡指标网格 |
| `zircon_hub/web/src/components/data/ProjectDetailSidebar.tsx` | 新建 | 项目详情右栏（快捷操作/源码引擎/包/项目管理四面板 + pendingDelete 分支） |
| `zircon_hub/web/src/components/data/SettingsSection.tsx` | 新建 | Settings 分节复合组件（overview/toolchain/paths/advanced/health-sidebar）+ SettingsPathField + SettingsDraft 类型 |
| `zircon_hub/web/src/components/data/index.ts` | 修改 | 追加 ProjectMetricsGrid / ProjectDetailSidebar / SettingsSection 三行 `export * from` |
| `zircon_hub/web/src/pages/ProjectsDashboard.tsx` | 修改 | 删内联工具栏（148-208）与对话框（289-329）及表单 state/effect，改组合 ProjectsToolbar + CreateProjectDialog |
| `zircon_hub/web/src/pages/ProjectDetailPage.tsx` | 修改 | 删内联指标网格（119-133）与右栏（188-231），改组合 ProjectMetricsGrid + ProjectDetailSidebar |
| `zircon_hub/web/src/pages/SettingsPage.tsx` | 修改 | 删四节内联面板与右栏（159-266）及 SettingsPathField/SettingsDraft 定义，改组合 SettingsSection |
| `zircon_hub/tests/project_page_copy_contract.rs` | 修改 | 拆分 dashboard/detail/settings 断言块：被迁移文案断言改指新组件文件 |
| `zircon_hub/tests/project_workflow_contract.rs` | 修改 | detail 8 条项目管理分发断言与 settings 的 SettingsPathField/HubIconButton/options 断言改指新组件 |
| `zircon_hub/tests/project_source_engine_contract.rs` | 修改 | 新建对话框 engineId 断言块改读 CreateProjectDialog.tsx，`state.` 前缀去除 |
| `zircon_hub/tests/ui_overlay_primitives_contract.rs` | 修改 | dashboard HubDialog 断言改为页面 `<CreateProjectDialog` + 组件内 HubDialog 断言 |
| `zircon_hub/tests/ui_metric_section_contract.rs` | 修改 | 详情四卡断言改读 ProjectMetricsGrid.tsx，页面加 `<ProjectMetricsGrid` 断言 |
| `zircon_hub/tests/ui_panel_slot_contract.rs` | 修改 | HubPanel 计数循环对 detail/settings 改读「页面+新组件拼接源」；dashboard 控件清单改指 ProjectsToolbar/CreateProjectDialog |
| `zircon_hub/tests/ui_inputs_contract.rs` | 修改 | dashboard/settings 输入控件清单收缩，新增 ProjectsToolbar/CreateProjectDialog/SettingsSection 条目 |
| `zircon_hub/tests/ui_input_navigation_api_contract.rs` | 修改 | dashboard/settings 控件断言改指新组件，分发断言留页面不动 |
| `zircon_hub/tests/ui_navigation_contract.rs` | 修改 | dashboard 的 gridView/listView 选项断言改指 ProjectsToolbar.tsx |
| `zircon_hub/tests/ui_data_display_contract.rs` | 修改 | detail 的 MetricCard/SourceEngineList、settings 的 HubList/HubTreeView/SourceEngineList/StatusBadge 断言改指新组件 |
| `zircon_hub/tests/ui_data_container_primitives_contract.rs` | 修改 | 同上家族断言落点刷新 |
| `zircon_hub/tests/tauri_react_shell_contract.rs` | 修改 | dashboard/detail/settings 组合面断言：被迁移控件名替换为新组件名，新增组件文件断言 |

#### M2 实施步骤

每步为一次可提交的小步；验证命令统一为：`cd zircon_hub && npm run typecheck && npm run build`，外加该步列出的契约单测。

1. **S1 ProjectsToolbar**：新建 `components/inputs/ProjectsToolbar.tsx`（上文代码形状）；`components/inputs/index.ts` 追加导出；`ProjectsDashboard.tsx` 删 148-208 行内联块、删除不再使用的 `GridViewIcon/FormatListBulletedIcon` import（仍被卡片视图分支使用的保留——核对后这两个 icon 仅工具栏使用，可删）与 `HubSearchField/HubSelect/HubToggle` import，插入 `<ProjectsToolbar ...>` 调用点；同步刷新契约（见联动表 S1 行）。验证：
   ```bash
   cargo test -p zircon_hub --test project_page_copy_contract --locked
   cargo test -p zircon_hub --test ui_panel_slot_contract --locked
   cargo test -p zircon_hub --test ui_navigation_contract --locked
   cargo test -p zircon_hub --test ui_inputs_contract --locked
   cargo test -p zircon_hub --test ui_input_navigation_api_contract --locked
   cargo test -p zircon_hub --test tauri_react_shell_contract --locked
   ```
2. **S2 CreateProjectDialog**：新建 `components/overlays/CreateProjectDialog.tsx`；overlays barrel 追加导出；`ProjectsDashboard.tsx` 删 289-329 行对话框、38-41 行表单 state、51-72 行三个表单 effect、87-99 行装配函数，删除 `HubDialog/HubComboBox/HubTextField` import，插入 `<CreateProjectDialog ...>`；刷新契约（联动表 S2 行）。验证：
   ```bash
   cargo test -p zircon_hub --test ui_overlay_primitives_contract --locked
   cargo test -p zircon_hub --test project_source_engine_contract --locked
   cargo test -p zircon_hub --test project_page_copy_contract --locked
   cargo test -p zircon_hub --test project_workflow_contract --locked
   cargo test -p zircon_hub --test ui_inputs_contract --locked
   cargo test -p zircon_hub --test ui_input_navigation_api_contract --locked
   cargo test -p zircon_hub --test ui_panel_slot_contract --locked
   cargo test -p zircon_hub --test tauri_react_shell_contract --locked
   ```
3. **S3 ProjectMetricsGrid**：新建 `components/data/ProjectMetricsGrid.tsx`；data barrel 追加导出；`ProjectDetailPage.tsx` 删 119-133 行网格，插入 `<ProjectMetricsGrid project={project} engineDetail={boundEngine?.status ?? text.projectBinding} text={text} />`，清理仅指标网格使用的 icon import（`EventOutlinedIcon/PushPinOutlinedIcon` 仅此处使用可删；`WarningAmberIcon/StorageOutlinedIcon` 在 116、51 行另有使用需保留）；刷新契约（联动表 S3 行）。验证：
   ```bash
   cargo test -p zircon_hub --test ui_metric_section_contract --locked
   cargo test -p zircon_hub --test project_page_copy_contract --locked
   cargo test -p zircon_hub --test ui_panel_slot_contract --locked
   ```
4. **S4 ProjectDetailSidebar**：新建 `components/data/ProjectDetailSidebar.tsx`；data barrel 追加导出；`ProjectDetailPage.tsx` 删 188-231 行右栏，插入 `<ProjectDetailSidebar ...>`（上文调用点），清理只剩侧栏使用的 import（`Inventory2OutlinedIcon`、`SourceEngineList`；`StatusBadge/QuickActions/HubButton` 页面其余处仍用需保留）；刷新契约（联动表 S4 行）。验证：
   ```bash
   cargo test -p zircon_hub --test project_workflow_contract --locked
   cargo test -p zircon_hub --test project_page_copy_contract --locked
   cargo test -p zircon_hub --test ui_panel_slot_contract --locked
   cargo test -p zircon_hub --test ui_data_display_contract --locked
   cargo test -p zircon_hub --test ui_data_container_primitives_contract --locked
   cargo test -p zircon_hub --test project_quick_actions_contract --locked   # 应零改动通过
   ```
5. **S5 SettingsSection**：新建 `components/data/SettingsSection.tsx`（含 SettingsDraft/SettingsPathField/MetricTone 迁移）；data barrel 追加导出；`SettingsPage.tsx` 删 159-266 行四节与右栏、272-289 行 SettingsPathField、21-33 行 SettingsDraft（改 `import type { SettingsDraft } from "../components/data/SettingsSection"`——经 barrel 导入亦可，统一走 `../components/data`），页面 tab 门控改组合 `<SettingsSection ...>`；刷新契约（联动表 S5 行）。验证：
   ```bash
   cargo test -p zircon_hub --test project_page_copy_contract --locked
   cargo test -p zircon_hub --test project_workflow_contract --locked
   cargo test -p zircon_hub --test tauri_react_shell_contract --locked
   cargo test -p zircon_hub --test ui_inputs_contract --locked
   cargo test -p zircon_hub --test ui_panel_slot_contract --locked
   ```
6. **收尾全量回归**：对五个新组件文件中每段自页面迁来的断言级源文本（上表「改成」列）执行 `rg -l "<片段>" zircon_hub/tests` 复核无漏网；然后：
   ```bash
   cd zircon_hub && npm run typecheck && npm run build
   cargo test -p zircon_hub --locked
   cargo fmt --all --check
   ```

#### M2 契约联动

> 「原文」列为 2026-06-12 实仓断言原文（节选）；「改成」列给出目标。`read_crate_file` 改落点指把断言移入读取新组件文件的 `assert_contains_all` 块。

**S1（ProjectsToolbar）**

| 测试 | 现有断言原文（节选） | 改成 |
|------|----------------------|------|
| `project_page_copy_contract.rs:300-325` | dashboard 块含 `"placeholder={text.searchPlaceholder}"`、`"{ value: \"all\", label: text.filterAll }"`、`"{ value: \"last-modified\", label: text.sortLastModified }"`、`"{ value: \"grid\", label: text.gridView"`、`"{ value: \"list\", label: text.listView"` | 五条移入新增的 `assert_contains_all("ProjectsToolbar.tsx", &read_crate_file("web/src/components/inputs/ProjectsToolbar.tsx"), ...)` 块；dashboard 块补 `"<ProjectsToolbar"` |
| `ui_navigation_contract.rs:193-205` | dashboard 块含 `"{ value: \"grid\", label: text.gridView"`、`"{ value: \"list\", label: text.listView"` | 两条改指 ProjectsToolbar.tsx；`"void onAction(HUB_ACTION.setProjectViewMode, value)"` 等分发断言留 dashboard 不动 |
| `ui_panel_slot_contract.rs:169-187` | dashboard 块含 `"HubSearchField"`、`"HubSelect"`、`"HubToggle"` | 三条替换为 `"ProjectsToolbar"`；新增 ProjectsToolbar.tsx 块承接原三条 |
| `ui_inputs_contract.rs:270-281` | dashboard 条目 vec 含 `"HubSearchField"`、`"HubSelect"`、`"HubToggle"` | 从 dashboard vec 删除，循环表新增 `("web/src/components/inputs/ProjectsToolbar.tsx", vec!["HubSearchField", "HubSelect", "HubToggle"])` 条目（该循环的 `assert_page_material_imports_do_not_include_input_primitives` 对组件文件同样成立） |
| `ui_input_navigation_api_contract.rs:413-429` | dashboard 块含 `"HubSearchField"`、`"HubSelect"`、`"HubToggle"` | 三条改指 ProjectsToolbar.tsx；六条 `void onAction(...)` 分发断言留 dashboard 不动 |
| `tauri_react_shell_contract.rs:483-508` | dashboard snippet 表含 `"HubSearchField"`、`"HubSelect"`、`"HubToggle"` | 替换为 `"ProjectsToolbar"`；`"onAction(HUB_ACTION.setProjectFilter, value)"` 等留表内不动 |
| `project_workflow_contract.rs:725-737`、`:870-874` | `"void onAction(HUB_ACTION.searchProjects, undefined, { query: value });"` 等 | **零改动**（分发回调留页面） |

**S2（CreateProjectDialog）**

| 测试 | 现有断言原文（节选） | 改成 |
|------|----------------------|------|
| `ui_overlay_primitives_contract.rs:235-249` | dashboard 块含 `"import { HubDialog } from \"../components/overlays\";"`、`"<HubDialog"`、`"open={state.projectSubpage === \"new-project\"}"`、`"title={text.newProjectDialog}"`、`"HubTextField label={text.projectName}"`、`"HubTextField label={text.location}"`、`"HubComboBox"`、`"actions={"` | dashboard 块改为 `"<CreateProjectDialog"`、`"open={state.projectSubpage === \"new-project\"}"`、`"onClose={() => void onAction(HUB_ACTION.viewAllProjects)}"`；新增 CreateProjectDialog.tsx 块承接 `"import { HubDialog } from \"./HubDialog\";"`、`"<HubDialog"`、`"title={text.newProjectDialog}"`、两条 HubTextField、`"HubComboBox"`、`"actions={"` |
| `project_page_copy_contract.rs:317-323` | `"title={text.newProjectDialog}"`、`"label={text.projectName}"`、`"state.projectTemplates.map((projectTemplate) =>"`、`"label: projectTemplate.optionLabel"`、`"placeholder={text.sourceEngine}"`、`"options={state.sourceEngines.map((engine) => ({"`、`"engineId: engineId || null"` | 移入 CreateProjectDialog.tsx 块；其中两条 `state.` 前缀改写为 `"templates.map((projectTemplate) =>"`、`"options={sourceEngines.map((engine) => ({"` |
| `project_source_engine_contract.rs:83-101` | dashboard 块含 `"const [engineId, setEngineId] = useState(state.activeSourceEngineId ?? state.sourceEngines[0]?.id ?? \"\");"`、`"return state.activeSourceEngineId ?? state.sourceEngines[0]?.id ?? \"\";"`、`"engineId: engineId || null"`；97-101 行 `assert_not_contains_any` 锁 `"engineId: state.activeSourceEngineId,"` | 整块改读 `web/src/components/overlays/CreateProjectDialog.tsx`；`state.activeSourceEngineId`→`activeSourceEngineId`、`state.sourceEngines`→`sourceEngines` 同步改写；负断言改锁 `"engineId: activeSourceEngineId,"` |
| `project_workflow_contract.rs:735` | dashboard 块含 `"state.projectSubpage === \"new-project\""` | **零改动**（open prop 在页面装配） |
| `tauri_react_shell_contract.rs:493-495` | dashboard snippet 表含 `"HubDialog"`、`"HubTextField"`、`"HubComboBox"` | 替换为 `"CreateProjectDialog"` |
| `ui_inputs_contract.rs:270-281` | dashboard vec 含 `"HubComboBox"`、`"HubTextField"`（S1 后剩余） | 从 dashboard vec 删除（最终 dashboard vec 为 `["from \"../components/inputs\"", "HubButton"]`）；循环表新增 `("web/src/components/overlays/CreateProjectDialog.tsx", vec!["HubButton", "HubComboBox", "HubTextField"])` |
| `ui_input_navigation_api_contract.rs:413-429` | dashboard 块含 `"HubComboBox"`、`"HubTextField"`（S1 后剩余） | 两条改指 CreateProjectDialog.tsx |
| `ui_panel_slot_contract.rs:169-187` | dashboard 块含 `"HubDialog"` | 替换为 `"CreateProjectDialog"` |

**S3（ProjectMetricsGrid）**

| 测试 | 现有断言原文（节选） | 改成 |
|------|----------------------|------|
| `ui_metric_section_contract.rs:89-108` | `project_detail_uses_four_metric_cards_then_collapses_responsively` 读 `web/src/pages/ProjectDetailPage.tsx`，断言 `"MetricCard label={text.status} value={project.status}"` 等四卡 + 三档 grid | `read_crate_file` 改读 `web/src/components/data/ProjectMetricsGrid.tsx`（断言原文全部逐字保留）；另对页面新增 `"<ProjectMetricsGrid"` 断言 |
| `project_page_copy_contract.rs:372-378` | `"MetricCard label={text.status}"`、`"text.pathUnavailable"`、`"MetricCard label={text.engine}"`、`"text.projectBinding"`、`"MetricCard label={text.lastModified}"`、`"MetricCard label={text.projectPin}"`、`"detail={project.templateLabel}"` | 移入 ProjectMetricsGrid.tsx 断言块；注意 `"text.projectBinding"` 留页面（`engineDetail` 在页面装配），其余移组件 |
| `ui_panel_slot_contract.rs:210-225` | detail 块含 `"MetricCard"`、`"gridTemplateColumns: \"repeat(4, minmax(0, 1fr))\""` | 两条改指 ProjectMetricsGrid.tsx |

**S4（ProjectDetailSidebar）**

| 测试 | 现有断言原文（节选） | 改成 |
|------|----------------------|------|
| `project_workflow_contract.rs:749-765` | detail 块 12 条；其中 `"void onAction(HUB_ACTION.packageProject, undefined, projectTarget)"`、`"void onAction(HUB_ACTION.installDevice, undefined, projectTarget)"`、`"void onAction(project.pinned ? HUB_ACTION.unpinProject : HUB_ACTION.pinProject, undefined, projectTarget)"`、`"void onAction(HUB_ACTION.removeFromHub, undefined, projectTarget)"`、`"void onAction(HUB_ACTION.requestDelete, undefined, projectTarget)"`、`"void onAction(HUB_ACTION.cancelDelete, undefined, projectTarget)"`、`"void onAction(HUB_ACTION.confirmDelete, undefined, projectTarget)"` | 七条移入新增 ProjectDetailSidebar.tsx 断言块（源文本逐字不变）；`"const projectTarget = projectTargetPayload(project);"`、`"void onAction(HUB_ACTION.viewAllProjects)"`、`"void onAction(HUB_ACTION.openEditor, undefined, projectTarget)"`、`"void onAction(action.id, undefined, quickActionProjectTarget)"` 留页面块 |
| `project_page_copy_contract.rs:382-390` | `"HubPanel title={text.quickActions}"`、`"HubPanel title={text.sourceEngines}"`、`"HubPanel title={text.package}"`、`"actionText.packageProject"`、`"actionText.installToDevice"` | 移入 ProjectDetailSidebar.tsx 块；`"HubPanel title={text.projectOverview}"`、`"HubPanel title={text.projectTree}"`、`"HubPanel title={text.projectActions}"` 留页面 |
| `ui_panel_slot_contract.rs:131-160` | 计数循环 `("ProjectDetailPage.tsx", 6)`，断言页面 `<HubPanel` 计数 ≥6 | 循环体对该项改读拼接源：`read_crate_file("web/src/pages/ProjectDetailPage.tsx") + &read_crate_file("web/src/components/data/ProjectDetailSidebar.tsx")`（页面 3 + 侧栏 4 = 7 ≥ 6，阈值不动） |
| `ui_panel_slot_contract.rs:210-225` | detail 块含 `"SourceEngineList"`、`"HubPanel title={text.quickActions}"`、`"HubPanel title={text.sourceEngines}"` | 三条改指 ProjectDetailSidebar.tsx |
| `ui_data_display_contract.rs:262-274`、`ui_data_container_primitives_contract.rs:250-261` | detail vec 含 `"SourceEngineList"`（及 S3 的 `"MetricCard"`） | 从 detail vec 移除，circulation 表新增 ProjectDetailSidebar.tsx / ProjectMetricsGrid.tsx 条目承接 |
| `tauri_react_shell_contract.rs:549-567` | detail snippet 表含 `"MetricCard"`、`"SourceEngineList"` | 替换为 `"ProjectMetricsGrid"`、`"ProjectDetailSidebar"` |
| `project_quick_actions_contract.rs:305-313` | `"<QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />"` 在 ProjectDetailPage.tsx | **零改动**（actions tab 面板留页面，整行保留） |

**S5（SettingsSection）**

| 测试 | 现有断言原文（节选） | 改成 |
|------|----------------------|------|
| `project_page_copy_contract.rs:513-534` | settings 块含 `"options={settingsText.buildProfileOptions}"`、`"options={settingsText.languageOptions}"`、`"HubPanel title={settingsText.buildDefaultsPanel}"`、`"HubPanel title={settingsText.configurationHealthPanel}"`、`"HubPanel title={settingsText.activeSourceEnginePanel}"` | 五条移入新增 `assert_contains_all("SettingsSection.tsx", &read_crate_file("web/src/components/data/SettingsSection.tsx"), ...)` 块；`"<Typography variant=\"h4\">{settingsText.heading}</Typography>"`、save/discard/restore 按钮、四条 MetricCard、`"HubTabs value={tab} onChange={setTab} options={settingsText.tabs}"` 留页面块 |
| `project_page_copy_contract.rs:538-587` | `"HubSwitch checked={draft.buildProfile === \"release\"} label={labels.releaseBuild} detail={buildProfileLabel}"`、`"HubCheckbox checked={draft.language === \"Chinese\"} label={labels.localizedUi} detail={languageLabel}"`、`"value={draft.buildProfile}"`、`"onChange={(value) => updateDraft(\"buildProfile\", value)}"`、`"value={draft.language}"`、`"onChange={(value) => updateDraft(\"language\", value)}"`、`"detail: languageLabel"` | 七条移入 SettingsSection.tsx 块（`updateDraft` 为同名 prop，源文本逐字不变）；`"const buildProfileLabel = settingsOptionLabel(...)"` 等计算行与 `"void onAction(HUB_ACTION.saveSettings, undefined, { settings: draft })"` 留页面；575-586 行负断言（`"MetricCard label={labels.buildProfile} value={draft.buildProfile}"` 等）对页面与组件两份源都跑 |
| `project_workflow_contract.rs:826-840` | settings 块含 `"SettingsPathField"`、`"HubIconButton"`、`"settingsText.buildProfileOptions"`、`"settingsText.languageOptions"` | 四条移入 SettingsSection.tsx 块；`"settingsDraftState(state)"`、`"state.settingsDraft ?? state.settings"`、`"void onAction(HUB_ACTION.updateSettingsDraft, undefined, { settings: nextDraft });"`、`"void onAction(HUB_ACTION.browseSettingsFolder, field, { field, initialDir, settings: draft });"`、`"state.ui.actions.browseFolder"` 留页面块 |
| `tauri_react_shell_contract.rs:710-738` | settings snippet 表 22 条 | 留页面：`"settingsText.heading"`、`"settingsDraftState(state)"`、`"HubStatusBanner"`、`"MetricCard"`、`"HubTabs"`、`"onAction(HUB_ACTION.saveSettings, undefined, { settings: draft })"`、`"onAction(HUB_ACTION.browseSettingsFolder, field, { field, initialDir, settings: draft })"`、`"state.ui.actions.browseFolder"`，并补 `"SettingsSection"`；其余（`"HubComboBox"`、`"HubTextField"`、`"HubIconButton"`、`"HubSwitch"`、`"HubCheckbox"`、`"HubList"`、`"HubTreeView"`、`"SourceEngineList"`、`"StatusBadge"`、`"LinearProgress"`、`"onAction(HUB_ACTION.selectEngine, engine.id)"`、`"settingsText.buildProfileOptions"`、`"settingsText.languageOptions"`）移入对 SettingsSection.tsx 的新 snippet 循环 |
| `ui_panel_slot_contract.rs:131-160` | 计数循环 `("SettingsPage.tsx", 7)` | 该项改读拼接源 `read_crate_file("web/src/pages/SettingsPage.tsx") + &read_crate_file("web/src/components/data/SettingsSection.tsx")`（7 个 HubPanel 全在组件内，计数不变） |
| `ui_inputs_contract.rs:300-311` | settings vec 含 `"HubCheckbox"`、`"HubComboBox"`、`"HubSwitch"`、`"HubTextField"` | settings vec 收缩为 `["from \"../components/inputs\"", "HubButton", "HubTabs"]`；循环新增 `("web/src/components/data/SettingsSection.tsx", vec!["HubCheckbox", "HubComboBox", "HubIconButton", "HubSwitch", "HubTextField"])` |
| `ui_input_navigation_api_contract.rs:462-473` | settings 块含 `"HubTextField"`、`"HubComboBox"`、`"HubCheckbox"`、`"HubSwitch"` | 四条改指 SettingsSection.tsx；`"HubTabs"` 与 saveSettings 分发断言留页面 |
| `ui_navigation_contract.rs:218-228` | `("SettingsPage.tsx", settings)` 要求 `"HubTabs"`、`"const [tab, setTab] = useState"` | **零改动**（tab 门控留页面） |
| `ui_data_display_contract.rs:371-379`、`ui_data_container_primitives_contract.rs:277-285` | settings vec 含 `"HubList"`、`"HubTreeView"`、`"SourceEngineList"`、`"StatusBadge"`（display 版另有 `"healthRows"`） | 控件四条改指 SettingsSection.tsx；`"MetricCard"`、`"healthRows"`（页面计算并传 props）留页面 vec |

### M3 运行时护栏与 mock 治理

切片：
1. `assertHubShellState` 顶层校验接入 `loadHubState` 与事件回调；校验失败 console 带字段名报错并维持当前状态（事件）/落 fallback（初始加载）。【已落地——转验收】
2. `hubData.ts` 收缩：保留 ui 文案树与空骨架，演示项目/构建历史/假时间删除【已落地】；`demoMode` 标记在 TopBar 以小徽标呈现（文案走 DTO，新增 key 登记到 07）。【残余工作】
3. `types/hub.ts` 清理 `?: X | null` 双重可空：统一为 `X | null`（后端 serde 总是发 null）或 `?: X`（确实可缺省），逐字段对照 Rust DTO 定稿。【未做】

测试阶段：
- `npm run typecheck && npm run build`；故意构造缺字段状态验证护栏与 fallback 路径。
- `cargo test -p zircon_hub --locked` 回归（fallback 状态相关契约，如 Settings health 首帧行字断言）。

#### M3 切片 1 验收清单（已落地，无需编码）

- `hubStateValidator.ts:30-49`：`assertHubShellState` 校验 `requiredStrings`（9 个，含 activePage/productName/projectSubpage）+ `requiredArrays`（12 个，含 projects/recentProjects/comingSoon）+ 6 个对象（taskSummary/team/settings/ui/ui.shell/ui.common），错误消息带字段名（`Hub state field '<field>' must be ...`）。
- `hubApi.ts:14-25`：初始加载 `try { return assertHubShellState(await invoke<unknown>("hub_state")); } catch { ... return fallbackShellState; }`；`hubApi.ts:48-54`：事件回调 catch 后 `console.warn("Ignored invalid hub-state-changed payload.", error)` 并不更新状态。均符合计划口径。
- 已有契约锁定：`project_workflow_contract.rs:681` 与 `ui_foundation_contract.rs:262` 锁 `"return assertHubShellState(await invoke<unknown>(\"hub_state\"));"`；`tauri_react_shell_contract.rs:796-797` 锁 `"onStateChanged(assertHubShellState(event.payload))"` 与 `"Ignored invalid hub-state-changed payload."`。
- 验收命令：`cargo test -p zircon_hub --test tauri_react_shell_contract --locked`；手测：`npm run tauri:dev` 下临时改 Rust 端字段名或在 devtools 里 `window.dispatchEvent` 伪造事件确认 console 警告与状态保持。

#### M3 切片 2 残余：demoMode 徽标（目标代码形状）

`demoMode` 仅由前端 fallback 状态置 true（`hubData.ts:23`），Rust `HubViewModel`（view_model.rs:43-75）**不发送**该字段——真实后端态下 `state.demoMode === undefined`，徽标自然隐藏；因此 `demoMode?: boolean`（types/hub.ts:748）保持可选，**不给后端加 demo_mode 字段**。新增的只有徽标文案 key（按索引 §3 文案归 Rust DTO 所有，落 `ui_text.rs` 并登记 07 计划）：

1. Rust：`zircon_hub/src/tauri_app/view_model/ui_text.rs` 的 `HubShellText`（22-63 行）在 `check_action_target` 后追加：
   ```rust
   pub demo_mode_badge: String,
   ```
   构造处（`live_updates_unavailable`/`action_failed` 同段，402-412 行附近）追加：
   ```rust
   demo_mode_badge: text.pair("Demo Data", "演示数据").to_string(),
   ```
2. TS 镜像：`types/hub.ts` 的 `HubShellText`（278-318 行）在 `checkActionTarget: string;`（303 行）后追加 `demoModeBadge: string;`；`hubData.ts` 的 `ui.shell`（343-393 行）在 `checkActionTarget` 行（368 行）后追加 `demoModeBadge: "演示数据",`。
3. TopBar 徽标：`TopBar.tsx:109-121` 的状态 chips 容器（`state.taskStatus.map` 所在 Box）首位插入：
   ```tsx
   {state.demoMode ? <StatusBadge label={state.ui.shell.demoModeBadge} tone="warning" /> : null}
   ```

#### M3 切片 3：`?: X | null` 清理矩阵（逐字段对照 Rust DTO 定稿）

依据：Rust 侧所有 `Option<T>` 字段均无 `skip_serializing_if`，serde 序列化**总是输出键**（None → null）。对照表（Rust 出处已核实）：

| types/hub.ts 字段（现行号） | Rust 出处 | 现状 | 改成 |
|------|------|------|------|
| `HubTaskSummary.recovery`（14） | view_model.rs:92 `recovery: Option<String>` | `recovery?: string \| null` | `recovery: string \| null` |
| `HubProjectDetail.engineId / templateId`（49-50） | view_model.rs:133-134 | `?: string \| null` | `engineId: string \| null`、`templateId: string \| null` |
| `HubProjectTemplate.disabledReason`（65） | project_templates.rs:18 `disabled_reason: Option<String>` | `?: string \| null` | `disabledReason: string \| null` |
| `HubSourceBuildHistoryItem.jobs`（81） | view_model.rs:158 `jobs: Option<u16>` | `?: number \| null` | `jobs: number \| null` |
| `HubActionHistoryItem.recovery / processId / outputDir`（182-185） | action_history.rs:20-23 `Option<String>/Option<u32>/Option<String>` | `?: X \| null` | 必填 `X \| null` |
| `HubShellState.selectedProjectId / activeSourceEngineId`（757-758） | view_model.rs:56-57 `Option<String>` | `?: string \| null` | `string \| null` |
| `HubShellState.selectedProject`（764） | view_model.rs:63 `Option<HubProjectDetail>` | `?: HubProjectDetail \| null` | `HubProjectDetail \| null` |
| `HubShellState.settingsDraft`（774） | view_model.rs:73 `settings_draft: HubSettingsSummary`（非 Option，总是发送） | `?: HubSettingsSummary \| null` | `settingsDraft: HubSettingsSummary \| null`，并在 `hubData.ts` fallback 增 `settingsDraft: null,`（fallback 语义=无草稿；页面 `state.settingsDraft ?? state.settings` 行为不变） |
| `HubShellState.demoMode`（748） | 后端无此字段 | `?: boolean` | **保持 `?: boolean`**（确实可缺省） |
| `CreateProjectPayload.engineId / NewProjectDraftPayload.engineId`（663、670） | 出站 payload；调用点总是显式 `engineId: engineId \|\| null` | `?: string \| null` | 必填 `engineId: string \| null` |
| `ImportProjectPayload.engineId`（680） | 出站 payload；现无调用点传该字段（dashboard 139 行 `void onAction(HUB_ACTION.importProject)` 无 payload） | `?: string \| null` | `engineId?: string`（缺省即不发送，serde Option 对缺省/null 等价） |
| `web/src/tauri/projectTarget.ts` 的函数参数 `project?: HubProjectDetail \| null` | 非 DTO 字段，参数可空惯用法且被 `project_quick_actions_contract.rs:277、284` 锁定原文 | — | **不动** |

fallback 数据核对：`hubData.ts` 已有 `selectedProjectId: null`（53）、`activeSourceEngineId: null`（54）、`taskSummary.recovery: null`（60）、`selectedProject: null`（70）、模板 `disabledReason: null/字符串`（39、49）——仅需新增 `settingsDraft: null`。

#### M3 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/src/tauri_app/view_model/ui_text.rs` | 修改 | `HubShellText` 增 `demo_mode_badge` 字段 + 中英文案构造 |
| `zircon_hub/web/src/types/hub.ts` | 修改 | `HubShellText` 增 `demoModeBadge`；按清理矩阵收敛 12 处双重可空 |
| `zircon_hub/web/src/data/hubData.ts` | 修改 | `ui.shell` 增 `demoModeBadge`；根级增 `settingsDraft: null` |
| `zircon_hub/web/src/components/shell/TopBar.tsx` | 修改 | 状态 chips 容器首位增 demoMode 条件徽标 |
| `zircon_hub/tests/tauri_react_shell_contract.rs` | 修改 | TopBar snippet 表（413-431）增徽标断言；581 行 selectedProject 断言改必填写法 |
| `zircon_hub/tests/ui_foundation_contract.rs` | 修改 | hubData 断言块（796-817）增 `"demoModeBadge:"`；789 行 selectedProject 断言改必填写法 |
| `zircon_hub/tests/project_workflow_contract.rs` | 修改 | 722 行 `"settingsDraft?: HubSettingsSummary \| null;"` → `"settingsDraft: HubSettingsSummary \| null;"` |

#### M3 实施步骤

1. **demoMode 徽标（Rust 文案 key）**：`ui_text.rs` 增字段与构造（上文形状）。验证：`cargo check -p zircon_hub --locked`；`cargo test -p zircon_hub --test ui_foundation_contract --locked`（应仍通过——断言为 assert_contains_all，加字段是增量安全）。
2. **demoMode 徽标（前端）**：`types/hub.ts` HubShellText 增 `demoModeBadge`；`hubData.ts` ui.shell 增 key；`TopBar.tsx:109-121` 插入条件徽标。刷新契约：`tauri_react_shell_contract.rs:413-431` TopBar snippet 表追加 `"state.demoMode ? <StatusBadge label={state.ui.shell.demoModeBadge} tone=\"warning\" /> : null"`；`ui_foundation_contract.rs:796-817` hubData 块追加 `"demoModeBadge:"`。验证：`cd zircon_hub && npm run typecheck && npm run build`；`cargo test -p zircon_hub --test tauri_react_shell_contract --locked`；`cargo test -p zircon_hub --test ui_foundation_contract --locked`。手测：浏览器直开 `npm run dev`（无 Tauri runtime 走 fallback）确认 TopBar 出现「演示数据」徽标；`npm run tauri:dev` 确认真实后端态无徽标。
3. **类型清理（DTO 入站字段）**：按清理矩阵修改 `types/hub.ts` 的 HubTaskSummary/HubProjectDetail/HubProjectTemplate/HubSourceBuildHistoryItem/HubActionHistoryItem/HubShellState；`hubData.ts` 增 `settingsDraft: null`。同变更刷新三处契约断言原文：`tauri_react_shell_contract.rs:581`、`ui_foundation_contract.rs:789`（`"selectedProject?: HubProjectDetail | null"` → `"selectedProject: HubProjectDetail | null"`）、`project_workflow_contract.rs:722`（settingsDraft 同法）。验证：`cd zircon_hub && npm run typecheck`（重点看 `??`/`?.` 链是否仍然类型成立——`selectedProject ?? null` 类用法不受影响）→ `npm run build`。
4. **类型清理（出站 payload）**：`CreateProjectPayload/NewProjectDraftPayload.engineId` 改必填 `string | null`（调用点已显式传值，CreateProjectDialog 装配处无需改）；`ImportProjectPayload.engineId` 改 `?: string`。验证：`cd zircon_hub && npm run typecheck && npm run build`。
5. **护栏负路径手测 + 全量回归**：devtools 中对 `hub-state-changed` 伪造缺 `recentProjects` 的载荷，确认 console 出现 `Hub state field 'recentProjects' must be an array` 且 UI 状态不变；然后：
   ```bash
   cd zircon_hub && npm run typecheck && npm run build
   cargo test -p zircon_hub --locked
   cargo fmt --all --check
   ```

#### M3 契约联动

| 测试 | 现有断言原文 | 改成 |
|------|--------------|------|
| `ui_foundation_contract.rs:789` | `"selectedProject?: HubProjectDetail \| null;"` | `"selectedProject: HubProjectDetail \| null;"` |
| `ui_foundation_contract.rs:796-817` | hubData 块含 `"demoMode: true"`、`"projects: []"`、`"selectedProject: null"` | 保留；追加 `"demoModeBadge:"` 与 `"settingsDraft: null"` |
| `tauri_react_shell_contract.rs:581` | `"selectedProject?: HubProjectDetail \| null"` | `"selectedProject: HubProjectDetail \| null"` |
| `tauri_react_shell_contract.rs:413-431` | TopBar snippet 表（`"SourceEnginePopover"` 等） | 追加徽标行断言（见实施步骤 2） |
| `project_workflow_contract.rs:722` | `"settingsDraft?: HubSettingsSummary \| null;"` | `"settingsDraft: HubSettingsSummary \| null;"` |
| `project_source_engine_contract.rs:314-321` | hubData 块含 `"demoMode: true"`、`"activeSourceEngineId: null"`、`"sourceEngines: []"` | **零改动** |
| 新增断言建议 | — | `ui_foundation_contract.rs` 的 types 断言块（772-794）追加 `"demoMode?: boolean;"`（锁定「后端不发送、字段保持可选」决策）；`tauri_react_shell_contract.rs` TopBar 块即为徽标行为锁 |

## 风险与协调

- 契约测试大量以源文本匹配组件文件内容：每个拆分切片必须同变更刷新断言，建议每切片后立即跑对应契约而非攒到里程碑末（M2 实施步骤已按此组织，并附 `rg` 复核步骤）。
- M3 收缩 fallback 与 `tauri-react-shell.md` 的"React fallback state mirrors that full row set"约束有张力：Settings health 行集合必须保留在骨架状态中，只删演示项目数据。【2026-06-12 复核：已满足——`hubData.ts:237-294` 保留 7 行 health rows，演示项目已删空】
- 与 03 计划 M2（fixture 剥离）联动：【2026-06-12 修订】原表述「截图流程改 seeded config 后本计划才可安全删除前端演示数据；顺序上 03.M2 先行或同批」的顺序依赖已大半解除——经实仓核实，两个截图脚本本就 seeded config 且真实落盘项目目录，且 `hubData.ts` 演示数据已被并行进程删除。剩余联动仅为：03.M2 若调整 seeded 项目集，截图基线变化与本计划无关，无顺序约束。
- 与 06 计划分工（拆分边界以本计划为准，06 在其上做样式细节）：本计划 M2 **不触碰** dashboard 卡片网格分支（`ProjectsDashboard.tsx:230-249`，06.M3 将以 `ProjectCardRail` 收编）、`ProjectTable`（06.M3 加 `onRowMenu`）、`NavigationDrawer`/`HubWindow` 调用行（06.M3 props 扩展并刷新 `ui_global_rules_contract.rs:328` 附近断言）；`ProjectsToolbar` 的边界是工具栏 grid 行（148-208 行），与 `ProjectCardRail` 无交叠。06.M3 还计划给 dashboard 增加 HubStatusBanner 与行菜单状态——均在本计划编辑区域之外。若 06 先行落地导致 dashboard 行号漂移，以组件/标识符定位。
- 【2026-06-12 现状修订注记】并行进程已落地：HubWindow 路由表（原文档「7 分支三元链」失效）、根级 HubErrorBoundary（挂载在 App.tsx 而非原计划的 main.tsx；文案走 `ui.shell` 既有 key 而非 `ui.common`）、`assertHubShellState`（hubApi 三函数均已接入）、`hubData.ts` 收缩至 661 行（原 995 行表述失效）。本计划 M1 与 M3 切片 1/2（除徽标）已改为盘点验收口径；对应契约（`ui_shell_composition_contract` 等）也已是新断言形态，M1 不再有契约刷新工作。
- 07 计划联动（仅注记，不改 07 文档）：本计划新增的唯一 Rust 文案 key 为 `HubShellText.demo_mode_badge`（"Demo Data"/"演示数据"，`ui_text.rs`），需在 07 本地化 schema 化时纳入 key 清单；ErrorBoundary 复用既有四 key，无新增。
- M2 把 `HUB_ACTION` 分发下放到 `ProjectDetailSidebar`/`SettingsSection` 两个 data 组件（业务复合组件），与"展示原子组件只收回调"的现有惯例（ProjectTable/QuickActions）形成两层；这是为保住契约分发断言原文、避免 10+ 回调 props 的权衡，已是定稿决策，06 及后续计划不应再回摆。
- `ui_panel_slot_contract` 的 HubPanel 计数循环改读「页面+组件拼接源」后，阈值语义从"页面内计数"变为"页面组合面计数"；若后续再拆组件，沿用拼接法并保持阈值不降。
