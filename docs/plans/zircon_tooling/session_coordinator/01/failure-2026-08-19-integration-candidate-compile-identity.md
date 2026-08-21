---
handoff_kind: failure
status: open
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

- RED：`test_submit_rejects_passed_ticket_for_a_different_source_manifest` 在修复前失败；manifest 为 `{}` 的 passed ticket 仍成功创建 `integration_ready` candidate。
- 修复：submit 先验证 durable manifest JSON 及其 canonical SHA-256、精确 path set、非 tombstone 64 位小写 SHA-256，再一次性读取全部候选字节。candidate blob 只从这批已验证字节创建，禁止再次读取 live path。
- Windows Git filter：同一验证字节通过 `git hash-object -w --stdin --path=<candidate>` 密封，使 CRLF/attributes clean filter 与后续 tree/index 语义一致；最初裸 `--stdin` 导致的两个 delayed-merge 回归已由 focused RED/GREEN 修复。
- Durable identity：candidate 行继续以外键持有 `compile_ticket_id`，`integration.candidate_submitted` event 新增 `sourceManifestHash`；terminal ticket 不能在被 candidate 引用时删除。
- GREEN：`python -m unittest tools.session_coordinator.tests.test_integration_candidates -v` 为 11/11，覆盖 malformed、path-set mismatch、tombstone、passed 后 byte drift、成功 blob/event identity、finalize、delayed merge、index-lock recovery 和通知幂等。
- 兼容迁移：上一版 `test_server.py` 已随 validation-result candidate `45caf6d7769f4159a278f546adcd5b10` 密封；server candidate 集成用例现按真实 worktree bytes 计算 SHA-256，不再用 `"a" * 64` 伪造 compile identity。
- GREEN：`python -m unittest tools.session_coordinator.tests.test_integration_candidates tools.session_coordinator.tests.test_server.ServerTests.test_compile_ticket_can_finalize_a_sealed_candidate_through_the_coordinator -v` 为 12/12，覆盖严格 manifest identity 与 server 端 sealed-candidate finalize。受管 ticket/candidate/commit 完成前保持 `status: open`。
