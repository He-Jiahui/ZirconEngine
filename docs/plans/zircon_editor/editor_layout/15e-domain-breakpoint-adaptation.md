---
related_code:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_editor/src/ui/workbench/autolayout/layout_tier.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/region_frames.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/side_width_allocation.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/compute.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/window_minimums.rs
  - zircon_editor/src/ui/workbench/autolayout/workbench_shell_geometry.rs
  - zircon_editor/src/ui/workbench/autolayout/region/tool_region/collapsed_constraints.rs
  - zircon_editor/src/ui/workbench/autolayout/region/tool_region/build.rs
  - zircon_editor/src/ui/workbench/autolayout/region_binding/workbench_constraint_token_name.rs
  - zircon_editor/src/ui/workbench/autolayout/region_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/drawer_layout.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/dock_header/side.rs
  - zircon_editor/src/tests/workbench/layout/editor_layout_contracts.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_drawer_breakpoints.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/STabDrawer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/STabDrawer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/STabSidebar.cpp
  - docs/ui-and-layout/editor-workbench-designs/drawer-collapsed-state-spec.png
  - docs/ui-and-layout/editor-workbench-designs/compact-editor-state-spec.png
  - docs/ui-and-layout/editor-workbench-designs/split-editor-state-spec.png
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15a-page-tab-strip-overflow.md
  - docs/plans/zircon_editor/editor_layout/15d-composite-density-and-alignment.md
  - docs/plans/zircon_editor/editor_layout/13-taffy-css-constraint-language.md
  - docs/plans/zircon_editor/editor_layout/16-relative-layout-and-resolution-adaptation.md
  - docs/plans/engine-code-structure-convention.md
status: in_progress
---
# 15e 领域层断点自适应(S15.5 深化)

> 本文是 `15` 计划 **S15.5** 切片的实现就绪深化:把抽屉/视口/底部/页签条在不同窗口宽度下的自适应,从"零散的 compact 阈值钳制"升级为**显式断点 tier + token 化阈值 + 抽屉折叠到 rail**,并把 `15a`(页签溢出)、`15d`(列丢弃)、本文(抽屉折叠)拼成一条统一响应式故事。是原子→复合→**领域**分层的收尾。父计划见 `15`。

> **断点是三层自适应模型的第③层(见 `16`)**:断点 tier 处理"窄到一定程度要改变结构"(折叠/降级),它**在**第①层根 DPI 缩放、第②层 flex 相对布局**之上**。硬约束:断点判定用**逻辑宽度** `logical_width = physical_width / scale_factor`,不是物理像素——否则同一物理布局在高 DPI 屏会误判 tier。当前阈值虽已 token 化进 `EditorDensityTokens`,但仍是**固定物理像素**(见下 §1.1 缺陷 4),需按 `16` 改读逻辑宽度。

## 0. 与 `16` 的关系(自适应分层定位)

| 层 | 规范 | 本文角色 |
| --- | --- | --- |
| ① 根 DPI 缩放 | `16` §3.4 | 消费:tier 判定前先 `physical / scale_factor` 取逻辑宽度 |
| ② flex 相对布局 | `13` + `16` §3.5/3.6 | 消费:抽屉/区域宽用 grow/basis%/min-max,折叠改的是"结构"不是"像素偏移" |
| ③ 断点 tier 降级 | **本文** | 拥有:`LayoutTier` 分档 + 折叠/丢列/页签溢出协同 |

本文不重做①②,只在其上做结构级降级。`region_frames.rs` 的竖向像素累加(`16` R2)由 `16` 修正,本文的 tier 决策改读逻辑宽度即可复用既有折叠/降级实现。

## 1. 现状(按代码核实)

壳几何已有**梯度 compact 钳制**,但非显式断点、阈值裸值、且**无折叠**:

