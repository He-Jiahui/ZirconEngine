# Frameworks 02 M1 Current-Source Acceptance

> 本文件按当前源码重新验收 `02-module-kernel-and-lifecycle-unification.md` 的 M1；历史 2026-07-13 focused binary 证据仅用于定位覆盖面，不替代本轮在 2026-07-14 runtime 改动之后的 fresh gate。

| 里程碑 | 完成项目 | 状态锚 | 日期 | 当前状态与证据 |
|---|---|---|---|---|
| M1 内核语义落地 | Current-source lifecycle, ordering, typed-error and rollback audit | `frameworks_02_m1_current_source_audited_managed_cargo_pending` | 2026-07-16 | **current source audited; managed Cargo pending**。生产源码确认 `InitLevel::{Kernel,Services,Scene,Editor,Post}`、`ModuleLifecycle::{build,ready,finish,cleanup}`、`ModuleDescriptor::{init_level,module_dependencies,lifecycle}`、层级/依赖拓扑排序，以及 `CoreError::{DuplicateModule,MissingModuleDependency,ModuleInitLevelViolation,ModuleDependencyCycle,ModuleReadyTimeout,ModuleActivationRollback,ModuleBatchActivationRollback}` 为当前内核契约。严格对照父计划又发现并修复旧 `ZirconError` 并行错误面：先写 `tools/tests/test_frameworks_02_core_error_single_source.py`，RED 精确命中旧 enum；随后将 channel/thread variants、task helper、asset worker、root/prelude 与结构守卫硬切到 `CoreError/CoreResult`。提交 `2c7824ef` **只新增 canonical Frameworks05 failure handoff**；Text raster consumer 修复与 managed focused gate `711bd7035e1f4e62a0def56214a6151b` 5/5 已完成，但源码仍属于未提交的 Frameworks05 parent Text batch，待 parent exact milestone commit 与 failure return，不把 handoff hash 或工作树修复冒充源码提交。single-module 与 batch activation 都在 ready 后进入 finish；batch 在全部模块 ready 后统一 finish，失败按依赖逆序 best-effort cleanup，并保留主错误和 rollback 错误。scoped rustfmt 与 diff-check 通过；`python tools/tests/test_runtime_init_level_naming.py` 4/4 passed，生产源码未发现退役 `InitLevel::Servers` 或 `"servers"` alias。独立最终复审为 Critical 0 / Important 0 / Minor 0。仍待 Windows managed package/lifecycle/order gates、Frameworks05 parent exact milestone commit 与 failure return；本行尚不声明 M1 accepted。 |

## 当前源码验收范围

### 只读验收 owner（本批不修改）

- `zircon_runtime/src/core/runtime/lifecycle.rs`
- `zircon_runtime/src/core/runtime/descriptors/module_descriptor.rs`
- `zircon_runtime/src/core/runtime/descriptors/module_order.rs`
- `zircon_runtime/src/core/runtime/handle/activation.rs`
- `zircon_runtime/src/core/runtime/handle/activation/module_lifecycle.rs`
- `zircon_runtime/src/core/runtime/handle/activation/batch.rs`
- `zircon_runtime/src/core/runtime/tests/activation/behavior/module_lifecycle.rs`
- `zircon_runtime/src/core/runtime/tests/registration/behavior/module_order.rs`
- `docs/zircon_runtime/core/runtime/lifecycle.md`

### Frameworks02 M1 本批修改范围

- `zircon_runtime/src/core/framework/error.rs`：删除并行错误 enum，将 channel/thread variants 迁入 canonical `CoreError`。
- `zircon_runtime/src/core/mod.rs`：根面只导出 `CoreError/CoreResult`。
- `zircon_runtime/src/core/runtime/tasks/mod.rs`：`spawn_named_thread` 返回 `CoreResult` 并映射 `CoreError::ThreadSpawn`。
- `zircon_runtime/src/asset/pipeline/worker_pool.rs`：`request` 返回 `CoreResult` 并映射 `CoreError::ChannelSend`。
- `zircon_runtime/src/asset/tests/pipeline/worker_pool.rs`：直接匹配 `CoreError::ChannelSend`，不再只比对 display 文本。
- `zircon_runtime/src/prelude.rs`：只精选导出 canonical `CoreError/CoreResult`。
- `zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/animation_resource.rs`：根导出结构断言硬切为 `CoreError/CoreResult`。
- `tools/tests/test_frameworks_02_core_error_single_source.py`：锁定 enum、根导出、prelude、task 与 asset 映射，并扫描全部生产 Rust consumer。
- `docs/zircon_runtime/core/framework/error.md`
- `docs/zircon_runtime/core/runtime/tasks.md`
- `docs/zircon_runtime/asset/worker_pool.md`
- `docs/plans/zircon_runtime/frameworks/02/2026-07-16-m1-current-source-acceptance.md`：本验收记录随 M1 精确 manifest 提交。

### 已提交的 current-docs 前置依赖

