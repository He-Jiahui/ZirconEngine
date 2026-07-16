---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: support-slice-exact-finalize-plan-output-conflict
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_workflow_node: M1.3
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/control_plane/actions/catalog.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/workflows/gates.py
  - tools/session_coordinator/control_plane/actions/models.py
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/plans.py
  - docs/plans/zircon_editor/editor/02/2026-07-14-world-sync-m1-milestone-manifest.md
tests:
  - python -m unittest tools.session_coordinator.tests.test_action_catalog.ActionCatalogTests.test_commit_lifecycle_accepts_slice_ids_without_widening_reconciliation -v
  - python -m unittest tools.session_coordinator.tests.test_workflow_commit.WorkflowCommitTests.test_slice_commit_succeeds_without_accepting_parent_milestone -v
  - ./tools/zircon-session.ps1 -Json milestone validate --session-id editor02-m1-3-inspection-hardening-20260715 --run-id ac800d3e33174e38a77ba5da7a8250f2 --milestone M1.3 --template coordinator-actions
  - ./tools/zircon-session.ps1 -Json finalize preview --session-id editor02-inspection-compile-sync-support-20260715 --message "fix(editor): sync inspection hierarchy compile contract" --path docs/plans/zircon_editor/editor/02/2026-07-14-world-sync-m1-output-records.md --path docs/zircon_runtime/scene/inspection.md --path zircon_runtime/src/scene/inspection/snapshot.rs --path zircon_runtime/src/scene/inspection/tests.rs
resolved_at: 2026-07-16
---


