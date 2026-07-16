# Frameworks 02 M1 Current-Source Acceptance

Plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
Milestone: M1
Status: completed
Files: ["docs/plans/zircon_runtime/frameworks/02/2026-07-16-m1-current-source-acceptance.md"]

> 本文件按当前源码重新验收 `02-module-kernel-and-lifecycle-unification.md` 的 M1；历史 2026-07-13 focused binary 证据仅用于定位覆盖面，不替代本轮在 2026-07-14 runtime 改动之后的 fresh gate。

| 里程碑 | 完成项目 | 状态锚 | 日期 | 当前状态与证据 |
|---|---|---|---|---|
| M1 内核语义落地 | Current-source lifecycle, ordering, typed-error and rollback audit | `frameworks_02_m1_current_source_all_gates_passed_native_acceptance_ready` | 2026-07-17 | **current source committed and audited; package and all M1 focused gates passed; native M1 acceptance ready**。生产源码确认 `InitLevel::{Kernel,Services,Scene,Editor,Post}`、`ModuleLifecycle::{build,ready,finish,cleanup}`、`ModuleDescriptor::{init_level,module_dependencies,lifecycle}`、层级/依赖拓扑排序，以及 `CoreError::{DuplicateModule,MissingModuleDependency,ModuleInitLevelViolation,ModuleDependencyCycle,ModuleReadyTimeout,ModuleActivationRollback,ModuleBatchActivationRollback}` 为当前内核契约。严格对照父计划又发现并修复旧 `ZirconError` 并行错误面：先写 `tools/tests/test_frameworks_02_core_error_single_source.py`，RED 精确命中旧 enum；随后将 channel/thread variants、task helper、asset worker、root/prelude 与结构守卫硬切到 `CoreError/CoreResult`。Frameworks02 M1 的 12 个精确 owner 与 Frameworks05 Text parent owner 均已由协调器落盘到 `ad2c6f989cfff927ff5679467ca0cc71e2e20c0e`；canonical failure lifecycle 随后以 `6c8957ab86925d0eed3e55a5914157dc891f382e` 从 open handoff 闭环为 Frameworks02 fixed record 与 Frameworks05 return record。single-module 与 batch activation 都在 ready 后进入 finish；batch 在全部模块 ready 后统一 finish，失败按依赖逆序 best-effort cleanup，并保留主错误和 rollback 错误。scoped rustfmt 与 diff-check 通过；`python tools/tests/test_runtime_init_level_naming.py` 4/4 passed，生产源码未发现退役 `InitLevel::Servers` 或 `"servers"` alias。Windows managed package job `49243a98297542d2ac583e6a76815993` 在 Plugins02 Sound manifest 漂移前已 exit 0；独立复审为 Critical 0 / Important 0 / Minor 0。首次 `module_lifecycle` job `121cae2a94ef4bdca36777b63738ab65` 在执行测试前被根 `Cargo.lock` 漂移拒绝；最低失败位为 Plugins02 当前 `cpal` → `kira` 依赖硬切尚未同步双 lockfile，canonical handoff 为 `docs/plans/zircon_plugins/02/failure-2026-07-17-sound-kira-root-lockfile-drift.md`。Plugins02 更新根 lock 后，Frameworks05 root `--locked` retry `8c17339dc12d4e39ac925d4c7be5e81e` 先以 exit 0 证明共享解析层恢复；随后 M1 retry job `4943374443604c469145c5352713adc3` / run `9359d3f821164a49b19ff4ef6288f03d` 通过 11/11，其中 9 个为 M1 lifecycle 直接测试、2 个为同名过滤命中的 M3 RuntimePlugin hard-cut 回归；job `4d7d6cbbbf5b4b25b6d4008b63bc9c98` / run `86ba1331a0144f54a3c63e73e4a17182` 通过 activation-order 4/4；job `8df0c41b56564934b13cded29fff0431` / run `a05ba666833c4197a684e6d7c8ebddd0` 通过 descriptor-default 1/1。M1 直接 focused 覆盖为 14/14，附带 M3 RuntimePlugin hard-cut 2/2；最终独立复审已完成，当前只待 native `milestone prepare/validate/review/commit`，本记录不把“ready”冒充已提交的 workflow acceptance。 |

## Scope Delivered

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

### Frameworks05 fixing-plan 已闭环依赖（排除于 M1 manifest）

