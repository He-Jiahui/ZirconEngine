---
related_code:
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
implementation_files:
  - zircon_app/src/entry/tests/profile_bootstrap.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
tests:
  - rustfmt --edition 2021 --check zircon_app/src/entry/tests/profile_bootstrap.rs
  - git diff --check -- zircon_app/src/entry/tests/profile_bootstrap.rs
  - managed validate-matrix.ps1 -Package zircon_app -SkipBuild
doc_type: milestone-detail
---

# Frameworks05 ManagerResolver 显式 Core 生命周期消费者修复

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M4 | App profile bootstrap 显式 Core owner | `completed` | 2026-07-15 | 三个失败夹具改为在 resolver 使用期保留 `CoreHandle`；两轮 Windows 受管 `zircon_app` 全包门通过；同一受管产物精确结果为 139 passed、0 failed、1 ignored。 |

## Scope Delivered

- `runtime_bootstrap_excludes_editor_module`、`runtime_bootstrap_without_linked_virtual_geometry_keeps_base_pipeline_lightweight` 与 `linked_runtime_render_feature_descriptors_rebuild_default_pipelines` 不再把唯一强 `CoreHandle` 移入 resolver。
- 三处统一以 `ManagerResolver::new(core.clone())` 保留明确的调用作用域 owner，服务仍通过 versioned `ManagerServiceHandle` 在 use point 解析。
- `ManagerResolver` 继续只保存 `CoreWeak`；未恢复旧 `resolve_*_manager`、Arc holder、兼容 wrapper、alias、shim 或 fallback。

## Fresh Testing Evidence

- Failure RED：来源受管 App 门为 136 passed、3 failed、1 ignored，三项均返回 `ServiceUnavailable("CoreRuntime")`。
- `rustfmt --edition 2021 --check zircon_app/src/entry/tests/profile_bootstrap.rs`：通过。
- `git diff --check -- zircon_app/src/entry/tests/profile_bootstrap.rs`：通过。
- current-source scan：`ManagerResolver::new(core)` 零命中；三个旧消费者均显式 clone Core。
- Windows managed App gate 首次重试未启动 Cargo：协调器以 `unmanaged_artifacts_detected` 拒绝外部 Text03 日志目录 `E:\cargo-targets\text-m4-logs`。该目录由原会话通过受管作业自然收敛；本切片未手工删除或接管。
- Windows managed App gate（verbose）job `5be1a16de4874780909f087eace94213`：`validate-matrix.ps1 -Package zircon_app -SkipBuild -VerboseOutput`，exit 0。
- Windows managed App gate（cache-warm）job `a1998dc2e40140568d4491a73aa07580`：`validate-matrix.ps1 -Package zircon_app -SkipBuild`，exit 0。
- 直接执行 job `a1998dc2e40140568d4491a73aa07580` 生成的同一测试产物：139 passed、0 failed、1 ignored；三个原失败 profile bootstrap 测试均通过，Runtime V2 operation ABI/loader 测试保持通过。

## Review

2026-07-15 独立只读 review：Critical=0 / Important=0。复核确认三处消费者均保留显式强 `CoreHandle`，resolver 字段仍为 `CoreWeak`、构造仅 downgrade、use point 显式 upgrade；没有强持有、伪服务、fallback、旧 `resolve_*_manager`、Arc holder、alias、shim 或兼容 wrapper。

## Remaining Gate

- coordinator 已将 lifecycle `manager-resolver-weak-core-test-consumer-drift` 原子迁移为 Editor03 下的 fixed 记录并更新来源/修复计划链接；本切片没有剩余修复门。提交仍并入 Frameworks05 M4 的既有精确 milestone manifest，不创建第二份 M4 manifest。
