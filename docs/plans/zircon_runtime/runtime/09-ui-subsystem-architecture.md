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
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/surface/ecs_projection.rs
  - zircon_runtime/src/ui/template
  - zircon_runtime/src/ui/template/build/interaction.rs
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
last_refined: 2026-06-21
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

#### 切片 2.1 taffy 单后端入口与 pass 序

- 目标文件：`ui/layout/taffy_bridge/{mod,compute}.rs`（唯一后端计算入口）、`ui/layout/pass/{pipeline,layout_tree,incremental,taffy_arrange}.rs`（pass 序显式化）、`ui/layout/style_mapping.rs`。
- 改动形态：盘点 taffy 直接调用面（执行前检查清单第 3 项的 Grep 结果）——除 taffy_bridge 外的直连改经 bridge（硬切换）；pass 序（measure→layout→arrange？按实仓定）写成显式序列并文档化；`style_mapping` 与 `interface/ui/style` 的映射职责单点。
- 调用方迁移：taffy 直连违规点（执行时枚举，预计 ≤5）。
- 验收：`runtime_09_taffy_layout_pass_order_uses_bridge_authority`（结构测试：Taffy tree build/compute 只在 bridge compute；full/incremental layout 消费 `UI_LAYOUT_PASS_ORDER`；`style_mapping` 只保留 DTO adapter 判词）。
- DoD：Grep `taffy::` path `zircon_runtime/src/ui` 的实际生产命中被分为 bridge compute、bridge style API、style DTO adapter、pass selection/reporting；直接 `TaffyTree::new()` / `compute_layout` 只允许在 `taffy_bridge/compute.rs`。

#### 切片 2.2 virtualization / scroll 边界声明