- `zircon_runtime/src/text/parallel/raster_pool.rs`：Frameworks05 Text hard-cut parent owner 已随 `ad2c6f989cfff927ff5679467ca0cc71e2e20c0e` 进入 Git 历史；该文件不重复进入 M1 状态提交。
- canonical lifecycle `text-raster-pool-zircon-error-consumer` 已由 coordinator `failure return` 闭环；`6c8957ab86925d0eed3e55a5914157dc891f382e` 精确提交 Frameworks02 `fixed-*` 与 Frameworks05 return record，并删除 open failure artifact。该回传提交不重复进入 M1 manifest。

## Fresh validation manifest

- Package compile：Windows managed `cargo check -p zircon_runtime --lib --locked`。
- Lifecycle behavior：Windows managed `cargo test -p zircon_runtime --lib --locked module_lifecycle`；当前过滤器实际覆盖 11 个测试，其中 9 个为 M1 直接契约（8 个 build/ready/finish/cleanup、timeout 与 rollback 行为，加上 `module_lifecycle_default_hooks_are_noop_and_ready`），另有 2 个 M3 RuntimePlugin lifecycle hard-cut 回归。M1 直接测试总数统计仍按 9 计，不把 collateral coverage 冒充 M1 新增覆盖。
- Descriptor/order behavior：Windows managed `cargo test -p zircon_runtime --lib --locked module_activation_order`，按当前源码覆盖 4 个层级/依赖排序测试；`module_descriptor_defaults_to_post_without_module_dependencies` 另用 exact filter 覆盖 1 个，三个命令合计 14 个不重复 M1 直接测试。
- Naming hard cut：`python tools/tests/test_runtime_init_level_naming.py`。
- Static：scoped rustfmt、`git diff --check`、退役 `Servers`/compat alias 扫描、plan-output audit、docs exact-scope convention gate。
- Error single source：`python tools/tests/test_frameworks_02_core_error_single_source.py`，以及 Frameworks05 Text raster handoff 的受管 focused gate。

## Fresh Testing Evidence