| 现状 | 落点 | 值 |
| --- | --- | --- |
| 侧栏 compact 阈值/上限/最小 | `geometry/region_frames.rs` | `COMPACT_SIDE_AVAILABLE_WIDTH=1100`、`COMPACT_LEFT_MAX=340`/`RIGHT_MAX=220`、`COMPACT_SIDE_MIN_WIDTH=196` |
| 侧栏 ultra-compact 阈值/上限 | 同上 | `ULTRA_COMPACT_SIDE_AVAILABLE_WIDTH=760`、`LEFT_MAX=220`/`RIGHT_MAX=160` |
| 底部 compact/ultra 高度 | 同上 | `COMPACT_BOTTOM_*`(avail 900→max 148/min 120)、`ULTRA_*`(avail 420→max 96/min 80) |
| 窗口最小宽 = Σ区域约束 min + 分隔 | `geometry/window_minimums.rs::compute_window_min_width` | 防止窗口窄到内容放不下 |

### 1.1 缺陷
1. **阈值非 token、非显式断点**:`1100/760/900/420` 等散落在 `region_frames.rs`,与 `15` 验收的 **640/900/1260** 三档不对齐,也不在约束 token 体系(违 `13` 的"阈值 token 化")。
2. **只钳不折叠**:侧栏窄到 `COMPACT_SIDE_MIN_WIDTH=196` 后**仍占位**,没有"折叠成图标 rail / 折成单抽屉"的 tier(参考 `drawer-collapsed-state-spec.png`)。于是 640px 窄窗下"左196+文档+右160+分隔"挤爆,内容溢出。
3. **三套自适应各自为政**:页签溢出(`15a`)、列丢弃(`15d`)、抽屉钳制(本文)没有统一的断点驱动——同一窗口变窄,三者应**协同降级**。
4. **阈值是物理像素,非逻辑宽度(接 `16` R3)**:即便阈值已 token 化进 `EditorDensityTokens`(`breakpoint_*_width`),它仍是**固定物理像素**,且 `layout_tier` 直接拿物理宽度分档。高 DPI(scale=2.0)下 3840px 物理 = 1920px 逻辑,本应是 Wide,却因物理值翻倍而误判——断点必须用 `logical_width = physical / scale_factor`。

## 2. 设计:显式断点 tier + token 阈值 + 折叠

### 2.1 规范断点(与 `15` 截图三档对齐,判定用逻辑宽度)
> 下表 `bp.*` 阈值均为**逻辑像素**;分档输入是 `logical_width = physical_width / scale_factor`(接 `16` §3.4),保证 1920@1.0 与 3840@2.0 落同一档。

| Tier | 逻辑宽度区间 | 抽屉 | 页签(`15a`) | 表格列(`15d`) |
| --- | --- | --- | --- | --- |
| **Wide** | ≥ `bp.wide`(1260) | 左右抽屉加宽(上限放宽);可加列 | 全称 | 全列 |
| **Regular** | `bp.narrow`..`bp.wide`(640–1260) | 左右抽屉常驻,宽度钳到 compact 上限/最小 | 够宽全称,否则 `…`/溢出 | 全列,窄列省略 |
| **Narrow** | ≤ `bp.narrow`(640) | **折叠**:右抽屉→图标 rail 或隐藏;仅保活动抽屉 | 溢出 `⋯` 收纳 | 丢弃低优先列 |
| **(可选)Ultra** | < `bp.ultra`(≈480) | 左右皆折叠为 rail,文档占满 | 仅 `⋯` | 仅 Name |

> 现有 `1100/760` 阈值并入 Regular 内的"compact/ultra-compact 钳制档"(保留行为),`640/1260` 作为**折叠/加宽**的新边界。三档 = 截图验收档,内部仍可细分 compact 子档。

补:高度维度决策(2026-07-02 评审收口):`bp.short` 式**高度 tier** 登记为**已知非目标**——当前矮窗行为靠 `COMPACT/ULTRA_BOTTOM_*` compact 钳制兜底(§1 现状表),不进本文断点 tier 体系;若后续矮窗(如 640x420 以下)出现结构级降级需求,另行立项,不在 S15.5 范围内追加。

