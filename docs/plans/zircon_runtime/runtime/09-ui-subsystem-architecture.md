---
related_code:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/mod.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/compute.rs
  - zircon_runtime/src/ui/layout/pass
  - zircon_runtime/src/ui/layout/pass/pipeline.rs
  - zircon_runtime/src/ui/layout/pass/layout_tree.rs
  - zircon_runtime/src/ui/layout/pass/incremental.rs
  - zircon_runtime/src/ui/layout/pass/responsive_mui.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime/src/ui/layout/style_mapping.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/surface/input
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/route_authority.rs
  - zircon_runtime/src/ui/surface/pointer
  - zircon_runtime/src/ui/surface/navigation
  - zircon_runtime/src/ui/surface/render/collection_rows/table.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs
  - zircon_runtime/src/ui/surface/surface/property_transaction.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events/state_invalidation.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events/template_action.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/tests/widget_menu_behavior.rs
  - zircon_runtime/src/ui/tests/widget_menu_behavior/control_anchored_overlays.rs
  - tools/tests/test_runtime_widget_menu_behavior_test_structure.py
  - tools/tests/test_runtime_ui_surface_incremental_rebuild_owner_structure.py
  - tools/tests/test_runtime_ui_surface_property_transaction_owner_structure.py
  - tools/tests/test_runtime_ui_pointer_component_state_owner_structure.py
  - tools/tests/test_runtime_ui_pointer_template_action_owner_structure.py
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/surface/ecs_projection.rs
  - zircon_runtime/src/ui/template
  - zircon_runtime/src/ui/template/asset/surface_index.rs
  - zircon_runtime/src/ui/template/asset/surface_index/node_resource_registration.rs
  - zircon_runtime/src/ui/template/build/interaction.rs
  - tools/tests/test_runtime_ui_asset_surface_node_resource_owner_structure.py
  - zircon_runtime/src/ui/platform_input/winit_translation.rs
  - zircon_runtime/src/ui/platform_input/winit_translation/keyboard.rs
  - zircon_runtime/src/ui/platform_input/winit_translation/pointer.rs
  - zircon_runtime/src/ui/platform_input/winit_translation/ime.rs
  - zircon_runtime/src/ui/platform_input/winit_translation/window.rs
  - tools/tests/test_runtime_ui_winit_translation_owner_structure.py
  - zircon_runtime/src/ui/v2
  - zircon_runtime/src/ui/dispatch
  - zircon_runtime_interface/src/ui
  - zircon_runtime/src/ui/surface/input/navigation.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_markdown.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/ui-and-layout/shared-ui-template-runtime.md
status: in_progress
last_refined: 2026-07-23
---

# 09 UI 子系统架构收束

Current-source UI architecture mirror 2026-08-14: `ui_architecture_boundary` reports `expected_source_file_count = 52`, `expected_ui_entry_count = 20`, `expected_surface_entry_count = 26`, `legacy_full_hits = 70`, `expected_legacy_full_hits = 70`, `legacy_production_hits = 0`, `expected_legacy_production_hits = 0`, `legacy_production_file_count = 0`, `expected_legacy_production_file_count = 0`, `taffy_production_hits = 175`, `expected_taffy_production_hits = 175`, `taffy_production_file_count = 10`, `expected_taffy_production_file_count = 10`, `runtime_v2_anchor_count = 10`, `interface_v2_anchor_count = 9`, `guard_anchor_count = 19`, `cargo_gate_anchor_count = 7`, `doc_anchor_count = 61`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This current snapshot supersedes older dated counts without rewriting their history.

承接两条上游线：(a) 子计划 05 的 legacy debt bucket 中 **UI input/render、UI template/layout、input 三桶**的 owner 落点即本计划；(b) 文本栈职责归 01-M2（本计划不重复）。渲染提交路径（glyphon GPU 提交、ui surface render 的 wgpu 侧）归 render 计划与 rhi_wgpu owner。

## 现状与证据（2026-06-12 实仓盘点）

