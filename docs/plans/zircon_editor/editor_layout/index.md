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
  - zircon_runtime/src/ui/layout/style_mapping.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/compute.rs
  - zircon_runtime_interface/src/ui/event_ui/mod.rs
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/surface/input
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/compact_layout.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/labels.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/summary_nodes.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/summary_layout.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/table_nodes.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_nodes.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_layout.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/tests.rs
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

1. **用户友好的设计接口**：编辑器作者(以及后续插件作者)用一套**声明式、可组合、低门槛**的布局描述接口拼装界面,而不是手写绝对坐标或散落像素调参,且布局**响应式 + 自适应不同分辨率/比例/DPI**(相对比例 + 逻辑单位 + 断点,见 16)。接口面向 `.zui` 组件资产 + 区域语义 + 布局预设,而非运行时内部结构。**这是工程化目标:界面由"声明 → token → 预设组件 → 类 CSS 约束 → 确定性渲染"的工作流生成,拒绝像素级直绘与一次性简单实现(见 §4 界面生成工作流)。**
2. **一致的设计风格**：所有面板、控件、停靠区共享同一套**设计 token(色板、密度、控件规格、间距、边框)**与状态优先级,保证 13 个编辑器页面与浮动窗口在视觉上是一套语言而不是拼贴。
3. **类 JetBrains 的设计架构**：主文档标签 + 活动栏 + 固定停靠抽屉区(5-6 槽) + 底部输出 + 状态栏 + 浮动窗口的工作台骨架,带布局预设与持久化。停靠语义、区域职责、抽屉行为与 JetBrains/Unreal 编辑器对齐。
4. **组件化、泛用化的拼装基元**：界面由**可复用、泛用化的 widget 组件 + slot 插槽**拼装,布局由**类 CSS 的 Taffy 约束语言 + token**声明,数据由**单向受控的绑定**驱动——借鉴 **Unreal Slate 的 `SWidget+FSlot`、React 的组件化/单向数据流(预设组件 + props/slots + 组合优于继承)、Material-UI 的 token 化设计系统(色板/间距节拍/断点/变体·状态)、Taffy 的类 CSS 算法**,但全部落到既有 seam(详见 12/13/11/14)。**硬目标:用 React 风格的 CSS 架构预设组件拼装,不为单页造控件、不内联像素样式(见 §4 界面生成工作流)。**
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
| 13 | 缺类 CSS 的 Taffy 约束语言规范(flex/grid/block 词汇映射、约束 token 化、family 决策);并缺 CSS 覆盖度盘点(哪些已实现/受限/需 DTO 扩展)与跨引擎依据 | 13 |
| 14 | 缺统领性思想综述(Unreal widget+slot / React 单向受控 / 失效增量如何映射既有 seam) | 14 |
| 15 | 缺把设计语言**落到像素**的组件视觉标准化(原子度量单源、文本省略、页签溢出、复合/领域自适应),真实截图仍很糙 | 15 |
| 15a | 缺主页签条几何单源与超宽溢出收纳(描画/命中不一致致 `Sce` + 错位),S15.3 深化 | 15a |
| 15b | 缺工作台 chrome 控件度量单源(~20 常量散落,需与 MUI 移植件二层切分),S15.1 深化 | 15b |
| 15c | 缺工作台 chrome 色板单源收口(retained 手写色板与中央 token 已漂移 border/text/error),S15.6 深化 | 15c |
| 15d | 缺复合控件密度/对齐(Asset Browser 固定列比例无最小宽致 `Siz/Rev`、数值列不右对齐),S15.4 深化 | 15d |
| 15e | 缺领域层断点自适应(compact 阈值裸值、只钳不折叠、与 640/900/1260 不对齐),S15.5 深化 | 15e |
| 16 | 缺相对布局与多分辨率自适应规范(壳层走向硬编码裸像素 + 手工像素累加、`scale_factor` 未参与布局、断点用物理像素;需 anchor 相对比例 / DPI 根缩放 / stretch / flex 充分利用),统领 13/15e | 16 |
| 17 | 缺文本渲染与排版规范(测量用 `font_size*0.5` 等宽近似致字符错位/溢出、字形按固定字号光栅化不随 DPI 重栅格致像素化、无文本大小自适应、换行基于 `max_chars` 近似);需测量=绘制真实字形度量 / DPI 重栅格 / 默认多行换行 / 自适应 | 17 |
| 18 | 缺输入响应与命中测试模型(命中散在 11 个手写 pointer bridge、路由次序未单点固化、无 pointer-events/相位/指针捕获/拖拽阈值统一);需命中单源 + capture/target/bubble 三相 + 捕获 + cursor | 18 |
| 19 | 缺焦点与导航模型(仅 `navigation_dispatcher` 参数名,无焦点作用域/Tab 顺序/方向键·手柄导航/焦点环/可达性);需 focusable+tab-index + 方向几何求解 + 模态陷焦 + `:focus-visible` | 19 |
| 20 | 缺真正的 USS/CSS 级联样式(仅固定优先级 selector + 无伪状态 v2 resolver,无选择器匹配/specificity/级联/var-token/computed/继承/transition,双路不同源);需级联引擎成熟化 `editor_ui/04` | 20 |
| 21 | 缺 GPU 提交与绘制管线细化(`10` 把批次合并/裁剪栈/图集/顶点/render-thread/dirty-region 推给运行时未规范);需批次键合并语义 + scissor/stencil 裁剪栈 + 动态图集 + 顶点吸附 + 局部重绘 | 21 |

