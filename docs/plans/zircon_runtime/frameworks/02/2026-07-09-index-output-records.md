# Frameworks index 镜像迁出记录

> 来源文件：`docs/plans/zircon_runtime/frameworks/index.md`
> 迁移说明：Frameworks 总索引中的 Runtime 15/Frameworks 02 镜像明细已迁入 Frameworks 02 产出目录。

## 2026-07-08 Frameworks 02 Runtime 15 root entries/root-layout current-child route sync 镜像

Frameworks 02 最新镜像：`Runtime 15 M3 root entries/root-layout current-child route sync` / `runtime_15_m3_root_entries_root_layout_current_child_route_sync_static_passed_cargo_deferred` 已同步为 `frameworks_02_m3_root_entries_root_layout_current_child_route_sync_static_passed_cargo_deferred`。本切片把 structure-convention guard 从旧父 route 读取升级到当前子 owner：`structure_convention/test_file_budget/root_entries.rs` 读取 `expected_slices/{status,date}/runtime_15/foundation/lock_poison.rs`，`structure_convention/test_file_budget/root_layout/module_layout.rs` 读取 `expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/test_file_budget.rs`，`structure_convention/test_file_budget/root_layout/status_scan.rs` 读取 `expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/runtime_row_data.rs`。验证镜像：focused Cargo `root_layout` 6/6、fresh binary `runtime_15_root_entries_guard_child_owners_are_folder_backed` 1/1、plan-status `status_output_tables` 2/2 通过；package/workspace Cargo 未声明通过。

验证镜像：scoped rustfmt passed；focused Cargo recompilation is pending because another active cargo/rustc lane is running. 本切片只修正 status-output/test-file-budget 测试守卫 current-child 路径，不声明 runtime/plugin/render/editor/text/ZUI 生产行为变更。

---

## 2026-07-08 Frameworks 02 Runtime 15 Runtime 07 owner-budget child-source current-route sync 镜像

Frameworks 02 最新镜像：`Runtime 15 M3 Runtime 07 owner-budget child-source current-route sync` / `runtime_15_runtime_07_owner_budget_child_source_current_route_sync_static_passed_cargo_deferred` 已同步为 `frameworks_02_m3_runtime_07_owner_budget_child_source_current_route_sync_static_passed_cargo_deferred`。本切片把 Runtime 07 owner-budget 结构守卫从旧父 route/source-map 读取升级到当前子 owner：`structure_convention/test_file_budget/runtime_07_performance_hotspots_owner_budget.rs`、`runtime_07_performance_hotspots_owner_budget_large_file.rs` 与 `runtime_07_performance_hotspots_owner_budget_mirror_docs.rs` 现在读取 `performance_hotspots/owner_budget/sources/load.rs`、`performance_hotspots/owner_budget/mirror_docs/source_inventory.rs` 和 `expected_slices/{status,date}/runtime_15/m3_structure_support/runtime07_script_maps/runtime07_owner_budget_maps.rs`。

验证镜像：focused owner-budget 3/3 passed；plan-status `status_output_tables` 2/2 passed；package/workspace Cargo 未声明通过。本切片只修正 performance-hotspots/structure-convention/status-output 测试守卫 current-child source/map 路径，不声明 runtime/plugin/render/editor/text/ZUI 生产行为变更。

---

## 2026-07-08 Frameworks 02 Runtime 15 runtime plugin lifecycle row-data current-child route sync 镜像

Frameworks 02 最新镜像：`Runtime 15 M3 runtime plugin lifecycle fixture row-data current-child route sync` / `runtime_15_runtime_plugin_lifecycle_fixture_row_data_current_child_route_sync_static_passed_cargo_deferred` 已同步为 `frameworks_02_m3_runtime_plugin_lifecycle_fixture_row_data_current_child_route_sync_static_passed_cargo_deferred`。本切片把 runtime plugin lifecycle fixture 结构守卫从旧父 row-data route 读取升级到当前子 owner：`structure_convention/test_file_budget/runtime_plugin_lifecycle.rs` 现在聚合读取 `expected_status_row_data/runtime_15/m3/scene_script_tests/plugin_extension_tests.rs` 与 `expected_status_row_data/runtime_15/m3/scene_script_tests/plugin_extension_tests/runtime_catalog_rows.rs`。