- 目标文件：`ui/layout/virtualization.rs`、`ui/layout/scroll.rs` + 文档。
- 改动形态：声明虚拟化窗口与滚动偏移的 owner 与失效时机（数据变更/视口变更）；补行为测试：`virtualized_list_only_materializes_visible_window`、`scroll_offset_invalidates_virtualization_window`、`non_virtualized_scroll_offset_keeps_full_window_dirty_domain`（名按实仓 API 定稿）。
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
| 横切 | UI architecture Markdown renderer split | ui_architecture_markdown_split_static_passed_cargo_deferred_tests_deferred | 2026-06-21 | Status anchor `ui_architecture_markdown_split_static_passed_cargo_deferred_tests_deferred`; `ui_architecture_markdown.py` now owns `render_ui_architecture_boundary_markdown(...)`; `ui_architecture_boundary.py` remains the 541-line audit/risk owner, and the Markdown owner is 110 lines. Direct `ui_architecture_boundary_audit` reports source files 52/52, ui entries 18/18, surface entries 20/20, legacy full hits 54/54, production legacy hits 0/0 across 0 files, taffy production hits 173/173 across 9 files, runtime ui::v2 anchors 10/10, interface ui::v2 anchors 9/9, guard anchors 19/19, pending UI owner/Cargo gate anchors 7/7, doc anchors 61/61, `mirror_docs_guard_present = true`, and `risks = []`. Validation: Python py_compile, direct audit, standalone `ui_architecture.rs` 18/18, and standalone `plan_status.rs` 33/33; full UI behavior filters and package Cargo remain deferred under the existing `ui/input/naming_boundary/layout/template` owner gate. |
| M3 | 3.1 UI resource resolver mapped invalidation | runtime_09_resource_resolver_mapped_invalidation_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | Extended `UiResourceResolver::invalidate_uris(...)` in `zircon_runtime/src/ui/template/asset/resource_ref/resolver.rs` so cached `asset://` / `project://` primary and fallback references are invalidated by their mapped runtime `ResourceLocator` string when a `UiResourceResolverSchemeMap` is configured. This keeps hot reload reports that carry `res://...` or `package://...` locators aligned with UI template references cached under their authoring URI. Behavior anchor added: `ui_resource_resolver_invalidates_mapped_ui_scheme_primary_and_fallback_uris`. Documentation updated in `docs/zircon_runtime/ui/template/asset/resource_ref/resolver.md`. Validation: `rustfmt --edition 2021 --check` passed for touched Rust; API/documentation anchor scan passed; conflict-marker scan clean; scoped `git diff --check` passed with LF/CRLF warnings only. Full Cargo and focused behavior tests remain deferred under the implementation-first direction, so Runtime 09 `ui/input/naming_boundary/layout/template` Cargo gates remain pending. |
| M3 | 3.1 UI resource resolver scheme mapping | runtime_09_resource_resolver_scheme_map_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | Added `UiResourceResolverSchemeMap` to `zircon_runtime/src/ui/template/asset/resource_ref/resolver.rs` and exported it through `ui::template::asset` and `ui::template`. Default resolver behavior is unchanged: valid `asset://` / `project://` template URIs still produce missing-resource diagnostics rather than invalid-URI diagnostics when no mapping is configured. Hosts can now opt into registry lookup by mapping `asset://` to a runtime scheme such as `res://`, or `project://` to `package://{package_id}/...`; mapped lookups preserve `#label` fragments and still require an existing `ResourceManager` record with the expected kind. Behavior anchors: `ui_resource_resolver_maps_asset_scheme_to_runtime_locator_when_configured`, `ui_resource_resolver_maps_project_scheme_to_package_locator_when_configured`, and `ui_resource_resolver_preserves_ui_scheme_labels_when_mapping_to_runtime_locator`. Validation: rustfmt touched Rust passed; API/documentation anchor scan passed; full Cargo and focused behavior tests are deferred under the implementation-first direction, so `ui/input/naming_boundary/layout/template` Cargo gates remain pending. |
| M0 | 0.1 模块边界图 | completed_static_passed | 2026-06-13 | `docs/zircon_runtime/ui/architecture.md` 记录 `runtime_09_m0_ui_architecture_static_passed`：覆盖当前 18 个 `ui/` 顶层条目、20 个 `surface/` 条目、owner/dependency map 与 M1-M3 工作集；新增 `runtime_absorption::ui_architecture::runtime_09_ui_architecture_doc_records_current_boundaries` 静态守卫；本切片无 UI 生产代码改动，Cargo 未启动（已有 active lanes）。 |
| M0 | 0.2 v2 裁决 | completed_static_passed | 2026-06-13 | `docs/zircon_runtime/ui/architecture.md` 记录 `v2-replacement-mainline`：v2 是替代主线；`.zui` 是生产 component suffix，`.v2.ui.toml` 保留为 v2 view/style/runtime fixture/editor chrome profile；old recursive template 路径为 migration/test-only，删除条件移交 M3；新增 `runtime_absorption::ui_architecture::runtime_09_v2_verdict_matches_runtime_and_interface_modules`；调用方盘点覆盖 `zircon_runtime/src`、`zircon_editor/src`、`zircon_app/src`。 |
| 横切 | UI architecture 结构镜像 | structure_audit_static_passed_cargo_pending | 2026-06-17 | `runtime_structure_audits/ui_architecture_boundary.py` 当前静态事实已随 M1.1 route authority、M1.2 navigation reply rename、M1.2 pointer reply rename、M1.2 pointer capture fallback rename、M1.2 table row label fallback rename、M1.2 template component-name fallback rename、M1.2 property visibility flag rename、M1.2 responsive MUI visibility flag rename、M1.2 accessibility open-state fallback rename、M1.2 layout engine backend name cutover、M1.2 surface default interaction fallback rename、M2.1 Taffy bridge/pass-order authority、M2.2 virtualization/scroll boundary 与 M3.1 template pipeline/generated policy 刷新：`expected_source_file_count = 52`、`expected_ui_entry_count = 18`、`expected_surface_entry_count = 20`、`legacy_full_hits = 54`、`expected_legacy_full_hits = 54`、`legacy_production_hits = 0`、`expected_legacy_production_hits = 0`、`legacy_production_file_count = 0`、`expected_legacy_production_file_count = 0`、`taffy_production_hits = 175`、`expected_taffy_production_hits = 175`、`taffy_production_file_count = 10`、`expected_taffy_production_file_count = 10`、`runtime_v2_anchor_count = 10`、`interface_v2_anchor_count = 9`、`guard_anchor_count = 19`、`cargo_gate_anchor_count = 7`、`doc_anchor_count = 61`、`missing_doc_anchors = []`、`missing_cargo_gate_anchors = []`、`mirror_docs_guard_present = true`、`risks = []`；这仍是静态结构证据，M1 production legacy 命名桶已清零，完整 UI Cargo behavior filters 仍等待 editor UI owner/Cargo lanes 空窗。 |
| 横切 | UI architecture 镜像文档守卫 | mirror_docs_guard_static_passed_cargo_pending | 2026-06-17 | `runtime_absorption::ui_architecture::runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts` 与 `ui_architecture_boundary` 把 `docs/zircon_runtime/ui/architecture.md`、本计划、runtime index、M0 review 与 runtime-interface convergence 固定到同一组结构审计事实：`expected_source_file_count = 52`、`expected_ui_entry_count = 18`、`expected_surface_entry_count = 20`、`legacy_full_hits = 54`、`expected_legacy_full_hits = 54`、`legacy_production_hits = 0`、`expected_legacy_production_hits = 0`、`legacy_production_file_count = 0`、`expected_legacy_production_file_count = 0`、`taffy_production_hits = 175`、`expected_taffy_production_hits = 175`、`taffy_production_file_count = 10`、`expected_taffy_production_file_count = 10`、`runtime_v2_anchor_count = 10`、`interface_v2_anchor_count = 9`、`guard_anchor_count = 19`、`cargo_gate_anchor_count = 7`、`doc_anchor_count = 61`、`missing_doc_anchors = []`、`missing_cargo_gate_anchors = []`、`mirror_docs_guard_present = true`、`risks = []`。验证：rustfmt check、Python py_compile、direct `ui_architecture_boundary_audit`、standalone rustc 18/18；UI behavior filters 继续 pending。 |
| 横切 | Runtime 15 runtime UI support split 后 UI entry 基线同步 | runtime_15_runtime_ui_dead_code_support_split_coremin_check_passed | 2026-06-22 | Runtime 15 runtime UI dead-code support split 删除旧生产 `ui/runtime_ui/` 并新增生产 `ui/public_runtime_frame.rs` 与 test-only `ui/tests/runtime_ui_support` 后，Runtime 09 UI 架构审计的当前扫描事实更新为 `expected_ui_entry_count = 18`、`ui_taffy_production_hits=175` / `ui_taffy_production_files=10`；`ui/` 扫描 entry 为 14 个目录加 `module.rs`、`prelude.rs`、`public_runtime_frame.rs`、`style.rs`，`mod.rs` 仍按审计规则排除。结构守卫继续读取 test-support `RuntimeUiManager` 以锁定 direct pointer/navigation leaf helper 判词，完整 UI behavior filters 仍 pending。 |
| M1 | 1.1 路由单点 | runtime_09_m1_1_ui_input_route_authority_static_passed_cargo_pending | 2026-06-16 | 新增 `zircon_runtime/src/ui/surface/input/route_authority.rs` 并接入 `surface/input/dispatch.rs`：所有 normalized `UiInputEvent` 分支汇总后统一调用 `annotate_authoritative_input_dispatch`，诊断 notes 写入 `route_authority=runtime_09_m1_1_ui_input_route_authority;policy=...;stages=...`，阶段列表由 `UI_INPUT_ROUTE_ORDER` 投射；`runtime_09_ui_input_events_route_through_single_dispatch_authority` 锁定单点出口、route-order 消费和 docs/index 状态锚。旁路清单已给 owner 判词 `runtime_09_m1_1_direct_pointer_navigation_routes_are_leaf_owner_helpers`：`dispatch_pointer_event*` / `dispatch_navigation_event` 仍是低层 leaf helper，不是 normalized `UiInputEvent` 权威入口；standalone rustc `ui_architecture.rs` 6/6 通过；Cargo/行为测试按用户要求延后。 |
| M1 | 1.2 navigation legacy reply rename | runtime_09_m1_2_navigation_legacy_reply_renamed_static_passed_cargo_pending | 2026-06-16 | `zircon_runtime/src/ui/surface/input/navigation.rs` 的本地路由回复变量从 `legacy` 硬切为 `routed_reply`，行为路径不变；状态锚 `runtime_09_m1_2_navigation_legacy_reply_renamed_static_passed_cargo_pending` 与新增 `runtime_09_navigation_legacy_reply_rename_reduces_ui_input_debt` 守卫锁定该文件不再含 `legacy`，UI legacy 基线从原始 `ui_legacy_hits=167` / `ui_legacy_production_hits=102` / `ui_legacy_production_files=12` 降为当前 `ui_legacy_hits=153` / `ui_legacy_production_hits=88` / `ui_legacy_production_files=11`；剩余 11 个 production legacy 文件继续归 M1.2 后续处置；Cargo 仍等待 active lanes 空窗。 |
| M1 | 1.2 pointer legacy reply rename | runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending | 2026-06-16 | `zircon_runtime/src/ui/surface/input/pointer.rs` 与 `zircon_runtime/src/ui/surface/input/pointer_reply.rs` 的本地路由结果变量从 `legacy` 硬切为 `routed_result`，行为路径不变；状态锚 `runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending` 与 `runtime_09_pointer_legacy_reply_rename_reduces_ui_input_debt` 守卫锁定两个文件不再含 `legacy`，UI legacy 当轮基线降为 `ui_legacy_hits=104` / `ui_legacy_production_hits=39` / `ui_legacy_production_files=9`；剩余 9 个 production legacy 文件继续归 M1.2 后续处置。验证：rustfmt check、Python py_compile、direct `ui_architecture_boundary_audit`、standalone rustc 10/10；Cargo/行为测试按当前实现优先要求延后。 |
| M1 | 1.2 pointer capture fallback rename | runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending | 2026-06-16 | `zircon_runtime/src/ui/surface/input/state/pointer_capture.rs` 与 `zircon_runtime/src/ui/surface/input/effect/focus_pointer.rs` 的 pointer capture 判断从旧 fallback 命名硬切到 `has_pointer_capture_for_owner`；Editor UI 01.M4.S1 后不再保留 unindexed single-pointer fallback，high-precision pointer 只接受 indexed `UiPointerId` capture map 事实。状态锚 `runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending` 与 guard `runtime_09_pointer_capture_fallback_rename_reduces_ui_input_debt` 锁定旧 API 名称不再回归，UI legacy 当前基线降为 `ui_legacy_hits=102` / `ui_legacy_production_hits=37` / `ui_legacy_production_files=7`；剩余 7 个 production legacy 文件继续归 M1.2 后续处置。验证：rustfmt check、Python py_compile、direct `ui_architecture_boundary_audit`、standalone rustc 11/11；Cargo/行为测试按当前实现优先要求延后。 |
| M1 | 1.2 table row label fallback rename | runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending | 2026-06-16 | `zircon_runtime/src/ui/surface/render/collection_rows/table.rs` 的 row-label fallback splitter 从 `split_legacy_table_text` 硬切为 `split_row_label_table_text`，行为路径不变；状态锚 `runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending` 与 guard `runtime_09_table_row_label_fallback_rename_reduces_ui_render_debt` 锁定旧 helper 名称不再回归，UI legacy 当前基线降为 `ui_legacy_hits=100` / `ui_legacy_production_hits=35` / `ui_legacy_production_files=6`；剩余 6 个 production legacy 文件继续归 M1.2 后续处置。验证：rustfmt check、Python py_compile、direct `ui_architecture_boundary_audit`、standalone rustc 12/12；Cargo/行为测试按当前实现优先要求延后。 |
| M1 | 1.2 template component-name fallback rename | runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending | 2026-06-16 | `zircon_runtime/src/ui/template/build/interaction.rs` 的 template interaction fallback 从 `legacy_component_interaction_fallback` / `legacy_interactive` 硬切为 `component_name_interaction_fallback` / `component_name_interactive`，行为路径不变；状态锚 `runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending` 与 guard `runtime_09_template_component_name_fallback_rename_reduces_ui_template_debt` 锁定旧 helper/local 名称不再回归，UI legacy 当前基线降为 `ui_legacy_hits=95` / `ui_legacy_production_hits=30` / `ui_legacy_production_files=5`；剩余 5 个 production legacy 文件继续归 M1.2 后续处置。验证：rustfmt check、Python py_compile、direct `ui_architecture_boundary_audit`、standalone rustc 13/13；Cargo/行为测试按当前实现优先要求延后。 |
| M1 | 1.2 property visibility flag rename | runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending | 2026-06-16 | `zircon_runtime/src/ui/surface/property_mutation.rs` 的 visibility transition helper 参数从 `legacy_visible` 硬切为 `state_visible_flag`，行为路径不变；状态锚 `runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending` 与 guard `runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt` 锁定旧 local 名称不再回归，UI legacy 当前基线降为 `ui_legacy_hits=92` / `ui_legacy_production_hits=27` / `ui_legacy_production_files=4`；剩余 4 个 production legacy 文件继续归 M1.2 后续处置。验证：rustfmt check、Python py_compile、direct `ui_architecture_boundary_audit`、standalone rustc 14/14；Cargo/行为测试按当前实现优先要求延后。 |
| M1 | 1.2 responsive MUI visibility flag rename | runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending | 2026-06-16 | `zircon_runtime/src/ui/layout/pass/responsive_mui.rs` 的 responsive visibility DTO 从 `legacy_visible` 硬切为 `state_visible_flag`，行为路径不变；状态锚 `runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending` 与 guard `runtime_09_responsive_mui_visibility_flag_rename_reduces_ui_layout_debt` 锁定旧 local 名称不再回归，UI legacy 当前基线降为 `ui_legacy_hits=84` / `ui_legacy_production_hits=19` / `ui_legacy_production_files=3`；剩余 3 个 production legacy 文件继续归 M1.2 后续处置。验证：rustfmt check、Python py_compile、direct `ui_architecture_boundary_audit`、standalone rustc 15/15；Cargo/行为测试按当前实现优先要求延后。 |
| M1 | 1.2 accessibility open-state fallback rename | runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending | 2026-06-16 | `zircon_runtime/src/ui/accessibility/extract.rs` 的 open-state fallback property set 从 `legacy_properties` / `legacy_property` 硬切为 `fallback_properties` / `fallback_property`，行为路径不变：authored `open_property` 仍优先，随后同名 component-state，再检查 `expanded` / `popup_open` / `open` 兼容属性与 runtime expanded/popup flags；状态锚与 guard `runtime_09_accessibility_open_state_fallback_rename_reduces_ui_a11y_debt` 锁定旧 helper/local 名称不再回归，UI legacy 当轮基线降为 `ui_legacy_hits=76` / `ui_legacy_production_hits=11` / `ui_legacy_production_files=2`；剩余 2 个 production legacy 文件继续归 M1.2 后续处置。验证：rustfmt check、Python py_compile、direct `ui_architecture_boundary_audit`、standalone rustc 16/16；Cargo/行为测试按当前实现优先要求延后。 |
| M1 | 1.2 layout engine backend name cutover | runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending | 2026-06-16 | `zircon_runtime_interface/src/ui/layout/engine.rs` 与 `zircon_runtime/src/ui/layout/pass/engine.rs` 将公开后端名从 `LegacyZircon` / `legacy_zircon` / `legacy_selected_count` 硬切为 `UiLayoutEngineBackend::Zircon`、`UiLayoutEngineCapability::zircon()` 与 `zircon_selected_count`，不保留兼容别名；`runtime_09_layout_engine_backend_name_cutover_reduces_ui_layout_debt` 锁定旧公开名称不再回流。UI legacy 当前基线降为 `ui_legacy_hits=63` / `ui_legacy_production_hits=9` / `ui_legacy_production_files=1`；剩余 1 个 production legacy 文件为 `surface/surface/default_interactions.rs`。验证：rustfmt check、Python py_compile、direct `ui_architecture_boundary_audit`、standalone rustc 17/17；Cargo/行为测试按当前实现优先要求延后。 |
| M1 | 1.2 surface default interaction fallback rename | runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending | 2026-06-17 | `zircon_runtime/src/ui/surface/surface/default_interactions.rs` 的 `default_open_boolean_value(...)` 将 open-state fallback property set 从 `legacy_properties` / `legacy_property` 硬切为 `fallback_properties` / `fallback_property`，行为路径不变：authored property 仍优先，随后同名 component-state，再检查 fallback aliases 与 canonical runtime open flag；状态锚与 guard `runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt` 锁定旧 local 名称不再回归，UI legacy 当轮基线降为 `ui_legacy_hits=54` / `ui_legacy_production_hits=0` / `ui_legacy_production_files=0`，M1 production legacy 命名桶清零。验证：rustfmt check、Python py_compile、direct `ui_architecture_boundary_audit`、standalone rustc 18/18；Cargo/行为测试按当前实现优先要求延后。 |
| M2 | 2.1 taffy 单入口 | runtime_09_m2_1_taffy_bridge_pass_order_static_passed_cargo_pending | 2026-06-16 | `zircon_runtime/src/ui/layout/taffy_bridge/{mod,compute}.rs` 取代旧 `taffy_bridge.rs` 单文件形态；`compute_taffy_child_frames(...)` 是 Taffy tree build / `TaffyTree::new()` / `compute_layout` 唯一 owner；`layout/pass/taffy_arrange.rs` 只保留 eligibility、fallback report 与递归 arrange；`layout/pass/pipeline.rs` 新增 `UI_LAYOUT_PASS_ORDER`，full 与 incremental layout 均通过 `assert_layout_pass_stage(...)` 消费同一 pass 序。`runtime_09_taffy_layout_pass_order_uses_bridge_authority` 锁定 `runtime_09_m2_1_style_mapping_remains_taffy_dto_adapter` 判词：`style_mapping.rs` 仍是 Taffy DTO adapter，不是后端执行入口。验证：rustfmt check、`cargo check -p zircon_runtime` 通过（现有 warning noise）；完整 `layout` / `ui` behavior filters 按用户要求后续补跑。 |
| M2 | 2.2 虚拟化边界 | runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending | 2026-06-16 | `zircon_runtime/src/ui/layout/scroll.rs` 新增 `UiScrollVirtualizationPlan` 与 `plan_scrollable_virtual_window(...)`，统一裁决 scroll offset clamping、viewport/content extent 与 virtual-window visible_range invalidation；`layout/pass/arrange.rs` 与 `tree/node/scroll.rs` 均消费该 planner，后者以 OR 方式保留既有 visible_range dirty；`scroll_virtualization.rs` 新增 `virtualized_list_only_materializes_visible_window`、`scroll_offset_invalidates_virtualization_window`、`non_virtualized_scroll_offset_keeps_full_window_dirty_domain`。验证：rustfmt check、Python py_compile、direct `ui_architecture_boundary_audit`、standalone `ui_architecture.rs` 8/8、`cargo check -p zircon_runtime --lib --locked` 通过；focused `cargo test -p zircon_runtime --lib scroll_virtualization --locked --jobs 1` 304s timeout no result，完整 layout/ui behavior filters 按用户要求后续补跑。 |
| M3 | 3.1 模板边界 | runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending | 2026-06-16 | `zircon_runtime/src/ui/template/pipeline.rs` 新增 `UiTemplateRuntimePipeline`、`UI_TEMPLATE_RUNTIME_PIPELINE_STAGES` 与 `UiTemplateRuntimePipelineError`，把旧 recursive template 路径固定为 `load -> validate -> instance -> build`；`UiTemplateInstance::from_validated_document(...)` 提供已验证实例化入口；`UiRuntimeCompiledAssetArtifact` 记录 `runtime_09_m3_1_binary_leaf_dto_artifact_not_generated_source`，声明当前编译产物是 binary/TOML DTO payload 而不是 generated source，未来若写源码必须使用 `// @generated <generator> - do not edit by hand`。`template_pipeline.rs` 已补 `template_validate_rejects_unknown_component_contract`、`template_instance_failure_surfaces_loader_error`、`compiled_template_artifact_stays_binary_leaf_dto_not_generated_source` 三个验收锚；按当前实现优先要求，完整 `template`/`ui` behavior filters 后续补跑。 |
| 横切 | UI owner/Cargo pending gate | code_static_pending_owner_cargo | 2026-06-13 | 新增 `runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation`，保持 Runtime 09 在 `ui/input/naming_boundary/layout/template` 验证线通过前为 `in_progress`：M0.1/M0.2 仅为静态文档/守卫证据，`runtime_absorption::ui_architecture` Cargo 尚未运行；M1 输入路由、M1 legacy 处置、M2 taffy/layout、M3 template generated/failure-path 仍等待 editor UI owner 与 Cargo lanes 空窗。 |