- **模块树**（`ui/` 实测 17 条目）：`layout/`（constraints、pass/、scroll、style_mapping、taffy_bridge、virtualization）、`surface/`（2026-06-13 current scan 为 20 条目；2026-06-12 旧盘点为 21 条目：input/、pointer/、navigation/、render/、ecs_projection、focus、frame_hit_test、interaction_gate、popup_stack、node_pool、property_mutation、reflection_snapshot、slots、timeline、diagnostics、arranged、component_state、surface(.rs+/)）、`template/`（asset/build/instance/loader/validate）、`dispatch/`、`binding/`、`component/`、`tree/`、`text/`（归 01-M2）、`theme/`、`event_ui/`、`accessibility/`、`runtime_ui/`、`v2/`、`module.rs`、`style.rs`。
- **双代并存**：`ui/v2/` 与非 v2 路径并存；`zircon_runtime_interface/src/ui/` 同样有 `v2/`——v2 的定位（替代中 / 实验 / 并行契约）未在计划层文档化，是首要裁决项。
- **镜像契约面**：`zircon_runtime_interface/src/ui/` 有 22 条目（accessibility/binding/component/dispatch/ecs/event_ui/focus/layout/navigation/picking/pipeline/skin/style/surface/template/text/tree/v2/widget/window…），与 runtime `ui/` 高度同构——哪些是共享 DTO（合法镜像）、哪些是重复定义（漂移风险）未盘点；镜像同步规则归子计划 10 的契约守卫，本计划负责 runtime 侧形状。
- **legacy 集中区实证**（05 计划继承）：`surface/input/navigation.rs:22-54` 的 `legacy` 路由回复变量承载真实运行语义（route/focus/diagnostics 取值源）；`runtime_naming_boundary` 审计的 legacy debt bucket 中 runtime UI input/render、UI template/layout、input 三桶归本计划裁决处置。
- **既有测试基建**：`ui/tests/`（注意：`ui/tests/runtime_input_manager.rs`、`ui/tests/style_mapping.rs` 在 05 执行期曾出现编译错误，归活动 editor UI 会话——本计划动工前必须确认其已修复）。
- 职责声明锚（CLAUDE.md）：共享 UI 契约类型住 `zircon_runtime_interface::ui`；layout pass、dispatch、render extraction、text/layout 引擎、template 编译、surface/tree 突变住 `zircon_runtime::ui`——本计划以此为合法性判据。
- 参考锚点（每点一行）：bevy_ui taffy 集成 — `dev/bevy/crates/bevy_ui/src/layout/`；Fyrox retained UI（widget 树 + 消息路由）— `dev/Fyrox/fyrox-ui/src`；UE Slate/UMG 分层（不在 dev/ 则仅作概念对照）。

补充参考锚点（2026-06-13 实测核验，实现型切片动工前先读——index 公约 §7.9）：

- Fyrox retained 控件树 + 消息路由的完整 Rust 实现（M1 路由单点对照首选）— `dev/Fyrox/fyrox-ui/src/{control.rs,canvas.rs,button.rs}` 及同目录控件族
- Godot Control/Container 布局族 C++ 实现（M2 布局/容器语义第二对照）— `dev/godot/scene/gui/`
- bevy_ui 的 taffy 桥接实现细节（M2.1 单后端入口对照）— `dev/bevy/crates/bevy_ui/src/layout/{mod.rs,ui_surface.rs}`

## 目标

1. v2 与非 v2 双代定位裁决：替代关系给迁移路线图（带删除条件），并行契约给分工判词——消除"莫名双代"。
2. 输入路由收束：pointer/navigation/input 三族的 legacy 路径逐文件处置（改名/文档化/移交），dispatch 单点权威化。
3. layout 管线权威化：taffy_bridge 为唯一布局后端入口、pass 序显式、virtualization 与 scroll 的边界声明。
4. template 编译/实例化边界定稿，与 02-M4 generated 规则衔接（模板产物若属生成物必须带标记）。

## 与 editor_layout / editor_ui / render 的关系(职责链单源)

本子系统(`zircon_runtime/src/ui/**`)是 UI 职责链的**引擎实现层**,位置:

```
editor_layout/(规范/契约,DTO 落 zircon_runtime_interface) → editor_ui/(运行时能力) → 本子系统(引擎 UI 实现) → render(rhi_wgpu 上屏)
```

- **本子系统实现谁的契约**:`editor_layout/` 定义的契约在此落地——`13`/`02` 约束求解 → `ui/layout/{taffy_bridge,style_mapping,pass}`;`18` 输入响应/命中 + `19` 焦点导航 → `ui/surface/{input,pointer,navigation,focus,frame_hit_test}`(本计划目标 2 的 dispatch 单点权威正是 `18` 的"路由次序单源 + 命中单源");`20` USS 级联样式 → `ui/v2/style.rs`(伪状态/选择器/computed,本计划与 `editor_ui/04` 协同);`17` 文本测量=绘制 → `ui/text`;`21` 提交契约的提取侧 → `ui/surface/render`。
- **契约单源**:对应 DTO 住 `zircon_runtime_interface::ui`(本计划只负责 runtime 侧形状,契约同步守卫归子计划 10);**不在本子系统另立设计语言/约束语义**,语义以 `editor_layout/NN` 为准。
- **下游**:GPU 提交/批次/裁剪/图集上屏归 `render`(见 `docs/plans/zircon_runtime/render/14` 2D/UI stack + `editor_layout/21` 提交契约);本子系统只产出 render extract,不直接发 wgpu。
- **勾稽**:`editor_layout/index §6.1` ↔ `editor_ui/index §3.1` ↔ 本节 ↔ `render/14`。

## 非目标

- 文本栈（shaper/SDF/glyphon 职责）归 01-M2；GPU 提交与 render extract 消费侧归 render 计划。
- `zircon_runtime_interface::ui` 的契约同步守卫归 10；editor 侧 retained host/workbench 归 editor 计划。
- 不做视觉/交互行为重设计——只收结构与语义边界。

