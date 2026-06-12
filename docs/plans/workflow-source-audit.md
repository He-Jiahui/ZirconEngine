# Workflow 来源审计与有效信息整理

审计源目录：`C:\Users\HeJiahui\.claude\projects\E--Git-ZirconEngine`

目标目录：`E:\Git\ZirconEngine\docs\plans`

审计时间：2026-06-12

## 采纳原则

本文件只整理 workflow 对计划目录有价值的信息，不把聊天转录和失败日志当成计划正文。

采纳为有效信息：

- workflow 的名称、目标、阶段和目标计划目录。
- workflow 脚本里明确列出的子计划、focus、硬约束和执行顺序。
- workflow 结果中成功的子计划摘要。
- 当前已存在于 `docs/plans` 的计划文件、行数、明显细化标记。
- memory 中明确指定的权威计划入口。

不直接采纳为事实：

- API 502、socket closed、stall 等运行噪声。
- verify 阶段没有完成的 draft 内容。
- 子代理探索报告中的行号和标识符，除非后续执行前重新按 live worktree 核验。
- 失败 workflow 声称的整体 completed 状态；以子代理实际 done/error 和落盘文档为准。

## Workflow 总表

| 来源 | workflow | 目标目录 | 阶段 | 运行状态 | 可采纳信息 | 未采纳/需重做 |
| --- | --- | --- | --- | --- | --- | --- |
| `f9e1e9da-de33-4c6a-b5f2-9167fe5fe281/workflows/wf_a76fbb0c-aac.json` | `refine-runtime-plans` | `docs/plans/zircon_runtime/runtime` | Gather -> Draft -> Verify | 顶层显示 completed，但 `result.failed = 01-07`；12 个代理 done，7 个代理 error。 | 01-07 gather 全部完成；draft:01/03/04/05/07 完成；`01` 当前已落盘为 250 行并带 `last_refined: 2026-06-12`。 | `02/06` draft 失败；`03/04/05/07` draft 未经 verify；`01` verify 代理在 Write 后 502，仍需人工抽查。 |
| `88419d5a-2022-4d2f-ac12-a352258aa773/workflows/wf_234cfcdf-454.json` | `deepen-hub-plans` | `docs/plans/zircon_hub` | Deepen -> Verify -> Fix -> Index | 顶层显示 completed；10 个代理中 2 个 done、8 个 error。 | `01` 与 `06` deepening 成功并写入；当前文档分别约 815 行和 604 行。 | `02-05/07` deepen 失败或中断；`verify:01/06` 与 `index-update` 失败；Hub index 未同步工程化增量。 |
| `a28862b2-9cc7-44f8-a626-868f46c3b8be/workflows/wf_39de3956-0fa.json` | `deepen-plugin-plans` | `docs/plans/zircon_plugins` | Core -> Plugins -> Audit -> Fix | failed；Core 阶段 01 代理 502，未进入下游并行深化。 | 脚本完整保留 01 core 的 API 定稿要求，以及 02-10 的功能覆盖 focus。 | 没有可信的工程化重写结果；现有插件文档按基线计划看待。 |
| `718a9667-6a03-4733-ba48-327248518ca7/workflows/scripts/editor-ui-plan-engineering-deepening-wf_cbcbc1a2-cd3.js` | `editor-ui-plan-engineering-deepening` | `docs/plans/zircon_editor/editor_ui` | Explore -> Refine -> Verify -> Repair -> Index | 未发现对应顶层 workflow result JSON；仅有脚本与子代理日志。 | 脚本给出 10 条全局 UI 约束、9 个领域、工程化文档结构要求。 | 不能证明 01-09 已完成工程化重写；当前文档按基线计划看待。 |
| `memory/render-alignment-plan-set.md` | render plan memory | `docs/plans/zircon_runtime/render` | memory | 已存在 memory 入口。 | 指定 `docs/plans/zircon_runtime/render/index.md + 01-16` 为渲染管线 UE/Unity 对齐权威计划集。 | 该 memory 不是 workflow result；需要以后续实现时按源码重核。 |

## 已落盘计划集状态

### Runtime 渲染管线