验证镜像：focused lifecycle guard 1/1 passed；plan-status `status_output_tables` 2/2 passed；package/workspace Cargo 未声明通过。本切片只修正 structure-convention/status-output 测试守卫 current-child row-data 路径，不声明 runtime/plugin/render/editor/text/ZUI 生产行为变更。

---

## 2026-07-08 Frameworks 02 Runtime 15 shader prewarm manifest current-child route sync 镜像

Frameworks 02 最新镜像：`Runtime 15 M3 shader prewarm manifest current-child route sync` / `runtime_15_shader_prewarm_manifest_current_child_route_sync_static_passed_cargo_deferred` 已同步为 `frameworks_02_m3_shader_prewarm_manifest_current_child_route_sync_static_passed_cargo_deferred`。本切片把 shader prewarm manifest 结构守卫从旧父 row-data route/source-facade 读取升级到当前子 owner：`structure_convention/test_file_budget/shader_prewarm_manifest.rs` 现在聚合读取 `expected_status_row_data/runtime_15/m3/status_support.rs` 与 `expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/render_shader_support.rs`，`structure_convention/test_file_budget/shader_prewarm_manifest/builtin_template_source.rs` 则匹配当前 `graphics/scene/mod.rs` 的 `resources::{default_pipeline_key, PipelineKey, ResourceStreamer}` crate-wide facade。

验证镜像：focused validation 待本轮记录；package/workspace Cargo 未声明通过。本切片只修正 structure-convention/status-output 测试守卫 current-child row-data/source-facade 路径，不声明 shader prewarm/render runtime 生产行为变更。

---

## 2026-07-08 Frameworks 02 Runtime 15 UI text pipeline test owner split 镜像

Frameworks 02 最新镜像：`Runtime 15 M3 UI text pipeline test owner split` / `runtime_15_m3_ui_text_pipeline_test_owner_split_static_passed_cargo_deferred` 已同步为 `frameworks_02_m3_ui_text_pipeline_test_owner_split_static_passed_cargo_deferred`。本切片删除旧 `zircon_runtime/src/ui/tests/text_pipeline.rs` flat owner，把测试硬切到 `zircon_runtime/src/ui/tests/text_pipeline/` folder-backed tree：`mod.rs` 挂载 route，`fixtures.rs` 承接共享 fixture，`font_registry.rs`、`layout_request.rs`、`measure_cache.rs`、`surface_cache.rs`、`render_extract_prewarm.rs` 分别承接现有断言。

验证镜像：scoped rustfmt passed；focused `text_pipeline` cargo test 15/15 passed；direct `runtime_15_no_oversized_test_files` 1/1 passed；当前全量 structure filter 为 1226/1303 passed、77 failed remaining，剩余失败不来自 `text_pipeline` 或 oversized-test-file budget。Package/workspace Cargo remains deferred。

---

## 2026-07-08 Frameworks 02 Runtime 15 current-child route + IBL writeback budget 镜像

Frameworks 02 最新镜像：`Runtime 15 M3 current-child route plus IBL runtime writeback budget cleanup` / `runtime_15_m3_current_child_route_ibl_writeback_budget_cleanup_static_passed_cargo_deferred` 已同步为 `frameworks_02_m3_current_child_route_ibl_writeback_budget_cleanup_static_passed_cargo_deferred`。本切片把 structure-convention guard 从旧父 route mirror 继续收束到 current child owners，并把 IBL runtime writeback 测试从生产 owner 中硬切出去：`graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback.rs` 现在只保留 56 行 production route owner，测试位于 `graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback/tests.rs`，metrics 位于 `graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback/tests/metrics.rs`。

验证镜像：scoped rustfmt passed；focused current-child structure guards 12/12 passed；production-file budget guard passed；`runtime_graph_writeback` 4/4 passed；当前全量 structure filter 经后续 UI text pipeline split 为 1226/1303 passed、77 failed remaining。Package Cargo remains deferred；不声明 package/workspace Cargo pass。