### 全局硬约束（继承总计划 §4，违反即返工）

- 不新增 crate；硬切换不留兼容层；动态边界只传 ABI-safe 值；非网络语义 server 命名是 blocker；generated 产物只许 leaf DTO/table。

## 执行前检查清单

1. **强制前置**：editor UI 活动会话（曾致 `ui/tests/runtime_input_manager.rs`、`ui/tests/style_mapping.rs` 编译错误）状态确认——`git status --porcelain -- zircon_runtime/src/ui/`，脏区避让；`cargo check -p zircon_runtime --lib --locked` 必须先绿。
2. 05 移交确认：legacy debt bucket 的 UI 三桶清单（`runtime_naming_boundary` 审计 JSON）取到手，作为 M2 工作集输入。
3. 事实重核：
   - `ls zircon_runtime/src/ui/ zircon_runtime/src/ui/v2/ zircon_runtime_interface/src/ui/v2/`
   - `grep -rn "legacy" zircon_runtime/src/ui --include=*.rs | wc -l`（UI 区 legacy 基线）
   - `grep -rln "taffy" zircon_runtime/src/ui --include=*.rs`（布局后端入口面）
4. 基线记录：`cargo test -p zircon_runtime --lib ui --locked` 通过数记入状态节。

## 里程碑

### M0 模块边界图与双代裁决（先证据后动刀）

#### 切片 0.1 UI 模块边界图

- 目标文件：`docs/zircon_runtime/ui/`（执行时核验既有镜像文档：`ls docs/zircon_runtime/ui/`；架构图落 `architecture.md`，有则扩展）。
- 改动形态：纯文档。画出 17 模块的依赖方向图（layout ← surface ← dispatch；template → tree/component；binding/event_ui 的挂接位），每条边标注 owner 文件；对照 CLAUDE.md 职责声明标出越界边（若有）。
- 验收：图覆盖全部 17 条目；越界边清单（可为空）。
- DoD：`architecture.md` 落地，越界清单进 M2/M3 工作集。

#### 切片 0.2 v2 双代定位裁决

- 目标文件：同 0.1 文档 + 本计划状态节。
- 改动形态：盘点 `ui/v2/` 与 `interface/ui/v2/` 的内容物、调用方（Grep `ui::v2`，path `zircon_runtime/src zircon_editor/src zircon_app/src`）、与非 v2 的能力重叠面；三选一判词：(a) v2 是替代代——产出逐模块迁移路线（带非 v2 删除条件，硬切换分片）；(b) v2 是并行契约（如 editor workbench 专用面）——写分工边界与命名理由；(c) v2 是死代码——删除切片。
- 调用方迁移：裁决期无；(a)/(c) 的迁移/删除作为后续独立切片入状态节排期。
- 验收：判词 + 调用方实测清单；无"暂不裁决"。
- DoD：判词落文档；若 (a)/(c)，路线图每步有 DoD。

#### M0 测试阶段（milestone-first）

- 纯审计：`git status --porcelain` 仅 docs 变更。

### M1 输入路由收束（承接 05 debt bucket）

#### 切片 1.1 路由单点权威化

- 目标文件：`ui/surface/input/`、`ui/surface/pointer/`、`ui/surface/navigation/`、`ui/dispatch/`（路由汇聚点执行时核验：Grep `dispatch_navigation_event|dispatch_pointer`，path `zircon_runtime/src/ui`）。
- 改动形态：定稿"一次输入事件的权威路径"（platform/input → surface 命中（frame_hit_test）→ interaction_gate → dispatch → 组件/焦点/popup_stack），文档化 + 对越权直连（绕过 dispatch 的旁路）列违规清单逐项收束。
- 与 12-M0.2 互引：12 定义全局输入消费边界为 UI surface/capture/popup/focus 优先、玩法/action mapping 只消费 UI 未处理或无 UI/headless 输入；本切片只收束 UI 内部 `interaction_gate` / dispatch 权威路径，不把全局 action mapping 逻辑并入 UI。
- 调用方迁移：按违规清单（M0 越界边 + 本切片盘点，预计 ≤10 全列）。
- 验收：`ui_input_events_route_through_single_dispatch_authority`（结构/行为测试，归属 `ui/tests/` 既有树）。
- DoD：旁路清单清零或每条带 owner 判词。

#### 切片 1.2 legacy 路由路径处置

- 目标文件：05 审计 JSON 中 UI 三桶的具体文件（代表：`surface/input/navigation.rs` 的 `legacy` 回复变量族）。
- 改动形态：逐文件三选一（05 已分类，本切片执行处置）：改名为语义词（如 `routed_reply`，行为零变化、纯重命名切片）/ 保留 + 注释语义 / 真实迁移债展开为独立切片。改名类切片一次提交闭合（硬切换）。
- 调用方迁移：改名项的引用面（Grep 旧名逐文件，预计局部 ≤5 文件）。
- 验收：处置后 `runtime_naming_boundary` 审计的 UI 桶计数下降或语义注释齐备；`cargo test -p zircon_runtime --lib naming_boundary --locked` 仍绿（白名单同步更新）。
- DoD：UI 三桶在审计输出中状态翻转（debt → resolved/documented）。