## 3. 分层与依赖

```
统领思想层 (14 Unreal widget+slot / React 单向受控 / 失效增量 —— 心智模型,统领 10-13)
自适应规范层 (16 相对布局 / DPI 根缩放 / scale 模式 / stretch / flex 充分利用 —— 多分辨率自适应,统领 13/15e)
文本规范层 (17 测量=绘制真实字形度量 / DPI 重栅格 / 默认多行换行 / 文本自适应 —— 统领文本,接 13/16/10/15)
交互系统规范层 (18 输入响应/命中单源/三相 + 19 焦点导航/Tab/方向/焦点环 —— 横切交互,接 11/20/editor_ui/01)
      ↓ 指导
设计语言层 (01 设计 token + 风格契约)
      ↓ 喂入
拼装基元层 (12 widget/slot 组件化 + 13 类 CSS Taffy 约束语言 + 20 USS 级联样式 + 11 数据绑定)
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
提交规范层 (21 批次合并/裁剪栈/动态图集/顶点吸附/dirty-region —— 深化 10 的 batch→上屏,落到运行时 render)

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
- **13 类 CSS Taffy 约束语言**:flex/grid/block 类 CSS 词汇 → `UiLayoutStyle` 映射(每条带 `style_mapping.rs` 现状映射证据,证明求解能力已具备);约束 token 化(chrome 禁裸值)+ 取值规范化边界(Percent 归一 0..1、gap/padding 不可 auto、align-content 无 baseline);family 决策;`UiSlotKind`↔family 衔接 12;**CSS 覆盖三档矩阵 T1 已实现 / T2 受限 / T3 DTO 扩展候选**,每条带 Bevy/Unreal/Slint/Godot/Material-UI 已核实源码依据。
- **14 Unreal+React 组件思想综述**:统领性心智模型,把 widget+slot / 单向受控 / 失效增量逐条对齐到既有 DTO 与 10-13,纯思想不出代码。
- **15 组件视觉标准化(从原子到领域)**:把 01-14 的结构**落到像素**;借 UE Slate 度量/形状/响应、保留 Zircon teal 色板;按原子(按钮/文本/容器/图像/间隙)→复合(搜索栏/工具栏/状态栏/列表/弹窗)→领域(抽屉/窗口)自下而上标准化并自适应,每层以 `docs/tests/editor/` 截图验收。修复"页签被裁成 Sce"、文本硬裁剪、度量散落、色板两套等真实成品缺陷。
- **16 相对布局与多分辨率自适应**:自适应规范层,统领 13/15e。三层模型(① DPI 根缩放 → ② flex 相对布局 → ③ 断点 tier);相对优先单位(grow/basis%/min-max > 逻辑单位 token > 物理像素仅 center);anchor/stretch→flex 映射(带 UE `Anchors.h`/`SConstraintCanvas.cpp`/`SScaleBox.h`/`SWindow.h` 源码证据);点名壳层四反模式(裸像素、像素累加、断点用物理像素、`scale_factor` 未用)并给 flex 容器修正。纯规范不出代码。
- **17 文本渲染与排版**:文本规范层,统领文本。测量=绘制(真实字形 advance/kerning/ascent 替换 `font_size*0.5` 等宽近似,治错位/溢出);字形按 `scale_factor` 重栅格(治像素化,接 16);默认多行换行(word + 超长词逐字回退,接真实字形宽)+ 省略分离;文本自适应(auto-wrap 两阶段 + 可选 clamp 字号);baseline/度量单源。带 UE `FShapedGlyphEntry`/`ETextWrappingPolicy`/`STextBlock`、godot `AutowrapMode`/`OverrunBehavior`、bevy `FontSmoothing` 源码证据。纯规范不出代码。
- **18 输入响应与命中测试**:交互系统层。命中测试单源(治 11 个手写 pointer bridge)+ `pointer-events`/命中可见性 + capture/target/bubble 三相(≈ Unity UI Toolkit TrickleDown/BubbleUp)+ 指针捕获 + 拖拽阈值统一机 + cursor 声明。产出 hover/active/press 语义态喂 20,运行时派发归 `editor_ui/01`。带 UE `FHittestGrid`/`FReply`/`EVisibility`、Unity 事件相位/picking-mode、Slint `InputEventFilterResult`、Godot `MOUSE_FILTER`、Bevy `FocusPolicy` 已核实证据。纯规范不出代码。
- **19 焦点与导航**:交互系统层。可聚焦性 + Tab 顺序 + 方向键/手柄导航(几何求解 + 显式邻居)+ 边界规则(wrap/stop/escape/trap)+ 焦点作用域(模态陷焦/还原)+ `:focus-visible` 焦点环;产出 focus/focus-within 态喂 20,派发归 `editor_ui/01`。带 UE `EUINavigation`/`FNavigationReply`/`FHittestGrid::FindNextFocusableWidget`、Unity `FocusController`/`NavigationMoveEvent`/`tabIndex`、Godot `focus_neighbor`、Slint focus chain 已核实证据。纯规范不出代码。
- **20 USS 级联样式系统**:拼装基元层,成熟化 `editor_ui/04`。把"固定优先级 selector + 无伪状态 resolver"升级为真正的级联引擎:选择器(type/class/name/伪状态/后代)+ specificity + 级联 + 自定义属性(01 token 作 `var(--…)`)+ computed style + 继承 + transition;伪状态来源 18/19;retained 软绘与 render extract 同源(治双路漂移)。带 Unity USS(slide-toggle.uss 已核实)、CSS、Slint、Godot theme variation 证据。纯规范不出代码。
- **21 GPU 提交与绘制管线**:提交规范层,深化 `10` 的 batch→上屏。批次合并键(shader+texture+clip,clip 进键)+ layer/z 排序 + 裁剪栈(轴对齐 scissor / 非矩形 stencil)+ 动态图集(字形/图标合批,接 17 scale)+ 顶点装配与像素吸附(接 16,Taffy disable_rounding 后吸附归此层)+ render-thread 提交边界(传批次计划 DTO 非 wgpu 对象)+ dirty-region 局部重绘(接 09)。wgpu 实现归运行时 render 框架。带 UE `FSlateElementBatcher`/`FSlateClippingManager`/`FSlateRHIRenderer`、Unity UIR/dynamic atlas、Bevy `UiBatch`、Slint `PartialRenderer` 已核实证据。纯规范不出代码。

依赖波次:思想前置 `14`(指导全局);自适应前置 `16`(统领 13/15e 的相对布局/DPI/断点);文本前置 `17`(统领文本测量/光栅化/换行/自适应,接 13/16);视觉前置 `01`;承载前置 `09`。主链 `14 → 01 → {12, 13, 20, 11} → 02 → 03 → {04, 05} → 06`;交互系统 `{18, 19}` 横切(接 11/20,运行时落 `editor_ui/01`);渲染收口 `10`(依赖 01/13/09/17)→ 提交 `21`(深化 10 的 batch→上屏,落运行时 render);承载链 `09 → {07, 08}`;像素收口 `15`(依赖 01/10/12/13/16/17,把度量/文本/页签/复合/领域逐层标准化到截图验收)。12/13/20/11 为拼装基元,先于 02 的区域声明落地(02 用它们拼区域);10 依赖 01 token、13 约束几何、20 computed style、09 脏集;13/15e 的相对布局与断点判定遵 16;文本测量/换行/光栅化遵 17;交互态(hover/focus)由 18/19 产出喂 20 伪状态。

## 4. 界面生成工作流与全局设计语言约束(所有子计划必须遵守)

### 4.0 界面生成工作流(工程化硬目标,先于一切实现)

编辑器界面**不是**靠在 Rust 壳代码里逐个摆控件、调像素拼出来的;它由一条**单向、分层、可测试、可复用、可增量**的生成工作流产出。任何子计划、任何 UI 落地都必须沿这条流水线走,**这是工程化目标,不是建议**:

```
设计意图(参考图结构,不取像素)
   ↓ 取
