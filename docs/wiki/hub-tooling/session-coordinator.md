---
related_code:
  - tools/dev/zircon-session.ps1
  - tools/dev/local-cargo.ps1
  - tools/dev/local_cargo.py
  - tools/jenkins/jenkins_coordinator.py
  - docs/testing-and-validation/milestone-validation-policy.md
implementation_files:
  - tools/dev/local-cargo.ps1
  - tools/dev/local_cargo.py
  - tools/jenkins
plan_sources:
  - docs/testing-and-validation/milestone-validation-policy.md
  - docs/tooling/coordinator-retirement.md
  - docs/plans/zircon_tooling/session_coordinator/01
tests:
  - tools/tests/test_local_cargo.py
  - .github/workflows/ci.yml
doc_type: operations-reference
---

# Agent 本地验证与历史协调器

旧 Session Coordinator 及其服务、托盘、自动登记、租约、验证票据和提交接口已经退役。`tools/dev/zircon-session.ps1` 是返回退出码 3 的停用入口；不要为普通开发恢复旧服务、队列或自动同步。历史数据与恢复约束见 [协调器退役记录](https://github.com/He-Jiahui/ZirconEngine/blob/main/docs/tooling/coordinator-retirement.md)。

当前由 Agent 按[直接开发管理标准](https://github.com/He-Jiahui/ZirconEngine/blob/main/docs/tools/agent-artifact-management.md)管理授权的原生 Git/worktree、生成、复用和独立本地命令证据。执行节奏遵循 [里程碑验证策略](https://github.com/He-Jiahui/ZirconEngine/blob/main/docs/testing-and-validation/milestone-validation-policy.md)，具体命令遵循 [Zircon Dev 验证指南](https://github.com/He-Jiahui/ZirconEngine/blob/main/.codex/skills/zircon-dev/validation/guide.md)。本地证据不自动授予全工作区、里程碑或产品接受。

## 当前本地入口

```powershell
# 预览命令；不会编译，也不是通过证据
.\tools\dev\local-cargo.ps1 -DryRun check -p zircon_runtime --no-default-features --features core-min --lib --locked

# 在该变更的编译批次到期时运行
.\tools\dev\local-cargo.ps1 check -p zircon_runtime --no-default-features --features core-min --lib --locked
```

本地入口核验实际物理输出路径、35 GiB 空间保留量及原生 Cargo 锁，默认使用独立的 `zircon-local` 命名空间。普通回执记录实际命令与观察到的 Cargo 根进程退出码；不是整棵进程树终止、产物来源或封存输入证明。

所有编译产物、构建目录、编译缓存和临时编译输出必须物理位于盘根 `D:\cargo-targets`、`E:\cargo-targets` 或 `F:\cargo-targets` 下。其他盘、仓库 `target`、`targets`、`ZirconBuilds`、嵌套同名目录及路径别名全部拒绝。普通验证默认 Windows；只有明确的 Linux 专属失败、工具、CI 复现或平台要求才使用 WSL，并遵守同一物理输出边界。

## 历史 Jenkins 接受边界

[原 Jenkins 指南](https://github.com/He-Jiahui/ZirconEngine/blob/main/.codex/skills/jenkins-coordination/guide.md)仅用于解释历史激活、请求和接受回执，不再是可调用技能或当前开发入口。配置、凭据、状态、原请求和验收结果保留，不因取消开发流程而改写成功；也不宣称主机服务已停止。服务运维、恢复旧请求或变更启动项需要单独明确授权。

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

独立命令的 JSON 结果、历史 Jenkins 请求回执和里程碑接受记录各有范围。报告应标明实际完成与仍 pending 的证据，并保留原操作身份；不为迁移普通开发而伪造回执、重放旧队列或删除未知数据。