当前状态：完整计划集已落盘。

入口：

- `docs/plans/zircon_runtime/render/index.md`

子计划：

- `01-render-graph-rdg-alignment.md`
- `02-mesh-draw-command-pipeline.md`
- `03-gpu-scene-gpu-driven.md`
- `04-visibility-culling.md`
- `05-lighting-shadows.md`
- `06-temporal-pipeline.md`
- `07-postprocess-color-pipeline.md`
- `08-material-shader-permutation.md`
- `09-camera-render-ordering.md`
- `10-renderer-family.md`
- `11-environment-lighting.md`
- `12-effects-particles.md`
- `13-texture-pipeline.md`
- `14-2d-stack.md`
- `15-terrain-vegetation.md`
- `16-compute-neural.md`

有效信息：

- 骨架层 01-08 是后续渲染能力的前置，不应在 runtime 其他计划里展开重复设计。
- 能力层 09-16 承接相机、renderer family、环境光、粒子、纹理、2D、地形植被和 compute/NN。
- memory 明确要求渲染任务先读 render index。

### Runtime 架构收束

入口：

- `docs/plans/zircon_runtime/runtime/index.md`

当前文件状态：

| 文件 | 当前状态 | workflow 有效信息 |
| --- | --- | --- |
| `01-tech-stack-and-dependency-governance.md` | 已落盘工程化稿，带 `last_refined: 2026-06-12`、执行前检查清单、状态与产出记录。 | gather 和 draft 完成；verify 代理在 Write 后 502，执行前仍需抽查路径、测试名和命令。 |
| `02-core-spine-and-root-surface.md` | 基线计划。 | gather 完成；draft 失败。focus 是 core 根条目、散件调用方、foundation 重叠、lib.rs alias、generated 标记、root-surface 文档。 |
| `03-schedule-and-frame-loop-alignment.md` | 基线计划。 | gather 和 draft 完成；verify 失败。focus 是 SystemStage、Schedule/runner/executor/conflict graph、frame loop 顺序、FrameClock、builtin stage、Time、schedule tests。 |
| `04-asset-pipeline-alignment.md` | 基线计划。 | gather 和 draft 完成；verify 失败。focus 是 asset handle、加载状态、worker_pool、watch debounce、facade events、manager contracts、asset tests、config_store 使用。 |
| `05-scene-editor-boundary-closeout.md` | 基线计划。 | gather 和 draft 完成；verify 失败。focus 是 editor_projection、serialization_guard、inspection snapshot、editor 命中、legacy 命中、命名审计文档。 |
| `06-plugin-surface-and-lifecycle.md` | 基线计划。 | gather 完成；draft 断连。focus 是 plugin/mod.rs 公开面、NativePlugin 调用方、ABI 常量、ZrVM 空指针现场、hot reload、script/vm 模块、feature 与测试。 |
| `07-runtime-performance-hotpath.md` | 基线计划。 | gather 和 draft 完成；verify 失败。focus 是 diagnostics API、计数断言范例、RenderFrameExtract、tracing span、vampire FPS 测试、profiling profile/feature、ECS query cache。 |

建议恢复顺序：

1. 抽查并确认 01 当前落盘稿是否仍与 live worktree 一致。
2. 重新生成 02 和 06。
3. 从 subagent `StructuredOutput` 恢复 03/04/05/07 草稿，但必须先完成路径、标识符、测试名、命令核验，再覆盖子计划。
4. 更新 runtime index，标注各计划细化状态和跨计划硬约束。

### Zircon Hub

入口：

- `docs/plans/zircon_hub/index.md`

当前文件状态：

