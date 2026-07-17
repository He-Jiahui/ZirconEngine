---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_id_report.rs
  - zircon_runtime/src/text/native_buffer.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m4_surface_cleanup.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/ui_text_template.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/renderer_output_accessors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services/plugin_bridge.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/plugin_bridge_table_reports.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_ui_text_font_id_report.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/script_vm_lock_poison.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/root_inventory.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/sources/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/status/root_inventory.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/status/root_sources.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/status/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/d13_sdk/parent_mounts/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/d13_sdk/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/native_plugin_loader/routes/source_helper_status.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/native_plugin_loader/routes/status_current.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/native_plugin_loader/source_helper_status.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/native_plugin_loader/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_plugin_static_manifest.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/native_plugin_loader.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/child_split_status.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/runtime_plugin_catalog_features.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/runtime_plugin_lifecycle.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/runtime_plugin_package_manifest.rs
  - docs/zircon_runtime/graphics/text.md
  - docs/zircon_runtime/ui/text.md
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_id_report.rs
  - zircon_runtime/src/text/mod.rs
  - zircon_runtime/src/text/native_buffer.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_ui_text_font_id_report.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
tests:
  - cargo test -p zircon_runtime --lib text_font --locked --jobs 1 --color never -- --test-threads=1
  - python tools/tests/test_runtime_plan_status_canonical_archive_sources.py
  - 'powershell -NoProfile -Command "$raw = .\tools\zircon-session.ps1 session show --session-id runtime15-archive-consumer-hardcut-20260717 | ConvertFrom-Json; $rust = @($raw.session.write_scope | Where-Object { $_.EndsWith(".rs") }); rustfmt --edition 2021 --check @rust"'
  - 'powershell -NoProfile -Command "$raw = .\tools\zircon-session.ps1 session show --session-id runtime15-archive-consumer-hardcut-20260717 | ConvertFrom-Json; $scope = @($raw.session.write_scope); git diff --check -- @scope"'
doc_type: milestone-detail
status_anchor: runtime_15_screen_space_ui_text_font_id_report_owner_hard_cut_managed_current_source_passed
---

# Runtime 15 Screen-Space UI Text Font-ID Report Owner Hard Cut

Plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
Milestone: M4.1
Status: completed
Files: ["docs/plans/zircon_runtime/runtime/15/2026-07-17-screen-space-ui-text-font-id-report-mount-drift-return.md","docs/plans/zircon_runtime/text/01/fixed-2026-07-17-screen-space-ui-text-font-id-report-mount-drift.md","docs/zircon_runtime/graphics/text.md","docs/zircon_runtime/ui/text.md","zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs","zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_id_report.rs","zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m4_surface_cleanup.rs","zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/ui_text_template.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/renderer_output_accessors.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services/plugin_bridge.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/plugin_bridge_table_reports.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_ui_text_font_id_report.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/script_vm_lock_poison.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/root_inventory.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/sources/status_mirrors.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/status_mirrors.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/status/root_inventory.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/status/root_sources.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/status/status_mirrors.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/d13_sdk/parent_mounts/status_mirrors.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/d13_sdk/status_mirrors.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts/status_mirrors.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/status_mirrors.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer/status_mirrors.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/native_plugin_loader/routes/source_helper_status.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/native_plugin_loader/routes/status_current.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/native_plugin_loader/source_helper_status.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/native_plugin_loader/status_mirrors.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_plugin_static_manifest.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/native_plugin_loader.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/child_split_status.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/status_mirrors.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/runtime_plugin_catalog_features.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/runtime_plugin_lifecycle.rs","zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/runtime_plugin_package_manifest.rs","zircon_runtime/src/text/mod.rs","zircon_runtime/src/text/native_buffer.rs","zircon_runtime/src/text/render_state.rs"]

