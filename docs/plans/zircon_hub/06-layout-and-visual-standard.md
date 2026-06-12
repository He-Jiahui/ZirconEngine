---
related_code:
  - zircon_hub/web/src/theme/tokens.ts
  - zircon_hub/web/src/theme/muiTheme.ts
  - zircon_hub/web/src/components/shell/TopBar.tsx
  - zircon_hub/web/src/components/shell/NavigationDrawer.tsx
  - zircon_hub/web/src/components/shell/HubWindow.tsx
  - zircon_hub/web/src/components/data/ProjectCard.tsx
  - zircon_hub/web/src/components/data/ProjectTable.tsx
  - zircon_hub/web/src/components/data/ProjectCover.tsx
  - zircon_hub/web/src/components/data/StatusBadge.tsx
  - zircon_hub/web/src/components/data/QuickActions.tsx
  - zircon_hub/web/src/components/inputs/HubButton.tsx
  - zircon_hub/web/src/components/inputs/HubIconButton.tsx
  - zircon_hub/web/src/components/overlays/HubMenu.tsx
  - zircon_hub/web/src/components/overlays/UserMenuPopover.tsx
  - zircon_hub/web/src/pages/ProjectsDashboard.tsx
  - zircon_hub/web/src/types/hub.ts
  - zircon_hub/web/src/data/hubData.ts
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/view_model/ui_text.rs
  - zircon_hub/src/state/task_status.rs
  - zircon_hub/tests/ui_visual_standard_contract.rs
  - zircon_hub/tests/ui_workspace_split_contract.rs
  - zircon_hub/tests/ui_project_layout_contract.rs
  - zircon_hub/tests/ui_shell_header_contract.rs
  - zircon_hub/tests/ui_project_browser_table_contract.rs
  - zircon_hub/tests/ui_global_rules_contract.rs
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-visual-state-matrix.ps1
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-project-pages.ps1
plan_sources:
  - docs/plans/zircon_hub/index.md
  - docs/plans/zircon_hub/05-frontend-componentization-and-type-safety.md
  - docs/zircon_hub/ui/responsive-component-system.md
status: planned
---

# 06 布局与视觉标准对齐（参考图基线）

## 现状与证据

参考基线：深色青绿主题的桌面 Hub 参考图（设计稿）——顶栏（品牌 + 引擎版本下拉 + Running/Success/Warning/Error 状态 chips + 通知/帮助/设置图标 + 用户菜单 + 窗口控制），左侧常驻导航（页面列表 + Engine Status 面板 + Check for Updates + Collapse），Projects 页（标题/副标题 + Import/New Project 双按钮 + 搜索框 + 筛选/排序下拉 + 网格/列表视图切换 + 项目卡片横排 rail（封面、引擎徽标、名称、路径、修改时间、版本/平台 chips、角部菜单、右缘滚动箭头）+ 下方 Recent Projects 表与 Quick Actions 面板两栏）。整体骨架已实现，差距在细节：

- 断点跳变：卡片 rail `repeat(4, minmax(220px, 296px))` 在 1360px / 1080px 直接跳列数，过渡突兀（`ProjectsDashboard.tsx` 卡片网格）。
- 窄屏溢出：TopBar `gridTemplateColumns: "222px minmax(250px, 1fr) auto"` 的 250px 下限在折叠态可能挤出窗口（`TopBar.tsx:59` 附近）。
- `ProjectTable` `tableLayout: fixed` + 百分比列宽，缺外层横向滚动容器，极窄时文本硬截断。
- token 旁路（2026-06-12 复核后的实仓清单）：`muiTheme.ts:13` 硬编码 `contrastText: "#071515"`（原文误记在 HubButton.tsx，已修正，见风险章节注记）；`HubButton.tsx` 硬编码 `"#eefefe"`（13 行）、`"#292929"`（26 行）、`"#ffd8d5"`（40 行）；`HubIconButton.tsx` 硬编码 `"#eefefe"`（22 行）、`"#292929"`（27 行）；`TopBar.tsx:152` 与 `UserMenuPopover.tsx:31` 硬编码 Avatar `bgcolor: "#4b4f52"`；`ProjectCover.tsx:21` 硬编码 `"#141414"`；`HubWindow.tsx:31` 硬编码背景渐变 `#161616/#111111`；`StatusBadge.tsx:71` 与 `NavigationDrawer.tsx:120` 硬编码 `borderRadius: 999`，均绕过 `tokens.ts`。
- 参考图细节缺口需逐项核对：卡片右缘的 rail 滚动箭头、卡片封面上的引擎徽标定位、Recent Projects 表行的悬停/角部菜单、Quick Actions 行的图标+标题+描述+chevron 结构、空态与错误态版式。

> 落地状态终核（2026-06-12）：本章四条「差距」在文档写成后已被工作树并行进程基本清零——卡片网格已改 auto-fill 并收编进新组件 `ProjectCardRail.tsx`；TopBar 中列已是 `minmax(0, 1fr)`（`TopBar.tsx:61`）；`ProjectTable` 已外包 `overflowX: "auto"` 容器并加 `minWidth: 560`；上列 token 旁路裸 hex 已全部迁入 `tokens.ts`（新增键名以实仓为准：`textOnPrimary`、`panelHover`、`dangerText`、`avatar`、`coverBackdrop`、`tooltip`、`gradients.window`、`radius.pill`），`StatusBadge.tsx`/`NavigationDrawer.tsx` 的 `borderRadius: 999` 已改 `hubTokens.radius.pill`。本章「现状」口吻的行号与字面量均为落地前基线，核对时以各里程碑的终核注记为准。

## 目标

1. 响应式平滑：卡片 rail 改 `repeat(auto-fill, minmax(clamp(220px, 22vw, 296px), 1fr))` 类自适应（或保留固定档位但补 1080–1360 间的中间档），消除可见跳变；TopBar 中列下限改 `minmax(0, 1fr)` 并给搜索区内部留最小宽度。
2. 溢出治理：`ProjectTable` 外包 `overflow-x: auto` 容器；所有两栏 `minmax(0, 1fr)` 网格复核 `minWidth: 0` 链路完整（契约 `ui_workspace_split_contract` 的共享栅格规则保持）。
3. token 一元化：色值、圆角、阴影、间距全部经 `tokens.ts` / `muiTheme.ts`；新增 `radius.pill`、按钮对比色等缺失 token，删除组件内字面量；`ui_visual_standard_contract` 增补"组件内禁止裸十六进制色值"类源断言（白名单 tokens/theme 两文件）。
4. 参考图细节补齐（按页核对清单执行）：
   - Projects：卡片 rail 滚动箭头与渐隐遮罩；卡片 hover 提升；版本 chip（青绿描边）与平台 chip（中性）双色制；Recent 表角部菜单接 pin/detail/delete 动作。
   - 顶栏：状态 chips 仅在对应状态存在时点亮（数据驱动，不常亮演示四色）；引擎版本下拉为真实 Source Engine 选择器。
   - 侧栏：Engine Status 面板（状态点 + 版本 + Up to date / Check for Updates）数据驱动；折叠态图标列对齐。
   - 全页面：空态用 `EmptyStateBlock` 统一版式；运行中任务在状态横幅 + 任务行呈现进度。
5. 截图验收矩阵固化为可重复流程：seeded config（03.M2 产物）+ WebView 动作捕获（沿用 `tauri-react-shell.md` 既有截图方案），覆盖 Projects / New Project / Project Detail / Editor / Builds / Cloud / Settings × 中文默认 / 错误 / 运行中 / 空态。

## 非目标

- 不改主题方向与既有 token 值体系（深色青绿基线已与参考图一致）。
- 不做移动端适配：最小支持宽度按桌面窗口下限（tauri.conf.json 窗口约束）定稿，不为 <760px 真机场景投入。
- 不重绘图标/封面资产。

## 里程碑

### M1 溢出与断点修复

切片：响应式平滑（目标 1）+ 溢出治理（目标 2），逐组件小步提交；每步跑对应 ui 契约。

> 通用注意：下文行号为 2026-06-12 基线；按 05 计划定稿分工，05 不触碰 dashboard 卡片网格 / `ProjectTable` / `NavigationDrawer`（均归 06 自管），与 05 的顺序协调仅剩其余拆分引起的行号漂移，漂移时以引用的函数名/标识符定位。前端 `npm run typecheck` / `npm run build` 一律在 `zircon_hub/`（`package.json` 所在目录，非 `zircon_hub/web/`）下执行。
>
> 落地状态终核（2026-06-12）：M1 三处代码改动与对应契约刷新均已被并行进程落地，本里程碑转为「盘点补缺/验收」口径。剩余工作仅：①契约联动末条的两处防回归 `assert_not_contains_any` 断言；②步骤 4 的手工窗口拖动验收；③ `ui_visual_standard_contract.rs` ProjectTable 段的 `overflowX` 断言增补。