| 文件 | 当前状态 | workflow 有效信息 |
| --- | --- | --- |
| `01-action-dispatch-and-typed-payload.md` | 已落盘工程化稿，约 815 行。 | HubActionId 单一来源、payload DTO 与统一校验、前后端契约守卫已深挖。verify 代理失败，需执行前抽查。 |
| `02-background-task-framework-and-persistence.md` | 基线计划。 | focus 是 BackgroundTask trait、泛型执行器、HubConfig 原子持久化、persist 单点化、交付失败清理、TOCTOU、worker catch_unwind。 |
| `03-project-lifecycle-robustness.md` | 基线计划。 | focus 是 create/import 失败语义、路径规范化、picker 取消、visual fixture 退出生产、回收站删除转义单测。 |
| `04-settings-draft-and-source-engine.md` | 基线计划。 | focus 是 SettingsFieldSpec、draft discard/restore、folder picker Ok(None)、Source Engine 深校验与预检。 |
| `05-frontend-componentization-and-type-safety.md` | 基线计划。 | focus 是 HubWindow 路由表、页面组件拆分、ErrorBoundary、assertHubShellState、mock 收缩、双重可空清理。 |
| `06-layout-and-visual-standard.md` | 已落盘工程化稿，约 604 行。 | 断点/溢出、token 一元化、参考图细节、截图矩阵已深挖。verify 代理失败，需执行前抽查。 |
| `07-localization-schema-and-coming-soon.md` | 基线计划。 | focus 是 HubMessageId/HubMessage、action_history 语言跟随、coming-soon 目录、ui_text 覆盖审计。deepen 代理在 Edit 后断连，当前正文不能按完成看待。 |

Hub index 缺口：

- workflow 计划本应增补跨计划共享类型/模块所有权地图：`HubActionId`、payload 校验入口、`BackgroundTask`、persist 单点、`SettingsFieldSpec`、`HubMessageId`/`HubMessage`、`assertHubShellState`、coming-soon DTO。
- workflow 计划本应增补切片级执行清单：按阶段 A -> D 展开 `01.M1.1` 形式并标注可并行项。
- `index-update` 代理失败，所以这些内容尚未可信落盘。

### Zircon 插件生态

入口：

- `docs/plans/zircon_plugins/index.md`

当前文件状态：

- `01-plugin-architecture-core.md` 到 `10-editor-integration.md` 均存在，但按 baseline 计划看待。
- workflow 的工程化深化停在 01 core；没有下游 02-10 的成功结果。

从脚本保留的有效 focus：

| 文件 | 继续细化时必须覆盖 |
| --- | --- |
| `01-plugin-architecture-core.md` | Runtime Plugin Interface v2；调度标签；`SystemRegistration`；`TypedExtensionPoint`；`RuntimePlugin` register/finish/activate/deactivate；capability gate；`plugin.toml` 单源 schema；Native ABI v3 分域宿主函数表。 |
| `02-sound.md` | CompiledMixGraph 双缓冲；音频线程零分配零锁；DspEffect 标准效果；3D SpatializerStack；ChannelLayout；timeline automation；CPAL 输出线程边界。 |
| `03-physics.md` | PhysicsBackend；builtin/jolt 裁决；形状全集；CollisionLayer/Mask；trigger；约束族；RagdollProfile；raycast/shapecast/overlap；scene hook 调度锚点。 |
| `04-animation.md` | ParameterApply -> StateMachineStep -> GraphEvaluate -> PoseBlend -> PoseApply；PoseBuffer SoA；avatar mask；状态机；GPU skinning；IK；timeline/tracks/clips。 |
| `05-navigation.md` | bake pipeline；`.znavmesh` 格式；recast/detour 选型；NavWorld；TileCache obstacle；OffMeshTraverseState；NavMeshModifier/Surface；无 physics 退化路径。 |
| `06-ai.md` | `.btree.toml`；行为树节点库；observer aborts；typed blackboard；perception 分帧预算；MoveTo 与 ScriptTask 接口。 |
| `07-net.md` | `service_types.rs` 拆分；NetWorker 线程模型；Transport trait；TLS/rustls；HTTP/WebSocket；Session/RPC；ReplicationSchema；可靠 UDP；content download DTO。 |
| `08-zr-vm.md` | TypeDescriptor/FieldAccessor/MethodAccessor；反射模型；dense call site；VmCallbackHandle；四个扩展点通道；GC 协约；VmStateBlob 迁移。 |
| `09-export-publishing.md` | `[export_profiles.<name>]` schema；ExportBuildPlan；SourceTemplate/LibraryEmbed/NativeDynamic；template 包；zrpack；资产闭包裁剪；deterministic build；`zircon export` 阶段机。 |
| `10-editor-integration.md` | editor 扩展点签名级约定；反射驱动 default drawer；EditorOperationStack；viewport overlay gizmos；DiagnosticPath；play-in-editor 状态镜像；AI Workbench 风格对位。 |