### 2.2 token 化阈值(逻辑单位,遵 `13`/`16`)
断点阈值与抽屉宽/底高改为**约束 token**(接 `region_binding/workbench_constraint_token_name.rs` 体系),且 token 值是 **DPI 无关逻辑单位**(`16` §3.2):`--breakpoint-narrow`/`--breakpoint-wide`/`--breakpoint-ultra`、`--left-drawer-width`/`--right-drawer-width`/`--bottom-output-height`、`--drawer-rail-width`(折叠后 rail 宽)。`region_frames.rs` 的裸 `1100/760/340/220/196…` 收敛为 token 默认值(单源,可被布局预设 `04` 覆盖)。tier 判定取逻辑宽度后再与这些逻辑阈值比较。

### 2.3 抽屉折叠到 rail(新 tier)
- Narrow tier:把指定侧抽屉的 `RegionState` 强制走已存在的 collapsed rail 约束,几何产出 `--drawer-rail-width`(仅图标条/内容壳隐藏,点击图标弹出 overlay 抽屉的交互归 03/07),复用 `region/tool_region/collapsed_constraints.rs` 的折叠约束。
- 折叠优先级:先折 Right(属性/细节),再折 Left(放置/文件);活动抽屉(用户正在用的)最后折。
- 折叠焦点转移(2026-07-02 评审收口):若折叠发生时键盘焦点位于被折叠抽屉的子树内,焦点**还原到该抽屉对应的 rail 图标**(可聚焦),不静默丢焦;还原语义走 19 的焦点作用域还原(稳定逻辑标识 + 回退作用域首个可聚焦),该用例已回挂 19 测试矩阵。
- 与 `compute_window_min_width` 协同:折叠后窗口 min 宽显著下降,640px 才真正可用。

实现补记(2026-07-10):Regular tier 不能只让左右抽屉分别 compact,还必须给中央文档区保留共同预算。`--minimum-document-width-fraction` 现由中央 density token 定义为 0.5,raw geometry 与 componentized drawer bridge 同时消费 `side_width_allocation.rs` 的 larger-side-first 分配；侧栏页签投影同时采用“活动标签全文优先、非活动标签图标化、超额标签折叠”的相对槽位策略。该补记只关闭 Regular 共同预算与页签自适应,不扩张到 overlay drawer 交互。

### 2.4 统一响应式协同
单一"断点 tier"决策点(落 `geometry` 下新 owner `breakpoint.rs`)输出 `LayoutTier`,被三处消费:
- 抽屉几何(本文:钳/折叠)
- 页签条(`15a`:tier=Narrow 时强制溢出阈值更激进)
- 表格列(`15d`:tier=Narrow 时丢列优先级更激进)

## 3. 结构/债务纪律
- `LayoutTier` 决策与阈值落 `geometry/breakpoint.rs` owner 叶子 + 单测;`region_frames.rs` 改读 token/tier,删裸阈值。
- 阈值 token 接既有 `workbench_constraint_token_name` 体系,不新造平行体系。
- 无 `unwrap/expect/TODO/allow(dead_code)/裸 Result`;文件 ≤800;折叠走既有 `collapsed_constraints` 扩展,不另起 docking 实现(那是 `03/07`)。
- 折叠/丢列若隐藏内容,`log` 标注(遵 `15` 附录 C"无静默截断")。

## 4. 测试矩阵
| 测试 | 断言 |
| --- | --- |
| `tier_classifies_by_width`(640/900/1260) | 返回 Narrow/Regular/Wide |
| `narrow_collapses_right_drawer_first` | ≤640 → Right rail 宽,Left 仍在 |
| `wide_relaxes_drawer_max_width` | ≥1260 → 抽屉宽上限放宽 |
| `window_min_width_drops_after_collapse` | 折叠后 `compute_window_min_width` 显著下降,≤640 |
| `breakpoint_thresholds_are_token_sourced` | 阈值来自 token 默认,非裸字面 |
| `tier_drives_tab_and_column_degradation`(集成) | Narrow tier 同时触发页签溢出(`15a`)与丢列(`15d`) |

