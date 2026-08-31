---
handoff_kind: failure
status: open
created_at: 2026-08-05
updated_at: 2026-08-27
summary_slug: import-diagnostics-editor-log-source-bridge
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/09
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/asset/import_flow
  - zircon_editor/src/core/asset/import_flow/diagnostics.rs
  - zircon_editor/src/core/asset/import_flow/submit.rs
  - zircon_editor/src/core/asset/import_flow/tests/diagnostics.rs
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_editor/src/core/logging
  - tools/tests/test_editor09_import_diagnostics_submission_contract.py
tests:
  - python tools/tests/test_editor09_import_diagnostics_submission_contract.py
  - import completion failure and warning projection
  - repeated import completion cursor/dedup contract
  - asset jump target and canonical LogSource import channel
---

# Editor09: import diagnostics are not projected into EditorLog

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：M3.1 six-source logging aggregation.
- 修复责任计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 交接原因：import admission, completion identity and asset jump data belong to the Editor09 import/asset manager boundary; Editor17 cannot reconstruct them from worker-pool counters.

## 失败现象与复现证据

The import pipeline and editor asset manager own completion and failure facts, but production code does not publish them to `EditorLogService`. Import outcomes are consequently unavailable to the typed Activity log filter/jump model, despite `LogSource::import()` already existing in the canonical core.

## 最低共享层根因

Editor09 owns import completion identity and asset jump data, but its producer boundary has no cursor-based projection to the editor-owned log service.

## 架构修复验收

- Editor09 projects each new import warning/error/completion through `EditorLogService` with `LogSource::import()`, preserving typed asset URI or document jump data when it exists.
- The producer is bounded and cursor-based over its own completion identity; repeated frame observation and snapshot resync cannot multiply identical records.
- Import job Progress and log records remain separate canonical projections: the producer must not manufacture a private history, toast queue or second completion receiver.
- Tests cover success, warning, terminal failure, duplicate observation, asset jump dispatch and bounded retention under a completion storm.

## 禁止临时方案

- Do not parse worker-pool diagnostic text in retained UI, write logs directly from Runtime04, or add a duplicate import diagnostics store.
- Do not use `LogSource::editor()` for import facts or omit their existing typed jump identity.

## 修复结果与回传

Open state: `source_bridge_complete / submission_transaction_complete / managed_validation_and_fixed_return_pending`. Editor17 still does not own asset/import code; Editor09 focused Cargo and the upward Editor17 Activity-log validation remain pending, so this canonical artifact stays open.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-27 | Editor09 import terminal -> canonical EditorLog source bridge | `source complete / managed validation pending` | Current-source recheck confirms the production model ingress now routes `RetainedEditorHost -> EditorAssetManager -> EditorAssetImportFlow`, and the flow owns the canonical `EditorLogService`. Success, not-imported warning, typed terminal failure, model cancellation/panic, duplicate result observation, bounded storm retention and asset jump projection already use `LogSource::import()` without a private UI history. The remaining admission race was hard-cut with one shared `DeferredSubmissionDiagnostic<T>` state machine: asset/model jobs remain pending until job admission arms them; a completion before arm is deferred; a rejected submission replaces the pre-arm Drop cancellation with the real `JobSubmitError`; emitted terminals are idempotent. Added the asset rejection regression proving exactly one `result=rejected` error with `LogJumpTarget::Asset`, plus a Python RED 1/2 -> GREEN 2/2 source contract; combined Editor09 static contracts are 6/6, rustfmt and scoped `git diff --check` pass, and all four touched owners remain below 800 lines (274/202/175/326). Cargo and product Activity-log validation were intentionally not started while managed validation is occupied; this row is not fixed/accepted. |
| 2026-08-05 | Editor09 import diagnostics -> Editor17 M3.1 source bridge | `open_handoff_recorded` | `LogSource::import()` exists but current production searches find no import producer calling the canonical log service. The handoff records the exact owner boundary and required bounded projection without changing import sources. |
| 2026-08-23 | Editor09 M2 current-source architecture re-audit | `open / hard-cut-required` | `EditorAssetImportFlow` and `EditorAssetIndex` have no production construction site; `DefaultEditorAssetManager` maintains a separate UI-host catalog projection. Adding an isolated log bridge would be dormant scaffolding and preserve duplicate asset state. The corrective slice must first hard-cut the catalog/index ownership and route production import entry points through the sole flow, then project completion identity to `EditorLogService`; no compatibility fallback or second completion store is permitted. |
| 2026-08-23 | Editor09 M2 retained-host ingress audit | `open / hard-cut-required` | `RetainedEditorHost::import_model_into_project` and `default_project_material_id` still invoke runtime `AssetManager::import_asset` directly, then synchronously refresh the workspace. This is the concrete old ingress that must migrate to the single Editor09 job/index owner. Unreal `FAssetData` confirms the target separation: registry records are query data, while import work is not authored by browser rendering consumers. |
