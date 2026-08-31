---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-error-owner-typed-error-guard-stale-path
origin_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
fixing_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
origin_child_dir: docs/plans/zircon_editor/editor/10
fixing_child_dir: docs/plans/zircon_runtime/frameworks/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/animation_resource.rs
  - zircon_runtime/src/core/runtime/error.rs
tests:
  - cargo test -p zircon_runtime --lib project_asset_manager --locked --jobs 1 -- --nocapture --test-threads=1
  - python -m unittest tools.tests.test_frameworks_01_runtime_error_owner_boundary tools.tests.test_frameworks_02_core_error_single_source
---

# Frameworks01：Runtime error owner 硬切后 typed-error guard 仍读取旧路径

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 来源执行者：`editor10-project-reference-regression-20260717`
- 来源执行切片：manager-owned project source index current-source Runtime gate
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 交接原因：失败发生在 `zircon_runtime` lib-test 编译期，最低共享原因是 Frameworks01 已删除 error owner 后，Runtime code-review typed-error guard 没有同步切到 kernel-owned `core/runtime/error.rs`。

## 失败现象与复现证据

- Editor10 受管 job `5aeb9ef5c17f4023bb66266dd1a31697` / run `35f40dc64a6a44a0bb44b3c91b357fe3` 于 2026-07-18 06:20:51+08 自然结束，exit 101、0 tests、live PIDs empty。
- exact command：`cargo test -p zircon_runtime --lib project_asset_manager --locked --jobs 1 --color never -- --nocapture --test-threads=1`。
- raw stderr：`.codex/state/session-coordinator/cargo-runs/5aeb9ef5c17f4023bb66266dd1a31697/35f40dc64a6a44a0bb44b3c91b357fe3/stderr.log`。
- 唯一编译 error：`animation_resource.rs:111` 的 `include_str!("../../../../core/framework/error.rs")` 读取已物理删除的文件；rustc 最终报告 1 previous error / 370 warnings。
- 该 job 另有三条 post-start Render source race，因此不能作为 Editor10 immutable acceptance；但 missing-file error 本身由同一 rustc 的 raw diagnostic 直接证明，与 Editor10/Render 业务逻辑无关。

## 最低共享层根因

Frameworks01 已把 `CoreError/CoreResult` 从 `core/framework/error.rs` 硬切到 kernel owner `core/runtime/error.rs` 并删除旧文件；`review_f6_core_resource_registry_rename_uses_resource_error` 虽已改为验证 `ResourceRegistryError`，仍通过旧 framework 路径执行负向断言。测试镜像因此同时宣称 hard cut 和依赖被删除 owner，违反 single-source/hard-cut 合同。

## 架构修复验收

- typed-error guard 的负向 `CoreError` 检查读取 `core/runtime/error.rs`，不得恢复 `core/framework/error.rs`。
- 保留 ResourceRegistryError 与 CoreError 不重新连接的负向断言：runtime error 不包含 legacy resource variants/`ResourceRegistryError`，resource owner 与 registry 不引用 `CoreError`。
- Frameworks01/Frameworks02 两个静态 owner guards 保持 GREEN，并执行 fresh source-bound `zircon_runtime --lib` gate，证明没有其他已删除 error-owner 路径残留。
- 修复进入 Frameworks01 successor exact manifest、独立复审和受管提交后，才能生成 canonical fixed return；Editor10 必须在该 SHA 与 Render freeze 后新建 reservation，不能复用 `5aeb...`。

## 禁止临时方案