- `python tools/tests/test_frameworks_02_core_error_single_source.py`：1/1 passed；锁定 canonical enum、root/prelude、task/asset typed mapping 与全部生产 Rust stale consumer 扫描。
- `python tools/tests/test_runtime_init_level_naming.py`：4/4 passed。
- scoped `rustfmt --edition 2021 --check` 与 exact `git diff --check`：passed。
- plan-output audit：passed；failure handoff validator：171 artifacts / 0 errors；共享 Git index：0 staged paths。
- Windows managed package gate：job `49243a98297542d2ac583e6a76815993` / run `fbe4cdbbb3d446228d9c22e4f748a7dd` 执行 `cargo check -p zircon_runtime --lib --locked --color never`，1m15s 完成，exit 0；507 warnings、0 errors。该 job 使用 coordinator warm compatibility `841a130ffbd3fd2e938e76b488988119044b676acced751dae7166d95d7f1025` 并已正常 release。
- Windows managed focused 第一次尝试：reservation `0dc4d21a3e9147ceaa254ec9c68a9d9d`、job `121cae2a94ef4bdca36777b63738ab65`、run `2bb7a3ae86374e7c966e985568af5ca0` 原样执行 `cargo test -p zircon_runtime --lib --locked module_lifecycle --color never`，6 秒内 exit 101；未进入编译或测试，stderr 为根 `Cargo.lock` 需要更新。底层复现 job `2782d9dc535f479988e47505b77e5b43` / run `440583b5bf3b47ecbd5dba14f2f7df09` 的 `cargo metadata --format-version 1 --locked --offline` 同样 exit 101。只读差异确认 `zircon_plugins/sound/runtime/Cargo.toml` 已将旧可选 `cpal` 改为 `kira = "0.12.2"`，而当前根 lock 中 `zircon_plugin_sound_runtime` 仍列 `cpal` 且无 `kira` package；故最低失败位是 Plugins02 双 lockfile 原子同步，不是 Frameworks02 lifecycle 实现。canonical handoff `docs/plans/zircon_plugins/02/failure-2026-07-17-sound-kira-root-lockfile-drift.md` 已 open；不移除 `--locked`、不修改 foreign Sound owner、不把本次解析失败计作 lifecycle 测试结果。
- Windows managed lifecycle retry：Plugins02 root lock 更新后，Frameworks05 原始 root `--locked` retry job `8c17339dc12d4e39ac925d4c7be5e81e` 先以 1/1、exit 0 验证共享解析层与 Runtime lib-test 可用；随后 reservation `9b15eafb14bb42aa8d4665bd599cec55`、job `4943374443604c469145c5352713adc3`、run `9359d3f821164a49b19ff4ef6288f03d` 原样执行 `cargo test -p zircon_runtime --lib --locked module_lifecycle --color never`，11 passed / 0 failed / 8168 filtered，exit 0，3.14s 完成。11 个命中中 9 个是 M1 lifecycle 直接契约，另外 2 个是 `runtime_plugin_lifecycle_hard_cuts_to_kernel_module_lifecycle` 与 `runtime_plugin_embedded_descriptor_uses_kernel_module_lifecycle`；本记录保留该附带 M3 证据但不改变 M1 9 个直接测试计数。
- Windows managed activation-order：reservation `263913f09f8943638e678c8b632a20b4` 在更早 Render warm-pool job `1eca3d1ab532400e95428876edaa8328` 运行期间保持 pending，并续期到 04:52:32+08；该外部 job 自然释放后，reservation 原样消费为 job `4d7d6cbbbf5b4b25b6d4008b63bc9c98` / run `86ba1331a0144f54a3c63e73e4a17182`，执行 `cargo test -p zircon_runtime --lib --locked module_activation_order --color never`，4 passed / 0 failed / 8175 filtered，exit 0，18.98s 完成。四个命中精确覆盖层级/声明依赖排序、missing dependency、later-level violation 与 same-level cycle。
- Windows managed descriptor default：reservation `a4d18e6417fb4c6198117787a3f1200f`、job `8df0c41b56564934b13cded29fff0431`、run `a05ba666833c4197a684e6d7c8ebddd0` 执行 `cargo test -p zircon_runtime --lib --locked module_descriptor_defaults_to_post_without_module_dependencies --color never`，1 passed / 0 failed / 8178 filtered，exit 0，5.78s 完成；确认默认 `InitLevel::Post` 且无隐式 module dependency。三组命令合计 M1 直接测试 14/14，并附带执行 M3 RuntimePlugin lifecycle hard-cut 2/2。
- 独立最终复审：Critical 0 / Important 0 / Minor 0；确认无 shim/alias、a6 maintenance 为 3-file exact、M1 owned/adopted 边界与 raster 排除准确。package gate 后的当前记录复审再次为 Critical 0 / Important 0 / Minor 0；确认 `ad2c6f98` focused membership 13/13、`6c8957ab` failure return 三文件精确、当前状态仍未提前声明 accepted。
- Frameworks05 Text raster direct gate：Windows managed job `711bd7035e1f4e62a0def56214a6151b`，`cargo test -p zircon_runtime --lib --locked text_raster_worker_pool --color never`，5 passed / 0 failed / 8173 filtered，exit 0；fixing session 已正常 release job。Text parent owner 已落盘到 `ad2c6f989cfff927ff5679467ca0cc71e2e20c0e`，当前 HEAD 再次运行 `python tools/tests/test_frameworks_02_core_error_single_source.py` 为 1/1 passed；failure lifecycle 已由 `6c8957ab86925d0eed3e55a5914157dc891f382e` 回传 fixed。
- `python tools/tests/test_runtime_plan_status_archive_ownership.py`：1/2，唯一失败为 foreign `05-scene-editor-boundary-closeout.md: missing numbered-archive status/evidence records`；本记录不把该仓库级缺口写成 M1 通过，也不归因于 Runtime15 archive-path retained hunk。

## Review

- 最终独立 reviewer 对当前记录、`ad2c6f98` hard-cut owner、全部 managed job/run、14/14 M1 直接测试计数、Plugins02 Sound 外部归因和父计划/MVP 前置关系完成只读核对，结论为 Critical 0 / Important 0 / Minor 0；确认可进入 native milestone review，但提交前不把 `completed` 冒充 workflow `accepted`。
- 本里程碑只验收内核生命周期、排序、类型化错误和 rollback，不提前声明 M2 profile/plugin-group 组装完成。
- 不把历史 focused binary 或本轮窄测试冒充全 Runtime wave gate；全量 `cargo test -p zircon_runtime --lib --locked` 仍按 policy §4 留在依赖完整波次收口。
- 不恢复旧生命周期 trait、错误字符串 facade、`InitLevel::Servers` alias 或第二套 descriptor/排序器。
