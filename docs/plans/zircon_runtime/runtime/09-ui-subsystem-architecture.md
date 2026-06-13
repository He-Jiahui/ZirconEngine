---
related_code:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/layout/taffy_bridge.rs
  - zircon_runtime/src/ui/layout/pass
  - zircon_runtime/src/ui/layout/style_mapping.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/surface/input
  - zircon_runtime/src/ui/surface/pointer
  - zircon_runtime/src/ui/surface/navigation
  - zircon_runtime/src/ui/surface/ecs_projection.rs
  - zircon_runtime/src/ui/template
  - zircon_runtime/src/ui/v2
  - zircon_runtime/src/ui/dispatch
  - zircon_runtime_interface/src/ui
  - zircon_runtime/src/ui/surface/input/navigation.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/ui-and-layout/shared-ui-template-runtime.md
status: in_progress
last_refined: 2026-06-14
---

# 09 UI 子系统架构收束

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

#### 切片 2.1 taffy 单后端入口与 pass 序

- 目标文件：`ui/layout/taffy_bridge.rs`（唯一后端入口声明）、`ui/layout/pass/`（pass 序显式化）、`ui/layout/style_mapping.rs`。
- 改动形态：盘点 taffy 直接调用面（执行前检查清单第 3 项的 Grep 结果）——除 taffy_bridge 外的直连改经 bridge（硬切换）；pass 序（measure→layout→arrange？按实仓定）写成显式序列并文档化；`style_mapping` 与 `interface/ui/style` 的映射职责单点。
- 调用方迁移：taffy 直连违规点（执行时枚举，预计 ≤5）。
- 验收：`taffy_is_only_reachable_through_layout_bridge`（结构测试：UI 区 taffy 引用仅 bridge 文件）。
- DoD：Grep `taffy::` path `zircon_runtime/src/ui` 仅 taffy_bridge.rs 命中。

#### 切片 2.2 virtualization / scroll 边界声明

- 目标文件：`ui/layout/virtualization.rs`、`ui/layout/scroll.rs` + 文档。
- 改动形态：声明虚拟化窗口与滚动偏移的 owner 与失效时机（数据变更/视口变更）；补行为测试：`virtualized_list_only_materializes_visible_window`、`scroll_offset_invalidates_virtualization_window`（名按实仓 API 定稿）。
- 调用方迁移：无公共面变化。
- 验收：两测试绿。
- DoD：边界文档 + 测试落地。

#### M2 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib layout --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib ui --locked`

### M3 template 编译与实例化边界

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

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M0 | 0.1 模块边界图 | completed_static_passed | 2026-06-13 | `docs/zircon_runtime/ui/architecture.md` 记录 `runtime_09_m0_ui_architecture_static_passed`：覆盖当前 17 个 `ui/` 顶层条目、20 个 `surface/` 条目、owner/dependency map 与 M1-M3 工作集；新增 `runtime_absorption::ui_architecture::runtime_09_ui_architecture_doc_records_current_boundaries` 静态守卫；本切片无 UI 生产代码改动，Cargo 未启动（已有 active lanes）。 |
| M0 | 0.2 v2 裁决 | completed_static_passed | 2026-06-13 | `docs/zircon_runtime/ui/architecture.md` 记录 `v2-replacement-mainline`：v2 是替代主线；`.zui` 是生产 component suffix，`.v2.ui.toml` 保留为 v2 view/style/runtime fixture/editor chrome profile；old recursive template 路径为 migration/test-only，删除条件移交 M3；新增 `runtime_absorption::ui_architecture::runtime_09_v2_verdict_matches_runtime_and_interface_modules`；调用方盘点覆盖 `zircon_runtime/src`、`zircon_editor/src`、`zircon_app/src`。 |
| 横切 | UI architecture 结构镜像 | structure_audit_static_passed_cargo_pending | 2026-06-14 | 新增 `runtime_structure_audits/ui_architecture_boundary.py` 并接入 `audit_runtime_structure.py`；当前静态事实：source/doc files 11/11、`ui/` entries 17/17、`surface/` entries 20/20、UI legacy full-tree hits 167/167、UI legacy production hits 102/102、UI legacy production files 12/12、UI taffy production hits 161/161、UI taffy production files 7/7、runtime `ui::v2` anchors 10/10、interface `ui::v2` anchors 9/9、guard anchors 4/4、pending UI owner/Cargo gate anchors 7/7、doc anchors 10/10、`risks = []`；这仍是静态结构证据，M1-M3 源码切片与 UI Cargo 仍等待 editor UI owner/Cargo lanes 空窗。 |
| M1 | 1.1 路由单点 | 待开始 | — | — |
| M1 | 1.2 legacy 处置 | 待开始 | — | — |
| M2 | 2.1 taffy 单入口 | 待开始 | — | — |
| M2 | 2.2 虚拟化边界 | 待开始 | — | — |
| M3 | 3.1 模板边界 | 待开始 | — | — |
| 横切 | UI owner/Cargo pending gate | code_static_pending_owner_cargo | 2026-06-13 | 新增 `runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation`，保持 Runtime 09 在 `ui/input/naming_boundary/layout/template` 验证线通过前为 `in_progress`：M0.1/M0.2 仅为静态文档/守卫证据，`runtime_absorption::ui_architecture` Cargo 尚未运行；M1 输入路由、M1 legacy 处置、M2 taffy/layout、M3 template generated/failure-path 仍等待 editor UI owner 与 Cargo lanes 空窗。 |

基线数值（开工首日记录）：

- `ui/` 模块条目基线：17；`surface/` 条目：20（2026-06-13 current scan；2026-06-12 旧 ls 为 21）
- UI 区 legacy 命中基线：`ui_legacy_hits=167`（full `zircon_runtime/src/ui/**/*.rs`，含 tests/fixtures）；生产代码基线：`ui_legacy_production_hits=102` / `ui_legacy_production_files=12`
- taffy 引用文件基线：`ui_taffy_production_hits=161` / `ui_taffy_production_files=7`（排除 tests/fixtures；M2.1 应收敛到单一 owner 或更新 owner 判词）
- 基线守卫：`runtime_absorption::ui_architecture::runtime_09_ui_architecture_baselines_match_current_source_scan`
- `cargo test -p zircon_runtime --lib ui --locked` 通过数基线：未记录，本轮 M0 不启动 Cargo，因为已有 active `cargo`/`rustc` lanes；后续源码切片开工前重跑。

## 风险与协调

- **editor UI 活动会话是本计划最大冲突源**（`ui/tests/` 两文件曾编译错误即其工作区）：每个切片动工前 `git status` 核 `ui/**` 脏区，脏文件先避让；禁止回退其改动。
- v2 裁决若判"替代代"，迁移路线横跨 editor（workbench 模板消费方）——路线图必须与 editor 计划/会话共定，本计划不单方面删非 v2。
- 输入路由收束与 03 的 UI extract 旁路归一（03-M1 切片 1.2）相邻：dispatch 改动错峰执行，先开工者在状态节登记。
- 与 12 的交接面：12-M0.2 已裁决 UI 命中事件优先进入本计划的 interaction_gate/dispatch，action mapping 只处理 UI 未处理输入；后续 09 切片不得重复定义 gameplay 动作绑定。
- interface::ui 镜像契约的同步守卫归 10 计划——本计划发现的重复定义类型清单移交 10 的 M2 工作集，不在本计划修。
