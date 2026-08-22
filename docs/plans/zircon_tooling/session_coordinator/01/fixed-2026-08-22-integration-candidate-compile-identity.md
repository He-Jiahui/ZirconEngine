---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-19
summary_slug: integration-candidate-compile-identity
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/integration_candidates.py
  - tools/session_coordinator/tests/test_integration_candidates.py
  - tools/session_coordinator/tests/test_server.py
resolved_at: 2026-08-22
---


# integration-candidate-compile-identity: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Integration candidate compile trust chain
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Integration candidate compile trust chain` — Create a passed validation ticket whose source_manifest_json is empty or hashes different bytes, then call IntegrationCandidateService.submit for a live leased file; observe integration_ready with a newly written Git blob.

## 最低共享层根因

IntegrationCandidateService validates only compile ticket owner and passed status, then hashes live worktree paths independently without comparing the ticket source manifest to the exact bytes sealed into candidate blobs.

## 架构修复验收

- Candidate submission requires the compile ticket source manifest path set to exactly equal the candidate path set.
- Every sealed blob is produced from the same byte buffer whose SHA-256 matches the immutable compile ticket manifest.
- Path, hash, tombstone, and malformed-manifest mismatches fail before candidate rows, blobs, events, or notifications are created.
- Durable candidate evidence records the bound compile ticket and source manifest hash.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：Candidate submission trusted only a passed ticket status and independently reread live paths, so sealed Git blobs were not bound to the exact compile-tested bytes.
- 架构修复：Require canonical source-manifest hash and exact candidate path set, read each candidate once, compare SHA-256 to the ticket, and create Git-filter-aware blobs only from those verified bytes while durably recording the ticket and manifest identity.
- 验证：Managed ticket c03cf7f8f5f14b9c94ec75d134bd6231 passed 12 focused compile-identity and server finalize tests for manifest 662a2f41eec204e43770cdfaea3d75bf0d1ee2b66c75284ec25da6000d1d0e8a; integration_candidates.py and its tests remain byte-identical. Current test_server.py was additionally validated in full by ticket 8207f4f1e5464f499d055ccb169400ab, 130/130.
- 回传：Integration candidates now seal only the exact bytes named by their passed compile ticket and persist the bound source-manifest identity.
