---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: manager-resolver-weak-core-test-consumer-drift
origin_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_editor/editor/03
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
related_code:
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_app -SkipBuild
resolved_at: 2026-07-15
---


# Frameworks05：ManagerResolver 弱 Core 生命周期消费漂移

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-15 | Editor03 为 Runtime V2 hard-cut 执行 `zircon_app` 受管全包门；140 项中 136 通过、1 ignored，3 个 profile bootstrap 用例统一失败于 `ServiceUnavailable("CoreRuntime")`。Runtime V2 loader、required operation tail、missing-function 与 operation ABI 测试均已通过；剩余失败来自 Frameworks05 manager hard-cut 后的测试消费者生命周期漂移。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 来源执行切片：M3.2 Runtime V2 operation ABI / loader 的 App 上层受管验收门
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：三个失败点都消费 Frameworks05 新 `ManagerServiceHandle + CoreWeak ManagerResolver` 合同；Editor03 不拥有 Core manager 生命周期。
- 生命周期键：`manager-resolver-weak-core-test-consumer-drift`

## 失败现象与复现证据

Windows 受管命令 `./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_app -SkipBuild` 当前编译通过，测试二进制执行结果为 `136 passed; 3 failed; 1 ignored`。失败为：

- `runtime_bootstrap_excludes_editor_module`；
- `runtime_bootstrap_without_linked_virtual_geometry_keeps_base_pipeline_lightweight`；
- `linked_runtime_render_feature_descriptors_rebuild_default_pipelines`。

三者都以 `let resolver = ManagerResolver::new(core);` 把唯一强 `CoreHandle` 按值移入构造器。Frameworks05 新 `ManagerResolver::new` 只保存 `core.downgrade()`，构造结束后该唯一强引用被释放；随后的 `rendering_handle` / `render_framework_handle` 在 `upgrade_core()` 中稳定返回 `ServiceUnavailable("CoreRuntime")`。

同一二进制内 Editor03/Runtime10 相关测试均已通过，包括 required V2 operation function 缺失拒绝、V2 operation tail layout、消费端不重查 required capability，以及 operation DTO ABI 拒绝。

## 最低共享层根因

Frameworks05 已把 manager resolver 硬切为不跨生命周期强持有 Core 的弱引用模型，但 App profile bootstrap 测试仍沿用旧 resolver-owning-Core 的消费方式。最低修复层是 manager hard-cut 的消费者迁移：调用方必须在 resolver 使用期间保留明确的强 `CoreHandle`，而不是让 resolver 隐式拥有它。

## 架构修复验收

- 三个 profile bootstrap 消费者在 `ManagerResolver` 使用期间显式保留强 `CoreHandle`，并继续通过 versioned `ManagerServiceHandle` 在 use point 解析。
- 不得把 `ManagerResolver` 改回强持有 Core，不得恢复旧 `resolve_*_manager`、Arc holder、兼容 wrapper 或隐式 fallback。
- 聚焦执行上述三个测试均通过，再重跑 `zircon_app` 受管全包门；回传结果必须保留 140 项总计与 Runtime V2 测试通过证据。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止让 `upgrade_core()` 在 Core 已销毁时返回伪服务，禁止跳过三个测试或弱化 render framework 可用性断言。
- 禁止把 resolver 改为新的跨帧/跨域强 Arc owner；强 `CoreHandle` 必须由真实调用作用域明确持有。

## 修复结果与回传

- 根因：Three App profile bootstrap consumers moved the only strong CoreHandle into a ManagerResolver that intentionally stores CoreWeak, so subsequent use-point resolution observed a destroyed CoreRuntime.
- 架构修复：Retained an explicit strong CoreHandle in each consumer scope by constructing ManagerResolver from core.clone(); versioned ManagerServiceHandle resolution remains unchanged and no strong resolver owner or legacy Arc holder was restored.
- 验证：Windows managed zircon_app package gate completed with exit code 0; the 140-test library binary has 139 passed, 0 failed, 1 ignored, and all binaries plus doc-tests completed.
- 回传：Editor03 may resume the full App gate and Runtime V2 acceptance; Frameworks05 weak resolver lifetime contract is preserved.
