---
handoff_kind: failure
status: open
created_at: 2026-08-18
summary_slug: missing-subasset-parent-fallback
origin_plan: docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/optimize/zircon_runtime/04
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/reference_resolver.rs
  - zircon_runtime/src/asset/reference_resolution_error.rs
  - zircon_runtime/src/asset/migration/resolver.rs
  - zircon_runtime/src/asset/importer/ingest/import_model.rs
tests:
  - cargo test -p zircon_runtime --lib asset::reference_resolver::tests::resolution_reports_guid_path_repair_dangling_and_conflict_states --locked --jobs 1 -- --exact --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib asset::importer::ingest::import_model::tests::importer_outcome_exposes_complete_guid_repair --locked --jobs 1 -- --exact --nocapture --test-threads=1
---

# Runtime 04：缺失 subasset label 静默回退父资产

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md`
- 来源执行切片：P1-8 missing subasset semantic identity
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：最低共享原因位于 Runtime04 project reference resolver；调用方不能可靠判定被回退的父资产是否仍是原语义目标。

## 失败现象与复现证据

`entry_by_hint` 对缺失 labeled locator 执行 `.or(base_entry)`；GUID 快速路径也会在 persisted sub 与 registry label 不一致时直接返回。现有测试把 `#MissingMesh` 修复为无 label 父资产，导致重新保存后永久改变引用目标。

## 最低共享层根因

resolver 把 GUID/path 身份修复与 subasset 语义修复合并处理，没有要求 resolved registry entry 保持 persisted label，也没有为缺失 label 返回 typed dangling diagnostic 和同源候选。

## 架构修复验收

- stale GUID 仅可修到仍存在的 exact labeled entry，并保留相同 subasset label。
- 缺失 label 在 GUID 与 path-hint 两条路径均返回 typed dangling error，候选稳定列出同源 labeled entries，绝不退回父资产。
- migration 将该错误归类为 dangling reference；原 importer repair reproduction 与 Runtime/Editor/App 批量上行门通过。

## 禁止临时方案

- 禁止 importer/call-site 捕获错误后删除 label、重试父 locator 或伪造 repair。
- 禁止 registry alias、隐式 label rename、测试专用分支或削弱 subasset identity 断言。

## 修复结果与回传

Open state: `validation_pending`; no pass is claimed.