# Session Coordinator 01：已验收 support slice 无法在父 milestone pending 时精确提交

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-15 | Editor02 inspection compile-sync 的 3 个非计划业务路径与 1 个编号子计划记录均已 current-hash 认证，独立复核 P0/P1 为 0，`rustfmt --check`、scoped `git diff --check` 通过，历史受管 scene 门为 596/0；协调器仍无法形成精确 support commit，且所有保护均未绕过。 |
| `OPEN / 第二独立复现` | 2026-07-15 | Editor03 主计划补充 `M3.2` slice 后，TopologyParser 能正确导入 slice，但 validation/review/commit 的 action model 只接受 `M<number>`，gate 与 manifest service 又只查询 `kind='milestone'`；`M3.2` 因而返回 `workflow_milestone_not_found`，无法提交已复核的 operation-factory 切片。直接把该切片提交为 M3 会错误接受尚未完成的 M3.1/M3.3。 |
| `OPEN / NATIVE SLICE BACKEND GREEN` | 2026-07-15 | 当前共享源码已接通 `M<number>.<positive-number>` 的 typed action、manifest、gate refresh、review、commit 与 node-scoped attempt；slice commit 只成功自身，父 milestone 会继续被 sibling slice 与 milestone dependency 阻塞，reconciliation 仍严格只接受完整 milestone。fresh 相关回归 `46/46` 通过；干净 HEAD 覆盖最小 6 个代码/测试文件的独立复放为 `3/3`。 |
| `OPEN / SUPPORT LIFECYCLE + ATOMIC OWNER BLOCKED` | 2026-07-15 | 首轮独立复核为 `P0=0 / P1=2 / P2=2`；两项 P2（pending/sibling/dependency 证据与 generic slice summary 分叉）修正后的 fresh 复核确认全部关闭、无新增 P0/P1，定向 `2/2`、完整 workflow commit `17/17`、scoped diff-check 通过。两项既有 P1 仍成立：显式 support/failure-return commit 类型和 Editor02 4 文件真实重放尚未实现；最小 6 文件闭包混有 `session-coordinator-artifact-governance-20260715` 的 active lease/current changes，当前 Session 不得冒领或原子提交。因此本 handoff 保持 `open`，不得改名为 `fixed-*`。 |
| `OPEN / REAL M1.3 REPLAY REACHES FAILURE AUDIT` | 2026-07-15 | Editor02 topology 维护已原子提交 `df12e9d4591f160c53f4594b76b6dd1b65a2b039`，run `ac800d3e33174e38a77ba5da7a8250f2` 激活原生 `M1.3`。四文件 manifest、plan-output 与 independent review 均 accepted；validation-copy `faff074a8e6142cdbd26163da63744ea` 自然结束并写入 `managed_validation_succeeded`。CLI 同步调用仍错误返回 `invalid_response`，因为 deferred validation action succeeded 后没有即时 result payload。最终唯一 gate 类别为 plan-wide `failure_audit`：它同时把 Runtime15 depth-prepass 上行 scene failure 与本 Coordinator failure 施加到 M1.3，说明 native slice 尚不能在父计划 unrelated failure pending 时完成真实 exact commit。 |
| `OPEN / CURRENT HEAD REPLAY CONFIRMS BOTH PROTOCOL DEFECTS` | 2026-07-15 | HEAD 前进到 `3bbf00b19f5eeede84600229a36cff80a9c1155e` 后，Editor02 四文件以 current hashes 重新 prepare 为 fingerprint `c851123e69073490d549bf923c75588a6cc57d968692ccefd1e40ba2bf01d49d`。deferred action `23724c0133fc4edea4093760ba5a8d41` 再次先向 CLI 返回 `invalid_response`，但服务端实际创建 job `c8961adaa03a4cd6905aa1ec89d3c36e` / run `3a49ba945c40479da876ebe7dca5fa04`，自然结束 `exit 0` 并绑定 `managed_validation_succeeded`；fresh reviewer `ccf0ac197b1541eb81f058bee9b3451c` 为 `P0=0/P1=0`。随后 `milestone commit M1.3` 只因 `failure_audit` 拒绝，结构化返回 Runtime15 depth-prepass 与本 support-slice 两个 lifecycle key，证明 manifest、plan-output、validation、review 已全部接受而 node-scoped Failure 过滤仍未实现。本记录新增 `origin_workflow_node: M1.3` 作为结构化 owner 输入；记录写入会改变 Failure 指纹，故真正修复回传后仍须 final prepare/validation/review，不能复用本轮 evidence 提交。 |
| `OPEN / FIVE-FILE CURRENT-SOURCE REPLAY` | 2026-07-15 | 新协调器实例 `ecdd1f84df9d414f898f817ef36f16e0` 已加载 current-source manifest 选择修复。Editor02 version 2 `e88bff87699644c689f58bfc2be53ae4` 在 M1.3 绑定精确五文件 manifest `a52bc68e6543a272753e7a2914f270af8e030989ada64ffa8ac7361ba09cb8ec`，包含 Runtime15 depth-prepass 的 canonical `fixed-*` 回传；validation action `68b94656576b4c27aaedcac814ec5cf4` 虽因 deferred client polling 缺陷先向 CLI 返回 `invalid_response`，服务端仍唯一创建 job `4b3602f2f8884682a1b63555b993f7d3` / run `66d89726e0cd4298904d8adc9f4f872d`，`24/24`、`exit 0`、`managed_validation_succeeded`。fresh review `289a4565459d4aa7b5dfbbcf94a06ced` 为 `P0=0/P1=0`；当前 M1.3 的 manifest、plan-output、validation、review 均 accepted，唯一拒绝项为本 lifecycle 的 `failure_audit`。这证明 current-source 记录选择已生效，但 deferred action polling 与本 failure 的最终 return 仍必须由 Coordinator01 完成。 |
| `OPEN / DEFERRED ACTION POLLING ACCEPTED` | 2026-07-15 | Python client 已按 action ID 轮询 `previewed` / `executing` 到终态。RED 回归先复现 `expected succeeded, actual executing`，实现后定向 `1/1` 与 `test_milestone_cli` `5/5` 通过；Editor02 M1.3 真实 action `34316e6cd8ce4fb2a9cbc7f9f079f221` 等待约 58 秒后由 CLI 正常返回唯一 job `dabf5b4394d04cf18aa061bb0d7c090c` / run `38ecc821b332447d9512337ed70b796d`，随后 `24/24`、`exit 0`。本行只验收 deferred 子缺陷；node-scoped Failure gate、共享源码原子提交和 lifecycle return 尚未完成，因此 artifact 继续保持 open。 |
| `OPEN / FINALIZER SCOPE RED` | 2026-07-15 | 独立复核确认 `MilestoneWorkflowService` 的 gate/fingerprint 已按 workflow node 过滤，但 `GitFinalizeService.commit_milestone` 在 staging 前与 `update-ref` 前仍两次调用 plan-wide `_require_failure_acceptance`，因此 unrelated sibling Failure 最终仍会报 `finalize_open_failure`。新增 production-like `test_milestone_failure_scope`：explicit finalize 的 plan-wide 用例 `1/1` 通过，milestone sibling/own/parent/legacy/fixing/二次锁内变化 `7/7` 以缺少必填 `failure_workflow_node_keys` API 或专用 guard 精确 RED。修复必须 hard-cut 为非空必填 scope，并让锁内两次检查使用同一 immutable tuple；禁止 `None`、默认 plan-wide 回退或 callback 绕行。`git_finalize.py`、`milestones.py` 及既有大测试仍由 active `session-coordinator-artifact-governance-20260715` 管理，本 Session 只登记 failure 与新聚焦测试，不冒领其共享源码。 |
| `OPEN / FINALIZER GREEN, AWAIT OWNER COMMIT` | 2026-07-16 | finalizer hard cut 已完成：workflow commit 只解析一次 canonical、非空 node tuple，并在 initial fingerprint、锁内 precommit fingerprint、staging 前与 update-ref 前复用；explicit finalize 仍独立 plan-wide。fresh 回归为 scope `11/11`、workflow `22/22`、finalizer `40/40`、数据库/Failure/deferred/scope 组合门 `41/41`，语法和 diff-check 通过；独立 review `P0=0 / P1=0 / P2=0`。由于 `git_finalize.py` 的整文件 current work 仍归 active artifact-governance owner，必须由该 owner 原子提交共享闭包并加载 schema 35；随后才可执行 Editor02 五文件 fresh replay 和本 lifecycle return，因此本 artifact 继续为 `open`。 |
| `FIXED / RETURNED TO EDITOR02` | 2026-07-16 | Coordinator01 完成 structured `origin_workflow_node`、slice/parent Failure closure、required immutable node tuple 与 deferred action polling；fresh scope `11/11`、workflow `22/22`、finalizer `40/40`、组合门 `41/41`、独立 review `P0/P1/P2=0/0/0`，authoritative schema35 daemon 已加载。Failure 已原子回传为本 fixed artifact；Editor02 上行仍须以当前 exact-6 manifest 重新 validation/review/commit，父 M1 保持 pending。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：M1 inspection compile-sync 独立支持切片
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：实现与测试已经验收，唯一阻断位于协调器对 numbered-plan milestone、plan-output ownership、support Session 和 degraded shared baseline 的组合提交策略。