#### M1 测试阶段（milestone-first）

- `cargo check -p zircon_runtime --lib --locked`（切片期）
- `cargo test -p zircon_runtime --lib ui --locked`（全族无回归）；`cargo test -p zircon_runtime --lib input --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib naming_boundary --locked`（白名单一致性）

### M2 layout 管线权威化

**Dependencies:** M1 accepted.

#### 切片 2.1 taffy 单后端入口与 pass 序

- 目标文件：`ui/layout/taffy_bridge/{mod,compute}.rs`（唯一后端计算入口）、`ui/layout/pass/{pipeline,layout_tree,incremental,taffy_arrange}.rs`（pass 序显式化）、`ui/layout/style_mapping.rs`。
- 改动形态：盘点 taffy 直接调用面（执行前检查清单第 3 项的 Grep 结果）——除 taffy_bridge 外的直连改经 bridge（硬切换）；pass 序（measure→layout→arrange？按实仓定）写成显式序列并文档化；`style_mapping` 与 `interface/ui/style` 的映射职责单点。
- 调用方迁移：taffy 直连违规点（执行时枚举，预计 ≤5）。
- 验收：`runtime_09_taffy_layout_pass_order_uses_bridge_authority`（结构测试：Taffy tree build/compute 只在 bridge compute；full/incremental layout 消费 `UI_LAYOUT_PASS_ORDER`；`style_mapping` 只保留 DTO adapter 判词）。
- DoD：Grep `taffy::` path `zircon_runtime/src/ui` 的实际生产命中被分为 bridge compute、bridge style API、style DTO adapter、pass selection/reporting；直接 `TaffyTree::new()` / `compute_layout` 只允许在 `taffy_bridge/compute.rs`。

#### 切片 2.2 virtualization / scroll 边界声明

- 目标文件：`ui/layout/virtualization.rs`、`ui/layout/scroll.rs` + 文档。
- 改动形态：声明虚拟化窗口与滚动偏移的 owner 与失效时机（数据变更/视口变更）；补行为测试：`retained_virtual_list_only_arranges_visible_window`、`scroll_offset_invalidates_virtualization_window`、`non_virtualized_scroll_offset_keeps_full_window_dirty_domain`。首项只声明 retained-child geometry windowing，不声明实例物化已被限制。
- 调用方迁移：无公共面变化。
- 验收：两测试绿。
- DoD：边界文档 + 测试落地。

#### M2 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib layout --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib ui --locked`

### M3 template 编译与实例化边界

**Dependencies:** M2 accepted.

#### 切片 3.1 编译产物与 generated 规则衔接

- 目标文件：`ui/template/{asset,build,instance.rs,loader.rs,validate.rs}` + `docs/ui-and-layout/shared-ui-template-runtime.md`（既有，口径刷新）。
- 改动形态：定稿 build（编译）→ instance（实例化）→ validate（校验）三段边界；编译产物若落盘为生成文件，必须符合 02-M4 的 `@generated` 标记规范且只含 leaf DTO/table；validate 失败路径补测试。
- 调用方迁移：无公共面变化（边界声明 + 测试）。
- 验收：`template_validate_rejects_unknown_component_contract`、`template_instance_failure_surfaces_loader_error`（名按实仓定稿）。
- DoD：三段边界文档化；与 02-M4 守卫无冲突。

#### M3 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib template --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib ui --locked`（收尾全族）
- 验收证据：边界文档 + 失败路径测试；`shared-ui-template-runtime.md` 与代码一致。

### M4 增量 arranged tree 与 hit grid patch

**Dependencies:** M3 accepted.

- 目标：消费 layout geometry changed set，局部更新 arranged geometry 与 hit-grid 空间索引；只有结构、排序或裁剪语义变化才升级为全量重建。
- 验收：单节点 parent-directed 尺寸变化的 arranged/hit outer visits 与 changed node 数线性，未变化 draw-order entry 与 hit cell 不重建；auto parent、clip、z-order、popup、scroll、detach/attach 保持正确性。

### M5 增量文本与 render extract patch

**Dependencies:** M4 accepted.

- 目标：按 text/layout/resource generation 保留 shaped/layout buffer，局部更新受影响 render command ranges，禁止先全量 extract 再比较缓存。
- 验收：单文本变化只 shape/rebuild 目标文本和必要命令；稳定 extract 的 node visit、临时 command Vec 与 payload clone 为零。

### M6 Arc frame、Workbench 局部同步与虚拟化

**Dependencies:** M5 accepted.

- 目标：以 generation-owned `Arc` artifacts 发布 surface frame，Workbench/native window 使用 generation cursor，长列表只物化可见窗口。
- 验收：稳定 frame read 不复制 arranged/render/hit/focus/report/ECS projection；pane/native 更新只随对应 generation，长时间 retained-memory 无持续增长。

