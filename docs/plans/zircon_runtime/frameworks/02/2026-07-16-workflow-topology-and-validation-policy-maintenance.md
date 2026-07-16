# Frameworks 02 Workflow Topology And Validation Policy Maintenance

> 本文件记录 `02-module-kernel-and-lifecycle-unification.md` 的 docs-only 工作流维护；不替代 M1、M2、M3 的业务实现、编译、测试、运行或产品验收。

| 切片 | 完成项目 | 状态锚 | 日期 | 当前状态与证据 |
|---|---|---|---|---|
| Maintenance | Machine-readable M1→M2→M3 topology and validation-policy adoption | `frameworks_02_workflow_topology_and_validation_policy_maintained` | 2026-07-16 | **accepted docs maintenance**。在父计划写入 `zircon-workflow` schema 1，严格镜像既有三个标题及其线性依赖：M1 无依赖、M2 依赖 M1、M3 依赖 M2；不新增、合并或改名业务里程碑。父计划同时采用先前已存在且经审计的测试阶段差异，以 `docs/plans/milestone-validation-policy.md` §3/§4 将 routine Cargo 从逐切片调整为 milestone focused batch 与 wave-wide regression，并把该权威文件加入 `plan_sources`；每个里程碑保留 package compile gate，原先把多个过滤词拼在一个 Cargo 命令后的无效写法已拆为逐条可执行的 focused 命令，原有 runtime lib、zircon_app package 与插件工作区全量回归命令均显式保留到 wave gate。current-source topology parser 返回 `source=zircon-workflow`、M1→M2→M3 与 topology hash `46417a239e8906b0c930514b70a1f7770205d8eadfa8e8de8ef3fb30004c5c7f`；coordinator plan audit、plan-output audit、scoped `git diff --check` 通过，fresh docs checker 保持全局预期 RED 218 missing / 33 affected docs、exact scope violation 0。独立审查首轮发现并修正 2 Important：M1 focused gate 不再宣称全量 runtime 零回归；M2 不再含糊丢失 zircon_app 全 package 回归，所有 wave-wide 结论只由波次收口证据确认；最终复审为 0 Critical / 0 Important / 0 Minor。本维护不声明 Frameworks02 M2 或全局计划完成。 |

## 精确范围

- `docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md`
- `docs/plans/zircon_runtime/frameworks/02/2026-07-16-workflow-topology-and-validation-policy-maintenance.md`

## 边界

- 拓扑只转写既有 M1/M2/M3 标题和顺序，不改变计划设计。
- focused gate 不能冒充全量 package、workspace、运行或产品验收。
- 同目录的 `2026-07-16-m2-plugin-group-typed-error-propagation.md` 属于后续业务 M2 manifest，本提交不吸收它。
- 业务 M2 仍必须通过 coordinator native `milestone prepare/validate/review/commit`。
