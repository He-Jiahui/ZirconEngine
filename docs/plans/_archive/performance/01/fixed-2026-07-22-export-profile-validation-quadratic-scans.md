---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: export-profile-validation-quadratic-scans
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/09-export-publishing.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest/feature_selection.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest/profile_projection.rs
  - zircon_runtime/src/plugin/export_build_plan/project_manifest_validation/duplicates.rs
  - zircon_runtime/src/plugin/export_build_plan/project_manifest_validation/identity.rs
tests:
  - 1/100/1000 plugin and feature export validation scaling benchmark
  - export diagnostics order and byte-equivalence test
  - external provider lookup build-count test
resolved_at: 2026-07-22
---


# Plugins09：export profile validation 重复线性扫描

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：MVP plugin export-build-plan 逐文件静态审查
- 修复责任计划：`docs/plans/zircon_plugins/09-export-publishing.md`
- 交接原因：manifest/profile/provider 的索引必须属于单次 export plan generation，不能在每个 validator 内各建临时 cache。

## 失败现象与复现证据

`project_duplicate_selection_diagnostics` 用 `Vec::find` 对 plugin ids 和每个 owner 的 feature ids 查重；
`sanitize_project_identity_rows` 又用独立 Vec 重做相同线性查重。profile diagnostics/projection 对 selected plugin、
owner 和 feature id 多次执行 nested `iter().any/find`；external feature provider 校验还为每个 feature 全扫一次
manifest selections。P 个 packages、每个 F 个 features 时，单次 export plan validation 含多条 O(P²) 与 O(F²)
路径。

本次性能审计已直接消除 feature id normalization 的循环内 String 分配，以及 namespace/identity validator 的
split-segment Vec 与 formatted owner-prefix 分配；剩余问题是共享索引/执行拓扑，不应继续用局部微调掩盖。

## 最低共享层根因

Export build plan 没有一次性的、保持 manifest 首次出现顺序的 validation projection。每个 diagnostic、sanitize、
provider 和 profile consumer 都从原 Vec 重新搜索，造成重复工作且无法记录统一 build-count。

## 架构修复验收

- 单次 export generation 为 source-faithful diagnostics 与 catalog-completed generation 两个不同拓扑视图分别建立
  package id、owner feature id 与 provider package membership 索引；每个视图只构建一次，validator 内不得重建。
- duplicate diagnostics 与 sanitize 共用同一 first-occurrence/duplicate facts，但继续按现有 manifest 顺序发出诊断。
- required/fatal 提升、target-mode filter、无效 id 清理和 external provider 语义逐项不变。
- 1/100/1000 packages × 1/10/100 features 的查重/provider/profile projection 总访问线性增长。
- 新旧 diagnostics 文本、顺序、fatal 分类与最终生成文件做 byte-equivalence 回归。

## 禁止临时方案

- 不得把 manifest Vec 改成 HashMap 后丢失首次出现顺序或重复项诊断。
- 不得给每个 validator 单独建立一份索引，重复解析同一 generation。
- 不得把 export-time 问题描述成 frame 热点；其优先级由大型项目导出预算决定。

## 修复结果与回传

- 根因：Export build planning lacked generation-owned ordered manifest and profile projections, so duplicate, identity, provider, sanitize, and profile consumers repeatedly scanned package and feature vectors with quadratic growth.
- 架构修复：Build exactly one source-faithful and one catalog-completed ProjectPluginManifestValidationProjection per generation plus one ExportProfileSelectionProjection per profile; reuse ordered indexed facts, refresh only completed dynamic facts, and keep all projection APIs private to crate::plugin::export_build_plan with no alias or shim.
- 验证：Managed job f942c000761f44a5996f9b694bff96e0 passed the two-dimensional scale, explicit profile, missing identity, diagnostics, and target mismatch regressions before exposing the test self-match. Current-source successor job 0d99e095587b4e62a563d0a15441e8c3 run 35163227122e4922ace66cbbdd28a5fc passed 1 of 1, exit 0, released with no PIDs; 10570-file pre1, pre2, and post hash all equal 362f488cbcf0a3d81db4f28e4de3ae29174086f72825592bafbb4d52eef3796a. Snapshot974 review C0 I0 M0; rustfmt, diff-check, and Python contracts 7 of 7 passed.
- 回传：Performance01 may resume the export-profile-validation gate as fixed. Frameworks03 Runtime export-plan acceptance is complete; App and Editor consumer focused gates remain pending and do not reopen this performance failure.
