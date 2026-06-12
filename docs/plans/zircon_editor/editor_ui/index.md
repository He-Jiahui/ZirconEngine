---
related_code:
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/layout/mod.rs
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/v2/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/src/ui/host/module.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SWidget.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/SlateApplication.h
design_references:
  - docs/ui-and-layout/ai-workbench-style/ai-workbench-web-framework.png
  - docs/ui-and-layout/ai-workbench-style/component-prototype/index.html
  - docs/ui-and-layout/editor-workbench-designs/STYLE-NOTES.md
plan_sources:
  - .codex/plans/ZirconEngine 宿主编辑器 UI 基础能力计划.md
  - .codex/plans/Shared Slate-Style UI Layout, Render, And Hit Framework.md
  - .codex/plans/UI Layout 架构评审与 Taffy 收敛计划.md
  - .codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md
  - .codex/plans/Zircon Editor Runtime UI Rust-Owned Retained Host 重构计划.md
  - .codex/plans/Zircon Editor Workbench Shell V1.md
  - .codex/plans/GPU Command Stream 接管 Editor UI 渲染计划.md
  - .codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md
---

# Zircon Editor UI 总体架构计划

本目录是「ZirconRuntime 承载完整 UI 运行时（渲染、输入、布局、资产管理）→ ZirconEditor 用该能力拼装编辑器模块」的总计划。它承接 `.codex/plans` 中已完成与归档的 UI 计划，聚焦把宿主编辑器 UI 做成**真实可渲染、可布局、可响应输入、可资产化管理**的运行层，最终让编辑器主界面达到设计图的画面组织结构——不是逐像素复刻，而是组件化、可统一描述的结构对齐。各子计划已细化到工程落地深度（接口草案、文件落点、切片化里程碑、测试矩阵、依赖表）。

## 1. 设计权威与参考映射

主架构参考为 `dev/UnrealEngine` 的 Slate（成熟框架依据），辅以 dev 目录下的 Fyrox / Godot / Slint / bevy，样式与组件分类参考本地 `dev/material-ui`（MUI），动画编辑参考 `dev/theatre`：

| 参考物 | Zircon 对应物 | 取什么 / 不取什么 |
|--------|---------------|------------------|
| `SWidget` / `SPanel` / `FChildren` | `UiTree` 节点、Slot、`UiComponentDescriptor` | 取组件树 + Slot 模型；不取 C++ 宏体系与 UMG 层 |
| `FGeometry` / `FArrangedChildren` / `FWidgetPath` | `UiArrangedTree` / `UiArrangedNode` / `UiHitPath` | 取 arranged geometry 为唯一空间事实 |
| `FSlateApplication` / `GenericApplicationMessageHandler` | Zircon 统一输入管理层（计划 01） | 取统一入口 + preview(tunnel)/bubble/direct/focus-path/capture 路由 |
| `FReply` | `UiDispatchReply`（已在 interface，计划 01 补全应用面） | 取声明式副作用：handled、capture、focus、drag、popup |
| `OnPaint` / `FSlateDrawElement` | `UiRenderExtract` / `UiRenderCommand`（已闭环） | 取 draw-list 提取；渲染由 GPU command stream 承担 |
| Slate 每 Panel 手写 `OnArrangeChildren` | **不照搬**：Flex/Grid/Block/Wrap 统一交 Taffy（计划 02） | 仅 Overlay/Canvas/Scroll/Virtual/docking 保留 Zircon 自有布局 |
| MUI 组件分类与 sx/variant 体系 | material_foundation catalog + `.zui` 组件资产（计划 06） | 取自底向上组件族、variant/state 语义、类 CSS 的统一布局描述 |
| Fyrox `DockingManager` / Inspector | editor workbench docking（计划 08） | 取 retained tree + docking 语义参考 |
| Slint | `zircon_hub` 专用，不回流 editor | 仅作声明式 UI 语法参考 |
| theatre 时间轴 / keyframe 编辑 | UI 动画引擎 + 动画编辑器面板（计划 07） | 取 sheet/object/track/keyframe 数据模型与编辑交互 |

## 2. 现状评审结论

既有计划已落地的事实（详见各子计划「现状（按代码核实修正）」节，已逐项对照真实代码），新计划在此之上推进，不重做：

