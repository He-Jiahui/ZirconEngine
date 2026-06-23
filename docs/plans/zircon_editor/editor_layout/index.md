---
related_code:
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime_interface/src/ui/layout/mod.rs
  - zircon_runtime/src/ui/layout/mod.rs
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/workbench_shell_geometry.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/assets/ui/editor/components
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/batch.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime_interface/src/ui/component/data_binding/mod.rs
  - zircon_runtime_interface/src/ui/component/descriptor/slot_schema.rs
  - zircon_runtime_interface/src/ui/widget.rs
  - zircon_runtime_interface/src/ui/layout/style.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/compute.rs
design_references:
  - docs/ui-and-layout/editor-workbench-designs/STYLE-NOTES.md
  - docs/ui-and-layout/editor-workbench-designs/main-tabs-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/tool-drawers-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/scene-drawer-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/inspector-drawer-content-spec.png
  - docs/ui-and-layout/editor-workbench-designs/material-drawer-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/drawer-expanded-state-spec.png
  - docs/ui-and-layout/editor-workbench-designs/split-editor-state-spec.png
  - docs/ui-and-layout/editor-workbench-designs/command-palette-window-spec.png
  - docs/ui-and-layout/editor-workbench-designs/preferences-window-workbench.png
  - docs/ui-and-layout/ai-workbench-style/ai-workbench-web-framework.png
  - docs/ui-and-layout/ai-workbench-style/STYLE-NOTES.md
  - docs/ui-and-layout/ai-workbench-style/prototype/README.md
  - docs/ui-and-layout/ai-workbench-style/component-prototype/web-native-handoff-matrix.md
plan_sources:
  - docs/plans/zircon_editor/editor_ui/index.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/09-editor-modules-and-design-parity.md
  - docs/plans/engine-code-structure-convention.md
status: in_progress
---
# Zircon Editor 布局设计计划总览

本目录是「编辑器 UI 布局 / 设计语言 / 停靠架构」的权威计划集。它与同级 `editor_ui/` **平行而不重复**：`editor_ui/` 计划运行时 UI 能力的构建（布局引擎、输入派发、样式选择器、组件目录、壳级承载等),本目录计划**编辑器外观如何被组织、被声明、被复用**——即把 `editor-workbench-designs` 参考图里的画面结构,沉淀为一套用户友好的设计接口、一致的设计语言、类 JetBrains 的停靠工作台架构。

设计目标五条,逐条对应本目录子计划:

1. **用户友好的设计接口**：编辑器作者(以及后续插件作者)用一套**声明式、可组合、低门槛**的布局描述接口拼装界面,而不是手写绝对坐标或散落像素调参。接口面向 `.zui` 组件资产 + 区域语义 + 布局预设,而非运行时内部结构。
2. **一致的设计风格**：所有面板、控件、停靠区共享同一套**设计 token(色板、密度、控件规格、间距、边框)**与状态优先级,保证 13 个编辑器页面与浮动窗口在视觉上是一套语言而不是拼贴。
3. **类 JetBrains 的设计架构**：主文档标签 + 活动栏 + 固定停靠抽屉区(5-6 槽) + 底部输出 + 状态栏 + 浮动窗口的工作台骨架,带布局预设与持久化。停靠语义、区域职责、抽屉行为与 JetBrains/Unreal 编辑器对齐。
4. **组件化、泛用化的拼装基元**：界面由**可复用、泛用化的 widget 组件 + slot 插槽**拼装,布局由**类 CSS 的 Taffy 约束语言 + token**声明,数据由**单向受控的绑定**驱动——借鉴 Unreal Slate 的 `SWidget+FSlot`、React 的组件化/单向数据流、Taffy 的类 CSS 算法,但全部落到既有 seam(详见 12/13/11/14)。
5. **确定性渲染与增量刷新**：声明出来的界面经**确定性渲染管线(resolve→extract→command→batch→上屏)**产出像素,且任何变更只**增量重算/重提取脏部分**,复杂编辑器不卡(详见 10/09)。

## 1. 设计权威与参考映射