① 设计 token 层 (01)  —— Material-UI 风格 token 化设计系统:色板/间距节拍(8px 基数)/排版/断点 tier/状态优先级,唯一视觉源;token 暴露为自定义属性供 var() 引用(20)
   ↓ 喂入
② 预设组件层 (12/11/14) —— React 风格 CSS 架构组件:组件三层(原子/组合/区域)+ slot 插槽 + 单向受控 props;可复用、可组合、有契约,组合优于继承
   ↓ 拼装
③ 类 CSS 约束 + 级联样式层 (13 布局 / 20 视觉) —— flex/grid/block + 相对档 + token 交 Taffy 求解(禁裸像素几何);选择器/specificity/级联/var/computed style 出最终视觉(USS 式,治固定优先级)
   ↓ 声明
④ 响应式 / 自适应层 (16/15e) —— 随窗口/容器连续伸缩(响应式)+ 随分辨率/DPI/断点/scale 模式改变缩放与结构(自适应);DPI 根缩放,断点用逻辑宽度
   ↓ 上屏
⑤ 渲染 + 提交层 (10 提取→命令→批次 / 21 批次合并·裁剪栈·图集·顶点吸附·dirty-region / 09 增量) —— 确定性管线产像素,只增量重算/重提取脏部分,批次化上屏

   ┃ 横切:交互系统 (18 输入响应/命中单源/三相 + 19 焦点导航/Tab/方向/焦点环) ——
   ┃ 命中/相位/焦点产出 hover/active/focus 语义态,回喂 ③ 的 20 伪状态选择器,不直接改视觉(单向受控)