### M7 真实运行时验收与旧路径删除

**Dependencies:** M6 accepted.

- 目标：删除 event-time full surface/projection/render fallback，接入 diagnostics、GPU/softbuffer submit、bundle 构建和真实窗口采样。
- 验收：受管 build、focused contracts、profiling bundle smoke、像素对拍、600-event 样本及 hover/scroll/typing/docking 压力全部通过；独立 EXE 可携带完整资产/DLL 启动。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`09/2026-07-09-ui-subsystem-architecture-output-records.md`](09/2026-07-09-ui-subsystem-architecture-output-records.md)
- fixed 已修复：[runtime-rich-table-layout-recursion](09/fixed-2026-07-12-runtime-rich-table-layout-recursion.md)

## 性能审阅交接

- 2026-07-23 performance handoff：`zircon_runtime_interface/src/ui/pipeline/**` 6/6静态反查确认`UiSurface::surface_frame()`每次都重建固定10行pipeline report、多个dirty-reason Vec与skipped note String。Runtime09按PERF-MVP-278让rebuild发布generation-owned frame/report，稳定generation不得重建stage rows或owned notes；保留10-stage serde与archived diagnostic兼容。验收1/1k/10k nodes及input/layout/render/window单域变化的report builds、owned bytes、Arc owners与p95；current-source Cargo/F4 trace待完成。
- 2026-07-23 binding performance handoff：`ui/binding/**` 10/10反查确认`UiEventManager::invoke_binding`每event格式化完整native String查表并clone arguments/binding/result；新增PERF-MVP-572。Runtime09在route generation发布`UiRouteId`/typed handle与shared binding/default arguments，native codec只留authoring/serde/error边界并加bytes/args/nodes/depth/string硬预算。property mutation report重复property/value/message/dirty所有权并入PERF-MVP-265；验收1/100/10k routes/fields、1M events和0/1KiB/1MiB payload的format/clone/owner/transaction/p95，current-source Cargo/F4待完成。
- 2026-07-23 ECS projection handoff：`ui/{ecs.rs,ecs/**}` 2/2反查确认snapshot/delta分别重扫totals/mask/10-stage/8-domain并为各组建BTreeSet/Vec，single-stage/domain query也忽略carried derived rows重算全部，`diff_from`另建previous/current双BTreeMap。Runtime09按PERF-MVP-278发布generation-owned projection+node index+derived rows，并从authoritative changed set直接产delta；稳定generation visits/alloc=0，单delta随changed rows。1/1k/10k/100k nodes记录passes/visits/tree alloc/published bytes/p95，current-source Cargo/F4待完成。
- 2026-07-23 event/control DTO交接：`ui/event_ui/**` 4/4确认context/result/notification重复拥有binding/arguments/JSON，Tree/Node/Property response为wide owned reflection snapshot。Runtime09把route handle/shared payload归PERF-MVP-572，subscriber有界fanout归252，generation reflection artifact/delta归278/456；stable control query与invoke不得重建/深clone全量payload。验收routes/nodes/subscribers 1/100/10k、1MiB payload与1M events的owners/clone/queue/p95。
- 2026-07-23 layout contract交接：`ui/layout/**` 11/11反查确认每container engine selection现建Taffy/Zircon capability Vec，incremental report clone全部untouched selection并全量汇总；归PERF-MVP-263。Runtime09用static capability mask、generation-owned aggregate与debug-gated detail，stable report rows/heap=0；slot索引归260、persistent Taffy归261、visible-only arrange归262、compiled style共享归274/312。验收1/100/10k nodes/slots/tracks的alloc/clone/probe/upsert/p95与12条合同。
- 2026-07-23 skin contract交接：`ui/skin/**` 2/2当前只有合同测试consumer；产品接线前须把四个owned preset构造器收口到single static/theme-generation catalog，按compact token identity借用查询，禁止paint/pane逐次重建31组token String/Vec。归PERF-MVP-264并联动EditorUI04；验收1M token lookups stable generation constructor/clone=0、lookup近O(1)，reload一次原子发布且保留serde/token值。
- 2026-07-23 window contract交接：`zircon_runtime_interface/src/ui/window/**` 8/8确认batch无entry/bytes/age与drain预算，只去相邻redraw；ABI adapter逐项push且late error丢弃此前转换，下游同步route并收集N个results。Runtime09按PERF-MVP-314拥有resize/scale geometry barrier与render-only dirty帧尾合并：barrier后首个位置事件必须消费新layout/hit generation，不能为压缩move而跨越geometry语义；generation稳定时window metadata/report不得重建。Runtime12/EditorUI01分别拥有连续量coalesce与产品batch入口；验收100k mixed事件的layout/render/hit rebuild、result bytes、queue age与p95。
- 2026-07-23 UI root contract交接：interface clean root 7/7确认accessibility snapshot为wide owned Vec且node lookup线性，theme/design token构造重建String/Vec，text/widget event持before+after/previous+current/selection wide payload，Auto behavior在accessibility/focus/render/input重复字符串识别。Runtime09把这些DTO绑定到tree/theme/text/component generation：stable generation不重建snapshot/theme/event projection，node与behavior使用dense index/compiled mask；分别回链PERF-MVP-256/257、157/251/264、265/283/295并交给EditorUI01/03/04/06。验收1/100/10k nodes、1M stable lookup和100k widget/text events的build/clone/probe/p95。
- 2026-07-23 component contract交接：interface `ui/component/**` 25/25确认descriptor schema/event/slot线性查、default node/template全量深clone、UiValue递归TOML/display投影以及event envelope/patch多层owned String/BTreeMap。Runtime09让compiled component generation发布descriptor/field/event/slot dense handles、canonical template payload与single value owner；surface/input/render/accessibility只消费handle+changed receipt，stable generation不得重新parse/format/clone wide DTO。分别回链PERF-MVP-264/265/274/278/283/290/305并交给EditorUI05/06；验收10k descriptors/instances、100k events与1MiB values的lookup/clone/parse/p95。
- 2026-07-23 template contract交接：interface `ui/template/**` 56/56确认package/cache record重复header/key/snapshot并对同一artifact双重hash，expression/selector/resource/action反复构建char/token/lowercase/path String且parser无规模预算，resource/selector变化又过宽rebuild/dirty。Runtime09让single compiled generation owner发布artifact fingerprint、metadata、typed AST/selector/resource index和精确node/domain impact；runtime surface只消费dense handle，stable generation parse/hash/metadata clone=0。回链PERF-MVP-306/308/309/311/312并交给EditorUI04/05；验收10k assets/nodes/rules/dependencies及100 MiB artifact的hash/parse/clone/dirty/rebuild/p95。
- 2026-07-23 dispatch clean合同交接：interface `ui/dispatch/**` clean 18/19确认handler context/result强制owned route，input result又同时拥有full event/reply/diagnostics/effect分类/component/binding payload；产品pointer/navigation确有per-node route clone。Runtime09让surface dispatch共享single route generation并发布compact changed receipt，normal frame不保留wide diagnostic/event副本；回链PERF-MVP-254/265/278/293/294并交给EditorUI01/Runtime12。dirty `input/reply.rs`仍由原owner独立验收。
- 2026-07-23 v2 clean合同交接：interface `ui/v2/**` clean 4/7确认dense u32 arena为正向基线，但compiled document同时拥有arena、String→handle map和再次复制source/component/children的graph；arena payload仍为wide TOML maps。Runtime09让single compiled generation owner发布dense identity/topology与typed layout/style/event side tables，surface只持handle+mutable delta；回链PERF-MVP-274/276/312并交给EditorUI04/05。3个dirty文件仍由原owner独立验收。
- 2026-07-23 tree clean合同交接：interface `ui/tree/**` clean 7/11确认`insert_root/insert_child`每次全扫现有nodes求max paint order，template builder逐节点调用后构成O(N²)。Runtime09按PERF-MVP-573让bulk generation直接赋dense paint order，动态tree维护唯一derived next-order cursor，deserialize/import每generation只重建一次；不得增加与canonical order竞争的第二truth。验收100k nodes的bulk max scans=0、后续insert O(1) order，并保留paint/hit/navigation/serde语义。
- 2026-07-23 surface clean合同交接：interface `ui/surface/**` tracked-clean 30/47确认arranged lookup线性、surface frame/debug/timeline持wide全量artifact、route/focus保留多份路径和无界历史、brush/resource/style存在重复owner；文本source-map与shaped→paint投影还有grapheme/run重复扫描。Runtime09让surface发布generation-owned arranged/frame/render/text handles与compact current state，stable access/build/clone=0；分别回链PERF-MVP-254/277/278/280/282/288/289/292/293/296，并交给EditorUI03。9个dirty tracked与8个foreign untracked文件仍由原owner独立验收。
- 2026-07-23 clean contract tests性能交接：interface本批22个clean测试强覆盖serde/ABI/default/order/error语义，但ECS/layout/v2/surface fixture多为1–3项，只有pipeline测试100次pointer move零layout，未覆盖allocation/retained bytes/p95。Runtime09在PERF-MVP-263/274/278/312的既有门禁补1/1k/10k/100k nodes、stable 300 frames及changed-row比例，记录passes/visits/BTree+Vec alloc/published bytes；stable generation build/clone/heap=0，delta成本随changed rows。world-sync NotModified测试仍先构造rows，PERF-MVP-563须增加unchanged generation row build/visit/bytes=0断言。
- 2026-07-23 interface current-source收口交接：UI 218/218、tests 31/31已静态读完。`UiRenderCommand`当前已做到每conversion只用一次无临时Vec JSON hash，但stable command仍全序列化，Runtime09按PERF-MVP-178让mutation owner发布typed generation，paint不得由DTO serde求generation。foreign `UiBatchPlan`尚无runtime submit caller；产品Runtime Diagnostics已用它构造cache/parity/visualizer，出现重复key/resource/effect clone、paint→batch线性membership与O(P³..P⁴) overdraw。按280发布single render generation artifact+paint→batch index+bounded spatial debug sections，接入submit前与Render17共用同一batch authority；debug off工作=0，不能让interface DTO成为第二套renderer。

