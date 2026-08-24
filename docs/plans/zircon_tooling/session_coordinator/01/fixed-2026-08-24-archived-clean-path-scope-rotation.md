---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-24
summary_slug: archived-clean-path-scope-rotation
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/ownership_transfers.py
  - tools/session_coordinator/tests/test_ownership_transfers.py
resolved_at: 2026-08-24
---

# archived-clean-path-scope-rotation: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Render17 isolated-patch fixed artifact canonical relocation and Frameworks01 clean baseline scope rotation
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Render17 isolated-patch fixed artifact canonical relocation and Frameworks01 clean baseline scope rotation` — ownership transfer preview rejects an exact path attributed to an archived Session when currentHash equals baselineHash, so no executable successor can claim the clean committed path even though the prior owner is terminal and has no live lease

## 最低共享层根因

path_matches_baseline is treated as an unconditional transfer blocker instead of distinguishing an archived attributed owner from an unowned clean path

## 架构修复验收

- Allow transfer of a clean baseline path only when durable attribution names an archived source Session, its attributed hash equals the current/baseline hash, and no foreign live lease overlaps
- Keep unowned clean baseline paths, executable source owners, and foreign live leases ineligible
- Revalidate baseline/current/source identity transactionally and prove preview-to-apply drift fails closed
- Use the production repair to relocate the Render17 fixed artifact and update its return receipt without direct Git or index operations

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：Ownership transfer treated every path matching the baseline as ineligible, even when an archived Session held an exact durable attribution and no live lease existed.
- 架构修复：Permit clean-baseline transfer only for archived exact attribution, preserve all unowned, executable-owner, stale-hash, and foreign-lease rejections, and revalidate the full preview transactionally before apply.
- 验证：Ownership transfer suite 12/12; production archived/future previews and applies succeeded; Render17 relocation commit 9a217cce07c574cbec8dda70b3e1142eeedbc9a9 contains exactly three paths; rollover 74565f47ff4947019f707f1a06020d84 restored the 19-path shared index and cleared mutex/lock/Cargo state.
- 回传：Coordinator01 may consume archived exact-attribution paths through the official transfer preview/apply protocol; the Render17 fixed record now lives in its canonical origin child directory.