- maintenance commit `a6a3bc72990c8af99ae227bf65364ad78ccf6d64` 以三文件精确 manifest 更新 `docs/engine-architecture/core-runtime-service-registry.md`、Runtime02 与 Runtime04 parent plan 的当前错误合同。Frameworks02-owned hunk 将 live facade 改为 `CoreError/CoreResult`、补入 M1 plan/test 反向映射并删除三份 current docs 的旧符号；`core-runtime-service-registry.md` 同文件已有的 Runtime15 `dependency_cycles.rs` related-code、rustfmt 清单与结构收敛证据原样保留并在该 maintenance commit 中显式采用。该文档已独立提交，因此不进入后续 M1 business manifest。

### 同文件保留并显式采用的外部有效 hunk

- `zircon_runtime/src/core/framework/error.rs` 同时保留 Frameworks05 已写入的 `ServiceIdentityIndexExhausted`、`ServiceUnavailable`、`StaleServiceHandle`。这些 variants 不是 M1 新功能，但与删除第二错误 enum 位于同一 canonical owner 文件；最终整文件 manifest 会显式采用并由本轮 package gate/独立审查共同覆盖，不隐式吞并。
- `zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/animation_resource.rs` 同时保留 Runtime15 的 numbered-archive path 同步；本轮只把根导出断言从旧错误面改为 `CoreError/CoreResult`。最终整文件 manifest 会披露该依赖；仓库级 archive ownership test 当前另有 Scene plan foreign failure，不把该失败记为本 hunk 通过。

### Frameworks05 fixing-plan 依赖（排除于 M1 manifest）

- `zircon_runtime/src/text/parallel/raster_pool.rs`：未跟踪的 Text hard-cut 文件，由 `frameworks05-zircon-error-text-consumer-failure-20260716` 单独拥有与提交。
- `docs/plans/zircon_runtime/frameworks/05/failure-2026-07-16-text-raster-pool-zircon-error-consumer.md`：`2c7824ef` 已提交 open handoff；后续状态更新/return 归 fixing-plan，不进入 M1 manifest。

## Fresh validation manifest

- Package compile：Windows managed `cargo check -p zircon_runtime --lib --locked`。
- Lifecycle behavior：Windows managed `cargo test -p zircon_runtime --lib --locked module_lifecycle`，按当前源码覆盖 9 个测试：8 个 build/ready/finish/cleanup、timeout 与 rollback 行为，加上 `module_lifecycle_default_hooks_are_noop_and_ready`。
- Descriptor/order behavior：Windows managed `cargo test -p zircon_runtime --lib --locked module_activation_order`，按当前源码覆盖 4 个层级/依赖排序测试；`module_descriptor_defaults_to_post_without_module_dependencies` 另用 exact filter 覆盖 1 个，三个命令合计 14 个不重复 M1 直接测试。
- Naming hard cut：`python tools/tests/test_runtime_init_level_naming.py`。
- Static：scoped rustfmt、`git diff --check`、退役 `Servers`/compat alias 扫描、plan-output audit、docs exact-scope convention gate。
- Error single source：`python tools/tests/test_frameworks_02_core_error_single_source.py`，以及 Frameworks05 Text raster handoff 的受管 focused gate。

## 已完成的本轮静态证据

- `python tools/tests/test_frameworks_02_core_error_single_source.py`：1/1 passed；锁定 canonical enum、root/prelude、task/asset typed mapping 与全部生产 Rust stale consumer 扫描。
- `python tools/tests/test_runtime_init_level_naming.py`：4/4 passed。
- scoped `rustfmt --edition 2021 --check` 与 exact `git diff --check`：passed。
- plan-output audit：passed；failure handoff validator：167 artifacts / 0 errors；共享 Git index：0 staged paths。
- 独立最终复审：Critical 0 / Important 0 / Minor 0；确认无 shim/alias、a6 maintenance 为 3-file exact、M1 owned/adopted 边界与 raster 排除准确。
- Frameworks05 Text raster direct gate：Windows managed job `711bd7035e1f4e62a0def56214a6151b`，`cargo test -p zircon_runtime --lib --locked text_raster_worker_pool --color never`，5 passed / 0 failed / 8173 filtered，exit 0；fixing session 已正常 release job。该文件属于尚未提交的整个 `zircon_runtime/src/text/` parent batch，不能单文件提交；handoff 继续保持 open，等待 Frameworks05 parent exact milestone commit 与 `failure return`。
- `python tools/tests/test_runtime_plan_status_archive_ownership.py`：1/2，唯一失败为 foreign `05-scene-editor-boundary-closeout.md: missing numbered-archive status/evidence records`；本记录不把该仓库级缺口写成 M1 通过，也不归因于 Runtime15 archive-path retained hunk。

## 边界

- 本里程碑只验收内核生命周期、排序、类型化错误和 rollback，不提前声明 M2 profile/plugin-group 组装完成。
- 不把历史 focused binary 或本轮窄测试冒充全 Runtime wave gate；全量 `cargo test -p zircon_runtime --lib --locked` 仍按 policy §4 留在依赖完整波次收口。
- 不恢复旧生命周期 trait、错误字符串 facade、`InitLevel::Servers` alias 或第二套 descriptor/排序器。