- 禁止恢复旧 `core/framework/error.rs` 文件、re-export、alias 或 shim。
- 禁止删除 F6 typed-error guard 或弱化 single-source 负向断言。
- 禁止把 `5aeb...` 的 source-raced exit 101 记为 Editor10 功能失败或 acceptance。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-07-18 | Frameworks01 M1 Runtime error owner / typed-error guard mirror | `open-raw-compile-evidence-recorded` | 已记录 job/run/command/raw stderr；唯一 error 精确定位到 `animation_resource.rs:111` 读取已删除 framework error owner。Editor10 的 5 条 app 可见性错误在本轮不再出现。 | Frameworks01 owner 以 successor scope 修改 guard、执行静态 2/2 + fresh Runtime lib gate、独立复审、managed commit/fixed return；随后 Editor10 重建源快照。 |
| 2026-07-18 | Frameworks01 M1 Runtime error owner / typed-error guard mirror | `fix-implemented-static-review-green` | successor `frameworks01-m1-runtime-error-owner-dag-prerequisite-r2-20260718` 已领取原 23 路径加 guard/failure record 的 exact25 scope；guard 已将负向 `CoreError` 来源硬切到 `core/runtime/error.rs`。Standalone guard 2/2、Frameworks01/02 Python owner guards 2/2、rustfmt/diff-check 和旧 include 扫描均 GREEN，exact25 独立复审 Critical/Important/Minor = 0/0/0；未恢复旧文件、alias、re-export 或 shim。 | 等待 fresh exact25 source manifest、受管 Runtime lib gate、managed commit/fixed return；Runtime04 `d995...` 的旧源运行必须自然结束且不得复用为验收。 |

## 修复结果与回传

Resolving state：guard 硬切、下层静态验证与 exact25 独立复审已完成，尚未完成 fresh source-bound Cargo、受管提交和 canonical fixed return。当前没有恢复旧架构，也不声明 Editor10 或 Runtime04 验收通过。

## r3 集成源码验证恢复（2026-08-08）

- successor `frameworks01-runtime-error-owner-guard-validation-r3-20260808` 已无冲突领取 failure、
  typed-error guard、kernel error owner、已删除 framework error 墓碑与两份 Python owner guards 的 exact6。
- 当前集成源码仍由 `animation_resource.rs` 读取 `core/runtime/error.rs`；旧
  `core/framework/error.rs` 不存在，`ResourceRegistryError` 与 `CoreError` 的双向负断言均保留，
  未增加兼容文件、alias、re-export 或 shim。
- 状态保持 `open` / `resolving_failure`：实现、静态证据与原 exact25 独立复审 C0/I0/M0 已完成；
  r3 fresh managed Runtime lib receipt、terminal GREEN、fixed return 与 milestone commit 仍 pending。

## r4 stale-snapshot 前向恢复（2026-08-13）

- r3 ticket `802643d2e9724be59c087693681132dc` 的 source manifest 已相对当前集成源码漂移，
  现已记录为 `snapshot_stale`；该 ticket 不得作为 GREEN/RED 或 closeout 证据复用。
- fresh successor `frameworks01-runtime-error-owner-guard-validation-r4-20260813` 已接管当前 exact5
  实体路径，另以 `zircon_runtime/src/core/framework/error.rs = null` 固定 hard-cut tombstone。
- 当前 `HEAD` 已包含 F6 guard 对 `core/runtime/error.rs` 的正确读取；working tree 中同文件的
  `animation/sequence/apply.rs` 到 `compiled.rs` 更新属于后续结构迁移的必要适配，未恢复任何旧
  error owner、alias、re-export 或 shim。
- fresh Python owner guards 2/2 GREEN，scoped `git diff --check` GREEN（仅工作区既有 CRLF
  提示）。exact6 不可变二次审查 pre/post fingerprint 均为
  `566ad849fe2ed68b0aa4b120a0f664185bc9cc51be184637c4f18d82d098870c`，drift 0，
  Critical/Important/Minor = 0/0/0。当前状态仍为 `open` / `resolving_failure`：待 fresh
  managed Runtime lib terminal、原子提交与 canonical fixed return 完成后才可关闭。

## r5 current-HEAD 隔离验证与共享 harness 阻塞（2026-08-19）

- successor `frameworks01-runtime-error-owner-guard-closeout-r5-20260819` 以
  `25e09a23178000f2e783ce2143cf70a8b118d404` 为 coordinator base，在当前 HEAD
  `bea1acf91b909525ab1759e2c800858b0eda6528` 复核 F6 owned blobs；guard 实现相对 base 与
  HEAD 均未漂移，`core/framework/error.rs` 继续物理不存在，未恢复 compatibility owner。
