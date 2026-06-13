---
related_code:
  - zircon_runtime/src/scene/inspection/mod.rs
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - zircon_runtime/src/scene/tests/authoring_boundary.rs
  - zircon_runtime/src/scene/tests/inspection.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/derived_state.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/recent_static_guards.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/plugin/core_profiles.rs
  - zircon_runtime/src/ui/surface/input/navigation.rs
  - zircon_editor/src/scene
  - zircon_hub/src/projects/metadata.rs
  - docs/engine-architecture/non-network-server-naming-m1.md
  - docs/engine-architecture/hard-cutover-migration-smells-m1.md
  - docs/engine-architecture/runtime-architecture-review-m0.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
status: in_progress
last_refined: 2026-06-14
---

# 05 scene/editor 边界收尾

## 现状与证据（2026-06-12 重核）

旧文两项核心"残留"已被解决，本计划工作面相应缩小并转向白名单机器化：

- **空目录已删（矫正）**：`zircon_runtime/src/scene/editor_projection/` 已不存在（`ls` 报 No such file or directory）。全仓 `editor_projection` 残留引用仅 1 处文本：`scene/tests/component_structure.rs`（执行时核验语义：是"断言不存在"的守卫还是过期注释）。原 M1 切片 1 的"删目录"工作已完成。
- **序列化纯净守卫已枚举完备（矫正）**：`scene/tests/authoring_boundary.rs` 已有双 token 禁入清单——`SERIALIZED_AUTHORING_TOKENS`（19 词：selection/selection_anchors/scene_gizmos/gizmo/overlay/camera_override/preview_lighting/viewport_camera/SceneViewportSettings/pane 等）与 `SOURCE_AUTHORING_TOKENS`（25 词：含 SelectionHighlightExtract/GridOverlayExtract/SceneGizmoOverlayExtract 等 extract 类型名）；helper `assert_text_excludes_authoring_tokens` + 负例自检 `authoring_boundary_guard_fails_on_representative_tokens`。消费方覆盖 world project、dynamic scene、asset scene、inspection JSON 与 serialization source 守卫。
- **inspection 守卫已存在（矫正）**：`scene/tests/inspection.rs` 已有 `world_inspection_serialization_excludes_editor_authoring_tokens` 与 `world_inspection_filters_missing_focus_without_storing_authoring_state`。原 M2 切片 2 的"给 inspection 加守卫"已完成。
- **inspection 公共面**：`scene/inspection/mod.rs` 仅导出 `WorldInspectionField` / `WorldInspectionHierarchyRow` / `WorldInspection` 三类型（field/hierarchy/snapshot 三文件），形状中性。
- **剩余真实工作 1——"editor" 命名白名单**：runtime 内 "editor" 命中约 181 文件（执行时重核：Grep `-l editor`，path `zircon_runtime/src`），混杂三类：合法 editor-host 目标语义（`dynamic_api/session.rs` 的 editor 模式、`plugin/core_profiles.rs:10` `pub struct EditorCoreProfile`、native loader 的 editor_host 校验）、测试夹具、待裁决 authoring 残留。无白名单则无法机器化判定新增违规。
- **剩余真实工作 2——"legacy" 命名裁决**："legacy" 403 处/84 文件。实仓抽样（`ui/surface/input/navigation.rs:22-54`）：`legacy` 是 `dispatch_navigation_event` 旧路由回复的本地变量名，承载真实运行语义（route/focus/diagnostics 全从它取值）——属"领域词化的迁移痕迹"，需逐类裁决是改名、文档化还是列债。
- 命名审计文档锚点（2026-06-12 实测存在）：`docs/engine-architecture/non-network-server-naming-m1.md`、`hard-cutover-migration-smells-m1.md`、`runtime-root-surface-m1.md`、`runtime-architecture-review-m0.md`——白名单产出并入这些既有口径，不另起新文件。

## 目标

1. 物理与文本残留清零：`editor_projection` 的最后 1 处文本残留裁决处理。
2. "editor" 与 "legacy" 命名逐类裁决，产出白名单 + 机器化守卫，使违规新增可被测试拒绝。
3. 守卫从"已存在"升级为"覆盖矩阵可证"：三序列化出口 × 双 token 表 × inspection 的覆盖关系成表，token 清单有维护公约。

