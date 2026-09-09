---
related_code:
  - zircon_hub/src/tauri_app/runtime_state
  - zircon_hub/src/build
  - tools/cargo-zircon/src/main.rs
  - tools/cargo-zircon/src/product_receipt_cli
implementation_files:
  - zircon_hub/src/tauri_app/runtime_state
  - tools/cargo-zircon/src
plan_sources:
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
tests:
  - zircon_hub/src/tauri_app/runtime_state
  - tools/cargo-zircon/src
doc_type: recipe-guide
---

# Hub 自动化方案与可复用流程

本页把公开 API 组合成可复制的工程流程。每个方案都强调 admission、取消、证据和恢复；不要把这些步骤压缩为一个没有状态记录的脚本。

## 方案 A：创建项目并首次启动

```text
CreateProjectRequest::new
  -> validate_launch_fields
  -> target_root conflict check
  -> template materialization
  -> validate_project_root
  -> select SourceEngineInstall
  -> EditorLaunchCommand::from_staged_engine
  -> snapshot + recent history
```

```rust
let request = CreateProjectRequest::new(template, "Weather", parent, Some(engine_id));
request.validate_launch_fields()?;
let root = request.target_root();
// Tauri action owns creation and cleanup; external callers should not copy templates manually.
```

目标已存在、模板未知或引擎不可用时在启动进程前返回；不要创建半成品项目再让编辑器处理。

## 方案 B：源码引擎构建

```text
validate_source_engine
  -> BuildCommandOptions
  -> for_editor_runtime
  -> run_build_command(token)
  -> BuildExecutionReport
  -> record_build
  -> staged_editor_executable_exists
```

失败时保留最近一次报告和 capture；成功时才更新 last-build。CI 还应把 profile、jobs、Python、Cargo 和 source commit 写入 receipt request。

## 方案 C：插件检查与 receipt

```powershell
cargo zircon plugin sync-manifest --root $repo
cargo zircon plugin check-manifest --root $repo
cargo zircon plugin check --root $repo --artifact-root $artifacts
cargo zircon plugin validate $plugin --artifact $artifact
cargo zircon product-receipt verify --receipt $receipt --trust-registry $trust --artifact-root $artifacts
```

sync 产生写入，check-manifest 只读漂移检查，check 扫描 workspace，validate 检查单包/动态库，receipt verify 检查发布物。每一步都应保存 stdout/stderr 和退出码。

## 方案 D：打包和设备安装

```rust
let request = ProjectPackageRequest::new("Weather", project_root, output_root);
let token = TaskCancellationToken::new(task_id);
match package_project(&request, &token)? {
    TaskExecutionOutcome::Completed(report) => {
        // Build DeviceInstallRequest from report.package_dir.
    }
    TaskExecutionOutcome::Cancelled => { /* show cancelled and cleanup */ }
}
```

打包不会复制 `.git`/`target`；设备安装只接收 package 目录。安装后校验 file receipt 和 device root containment。

## 并发与幂等

一个 project scope 同时只允许一个 destructive action。使用 action ID admission、task cancellation 和 storage receipt；重复请求应 replay 已有结果或返回 conflict。不要通过 UI 禁用按钮代替 Rust 侧 admission。

## 观察性模板

每次自动化运行写入：

```text
action_id, task_id, project_key, engine_id, profile,
started_at, finished_at, status, exit_code,
message_id, capture_dir, receipt_id, artifact_digest
```

路径和 URL 脱敏，secret 永不落盘。日志摘要用于界面，完整日志只放在受控 capture 目录。

## 故障矩阵

| 阶段 | 失败证据 | 可重试动作 |
| --- | --- | --- |
| 请求校验 | `CreateProjectRequestError` | 修正字段 |
| 源码校验 | `SourceEngineValidation` | 选择正确 checkout |
| 子进程 | report exit code | 修复 toolchain/参数 |
| manifest | diagnostic code | sync 后审阅 diff |
| artifact | native validation | 重新构建同一 target |
| receipt | digest/trust error | 重新签发或更新 registry |

## 与 Unreal/Godot 的对照

Unreal AutomationTool 的 cook/package/deploy 可串成 CI pipeline；Godot 常以导入后导出为主。Zircon 方案把项目、引擎、插件和 receipt 组成显式阶段，每阶段均可取消、重放和审计。

## 生产检查清单

- [ ] 所有 destructive action 有 admission 和 cancellation。
- [ ] 所有外部进程使用参数数组。
- [ ] 所有产物有 digest、receipt 或 report。
- [ ] 失败不会删除用户已有目录。
- [ ] CI 目标目录与 toolchain 固定。
- [ ] 日志已脱敏且可按 ID 定位。

## 相关页面

- [项目与历史](projects-history.md)
- [引擎构建](engine-builds.md)
- [项目包与 Receipt](packages-receipts.md)
- [安全与排障](operations-security.md)