布局与设计语言的视觉权威是 `editor-workbench-designs/` 下的设计图集(约 250 张 PNG,见该目录 `STYLE-NOTES.md` 的完整清单与分类标签),以及 `ai-workbench-style/ai-workbench-web-framework.png` 的壳级色彩/结构权威。本目录**只取设计意图与结构语义**,最终 chrome 由 `.zui` 资产 + 运行时确定性渲染产出,不内嵌 PNG 截图,不照搬 web 原型代码。

| 参考物 | 取什么 | 不取什么 |
| --- | --- | --- |
| `main-tabs-layout-spec.png` / `tool-drawers-layout-spec.png` | 主文档标签条 + 活动栏 + 抽屉槽位的骨架结构 | 不取像素坐标;改用 Taffy 约束 + 区域语义 |
| `scene-drawer-layout-spec.png` / `inspector-drawer-content-spec.png` | 抽屉区职责分配(放置/层级/属性/控制台) | 不取单页内联样式;改用统一组件族 |
| `material-drawer-layout-spec.png` / 各 `*-editor-page` 设计图 | 每个编辑器页面在同一壳内的区域填充模式 | 不为每页造一套壳 |
| `drawer-expanded-state-spec.png` / `split-editor-state-spec.png` / 各 `*-state-spec` | 抽屉展开/折叠、分屏、激活态等状态语义 | 不取一次性状态贴图 |
| `command-palette-window-spec.png` / `preferences-window-workbench.png` / 各 `floating-window` | 浮动窗口(命令面板/偏好)在工作台之上的浮层规则 | 不取模态实现细节 |
| `STYLE-NOTES.md` 色板与控件规则 | 近黑面板色阶 `#111416`/`#171a1d`/`#1b1f23`/`#252b31`、teal `#3cc7d6` 仅用于激活/选中/焦点;圆角矩形、1px 边框、扁平态、28-32px 控件高、低圆角;**禁止**渐变/辉光/阴影/嵌套卡片/英雄字号 | 不取参考图里的任何具体定位数值 |
| `ai-workbench-style/prototype/README.md` Rust 迁移映射 | top-bar→壳命令条、page-tabs→主文档标签、tool-rail→垂直命令栏、drawer/dock/panel→停靠槽、center-stage→活动视口、bottom-output→控制台/时间轴;`--left-drawer-width`/`--right-drawer-width`/`--bottom-output-height` 作为 Taffy 尺寸 token;布局预设(Authoring/Review/Focus/Debug)→保留布局档案 | 不取 web 实现 |
| `component-prototype/web-native-handoff-matrix.md` 现有 `.zui` 资产清单 | 已存在的壳/组件 `.zui` 作为布局可组合的物料 | 不重复造已有资产 |

### 1.1 抽屉区职责(来自 STYLE-NOTES 的 Drawer Zone Roles)

类 JetBrains 工作台把停靠抽屉分为固定语义槽,布局计划据此分配,不让任意面板落在任意槽:

| 槽位 | 职责 | 典型面板 |
| --- | --- | --- |
| 左上 | 放置/预制体工具 | 放置面板、Prefab 工具栏 |
| 左下 | 文件/工程树 | 工程树、文件系统树 |
| 右上 | 层级/结构 | 场景树、节点结构、图谱大纲 |
| 右下 | 属性/动画/细节 | Inspector、属性、动画细节、资源行 |
| 底部 | 控制台/诊断/时间轴 | 输出、校验、构建、队列、时间轴 |

## 2. 现状评审结论(按代码核实)

布局/设计语言所需的运行层设施大多已由 `editor_ui/` 计划落地,本目录在其上做**声明层与设计语言层**的工作,不重做运行时:

- **布局引擎**:Taffy 已是后端;壳级 autolayout 模块(`workbench_shell_geometry`/`region`/`constraints`)已存在;尺寸 token 概念已在 web 原型映射中确立。
- **样式选择器**:`UiPainterStyleSelector` 已按 family 折叠,状态优先级单源已成立。本目录补**中央设计 token 文档化 + token 到选择器的喂入路径**。
- **壳与区域**:Rust-owned retained host;壳区域 `.zui` 已有 8 件(top_toolbar/main_band/activity_rail/status_bar/component_drawer/scene_tree_panel/inspector_panel/viewport_panel);view registry / window registry / preset 模块齐备;11 个 core module workspace 存在。
- **组件物料**:material_foundation catalog + workbench primitive `.zui` 资产齐备,可直接作为布局可组合的物料。