## 非目标

- 不迁移任何已在 `zircon_editor` 的投影/选中/gizmo 逻辑；不改 inspection 三类型的数据形状（除非裁决发现 authoring 泄漏）。
- 不在本计划处理 ui/surface/input 的输入路径重构（"legacy" 若裁决为迁移债，移交 UI 边界 owner 计划）。
- 渲染骨架内容归 render 计划 01-08。

### 全局硬约束（继承总计划 §4，违反即返工）

- 硬切换不留兼容层；不新增 crate；非网络语义 server 命名是 blocker（白名单审计顺带复核，口径并入 `non-network-server-naming-m1.md`）。

## 执行前检查清单

1. 活动会话对齐：serialization 守卫与 inspection 均在 `20260604-1232` 会话工作区延长线上——执行前确认该会话对应切片已完成或已交接，避免双写同一守卫。
2. worktree 脏文件检查：`git status --porcelain -- zircon_runtime/src/scene/ docs/engine-architecture/`。
3. 事实重核：
   - `ls zircon_runtime/src/scene/ | grep editor_projection`（应无输出）
   - `grep -rn "editor_projection" zircon_runtime/src --include=*.rs`（应仅 component_structure.rs 1 处）
   - `grep -rl "editor" zircon_runtime/src --include=*.rs | wc -l`（editor 命中文件数基线重核）
   - `grep -rn "legacy" zircon_runtime/src/ui/surface/input --include=*.rs | wc -l`
4. 基线记录：`cargo test -p zircon_runtime --lib scene:: --locked` 通过数记入状态节。

## 里程碑

### M1 残留收尾与命名白名单裁决

#### 切片 1.1 editor_projection 文本残留处理

- 目标文件：`zircon_runtime/src/scene/tests/component_structure.rs`（唯一残留处）。
- 改动形态：核验该处语义——若是"目录不得复活"的守卫断言则保留并加注释；若是过期文本则删除该词。无其他代码改动。
- 调用方迁移：无。
- 验收：Grep `editor_projection` 全仓结果与裁决一致（0 处或仅守卫 1 处）。
- DoD：重核命令输出与判词记入状态节。

#### 切片 1.2 "editor" 命中三分类白名单

- 目标文件：`docs/engine-architecture/runtime-root-surface-m1.md` 或 `runtime-architecture-review-m0.md`（并入既有审计口径，执行时与 `20260604-1232` 会话定稿落点，禁止另起新文件）；本计划状态节（清单副本）。
- 改动形态：纯文档 + 裁决。分类规则（已核实锚点）：
  - **白名单（合法 editor-host 目标语义）**：`dynamic_api/session.rs` editor 会话模式、`plugin/core_profiles.rs` `EditorCoreProfile`（:10）/`RuntimeCoreProfile`（:4）双 profile、native loader 的 editor_host 校验行、`builtin` 的 target mode 词汇。
  - **测试夹具**：测试文件内的 editor 字样按文件粒度白名单。
  - **违规清单（authoring 语义）**：生产代码中 selection/gizmo/inspector 词根（与 `SOURCE_AUTHORING_TOKENS` 25 词复用判据）命中者，逐项给迁移/删除条目。
  - 枚举命令：Grep `editor`，path `zircon_runtime/src`，glob `**/*.rs`（约 181 文件，按顶层目录分桶逐桶裁决）。
- 调用方迁移：无（裁决期；违规项的迁移在切片 1.4 或移交 owner）。
- 验收：白名单三分类表齐备，每桶有计数；违规清单每项有处置判词。
- DoD：清单落审计文档；违规清单为空或每条带 owner。

#### 切片 1.3 "legacy" 逐类裁决

- 目标文件：同 1.2 的审计文档落点；代表区域 `ui/surface/input/`（pointer_reply/pointer/navigation 集中区）。
- 改动形态：纯文档 + 裁决。三分类：
  - **领域词（有真实运行语义）**：如 `navigation.rs:22-54` 的 `legacy` 路由回复变量——route/focus/diagnostics 取值来源；裁决"保留 + 代码注释说明语义"或"改名为语义词（如 `routed_reply`）"，二选一判词。
  - **测试夹具**：asset 测试夹具中 legacy 样例按文件白名单。
  - **真实迁移债**：列入 UI 输入路径 owner 计划，本计划不展开。
  - 枚举命令：Grep `legacy`，path `zircon_runtime/src`，glob `**/*.rs`（403 处/84 文件基线，执行时重核）。