#### M1 目标代码形状

（1）卡片网格平滑（`ProjectsDashboard.tsx` 网格视图分支）。现为 `repeat(4, minmax(220px, 296px))` + 1360/1080/760 三档媒体查询硬跳变（231-247 行附近），改为 auto-fill 单声明，删除该网格上的全部三个媒体查询（页面其余 `@media (max-width:` 块保留，满足 `ui_visual_standard_contract` 的页面响应式断言）：

```tsx
// ProjectsDashboard.tsx —— viewMode === "grid" 分支的卡片容器
<Box
  sx={{
    display: "grid",
    gridTemplateColumns: "repeat(auto-fill, minmax(clamp(220px, 22vw, 296px), 1fr))",
    gap: 2,
    mt: 2.3,
  }}
>
  {dashboardProjects.map((project) => (
    <ProjectCard ... />
  ))}
</Box>
```

说明：窗口最小宽 960（`tauri.conf.json` `minWidth: 960`），侧栏 222/78，主区最小约 730px；`clamp(220px, 22vw, 296px)` 在 1000-1568px 区间使列宽连续过渡，auto-fill 自动决定 2/3/4 列，无可见跳变，760px 以下场景不存在（非目标已声明）。

> 落地状态终核（2026-06-12）：已落地，转盘点验收。auto-fill 网格并未留在 `ProjectsDashboard.tsx`，而是随 M3 S1-a 一并收编进 `web/src/components/data/ProjectCardRail.tsx:20`，原 `repeat(4, ...)` 三档媒体查询已删除；页面其余 `@media (max-width:` 块保留（现存 980px/1180px 两处）。遗留失配：`ui_project_layout_contract.rs:110` 与 `ui_visual_standard_contract.rs:594` 的 auto-fill 断言已刷新为新串，但仍读取 `ProjectsDashboard.tsx`，与实仓（串已移入 ProjectCardRail）不一致、预计为红，需按 M3 契约联动完成断言落点迁移。

（2）TopBar 中列下限归零（`TopBar.tsx:59` 附近）。现为 `gridTemplateColumns: "222px minmax(250px, 1fr) auto"`，改为：

```tsx
gridTemplateColumns: "222px minmax(0, 1fr) auto",
```

搜索/选择区内部最小宽度已存在：引擎选择 `ButtonBase` 自带 `width: 190, minWidth: 160`（84-85 行附近），中列容器已有 `minWidth: 0, overflow: "hidden"`（80 行附近），状态 chips 容器在 `@media (max-width: 1260px)` 下隐藏（109-121 行附近），三者保持不动。980px 媒体分支本就是 `"78px minmax(0, 1fr) auto"`（63-65 行附近），不改。

> 落地状态终核（2026-06-12）：已落地，转盘点验收——`TopBar.tsx:61` 已为 `"222px minmax(0, 1fr) auto"`，980px 分支不变（现 65-67 行），引擎选择 `ButtonBase` 现位于 86-87 行；`ui_visual_standard_contract.rs:298` 与 `ui_shell_header_contract.rs:71` 断言均已同步刷新。

（3）`ProjectTable` 外包横向滚动容器（`ProjectTable.tsx`）。现为裸 `<Table size="small" sx={{ tableLayout: "fixed" }}>`（24 行附近），改为：

```tsx
// import 行（现第 3 行）追加 Box：
import { Box, IconButton, Table, TableBody, TableCell, TableHead, TableRow, Typography } from "@mui/material";

export function ProjectTable({ projects, selectedProjectId, labels, onSelect, onOpenDetail }: ProjectTableProps) {
  return (
    <Box sx={{ overflowX: "auto", minWidth: 0 }}>
      <Table size="small" sx={{ tableLayout: "fixed", minWidth: 560 }}>
        {/* TableHead / TableBody 原样不动 */}
      </Table>
    </Box>
  );
}
```

`minWidth: 560` 取自列最小可读宽度（32%+18%+16%+location+42px 操作列在 560px 时仍不挤压缩略图行高 36）；窄于 560 时容器内滚动而非页面级滚动，`HubPanel` 自身 `overflow: "hidden"`（契约锁定）保证滚动条留在面板内。两栏 `minmax(0, 1fr)` 栅格与 `gap: 1.4`、1180px 折叠规则由 `ui_workspace_split_contract` 锁定，本切片不触碰。

> 落地状态终核（2026-06-12）：已落地，转盘点验收——`ProjectTable.tsx:25-26` 即上述目标形状（含 `Box` import 与 `minWidth: 560`）；`ui_project_browser_table_contract.rs:60/66/67` 已含新断言（import 行、`overflowX` 容器行、`minWidth: 560` Table 行）。补缺项：`ui_visual_standard_contract.rs` 的 ProjectTable 段（现 555-560 行附近）尚未增补 `overflowX` 断言。

#### M1 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/web/src/pages/ProjectsDashboard.tsx` | 修改 | 卡片网格改 auto-fill+clamp，删除 1360/1080/760 三档媒体查询 |
| `zircon_hub/web/src/components/shell/TopBar.tsx` | 修改 | 顶栏中列 `minmax(250px, 1fr)` → `minmax(0, 1fr)` |
| `zircon_hub/web/src/components/data/ProjectTable.tsx` | 修改 | 表格外包 `overflowX: "auto"` Box，Table 加 `minWidth: 560` |
| `zircon_hub/tests/ui_project_layout_contract.rs` | 修改 | 刷新卡片网格三条断言为 auto-fill 单条 |
| `zircon_hub/tests/ui_visual_standard_contract.rs` | 修改 | 刷新 TopBar 网格、ProjectsDashboard 卡片网格断言；ProjectTable 增 `overflowX` 断言 |
| `zircon_hub/tests/ui_shell_header_contract.rs` | 修改 | 刷新 TopBar 网格断言 |
| `zircon_hub/tests/ui_project_browser_table_contract.rs` | 修改 | 刷新 ProjectTable import 行与 Table 声明断言，增滚动容器断言 |

> 落地状态终核（2026-06-12）：上表 7 行除 `ui_visual_standard_contract.rs` 的 ProjectTable `overflowX` 增补外均已落地（`ProjectsDashboard.tsx` 的卡片网格实际由新建的 `ProjectCardRail.tsx` 承载，断言落点迁移归 M3）。

#### M1 实施步骤

1. 改 `ProjectsDashboard.tsx` 卡片网格（231-247 行附近）：替换 `gridTemplateColumns` 为 auto-fill 形态，删除三个 `@media` 覆盖；同变更刷新 `ui_project_layout_contract.rs:110-112` 与 `ui_visual_standard_contract.rs:527` 的网格断言（见契约联动）。验证：`zircon_hub/` 下 `npm run typecheck && npm run build`；`cargo test -p zircon_hub --test ui_project_layout_contract --locked`、`cargo test -p zircon_hub --test ui_visual_standard_contract --locked`。
2. 改 `TopBar.tsx:59` 网格声明；同变更刷新 `ui_visual_standard_contract.rs:235` 与 `ui_shell_header_contract.rs:71`。验证：`npm run typecheck`；`cargo test -p zircon_hub --test ui_shell_header_contract --locked`、`cargo test -p zircon_hub --test ui_visual_standard_contract --locked`。
3. 改 `ProjectTable.tsx`：import 行加 `Box`（第 3 行），根节点外包滚动容器（24 行附近）；同变更刷新 `ui_project_browser_table_contract.rs:60/66` 与 `ui_visual_standard_contract.rs` ProjectTable 段。验证：`npm run typecheck && npm run build`；`cargo test -p zircon_hub --test ui_project_browser_table_contract --locked`、`cargo test -p zircon_hub --test ui_workspace_split_contract --locked`。
4. 手工窗口拖动验收（测试阶段既有口径）：`npm run tauri:dev` 或既有 fast-build 产物，从 1568 拖到 960 宽，确认页面级无横向滚动条、表格滚动条只出现在面板内、顶栏无溢出。

> 落地状态终核（2026-06-12）：步骤 1-3 已落地（含契约刷新；步骤 1 断言现位于 `ui_project_layout_contract.rs:110` 与 `ui_visual_standard_contract.rs:594`，步骤 2 断言现位于 `ui_visual_standard_contract.rs:298`）。步骤 4 手工拖动验收未见记录，保留执行。

#### M1 契约联动

- `ui_project_layout_contract.rs`（110-112 行附近）现有断言原文：
  - `"gridTemplateColumns: \"repeat(4, minmax(220px, 296px))\""`
  - `"gridTemplateColumns: \"repeat(3, minmax(220px, 1fr))\""`
  - `"gridTemplateColumns: \"repeat(2, minmax(220px, 1fr))\""`
  三条删除，改为一条：`"gridTemplateColumns: \"repeat(auto-fill, minmax(clamp(220px, 22vw, 296px), 1fr))\""`。