## 5. 验收
- 三档截图刷新 `docs/tests/editor/`:
  - `editor-window-m3-svg-icon-scale-large-1260x780`:抽屉加宽、留白合理不空旷。
  - `editor-window-m3-workbench-900x620`:左右抽屉常驻、四区清晰。
  - `editor-window-m3-svg-icon-scale-small-640x420`:右抽屉折叠为 rail、页签 `⋯`、表格丢列,**不破版**。
- 命令:`cargo test -p zircon_editor --lib`(`workbench_shell_geometry`/region extents/breakpoint)+ `capture_m3_gui_acceptance_visual_artifacts --ignored`。

## 6. 与既有计划关系
- `16`:本文是三层自适应模型第③层(断点降级);DPI 根缩放、flex 相对布局、`region_frames` 竖向像素累加修正归 `16`。本文 tier 判定改读逻辑宽度即对齐。
- `13`:阈值/抽屉宽 token 化由本文落到约束体系(token 为逻辑单位)。
- `04`:布局预设可覆盖断点 token/抽屉宽(持久化),本文只给默认 + tier 决策。
- `03/07`:折叠的"承载/docking 运行时"属 03/07;本文只做几何 tier + rail 折叠的自适应几何,不重做停靠语义。
- `15a`/`15d`:本文的 `LayoutTier` 作为它们激进降级的统一触发。

## 7. 实现顺序
1. `geometry/breakpoint.rs`:`LayoutTier` + token 阈值 + 分类纯函数 + 单测(RED→GREEN)。
2. `region_frames.rs` 改读 tier/token,裸阈值收敛为 token 默认。
3. 扩 `collapsed_constraints` 支持 rail 折叠;`compute` 在 Narrow tier 标记折叠 + 产 rail 几何。
4. 把 tier 注入页签(`15a`)/列(`15d`)降级阈值(集成)。
5. 三档截图 + region extents 测试;写状态。

## 8. 边界
不重做 docking 运行时/吸附(`03/07`);不做用户拖拽列宽/抽屉宽持久化(`04`);折叠后的 overlay 抽屉交互细节归承载层,本文只产几何与 tier。

