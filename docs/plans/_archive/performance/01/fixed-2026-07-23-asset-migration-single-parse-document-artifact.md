---
handoff_kind: fixed
status: fixed
closeout_status: pending_validation
created_at: 2026-07-23
resolved_at: 2026-07-23
summary_slug: asset-migration-single-parse-document-artifact
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/assets/material/mod.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/asset/assets/project_document.rs
  - zircon_runtime/src/asset/assets/project_document/codec.rs
  - zircon_runtime/src/asset/assets/project_document/material.rs
  - zircon_runtime/src/asset/assets/project_document/model.rs
  - zircon_runtime/src/asset/assets/project_document/scene.rs
  - zircon_runtime/src/asset/migration/document.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/document_migration.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib asset::tests::migration::project_commandlet::document_migration::retired_project_reference_without_subasset_omits_toml_null_and_is_idempotent --locked --jobs 1 -- --nocapture --test-threads=1
---

# Runtime04：asset migration single-parse document artifact

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset migration 性能审查 PERF-MVP-511；经批准从 single-inventory lifecycle 拆分
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：project document codec、formal readers 与 migration artifact 属于 Runtime04；本 lifecycle 不拥有 resolver index 或 scale matrix。
- 生命周期键：`asset-migration-single-parse-document-artifact`
- 本记录仅关闭 shared typed document artifact 与 persisted-reference JSON→TOML 投影；indexed resolver generation 和 scale matrix 继续保持 open。

## 失败现象与复现证据

Managed fresh snapshot 1073 的 broad project-commandlet run 稳定产生 22 个 `invalid type: null` 失败；临时给 18 个下游 authoring DTO 字段添加 `skip_serializing_if` 后失败数完全不变，证明缺陷发生在 formal reader/DTO serialization 之前。

## 最低共享层根因

Migration 已将同一 mutable TOML artifact 交给 formal scene/model/material reader，但 retired reference 的单对象迁移仍把 `PersistedAssetReference` JSON 直接转换为 `toml::Value`。公共 JSON 合同为无 subasset 的 project reference 保留 `"sub": null`，而 TOML 不接受 null，因此失败发生在 formal reader 之前。material/model/scene 下游 authoring DTO 的 `skip_serializing_if` 无法触达该边界。

## 架构修复验收

- `ProjectDocumentArtifact` 成为一次 parse 后的 generation-owned typed artifact；migration 与 formal reader 不再重新 parse String，也不恢复 whole-document JSON clone/scrub。
- `document.rs` 由一个 shared persisted-reference JSON→TOML table helper 同时服务 current 与 retired reference；仅在 `kind = project` 且 `sub = null` 时省略 `sub`。
- `zircon_runtime_interface` 的 `AssetRef` JSON 公共合同保持原样；labeled subasset 的 `sub = "Mesh0"` 继续保留并由 formal reader 重载。
- 已撤回未触达根因的 18 个 DTO 临时 `skip_serializing_if` 属性。

## 修复结果与回传

- 根因：retired reference 的 bounded JSON object 被直接投影为 TOML，公共 project reference JSON 中的 `sub:null` 在 formal reader 之前触发 TOML null 错误。
- 架构修复：`ProjectDocumentArtifact` 保持一次 typed parse；current/retired reference 共用一个 bounded JSON→TOML table helper，只省略 project `sub:null`，不修改 interface JSON contract、不递归清洗 document。
- 验证：RED snapshot 1073 / fingerprint d3c045c5，managed job `e9fda9e4cb9347ef911391292f5f7738` / run `5475bdc5efa3469bb7369161b151fb86` natural released exit101/no PIDs；43 tests 中 21 passed / 22 failed。新增 `retired_project_reference_without_subasset_omits_toml_null_and_is_idempotent`。source-bound snapshot 1075 / fingerprint `4ed783b4a04e10642c7df0bc2f71ec8d9e10247344cd4e468e7155fd727a12a7`，reservation `cc70554659264b72a508dea9059caf35` → job `30d72f6d3b81419598e57dce527c4042` / run `c17327450e4041fa8339632cf97d2666` natural released exit0/no PIDs；raw stdout `running 1 test`，1 passed / 0 failed / 0 ignored / 8879 filtered，0.46s，build 56m58s。后续独立 closeout review 为 C0/I1/M0：public `ZMaterialDocument::to_project_toml_string` 丢失既有 unsupported-version rejection；必须先恢复唯一借用型 version validator 并增加 public serializer regression gate，因此 closeout 保持 pending_validation。
- 回传：Runtime04 canonical return 已写入；indexed resolver generation 与 scale acceptance matrix 继续保持 open，不吸收其路径或验收。

## 禁止临时方案

- 不得修改 `AssetRef` JSON null 合同，不得递归清洗整份 document，不得恢复完整 TOML→JSON→TOML clone。
- 不得增加兼容 shim、fallback parser、test-only bypass 或复制 current/retired reference 投影真相。