- `ui_visual_standard_contract.rs:527` 现有断言原文 `"gridTemplateColumns: \"repeat(4, minmax(220px, 296px))\""` → 同上 auto-fill 串。
- `ui_visual_standard_contract.rs:235` 与 `ui_shell_header_contract.rs:71` 现有断言原文 `"gridTemplateColumns: \"222px minmax(250px, 1fr) auto\""` → `"gridTemplateColumns: \"222px minmax(0, 1fr) auto\""`。
- `ui_project_browser_table_contract.rs:60` 现有断言原文 `"import { IconButton, Table, TableBody, TableCell, TableHead, TableRow, Typography } from \"@mui/material\";"` → `"import { Box, IconButton, Table, TableBody, TableCell, TableHead, TableRow, Typography } from \"@mui/material\";"`；同文件 66 行 `"<Table size=\"small\" sx={{ tableLayout: \"fixed\" }}>"` → `"<Table size=\"small\" sx={{ tableLayout: \"fixed\", minWidth: 560 }}>"`，并新增 `"overflowX: \"auto\""` 断言。
- `ui_visual_standard_contract.rs` ProjectTable 段（491-499 行附近，现含 `"Table size=\"small\""`、`"tableLayout: \"fixed\""`）新增 `"overflowX: \"auto\""`。
- 新增断言（防回归）：在 `ui_shell_header_contract.rs` 的 `topbar_owns_brand_engine_status_user_and_window_control_regions` 中加 `assert_not_contains_any` 项 `"minmax(250px, 1fr)"`；在 `ui_project_layout_contract.rs` 中加 `assert_not_contains_any` 项 `"repeat(4, minmax(220px, 296px))"`。

> 落地状态终核（2026-06-12）：前五条「现有断言原文 → 改为」均已完成，断言现行位置：layout `:110`、visual standard `:594`（卡片网格）与 `:298`（TopBar）、shell header `:71`、browser table `:60/66/67`；visual standard 的 ProjectTable 段现位于 555-560 行附近（基线 491-499，已漂移），`overflowX` 增补仍缺。末条「新增断言（防回归）」两处 `assert_not_contains_any` 实测均未落地，为本里程碑剩余补缺项。

测试阶段：
- `npm run typecheck && npm run build`；`cargo test -p zircon_hub ui_workspace_split --locked`、`ui_project_layout`。
- 手工：窗口从最大拖到最小宽度，记录无横向滚动条（页面级）与无内容裁切。

### M2 token 一元化与守卫

切片：盘点 `rg '#[0-9a-fA-F]{6}' web/src/components web/src/pages` 全部命中迁入 tokens；`ui_visual_standard_contract` 增补裸色值守卫断言。

> 盘点结果（2026-06-12 实测，`web/src/components` + `web/src/pages` 共 10 处裸 hex）：`TopBar.tsx:152`、`UserMenuPopover.tsx:31`（`#4b4f52`）；`HubWindow.tsx:31`（`#161616`/`#111111`）；`HubIconButton.tsx:22/27`（`#eefefe`/`#292929`）；`HubButton.tsx:13/26/40`（`#eefefe`/`#292929`/`#ffd8d5`）；`ProjectCover.tsx:21`（`#141414`）。`pages/` 目录无命中。另：`rgba(...)` 字面量在两目录共 83 处，多为 hover/选中蒙层且被契约源文本断言密集锁定，本里程碑明确不迁移（守卫只拦裸 hex），与目标 3 的"全部经 tokens"以 hex 口径收口。
>
> 落地状态终核（2026-06-12）：上列 10 处裸 hex 已全部迁入 token（两目录现裸 hex 为 0；`rgba(...)` 实测仍 83 处，维持不迁移口径）。守卫断言已落地，但名称与形状以实仓为准：`ui_visual_standard_contract.rs:133` `component_and_page_styles_do_not_bypass_visual_tokens`——单测试函数，递归收集两目录 `.tsx`（不含 `.ts`），以 `windows(4)` 判定 `#` 后连续 3 位 hex 即违例，并额外拦截 `borderRadius: 999`；非下文提案的三函数形状。本里程碑转「盘点补缺/验收」口径，唯一契约补缺见 M2 契约联动注记。

#### M2 目标代码形状

（1）`tokens.ts` 扩展（基线为 radius 三键、colors 15 键；实仓现已扩展完毕：radius 六键 11-18 行、colors 21 键 19-41 行、gradients 42-45 行、shadows 46-49 行）：

```ts
export const hubTokens = {
  window: {
    // 现有 7 键不动
  },
  radius: {
    compact: 7,
    panel: 8,
    card: 8,
    thumb: 4,
    brandMark: 6,
    pill: 999,
  },
  colors: {
    // 现有 15 键不动，追加：
    textOnAccent: "#eefefe",
    textOnPrimary: "#071515",
    dangerText: "#ffd8d5",
    panelHover: "#292929",
    avatar: "#4b4f52",
    coverBackdrop: "#141414",
    tooltip: "#242424",
  },
  gradients: {
    window:
      "radial-gradient(circle at 30% 18%, rgba(38,86,82,0.13), transparent 30%), linear-gradient(180deg, #161616 0%, #111111 100%)",
  },
  shadows: {
    // 现有 2 键不动
  },
} as const;
```

> 落地状态终核（2026-06-12）：`tokens.ts` 已按上述形状落地。键名以实仓为准（本文已同步改写）：原提案的 `accentContrastText`/`hoverPanel` 实仓定名为 `textOnPrimary`/`panelHover`；`radius.thumb`/`radius.brandMark` 为并行落地的既有键，一并列出。

（2）调用点替换（与现状的差异点）：

```tsx
// HubButton.tsx —— 现 13/26/40 行字面量，改为：
primary: { color: hubTokens.colors.textOnAccent, ... "&:hover": { ... } },
secondary: { ... "&:hover": { backgroundColor: hubTokens.colors.panelHover, ... } },
danger: { color: hubTokens.colors.dangerText, ... },

// HubIconButton.tsx —— 现 22/27 行，改为：
color: selected ? hubTokens.colors.textOnAccent : hubTokens.colors.textSoft,
backgroundColor: hubTokens.colors.panelHover,  // hover 分支

// TopBar.tsx:152 / UserMenuPopover.tsx:31 —— Avatar：
<Avatar sx={{ width: 36, height: 36, bgcolor: hubTokens.colors.avatar, fontSize: 14 }}>
// UserMenuPopover 为 width/height 38，同样仅替换 bgcolor。

// ProjectCover.tsx:21：
backgroundColor: hubTokens.colors.coverBackdrop,

// HubWindow.tsx:31 —— 整段 background 串改引 token：
background: hubTokens.gradients.window,

// muiTheme.ts:13 / 101 / 129（白名单文件内的顺手一元化）：
contrastText: hubTokens.colors.textOnPrimary,
backgroundColor: hubTokens.colors.panel,        // MuiMenu paper，现 "#202020"
backgroundColor: hubTokens.colors.tooltip,      // MuiTooltip，现 "#242424"

// StatusBadge.tsx:71 / NavigationDrawer.tsx:120 —— 圆点：
borderRadius: hubTokens.radius.pill,
```

> 落地状态终核（2026-06-12）：上列调用点替换已全部落地，转盘点验收（现行位置：`HubButton.tsx:13/26/40`、`HubIconButton.tsx:22/27`、`TopBar.tsx:155`、`UserMenuPopover.tsx:32`、`ProjectCover.tsx:22`、`HubWindow.tsx:48`、`muiTheme.ts:13/101/129`、`StatusBadge.tsx:71`、`NavigationDrawer.tsx:136`）。

（3）守卫断言（落在 `ui_visual_standard_contract.rs`，纯 std、零新依赖，白名单 = 只扫 `web/src/components` 与 `web/src/pages` 两目录，`theme/`、`styles.css`、`data/` 天然不在扫描范围）：