- **布局**：Taffy 已是布局后端（taffy_bridge）；引擎选择/13 种 fallback reason 枚举已在 interface；增量布局、滚动、虚拟化模块存在。
- **输入**：`UiDispatchReply`/`UiDispatchEffect`/`UiInputRoutePolicy`（含 PreviewTunnel/Direct/FocusPath/PointerCapture）/批入口/reply 应用器已在 interface + runtime；缺统一 manager 门面、触摸指针表与 editor 桥收编。
- **唯一事实**：`UiSurfaceFrame` 同时驱动布局、命中、渲染提取；`UiSurface` 已暴露 mutate_property/reflector_snapshot/dispatch 全套宿主 API。
- **资产**：`.zui` + `.v2.ui.toml` 双轨资产类型已注册；依赖失效 graph/fingerprint、resource_ref 路径级解析、watcher 全模块已存在；缺 theme/icon 类型、消费级 resolver、persistent cache。
- **样式**：`UiPainterStyleSelector` 已在 interface 按 family 折叠（优先级单源已成立）；缺中央 theme 文档、v2 伪状态、组件内联分支清理。
- **渲染**：GPU Command Stream 已接管 editor UI 渲染（software_fallback_count=0 验收过），编辑器无 raw wgpu 依赖。
- **编辑器壳**：Rust-owned retained host；shell 区域 L4 `.zui` 已有 8 件；view registry / window registry / preset / autolayout 模块齐备；11 个 core module workspace 存在。
- **风格**：暗色 Material token 定调；状态优先级固定 disabled > pressed > selected/focused > hovered > default。

骨架缺口与归属（本计划主战场）：

| # | 缺口 | 归属子计划 |
|---|------|-----------|
| 1 | 统一输入 manager 门面、winit 翻译双实现收口、editor 11 个 pointer bridge 手写命中 | 01 |
| 2 | 触摸多指针表 / IME 组合闭环 | 01 / 03 |
| 3 | 文本栈权威未定（glyphon 挂名未接、CJK fallback、测量缓存） | 03 |
| 4 | v2 伪状态解析、组件内联状态分支 | 04 |
| 5 | 中央 theme 文档、token 链、热重载 | 04 |
| 6 | UiThemeAsset/UiIconAsset、消费级 resolver、persistent cache（归档 M15） | 05 |
| 7 | UI 动画引擎与 keyframe 资产（归档 M19） | 07 |
| 8 | painter 双轨退役、菜单/命令/快捷键/浮窗/布局持久化 | 08 |
| 9 | 模块数据接线（场景命令栈、事件词汇、反射链） | 09 |
| 10 | 布局调试器（归档 M20）、统一布局属性集 | 02 |

## 3. 分层与归属

固定映射，所有子计划共享，不新增 crate：

| 层 | 归属 | 说明 |
|----|------|------|
| UI 契约（DTO、事件、布局 spine、render 命令） | `zircon_runtime_interface/src/ui/**` | ABI 安全值与序列化载荷，不过 trait object |
| UI 运行时（树、布局、命中、分发、文本、提取、模板编译、动画） | `zircon_runtime/src/ui/**` | 计划 01–04、07 主战场 |
| UI 资产管理（.zui/.ui.toml 加载、缓存、热重载、依赖、包验证） | `zircon_runtime/src/asset/**` + `ui/template`、`ui/v2` | 计划 05 |
| 组件库（material_foundation catalog + `.zui` 组件资产） | runtime catalog + `zircon_editor/assets/ui/editor/components/**` | 计划 06 |
| 编辑器工作台（workbench 模型、docking、窗口、模块） | `zircon_editor/src/ui/**` + 业务归 `core/`/`scene/` | 计划 08、09 |
| UI 渲染 | runtime GPU command stream + scene renderer UI pass | 已闭环，本计划不改其路径 |

## 4. 子计划地图与阶段

| 计划 | 文档 | 里程碑数 |
|------|------|---------|
| 01 Slate 式输入与事件内核 | `01-slate-input-dispatch-core.md` | M1–M5 |
| 02 布局：Taffy 权威与特殊容器 | `02-layout-taffy-and-containers.md` | M1–M5 |
| 03 文本与字体栈定稿 | `03-text-and-font-stack.md` | M1–M5 |
| 04 样式主题与 Painter 状态选择器 | `04-style-theme-and-painter-selector.md` | M1–M6 |
| 05 UI 资产管理收束 | `05-ui-asset-management.md` | M1–M5 |
| 06 MUI 式组件库落地 | `06-component-library-mui.md` | M1–M5 |
| 07 UI 动画与 theatre 式时间轴 | `07-ui-animation-theatre.md` | M1–M4 |
| 08 Workbench Shell 切到 Runtime UI | `08-workbench-shell-on-runtime-ui.md` | M1–M6 |
| 09 编辑器模块与设计图对齐 | `09-editor-modules-and-design-parity.md` | M1–M5 |

阶段划分（与「先等 runtime 大模块完成」的 gating 对应）：