骨架缺口与归属(本计划主战场):

| # | 缺口 | 归属子计划 |
| -- | --- | --- |
| 1 | 缺中央设计 token 资产与文档化的设计语言契约 | 01 |
| 2 | 缺面向编辑器作者的声明式布局描述接口(区域语义 + 槽位 + 约束 token) | 02 |
| 3 | 缺类 JetBrains 停靠工作台架构的完整区域职责与抽屉行为规范 | 03 |
| 4 | 缺布局预设与持久化(布局档案、按页面/按用户) | 04 |
| 5 | 缺每个编辑器页面在同一壳内的布局模板与状态规范 | 05 |
| 6 | 缺浮动窗口(命令面板/偏好/独立编辑器)浮层规则与设计对齐验收 | 06 |
| 7 | 缺 Chrome 式页签合并 + 抽屉可独立成窗口/转移吸附的承载语义 | 07 |
| 8 | 缺插件页面接口与编辑器消息交互入口 | 08 |
| 9 | 缺增量(非全量)消息总线与刷新协议,避免编辑器复杂时全局卡顿 | 09 |
| 10 | 缺真实渲染管线契约(style+layout→draw command→上屏)与禁用视觉/token 单源/脏区增量提取 | 10 |
| 11 | 缺数据绑定规范(`$` 表达式分流、单向受控、view-model 派生、绑定级脏依赖) | 11 |
| 12 | 缺 widget/slot 组件化与泛用化规范(组件三层、slot 契约、prop 契约、组件目录) | 12 |
| 13 | 缺类 CSS 的 Taffy 约束语言规范(flex/grid/block 词汇映射、约束 token 化、family 决策) | 13 |
| 14 | 缺统领性思想综述(Unreal widget+slot / React 单向受控 / 失效增量如何映射既有 seam) | 14 |

## 3. 分层与依赖

```
统领思想层 (14 Unreal widget+slot / React 单向受控 / 失效增量 —— 心智模型,统领 10-13)
      ↓ 指导
设计语言层 (01 设计 token + 风格契约)
      ↓ 喂入
拼装基元层 (12 widget/slot 组件化 + 13 类 CSS Taffy 约束语言 + 11 数据绑定)
      ↓ 组合
布局接口层 (02 声明式布局描述 / 区域语义 / 约束 token)
      ↓ 组合
停靠架构层 (03 工作台骨架 / 抽屉区职责 / 停靠语义)
      ↓ 落到
页面与浮窗层 (05 编辑器页面布局模板 + 06 浮动窗口)
      ↕ 横切
布局档案层 (04 预设 / 持久化)
      ↓ 上屏
渲染层 (10 resolve→extract→command→batch→上屏 + 脏区增量提取)

承载与通信(横切全部布局,07/08/10/11 之基座):
增量消息总线 (09) ← 承载 ← 窗口化/Chrome 页签/可吸附抽屉 (07)
                            ← 承载 ← 插件页面接口与消息交互 (08)
                            ← 喂脏 → 数据绑定增量 (11) / 渲染增量提取 (10)
```