- 调用方迁移：无（裁决期）。
- 验收：84 文件分桶判词齐备；改名项（若有）列成独立切片清单。
- DoD：清单落审计文档，UI 债条目已移交（文档交叉引用）。

#### 切片 1.4 命名守卫机器化

- 目标文件：`zircon_runtime/src/scene/tests/authoring_boundary.rs`（扩展）或 `zircon_runtime/src/tests/`（新守卫文件，执行时按 1.2/1.3 白名单形态定稿落点）。
- 改动形态（已定稿）：

  ```rust
  #[test]
  fn runtime_editor_and_legacy_naming_is_classified_by_owner() {
      // 扫描 zircon_runtime/src，按文件 owner 分类 editor / legacy 命名；
      // 新增未分类命名会失败，已分类 debt 留给对应 owner 切片。
  }
  ```

- 调用方迁移：无。
- 验收：守卫对违规注入有负例（参照既有 `authoring_boundary_guard_fails_on_representative_tokens` :62 的自检模式）。
- DoD：`cargo test -p zircon_runtime --lib naming_boundary --locked` 全绿且白名单外新增命名会失败。

#### M1 测试阶段（milestone-first）

- 切片期：`cargo check -p zircon_runtime --lib --locked`
- 里程碑末：
  - `cargo test -p zircon_runtime --lib scene:: --locked`（无回归）
  - `cargo test -p zircon_runtime --lib authoring --locked -- --nocapture`（新守卫）
  - `cargo test -p zircon_runtime --lib naming_boundary --locked -- --nocapture`（命名白名单）
- 验收证据：白名单/违规清单写入既有命名审计文档（与 `20260604-1232` 口径合并）；守卫进常驻测试树。

### M2 守卫覆盖矩阵审计与维护公约

#### 切片 2.1 覆盖矩阵成表

- 目标文件：`docs/zircon_runtime/scene/inspection.md`（既有，刷新守卫说明；执行时核验：`ls docs/zircon_runtime/scene/`）。
- 改动形态：纯文档。成表：行 = 四个出口（world 序列化、dynamic scene、asset scene、inspection 快照），列 = 双 token 表（SERIALIZED 19 词 / SOURCE 25 词）与负例自检；逐格标注承载测试（已核实：`world_basics.rs` / `component_structure.rs` / `derived_state.rs` / `asset_scene.rs` / `inspection.rs:148,161`）。空格（某出口未覆盖某表）即补测试条目。
- 调用方迁移：无。
- 验收：矩阵无空格，或空格有补测试切片。
- DoD：矩阵落 `inspection.md` 且每格可点到测试名。

#### 切片 2.2 token 清单维护公约

- 目标文件：`scene/tests/authoring_boundary.rs`（注释公约）+ `docs/zircon_runtime/scene/inspection.md`（公约正文）。
- 改动形态：定稿公约——editor 侧新增 authoring 状态类型（如新 overlay/gizmo extract 类型）时，`SOURCE_AUTHORING_TOKENS` 必须同 PR 追加该类型名；公约写明判定规则（出现在 `zircon_editor/src/scene` 的 extract/投影类型名默认入表）。补一条结构测试（签名草案）：`authoring_token_tables_stay_sorted_and_deduplicated`（防清单腐化）。
- 调用方迁移：无。
- 验收：公约 + 结构测试。
- DoD：测试绿，公约可执行（有判定规则而非倡议）。

#### M2 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib authoring --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib inspection --locked -- --nocapture`
- 验收证据：覆盖矩阵 + 公约 + 守卫测试全绿；`docs/zircon_runtime/scene/inspection.md` 刷新。

### M3 收尾闭环（2026-06-12 二次细化新增；M1/M2 全切片已完成后的关账步骤）

#### 切片 3.1 legacy debt bucket 移交确认

