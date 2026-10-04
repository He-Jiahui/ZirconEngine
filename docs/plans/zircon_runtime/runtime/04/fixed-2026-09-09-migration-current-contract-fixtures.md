---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-09-09
summary_slug: migration-current-contract-fixtures
origin_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_runtime/runtime/04
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/tests/migration/project_commandlet/mod.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/command_flow.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/scale_acceptance.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/crash_windows.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/source_boundary.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/document_migration.rs
resolved_at: 2026-09-09
---

# migration-current-contract-fixtures: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 来源执行切片：Runtime04 managed migration acceptance on immutable input 3325
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Runtime04 managed migration acceptance on immutable input 3325` — Managed job 75e6b363944e4617b802b9064e274677: cargo test -p zircon_runtime --no-default-features --locked --lib asset::tests::migration; 63 passed, 8 failed, 1 ignored. Input manifest 9d5e59424e1729c118aa250bd929eea1f7b918871bc5eb54504e4ad61c7243a3.

## 最低共享层根因

Migration tests compare operational Windows paths against display-only report paths, assume retired basename journal identity, expect unregistered GUID rebinding, and assume the graphics-only WGSL matcher in no-default-feature validation.

## 架构修复验收

- Preserve current GUID authority and all original migration failure and idempotence boundaries; update fixtures to current production contracts.
- The exact asset::tests::migration filter executes all non-ignored tests and passes on the frozen source; preserve original failure evidence and separately retain scale acceptance.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：Eight migration fixtures retained obsolete operational/display path comparisons, basename-based journal identity, unregistered GUID rebinding, or a graphics-only source matcher in no-default validation. Original job 75e6b363944e4617b802b9064e274677 was 63 passed, 8 failed, 1 ignored.
- 架构修复：Snapshot 3329 updates six commandlet test files to the existing production contracts: ProjectPaths display assertions, opaque journal owner tokens, always-registered JSON discovery, same-GUID hint repair, missing-GUID rejection, and legal reference migration alongside independent sidecar minting. Production GUID authority, transaction recovery and idempotence boundaries are preserved.
- 验证：Managed Windows job 9aa5c29f3f6640a1808d55c69973254f executed cargo test -p zircon_runtime --no-default-features --locked --lib asset::tests::migration on immutable input runtime04-migration-fixtures-3329-20260909, manifest 3d03d4e4fdb130c29e772a3fe2d08d29459a5e3737f0a24fa41ef932b42fe644: 71 passed, 0 failed, 1 ignored, exit 0. All six fixture hashes match snapshot 3329. Exact command, managed receipt and test names are retained in results/runtime04-migration-3329-r2.json and its sibling log.
- 回传：All non-ignored tests and the original eight failing cases passed. The ignored 1/1k/100k workload remains owned by asset-migration-scale-acceptance-matrix. Return this local fixture lifecycle for canonical independent C0/I0/M0 review; Git closeout and coordinator WeCom delivery remain pending.
