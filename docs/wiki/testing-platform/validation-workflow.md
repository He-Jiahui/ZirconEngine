---
related_code:
  - tools/dev/local-cargo.ps1
  - tools/dev/local_cargo.py
  - .codex/skills/zircon-dev/validation/guide.md
  - .github/workflows/ci.yml
implementation_files:
  - tools/dev/local-cargo.ps1
  - tools/dev/local_cargo.py
  - tools/jenkins
plan_sources:
  - docs/plans/milestone-validation-policy.md
  - docs/tooling/coordinator-retirement.md
tests:
  - .github/workflows/ci.yml
  - tools/tests/test_local_cargo.py
doc_type: workflow-reference
---

# 验证工作流

[里程碑验证策略](../../plans/milestone-validation-policy.md) 是范围和节奏的权威；命令细节见 [Zircon Dev 验证指南](../../../.codex/skills/zircon-dev/validation/guide.md)。旧本地协调器、受管 matrix、登记、租约和提交工作流已经退役，不能把旧脚本当作当前执行前提，也不能为了运行旧计划恢复服务。

## 按变更选择验证

普通实现切片使用格式、`git diff --check` 和实际相关的结构检查。小型文档改动检查导航、引用和 metadata，不触发 Rust 编译。已记录的失败、公共或 ABI 契约、unsafe 与持久化行为变更按策略立即运行受影响的聚焦回归。

里程碑批次运行一个覆盖变更目标和 feature 的 package-scoped check，加上对应行为及边界回归；共享契约、根 manifest、锁文件、toolchain 或声明的 wave/release 边界才扩大范围。测试保持完整，缩小重复构建和调度次数，不能通过删除断言减少验证。

```powershell
# 只渲染命令；不产生通过证据
.\tools\dev\local-cargo.ps1 -DryRun check -p zircon_runtime --locked

# 到达相应编译批次后运行
.\tools\dev\local-cargo.ps1 check -p zircon_runtime --locked
.\tools\dev\local-cargo.ps1 test -p zircon_runtime --lib <focused-filter> --locked
```

保留 `--locked`、真实筛选与 feature 设置。release 使用 `--release`，profiling 使用工作区的 `--profile profiling`。ignored 测试必须有明确的测试目标或筛选。旧 `validate-matrix.ps1` 的 export/profile/convention 参数属于迁移资料，所需命令应从当前 CI 和实际测试 owner 组合，不重启旧协调器执行它们。

## 输出和所有权

所有编译产物、构建目录、编译缓存及临时编译输出必须物理位于盘根 `D:\cargo-targets`、`E:\cargo-targets` 或 `F:\cargo-targets`。拒绝其他盘、仓库 `target`、`targets`、`ZirconBuilds`、嵌套同名目录及路径别名。本地入口核验实际路径与 35 GiB 空间保留量，保留原生 Cargo 锁，并使用独立 `zircon-local` 命名空间；不删除历史池、回执或锁来使新检查通过。

Windows PowerShell 是默认环境。WSL 仅用于具体的 Linux 专属失败、工具、CI 复现或明确的平台要求；不要重复运行无关的 Windows/WSL 全套验证。平台选择逻辑测试也不等于真实设备、GPU 或窗口验证。

## 异步请求和接受边界

独立 Jenkins 协调器遵循 [jenkins-coordination 技能](../../../.codex/skills/jenkins-coordination/SKILL.md)。先核验当前启用记录、`functionalTestsAllowed` 和实际证据，再在任务授权内运行功能测试或提交有边界的命令。已经激活的唯一入口约束继续适用，不能绕开它使用独立命令。

提交后保存原请求 ID；客户端超时或终端无输出时先核对该请求，避免重复提交。按真实阶段记录状态、stdout/stderr、退出码、timing 与 artifact digest。源码交接使用补丁和哈希，agent 不自行创建源码快照或备份；必要验证源码由独立协调器准备。

本地命令通过只证明该命令和对应源码范围。dry run、pending 回执、历史票据与本地退出码都不授予 Jenkins、全工作区、里程碑或生产迁移接受。产品、截图、真实设备或编辑器 host 验证在改变这些输出的边界单独完成，不以策略测试模拟替代。

## CI 和证据

`.github/workflows/ci.yml` 及对应 profile/feature、export policy 工作流是 CI 权威。记录实际平台、物理输出目录、profile、features、源码身份、命令、筛选、失败修复及未完成的外部验证。重复检查只针对新修改、已知失败或未解决问题；没有这些理由时复用仍匹配当前源码的证据。