## 失败现象与复现证据

Editor02 父 M1 的既有 `2026-07-14-world-sync-m1-milestone-manifest.md` 明确保持 `Status: pending`，并声明 Runtime15 failure 返回前不得绑定父 M1。短生命周期 compile-sync 只需要提交：

- `zircon_runtime/src/scene/inspection/snapshot.rs`；
- `zircon_runtime/src/scene/inspection/tests.rs`；
- `docs/zircon_runtime/scene/inspection.md`；
- `docs/plans/zircon_editor/editor/02/2026-07-14-world-sync-m1-output-records.md`。

当前协调器依次拒绝所有合法精确路径：

- numbered-plan `milestone validate M1` 自动读取父 M1 全量 manifest，返回 `milestone_manifest_not_attributed`，要求约 30 个不属于 compile-sync Session 的文件；
- numbered-plan Session 不能通用转为 completed，返回 `session_goal_close_requires_milestone`，正确阻止绕过父 M1；
- 无 plan 的 support Session 携带编号子计划记录时返回 `finalize_session_plan_missing`；
- 把计划记录归还 numbered-plan Session、support Session 只提交 3 个非计划路径时，explicit finalize 返回 `finalize_baseline_degraded`，不能在共享脏工作树中执行已认证的精确 staging。

共享暂存区始终为 0；没有手工 `git add`、没有 `--maintenance`、没有改写父 M1 manifest，也没有接受全局 degraded baseline。

## 最低共享层根因

协调器目前只有两条互斥提交路径：numbered-plan milestone 要求使用父里程碑全量 manifest，explicit finalize 又要求 Session completed 且 baseline 不 degraded。缺少“父 milestone 仍 pending，但一个已验收、带对应 child-plan 记录的 support/failure slice 可以按 current-hash exact manifest 提交”的受管模型。

Topology importer 已将 `M3.2` 等节点建为 `kind='slice'`，并建立 `slice -> owning milestone`
依赖边，但 action 参数、manifest、gate、review 与 commit service 仍硬编码 milestone-only 查询。
这造成模型已经声明 slice、执行层却无法产生 slice attempt/commit 的半接线状态。

2026-07-15 的真实 M1.3 重放进一步证明第二层根因：action/manifest/review/validation
均已支持 slice 后，`refresh_gates` 仍调用 `open_related_to_plan(plan_path)`，把整份来源计划的
所有 open failure 无差别写入每一个 slice gate。M1.3 与 Runtime15 depth-prepass/Render18
pipeline layout 没有代码或测试依赖，却仍被其 failure lifecycle 阻塞。deferred validation
action 的 CLI 又要求同步 result payload，导致 job 已创建并最终 accepted 时调用方先收到
`invalid_response`。

