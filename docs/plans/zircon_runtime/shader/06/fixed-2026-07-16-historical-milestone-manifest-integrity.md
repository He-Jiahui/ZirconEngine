---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: historical-milestone-manifest-integrity
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/workflows/milestones.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_workflow_commit.WorkflowCommitTests.test_reconcile_accepted_milestone_copies_immutable_evidence_between_equal_topologies -v
resolved_at: 2026-07-16
---


# Session coordinator: historical milestone manifest integrity

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行者：`shader06-pmrem-artifact-layout-finalize-20260714`
- 来源执行切片：EC-M3 离线 IBL 资产与派生物的已验收提交 `f6f9cf8f29c60976288353268c9319c399276ffd`
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：跨运行导入必须从协调器保存的不可变证据验证历史清单；该保存边界由协调器负责，而不是 Shader06 重新修改旧文件。

## 失败现象与复现证据

Shader06 的两个运行具有相同计划路径与拓扑哈希
`bd8915a280650016a61d2c3a428c2dc872500260880d03ced303d4718c58025a`。

- M3 的 source accepted attempt、reconciled commit intent 与提交 `f6f9cf8f29c60976288353268c9319c399276ffd` 均存在，且该提交是当前 `HEAD` 的祖先；但其记录的 manifest hash 为 `6315b7ebc41504a025fcf3a23737eabd6dfc0e1a1029e3ffd38d46c1743d13fe`，由该提交树精确重建得到 `a44386c050dc79ac07668125a650a920dcc867c0940b2510d6d7e5e8df025748`。
- M4 的同类重建通过：`029f870fcceb84d076da2040791fd0ee01e8f8ee` 与 manifest hash `262aa837a04dd9d326e7714453bc8389046bfed8aaf2248d6f32cdb587393e92` 一致。
- 因为批量导入必须保证所有节点均有精确内容证明，M3/M4 不得部分导入，且不得让 Shader06 重新暂存、修改或重复提交历史文件。

2026-07-15 对当前 M5 target run 执行严格 M1/M2-only controlled action 后，还发现 M1
存在同类历史完整性缺口：`c228d91c9ff7b2a167237570513c9257e05bee66` 的 9 条 manifest
路径均可从提交树读取，但按排序路径/文件 blob 重建为
`10869e5d2e0705e6c759d2fdb79b8bb97d30ca26e3ecfb50aa4b94d6060b0df1`，与保存的
`e93f9e41e8e2ceb7b608d621a498e5f5d61d3acaf173e5331ecaccd18c1b9188` 不同。M2
`4305439299b7d17e36023a7bb4ea56ecadd91837` 的 4 条路径则精确匹配
`0556f0416b1e1e690fd398dda6ab75d506b809c7c9f9fd3f7b86a43e0433a5b9`。不过 M2 依赖
M1，因此不得单独导入 M2 来制造依赖跳跃。

## 最低共享层根因

历史 `workflow_milestone_manifests` 仅保存聚合 hash 与路径列表，没有保存每条路径在已验收提交中的 blob digest 或可验证的提交树证明。早期完成路径允许 manifest 绑定与最终提交之间出现内容不一致时仍生成 accepted attempt，因此现在无法仅凭该不可变记录证明 M3 的清单内容。

## 架构修复验收

- 协调器在绑定 manifest 时持久化按路径排序的 blob digest 证明，并把它与 finalize 的实际 scoped commit tree 进行一致性验证。
- `milestone.reconcile_accepted` 必须从 accepted attempt 所指的原始 topology version 查找同版本 manifest，而不能错误地以运行当前版本重取历史清单；计划路径、当前拓扑哈希、节点身份、accepted attempt、祖先提交和每路径内容证明仍须全部通过。
- 开放 Failure 作为对账审计上下文持久化到 action/attempt evidence，不得遮蔽已逐路径验证的、无依赖缺口的历史节点；它也不得放宽 M3 的 manifest-content 校验，M3 仍保持拒绝。
- 对账是 coordinator-owned 的证据写入，不触碰目标 Session 的工作树；目标 run 因等待外部 Cargo 而变为 `stale` 时仍可导入，终态（succeeded/cancelled/archived）run 则必须拒绝。
- 对旧记录提供受管迁移：只能由历史 commit tree 重建并匹配的记录获得 attestation；不匹配记录保持拒绝并返回明确审计结果。
- 对账动作必须拒绝跳过尚未 accepted 的依赖；M2 单独匹配也不能越过不匹配的 M1。只有完整依赖闭包都经验证时，才可导入。
- 重新运行 Shader06 M1/M2/M3/M4 对账；只有每个请求节点及其依赖均已验证时，才允许在包含 M5 的新拓扑中恢复对应 accepted state。

## 禁止临时方案

- 不得仅凭相同 topology hash、commit SHA 或当前工作树内容复制 M3 证据。
- 不得将不匹配的 M3 hash 覆盖为当前或 commit-tree hash。
- 不得要求 Shader06 弄脏、暂存或重复提交已完成的 M3 文件。
- 不得弱化对账动作的 manifest-content 校验或允许只导入 M4 后跳过 M3 依赖。

## 修复结果与回传

- 根因：历史 milestone 记录没有可复建的逐路径 blob 证明；开放 handoff 被直接作为所有 Shader06 节点的 failure_audit 输入，错误阻断了已独立绑定当前源码 hash 的 M1 attestation。
- 架构修复：保留历史 M1/M3 manifest-content mismatch 的严格拒绝，不导入、不覆写旧 hash；failure return 改为 child_record_only 原子回传，允许独立的一文件 current-source attestation 作为新的 M1 证据，而不写任何父计划定义。
- 验证：当前 M1 manifest aac000e4ba984373b8c1e7532c795750 的 source hash 为 52d4fe77fe0a0439718d05f4c71c8932ed2631134f5bdc49bbdbf078930566d5；受管 validation 9166e9dd67dd4a208f091eeb033bfdc6 已 accepted，独立 review 151197bdc19c4d709578948e735eda82 为 0 Critical/0 Important。历史 c228d91c 与 f6f9cf8f 的不匹配结论未被修改。
- 回传：Shader06 可在既有 run 7a5653d606a64662b2618a1662968e3a 上重新执行 milestone validate；不得新建 topology，不得导入或重写历史 manifest。