---

## 2026-07-07 Frameworks 02 Runtime 15 production-file budget UI/IBL/project owner split 镜像

Frameworks 02 最新镜像：`Runtime 15 production-file budget UI/IBL/project owner split` / `runtime_15_production_file_budget_ui_ibl_project_owner_split_static_passed_cargo_check_offline_locked_blocked` 已同步为 `frameworks_02_m3_production_file_budget_ui_ibl_project_owner_split_static_passed_cargo_check_offline_locked_blocked`。本切片按 frameworks 02 的模块内核/生命周期守卫口径，把 production-file budget 热点继续拆到 child owner：UI render color/geometry/background tests、UI text font-assets/native-bitmap-atlas test、IBL bake dispatch tests、project render PBR/HDRI helpers 均完成硬切换，父 route 不承接旧实现体。

验证镜像：scoped rustfmt passed；standalone structure-convention `production_file_budget --test-threads=1` 通过 104/104；runtime tests no-default-features offline cargo check passed with warnings only。Package `--locked` gate 被当前非本切片 `Cargo.lock` drift 阻塞；不声明 locked Cargo pass。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 priority plan docs source-tree 镜像

Frameworks 02 最新镜像：`Runtime 15 M3 priority plan docs source-tree reconciliation` / `runtime_15_priority_plan_docs_source_tree_reconciliation_static_passed_cargo_deferred` 已同步为 `frameworks_02_m3_priority_plan_docs_source_tree_reconciliation_static_passed_cargo_deferred`。本切片按 frameworks 02 的模块内核/生命周期守卫口径，把 priority-plan-doc guard source aggregation 收束到 `structure_convention/test_file_budget/priority_plan_docs/status_sources.rs`，把 source ownership 检查下沉到 `priority_plan_docs/guard_tests/inventory_sync/source_ownership.rs`，并让 production support priority row source blob 读取 child row files。

验证镜像：scoped rustfmt passed；standalone structure-convention harness 重新编译；focused `priority_plan_docs --test-threads=1` 通过 25/25；plan-status `status_output_tables --test-threads=1` 通过 2/2。Package Cargo remains deferred；不声明 package Cargo pass。

本目录是 ZirconEngine 整体组织架构的权威框架计划集。它回答一个问题：**在保持公开三包形态（`zircon_app` / `zircon_runtime` / `zircon_editor`）与既有收束规则的前提下，如何把当前的半成品引擎组织形态推进到"开发者友好、插件开发友好、核心简洁、工程化精细、可维护、鲁棒、编译快速、功能高度解耦"的最终框架形态**，并把开发规范固化为可执行的守卫机制。

与既有计划集的分工与优先级：

- `docs/plans/zircon_runtime/runtime/`：子系统语义级对齐（调度、asset、ECS、UI、脚本等）的权威。本计划集不重复其内容；两者交叠处（runtime/01 依赖治理、runtime/02 core 脊柱、runtime/06 插件面、runtime/15 结构规范），**宏观组织决策（crate 拓扑、feature 矩阵、DX 工具链、守卫机制）以本目录为准，子系统内部语义以 runtime/ 计划集为准**。
- `docs/plans/zircon_runtime/render|shader|text/`：渲染域权威计划集，本计划集只约束其所在 crate/feature 的组织边界，不触碰渲染语义。
- `.codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md` 与 `全系统重构方案.md`：三包公开形态、core 脊柱角色、Scene/Editor 边界仍是绑定规则。本计划集在其之上做**内部组织的演进**，其中"runtime 内部 crate 化"（计划 01）是对"单 crate 吸收层"实现方式的一次显式修订，理由与决策记录见下文 §3 D1。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 typed-error child-ownership source-tree 镜像