```

每一层的**工程化判据**(违反即视为未完成,不得以"能显示"结案):

| 层 | 必须(MUST) | 禁止(MUST NOT) |
| --- | --- | --- |
| ① token | 视觉值一律取自中央 token(色/间距/字号/圆角/断点);Material-UI 式语义角色与变体 | 在组件/壳里写裸 hex、裸尺寸、裸断点阈值 |
| ② 组件 | 用预设组件 + slot 填充表达页面差异;props 单向受控;React 式声明 | 为单个页面/场景新造 primitive;在 Rust 壳代码内联控件或样式;view 侧 mutate state |
| ③ 约束+样式 | 布局用类 CSS 词汇 + token + 相对档交 Taffy;视觉用选择器+specificity+级联出 computed style(USS 式) | 手写绝对坐标;手工像素累加;裸物理像素(除 center 自由区);在绘制族写 `if state==...` 状态分支(应走伪状态选择器) |
| ④ 自适应 | 响应式连续伸缩 + 断点 tier 结构降级 + DPI 根缩放 + scale 模式;断点判定用逻辑宽度 | 固定单一分辨率布局;断点/字号用裸物理像素;`scale_factor` 不参与布局 |
| ⑤ 渲染+提交 | 确定性管线 + 批次合并(键含裁剪) + 裁剪栈 + 动态图集 + dirty-region 增量上屏 | 全量重建为常规入口;每节点一 draw call;绕过管线直接画 chrome |
| 横切 交互 | 命中测试单源 + capture/target/bubble 三相 + 指针捕获 + 全键盘可达 + 焦点环;交互态喂样式 | 在壳代码逐控件写命中矩形 + 手写 hover/press 状态机;键盘不可达;无焦点反馈 |

**一票否决的"简单实现"反模式**(出现任一即打回,无论是否"看起来对"):

- ✗ 直接用物理像素坐标/尺寸驱动 chrome UI(像素级直绘)。
- ✗ 手工竖向/横向像素累加算区域几何(见 16 R2)。
- ✗ 为某个页面一次性硬编码布局/控件,而非走 token+预设组件+约束三层(为单页造轮子)。
- ✗ 在 Rust 壳/host 代码里内联样式值或 widget 树,绕过 `.zui` 资产 + 组件契约。
- ✗ 在绘制族里按状态硬编码颜色/边框分支,而非走 20 的伪状态选择器(见 20)。
- ✗ 自带命中矩形 + 手写 hover/press/拖拽状态机,而非走 18 的命中单源 + 三相 + 捕获(见 18)。
- ✗ 键盘/手柄不可达、无焦点环(见 19)。
- ✗ 每节点一个 draw call、每帧全量重画,而非批次合并 + dirty-region 增量(见 21)。
- ✗ 断点阈值、字号、间距用裸物理像素而非逻辑单位 token(见 16 R3)。
- ✗ 用 `font_size*0.5` 等宽近似替代真实字形度量(见 17)。
- ✗ 把"能跑/能显示"当完成标准,跳过可复用/可测试/可增量的工程化要求。

> 本工作流是 §4.1 起各条约束的总纲;下列编号约束是它在各层的具体落点。新功能开工前先对照本流水线确认"我在哪一层、用哪层的语汇",再动手。

### 4.1 设计语言与落地约束

1. 视觉权威 = `editor-workbench-designs/*-layout-spec.png` 的结构 + `ai-workbench-web-framework.png` 的色彩/壳结构;交互语义 = `component-prototype`。
2. 色板单源:近黑面板色阶 `#111416`/`#171a1d`/`#1b1f23`/`#252b31`;teal `#3cc7d6` **仅**用于激活标签/选中/焦点/关键态,不做装饰。
3. 控件规则:圆角矩形、1px 边框、扁平态、28-32px 控件高、低圆角;**禁止**渐变、辉光、阴影、嵌套卡片、英雄字号。
4. 状态优先级固定:disabled > pressed > selected/focused > hovered > default(与 `editor_ui/04` 选择器一致)。
5. 布局描述唯一方式:Flex/Grid/Block/Wrap 走 Taffy;Overlay/Canvas/Scroll/Virtual/docking 走壳 autolayout;**不**手写绝对坐标。尺寸**优先用相对档**(`flex-grow` 权重 / `flex-basis` 百分比 / `auto` / `min`-`max`),固定厚度才用逻辑单位 token,物理像素裸值仅 center 自由区(16/13)。
6. 抽屉区职责按 §1.1 固定,新面板入槽前先确认槽位语义匹配。
7. 一切布局描述落在 `.zui` 资产 + 区域语义,不在 Rust 壳代码内联具体像素;结构性根 wiring 文件保持薄(遵守 `engine-code-structure-convention.md`)。
8. 不内嵌设计 PNG 截图;不照搬 web 原型的 HTML/CSS 实现。
9. 组件化:界面由组件三层(原子/组合/区域)+ slot 拼装;新页面不新增 primitive,差异靠 slot 填充表达(12)。
10. 约束语言:布局用类 CSS 词汇 → Taffy 求解;约束尺寸 token 化,chrome 资产三处(资产扫描/13 约束/10 渲染)一致禁裸值(13)。
11. 数据流:state→view 单向受控,view 改动一律走事件→命令,禁止 view 侧 mutate state/view-model(11)。
12. 增量即默认:任何变更产生最小脏集(09 视图级 + 11 绑定级),只重算/重提取(10)脏部分;无全量刷新常规入口。
13. 取思想不取运行时:不引入虚拟 DOM/运行时整树 diff,不照搬 Slate 对象模型;一切落到既有 retained 模板 + Taffy + 绑定表达式 + 09 脏集(14)。
14. 多分辨率自适应 + DPI 缩放:布局**像素无关**——几何来自相对比例 + 逻辑单位 token + 约束 + 断点 tier;token 是 **DPI 无关逻辑单位**,渲染前统一乘 `scale_factor`;断点判定用**逻辑宽度**(`physical / scale_factor`);禁止硬编码裸物理像素 + 手工像素累加(16,统领 13/15e)。
15. 文本质量:文本布局几何来自**真实字形度量**(测量=绘制,禁 `font_size*0.5` 等宽近似);字形按 `scale_factor` **重栅格**(禁固定字号拉伸致像素化);默认**多行换行**(word + 超长词逐字回退,单行+省略为显式特例);baseline/字号/行距度量单源(17)。
16. **响应式是默认,不是选项**:每个区域/组件/slot 在窗口或父容器尺寸变化时必须**连续伸缩**(grow/basis%/min-max),而非固定一套尺寸;无"仅在某分辨率下成立"的布局。响应式 = 同一结构随尺寸平滑变化(16 §3.2);自适应 = 跨断点 tier 改变结构与缩放(16 §3.1 / 15e)。两者都必须具备。
17. **Material-UI 风格的 token 化设计系统**:色板、间距(8px 基数节拍)、排版、断点、组件变体与状态全部 token 化、语义化(01);组件以"变体 × 状态"取样式,不在调用点写一次性视觉值。对标 `mui-system` 的 spacing/breakpoints/sizing(13 §3.2 / §11)。
    补注(2026-07-02 评审收口):**唯一 token 引用文法由 01 定义**——`$token` 内联(资产/约束侧)与 `var(--token)` 级联(20 样式侧)两种形态,两者由 01 的"token → 自定义属性注册表"单源映射。当前各计划实际写法**多种并存待收束**:`$--left-drawer-width`(02)、`$gap.m`(13)、`$editor.surface.1`(10)、`var(--editor-surface-1)`(20)、`editor.surface.recessed`(15c),共五种;收束时以 01 文法为准回改,各处不得再新增第六种写法。
18. **React 风格的预设组件,不许像素级直绘**:界面=预设组件(原子/组合/区域)+ slot 组合 + 单向受控 props 声明而成(12/11/14);**严禁**用像素坐标/尺寸在壳代码里直接画 UI 或为单页造控件;页面差异只能靠 slot 填充与 props 表达。像素裸值仅 center 自由区/用户内容豁免(13 §3.7)。
19. **工程化优先于"能跑",拒绝简单实现**:任何 UI 落地必须落在 token+预设组件+类 CSS 约束+确定性渲染四层(§4.0),并满足可复用、可测试、可增量;一次性硬编码、内联样式、绕层直绘、跳过脏集增量的实现一律视为未完成,即便视觉上正确也要重做。

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
| W-A | 16 相对布局/多分辨率/DPI 自适应规范(统领 13/15e,纯规范) | — |
| W-X | 17 文本渲染与排版规范(测量=绘制/DPI 重栅格/多行换行/自适应,纯规范) | — |
| W12a | 12.S1 组件三层 + slot/prop 契约 + 目录骨架 | W2,W-T |
| W12b | 12.S2 slot 填充校验 + 现有物料归层补契约 | W12a |
| W13a | 13.S1 类 CSS 约束词汇映射 + token 化 + family 决策 | W2,W12a,W-A |
| W13b | 13.S2 slot↔family 映射接入 + chrome 约束 token 化复核 | W13a |
| W11a | 11.S1 受控契约 + `$` 分流 + 绑定依赖图 | W12a,W0a |
| W11b | 11.S2 view-model 派生接 09 脏集 + 局部写回 | W11a,W0b |
| W10a | 10.S1 渲染契约 guard + 管线文档 + 禁用视觉拦截 | W2,W13a,W-X |
| W10b | 10.S2 脏视图增量提取接 09 + 验收指标 | W10a,W0b |
| W15a | 15.S1 retained host 中央度量 token 单源 + 各绘制族硬切换 | W2,W12a,W13a |
| W15b | 15.S2 文本原子优雅省略(替代硬裁剪) | W15a,W-X |
| W15c | 15.S3 页签条单行 + 溢出收纳(修复 "Sce") | W15b,W-X |
| W15d | 15.S4 复合控件统一密度/对齐 | W15a |
| W15e | 15.S5 抽屉/窗口三档宽自适应截图验收 | W15c,W15d,W-A |
| W15f | 15.S6 retained painter 色板由中央 token 喂入(收口 01.S2) | W15a,W2,W10a |
| W-I1 | 18.S1 pointer-events/cursor DTO + 命中单源契约 + 三相次序文档 | W2 |
| W-I2 | 18.S2/S3 bridge 命中迁命中单源 + 指针捕获/拖拽阈值统一(衔接 editor_ui/01) | W-I1 |
| W-N1 | 19.S1 focusable/tab-index/边界 DTO + Tab 链 + focus 态 | W-I1 |
| W-N2 | 19.S2/S3 方向导航几何求解 + 焦点作用域 trap/还原 + focus-visible 焦点环 | W-N1,W20a |
| W20a | 20.S1 选择器 + specificity + 级联引擎 + var/token + computed style | W2,W13a |
| W20b | 20.S2/S3 v2 resolver 接级联引擎 + retained/extract 同源 + 内联状态分支清除 | W20a,W-I1 |
| W21a | 21.S1 批次键合并语义 + layer 排序 + 裁剪栈(scissor/stencil)契约 | W10a |
| W21b | 21.S2/S3 动态图集合批 + 顶点吸附 + dirty-region 局部重绘(接 09) | W21a,W0b,W-X |

## 6. 与 `editor_ui/` 的边界

| 维度 | `editor_ui/` 负责 | `editor_layout/` 负责 |
| --- | --- | --- |
| 布局引擎 | Taffy 后端、增量布局、虚拟化(02) | 用引擎能力声明区域/槽位/约束(本 02) |
| 样式 | 选择器机制、伪状态、组件内联清理(04) | 中央 token、设计语言契约(本 01);USS 级联/选择器/specificity/computed 语义(本 20) |
| 壳 | 承载切换、docking 运行时、窗口注册(08) | 壳骨架结构、抽屉区职责、停靠语义(本 03) |
| 模块 | 模块数据接线、EditorRuntimeClient(09) | 模块页面布局模板、状态规范(本 05) |
| 组件 | 组件目录运行、prototype 实例化器、widget 行为(02/06) | 组件三层/slot 契约/prop 契约/组件目录规范(本 12) |
| 约束 | Taffy 后端、measure/arrange pass、虚拟化(02) | 类 CSS 约束词汇映射、约束 token 化、family 决策(本 13) |
| 绑定 | 绑定表达式解析器、UiEventRouter、update_report(05/10) | 单向受控契约、view-model 派生、绑定级脏依赖图(本 11) |
| 渲染 | extract/command/batch DTO、render framework、上屏(11/12) | chrome 渲染契约、token 单源、禁用视觉拦截、脏区增量提取(本 10) |
| 输入 | Slate 式事件内核:归一化/路由/Reply 派发、navigation dispatcher(01) | 输入响应语义契约:命中单源、pointer-events、capture/target/bubble 三相、捕获、cursor(本 18) |
| 焦点导航 | 焦点/导航运行时派发(01) | 焦点与导航语义契约:focusable/tab/方向几何/边界/作用域/焦点环(本 19) |
| GPU 提交 | wgpu pipeline/bind group/draw、render framework 上屏(rhi/rhi_wgpu) | 提交契约:批次合并键、裁剪栈、动态图集、顶点吸附、dirty-region(本 21,深化 10) |

本目录不重复 `editor_ui/` 的运行时构建;若发现某缺口属于运行时能力(而非声明/设计语言),应回流到 `editor_ui/` 对应子计划,而不是在此重做。

### 6.1 与 editor_ui / runtime / render 的关系(职责链单源)

`editor_layout/` 是**规范/契约层**,不持有运行时实现;它定义"界面如何被声明、被组织、被约束、被验收",由下游三层落地。四层职责链单向、不重叠:

```
editor_layout/  规范层  —— 设计语言 + 声明接口 + 约束/样式/输入/导航/提交契约(本目录,纯规范/契约,多为 docs + DTO)
      ↓ 规范约束(契约 DTO 在 zircon_runtime_interface)
editor_ui/      运行时 UI 能力 —— Slate 输入内核(01)、Taffy 布局后端(02)、样式选择器/级联 resolver(04)、文本栈(03)、组件目录(06)、壳承载(08)
      ↓ 调用
zircon_runtime  引擎运行时 —— ui/ 子系统(layout pass、style_mapping、surface、render extract/command/batch)、taffy_bridge、字体光栅、事件运行时
      ↓ 提交
render(rhi / rhi_wgpu) GPU —— wgpu pipeline/bind group/draw/clip/atlas 上屏
```

- **契约落点**:`editor_layout/` 的 DTO 契约统一落在 `zircon_runtime_interface`(中立 ABI 层),`editor_ui/` 与 `zircon_runtime` 都消费同一契约,保证规范单源。
- **谁实现谁**:18/19 的输入·导航语义 → `editor_ui/01` 派发实现 → `zircon_runtime` 事件运行时;20 的级联样式 → `editor_ui/04` + `zircon_runtime` v2 resolver;13 的约束 → `zircon_runtime` taffy_bridge/style_mapping;21 的提交 → `zircon_runtime` render extract/batch → render(rhi_wgpu)上屏。
- **回流规则**:`editor_layout/` 发现的缺口若属运行时能力,回流到 `editor_ui/` 对应子计划;若属引擎内部(布局 pass、光栅、wgpu),回流到 `zircon_runtime`/render,**不在规范层写实现**。
- (2026-07-02 评审收口)**文本实现权威** = `docs/plans/zircon_runtime/text/`(排版/字形度量/光栅/caret/IME 契约均以其为准);本目录 `17` 已降级为**编辑器侧排版验收规范**,不再持文本实现契约。
- (2026-07-02 评审收口)**2D/GPU 文本图集与 glyph quad 批实现权威** = zircon_runtime render 计划(render/14);`21` 只持 UI 批次契约(批次键/裁剪栈/提交 DTO),不持图集/字形批实现。
- 反向:`editor_ui/`、`zircon_runtime`、render 的计划须在各自"现状/边界"标注"遵 `editor_layout/NN` 契约",形成双向勾稽(见各下游计划)。

## 7. 编辑器完成阶段记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

实现切片与运行时验证统一遵循 [`milestone-validation-policy.md`](../../milestone-validation-policy.md)：规范/契约切片只运行静态一致性检查，关联 editor_ui/runtime/render 的 Cargo 与实机验证按依赖里程碑集中执行。

Editor Layout 总索引中的完成阶段明细已迁入 Editor Layout 15 产出目录。

- 迁入记录：[`15/2026-07-09-index-output-records.md`](15/2026-07-09-index-output-records.md)