- fresh 静态命令
  `python -m unittest tools.tests.test_frameworks_01_runtime_error_owner_boundary tools.tests.test_frameworks_02_core_error_single_source`
  以 2/2 GREEN 自然结束，耗时 17.569 秒。该证据只证明 error-owner/source-shape 合同，
  不替代 Rust 编译门或独立复审。
- immutable validation copy `5356527e2a644513bae4f9a12935ce52`、input manifest
  `acc4075842055a5786c19bb666b85928613d5522e93ce206d2988403bf822751` 固定同一 F6
  owned closure，并将外部 `zr_vm` 固定到 commit
  `503fb72163cd20ddf32a38f8a330083712f5d648`。exact command 保持本记录 frontmatter
  声明的 `project_asset_manager` gate；run `08933d1868e1429c8e23c3294c222f47` 自然以
  101 结束，54 compile errors、1,455 warnings、0 tests。错误码集合为
  E0063/E0277/E0282/E0308/E0425/E0433/E0596/E0599/E0624；原始旧 include/E0432 未复现，
  因此 F6 stale path 不在本轮诊断中，但 exact gate 仍不是 GREEN。
- 该隔离副本按 Session-only 合同不覆盖共享工作树 715 份 foreign dirty Rust blobs。
  `docs/plans/mvp/00/2026-08-18-m02-m03-runtime-editor-app-compile-convergence.md` 已将 HEAD
  上 54 条 Runtime lib-test 错误归属为 current public-API hard-cut 后的共享 harness 收敛，
  禁止由 Frameworks01 增加 alias、feature bypass 或上层兼容路径。
- 为区分 HEAD 基线与共享工作树修复，外部受管 job
  `b65365a33a0d443d9f0d96410abf5546` 于 21:37:23--22:16:01 自然运行并 release，
  live process tree 为空、exit 1。它完成 `zircon_runtime` production build 并进入 lib-test
  target，但 retained D 盘 target 只有 `zircon_runtime-0c255ba51a4c1888.d`（2,673,546
  bytes），没有对应 test executable，故只能证明 test harness 在链接/执行前仍 RED；该
  validate-matrix job 没有持久化逐条 rustc diagnostic，不能据此猜测或越权修改 foreign owner。
- 状态保持 `open` / `resolving_failure`。只有共享 harness 修复原子进入 HEAD 后，才能按新
  source hash 重建 F6 immutable copy；旧 RED copy 和共享工作树 job 均不得复用为 closeout
  ticket。随后仍需 exact Rust GREEN、fresh 静态 GREEN、真实独立复审、canonical Failure
  return 与 coordinator milestone commit。

## r7 current-HEAD owner guard confirmation（2026-08-22）

- successor `frameworks01-open-failure-convergence-r7-bee4c707-20260822` 在 current HEAD
  `bee4c707b714738346b49bba15c59468b8bd9b39` 重新核对 typed-error owner。guard 继续读取
  `core/runtime/error.rs`，旧 `core/framework/error.rs` 物理不存在；tracked Runtime Rust source
  对旧 literal path 的命中为 0，未恢复 alias、re-export、shim 或第二 error owner。
- fresh 静态命令
  `python -m unittest tools.tests.test_frameworks_01_runtime_error_owner_boundary tools.tests.test_frameworks_02_core_error_single_source`
  为 2/2 GREEN，测试本体耗时 23.743 秒，端到端命令耗时 25.41 秒。该结果只确认当前源码的
  single-source/hard-cut contract，不替代 Rust lib-test 编译门。
- r7 首次成功入池前的两次申请均在 Cargo 进程创建前由共享 reuse pool 拒绝；这两次申请没有
  产生可作为 GREEN/RED 的 r7 Cargo job。其后的首次 terminal run 由下节记录并取代“尚未启动”
  状态；current-source review、canonical fixed return 与 coordinator commit 仍未完成，Failure
  保持 `open` / `resolving_failure`。

## r7 managed current-source compile result（2026-08-22）