`Runtime 15 M3 typed-error child-ownership source-tree reconciliation` / `runtime_15_typed_error_child_ownership_source_tree_reconciliation_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_typed_error_child_ownership_source_tree_reconciliation_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/code-review/status-output 测试守卫 owner：`typed_error_child_owners` root/child path inventories 保留 real child path audit，typed-error source blobs 改为 path-aware，typed-error structure status row/status-date map helpers 统一走 status-doc 聚合，native plugin loader 与 moved-guard absence 历史 anchors 由 child source trees 承接。验证镜像：structure-convention harness 重新编译通过（327 existing warnings），focused `typed_error_child_owners` 93/93，wide `code_review_findings` 218/218；Cargo gate deferred。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 typed-error status-doc source/status-map 镜像

`Runtime 15 M3 typed-error status-doc source/status-map reconciliation` / `runtime_15_typed_error_status_doc_source_status_map_reconciliation_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_typed_error_status_doc_source_status_map_reconciliation_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/code-review/status-output 测试守卫 owner：`typed_error_child_owners/status_docs/sources.rs` 统一读取 typed-error status-doc row children、typed-error structure assertion row children、typed-error map child owners 与 typed-error-structure map child owners，status-doc status-current guards 不再直接拼接旧父 map；`paths/status_slices.rs` direct child anchor 改为其拥有的 `#[path = "status_slices/paths.rs"]` mount。验证镜像：structure-convention harness 重新编译通过（303 existing warnings），focused `typed_error_status_doc` 51/51；Cargo gate deferred。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 owner-path budget groups 镜像

`Runtime 15 M3 child-groups owner-path budget groups folder-backed split` / `runtime_15_m3_child_groups_owner_path_budget_groups_folder_backed_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_child_groups_owner_path_budget_groups_folder_backed_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/status-output 测试守卫 owner：`root_owner_paths/m3_child_group_owner_paths.rs` 保持 route/group owner，budget groups 拆入 `m3_child_group_owner_paths/{root_guard_paths,owner_path_routes,plan_status_row_paths,folder_backed}.rs`，并由 `runtime_15_m3_child_group_owner_paths_are_folder_backed` 锁定无旧式回流。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 legacy guard body 镜像

`Runtime 15 M3 status-output expected-slice legacy guard body folder-backed split` / `runtime_15_status_output_expected_slice_legacy_guard_body_folder_backed_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_status_output_expected_slice_legacy_guard_body_folder_backed_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/status-output 测试守卫 owner：`structure_convention/test_file_budget/status_output_expected_slices/legacy_maps/guard_body.rs` 保持 route owner，checks 拆入 `structure_convention/test_file_budget/status_output_expected_slices/legacy_maps/guard_body/budgets.rs`、`structure_convention/test_file_budget/status_output_expected_slices/legacy_maps/guard_body/folder_backed.rs`、`structure_convention/test_file_budget/status_output_expected_slices/legacy_maps/guard_body/legacy_routes.rs`、`structure_convention/test_file_budget/status_output_expected_slices/legacy_maps/guard_body/paths.rs` 与 `structure_convention/test_file_budget/status_output_expected_slices/legacy_maps/guard_body/status_mirrors.rs`，并由 `runtime_15_status_output_expected_slice_legacy_guard_body_is_folder_backed` 锁定无旧式回流；Cargo gate deferred。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 module-layout guard body 镜像

