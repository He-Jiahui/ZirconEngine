---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-22
summary_slug: ui-asset-binding-canonical-loader-api-tests
origin_plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
fixing_plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime/74
fixing_child_dir: docs/plans/optimize/zircon_runtime/74
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/template/asset/loader.rs; zircon_runtime/src/ui/tests/asset_binding/compiled_program.rs:438,518
---

# ui-asset-binding-canonical-loader-api-tests: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md`
- 来源执行切片：Render11 Shader06 realtime IBL managed library validation
- 修复责任计划：`docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Render11 Shader06 realtime IBL managed library validation` — Windows managed validate-matrix compilation of zircon_runtime with text_oversized_run_keeps_one_logical_shaped_line reports 13 E0599 errors in zircon_runtime/src/ui/tests/asset_binding/compiled_program.rs because tests call UiAssetLoader::load_str while the canonical loader exposes load_toml_str, plus E0282 at lines 438 and 518 for untyped serialized.try_into().

## 最低共享层根因

The UI asset-binding test suite retained the pre-cutover generic loader spelling and ambiguous conversion inference after the canonical TOML-specific UiAssetLoader contract became the only production entry point.

## 架构修复验收

- Tests use the canonical TOML loader API and explicit compiled-program conversion type where required; no deprecated load_str compatibility method is introduced; the originating managed zircon_runtime validation advances past the reported UI binding E0599 and E0282 errors.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

Open state: `source_updated_static_green_cargo_pending`; the coordinator must keep the validation ticket and route this Plan to its managed validation stage.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-24 | Runtime74 canonical UI test-consumer hard cut | `source_updated_static_green_cargo_pending` | Migrated all 18 `UiAssetLoader::load_str` consumers in `asset_binding/compiled_program.rs` to `load_toml_str`; annotated the two formerly ambiguous deserializations as `UiCompiledBindingProgram`; migrated `default_interaction_schema` from the removed `UiCompiledDocument::root` field to `template_instance().root`; and supplied the current four-argument `UiBindingMutationTransaction::commit` contract to its benchmark consumer. Static contract scan reports legacy loader calls `0`, canonical calls `18`, untyped corrupted-program conversions `0`; exact `rustfmt --check` passed. No Cargo or coordinator receipt was run, so this remains open and cannot be returned as `fixed-*`. |