## 2026-08-27 Widget Menu Control-Anchored Test Owner Split

状态：`runtime_09_15_widget_menu_control_anchored_test_owner_split_static_passed_cargo_deferred`。

当前非算法结构切片把 `ui/tests/widget_menu_behavior.rs` 中 5 个 control-anchored
popup/dropdown overlay、frame hit authority、incremental input-policy 与 focus restoration 测试
原样迁入 folder-backed `widget_menu_behavior/control_anchored_overlays.rs`。父 owner 从 861 行
降到 625 行并保留 11 个 menu/popup dismissal tests 与共享 surface fixtures；child 为 239 行
和 5 个测试。另一会话新增的 typed component event 与 binding mode 字段仍位于父级共享
`binding(...)` helper，两个锚均由静态回归锁定。

Python 结构回归 1/1、定向 Rust `rustfmt --check`、迁移测试体规范化 SHA-256 等价与
scoped diff check 通过。popup stack、hit-test grid、input-policy patch、focus 和 dispatch
算法均未修改；Cargo 未执行，因此不声明 Runtime09/15 或 UI 产品验收，也未触发 milestone
commit/企微同步。

## 2026-08-27 UiSurface Incremental Rebuild Owner Split

状态：`runtime_09_15_ui_surface_incremental_rebuild_owner_split_static_passed_cargo_profile_deferred`。