`Runtime 15 M3 expected-slice module-layout guard body folder-backed split` / `runtime_15_expected_slice_module_layout_guard_body_folder_backed_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_expected_slice_module_layout_guard_body_folder_backed_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/status-output 测试守卫 owner：`structure_convention/test_file_budget/status_output_expected_slices/module_layout/guard_body.rs` 保持 route owner，checks 拆入 `structure_convention/test_file_budget/status_output_expected_slices/module_layout/guard_body/budgets.rs`、`structure_convention/test_file_budget/status_output_expected_slices/module_layout/guard_body/child_ownership.rs`、`structure_convention/test_file_budget/status_output_expected_slices/module_layout/guard_body/folder_backed.rs`、`structure_convention/test_file_budget/status_output_expected_slices/module_layout/guard_body/paths.rs`、`structure_convention/test_file_budget/status_output_expected_slices/module_layout/guard_body/route_mounts.rs` 与 `structure_convention/test_file_budget/status_output_expected_slices/module_layout/guard_body/status_mirrors.rs`，并由 `runtime_15_expected_slice_module_layout_guard_body_is_folder_backed` 锁定无旧式回流；Cargo gate deferred。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 guard maps 镜像

`Runtime 15 M3 status-output expected-slice guard maps folder-backed split` / `runtime_15_status_output_expected_slice_guard_maps_folder_backed_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_status_output_expected_slice_guard_maps_folder_backed_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/status-output 测试守卫 owner：`structure_convention/test_file_budget/status_output_expected_slices/maps.rs` 保持 route owner，checks 拆入 `structure_convention/test_file_budget/status_output_expected_slices/maps/guard_body.rs`、`structure_convention/test_file_budget/status_output_expected_slices/maps/guard_body/budgets.rs`、`structure_convention/test_file_budget/status_output_expected_slices/maps/guard_body/child_ownership.rs`、`structure_convention/test_file_budget/status_output_expected_slices/maps/guard_body/folder_backed.rs`、`structure_convention/test_file_budget/status_output_expected_slices/maps/guard_body/paths.rs`、`structure_convention/test_file_budget/status_output_expected_slices/maps/guard_body/route_mounts.rs` 与 `structure_convention/test_file_budget/status_output_expected_slices/maps/guard_body/status_mirrors.rs`，并由 `runtime_15_status_output_expected_slice_guard_maps_is_folder_backed` 锁定无旧式回流；Cargo gate deferred。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 foundation expected-slice maps guard 镜像

`Runtime 15 M3 foundation expected-slice maps folder-backed split` / `runtime_15_foundation_expected_slice_maps_folder_backed_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_foundation_expected_slice_maps_folder_backed_static_passed_cargo_deferred`：`runtime_15/foundation.rs` 为 route owner，`runtime_15/foundation/lock_poison.rs` 等 child maps 承载具体 status/date entries，guard 为 `runtime_15_foundation_expected_slice_maps_are_folder_backed`。

`Runtime 15 M3 foundation expected-slice maps guard folder-backed split` / `runtime_15_foundation_expected_slice_maps_guard_folder_backed_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_foundation_expected_slice_maps_guard_folder_backed_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/status-output 测试守卫 owner：`structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/foundation.rs` 保持 route owner，checks 拆入 `runtime_15_expected_slice_maps/foundation/{budgets,child_sources,folder_backed,paths,route_mounts,status_mirrors}.rs`，其中 `runtime_15_expected_slice_maps/foundation/status_mirrors.rs` 同步 status row、Runtime 15/index/review/structure/module/session docs 与 Frameworks 02 mirrors。新增 `runtime_15_foundation_expected_slice_maps_guard_is_folder_backed` 锁定无旧式回流；Cargo gate deferred。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 naming-boundary render-graphics map rows guard 镜像

`Runtime 15 M3 naming-boundary render-graphics expected-slice map rows folder-backed split` / `runtime_15_naming_boundary_render_graphics_expected_slice_map_rows_folder_backed_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_naming_boundary_render_graphics_expected_slice_map_rows_folder_backed_static_passed_cargo_deferred`：`naming_boundary/render_graphics.rs` 为 route owner，`naming_boundary/render_graphics/expected_slice_rows.rs` 等 child maps 承载具体 status/date entries，guard 为 `runtime_15_status_output_naming_boundary_render_graphics_map_rows_are_folder_backed`。

`Runtime 15 M3 naming-boundary render-graphics map rows guard folder-backed split` / `runtime_15_naming_boundary_render_graphics_map_rows_guard_folder_backed_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_naming_boundary_render_graphics_map_rows_guard_folder_backed_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/status-output 测试守卫 owner：`structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/render_graphics_map_rows.rs` 保持 route owner，checks 拆入 `runtime_15_expected_slice_maps/naming_boundary/render_graphics_map_rows/{budgets,folder_backed,paths,route_mounts,status_mirrors,status_rows}.rs`，其中 `runtime_15_expected_slice_maps/naming_boundary/render_graphics_map_rows/status_mirrors.rs` 同步 status row、Runtime 15/index/review/structure/module/session docs 与 Frameworks 02 mirrors。新增 `runtime_15_status_output_naming_boundary_render_graphics_map_rows_guard_is_folder_backed` 锁定无旧式回流；Cargo gate deferred。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 foundation status mirrors 镜像

