---
related_code:
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/scripts/validation-stages.ps1
  - .codex/skills/zircon-dev/scripts/managed-cargo-storage.ps1
  - tools/zircon-session.ps1
  - .github/workflows/ci.yml
implementation_files:
  - .codex/skills/zircon-dev/scripts
  - tools/zircon-session.ps1
plan_sources:
  - docs/plans/milestone-validation-policy.md
  - docs/plans/zircon_tooling/session_coordinator/01
tests:
  - .github/workflows/ci.yml
  - tools/session_coordinator/tests
doc_type: workflow-reference
---

# 验证工作流

## 选择命令

普通改动从最小完整批次开始：一个 package-scoped `cargo check`，再运行该里程碑行为的 focused `cargo test`。只有共享 DTO、根 manifest、锁文件或 release wave 才升级到多包/全 workspace。

```powershell
# 只渲染命令，不运行 Cargo
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 `
  -DryRun -SkipBuild -SkipTest -RunExportPlatformContract -ExportContractPlatform headless

# 受管的运行时聚焦测试
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 `
  -Package zircon_runtime -SkipBuild -LibTests -TestFilter <filter>
```

支持 `development`、`release`、`profiling` profile；支持 `reuse`、`compact`、`diagnostic` storage mode。`-IgnoredTests` 只能和明确的非空 `-TestFilter`、`-LibTests`/`-TestTarget` 一起使用。

## 阶段

1. **Admission**：解析 repo、toolchain、目标目录和 compatibility key；拒绝不受管路径。
2. **Source snapshot**：创建 immutable validation copy 并记录 digest。
3. **Build**：在 coordinator Cargo lane 中运行 check/build。
4. **Test**：运行 unit/lib/bin/ignored 或 contract matrix。
5. **Evidence**：收集 stdout、stderr、exit code、timing、artifact digest。
6. **Release**：终止 session、释放 lane、删除该 job 的 scratch。

任何阶段失败都要保留 job id 和阶段名。不能用“终端没有输出”推断没有运行；以 JSON 状态为准。

## 平台矩阵

`-RunExportPlatformContract` 覆盖 windows、linux、macos、android、ios、web_gpu、wasm、headless。它验证导出 policy 的选择逻辑；不模拟每个平台的 GPU/窗口。Linux CI 失败只在需要 Linux 专属复现时才转 WSL。

## CI 对齐

`.github/workflows/ci.yml`、profile-feature-contract 和 export policy workflow 是 CI 权威。Windows 本地 validator 应尽量复用同一 profile、feature 和 test filter。不要为规避 busy lane 手工加 `--target-dir` 或启动第二个 pool。