```rust
#[test]
fn components_and_pages_keep_color_literals_in_theme_tokens() {
    let mut offenders = Vec::new();
    for root in ["web/src/components", "web/src/pages"] {
        collect_bare_hex_offenders(&crate_dir().join(root), &mut offenders);
    }
    assert!(
        offenders.is_empty(),
        "bare hex color literals must move into web/src/theme/tokens.ts: {offenders:?}"
    );
}

fn collect_bare_hex_offenders(dir: &Path, offenders: &mut Vec<String>) {
    for entry in fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to list {}: {error}", dir.display()))
    {
        let path = entry.expect("hub web source dir entry").path();
        if path.is_dir() {
            collect_bare_hex_offenders(&path, offenders);
            continue;
        }
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            continue;
        };
        if !matches!(extension, "ts" | "tsx") {
            continue;
        }
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for (index, line) in source.lines().enumerate() {
            if line_has_bare_hex_color(line) {
                offenders.push(format!("{}:{}", path.display(), index + 1));
            }
        }
    }
}

fn line_has_bare_hex_color(line: &str) -> bool {
    let bytes = line.as_bytes();
    for (position, byte) in bytes.iter().enumerate() {
        if *byte != b'#' {
            continue;
        }
        let run = bytes[position + 1..]
            .iter()
            .take_while(|candidate| candidate.is_ascii_hexdigit())
            .count();
        let terminated = !bytes
            .get(position + 1 + run)
            .is_some_and(|next| next.is_ascii_alphanumeric());
        if matches!(run, 3 | 4 | 6 | 8) && terminated {
            return true;
        }
    }
    false
}
```

> 落地状态终核（2026-06-12）：守卫已以不同形落地（`component_and_page_styles_do_not_bypass_visual_tokens`，详见 M2 切片注记），上述三函数代码形状仅作历史提案保留，不再实施。

#### M2 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/web/src/theme/tokens.ts` | 修改 | 追加 `radius.pill`、7 个色 token、`gradients.window` |
| `zircon_hub/web/src/theme/muiTheme.ts` | 修改 | `contrastText`/MuiMenu paper/MuiTooltip 三处字面量改引 token |
| `zircon_hub/web/src/components/inputs/HubButton.tsx` | 修改 | 3 处裸 hex 改 token |
| `zircon_hub/web/src/components/inputs/HubIconButton.tsx` | 修改 | 2 处裸 hex 改 token |
| `zircon_hub/web/src/components/shell/TopBar.tsx` | 修改 | Avatar `#4b4f52` 改 `colors.avatar` |
| `zircon_hub/web/src/components/overlays/UserMenuPopover.tsx` | 修改 | Avatar `#4b4f52` 改 `colors.avatar` |
| `zircon_hub/web/src/components/shell/HubWindow.tsx` | 修改 | 背景渐变改 `gradients.window` |
| `zircon_hub/web/src/components/data/ProjectCover.tsx` | 修改 | `#141414` 改 `colors.coverBackdrop` |
| `zircon_hub/web/src/components/data/StatusBadge.tsx` | 修改 | `borderRadius: 999` 改 `radius.pill` |
| `zircon_hub/web/src/components/shell/NavigationDrawer.tsx` | 修改 | 状态点 `borderRadius: 999` 改 `radius.pill` |
| `zircon_hub/tests/ui_visual_standard_contract.rs` | 修改 | 新增裸 hex 守卫测试 + 刷新被迁移字面量的源断言 + tokens 清单断言追加 |

> 落地状态终核（2026-06-12）：上表 11 行已全部落地（token 键名以实仓 `textOnPrimary`/`panelHover` 为准；守卫测试名为 `component_and_page_styles_do_not_bypass_visual_tokens`）。唯一残留补缺：muiTheme 段尚无 `contrastText: hubTokens.colors.textOnPrimary` 源断言（见契约联动注记）。

#### M2 实施步骤

1. 扩展 `tokens.ts`（radius/colors/gradients 三段追加，键序如上）；同变更在 `ui_visual_standard_contract.rs` 的 tokens 断言清单（现 167-211 行附近）追加 `"pill: 999"`、`"textOnAccent: \"#eefefe\""`、`"textOnPrimary: \"#071515\""`、`"dangerText: \"#ffd8d5\""`、`"panelHover: \"#292929\""`、`"avatar: \"#4b4f52\""`、`"coverBackdrop: \"#141414\""`、`"tooltip: \"#242424\""`、`"gradients:"`。验证：`npm run typecheck`；`cargo test -p zircon_hub --test ui_visual_standard_contract --locked`。
2. 迁移 `HubButton.tsx` / `HubIconButton.tsx` 字面量；刷新 `ui_visual_standard_contract.rs:364` 断言（见契约联动）。验证：`npm run typecheck`；`cargo test -p zircon_hub --test ui_visual_standard_contract --locked`、`cargo test -p zircon_hub --test ui_shell_header_contract --locked`。
3. 迁移 Avatar（TopBar/UserMenuPopover）、`ProjectCover`、`HubWindow` 渐变、`muiTheme.ts` 三处；刷新 `ui_visual_standard_contract.rs:223/244/316/439` 对应断言。验证：`npm run typecheck && npm run build`；`cargo test -p zircon_hub --test ui_visual_standard_contract --locked`、`cargo test -p zircon_hub --test ui_global_rules_contract --locked`。
4. `StatusBadge.tsx:71` 与 `NavigationDrawer.tsx:120` 改 `hubTokens.radius.pill`（两文件断言未锁 999，无契约刷新）。验证：`npm run typecheck`。
5. 在 `ui_visual_standard_contract.rs` 末尾新增守卫测试三函数（如上代码形状）。验证：`cargo test -p zircon_hub components_and_pages_keep_color_literals_in_theme_tokens --locked`。
6. 守卫演练：临时在任一组件加 `color: "#123456"`，确认 `cargo test -p zircon_hub components_and_pages_keep_color_literals_in_theme_tokens --locked` 报错并列出 `文件:行号`，随后撤销。

> 落地状态终核（2026-06-12）：步骤 1-5 已落地（步骤 1 的 tokens 断言追加键已全部在位，含 `"textOnPrimary: \"#071515\""` 等；步骤 5 的守卫以单函数 `component_and_page_styles_do_not_bypass_visual_tokens` 实现）。步骤 6 守卫演练未见记录，保留执行——演练时测试名以实仓守卫为准：`cargo test -p zircon_hub component_and_page_styles_do_not_bypass_visual_tokens --locked`。

#### M2 契约联动

- `ui_visual_standard_contract.rs:364` 现有断言原文 `"color: selected ? \"#eefefe\" : hubTokens.colors.textSoft"` → `"color: selected ? hubTokens.colors.textOnAccent : hubTokens.colors.textSoft"`。
- `ui_visual_standard_contract.rs:244` 现有断言原文 `"Avatar sx={{ width: 36, height: 36, bgcolor: \"#4b4f52\", fontSize: 14 }}"` → `"Avatar sx={{ width: 36, height: 36, bgcolor: hubTokens.colors.avatar, fontSize: 14 }}"`；316 行 UserMenuPopover 的 `"Avatar sx={{ width: 38, height: 38, bgcolor: \"#4b4f52\", fontSize: 14 }}"` 同形替换。
- `ui_visual_standard_contract.rs:439` 现有断言原文 `"backgroundColor: \"#141414\""` → `"backgroundColor: hubTokens.colors.coverBackdrop"`。
- `ui_visual_standard_contract.rs:222-224` HubWindow 段现有断言原文 `"radial-gradient(circle at 30% 18%, rgba(38,86,82,0.13), transparent 30%)"` 与 `"linear-gradient(180deg, #161616 0%, #111111 100%)"` → 合并为 `"background: hubTokens.gradients.window"`（渐变字面量改由 tokens 断言锁定，见步骤 1）。
- muiTheme 段（现 212-245 行附近）新增断言 `"contrastText: hubTokens.colors.textOnPrimary"`。
- 新增测试：`components_and_pages_keep_color_literals_in_theme_tokens`——断言要点：递归扫描两目录所有 `.ts/.tsx`，`#` 后接 3/4/6/8 位 hex 即违例，违例清单含 `文件:行号` 输出。
- 注意 `ui_inputs_contract.rs:250-262`、`ui_foundation_contract.rs:290-308`、`ui_shell_window_contract.rs:105-117` 对 tokens.ts 的 hex 断言均为 contains 型且只断既有键，token 追加不需刷新。

> 落地状态终核（2026-06-12）：上列断言替换均已完成，现行位置：HubIconButton 选中色断言 `:430`、TopBar Avatar `:307`、UserMenuPopover Avatar `:379`、ProjectCover `:506`、HubWindow 渐变 `:286`（`"background: hubTokens.gradients.window"`）。守卫测试已落地（名称/形状以实仓为准，见 M2 切片注记，上条「新增测试」提案不再实施）。`ui_inputs_contract.rs:251`、`ui_foundation_contract.rs:279-321`、`ui_shell_window_contract.rs:92-117` 实测确为 contains 型，token 追加未引发刷新，判断成立。唯一补缺：muiTheme 段（现 212-245 行附近）的 `"contrastText: hubTokens.colors.textOnPrimary"` 断言尚未新增。

