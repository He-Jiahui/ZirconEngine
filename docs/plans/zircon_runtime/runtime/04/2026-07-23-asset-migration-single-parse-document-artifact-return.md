---
record_kind: failure_return_status
status: fixed
closeout_status: pending_validation
resolved_at: 2026-07-23
summary_slug: asset-migration-single-parse-document-artifact
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
plan_link_mode: child_record_only
---

# asset-migration-single-parse-document-artifact 回传摘要

- 状态：最低 shared document conversion owner 已修复，但独立 closeout review 发现 public serializer version regression；failure return 暂为 `fixed`，closeout 保持 `pending_validation`。
- RED：managed job `e9fda9e4cb9347ef911391292f5f7738` / run `5475bdc5efa3469bb7369161b151fb86` natural released exit101/no PIDs；43 tests 中 21 passed / 22 failed，真实根因是 retired project reference JSON 的 `sub:null` 直接进入 TOML conversion。
- 修复：current/retired reference 共用一个有界 JSON→TOML table helper，只省略 project `sub:null`；一次 typed document artifact、public AssetRef JSON contract、labeled subasset 和 formal reader 行为保持不变；无效 DTO patch 已撤回。
- 复审：null omission、public AssetRef JSON、labeled subasset 与 DTO rollback 均通过；最终 closeout review 为 C0/I1/M0，唯一 Important 是 `ZMaterialDocument::to_project_toml_string` 不再拒绝公开构造的 unsupported version。
- 现有证据：snapshot 1075 / fingerprint `4ed783b4a04e10642c7df0bc2f71ec8d9e10247344cd4e468e7155fd727a12a7` 的 focused reservation `cc70554659264b72a508dea9059caf35` → job `30d72f6d3b81419598e57dce527c4042` / run `c17327450e4041fa8339632cf97d2666` natural released exit0/no PIDs；raw stdout `running 1 test`，1 passed / 0 failed / 0 ignored / 8879 filtered，0.46s，build 56m58s。该测试只证明 null omission，不能关闭 version regression；修复后必须 fresh snapshot + public serializer focused gate。
- 回传工件：[fixed-2026-07-23-asset-migration-single-parse-document-artifact.md](../../../performance/01/fixed-2026-07-23-asset-migration-single-parse-document-artifact.md)