- 目标文件：本计划状态节 + 各 owner 计划/审计文档交叉引用。
- 改动形态：核对 `runtime_naming_boundary` 审计输出中的 10 个 legacy debt bucket（runtime UI input/render、graphics、DDS、UI template/layout、input、asset、dynamic API、scene schema 等）——每个 bucket 必须可点到 owner 计划条目或审计文档判词；缺失者补交叉引用（UI 输入路径债 → UI 边界 owner 计划；DDS/graphics 债 → render 计划对应子计划）。
- 调用方迁移：无。
- 验收：10 bucket 全部有 owner 落点链接。
- DoD：移交表落状态节。

#### 切片 3.2 状态闭环

- 目标文件：本计划 frontmatter（status → completed）；`docs/plans/zircon_runtime/runtime/index.md` §3 状态行同步。
- 改动形态：收尾回归 `cargo test -p zircon_runtime --lib scene:: --locked`（全族无回归确认，此前验证用的是 naming_boundary/authoring/inspection 过滤词）；通过后翻转状态并在 §2.2 P6/P9 行补"已闭环"判词。
- 验收：scene:: 全族绿；index 与本计划状态一致。
- DoD：status: completed 落盘。

#### M3 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib scene:: --locked`
- 验收证据：命令输出摘要 + 状态翻转 + index 同步。

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1 | 1.1 文本残留处理 | 完成 | 2026-06-12 | `zircon_runtime/src/scene/editor_projection` 不存在；`editor_projection` 仅保留在 `component_structure.rs` 的禁止复活结构守卫中 |
| M1 | 1.2 editor 白名单 | 完成 | 2026-06-12 | `runtime_naming_boundary` 分类 runtime `editor` 命名：1260 locations / 0 unclassified；白名单并入 `runtime-architecture-review-m0.md` |
| M1 | 1.3 legacy 裁决 | 完成 | 2026-06-12 | `runtime_naming_boundary` 分类 runtime `legacy` 命名：516 locations / 0 unclassified；10 个 debt owner bucket 记录在审计输出 |
| M1 | 1.4 守卫机器化 | 完成 | 2026-06-12 | 新增 `runtime_absorption::naming_boundary` 与 `runtime_structure_audits/runtime_naming_boundary.py`；`cargo test -p zircon_runtime --lib naming_boundary --locked` 1 passed |
| M2 | 2.1 覆盖矩阵 | 完成 | 2026-06-12 | `docs/zircon_runtime/scene/inspection.md` 记录 world project / dynamic scene / asset scene / inspection JSON 覆盖矩阵；`cargo test -p zircon_runtime --lib authoring --locked` 17 passed；`cargo test -p zircon_runtime --lib inspection --locked` 6 passed |
| M2 | 2.2 维护公约 | 完成 | 2026-06-12 | `authoring_token_tables_stay_sorted_and_deduplicated` 固化 token 表排序去重；token 表 smoke check: serialized 19 / source 25；`inspection.md` 记录 editor authoring 类型新增时的同步规则；authoring/inspection Cargo 过滤测试通过 |
| M3 | 3.1 legacy debt bucket 移交确认 | 状态复核完成 | 2026-06-13 | `runtime_naming_boundary` 当前 gate status 为 `classified`：editor 1363 locations / 237 files / 0 unclassified；legacy 518 locations / 101 files / 0 unclassified；legacy 仍为 10 个 debt owner bucket，并已在 runtime 总览 P9 以 runtime UI input/render、graphics、DDS、UI template/layout、input、asset、dynamic API、scene schema owner 分派口径同步；未改行为代码。 |
| M3 | 3.1 non-network server gate fixture sync | 静态复核完成 | 2026-06-14 | `non_network_server_naming` 现在把 Hub UNC path fixture（`\\?\UNC\server\share\Game` / `\\server\share\Game`）列为允许上下文，而不是 runtime owner debt；当前审计：count 59、graphics-render-framework-debt 58、editor-workbench-authority-label-debt 1、allowed_context_count 94、observer_false_positive_count 87、unclassified 0、`risks` 仅剩已分类 owner migration debt；未改 Hub/Runtime 生产代码。 |
| M3 | 3.1 hard-cutover migration-smell owner sync | 静态复核完成 | 2026-06-14 | `hard_cutover_migration_smells` 当前审计：source_file_count 5839、legacy_reference_count 212、compat/shim 0、allowed_business_bridge_reference_count 300、migration_bridge_smell_count 0、classification groups 7、unclassified 0；新增/确认 runtime UI input 63、hybrid GI render 56、runtime graphics 30、Hub archived text 58、DDS 1、Net hyper client API 1、editor UI fixture 3。该切片只更新审计归类和文档证据，未改 UI/Hub/Net/graphics 生产行为代码。 |
| M3 | 3.2 状态闭环 | pending_full_scene_cargo | — | 本计划 frontmatter 从 `completed` 修正为 `in_progress`，因为计划 DoD 要求 `cargo test -p zircon_runtime --lib scene:: --locked` 全族无回归后才能最终关账；当前只有 `naming_boundary`、`authoring`、`inspection` scoped Cargo 通过记录。新增 `runtime_plan_last_refined_covers_latest_recorded_date`、`runtime_plan_status_does_not_claim_completed_while_validation_is_pending`、`runtime_plan_frontmatter_status_uses_known_lifecycle_values`、`runtime_index_status_map_matches_subplan_frontmatter`、`runtime_index_subplan_map_covers_existing_plan_files_without_stale_rows`、`runtime_index_problem_rows_reference_existing_subplans`、`runtime_index_execution_dependencies_reference_existing_subplans`、`runtime_index_in_progress_rows_record_remaining_gate`、`runtime_known_backlog_gaps_keep_owner_and_trigger_columns`、`runtime_subplans_keep_status_and_evidence_tables`、`runtime_subplan_status_records_keep_non_empty_evidence`、`runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs`、`runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation`、`runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation`、`runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation`、`runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation`、`runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation`、`runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation`、`runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation`、`runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation`、`runtime_10_m1_3_cargo_pending_gate_stays_explicit_until_dynamic_api_validation`、`runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff`、`runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass`、`runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation`、`runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass`、`runtime_14_module_family_cargo_gate_stays_visible_until_filters_pass`、`runtime_architecture_review_documents_all_absorption_guards` 与 `runtime_05_closeout_status_waits_for_full_scene_cargo_gate`，防止待验证计划误标 completed，并锁定 last_refined 覆盖最新状态日期、总表状态必须跟随子计划 frontmatter、总索引子计划地图必须精确覆盖 01-14 子计划文件、问题清单与执行依赖列必须指向现有子计划、所有 in_progress 总表行必须说明剩余 gate 或 blocker、总索引已知 backlog 缺口必须保留现状依据与 owner/触发条件、所有子计划必须保留状态/证据表且已启动记录必须写入具体证据、Runtime 01-14 近期静态/待验证守卫必须同步到计划/镜像文档/评审/索引（Runtime 02 `core/root/generated/export_build_plan/app/editor/plugin` gate 必须保持可见，Runtime 06 `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` gate 必须保持可见，Runtime 09 `ui/input/naming_boundary/layout/template` owner/Cargo gate 必须保持可见，Runtime 07 覆盖 `runtime_07_hotspot_inventory_requires_counted_evidence_before_m2`、`runtime_frame_schedule_stage.<SystemStage>` 与 `runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation`，Runtime 10 同时覆盖 `runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces`、`runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge` 与 Runtime 10 UI 镜像契约 M2 owner/Cargo gate，且 M1.3 在 `dynamic_api` Cargo 通过前保持 `code_static_passed_cargo_pending`，M2 在 Runtime 09/editor UI owner 交接与 interface/ui/editor Cargo 通过前保持 pending，Runtime 11 `tasks/ecs_schedule/worker_pool/rayon` Cargo gate 必须保持可见，Runtime 12 在 input/action_map/gamepad/app Cargo 通过前保持 pending gate 可见，Runtime 13 在 script filters Cargo 通过前保持 `code_static_pending_cargo`，Runtime 14 模块族守卫必须覆盖四族镜像文档，Runtime 05 closeout 必须保持 `pending_full_scene_cargo`）、Runtime 01 `tech_stack/text_shaper/plugin physics` Cargo gate、Runtime 02 `core/root/generated/export_build_plan/app/editor/plugin` gate、Runtime 03 `ecs_schedule/time/session/schedule_parallel` Cargo gate、Runtime 04 broader `asset::` / `worker_pool` Cargo gate、Runtime 06 `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` Cargo/native gate、Runtime 07 `extract/ecs_query/performance profiling/FPS` Cargo gate、Runtime 08 ECS 数据面 Cargo gate、Runtime 09 `ui/input/naming_boundary/layout/template` owner/Cargo gate、Runtime 10 UI 镜像契约 M2 owner/Cargo gate、Runtime 05 `scene::` Cargo closeout gate、Runtime 11 `tasks/ecs_schedule/worker_pool/rayon` Cargo gate 与 Runtime 14 模块族 Cargo/rustc gate 必须保持可见，以及 M0 评审必须覆盖 `runtime_absorption/mod.rs` 和所有已挂载 guard。 |

