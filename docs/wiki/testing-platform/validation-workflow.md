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
  - docs/testing-and-validation/milestone-validation-policy.md
  - docs/tooling/coordinator-retirement.md
tests:
  - .github/workflows/ci.yml
  - tools/tests/test_local_cargo.py
doc_type: workflow-reference
---

# 验证工作流

[里程碑验证策略](https://github.com/He-Jiahui/ZirconEngine/blob/main/docs/testing-and-validation/milestone-validation-policy.md) 是范围和节奏的权威；命令细节见 [Zircon Dev 验证指南](https://github.com/He-Jiahui/ZirconEngine/blob/main/.codex/skills/zircon-dev/validation/guide.md)。旧本地协调器、受管 matrix、登记、租约和提交工作流已经退役，不能把旧脚本当作当前执行前提，也不能为了运行旧计划恢复服务。

## 按变更选择验证

普通实现切片使用格式、`git diff --check` 和实际相关的结构检查。小型文档改动检查导航、引用和 metadata，不触发 Rust 编译。已记录的失败、公共或 ABI 契约、unsafe 与持久化行为变更按策略立即运行受影响的聚焦回归。

里程碑批次运行一个覆盖变更目标和 feature 的 package-scoped check，加上对应行为及边界回归；共享契约、根 manifest、锁文件、toolchain 或声明的 wave/release 边界才扩大范围。测试保持完整，缩小重复构建和调度次数，不能通过删除断言减少验证。

```powershell
# 只渲染命令；不产生通过证据
.\tools\dev\local-cargo.ps1 -DryRun check -p zircon_runtime --no-default-features --features core-min --lib --locked

# 到达相应编译批次后运行
.\tools\dev\local-cargo.ps1 check -p zircon_runtime --no-default-features --features core-min --lib --locked
.\tools\dev\local-cargo.ps1 test -p zircon_runtime --no-default-features --features core-min --lib <focused-filter> --locked
```

保留 `--locked`、真实筛选与 feature 设置。release 使用 `--release`，profiling 使用工作区的 `--profile profiling`。ignored 测试必须有明确的测试目标或筛选。旧 `validate-matrix.ps1` 的 export/profile/convention 参数属于迁移资料，所需命令应从当前 CI 和实际测试 owner 组合，不重启旧协调器执行它们。

## 输出和所有权

所有编译产物、构建目录、编译缓存及临时编译输出必须物理位于盘根 `D:\cargo-targets`、`E:\cargo-targets` 或 `F:\cargo-targets`。拒绝其他盘、仓库 `target`、`targets`、`ZirconBuilds`、嵌套同名目录及路径别名。本地入口核验实际路径与 35 GiB 空间保留量，保留原生 Cargo 锁，并使用独立 `zircon-local` 命名空间；不删除历史池、回执或锁来使新检查通过。

Windows PowerShell 是默认环境。WSL 仅用于具体的 Linux 专属失败、工具、CI 复现或明确的平台要求；不要重复运行无关的 Windows/WSL 全套验证。平台选择逻辑测试也不等于真实设备、GPU 或窗口验证。

## 异步请求和接受边界

Agent 按[直接开发管理标准](https://github.com/He-Jiahui/ZirconEngine/blob/main/docs/tools/agent-artifact-management.md)在任务 worktree 运行授权的本地生成与验证。普通本地回执只记录实际命令与观察到的根进程结果，不证明进程树终止、输入封存或产品产物来源。

异步执行保存原 job/request 身份；客户端超时或终端无输出时先核对原操作，避免重复执行。按真实阶段记录状态、stdout/stderr、退出码、timing 与 artifact digest。使用任务规范 worktree 及精确补丁/哈希，保留外来修改和原生锁，不另造源码快照或备份。

本地命令通过只证明该命令和对应源码范围。dry run、pending 回执和历史票据均不是当前通过证据，窄命令成功也不授予全工作区、里程碑或产品接受。历史 Jenkins 接受状态保留原样。产品、截图、真实设备或编辑器 host 验证在改变这些输出的边界单独完成，不以策略测试模拟替代。

## CI 和证据

`.github/workflows/ci.yml` 及对应 profile/feature、export policy 工作流是 CI 权威。记录实际平台、物理输出目录、profile、features、源码身份、命令、筛选、失败修复及未完成的外部验证。重复检查只针对新修改、已知失败或未解决问题；没有这些理由时复用仍匹配当前源码的证据。