`Runtime 15 M3 foundation expected-slice maps status mirrors folder-backed split` / `runtime_15_foundation_expected_slice_maps_status_mirrors_folder_backed_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_foundation_expected_slice_maps_status_mirrors_folder_backed_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/status-output 测试守卫 owner：`structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/foundation/status_mirrors.rs` 保持 route owner，checks 拆入 `runtime_15_expected_slice_maps/foundation/status_mirrors/{budgets,docs,folder_backed,paths,row_data}.rs`，其中 `runtime_15_expected_slice_maps/foundation/status_mirrors/row_data.rs` 同步 status row 与 status/date maps，`docs.rs` 同步 Runtime 15/index/review/structure/module/session docs 与 Frameworks 02 mirrors。新增 `runtime_15_foundation_expected_slice_maps_status_mirrors_are_folder_backed` 锁定无旧式回流；Cargo gate deferred。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 naming-boundary sources 镜像

`Runtime 15 M3 naming-boundary expected-slice sources folder-backed split` / `runtime_15_naming_boundary_expected_slice_sources_folder_backed_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_naming_boundary_expected_slice_sources_folder_backed_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/status-output 测试守卫 owner：`structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/naming_boundary/sources.rs` 保持 route owner，checks 与 helpers 拆入 `runtime_15_expected_slice_maps/naming_boundary/sources/{budgets,constants,folder_backed,guard_body,render_graphics,row_sources,status_mirrors,structure_route_maps}.rs`，关键 child anchors 包含 `naming_boundary/sources/constants.rs` 与 `naming_boundary/sources/row_sources.rs`，其中 `runtime_15_expected_slice_maps/naming_boundary/sources/status_mirrors.rs` 同步 status row、Runtime 15/index/review/structure/module/session docs 与 Frameworks 02 mirrors。新增 `runtime_15_status_output_naming_boundary_expected_slice_sources_are_folder_backed` 锁定无旧式回流；Cargo gate deferred。

`Runtime 15 M3 status output Runtime 15 expected-slice child-owner guard sources folder-backed split` / `runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_sources_folder_backed_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_status_output_runtime_15_expected_slice_child_owner_guard_sources_folder_backed_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/status-output 测试守卫 owner：`structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/child_owners/split_layout/sources.rs` 保持 route owner，checks 与 helpers 拆入 `runtime_15_expected_slice_maps/child_owners/split_layout/sources/{budgets,constants,folder_backed,row_sources,status_mirrors,status_support_maps}.rs`，关键 child anchors 包含 `child_owners/split_layout/sources/constants.rs` 与 `child_owners/split_layout/sources/row_sources.rs`，其中 `runtime_15_expected_slice_maps/child_owners/split_layout/sources/status_mirrors.rs` 同步 status row、Runtime 15/index/review/structure/module/session docs 与 Frameworks 02 mirrors。新增 `runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_sources_are_folder_backed` 锁定无旧式回流；Cargo gate deferred。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 typed-error structure assertions 镜像

`Runtime 15 M3 code review findings structure guard typed-error structure assertions folder-backed split` / `runtime_15_code_review_findings_structure_guard_typed_error_structure_assertions_folder_backed_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_code_review_findings_structure_guard_typed_error_structure_assertions_folder_backed_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/code-review/status-output 测试守卫 owner：`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/structure_assertions.rs` 保持 route owner，checks 与 helpers 拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/structure_assertions/source_trees.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/structure_assertions/current_checks.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/structure_assertions/folder_backed.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/structure_assertions/budgets.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/structure_assertions/status_mirrors.rs`。新增 `runtime_15_code_review_findings_structure_guard_typed_error_structure_assertions_guard_is_folder_backed` 锁定 route-only ownership 与文档镜像；Cargo gate deferred。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 P0 native fixture source status-map 镜像

`Runtime 15 M3 P0 native fixture source status-map reconciliation` / `runtime_15_p0_native_fixture_source_status_map_reconciliation_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_p0_native_fixture_source_status_map_reconciliation_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/code-review/status-output 测试守卫 owner：`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners/root_paths.rs` 命名 `review_guard_rows/p0_rows.rs` 与 `foundation_review_maps/p0_rows.rs` child owners，`root_sources.rs`、`root_inventory.rs` 与 `status_mirrors.rs` 读取 child rows/maps，`root_child_rows.rs` 显式保留 `delegation.rs`、`leaf_ownership.rs`、`status_mirrors.rs` 与 `budgets.rs` child-source anchors。关键守卫为 `runtime_15_p0_native_fixture_leaf_owner_guard_is_folder_backed`、`runtime_15_p0_native_fixture_leaf_owner_root_inventory_is_child_owned` 与 `runtime_15_p0_native_fixture_leaf_owner_guard_folder_backed_status_is_current`；Cargo gate deferred。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 plugin-importer DX source status-map 镜像

