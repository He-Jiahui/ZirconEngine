---
handoff_kind: fixed
status: fixed
closeout_status: accepted
created_at: 2026-07-22
resolved_at: 2026-07-23
summary_slug: asset-migration-single-inventory-generation
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/migration/mod.rs
  - zircon_runtime/src/asset/migration/run.rs
  - zircon_runtime/src/asset/migration/scan.rs
  - zircon_runtime/src/asset/migration/sidecar.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/source_boundary.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib asset::tests::migration::project_commandlet::source_boundary:: --locked --jobs 1 -- --nocapture --test-threads=1
---


# Runtime04：asset migration单一inventory generation缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset migration 17/17逐Rust文件性能审查，PERF-MVP-511
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：project file identity、sidecar/source分类与resolver path authority属于Runtime04，Editor10不能建立第二套扫描结果。
- 生命周期键：`asset-migration-single-inventory-generation`

## 失败现象与复现证据

一次migration依次执行recognized sources、transaction targets、authoring files和sidecar collect，至少四次递归同一roots，前三次逐目录收集并排序children。

## 最低共享层根因

migration没有消费project generation inventory，也没有独立运行时的一次typed walk；source、sidecar、authoring与allowed target由多个scanner重复推导。

## 架构修复验收

- 一次walk将每个entry分类为source/sidecar/authoring/transaction target并记录root-relative physical identity；directory visit/sort≤1/generation。
- 参考Godot `EditorFileSystem`的`filesystem_cache`与`scan_changes/update_file`，但保留Zircon transaction target白名单和严格link/reparse规则。
- focused source-bound suite 必须锁定 physical-alias/overlapping roots 的单次目录访问、current/retired/orphan/prospective sidecar 与 transaction-target inventory 投影、root-relative identity 和 `.zircon` 排除。link/reparse guards 只在成功创建 fixture 的主机上构成行为证据；Windows OS 1314 仅是环境跳过，不能声称已执行 preflight/recovery 全合同。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止仅缓存最后一次resolver结果而保留四套scanner。
- 禁止Editor10或sidecar模块维护第二份inventory authority。

## 批准的非重叠 lifecycle 拆分

2026-07-23 书面拆分获批后，本 lifecycle 只保留已经实现并验证的 single-walk inventory owner。原 PERF-MVP-511 中尚未完成的三个独立合同继续保持 open，不与本 fixed scope 重叠：

- [indexed resolver generation](../../zircon_runtime/runtime/04/failure-2026-07-23-asset-migration-indexed-resolver-generation.md)：`resolver.rs` 的 logical→physical O(1) generation index 与 per-reference filesystem probe=0。
- [single-parse document artifact fixed](fixed-2026-07-23-asset-migration-single-parse-document-artifact.md)：`document.rs`/formal reader 共享一次 typed parse artifact，完整 Value 深 clone=0。
- [scale acceptance matrix](../../zircon_runtime/runtime/04/failure-2026-07-23-asset-migration-scale-acceptance-matrix.md)：1/1k/100k files/dirs/refs、1/4 roots、dry-run/apply/unchanged/1% change 的计数与等价性证据。

## 修复结果与回传

- 根因：Migration run owned four independent recursive projections of the same roots, so source, sidecar, authoring and transaction targets had no single typed inventory authority.
- 架构修复：MigrationInventory now performs one sorted visited-path walk and publishes deduplicated source, sidecar, authoring and transaction-target projections; sidecar preflight consumes that inventory and preserves link/reparse safety.
- 验证：Managed job 3fef58a532ad49ff84fd9c47091653cf / run 157aeb3faaf14e1b95d89402b7482e39 natural released exit0/no PIDs；raw stdout `running 6 tests`，6 passed / 0 failed / 0 ignored / 8862 filtered，0.19s，build 76m11s；其 snapshot 985 只保留为修复前预审证据。随后 broad current-source job 9813f2d5493342ba8d80106b201e53f8 / run 90e3328ac3444c97b2c77e8d8d228923 natural released exit101/no PIDs，暴露 optional asset root 的 Scan NotFound 与已拆分 single-parse lifecycle 的 formal decode RED。最低修复仅跳过 `symlink_metadata` 的 NotFound，权限/reparse/其他 I/O 仍 fail-closed，并新增 missing-root 不重扫现有 root 的 focused test。snapshot 1072 / fingerprint 502f5b61 的 broad managed job 182a54dd88b34c42bbac163c4a694ca5 / run f127015c83a8456492f584ea3f1484ad natural released exit101/no PIDs：43 tests 中 inventory-owned 7 tests 全部 `ok`，22 个失败均位于 separately split single-parse/transaction document chain。最终 exact source-bound reservation 59f3c41220a84d19bc8d844376e140c0 → job 7218a7c923304242b30d27321a59fac4 / run 14a0429bbb2d470c972e1254f7912559 natural released exit0/no PIDs：raw stdout `running 7 tests`，7 passed / 0 failed / 0 ignored / 8873 filtered，0.17s；build 20m16s。独立只读复审 C0/I0/M0，closeout 验收为 accepted。
- 回传：Coordinator failure handoff 已完成 fixed return；r2 exact scope 的 inventory owner 7/7 已由当前源码 exact exit0 gate 证明。Separately split resolver index, single-parse document artifact and scale matrix remain open.
