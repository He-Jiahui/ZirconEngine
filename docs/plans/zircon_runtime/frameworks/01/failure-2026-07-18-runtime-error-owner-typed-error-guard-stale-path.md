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