- managed job `e9edd5eca8cf4341838d6b0e836a8c1e` 于 17:34:13--17:44:12 在 D 盘
  retained reuse pool 自然运行，17:44:17 release，live process tree 已清零。exact
  `zircon_runtime --lib project_asset_manager` gate 在 lib-test 编译阶段以 Cargo `101` / wrapper
  `1` 结束，0 个筛选测试执行，因此不是 GREEN。
- retained rustc fingerprint 包含 19 条实际 compiler error 与 1 条 abort summary、1,457 条
  warning；error code 分布为 E0599=14、E0282=2、E0061=1、E0063=1、E0425=1。旧
  `core/framework/error.rs` include、F6 E0432 和 resource/runtime error-owner 回连均为 0，
  因而本次 RED 没有复现 F6 stale-path 根因，但也不能替代整条 Rust GREEN。
- E0425 来自编译期间的 foreign `scene/tests/inspection.rs` 快照；该文件在 job 结束后于
  17:46:28 前向更新，current hash `65d689bf9f3d0d8628fd38a06e6b0be2c15867f7addb0e0f3ca3efe876d9c42b`
  已不存在报错表达式。仍可在 current source 复核的阻塞分别路由到 Runtime07 的
  `failure-2026-08-22-world-deserialize-node-cache-initializer.md`（World 新缓存字段初始化）和
  Runtime74 的 `failure-2026-08-22-ui-asset-binding-canonical-loader-api-tests.md`
  （canonical loader 测试）；新增 binding-transaction 调用仍由同一 Runtime74 active owner
  持有。建议其性能采样按一次 applied target 调用
  `commit(1, 0, Vec::new(), true)`；coordinator 因 delayed patch 会把 r7 从
  `resolving_failure` 降为 `waiting_lease` 而拒绝入队，Frameworks01 不绕过状态门，也不吸收或
  改写这些 foreign blobs。
- 待上述 owner 收敛并释放同一 reuse pool 后，必须以最终 exact4 attribution 创建稳定
  validation copy，重跑该 gate 并取得 current immutable review。Failure 继续保持 `open` /
  `resolving_failure`，不生成 fixed return，也不提交里程碑。
- formal `materialize-cargo` request `4f881849b5e54961ba4b9715147c5c41` 创建 job
  `93bd9c11c31b4e989151e6e5aceaec1e`，并成功固定 external source
  `E:\Git\zr_vm@ceadabbfa1436fcd0f2cc6ffd788b45120bb2acc`（source hash
  `f6eaf3aedead7538bd0da34fe4c6ecbc43c3cff0c8e2406a107c7c64db54e1d2`，仅包含两个
  Rust binding package，未吸收 external dirty blobs）。copy 在 Cargo 启动前于
  `materialization_prepare` 以 `validation_copy_baseline_drift` 终止：closure 中
  `ui/surface/binding_targets.rs`、`ui/template/asset/compiler/{binding_param_resolver,control_scope}.rs`
  与两份 `ui/tests/*/control_scope.rs` 已偏离 baseline，但尚无可供副本消费的 attribution。
  五条路径均属于 active Runtime74 r3 immutable scope；Frameworks01 不扩大 exact4 或转移 owner
  绕过该门。该 job 没有生成 input manifest、没有启动 Cargo，失败副本已从 F 盘清理。

## r9 current-source owner guard confirmation（2026-08-24）

- On current HEAD `f811b3bf474d70347199772a175422333dfb36f6`, the former
  `core/framework/error.rs` owner remains physically absent and the typed-error guard continues to
  read the single kernel owner at `core/runtime/error.rs`; no compatibility file or projection was
  introduced.
- Fresh command
  `python -B -m unittest tools.tests.test_frameworks_01_runtime_error_owner_boundary tools.tests.test_frameworks_02_core_error_single_source -v`
  is GREEN 2/2 in 42.618 seconds. This is exact current-source static evidence only.
- The four foreign `zr_rhi_wgpu` compile-blocker blobs are unchanged from managed Runtime job
  `246fdaf5d6c443f9b71149d744b5675e`, so r9 did not consume another shared Cargo window to reproduce
  the same pre-test failure. Exact Rust GREEN, current immutable review, canonical Failure return,
  and coordinator milestone commit remain pending; this Failure stays `open`.
