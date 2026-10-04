---
related_code:
  - tools/dev/zircon-session.ps1
  - tools/dev/local-cargo.ps1
  - tools/dev/local_cargo.py
  - tools/jenkins/jenkins_coordinator.py
  - docs/plans/milestone-validation-policy.md
implementation_files:
  - tools/dev/local-cargo.ps1
  - tools/dev/local_cargo.py
  - tools/jenkins
plan_sources:
  - docs/plans/milestone-validation-policy.md
  - docs/tooling/coordinator-retirement.md
  - docs/plans/zircon_tooling/session_coordinator/01
tests:
  - tools/tests/test_local_cargo.py
  - .github/workflows/ci.yml
doc_type: operations-reference
---

# Session Coordinator 与受管验证

旧 Session Coordinator 及其服务、托盘、自动登记、租约、验证票据和提交接口已经退役。`tools/dev/zircon-session.ps1` 是返回退出码 3 的停用入口；不要为普通开发恢复旧服务、队列或自动同步。历史数据与恢复约束见 [协调器退役记录](../../tooling/coordinator-retirement.md)。

当前开发使用独立本地命令证据或已核验激活状态的 Jenkins 协调器。执行节奏遵循 [里程碑验证策略](../../plans/milestone-validation-policy.md)，具体命令遵循 [Zircon Dev 验证指南](../../../.codex/skills/zircon-dev/validation/guide.md)。这两个入口都不自动授予全工作区、里程碑或生产迁移接受。

## 当前本地入口

```powershell
# 预览命令；不会编译，也不是通过证据
.\tools\dev\local-cargo.ps1 -DryRun check -p zircon_runtime --locked

# 在该变更的编译批次到期时运行
.\tools\dev\local-cargo.ps1 check -p zircon_runtime --locked
```

本地入口核验实际物理输出路径、35 GiB 空间保留量及原生 Cargo 锁，默认使用独立的 `zircon-local` 命名空间。它保留实际命令与退出码，不要求恢复旧 session、heartbeat 或 lease。已经启用的 Jenkins 唯一入口约束仍然适用；不能通过独立预览绕过它。

所有编译产物、构建目录、编译缓存和临时编译输出必须物理位于盘根 `D:\cargo-targets`、`E:\cargo-targets` 或 `F:\cargo-targets` 下。其他盘、仓库 `target`、`targets`、`ZirconBuilds`、嵌套同名目录及路径别名全部拒绝。普通验证默认 Windows；只有明确的 Linux 专属失败、工具、CI 复现或平台要求才使用 WSL，并遵守同一物理输出边界。

## Jenkins 的独立激活和接受

根据 [jenkins-coordination 技能](../../../.codex/skills/jenkins-coordination/SKILL.md)，先核验 `.codex/state/jenkins-coordinator/active.json` 的启用状态、`functionalTestsAllowed` 及对应证据。核验通过后，才在任务授权范围内运行协调器功能测试或提交有边界的命令。普通本地命令结果和旧票据都不能替代新的 Jenkins 接受回执。

异步提交后保留原请求 ID，并查询、归并同一请求。客户端超时或暂时没有终端输出不表示任务未运行；不得因此重复提交。pending、dry run 与本地退出码不能改写为里程碑或生产迁移接受。

## 需要保留的历史对象

| 对象 | 迁移时保留的内容 |
| --- | --- |
| Session 与 Lease | 原身份、路径归属、时间及冻结时的实际进程证据；不作为当前登记前提。 |
| Validation ticket/job | 原命令、输入身份、状态、控制请求与回执；不自动回放旧队列。 |
| Cargo pool/lane | 历史兼容性键、物料及原生锁；不删除旧池来使新检查通过。 |
| Validation copy 与 Patch | 来源、补丁和哈希；由获得授权的协调器准备验证源码，agent 不创建源码快照或备份。 |
| Review 与 Evidence | 原结论、审阅来源及未接受边界；新验收提供新的实际证据。 |

兼容性仍需要区分仓库、平台、toolchain、架构、profile、features、链接模式与完整命令。旧 `reuse`、`compact`、`diagnostic` 模式是历史迁移资料，不能推定为当前本地入口的参数，也不能混用不兼容编译产物。

## 失败与回执

命令失败时记录平台、物理输出目录、profile、features、测试筛选、源码身份、命令和实际退出码；聚焦到真实 owner 的失败后只重跑受影响批次。保持历史回执、数据库、对象、队列记录和仍有所有者的锁，不通过删除记录或伪造成功状态关闭任务。

独立命令的 JSON 结果、Jenkins 的请求回执和里程碑接受记录各有范围。报告应标明哪一种证据已完成、哪一种仍 pending，并保留可以恢复原请求的身份。
