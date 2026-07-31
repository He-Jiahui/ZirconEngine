---
handoff_kind: failure
status: open
created_at: 2026-07-24
summary_slug: untracked-deleted-failure-closeout-finalize
origin_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/runtime/04
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/workflows/failure_closeouts.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/tests/test_failure_closeout.py
  - tools/session_coordinator/tests/test_git_finalize.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_failure_closeout tools.session_coordinator.tests.test_git_finalize
---

# Session Coordinator 01：未跟踪 failure 删除墓碑无法完成原子 closeout

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 来源执行切片：asset migration single-inventory generation failure return / managed exact commit
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Runtime04 exact current-source 验证与独立复审均已 accepted；最低失败位于 failure closeout 对“从未进入 HEAD、当前也不存在”的 source failure artifact 与通用 Git finalizer dirty-path 合同之间。

## 失败现象与复现证据

Runtime04 lifecycle `asset-migration-single-inventory-generation` 已生成 exact 8-path closeout，其中 prior source artifact
`docs/plans/zircon_runtime/runtime/04/failure-2026-07-22-asset-migration-single-inventory-generation.md`
按 failure return 合同作为 `null` 删除墓碑保留在 manifest。该路径从未进入 Git `HEAD`，当前工作树也不存在；
`git ls-files --error-unmatch` 明确返回 pathspec 不属于任何 tracked file。

- 最终 source-bound reservation `59f3c41220a84d19bc8d844376e140c0` → job `7218a7c923304242b30d27321a59fac4` / run `14a0429bbb2d470c972e1254f7912559` natural released exit 0/no PIDs；raw stdout `running 7 tests`，7 passed / 0 failed / 0 ignored / 8873 filtered，0.17s，build 20m16s。
- closeout `fbd31041849d4f2a875d1c11f36db7a0`，fingerprint `b57bb5a86d1a85056203b490d18351152fe0ab63885835cf405934f889554eb1`；validation evidence `0b07e47ed7a64fa1b9472ca663c74e5d` accepted；independent review `59d3cdfaaf3c43ee8760bfd28ecd299e` accepted，C0/I0/M0。
- 第一次 managed commit 被 `finalize_unattributed_path` 拒绝；对 exact 8-path current bytes 执行 canonical `baseline attribute` 后返回 `status: attributed`。
- 第二次 managed commit 被 `finalize_path_unchanged` 拒绝，唯一 reported path 即上述未跟踪且已不存在的 source failure artifact。未执行手工 `git add`、intent-to-add、伪造文件或 index 绕过；共享 staged count 保持 0。

## 最低共享层根因

`FailureCloseoutService.prepare` 正确要求 prior child-record artifact 必须出现在 exact snapshot 中且 hash 为 `null`；但
`GitFinalizeService` 把所有 approved path 都要求为“相对 HEAD 有 workspace change”。对从未 tracked 的临时 source
failure artifact，合法删除墓碑既不能相对 HEAD dirty，也不能从 index 产生删除项，因此同一条路径同时被前者强制纳入、
又被后者以 `finalize_path_unchanged` 禁止，形成不可满足的原子 closeout 合同。

## 架构修复验收

- failure closeout finalizer 能识别 prepare 已证明的 source-artifact `null` 墓碑；当该 artifact 从未进入 HEAD 且当前不存在时，允许它作为生命周期完整性证据参与 exact manifest，但不要求产生 Git index 删除项。
- 该例外只能来自 authoritative failure closeout material 中的 prior source artifact；普通 milestone/commit manifest 的 unchanged path 仍必须被 `finalize_path_unchanged` 拒绝。
- 若 source artifact 在 HEAD 中 tracked，则仍必须形成真实 staged deletion；若当前文件重新出现、hash 非 `null`、lifecycle/path identity 不匹配或 attribution 漂移，必须 fail closed。
- 添加端到端回归：untracked source artifact → return_fixed → exact snapshot null tombstone → accepted validation/review → managed closeout commit 成功，提交只包含实际 fixed/return/source changes，index 终态为空。
- 复放 Runtime04 closeout；必须产出 exact managed commit SHA，且不吸收 resolver/single-parse/scale 或任何 foreign staged path。

## 禁止临时方案

- 不得通过手工 staging、`git add -N`、临时重建 source failure 文件、从 exact manifest 删除墓碑或修改 Runtime04 业务文件绕过。
- 不得全局放宽 unchanged-path 拒绝规则，也不得把所有 `null` path 视为可忽略。
- 不得复用已过期 attribution/closeout evidence 冒充 current-source acceptance；修复后按 coordinator policy 重建必要的 current graph evidence。

## 修复结果与回传

Open state: `待 Coordinator01 修复并复放 Runtime04 exact failure closeout`; no managed commit is claimed.