测试阶段：
- `cargo test -p zircon_hub ui_visual_standard --locked`；故意在组件中加裸色值验证守卫报错后撤销。

### M3 参考图细节对齐与截图矩阵

切片：
1. 按目标 4 清单逐页核对实现，每页一个切片；按 05 计划定稿分工，dashboard 卡片网格 / `ProjectTable` / `NavigationDrawer` 归 06 自管（05 不触碰），其余组件拆分落点以 05.M2 为准（在新组件上改，不在旧内联块上改）。
2. 截图矩阵脚本化（seeded config + 既有 WebView 捕获辅助），产出存入设计验证目录并登记到 `docs/zircon_hub` 的 design reference matrix。

切片细分：S1 Projects 页（卡片 rail 箭头/遮罩 + Recent 表角部菜单）、S2 顶栏状态 chips 数据驱动、S3 侧栏 Engine Status 数据驱动 + 折叠对齐、S4 全页面空态/运行态复核、S5 截图矩阵脚本化。已达标项（核对即可、不动代码）：卡片版本 chip 青绿描边 + 平台 chip 中性双色已在 `ProjectCard.tsx:76-85` `chipSx("accent"|"neutral")` 实现；卡片 hover 提升已有 `transform: "translateY(-1px)"`（24-26 行附近）；QuickActions 行已是 `36px minmax(0, 1fr) 24px` 图标+标题/描述+chevron 结构（`QuickActions.tsx:40` 附近）；引擎版本下拉已是真实 `SourceEnginePopover` 选择器（`TopBar.tsx:174-190`，`HUB_ACTION.selectEngine` 直连）。

> 落地状态终核（2026-06-12）：S1-S4 的组件/后端代码改动已被并行进程全部落地（`ProjectCardRail.tsx` 新建并接入、`ProjectTable` `onRowMenu`、`HubRecentProject.pinned`、`header_statuses` 数据驱动、`NavigationDrawer` 数据驱动、dashboard `HubStatusBanner`），但配套契约刷新与 view_model 单测大半未跟上（详见各切片注记与契约联动注记），其中四处契约断言已与实仓源码失配、预计为红；S5 截图矩阵脚本化完全未落地。S1-S4 转「盘点补缺/验收」口径，剩余实施重心：契约断言迁移/增补 → view_model 单测 → S5 脚本。

#### M3 目标代码形状

（S1-a）卡片 rail 右缘箭头与渐隐遮罩——新建 `web/src/components/data/ProjectCardRail.tsx`，把 M1 落地的 auto-fill 网格收编进该组件；箭头语义定为"还有更多项目 → 查看全部"导流（dashboard 卡片数被 `slice(0, 4)` 契约锁定，不做横向滚动；该决策见风险注记）：

```tsx
import ChevronRightIcon from "@mui/icons-material/ChevronRight";
import { Box } from "@mui/material";
import type { ReactNode } from "react";
import { HubIconButton } from "../inputs";

export interface ProjectCardRailProps {
  children: ReactNode;
  moreLabel: string;
  hasMore: boolean;
  onMore: () => void;
}

export function ProjectCardRail({ children, moreLabel, hasMore, onMore }: ProjectCardRailProps) {
  return (
    <Box sx={{ position: "relative", minWidth: 0 }}>
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
            sx={{
              position: "absolute",
              top: 0,
              right: 0,
              bottom: 0,
              width: 72,
              pointerEvents: "none",
              background: "linear-gradient(90deg, rgba(17,18,18,0) 0%, rgba(17,18,18,0.82) 100%)",
            }}
          />
          <HubIconButton
            label={moreLabel}
            onClick={onMore}
            sx={{ position: "absolute", top: "50%", right: 10, transform: "translateY(-50%)", width: 38, height: 38 }}
          >
            <ChevronRightIcon />
          </HubIconButton>
        </>
      ) : null}
    </Box>
  );
}
```

`ProjectsDashboard.tsx` 网格视图分支改为（`moreLabel` 复用 DTO 文案 `actionText.viewAllProjects`，不新增前端文案）：

```tsx
<Box sx={{ mt: 2.3 }}>
  <ProjectCardRail
    moreLabel={actionText.viewAllProjects}
    hasMore={visibleProjects.length > dashboardProjects.length}
    onMore={() => void onAction(HUB_ACTION.viewAllProjects)}
  >
    {dashboardProjects.map((project) => (
      <ProjectCard key={project.id} project={project} selected={project.id === state.selectedProjectId} openDetailsLabel={text.openProjectDetailsLabel} onOpen={handleOpenProject} />
    ))}
  </ProjectCardRail>
</Box>
```

> 落地状态终核（2026-06-12）：S1-a 已落地，转盘点验收——`ProjectCardRail.tsx` 与 barrel 导出（`data/index.ts:7`）、`ProjectsDashboard.tsx:179-193` 集成均与上述提案同构。落地形态与提案的细节差异（以实仓为准）：渐隐遮罩宽 96（非 72）、渐变串为 `"linear-gradient(90deg, rgba(17,18,18,0), rgba(17,18,18,0.94))"`（无显式百分比停靠点）、遮罩用 `height: "100%"` 并带 `aria-hidden`、箭头按钮 `right: 8` 且带 `backgroundColor: hubTokens.colors.panel`、外层容器仅 `position: "relative"`（无 `minWidth: 0`）。补缺项：契约断言落点迁移（dashboard → ProjectCardRail）未完成，`ui_project_layout_contract.rs:110` 与 `ui_visual_standard_contract.rs:594` 仍读 `ProjectsDashboard.tsx` 断言 auto-fill 串，与实仓不一致、预计为红。

（S1-b）Recent 表角部菜单接 pin/detail/delete。后端先给行 DTO 补 `pinned`（现 `HubRecentProject` 六字段无 pinned，`view_model.rs:105-114`；`recent_project_row` 在 385-395 行附近，`metadata_for_path` 取法对齐 455 行附近的 `project_detail_from_parts`）：

```rust
// view_model.rs —— struct HubRecentProject 追加字段：
pub pinned: bool,

// recent_project_row 改为：
fn recent_project_row(snapshot: &HubSnapshot, project: &RecentProject) -> HubRecentProject {
    let summary = project_summary(snapshot, project, false, snapshot.settings.language);
    let pinned = metadata_for_path(&snapshot.project_metadata, &project.path)
        .is_some_and(|metadata| metadata.pinned);
    HubRecentProject {
        id: summary.id,
        name: summary.name,
        engine_version: summary.engine_version,
        modified: summary.modified,
        location: summary.path,
        cover_id: summary.cover_id,
        pinned,
    }
}
```

前端：`types/hub.ts` 的 `HubRecentProject`（29-36 行附近）追加 `pinned: boolean;`；`hubData.ts` fallback 的 `browserProjects`（107 行起）与 `recentProjects`（149 行起）各行补 `pinned: false`。`ProjectTable.tsx` 加可选回调（菜单状态归页面所有，data 组件不跨家族引 overlays）：

```tsx
export interface ProjectTableProps {
  // 既有五项不动，追加：
  onRowMenu?: (project: HubRecentProject, anchor: HTMLElement) => void;
}

// MoreVert IconButton 的 onClick 改为：
onClick={(event) => {
  event.stopPropagation();
  if (onRowMenu) {
    onRowMenu(project, event.currentTarget);
    return;
  }
  onOpenDetail?.(project);
}}
```

`ProjectsDashboard.tsx` 持有菜单状态并复用既有 `HubMenu`（`components/overlays/HubMenu.tsx`，现已导出未被页面消费）与既有 action 文案 key（`HubActionText.pinProject/unpinProject/requestDelete`，`types/hub.ts:327-331`）：

```tsx
const [rowMenu, setRowMenu] = useState<{ anchor: HTMLElement; project: HubRecentProject } | null>(null);
const rowMenuItems = (project: HubRecentProject): HubMenuItem[] => [
  { id: HUB_ACTION.openProjectDetail, label: text.openProjectDetailsLabel },
  project.pinned
    ? { id: HUB_ACTION.unpinProject, label: actionText.unpinProject }
    : { id: HUB_ACTION.pinProject, label: actionText.pinProject },
  { id: HUB_ACTION.requestDelete, label: actionText.requestDelete },
];
const handleRowMenuSelect = (project: HubRecentProject, itemId: string) => {
  if (itemId === HUB_ACTION.openProjectDetail) {
    void onAction(HUB_ACTION.openProjectDetail, project.id);
    return;
  }
  if (itemId === HUB_ACTION.pinProject || itemId === HUB_ACTION.unpinProject) {
    void onAction(itemId, undefined, { projectId: project.id });
    return;
  }
  void onAction(HUB_ACTION.requestDelete, undefined, { projectId: project.id });
};
// 两处 <ProjectTable ...> 均传 onRowMenu={(project, anchor) => setRowMenu({ anchor, project })}
// 页面末尾渲染：
{rowMenu ? (
  <HubMenu anchorEl={rowMenu.anchor} open items={rowMenuItems(rowMenu.project)} onClose={() => setRowMenu(null)} onSelect={(itemId) => handleRowMenuSelect(rowMenu.project, itemId)} />
) : null}
```