- **01 设计 token 与风格契约**:把 STYLE-NOTES 的色板/密度/控件规格沉淀为中央 token 资产,定义设计语言契约,喂入样式选择器。
- **02 声明式布局接口**:面向编辑器作者的区域语义 + 槽位 + 约束 token 描述方式,基于 Taffy 与壳 autolayout。
- **03 类 JetBrains 停靠架构**:主文档标签 / 活动栏 / 抽屉区 / 底部输出 / 状态栏 / 浮窗的骨架与职责。
- **04 布局预设与持久化**:布局档案(Authoring/Review/Focus/Debug),按页面/用户保存恢复。
- **05 编辑器页面布局模板**:13 个页面在同一壳内的区域填充模板与状态规范。
- **06 浮动窗口与设计对齐**:命令面板/偏好/独立编辑器浮层规则,设计图对齐验收。
- **07 窗口化 / Chrome 页签 / 可吸附抽屉**:每种使用目的 = 可窗口化目的视图;窗口内 Chrome 浏览器式页签合并;抽屉注册进 window 且可独立成页面/转移吸附。
- **08 插件页面接口与消息交互**:插件页面作为一等目的视图接入,经消息协议与编辑器/其它页面通信。
- **09 增量消息总线与刷新**:网页式多协议消息(pub-sub/req-rep/broadcast),但严格增量、视图级脏集、帧末批刷,避免全局卡顿;是 07/08/10/11 的承载基座。
- **10 真实渲染管线与渲染规范**:resolve→extract→command→batch→上屏五段确定性管线;编辑器 chrome token 单源、无禁用视觉;脏视图增量提取接 09。
- **11 数据绑定与响应式刷新**:`$token`/`$prop`/`$param` 分流解析;单向受控数据流;view-model 纯函数派生;绑定级脏依赖图聚合喂 09。
- **12 widget/slot 组件化**:组件三层(原子/组合/区域)+ slot 契约 + prop 契约 + 组件目录;页面差异靠 slot 填充而非新增控件。
- **13 类 CSS Taffy 约束语言**:flex/grid/block 类 CSS 词汇 → `UiLayoutStyle` 映射;约束 token 化(chrome 禁裸值);family 决策;`UiSlotKind`↔family 衔接 12。
- **14 Unreal+React 组件思想综述**:统领性心智模型,把 widget+slot / 单向受控 / 失效增量逐条对齐到既有 DTO 与 10-13,纯思想不出代码。

依赖波次:思想前置 `14`(指导全局);视觉前置 `01`;承载前置 `09`。主链 `14 → 01 → {12, 13, 11} → 02 → 03 → {04, 05} → 06`;渲染收口 `10`(依赖 01/13/09);承载链 `09 → {07, 08}`。12/13/11 为拼装基元,先于 02 的区域声明落地(02 用它们拼区域);10 依赖 01 token、13 约束几何、09 脏集。

## 4. 全局设计语言约束(所有子计划必须遵守)

1. 视觉权威 = `editor-workbench-designs/*-layout-spec.png` 的结构 + `ai-workbench-web-framework.png` 的色彩/壳结构;交互语义 = `component-prototype`。
2. 色板单源:近黑面板色阶 `#111416`/`#171a1d`/`#1b1f23`/`#252b31`;teal `#3cc7d6` **仅**用于激活标签/选中/焦点/关键态,不做装饰。
3. 控件规则:圆角矩形、1px 边框、扁平态、28-32px 控件高、低圆角;**禁止**渐变、辉光、阴影、嵌套卡片、英雄字号。
4. 状态优先级固定:disabled > pressed > selected/focused > hovered > default(与 `editor_ui/04` 选择器一致)。
5. 布局描述唯一方式:Flex/Grid/Block/Wrap 走 Taffy;Overlay/Canvas/Scroll/Virtual/docking 走壳 autolayout;**不**手写绝对坐标。
6. 抽屉区职责按 §1.1 固定,新面板入槽前先确认槽位语义匹配。
7. 一切布局描述落在 `.zui` 资产 + 区域语义,不在 Rust 壳代码内联具体像素;结构性根 wiring 文件保持薄(遵守 `engine-code-structure-convention.md`)。
8. 不内嵌设计 PNG 截图;不照搬 web 原型的 HTML/CSS 实现。
9. 组件化:界面由组件三层(原子/组合/区域)+ slot 拼装;新页面不新增 primitive,差异靠 slot 填充表达(12)。
10. 约束语言:布局用类 CSS 词汇 → Taffy 求解;约束尺寸 token 化,chrome 资产三处(资产扫描/13 约束/10 渲染)一致禁裸值(13)。
11. 数据流:state→view 单向受控,view 改动一律走事件→命令,禁止 view 侧 mutate state/view-model(11)。
12. 增量即默认:任何变更产生最小脏集(09 视图级 + 11 绑定级),只重算/重提取(10)脏部分;无全量刷新常规入口。
13. 取思想不取运行时:不引入虚拟 DOM/运行时整树 diff,不照搬 Slate 对象模型;一切落到既有 retained 模板 + Taffy + 绑定表达式 + 09 脏集(14)。