结构化归属不能从中文正文或 slug 猜测。Failure frontmatter 需要保存来源 workflow node；
slice gate 只接受与自身相同的来源 node，完整 milestone closure 仍聚合其 child slices，缺少
结构化 node 的旧 handoff 保守按 plan-wide 处理。fixing plan 的 Failure Priority Gate 继续
无条件生效，不能借 node 过滤开始普通功能工作。

deferred action 的最低层同样不是验证执行失败：`validation.start` 在 action service 中先进入
`executing`，后台 materialize 并启动验证后才写入 result。CLI/client 必须以 action id 轮询到
终态再解析 validation-copy handle；不得把 `executing + result=null` 当成 completed，也不得
因为首个响应缺 job id 就重复创建验证副本。

## 架构修复验收

- 新增显式 support/failure-return commit 类型，允许绑定 fixing/origin plan 和编号子计划记录，但不把父 milestone/Goal 标为完成。
- 原生 slice ID `M<number>.<positive-number>` 可走同一 current-hash manifest、managed validation、独立 review 与原子 commit 流程；slice commit 只把该 slice node 记为 succeeded，父 milestone 继续等待 sibling slices 与既有依赖。
- milestone reconciliation 仍只接受完整 milestone，不允许以 slice evidence 冒充历史 milestone accepted。
- exact manifest 继续要求 live lease、current-hash attribution、独立复核与验证；只提交声明路径，并在 degraded shared baseline 中保留其他 Session 工作树/暂存区。
- failure audit 必须按结构化 slice/failure-return 归属判断，不得把同一父计划中与该 slice 无关的 open failure 全量施加；确属该 slice 的 failure 仍必须阻断。
- Failure importer/gate 使用 frontmatter 中的 workflow node，不解析 Markdown 正文；完整 milestone 聚合 child slice Failure，未知旧记录保守 plan-wide，fixing-plan priority 不削弱。
- deferred `validation.start` 成功必须向 CLI 返回可跟踪的 validation-copy/action handle；不得在后台 job 已创建时先报 `invalid_response`。
- 不需要把 support Session 伪装成 maintenance，也不需要修改父 milestone manifest；提交结果记录独立 SHA、subject、shortstat 与通知状态。
- 以本 lifecycle 当前收敛后的 6 文件 Editor02 M1.3 slice 重放：父 M1 保持 pending；inspection 源码/测试/模块文档、产出记录和两份 fixed handoff 形成唯一 current-hash manifest，旧 4/5 文件清单只保留为历史证据；foreign staged path 不得进入该原子提交。

## 禁止临时方案

- 不得使用 `--maintenance`、全局 baseline accept、手工 Git staging/commit、hook bypass 或直接 SQLite 修改。
- 不得把父 M1 manifest 改成 compile-sync 小清单，也不得冒领父 M1 其余文件。
- 不得丢弃编号子计划记录、把输出写到全局计划或错误地关闭 Editor02 M1/Goal。

## 修复结果与回传

- 根因：Milestone workflow gates and fingerprints were node-scoped, but the authoritative Git milestone finalizer still applied a plan-wide Failure guard and the action client returned before deferred validation reached a terminal result.
- 架构修复：Added structured origin_workflow_node persistence and exact slice/parent closure filtering; hard-cut milestone finalization to a required canonical immutable node tuple reused by initial context, precommit context, staging and update-ref guards; retained explicit finalize as plan-wide; polled deferred actions to terminal state.
- 验证：Fresh coordinator evidence: milestone scope 11/11, workflow commit 22/22, Git finalizer 40/40, database/Failure/deferred/scope 41/41, py_compile and diff-check passed; independent review P0=0 P1=0 P2=0; authoritative schema35 daemon loaded current source; handoff audit 152/152。Editor02 已实际绑定当前 6 文件 M1.3 manifest；首次 managed validation 使 `commit_manifest`、`failure_audit`、`plan_output` accepted，随后独立复核发现本 fixed 记录与产出记录仍使用 4/5 文件旧口径，故未提交并先修正记录。更新后哈希必须再次通过 managed validation 与独立 review，最终 accepted fingerprint 和 commit 结果写入协调器事件账本，不在本文件中预写自引用 SHA。
- 回传：本 fixed 状态只表示 Coordinator01 根因修复及 Failure 原子回传完成；Editor02 上行 acceptance 必须由当前 6 文件 exact manifest 的重新验证、零 P0/P1 独立复核和原子 slice commit共同完成。父 M1、Editor02 Goal 与受保护计划定义始终保持独立 pending，不得借此晋升。