payload 走 `{ projectId: project.id }`，遵守 projectPath > projectId > targetId 解析顺序（行 DTO 无原始 path，稳定 projectId 即正确档位）；`request-delete` 后端只置 `pending_delete_project_path` + warning 任务态（`project_actions.rs:267-268` 附近），确认/取消仍走 Project Detail 既有流，全局 `HubSnackbar` 呈现 recovery 提示，协议不变。

> 落地状态终核（2026-06-12）：S1-b 前后端代码均已落地，转盘点验收——`view_model.rs:113-121` `HubRecentProject` 已含 `pinned`，`recent_project_row`（399-410 行）按 metadata 取值，与提案逐行同构；`types/hub.ts:38` 已加 `pinned: boolean;`；`hubData.ts` fallback 的 `browserProjects`/`recentProjects` 现为空数组 `[]`（演示行已被并行进程删除），无行对象需补 `pinned: false`，该子步骤口径作废；`ProjectTable.tsx:20/69-76` 的 `onRowMenu` 与回退路径、`ProjectsDashboard.tsx:37/61-79/247-255` 的菜单状态 + `HubMenu` 渲染均与提案同构（`HubMenu` 已被页面消费，原「导出未消费」现状陈述失效）。补缺项：`ui_project_browser_table_contract.rs` 尚无 `onRowMenu` 与 HubRecentProject `pinned` 断言（现有 `"pub pinned: bool"`/`"pinned: boolean;"` 断言位于 `ui_project_scope_contract.rs:120/184`，指向 `ProjectMetadata` 与 `HubProjectDetail`，非本切片对象）；`recent_project_rows_carry_pinned_metadata` 单测未补。

（S2）顶栏状态 chips 数据驱动。现 `header_statuses`（`view_model.rs:321-333`）恒返回四 pills（Running 取当前 tone，Success/Warning/Error 常亮演示），改为：

```rust
fn header_statuses(snapshot: &HubSnapshot, text: HubTextBundle) -> Vec<HubStatusPill> {
    if snapshot.task_status.running {
        return vec![status("running", &text.status_label("Running"), "running")];
    }
    match snapshot.task_status.severity {
        TaskSeverity::Success => vec![status("success", &text.status_label("Success"), "success")],
        TaskSeverity::Warning => vec![status("warning", &text.status_label("Warning"), "warning")],
        TaskSeverity::Error => vec![status("error", &text.status_label("Error"), "error")],
        TaskSeverity::Info => Vec::new(),
    }
}
```

`TaskSeverity` 四变体见 `state/task_status.rs:18-25`，`TaskStatus::idle()` 为 `Info`（38-49 行附近）→ 干净首帧零 chips，运行中只亮 Running，结束后只亮对应结果 chip，与风险章节"数据驱动为准"的口径一致。前端 `TopBar.tsx:118-120` 的 `state.taskStatus.map(...)` 渲染零改动；`hubData.ts` fallback 的 `taskStatus`（63 行起四条中文 pills）收缩为 `taskStatus: []`。

> 落地状态终核（2026-06-12）：S2 已落地，转盘点验收，但落地形状与上述提案不同（以实仓为准）——`view_model.rs:330-348` 的 `header_statuses` 为：idle（`!running && severity == Info`）early-return 空 vec；running 返回单条 `("running", "running")`；否则经 `severity_tone` 返回单条结果 pill；label 取 `text.status_label(&snapshot.task_status.label)`（沿用任务自身 label），而非提案中固定的 `"Running"/"Success"/"Warning"/"Error"` 字样。前端 `hubData.ts:66` 已收缩为 `taskStatus: []`，`ui_shell_header_contract.rs:190` 断言已同步刷新。补缺项：`header_statuses_*` 三个单测未补。

（S3）侧栏 Engine Status 面板数据驱动 + 折叠对齐。`NavigationDrawer` 现仅收 `engineVersion: string`（props 30-35 行附近），面板内状态点恒绿、`text.upToDate` 恒显（109-152 行附近）；改为：

```tsx
export interface NavigationDrawerProps {
  activePage: string;
  text: HubShellText;
  engineVersion: string;
  sourceEngines: HubSourceEngineSummary[];
  activeSourceEngineId?: string | null;
  onAction: HubActionHandler;
}

// 组件体内（取法对齐 TopBar.tsx:28-31 的同型解析）：
const activeEngine =
  sourceEngines.find((engine) => engine.id === activeSourceEngineId) ??
  sourceEngines.find((engine) => engine.active);
const engineReady = Boolean(activeEngine);

// Engine Status 面板三处改动：
<Box sx={{ width: 8, height: 8, borderRadius: hubTokens.radius.pill, backgroundColor: engineReady ? hubTokens.colors.success : hubTokens.colors.warning }} />
<Typography variant="body2" sx={{ mt: 1.2, color: hubTokens.colors.textSoft }}>
  {activeEngine?.name ?? engineVersion}
</Typography>
<Typography variant="caption" sx={{ color: engineReady ? hubTokens.colors.success : hubTokens.colors.warning }}>
  {engineReady ? text.upToDate : text.noSourceEngineRegistered}
</Typography>
// Check for Updates 按钮保持 disabled（敬请期待，索引 §3.9）。

// 折叠态图标列对齐（ListItemButton/ListItemIcon，现 71-103 行附近）：
<ListItemButton sx={{ ..., justifyContent: collapsed ? "center" : "flex-start", "@media (max-width: 980px)": { justifyContent: "center" } }}>
  <ListItemIcon sx={{ minWidth: collapsed ? 0 : 40, justifyContent: "center", color: "inherit" }}>
```

文案全部为既有 DTO key：`text.upToDate`（Rust 侧 `ui_text.rs:426` `"Local version"/"本地版本"`）、`text.noSourceEngineRegistered`（`HubShellText` 已有字段，`types/hub.ts:285`），零新增文案。`HubWindow.tsx:38` 调用点同变更追加 `sourceEngines={state.sourceEngines} activeSourceEngineId={state.activeSourceEngineId}`。

> 落地状态终核（2026-06-12）：S3 已落地，转盘点验收，但落地形状与上述提案不同（以实仓为准）——`NavigationDrawer.tsx:30-37` props 已扩展（`activeSourceEngineId: string | null`，非可选 `?:`）；组件体内为 `const statusColor = activeEngine ? hubTokens.colors.success : hubTokens.colors.warning` 与 `const statusLabel = activeEngine?.status ?? text.noSourceEngineRegistered`——状态文案直接取引擎 `status` 字段（`HubSourceEngineSummary.status`），**未引入 `text.upToDate`**；折叠对齐为 `justifyContent: collapsed ? "center" : "flex-start"` + `ListItemIcon` `minWidth: collapsed ? 0 : 40`（未追加 980px 媒体查询）。`HubWindow.tsx:55-62` 调用点已传 `sourceEngines`/`activeSourceEngineId`（多行 JSX 形式）。补缺项：`ui_global_rules_contract.rs:328` 仍是旧的单行调用断言（与实仓多行调用失配）、`ui_visual_standard_contract.rs:323` 仍断言 `"backgroundColor: hubTokens.colors.success"`（实仓为 `backgroundColor: statusColor`），两处预计为红，刷新时替换串以实仓形状为准（见 M3 契约联动注记）。

（S4）全页面空态/运行态复核（核对清单，预期多数零代码）：各页空态已统一 `EmptyStateBlock`（`ui_visual_standard_contract` / `ui_workspace_split_contract` 已锁）；运行中任务的"状态横幅 + 进度"在 Builds/Browser 已有（`HubStatusBanner` + `LinearProgress`），唯 Dashboard 缺横幅——在 `ProjectsDashboard.tsx` 标题区下方加一行，与 `ProjectBrowserPage` 同型：

```tsx
<HubStatusBanner task={state.taskSummary} />
```

> 落地状态终核（2026-06-12）：S4 已落地，转盘点验收——`ProjectsDashboard.tsx:128-130` 已在标题区下方渲染 `<HubStatusBanner task={state.taskSummary} />`（外裹 `Box sx={{ mb: 1.4 }}`）；Builds/Browser 的横幅 + `LinearProgress` 既有实现核实在位。补缺项：`ui_project_layout_contract` dashboard 断言尚未追加该行。

