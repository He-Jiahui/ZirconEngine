---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-13
summary_slug: failure-closeout-proof-only-state-attribution-deadlock
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/tests/test_failure_closeout.py
  - tools/session_coordinator/tests/test_git_finalize.py
resolved_at: 2026-08-13
---


# failure-closeout-proof-only-state-attribution-deadlock: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：combined Failure closeout exact-manifest finalize
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`combined Failure closeout exact-manifest finalize` — Combined closeout 417433b3adc1415bbe256b75b6c45a81 prepared and accepted snapshot 1673 including two unchanged .codex/state cargo stderr provenance paths; commit request 1319fa0c667d447c946881e77c06c5f5 failed before index mutation with finalize_unattributed_path, while lease.claim correctly rejects Coordinator state.

## 最低共享层根因

GitFinalizeService requires attribution and live lease for every normalized closeout path before _require_owned_scope filters unchanged proof-only paths; immutable Coordinator state evidence can therefore be validated by snapshot but can never satisfy finalize admission.

## 架构修复验收

- Failure closeout keeps exact snapshot/precommit verification for unchanged proof-only evidence while requiring attribution and lease only for paths that materially differ from HEAD and enter the commit tree.
- Ignored Coordinator state remains forbidden from staging/commit; dirty, tampered, or non-proof unowned paths remain fail-closed.
- Add RED/GREEN regression using an unchanged ignored state-log proof path plus a dirty unowned rejection, then rerun failure closeout and git finalize suites.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：GitFinalizeService applied attribution and live-lease gates to the full immutable Failure proof manifest before filtering proof-only paths, although Coordinator state is intentionally unleaseable and must never enter Git.
- 架构修复：Failure closeout now partitions immutable Coordinator state from committable paths under the Git mutex: full paths remain lifecycle/snapshot evidence, only material committable paths require attribution and leases or enter the index, and Coordinator-state proof requires the under-mutex precommit snapshot guard.
- 验证：RED reproduced finalize_unattributed_path for an ignored .codex/state/session-coordinator stderr proof; GREEN: test_failure_closeout 20/20 and test_git_finalize 75/75 with ResourceWarning promoted to error. Independent review then exposed an O(P) repeated repository scan under the Git mutex; the scan-count regression failed at 14 versus 2 calls before the result was cached and passed with all four focused closeout gates.
- 回传：Failure closeout can commit exact lifecycle evidence containing immutable Coordinator state without leasing or staging that state; state tamper and ordinary unattributed dirty paths remain fail-closed.