## 9. 状态与产出记录
| 日期 | 切片 | 状态 | 产出/证据 | 后续项 |
| --- | --- | --- | --- | --- |
| 2026-06-25 | 15e/S15.5 深化立项 | planned | 代码核实壳几何已有梯度 compact 钳制(`COMPACT_SIDE_AVAILABLE_WIDTH=1100`/`ULTRA=760`、`COMPACT_SIDE_MIN_WIDTH=196`、`COMPACT/ULTRA_BOTTOM_*`)但**阈值裸值、与 640/900/1260 不对齐、只钳不折叠、三套自适应各自为政**;给出规范断点 tier(Wide≥1260/Regular/Narrow≤640/可选 Ultra)、阈值与抽屉宽 token 化(接 `workbench_constraint_token_name`)、抽屉折叠到 rail(复用 `collapsed_constraints`)、`LayoutTier` 统一驱动页签(`15a`)/列(`15d`)/抽屉协同降级、6 条测试矩阵、实现顺序。 | 按 §7 实现 S15.5;父计划 `15` 的 S15.5/D5 勾稽随之更新;至此 `15` 的原子→复合→领域三层深化文档(`15b`/省略+`15c`/`15d`/`15a`/`15e`)成套。 |
| 2026-06-25 | 15e/S15.5a 断点 tier + 窄屏右抽屉 rail 折叠 | implemented-focused-passed-screenshot-passed | 新增 `autolayout/layout_tier.rs` owner,把 640/900/1260 分类为 Narrow/Regular/Wide;`compute_workbench_shell_geometry` 在 Narrow 时让 Right 工具区域复用现有 collapsed rail 约束,`drawer_layout.rs` 同步把右 drawer shell/content 宽度降为 0,避免绘制层仍按完整右栏输出。桥接层测试迁入 `workbench_drawer_breakpoints.rs` 小 owner,不再扩大 3000+ 行 `workbench_projection.rs`。验证:`cargo test -p zircon_editor --lib narrow_workbench_geometry_collapses_right_drawer_to_rail --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short` 1/1;`workbench_layout_tiers_classify_reference_capture_widths` 1/1;`componentized_workbench_layout_collapses_right_drawer_shell_at_narrow_width` 1/1;`capture_m3_gui_acceptance_visual_artifacts --ignored` 1/1;`cargo fmt -p zircon_editor --check`;`cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short` 通过(仅既有 warning)。截图刷新到 `docs/tests/editor/`,640 图中右侧 Inspector 完整内容已折叠,1260 图中右侧仍保留。 | 本片只关闭右侧抽屉首段折叠;主页签 `15a` overflow popup/选择列表已具备基础,但阈值 token 化、`window_min_width` 深度下降、tier 驱动页签溢出策略、表格列 `15d` 窄档联动与 Ultra 档仍 open。 |
| 2026-06-25 | 15e/S15.5b 断点 tier 驱动主页签 overflow 策略 | implemented-focused-passed-screenshot-passed-build-passed | 继续复用 `autolayout/layout_tier.rs`: `page_tabs/metrics.rs` 将 Workbench 宽度映射为主页面页签 visible cap,Narrow tier 只保留 2 个可读 tab 并强制 overflow,Regular/Wide 在空间足够时保留全部页签;fallback chrome projection 和 retained host pointer geometry 同时消费该策略,并在页签 lane 里先预留 project path 宽度。验证:`fallback_page_chrome_narrow_tier_caps_visible_tabs_before_project_path` 1/1、`fallback_page_chrome_wide_tier_does_not_force_overflow_when_tabs_fit` 1/1、`narrow_tier_caps_visible_tabs_before_overflow` 1/1、`cargo check -p zircon_editor --lib` 通过;`capture_m3_gui_acceptance_visual_artifacts --ignored` 1/1 并刷新 `docs/tests/editor/editor-window-m3-svg-icon-scale-small-640x420.png`、`editor-window-m3-workbench-900x620.png`、`editor-window-m3-svg-icon-scale-large-1260x780.png`;最终 `cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never` 通过。 | 页签已接入领域 tier;仍需把表格列 `15d` 窄档丢列策略、breakpoint 默认 token 化、窗口 minimum 降低和 Ultra 档纳入后续切片。 |
| 2026-06-25 | 15e/S15.5c 断点 tier 驱动表格列降级 | implemented-focused-passed-screenshot-passed-build-passed | 新增 `retained_host/ui/template_layout_context.rs` 将 Workbench root 或 Asset Browser pane 的上下文宽度映射为 layout tier variant,再由表格列分配 owner 消费 `layoutNarrow` 触发 Size/Rev 低优先级列隐藏;这使窄屏抽屉/窗口下的页签、右抽屉和表格列共同受同一 tier 语义驱动。验证:`cargo fmt -p zircon_editor --check`;`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never` 通过;`table_columns_drop_numeric_cells_for_narrow_layout_context`、`asset_browser_table_nodes_receive_narrow_context_variant`、`table_nodes_receive_context_tier_variant` 均 1/1;`cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never` 通过;M3 截图 harness 1/1 刷新 `docs/tests/editor`。 | 抽屉折叠、主页签 overflow、表格列降级三条已接入 tier;剩余为 breakpoint 默认 token 化、窗口 minimum 进一步降低、popup anchor token 化和 Ultra 档。 |
| 2026-06-26 | 15e/S15.5d 断点默认 token 化 + window minimum + Ultra tier | implemented-focused-passed-check-passed-build-passed-screenshot-passed | `EditorDensityTokens::workbench_dense()` 与 `editor_tokens.zui` 现在拥有 breakpoint ultra/narrow/wide、compact side/bottom、minimum-window 与 Ultra-window limit 默认值;`autolayout/layout_tier.rs` 投影这些默认值并新增 Ultra tier,`region_frames.rs` 读取 owner 提供的 compact 默认值,`window_minimums.rs` 按当前窗口尺寸应用 token-backed 下限,允许 640x420/900x620 截图窗口。`page_tabs/metrics.rs` 与 `template_layout_context.rs` 同步把 Ultra 视为 narrow 降级语义。验证:runtime-interface `editor_design_tokens` 5/5;focused editor tests `workbench_breakpoint_defaults_are_sourced_from_design_tokens`、`compact_region_limits_follow_breakpoint_density_defaults`、`workbench_window_minimums_allow_reference_capture_sizes`、`workbench_layout_tiers_classify_reference_capture_widths` 均通过;`cargo fmt -p zircon_editor -p zircon_runtime_interface --check`;`cargo check -p zircon_editor --lib --tests`;`cargo build -p zircon_editor`;M3 screenshot harness 1/1 刷新 `docs/tests/editor/editor-window-m3-workbench-900x620.png`,输出均使用 `D:\cargo-targets\zircon-editor-components-0625` 且未写入 repo `target`。 | breakpoint 默认 token 化、窗口 minimum 降低和 Ultra tier 已关闭;S15.5 剩余为 popup anchor 余量 token 化,以及下一轮 page-tab/window chrome 组合观感复核。 |
| 2026-06-26 | 15e 文档对齐 `16`(逻辑宽度/DPI 自适应) | planned(文档侧,未改代码) | 配合新增 `16`(相对布局与多分辨率自适应规范)对齐本文叙述:新增 §0"与 16 的关系"分层定位表;§1.1 增缺陷 4(阈值是物理像素、`layout_tier` 直接拿物理宽度分档,高 DPI 误判 tier,需 `logical_width = physical / scale_factor`);§2.1/§2.2 明确断点阈值为**逻辑像素**、token 为逻辑单位;§6 增与 `16` 关系行。已落 token 化 tier 的实现(S15.5a–d)保留不变,后续按 `16` §6 把 tier 判定输入从物理宽度切换为逻辑宽度。 | 实现侧:`layout_tier` 分档输入改 `logical_width`(`16` S5);多 scale 一致性测试 `tier_uses_logical_width_consistent_across_scale`。 |
| 2026-07-05 | 15e/S15.5h 逻辑宽度断点与宿主 scale_factor 接线 | implemented-focused-passed-screenshot-passed | `autolayout/layout_tier.rs` 删除模糊 `*_for_width` 入口,拆成 `workbench_layout_tier_for_logical_width` 与 `*_for_physical_width(width, scale_factor)`;右抽屉折叠与窗口最小宽 physical helper 先按 `physical / scale_factor` 得到逻辑宽度。`compute_workbench_shell_geometry` 与 componentized `drawer_layout.rs` 接收 scale factor;`RetainedEditorHost` 启动/同步 `shell_scale_factor`,并把 `UiHostWindow.window().scale_factor()` 传入壳几何和 Workbench bridge。`HostContractState` 新增归一化 `window_scale_factor` 默认 1.0,`HostWindowHandle` 暴露 getter/setter,真实 winit event loop 在 `sync_host_window_state` 中写入 `Window::scale_factor()`。`page_tabs/metrics.rs` 与 `template_layout_context.rs` 保持逻辑宽度消费。新增 `window_scale_factor_defaults_to_one_and_filters_invalid_values`、`tier_uses_logical_width_consistent_across_scale`、`scaled_workbench_geometry_uses_logical_width_for_right_drawer_collapse`、`componentized_workbench_layout_collapses_right_drawer_shell_by_logical_width_under_scale`。验证:`cargo fmt -p zircon_editor` 通过;旧模糊 API 静态扫描 0 命中;scoped `git diff --check` 仅 CRLF 提示;focused `cargo test -p zircon_editor --lib window_scale_factor_defaults_to_one_and_filters_invalid_values ...` 1/1 通过;focused `cargo test -p zircon_editor --lib logical_width ...` 3/3 通过;M3 screenshot harness `capture_m3_gui_acceptance_visual_artifacts` 1/1 通过并刷新 `docs/tests/editor` 的 640/900/1260 PNG,target 同名截图扫描无匹配。 | 继续推进 popup anchor/窗口组合观感。 |
