---
related_code:
  - tools/zircon-session.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/scripts/managed-cargo-storage.ps1
  - .codex/skills/zircon-dev/scripts/coordinator-request-recovery.ps1
  - docs/plans/milestone-validation-policy.md
implementation_files:
  - tools/zircon-session.ps1
  - .codex/skills/zircon-dev/scripts
  - tools/session_coordinator
plan_sources:
  - docs/plans/milestone-validation-policy.md
  - docs/plans/zircon_tooling/session_coordinator/01
tests:
  - tools/session_coordinator/tests
  - .github/workflows/ci.yml
doc_type: operations-reference
---

# Session Coordinator 与受管验证

Session Coordinator 是验证资源和证据的调度边界。它把“谁在验证、验证什么源码、占用哪个 Cargo pool、结果在哪里”记录成可恢复的 job，而不是让多个终端各自猜测 target 目录。

## 核心对象

| 对象 | 含义 |
| --- | --- |
| Session | 人或自动化任务的长期身份与写入范围 |
| Validation ticket/job | 一次具体验证请求、兼容性描述和状态 |
| Cargo pool/lane | 按 repo、平台、toolchain、profile、feature 等键隔离的编译资源 |
| Validation copy | 从源快照物化的不可变工作副本 |
| Lease | 对共享源码或 Cargo lane 的暂时所有权 |
| Evidence | 命令、stdout/stderr、exit code、source digest 和 artifact 清单 |

## Windows 优先策略

普通 `check`、`build`、`test` 和 milestone 验证使用 Windows PowerShell。Cargo target 只能位于 `D:\cargo-targets`、`E:\cargo-targets`、`F:\cargo-targets`、`targets` 或 `ZirconBuilds` 下。仓库内 `target/`、用户目录和未受管临时目录都拒绝。

WSL 只在 Linux 专属失败、Linux-only 工具或用户明确要求时使用，并把 `/mnt/d|e|f/...` 作为 Windows 目标根的映射；不能把 WSL home 当 Cargo target。

## 推荐流程

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 `
  -Package zircon_runtime `
  -SkipBuild -LibTests -TestFilter export_visual_evidence
```

里程碑边界使用 `tools/zircon-session.ps1 milestone validate`，它会提交完整 compatibility description、创建 validation copy、申请 lane、运行命令并终止清理。`-DryRun` 只渲染命令，不创建 coordinator state 或目录。

## 兼容性键和资源模式

键至少包含 repository identity、平台、Rust toolchain、架构、workspace identity、Cargo profile、features、link mode、测试/构建开关和 target projection。`reuse` 保留热 pool；`compact` 减少调试物料；`diagnostic` 保留符号；三者不能互用同一不兼容配置。

一个 primary lane 同时只允许一个 owner；资源不足时必须等待 FIFO job。若提交后客户端超时，先通过 coordinator 查询原 job，不得启动并行 retry。

## 失败处理

- `cargo_cpu_lane_reserved`：表示 lane 已被占用，不是 Cargo 编译失败。
- source/materialization 错误：检查 validation copy 的源 digest 和外部 sibling 路径。
- 测试失败：把最小根因归到实际 owner，写入对应编号计划的 `failure-*.md`，不要在 MVP 计划复制正文。
- 终端失败：保留 job id、命令、返回码和最后一段输出，再释放 lease。

## 机器可读证据

脚本的 JSON readiness、ticket、heartbeat、finish 和 release 响应是自动化消费面。文本日志只是展示层；CI 或后续 agent 应解析字段而不是依赖自然语言。验证结束后报告 Windows/WSL、target 目录、profile、命令和结果，不能只写“已测试”。