当前非算法结构切片先复审 `UiSurface` rebuild 全路径与 Unreal Slate invalidation root/widget
list/heap/index 边界，再把 1194 行 `ui/surface/surface/rebuild.rs` 中完整 `rebuild_dirty`、增量
布局降级阈值和 layout-engine report patch/merge helper 硬切到 folder-backed
`rebuild/incremental.rs`。父 owner 现为 500 行，只保留 full rebuild、render extract、dirty
mutation 与 `compute_layout`；incremental child 为 711 行，继续修改同一 `UiSurface`，没有增加
facade、第二棵 tree、第二份 invalidation state 或兼容路由。

Python RED 先以父 owner 1194 行超预算失败，拆分后 production-owner guard 1/1 通过；移动的
4 个核心项 whitespace-normalized SHA-256 4/4 与拆分前一致，定向 rustfmt 和 scoped diff
check 通过。字体代次失效、1/4/256 降级阈值、arranged/hit/render 局部 patch、导航索引和
surface-frame publication 均保持原逻辑。P1-9 的 full-subtree frontier、P1-10 的 patch/rebuilt
诊断失真以及 persistent Taffy 仍开放；未生成 CPU/allocation/RSS/power 样本。Cargo、UI 产品
路径和 profile 验证延后，因此不声明 Runtime09/15 acceptance、瓶颈消失或算法最优，也未触发
milestone commit/企微同步。

## 2026-08-27 UiSurface Property Transaction Owner Split

状态：`runtime_09_15_ui_surface_property_transaction_owner_split_static_passed_cargo_profile_deferred`。

继续按 Unreal Slate attribute descriptor/value-change 到 typed invalidation reason 的职责边界，
把 959 行 `ui/surface/surface.rs` 中 surface property transaction 硬切到 folder-backed
`surface/property_transaction.rs`。该事务统一拥有 tree property、component state、runtime style、
focus/popup、editable text、clipboard revision 与 invalidation 的原子同步；483 行父 owner 继续
拥有 `UiSurface` 状态、构造、invalidation transaction、runtime style、hit/accessibility/debug
查询和 route projection，485 行 child 仍直接修改同一 surface，没有第二份 property store、
popup stack、edit state 或 invalidation truth。

Python RED 先以 959 行父 owner 超预算失败，拆分后 production-owner guard 1/1 通过；12 个
移动方法/helper 的 whitespace-normalized SHA-256 12/12 与拆分前一致，定向 rustfmt 与 scoped
diff check 通过。另一会话已有的 compiled binding event index、font generation、arranged
visibility、virtual-list materialization、hot-reload state、editable-text clipboard invalidation 和
runtime-anchored popup 命名均保留。本切片未改变 property/popup/text/focus 算法，也没有性能或
功耗样本；Cargo/UI 产品/profile 验证延后，不声明 Runtime09/15 acceptance 或瓶颈消失。

## 2026-08-27 UI Pointer Component State Owner Split

状态：`runtime_09_15_ui_pointer_component_state_owner_split_static_passed_cargo_profile_deferred`。