`Runtime 15 M3 plugin-importer DX source status-map reconciliation` / `runtime_15_plugin_importer_dx_source_status_map_reconciliation_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_plugin_importer_dx_source_status_map_reconciliation_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/code-review/status-output 测试守卫 owner：`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/root_paths.rs` 命名 `review_guard_maps/plugin_importer_maps.rs` status/date child owners，`root_inventory.rs`、`status_mirrors.rs`、`source_inventory.rs`、`status_docs/root_paths.rs`、`structure_assertions.rs`、`structure_assertions/review_mounts.rs` 与 `structure_assertions/d13_sdk.rs` 读取 child maps。关键守卫为 `runtime_15_plugin_importer_dx_structure_guard_is_folder_backed`、`runtime_15_plugin_importer_dx_structure_guard_root_inventory_is_child_owned` 与 `runtime_15_plugin_importer_dx_structure_guard_folder_backed_status_is_current`；Cargo gate deferred。
验证镜像：scoped rustfmt 通过；structure-convention harness 重新编译通过（warning_count=286）；focused `plugin_importer_dx_child_owners` 通过 25/25；plan-status harness 重新编译通过（warning_count=0）；`status_output_tables` 通过 2/2；package/workspace Cargo 未声明通过。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 direct assertions child-source 镜像

`Runtime 15 M3 code review findings direct assertions child-source sync` / `runtime_15_code_review_findings_direct_assertions_child_source_sync_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_direct_assertions_child_source_sync_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/code-review/status-output 测试守卫 owner：`direct_review_assertion_child_source_blob` 读取 direct-assertion nested child source，`structure_guard_children/folder_backed_summary/direct_assertions.rs` 读取 direct-assertion leaf owner。关键守卫为 `runtime_15_code_review_findings_structure_guard_folder_backed_summary_direct_assertions_are_child_owned`、`runtime_15_code_review_findings_direct_assertions_child_ownership_guard_folder_backed_status_is_current` 与 `runtime_15_code_review_findings_direct_assertions_guard_folder_backed_status_is_current`；验证镜像为 `direct_assertions` 27/27、`folder_backed_summary_child_ownership` 3/3、`plugin_importer_dx_child_owners` 25/25、`status_output_tables` 2/2；Cargo gate deferred。

---

## 2026-07-07 Frameworks 02 Runtime 15 M3 typed-error source-inventory helper 镜像

`Runtime 15 M3 typed-error source inventory helper source reconciliation` / `runtime_15_typed_error_source_inventory_helper_source_reconciliation_static_passed_cargo_deferred` 已镜像为 `frameworks_02_m3_typed_error_source_inventory_helper_source_reconciliation_static_passed_cargo_deferred`。该切片只整理 Runtime 15 M3 structure-convention/code-review/status-output 测试守卫 owner：source blob 改为 path-aware，status row/status/date map helpers 下沉到 `source_inventory/metadata/review_guard_paths.rs`，并读取 status-support source-inventory child rows 与 `review_guard_maps/typed_error_maps/source_inventory_rows.rs`。验证镜像为 `typed_error_source_inventory` 17/17 与 `status_output_tables` 2/2；Cargo gate deferred。