基线数值（开工首日记录）：

- `ui/` 模块条目基线：17；`surface/` 条目：20（2026-06-13 current scan；2026-06-12 旧 ls 为 21）
- UI 区 legacy 命中基线：`ui_legacy_hits=54`（full `zircon_runtime/src/ui/**/*.rs`，含 tests/fixtures；2026-06-17 M1.2 navigation + pointer reply + pointer capture fallback + table row label fallback + template component-name fallback + property visibility flag + responsive MUI visibility flag + accessibility open-state fallback rename + layout engine backend name cutover + surface default interaction fallback rename 后）；生产代码基线：`ui_legacy_production_hits=0` / `ui_legacy_production_files=0`
- taffy 引用文件基线：`ui_taffy_production_hits=175` / `ui_taffy_production_files=10`（排除 tests/fixtures；M2.1 已裁决为 bridge compute + style DTO adapter + pass selection/reporting，Taffy tree build/compute 仅 `taffy_bridge/compute.rs`）
- 基线守卫：`runtime_absorption::ui_architecture::runtime_09_ui_architecture_baselines_match_current_source_scan`
- `cargo test -p zircon_runtime --lib ui --locked` 通过数基线：未记录，本轮 M0 不启动 Cargo，因为已有 active `cargo`/`rustc` lanes；后续源码切片开工前重跑。

## 风险与协调

- **editor UI 活动会话是本计划最大冲突源**（`ui/tests/` 两文件曾编译错误即其工作区）：每个切片动工前 `git status` 核 `ui/**` 脏区，脏文件先避让；禁止回退其改动。
- v2 裁决若判"替代代"，迁移路线横跨 editor（workbench 模板消费方）——路线图必须与 editor 计划/会话共定，本计划不单方面删非 v2。
- 输入路由收束与 03 的 UI extract 旁路归一（03-M1 切片 1.2）相邻：dispatch 改动错峰执行，先开工者在状态节登记。
- 与 12 的交接面：12-M0.2 已裁决 UI 命中事件优先进入本计划的 interaction_gate/dispatch，action mapping 只处理 UI 未处理输入；后续 09 切片不得重复定义 gameplay 动作绑定。
- interface::ui 镜像契约的同步守卫归 10 计划——本计划发现的重复定义类型清单移交 10 的 M2 工作集，不在本计划修。