（S5）截图矩阵脚本化。复用既有三脚本（`capture-hub-project-pages.ps1`、`capture-hub-visual-state-matrix.ps1`、`compare-hub-tauri-references.ps1`，见 `tauri-react-shell.md` "Visual State Matrix" 节）：矩阵脚本已内置 seeded `hub.toml` 写入（`New-VisualProject` 系列函数）与 `ZIRCON_HUB_VISUAL_TASK_STATE` 任务态注入（常量 `runtime_state.rs:47`、读取 `runtime_state.rs:532`；环境变量实际由 `capture-hub-window.ps1` 设置，矩阵脚本经 `-VisualTaskState` 参数透传），与 03.M2 fixture 剥离后的 seeded config 路线一致。缺口为中文默认态：`capture-hub-visual-state-matrix.ps1` 现 seeds English config（脚本 138 行 `language = "English"`），需加 `-Language` 参数（`english|chinese`，默认 `chinese` 对齐"中文默认"验收）写入 seeded `hub.toml` 的 `language` 键，并把各页 wait-text 表扩成中英候选（实测脚本内 wait-text 现全为英文，如 `Manage engines`/`Launch Target`，**无中文候选先例**，中文候选需对照 `ui_text.rs` 逐页新增）。新增编排脚本 `run-hub-acceptance-matrix.ps1`（同 scripts 目录）按序执行：构建 → 项目页矩阵（`-CapturePendingDelete -CaptureBrowserMenus`）→ 状态矩阵（中文默认 + 英文对照）→ 参考图对比，产出归入 `target/hub-visual-check/**` 并在文档登记。

> 落地状态终核（2026-06-12）：S5 完全未落地——`capture-hub-visual-state-matrix.ps1` 尚无 `-Language` 参数与中文 wait-text，`run-hub-acceptance-matrix.ps1` 不存在，两份文档亦未登记，全部保留为待实施项。

#### M3 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/web/src/components/data/ProjectCardRail.tsx` | 新建 | 卡片 rail 容器：auto-fill 网格 + 右缘渐隐遮罩 + 查看全部箭头 |
| `zircon_hub/web/src/components/data/index.ts` | 修改 | barrel 追加 `export * from "./ProjectCardRail";` |
| `zircon_hub/web/src/pages/ProjectsDashboard.tsx` | 修改 | 网格分支换 ProjectCardRail；加 HubStatusBanner；持有行菜单状态 + HubMenu |
| `zircon_hub/web/src/components/data/ProjectTable.tsx` | 修改 | 追加 `onRowMenu` 可选回调，MoreVert 优先开菜单 |
| `zircon_hub/src/tauri_app/view_model.rs` | 修改 | `HubRecentProject` 加 `pinned`；`header_statuses` 改数据驱动；补单测 |
| `zircon_hub/web/src/types/hub.ts` | 修改 | `HubRecentProject` 加 `pinned: boolean` |
| `zircon_hub/web/src/data/hubData.ts` | 修改 | fallback 行数据补 `pinned: false`；`taskStatus` 收缩为 `[]` |
| `zircon_hub/web/src/components/shell/NavigationDrawer.tsx` | 修改 | Engine Status 数据驱动（props 扩展）+ 折叠图标列对齐 |
| `zircon_hub/web/src/components/shell/HubWindow.tsx` | 修改 | NavigationDrawer 调用点传 sourceEngines/activeSourceEngineId |
| `zircon_hub/tests/ui_project_layout_contract.rs` | 修改 | 刷新 dashboard 断言：ProjectCardRail/HubStatusBanner/行菜单 |
| `zircon_hub/tests/ui_visual_standard_contract.rs` | 修改 | 卡片网格断言改指 ProjectCardRail；drawer 状态点断言改条件式 |
| `zircon_hub/tests/ui_shell_header_contract.rs` | 修改 | 刷新 fallback `taskStatus` 四 pills 断言为 `[]` |
| `zircon_hub/tests/ui_project_browser_table_contract.rs` | 修改 | ProjectTable 增 `onRowMenu` 断言；view_model 段增 `pinned` 断言 |
| `zircon_hub/tests/ui_global_rules_contract.rs` | 修改 | 刷新 HubWindow→NavigationDrawer 调用行断言（328 行附近） |
| `.codex/.../scripts/capture-hub-visual-state-matrix.ps1` | 修改 | 加 `-Language` 参数（默认 chinese）+ 双语 wait-text 候选 |
| `.codex/.../scripts/run-hub-acceptance-matrix.ps1` | 新建 | 一键编排：构建 → 页面矩阵 → 状态矩阵（中/英）→ 参考对比 |
| `docs/zircon_hub/ui/tauri-react-shell.md` | 修改 | Visual State Matrix 节登记编排脚本与中文默认态覆盖 |
| `docs/zircon_hub/ui/responsive-component-system.md` | 修改 | design reference matrix 登记新产出路径 |

> 落地状态终核（2026-06-12）：上表代码行已全部落地——`ProjectCardRail.tsx`（新建）、`data/index.ts`、`ProjectsDashboard.tsx`、`ProjectTable.tsx`、`types/hub.ts`、`NavigationDrawer.tsx`、`HubWindow.tsx`、`hubData.ts`（`taskStatus: []`；fallback 行数据已为空数组，无 `pinned: false` 可补）、`view_model.rs`（`pinned` + `header_statuses`，**单测未补**）。契约五行中仅 `ui_shell_header_contract.rs` 已刷新，其余四行（layout / visual standard / browser table / global rules）未动；两个脚本行与两个文档登记行未落地。

#### M3 实施步骤

1. S2 后端：改 `view_model.rs:321-333` `header_statuses` 为数据驱动形态；在同文件 tests 模块（既有 `TaskStatus::running_operation` 操作先例见 1238/1255 行附近）补三个单测（见契约联动）。验证：`cargo test -p zircon_hub header_statuses --locked`。
2. S2 前端：`hubData.ts` `taskStatus`（63 行起）收缩为 `[]`；刷新 `ui_shell_header_contract.rs:190-194` 断言。验证：`npm run typecheck && npm run build`；`cargo test -p zircon_hub --test ui_shell_header_contract --locked`。
3. S1-b 后端：`HubRecentProject` 加 `pinned` + `recent_project_row` 取 metadata；`ui_project_browser_table_contract.rs` view_model 段（206-219 行附近）追加 `"pub pinned: bool"` 断言；types 段追加 `"pinned: boolean;"`。验证：`cargo test -p zircon_hub --test ui_project_browser_table_contract --locked`、`cargo test -p zircon_hub recent_project --locked`。
4. S1-b 前端：`types/hub.ts` 与 `hubData.ts` 补 `pinned`；`ProjectTable.tsx` 加 `onRowMenu`（MoreVert onClick 分支，63-75 行附近）；`ProjectsDashboard.tsx` 持菜单状态 + `HubMenu` 渲染 + 两处 `<ProjectTable>` 传 `onRowMenu`。验证：`npm run typecheck && npm run build`；`cargo test -p zircon_hub --test ui_project_browser_table_contract --locked`、`cargo test -p zircon_hub --test ui_project_layout_contract --locked`。
5. S1-a：新建 `ProjectCardRail.tsx` + barrel 导出；`ProjectsDashboard.tsx` 网格分支（M1 改造后的 auto-fill 容器）换成 `ProjectCardRail` 组合；刷新 `ui_project_layout_contract` 与 `ui_visual_standard_contract` 卡片网格断言落点（dashboard → ProjectCardRail）。验证：`npm run typecheck && npm run build`；`cargo test -p zircon_hub --test ui_project_layout_contract --locked`、`cargo test -p zircon_hub --test ui_visual_standard_contract --locked`。
6. S4：`ProjectsDashboard.tsx` 标题区下加 `HubStatusBanner task={state.taskSummary}`（import 自 `components/feedback`）；`ui_project_layout_contract` dashboard 断言追加该行。验证：同步骤 5 两条契约命令。
7. S3：`NavigationDrawer.tsx` props 扩展 + 面板/折叠改造；`HubWindow.tsx:38` 调用点同变更更新；刷新 `ui_global_rules_contract.rs:328` 与 `ui_visual_standard_contract.rs:259` 断言。验证：`npm run typecheck && npm run build`；`cargo test -p zircon_hub --test ui_global_rules_contract --locked`、`cargo test -p zircon_hub --test ui_visual_standard_contract --locked`。
8. S5：扩展 `capture-hub-visual-state-matrix.ps1`（`-Language` + 双语 wait-text）；新建 `run-hub-acceptance-matrix.ps1`；跑通一次全矩阵，产出落 `target/hub-visual-check/**`；更新 `tauri-react-shell.md` 与 `responsive-component-system.md` 登记（machine-readable header 保持，`hub_docs_contract` 守卫）。验证：`cargo test -p zircon_hub hub_docs --locked`；逐张对照参考图（测试阶段口径）。