计划状态镜像（2026-06-14）：新增 `runtime_plan_status_boundary` 并接入总结构审计，覆盖 plan-status support files 6/6、runtime subplans 14/14、runtime index subplan rows 14/14、problem rows 17/17、known backlog rows 7/7、status counts `in_progress=14`、core guard anchors 13/13、pending Cargo gate anchors 15/15、doc anchors 8/8、Runtime 05 closeout status `in_progress`、closeout anchors present、`risks = []`。这仍是静态结构证据，最终 closeout 继续等待 `pending_full_scene_cargo` 与 `cargo test -p zircon_runtime --lib scene:: --locked`。

基线数值（开工首日记录）：

- `editor_projection` 残留基线：目录 0、文本 1 处（component_structure.rs）
- "editor" 命中基线：224 文件（2026-06-12 PowerShell `Select-String` 重核）
- "legacy" 命中基线：500 处 / 98 文件（2026-06-12 PowerShell `Select-String` 重核）
- 命名守卫重核：editor 1260 locations / 0 unclassified；legacy 516 locations / 0 unclassified；legacy debt bucket 10（2026-06-12 `runtime_naming_boundary` JSON）
- 命名守卫复核：editor 1363 locations / 237 files / 0 unclassified；legacy 518 locations / 101 files / 0 unclassified；legacy debt bucket 10（2026-06-13 `runtime_naming_boundary_audit` 当前工作区）
- 非网络 server 命名门禁复核：count 59 / classification groups 2 / allowed contexts 94 / observer false positives 87 / unclassified 0（2026-06-14 `non_network_server_references` 当前工作区；Hub UNC fixture 已允许）
- 硬切换 migration-smell 门禁复核：source_file_count 5839 / legacy_reference_count 212 / allowed_business_bridge_reference_count 300 / classification groups 7 / unclassified 0（2026-06-14 `hard_cutover_migration_smells` 当前工作区；Hub archived text 与 Net hyper client API 已归类为非 Runtime 05 生产改动 owner debt）
- token 表基线：SERIALIZED 19 词 / SOURCE 25 词（authoring_boundary.rs:1-49）
- 守卫测试基线：authoring_boundary 2 + inspection 2 + serialization 出口消费文件 4（world_basics / dynamic_scene / asset_scene / inspection）+ source guard 1（component_structure）
- 验证记录：`cargo check -p zircon_runtime --lib --locked` 通过；`cargo test -p zircon_runtime --lib naming_boundary --locked` 1 passed；`cargo test -p zircon_runtime --lib authoring --locked` 17 passed；`cargo test -p zircon_runtime --lib inspection --locked` 6 passed（均使用 `E:\cargo-targets\zircon-runtime-naming-boundary-0612`）。`cargo test -p zircon_runtime --lib scene:: --locked` 全族闭环仍待干净 Cargo 通道。

## 风险与协调

- serialization 守卫与 inspection 均在 `20260604-1232` 会话 touched_modules 中——本计划两个里程碑都属其工作区延长线，执行前必须确认该会话对应切片已完成或已交接，避免双写同一守卫；白名单文档落点必须与其 m1 审计文档合并口径。
- 切片 1.4 的源扫描守卫与子计划 02 M4 的 generated 守卫同属"结构扫描测试"family：实现时共享扫描 helper（若 02 先落地则复用其遍历工具，反之亦然），避免两套文件遍历代码。
- "legacy" 改名类裁决（若选改名）触及 `ui/surface/input` 行为文件——该区域同时是 UI 输入路径计划地盘，改名切片必须移交或与其 owner 协调，本计划不单方面执行。