## 5. 里程碑波次

| 波次 | 内容 | 依赖 |
| --- | --- | --- |
| W1 | 01.S1 token 资产骨架 + 风格契约文档 | — |
| W2 | 01.S2 token 喂入样式选择器 + 验收 | W1 |
| W3 | 02.S1 区域语义 + 约束 token 接口草案 | W2 |
| W4 | 02.S2 声明式布局描述落到壳 autolayout | W3 |
| W5 | 03.S1 工作台骨架 + 抽屉区职责落地 | W4 |
| W6 | 03.S2 停靠语义(展开/折叠/分屏/激活) | W5 |
| W7 | 04.S1 布局预设档案 | W6 |
| W8 | 04.S2 持久化(按页面/用户) | W7 |
| W9 | 05.S1 核心页面布局模板(场景/材质/Inspector) | W6 |
| W10 | 05.S2 其余页面 + 状态规范 | W9 |
| W11 | 06.S1 浮动窗口浮层规则 | W7,W10 |
| W12 | 06.S2 设计对齐验收 + 收口 | W11 |
| W0a | 09.S1 增量消息总线 + 视图脏集(承载前置) | — |
| W0b | 09.S2 增量 refresh_view 替换全量重建 | W0a |
| W7a | 07.S1 目的视图 + Chrome 页签合并 | W6,W0b |
| W7b | 07.S2 tear_off + 抽屉拆/合/吸附 + 页签↔抽屉升降级 | W7a |
| W8a | 08.S1 插件页面接口 + 生命周期 | W7a,W0b |
| W8b | 08.S2 页面消息进事件运行时 + 插件示例 | W8a |
| W-T | 14 Unreal+React 思想综述(心智模型,统领 10-13) | — |
| W12a | 12.S1 组件三层 + slot/prop 契约 + 目录骨架 | W2,W-T |
| W12b | 12.S2 slot 填充校验 + 现有物料归层补契约 | W12a |
| W13a | 13.S1 类 CSS 约束词汇映射 + token 化 + family 决策 | W2,W12a |
| W13b | 13.S2 slot↔family 映射接入 + chrome 约束 token 化复核 | W13a |
| W11a | 11.S1 受控契约 + `$` 分流 + 绑定依赖图 | W12a,W0a |
| W11b | 11.S2 view-model 派生接 09 脏集 + 局部写回 | W11a,W0b |
| W10a | 10.S1 渲染契约 guard + 管线文档 + 禁用视觉拦截 | W2,W13a |
| W10b | 10.S2 脏视图增量提取接 09 + 验收指标 | W10a,W0b |

## 6. 与 `editor_ui/` 的边界

| 维度 | `editor_ui/` 负责 | `editor_layout/` 负责 |
| --- | --- | --- |
| 布局引擎 | Taffy 后端、增量布局、虚拟化(02) | 用引擎能力声明区域/槽位/约束(本 02) |
| 样式 | 选择器机制、伪状态、组件内联清理(04) | 中央 token、设计语言契约(本 01) |
| 壳 | 承载切换、docking 运行时、窗口注册(08) | 壳骨架结构、抽屉区职责、停靠语义(本 03) |
| 模块 | 模块数据接线、EditorRuntimeClient(09) | 模块页面布局模板、状态规范(本 05) |
| 组件 | 组件目录运行、prototype 实例化器、widget 行为(02/06) | 组件三层/slot 契约/prop 契约/组件目录规范(本 12) |
| 约束 | Taffy 后端、measure/arrange pass、虚拟化(02) | 类 CSS 约束词汇映射、约束 token 化、family 决策(本 13) |
| 绑定 | 绑定表达式解析器、UiEventRouter、update_report(05/10) | 单向受控契约、view-model 派生、绑定级脏依赖图(本 11) |
| 渲染 | extract/command/batch DTO、render framework、上屏(11/12) | chrome 渲染契约、token 单源、禁用视觉拦截、脏区增量提取(本 10) |