> 落地状态终核（2026-06-12）：步骤 1-7 的代码部分均已落地（落地形状差异见各切片注记），但其中契约刷新/单测子项均未完成：步骤 1 单测、步骤 2（契约已刷新，完成）、步骤 3 的 `pinned` 断言、步骤 4-6 的 browser table / layout / visual standard 断言迁移与追加、步骤 7 的 `ui_global_rules_contract.rs:328` 与 `ui_visual_standard_contract.rs:323`（基线 259 行，已漂移）刷新。步骤 8（S5）完全未落地。剩余实施按「先修失配红契约、再补单测、最后 S5」顺序收口。

#### M3 契约联动

- `ui_shell_header_contract.rs:190-194` 现有断言原文 `"taskStatus: ["`、`"{ id: \"running\", label: \"运行中\", tone: \"running\" }"`、`"{ id: \"success\", label: \"成功\", tone: \"success\" }"`、`"{ id: \"warning\", label: \"警告\", tone: \"warning\" }"`、`"{ id: \"error\", label: \"错误\", tone: \"error\" }"` → 删除五条，改为 `"taskStatus: []"`。
- `ui_visual_standard_contract.rs:323`（基线 259 行，已漂移）drawer 段现有断言原文 `"backgroundColor: hubTokens.colors.success"` → 以实仓落地形状为准改为 `"backgroundColor: statusColor"`，并增 `"const statusColor = activeEngine ? hubTokens.colors.success : hubTokens.colors.warning"` 断言（原提案的 `engineReady ? ...` 内联三元形状未被实仓采用）。
- `ui_global_rules_contract.rs:328` 现有断言原文 `"NavigationDrawer activePage={state.activePage} text={state.ui.shell} engineVersion={state.engineVersion} onAction={onAction}"` → 实仓 `HubWindow.tsx` 调用已改为多行 JSX，单行替换串无法匹配，应拆为多片段断言：`"sourceEngines={state.sourceEngines}"`、`"activeSourceEngineId={state.activeSourceEngineId}"` 等（`ui_visual_standard_contract.rs` HubWindow 段如有同串一并刷新）。
- `ui_project_layout_contract.rs` dashboard 断言（92-115 行附近）：auto-fill 网格串改为 `"ProjectCardRail"` + `"moreLabel={actionText.viewAllProjects}"` + `"hasMore={visibleProjects.length > dashboardProjects.length}"`；追加 `"HubStatusBanner task={state.taskSummary}"`、`"onRowMenu"`、`"HubMenu"`；`"const dashboardProjects = useMemo(() => visibleProjects.slice(0, 4), [visibleProjects]);"` 保持不动。
- `ui_visual_standard_contract.rs:588-598`（基线 526-531，已漂移）ProjectsDashboard 页面断言：auto-fill 网格串移除，改在 `shared_inputs_and_data_components_preserve_reference_density_and_states` 中读取 `ProjectCardRail.tsx` 并断言 `"repeat(auto-fill, minmax(clamp(220px, 22vw, 296px), 1fr))"`、`"pointerEvents: \"none\""`、`"linear-gradient(90deg, rgba(17,18,18,0)"`、`"ChevronRightIcon"`。
- `ui_project_browser_table_contract.rs:53-95`：保持 `"aria-label={`${labels.openDetails}: ${project.name}`}"`、`"event.stopPropagation();"`、`"onOpenDetail?.(project);"` 原文断言（rowMenu 缺省时回退路径不变）；追加 `"onRowMenu?: (project: HubRecentProject, anchor: HTMLElement) => void;"`；view_model 段追加 `"pub pinned: bool"`、types 段追加 `"pinned: boolean;"`。
- 新增 Rust 单测（`view_model.rs` tests 模块）：`header_statuses_show_only_running_pill_while_running`（`task_status = TaskStatus::running_operation(...)` → 仅 1 条 id=="running"）；`header_statuses_show_single_severity_pill_after_completion`（`TaskStatus::error(...)` → 仅 1 条 id=="error"；`TaskStatus::warning(...)` → 仅 warning）；`header_statuses_are_empty_when_idle`（`TaskStatus::idle()` → 空 vec）；`recent_project_rows_carry_pinned_metadata`（metadata 置 `pinned: true` 后行 DTO `pinned == true`）。

> 落地状态终核（2026-06-12）：上列联动中仅 `ui_shell_header_contract.rs` 一条已完成（`:190` 现为 `"taskStatus: []"`，四条旧 pills 断言已删）；其余各条均未落地。其中四处断言已与实仓源码失配、预计当前为红，应优先刷新：`ui_global_rules_contract.rs:328`（旧单行调用断言 vs 多行 JSX 新调用）、`ui_visual_standard_contract.rs:323`（旧恒绿断言 vs `statusColor`）、`ui_project_layout_contract.rs:110` 与 `ui_visual_standard_contract.rs:594`（auto-fill 串已移入 `ProjectCardRail.tsx`，断言仍读 dashboard）。新增 Rust 单测四个均未落地。`ui_project_layout_contract.rs` dashboard 测试现位于 86-143 行，`slice(0, 4)` 断言在 96 行保持。

测试阶段：
- 全量 `cargo test -p zircon_hub --locked` + `npm run build`。
- 视觉验收：截图矩阵逐张与参考图对照，确认无溢出/遮挡/英文硬编码残留；中文默认态文案完整。

## 风险与协调

- 与 05 的分工以 05 计划定稿为准：dashboard 卡片网格 / `ProjectTable` / `NavigationDrawer` 归 06 自管，05 组件拆分不触碰这些落点，不存在「样式修改落在将被拆除的内联块上」的硬顺序约束；顺序协调仅剩 05 其余拆分引起的行号漂移（以标识符定位）。【落地状态终核（2026-06-12）：本计划 M1/M2 与 M3 S1-S4 代码已被并行进程落地，顺序问题已成既成事实。】
- 状态 chips "数据驱动点亮"会改变首帧观感（干净环境只剩 Running 或全无）：与参考图的四 chips 常显有出入——参考图是组件状态总览图，实现以数据驱动为准，在截图矩阵中以错误/运行态用例覆盖各 chip 外观。【落地状态终核（2026-06-12）：数据驱动 `header_statuses` 已落地。】
- 契约对样式源文本断言密集：M1/M2 每个改动点同变更刷新断言，避免批量红。
- 事实修正注记（2026-06-12 细化时核实）：原"现状与证据"称 `HubButton.tsx` 硬编码 `contrastText: "#071515"`，实际该字面量在 `muiTheme.ts:13`（`palette.primary.contrastText`），`HubButton.tsx` 实际硬编码的是 `#eefefe`/`#292929`/`#ffd8d5`；现状清单已按实仓改写，M2 盘点以改写后的清单为准。【落地状态终核（2026-06-12）：该清单所列字面量已全部迁入 token，`muiTheme.ts:13` 现为 `contrastText: hubTokens.colors.textOnPrimary`。】
- 卡片 rail 箭头语义决策：dashboard 卡片数被 `ui_project_layout_contract` 的 `slice(0, 4)` 断言与 `view_model.rs` `PROJECT_CARD_LIMIT`（=12）共同约束，M3 将右缘箭头定为"查看全部"导流（`HUB_ACTION.viewAllProjects`）而非真实横向滚动；若后续验收要求真滚动 rail，需同步解除 `slice(0, 4)` 契约并改为 `gridAutoFlow: "column"` 滚动容器，届时单独立项。【落地状态终核（2026-06-12）：已按"查看全部"导流语义落地（`ProjectCardRail` `onMore` → `HUB_ACTION.viewAllProjects`），`slice(0, 4)` 契约未动。】
- M2 守卫口径：`rgba(...)` 字面量（components+pages 共 83 处）多为 hover/选中蒙层且被契约源文本逐串锁定，本计划不迁移、守卫不拦截；token 一元化以裸 hex 为收口口径。【落地状态终核（2026-06-12）：已落地的守卫除裸 hex 外还额外拦截 `borderRadius: 999`；`rgba(...)` 实测仍 83 处，维持不迁移。】
- 前端命令执行目录：`package.json` 位于 `zircon_hub/`（`vite.config.ts` 同级），`npm run typecheck`/`npm run build`/`npm run tauri:dev` 均在 `zircon_hub/` 下执行，而非 `zircon_hub/web/`。
- S5 中文 wait-text 扩表风险：`capture-hub-visual-state-matrix.ps1` 的逐页等待文本现为英文（Launch Target/Build Workflow 等），中文默认态运行需逐页补中文候选，漏补会导致截图脚本误判"React 仍在 fallback 首帧"而失败；扩表时对照 `ui_text.rs` 的中文文案逐项核对。