硬约束：

- 先定稿 01，再写 02-10。
- 下游文档必须逐字照用 01 的调度锚点、注册 API、trait 方法名、capability 类型、plugin.toml 字段和 ABI 函数表名。
- 每份工程化稿必须包含模块文件树、Rust 签名代码块、里程碑任务表和测试函数名。

### Editor UI

入口：

- `docs/plans/zircon_editor/editor_ui/index.md`

当前文件状态：

- `index.md + 01-09` 均存在，但都是较短基线计划。
- workflow 没有顶层 result JSON；不能确认 Explore/Refine/Verify/Repair/Index 已完成。

脚本保留的全局约束：

1. 共享 UI 契约只进 `zircon_runtime_interface::ui`；runtime-only 行为留在 `zircon_runtime::ui`。
2. `zircon_editor` 不引入 Slint、不引入 raw wgpu；editor UI 渲染继续走 GPU command stream。
3. Taffy 是 Flex/Grid/Block/Wrap 的权威布局；fallback 必须记录 reason。
4. 事件路由不允许按控件名称特判；热路径走编译后 route id。
5. 组件视觉状态只由样式选择器决定，组件逻辑只产出语义状态。
6. 不新建平行 UI 系统；组件来源是现有 `.zui` 资产与 component catalog。
7. 硬切换：新 owner 路径落地的同一变更内迁移调用方并删除旧路径。
8. 根部 wiring 文件保持薄；深行为进 owner 模块。
9. 视觉验收以结构正确、组件统一、主要控件可交互为准，不逐像素。
10. 壳与配色以 `ai-workbench-web-framework.png` 为准；布局结构以 editor-workbench-designs 的 spec PNG 为准；交互结构以 component-prototype 为准。

脚本列出的 9 个计划领域：

| 文件 | 主题 |
| --- | --- |
| `01-slate-input-dispatch-core.md` | 输入与事件内核 |
| `02-layout-taffy-and-containers.md` | 布局系统：Taffy 权威与特殊容器 |
| `03-text-and-font-stack.md` | 文本与字体栈 |
| `04-style-theme-and-painter-selector.md` | 样式主题与 Painter 状态选择器 |
| `05-ui-asset-management.md` | UI 资产管理收束 |
| `06-component-library-mui.md` | MUI 式组件库落地 |
| `07-ui-animation-theatre.md` | UI 动画与 theatre 式时间轴 |
| `08-workbench-shell-on-runtime-ui.md` | Workbench Shell 切到 Runtime UI |
| `09-editor-modules-and-design-parity.md` | 编辑器模块与设计图对齐 |

工程化补齐模板：

- 接口与数据结构草案。
- 模块与文件落点。
- 管线时序。
- 里程碑切片化。
- 测试矩阵。
- 风险与对策。
- 里程碑级依赖表。
- 完成定义。

## 后续执行清单

- [ ] Hub：补齐 `zircon_hub/index.md` 的跨计划所有权地图和切片级执行清单。
- [ ] Hub：重跑或人工细化 `02-05/07`，保持 01/06 已落盘内容。
- [ ] Runtime：抽查 `01` 的路径、标识符、测试名和命令，确认 Write 后 502 没有留下半成品。
- [ ] Runtime：重新生成 `02`、`06`。
- [ ] Runtime：恢复并核验 `03/04/05/07` draft 后再落盘。
- [ ] Plugins：单独完成 `01-plugin-architecture-core.md` 工程化定稿。
- [ ] Plugins：基于 01 的定稿 API 更新 `02-10`。
- [ ] Editor UI：按脚本模板补齐 `01-09` 的工程化章节，再更新 editor_ui index。
- [ ] 所有计划集：完成后在各 index 标注细化状态、验证口径和执行顺序。