## 状态与完成项目

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M4.1 | screen-space UI text font-id report 单 owner 与归档消费端硬切 | `runtime_15_screen_space_ui_text_font_id_report_owner_hard_cut_managed_current_source_passed` | 2026-07-17 | Windows 受管 job `c263b6875a644947a390abb6ba5b8203` / run `dd1c6d7b32824d59a10b8696e4ef6a33` 通过 `text_font` 47/47；独立复审 Critical 0 / Important 0 / Minor 0；scoped rustfmt、diff-check 与 canonical archive source 3/3 通过。 |

## Scope Delivered

- `NativeTextBuffer` 只保留 shaping 同源的 `primary_face`；`native_font_query` 是 family、weight、style、stretch 的权威 match query，shaping request 复用其 weight/style，code Monospace、request family 与 normal stretch 保持显式一致默认值，不再维护并行 font-id DTO。
- `font_id_report.rs` 成为 fallback/unmapped 统计的唯一 owner，直接遍历 backend `layout_runs` 的实际 glyph face id；`primary_face=None` 时仍完整统计，不用 family-name 或样式推断伪造结果。
- `text.rs` 显式挂载并调用 `accumulate_text_font_id_report`，父文件只负责 orchestration；旧 `NativeTextFontIdReport`、旧 `font_id_report` 方法、根 re-export 与重复累计路径全部删除，不保留 alias、shim 或兼容转发。
- `render_state.rs` 仅以 `pub(crate)` 暴露实际 render-framework stats consumer 所需的 font database；文档同步真实可见性与调用链。
- Runtime15 结构守卫锁定 parent mount、单 query、实际 glyph 计数、默认字体链、父/子文件预算及旧 owner 零回流。
- 27 个历史 plan-status 消费端硬切到 Runtime15 canonical archive record；唯一保留的 current-plan consumer 是仍由活跃父计划拥有的 folder-backed anchor 守卫。

## Fresh Testing Evidence

- 受管命令：`cargo test -p zircon_runtime --lib text_font --locked --jobs 1 --color never -- --test-threads=1`，47 passed / 0 failed / 8161 filtered out，exit 0；同次包含 `runtime_15_screen_space_ui_text_font_id_report_is_child_owner` 通过。
- `python tools/tests/test_runtime_plan_status_canonical_archive_sources.py`：3/3 通过。
- 当前 M4.1 manifest 共 39 个文件；其中 Rust 文件 `rustfmt --check` 通过，scoped `git diff --check` 通过，共享 Git index 为 0。
- 独立只读复审结论：Critical 0、Important 0、Minor 0；36 个既有文件全部为 modified-in-place，无 rename 或 mode change。
- `test_runtime_plan_status_archive_ownership.py` 的实现消费端已通过；Runtime15 `last_refined` 已由 maintenance commit `bd8b746dc5a661c24c6b2e70e2a4139c9e83905d` 闭合，当前剩余唯一报告是 foreign Runtime12 父计划的日期漂移，不把外部计划阻塞伪报为本代码切片失败。

## Review

- 独立只读复审在修正机器可读 manifest、query/attrs 表述与 archive ownership 当前漂移后通过：Critical 0、Important 0、Minor 0。

## 跨计划失败归还状态

Text01 交接的 `screen-space-ui-text-font-id-report-mount-drift` 已由 47/47 证明闭合，并通过协调器以 `child_record_only` 模式完成归还：canonical fixed artifact 为 [Text01 fixed record](../../text/01/fixed-2026-07-17-screen-space-ui-text-font-id-report-mount-drift.md)，Runtime15 仅保留 [return receipt](2026-07-17-screen-space-ui-text-font-id-report-mount-drift-return.md)。两个父计划没有被 failure return 重复改写。

## 父计划状态

Runtime15 父计划继续保持 `in_progress`。本切片完成 M4 的 text font-id owner 与 canonical archive consumer 硬切，不冒充 M1-M5 全计划完成；后续仍按计划推进剩余 prelude、dead-code 与聚合门禁。