- **阶段 A（runtime UI 内核）**：01 + 02 + 03。全部在 `zircon_runtime` 内完成，不动 editor 结构。
- **阶段 B（样式与资产）**：04 + 05。05 与 runtime 资产管理大模块联动；材质管理按 `docs/plans/zircon_runtime/render/08-material-shader-permutation.md` 推进，本计划只消费其资产接口。
- **阶段 C（组件库）**：06 + 07。
- **阶段 D（编辑器落地）**：08 → 09。**阶段 A/B 完成是阶段 D 的硬性 gate**：editor 在那之前继续使用现有 retained host 路径，不做半吊子切换。

里程碑级依赖见各子计划「里程碑级依赖表」节；跨计划汇总见 §7。

## 5. 全局边界约束（各子计划必须遵守）

1. 共享 UI 契约只进 `zircon_runtime_interface::ui`；runtime-only 行为（布局 pass、分发、提取、文本引擎、模板编译、surface 树变更）留在 `zircon_runtime::ui`。
2. `zircon_editor` 不引入 Slint、不引入 raw `wgpu`；editor UI 渲染继续走 GPU command stream。
3. Taffy 是 Flex/Grid/Block/Wrap 的权威布局；任何 fallback 必须记录 reason，不允许静默退回。
4. 事件路由不允许 host 按控件名称特判；热路径走编译后 route id，不解析 nativeBinding 字符串。
5. 组件视觉状态只由样式选择器决定，组件逻辑只产出语义状态。
6. 不新建平行 UI 系统：组件来源是现有 `.zui` 资产与 component catalog；`.zui` 只允许单组件 profile。
7. 硬切换：新 owner 路径落地的同一变更内迁移调用方并删除旧路径，不留兼容 re-export。
8. 根部 wiring 文件（`lib.rs`/`mod.rs`）保持薄；深行为进 owner 模块。
9. 视觉验收以「结构正确、组件统一、主要控件可交互」为准；逐像素差异修正只是后期 polish，不是设计回路。
10. 设计图权威：壳与配色以 `ai-workbench-web-framework.png` 为准（近黑表面 `#111416`–`#252b31`、teal `#3cc7d6` 仅用于激活/选中/焦点）；布局结构以 `editor-workbench-designs` 的 layout-spec / state-spec / content-spec PNG 为准；交互结构以 `component-prototype` 的组件族与路由契约为准。

## 6. 全局验收与测试基线

按 milestone-first 政策：实现切片期间只做轻量 check，里程碑末进入测试阶段。

- 切片期：`cargo check -p zircon_runtime --lib --locked`、`cargo check -p zircon_editor --lib --locked`
- 里程碑测试：`cargo test -p zircon_runtime --lib --locked`（按子计划过滤词收窄）、`cargo test -p zircon_editor --lib --locked`、`cargo test -p zircon_runtime_interface --locked`（凡动 interface）
- 组件契约：`zui_asset_governance` 测试 + component-prototype 的 `verify-native-component-contract.mjs` 等握手脚本
- 集成：`cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked`
- 实机验收：`cargo run -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor`
- 构建纪律沿 CLAUDE.md：共享 `CARGO_TARGET_DIR`、不并行重型构建、优先包级命令。

## 7. 全局里程碑依赖格与执行波次

节点记法「计划.里程碑」（如 01.M3）。关键跨计划边（完整前置见各子计划依赖表）：

| 边（前置 → 目标） | 含义 |
|-------------------|------|
| 01.M3 → 04.M2 / 06.M1 / 08.M1 | reply 统一后才有状态唯一生产者与组件行为 |
| 01.M5 → 08.M2 | 区域切换以对应 pointer bridge 收编为前提 |
| 02.M1 → 03.M3 / 04.M3 / 06.M1 / 07.M1 | 统一布局属性集是测量回调、样式词汇、组件与动画布局通道的共同底座 |
| 02.M3 → 06.M2；02.M4 → 08.M2；02.M5 → 09.M4 | 虚拟化→树表；docking 接缝→drawer 迁移；debug packet→调试器面板 |
| 03.M4 → 06.M1；03.M5 → 09.M1 | 编辑链→TextField；富文本→Console |
| 04.M1 → 05.M1；04.M4 → 06.M1 / 08.M1；04.M6 → 09.M4 | theme 类型→资产注册；Button selector→组件 DoD；热重载→token 预览 |
| 05.M2 → 04.M6 / 07.M3 / 08.M1；05.M3 → 06.M1 / 09.M2；05.M4 → 06.M1 | 热重载链、资产数据面、图标通道 |
| 06.M2 → 09.M1；06.M3 → 08.M4 / 08.M6；06.M4 → 08.M2 | 树表→模块；palette/toast→壳功能；L4 组合→区域切换 |
| 07.M3 + 08.M2 → 07.M4 | 时间轴面板承载 |
| 08.M3 → 09.M1（连同 08.M4）；08.M5 → 09.M4 | 全壳承载与命令入口是模块接线门槛；浮窗承载批次 3 |