当前非算法结构切片复审 887 行 `surface/pointer_component_events.rs` 与 Unreal
`SlateApplication` 输入路由、`SWidget` hover/invalidation 状态分工后，把 hover/pressed/focus
component state、runtime pseudo-style propagation、render dirty 与 minimal ancestor-root helper
硬切到 226 行 folder-backed `pointer_component_events/state_invalidation.rs`。674 行父 owner
继续拥有 pointer component event、damage、compiled binding 与 template-action payload 投影；child
仍修改同一 surface/component state/style/invalidation owner，没有第二事件路由或 state cache。

Python RED 先以 887 行父 owner 超预算失败，迁移后 production-owner guard 1/1 通过；7 个
移动方法/helper 的 whitespace-normalized SHA-256 7/7 与拆分前一致，定向 rustfmt 与 scoped
diff check 通过。本切片未改变 ancestor walk、style subtree propagation、dirty domain、event
ordering、binding 或 payload 算法，也没有 CPU/allocation/RSS/power 样本。Cargo/UI 产品/profile
验证延后，不声明 Runtime09/15 acceptance、规模最优或瓶颈消失。

## 2026-08-28 UI Pointer Template Action Owner Split

状态：`runtime_09_15_ui_pointer_template_action_owner_split_static_passed_cargo_profile_deferred`。

继续对照 Unreal `SlateApplication` 的 pointer routing 与 `Framework/Commands/UIAction.h` 的
action contract 分工，将 binding handle 校验、声明式 action/route 选择、missing-value policy、
payload expression/property resolution 迁入 folder-backed
`pointer_component_events/template_action.rs`。426 行父 owner 只保留 route-derived component
event、focus event、damage 与 binding event envelope；262 行 action child 继续读取同一 tree、
compiled binding、control index 与 component-state authority，没有第二 dispatch/action registry。

结构 RED 先以父 owner 674 行超出 550 行边界失败；迁移后 9 个方法规范化 SHA-256 9/9 与
拆分前一致。事件顺序、handle 映射、missing-value policy、expression evaluation 和 payload
allocation 算法均未改变；Cargo/UI 产品与 CPU/allocation/RSS/power profile 延后，不声明
Runtime09/15 acceptance、性能收益、瓶颈消失或算法最优。

## 2026-08-28 UI Asset Surface Node Resource Owner Split

状态：`runtime_09_15_ui_asset_surface_node_resource_owner_split_static_passed_cargo_profile_deferred`。

复审 918 行 `ui/template/asset/surface_index.rs` 后，将 retained tree node metadata 的容错资源
URI/fallback 投影迁入 175 行 folder-backed
`surface_index/node_resource_registration.rs`；758 行根 owner 继续拥有 surface/tree 的正反向
索引、资源反向边、受影响 surface 选择与 hot-reload targeting。编译期严格 schema 诊断仍由
`ui/template/asset/resource_ref/collect.rs` 单独拥有，不把两个生命周期和错误合同错误合并。

结构 RED 先以 918 行根 owner 超过 800 行边界失败；迁移后 11 个方法/helper 的
whitespace-normalized SHA-256 11/11 与拆分前一致。该边界对照 Unreal UMG generated asset
metadata 与运行时 brush resource access，并以 Slint compiler resource embedding 交叉检查。
本切片未改变 parser、schema、fallback、去重、反向索引或 hot-reload 算法；Cargo/UI 产品与
CPU/allocation/RSS/power profile 延后，不声明 Runtime09/15 acceptance、性能收益或瓶颈消失。

## 2026-08-28 Winit Translation Domain Owner Split

状态：`runtime_09_15_winit_translation_domain_owner_split_static_passed_cargo_product_profile_deferred`。

复审 785 行 `ui/platform_input/winit_translation.rs` 及 Unreal `SlateApplication` 的平台事件
分派边界后，将键盘、指针、IME 和窗口事件包装分别硬切到 folder-backed
`winit_translation/{keyboard,pointer,ime,window}.rs`。530 行根 owner 保留唯一
`WindowEvent` 路由、公开 `translate_winit_modifiers` 路径与既有内联行为测试；四个生产 child
分别为 40/161/52/51 行，继续直接生成同一 `UiWindowInputPumpEvent` 合同，没有增加输入队列、
窗口状态、dispatch authority 或兼容路径。Fyrox `process_os_event` 作为 Rust 交叉检查。

Python RED 先以 785 行根 owner 超过预估结构预算失败；保留约 430 行既有测试后，最终预算按
540 行锁定，生产路由约 100 行。拆分后 source/status guard 2/2 通过；17 个移动函数体相对
`HEAD` 的去空白 SHA-256 17/17 等价，定向 rustfmt 与 scoped diff check 通过。事件顺序、
synthetic 标志、touch ID、0.1 pixel-scroll 比例、IME byte clamp、窗口 metadata/metrics 和
normalize 算法均未改变。Cargo、UI 产品路径及 CPU/allocation/RSS/power profile 延后，不声明
Runtime09/15 acceptance、性能收益、瓶颈消失或算法最优，也未触发 milestone commit/企微同步。