本目录不重复 `editor_ui/` 的运行时构建;若发现某缺口属于运行时能力(而非声明/设计语言),应回流到 `editor_ui/` 对应子计划,而不是在此重做。

## 7. 编辑器完成阶段记录

本总览只汇总各子计划 `## 状态与产出记录` 的当前事实,不替代子计划状态表。完成判定以每个子计划最后一列的后续项和验证证据为准。

| 日期 | 范围 | 状态 | 产出/证据 | 后续项 |
| --- | --- | --- | --- | --- |
| 2026-06-23 | 01.S1 / 02.S1 / 03.S1 / 04.S1 / 05.S1 / 06.S1 首批布局架构落地 | implemented-static-passed-editor-cargo-blocked | 已新增中央 editor design token DTO 与资产、区域语义/职责校验、JetBrains 工作台骨架、四套布局预设、Scene/Material/Inspector 页面模板、命令面板/偏好浮层声明,并同步 `docs/ui-and-layout/design-language-contract.md` 与 `docs/ui-and-layout/workbench-skeleton-contract.md`。`cargo test -p zircon_runtime_interface --lib editor_design_tokens --offline` 3/3 通过;scoped rustfmt、`git diff --check`、新 editor layout 模块 `unwrap/expect/TODO/allow(dead_code)/Result<_, String>` 扫描通过。 | 继续 01.S2 token 喂入选择器与裸色清理、02.S2 壳 autolayout 资产加载与尺寸 token 替换、03.S2 停靠状态运行时、04.S2 页面/用户持久化、05.S2 其余页面模板、06.S2 设计对齐验收。`zircon_editor` Cargo gate 当前在进入 editor 代码前被 active render 下层编译漂移阻塞: `MeshPassCommandBuffers`、`CachedMeshDrawLookup/CachedMeshDrawKey`、`MeshDraw` 等 mesh import 未解析。 |
| 2026-06-23 | 01.S2 token feed 首段 + 新增布局资产 token 引用 | partial-runtime-interface-passed-editor-cargo-blocked | `EditorDesignTokens::resolve_painter_style(...)` 已作为 token 到 painter selector 的 feed path 落地,focused runtime-interface RED/GREEN 完成;新增 skeleton/floating layout 资产导入 `editor_tokens.v2.ui.toml` 并引用 `editor.*` token 名;`editor_layout_contracts.rs` 增加新增资产 token 引用静态 guard。 | 01.S2 尚未全量完成:旧 shell/module 资产仍有历史裸 hex/裸规格,需在 render 下层修复后恢复 editor Cargo lane,再继续资产族硬切和 retained painter 验收。 |
| 2026-06-23 | render 下层阻塞修复 + 01.S2/02.S2a editor 验证恢复 | partial-editor-verified | 按 support-first 原则先修复阻塞 editor lane 的下层 render mesh owner split 漂移:mesh root re-export `MeshPassCommandBuffers`,mesh_pass root re-export `CachedMeshDrawLookup`,`mesh_draw_command_list::builder` 改为正确上级路径。随后完成 `EditorDesignTokens::density_value_for_token_name(...)`、`WorkbenchSkeleton::preferred_region_extents_from_tokens(...)`,并把 `workbench_main_band.zui`/`workbench_scene_tree_panel.zui`/`workbench_inspector_panel.zui` 抽屉宽度改为 `$--left-drawer-width`/`$--right-drawer-width`。验证:`cargo test -p zircon_runtime_interface --lib editor_design_tokens --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-token-feed-0623 --message-format short --color never` 5/5 通过;`cargo check -p zircon_editor --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never` 通过;`cargo test -p zircon_editor --lib editor_layout_contracts --no-run --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never` 通过;直接运行测试二进制 `editor_layout_contracts --test-threads=1 --nocapture` 8/8 通过。 | 继续 02.S2 authored `shell_regions.v2.ui.toml` 加载入口、01.S2 旧资产族裸色/裸规格 hard cutover、03.S2 停靠状态运行时、04.S2 持久化、05.S2 其余页面模板、06.S2 设计对齐验收。 |
| 2026-06-23 | 02.S2 authored shell_regions asset ingestion | implemented-focused-passed | 新增 `shell_regions_asset` owner,把 `shell_regions.v2.ui.toml` 解析为 typed asset header + 通过职责校验的 `RegionBinding` 列表,再投影到 `WorkbenchSkeleton`;真实资产生成的 skeleton extents 已通过测试继续进入 `compute_workbench_shell_geometry(..., transient_region_preferred)`。验证:`cargo test -p zircon_editor --lib editor_layout_contracts --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never -- --test-threads=1 --nocapture` 10/10 通过,并完成 scoped rustfmt/debt/尾随空白/锁文件检查。 | 02.S2 focused path 已关闭;下一步按依赖进入 03.S2 停靠语义运行时,或继续 01.S2 旧资产族 token hard cutover。 |
| 2026-06-23 | 03.S2 docking command semantics | implemented-static-passed-lower-ui-support-repaired-cargo-timeout | 新增 layout command typed error owner,并把抽屉 collapse/activate、center split、focus active-tab 合同压到 `LayoutManager` 命令路径和 focused contract tests。生产触及文件没有新增 `unwrap`/`expect`/`TODO`/`allow(dead_code)`/裸 `Result<_, String>`;scoped rustfmt、diff check、尾随空白与生产债务扫描通过。 | clean target-dir focused test 暴露下层 runtime UI template style split 漂移;已把 `slot_contract` helper 通过 `style_apply` 父模块重新暴露,不回滚 child owner。下层 `cargo check -p zircon_runtime --lib --offline` 606s 超时无诊断,未取得 Cargo 通过;之后继续 04.S2 持久化、05.S2 页面模板与 06.S2 设计对齐时需保留该验证债。 |
| 2026-06-23 | 04.S2 page/user layout persistence | implemented-focused-passed | `layout_preset.rs` 新增页面/用户 scope、版本化持久化 entry/store、缺失/版本不匹配 Authoring 回退,并提供从 `WorkbenchLayout` 捕获/恢复抽屉 mode、extent/token 和 center split 形状的路径;`ui/host/layout_persistence.rs` 与 `editor_manager_layout.rs` 暴露页面布局保存/恢复接口;`ui/host/layout_commands.rs` 在 `ActivateMainPage` 时按 default 用户保存旧页并恢复目标页。验证:`cargo test -p zircon_editor --lib layout_preset_persistence --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623-clean-2309 --message-format short --color never -- --test-threads=1 --nocapture` 2/2 通过。 | 04.S2 focused path 已关闭;继续 05.S2 其余页面模板、06.S2 设计对齐验收,并保留 03.S2 全量 editor-layout Cargo 与旧 shell/module token hard cutover 验证债。 |
| 2026-06-23 | 05.S2 remaining page templates and state profiles | implemented-focused-passed | `PageLayoutTemplate::builtin_templates()` 与 `page_templates.v2.ui.toml` 补齐 13 个页面模板和默认状态,包括 focus/review/debug profile 与 split center 形状;新增 `page_layout_templates.rs` 验证页面集合、区域职责、状态字段和资产声明。验证:`cargo test -p zircon_editor --lib page_layout_templates --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623-clean-2309 --message-format short --color never -- --test-threads=1 --nocapture` 4/4 通过;过程中按 support-first 最小修复 runtime UI surface split 可见性/导入漂移。 | 05 focused path 已关闭;继续 06.S2 浮动窗口设计对齐、01.S2 历史资产 token hard cutover、03.S2 更宽 Cargo 复验债。 |
| 2026-06-23 | 06.S2 floating window design parity contracts | implemented-focused-passed-visual-host-pending | 新增 `FloatingWindowDesignContract` 和 `floating_window_design_parity.rs`,把命令面板、偏好、独立编辑器的 layer/modal/placement/content/interaction 合同固化到 Rust,并解析真实 floating `.zui` 资产验证 tokenized flat chrome、命令面板键盘 overlay 布局、偏好窗左导航/右内容结构。验证:`cargo check -p zircon_runtime --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-runtime-state-reducer-0623 --message-format short --color never` 通过;`cargo test -p zircon_editor --test integration_contracts --features integration-contracts floating_window_design_parity --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-runtime-state-reducer-0623 --message-format short --color never -- --test-threads=1 --nocapture` 4/4 通过;scoped rustfmt/diff/尾随空白/生产债务扫描通过。 | 06 focused path 已关闭;真实 retained-host 截图/像素比对待稳定窗口 harness 后补。继续 07/08/09 承载链、01.S2 历史资产 token hard cutover 与 03.S2 更宽 Cargo 复验债。 |
| 2026-06-23 | 09.S1 incremental message bus and view dirty set | implemented-focused-passed | 新增 `zircon_editor/src/core/editor_message/` folder-backed core owner,提供 `EditorTopic`、`EditorMessageBus`、pub-sub/request/broadcast、subscriber inbox、`EditorViewInvalidationMask` 与 `ViewDirtySet` 帧末 drain;测试覆盖主题精确唤醒、视图脏集合并、request-response target 校验和 broadcast 全量通知。验证:`cargo test -p zircon_editor --lib editor_message --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-runtime-state-reducer-0623 --message-format short --color never -- --test-threads=1 --nocapture` 4/4 通过。 | 09.S2 需桥接到 `EditorEventRuntime::refresh_view(view_id, mask)` 并把 core mask 转 retained-host mask;07/08 仍应等 09.S2 增量刷新入口落地后接入。 |
| 2026-06-23 | 09.S2 refresh_view dirty-drain bridge | implemented-focused-passed-partial-snapshot-backend | `EditorEventRuntimeState` 接入消息总线,新增 `refresh_view(...)`/`drain_pending_view_refreshes(...)` 与 `EditorViewRefreshReport`;事件分发和状态访问路径改为先记录 view dirty 再 materialize。验证:`cargo test -p zircon_editor --lib editor_message --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-runtime-state-reducer-0623 --message-format short --color never -- --test-threads=1 --nocapture` 5/5 通过。 | 当前 `EditorUiControlService` 只有 full snapshot publish,因此本切片保留 full snapshot materialize fallback;真正局部 snapshot/diff 发布仍是后续项。 |
| 2026-06-24 | 新增 10-14 子计划并入总览(渲染/绑定/组件化/约束语言/思想综述) | planned-docs-only | 按"现状核实+缺口+切片化+dev锚点"风格新增 `10-real-rendering-pipeline-and-contract.md`(resolve→extract→command→batch 五段渲染契约 + token 单源 + 禁用视觉拦截 + 脏视图增量提取)、`11-data-binding-and-reactive-contract.md`(`$` 分流 + 单向受控 + view-model 派生 + 绑定级脏依赖)、`12-widget-slot-componentization.md`(组件三层 + slot/prop 契约 + 组件目录)、`13-taffy-css-constraint-language.md`(类 CSS 词汇→`UiLayoutStyle` 映射 + 约束 token 化 + family 决策 + `UiSlotKind`↔family)、`14-unreal-react-composition-thesis.md`(Unreal widget+slot / React 单向受控 / Slate FastUpdate 失效增量统领心智模型);`index.md` 同步五条设计目标、缺口表 10-14、分层依赖图、全局约束 9-13、里程碑波次 W-T/W10-13、与 `editor_ui` 边界的组件/约束/绑定/渲染四行。全部子计划经 Explore 核实锚定既有 DTO:`UiRenderCommand`/`UiBatchPlan`/`UiBindingExpression`/`UiSlotKind`/`UiSlotSchema`/`UiLayoutStyle`/`taffy_bridge`。 | 各子计划仍为 planned;实现按波次 `14→01→{12,13,11}→…→10` 推进,先落 12/13 拼装基元再接 02 区域声明,10 渲染契约依赖 01/13/09。 |