推荐执行波次（同波内可并行；W1–W4 ≈ 阶段 A+B 主体，W5–W9 ≈ 阶段 B 尾+C，W10–W12 ≈ 阶段 D 壳，W13–W16 ≈ 阶段 D 模块）：

| 波次 | 里程碑集合 |
|------|-----------|
| W1 | 01.M1、02.M1、03.M1、04.M1（四个无前置起点） |
| W2 | 01.M2、02.M2、02.M3、02.M4、03.M2、03.M3、05.M1、05.M5、07.M1 |
| W3 | 01.M3、01.M4、02.M5、03.M5、05.M2、05.M3 |
| W4 | 01.M5、03.M4、04.M2、04.M6、05.M4、07.M3 |
| W5 | 04.M3 |
| W6 | 04.M4、07.M2 |
| W7 | 04.M5、06.M1 |
| W8 | 06.M2、06.M3、08.M1 |
| W9 | 06.M4 |
| W10 | 08.M2、06.M5 |
| W11 | 08.M3 |
| W12 | 08.M4、08.M5、08.M6、07.M4 |
| W13 | 09.M1 |
| W14 | 09.M2、09.M4 |
| W15 | 09.M3 |
| W16 | 09.M5 |

波次是并行度建议，不是合同：一波内某项滞后不阻塞同波其他项，但**跨波依赖边不可违反**。

## 8. 可用编辑器阶段定义（E0–E3）

| 级别 | 含义 | 达成里程碑（含其前置闭包） | 实机演示 |
|------|------|---------------------------|---------|
| **E0 壳可交互** | editor 全壳由 runtime UI 承载，旧 painter 路径删除 | 08.M3（≈ W11 完成） | 启动 editor：tabs 切换、drawer 开合改宽、activity rail、status bar、菜单快捷入口全部可点可聚焦 |
| **E1 最小可用编辑器** | Unity 式场景编辑回路 + 资产环 | 09.M1 + 09.M2 + 08.M4（≈ W14） | 新建工程→放置对象→树选中→Inspector 改 Transform→viewport 更新→Console 日志→Ctrl+Z 撤销；导入资产→浏览→双击打开→保存 |
| **E2 资产编辑器可用** | Material + UI Asset 两编辑器真实可用 | 09.M3（+07.M4 时间轴，≈ W15） | 改材质参数→viewport 反映；UI 资产编辑→热重载预览；时间轴加删 keyframe→即时回放 |
| **E3 完整编辑器** | 工具诊断面板真实数据 + 浮窗/通知/布局恢复 + 结构对齐审查收敛 + 可分发 | 09.M4 + 09.M5 + 08.M5/M6 + 05.M5（≈ W16） | 浮窗拖出合回、重启恢复布局、toast/通知中心、诊断/性能/构建面板；逐模块对照设计图差异清单收敛；`python tools/zircon_build.py` 产物可运行 |

「工程目标 = 能够做出可用的完整 editor」即 E3；E1 是第一个对外可演示的「真编辑器」节点，优先保证其关键路径（W1→W4→W7→W8→W11→W13→W14）。

## 9. 执行手册（贡献者工作流）

**认领一个切片**：
1. 读本 index 的 §7 确认该切片所属里程碑的前置波次已完成；读对应子计划的「设计 + 接口草案 + 切片行」。
2. 实施期：`cargo check -p <pkg> --lib --locked`（切片行标注的包）；遵守 §5 全局约束。
3. 切片完成：跑切片行的验证命令；含「硬切换」列的，过下方 checklist。
4. 里程碑末（最后一个切片合入）：跑子计划「完成定义」验收命令组；更新 `docs/zircon_runtime/ui/**` 或 `docs/zircon_editor/ui/**` 模块文档。

**硬切换 checklist（每个含删除义务的切片必过）**：
- [ ] 旧路径文件/代码段已物理删除（不是注释/feature 门）
- [ ] 全部调用方已迁移到新 owner 路径（grep 旧符号零命中）
- [ ] 无兼容 re-export / facade 桥残留
- [ ] 同一区域/职责无双路径并存
- [ ] 删除清单写进提交说明

**计划状态标记约定**：子计划 frontmatter `status: planned | in-progress | done`；里程碑完成在切片表行首加 ✅；index 不复制进度，以子计划为准。

**跨计划接口变更**：interface 层 DTO 由「拥有该 DTO 的计划」负责（01=dispatch/window、02=layout、04=style/theme、09=runtime 事件词汇）；变更集中一次（01 M1.S4、09 M1.S1 模式），必跑 `cargo test -p zircon_runtime_interface --locked`；消费方在同波次或下一波内跟进，不留长期适配层。

**每波收口**：波内全部里程碑完成后，跑一次 `cargo test --workspace --locked` 与实机冒烟，再开下一波（防止跨波回归累积）。
