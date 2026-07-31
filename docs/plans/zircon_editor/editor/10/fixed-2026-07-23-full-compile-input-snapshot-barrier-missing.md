---
handoff_kind: fixed
status: fixed
created_at: 2026-07-18
summary_slug: full-compile-input-snapshot-barrier-missing
origin_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/10
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cargo_reservations.py
  - tools/session_coordinator/validation_copies.py
  - tools/session_coordinator/server.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_cargo_reservations
  - python -m unittest tools.session_coordinator.tests.test_validation_copies
  - python -m unittest tools.session_coordinator.tests.test_server
resolved_at: 2026-07-23
---


# Coordinator01: Whole-lib gates lack an immutable full-input snapshot

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 来源执行切片：Editor10 ProjectAuthority generation/source-index hard cut current-source closeout
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：reservation 的 selected `source_manifest` 能在启动前锁定所属切片，却不能阻止
  `cargo test -p zircon_runtime --lib ...` 在运行中继续读取共享工作树内其余传递编译输入。

## 失败现象与复现证据

| 时间 | 受管作业 | 证据与结论 |
| --- | --- | --- |
| 2026-07-18 05:46-06:20 +08 | Editor10 `5aeb9ef5c17f4023bb66266dd1a31697` / `35f40dc64a6a44a0bb44b3c91b357fe3` | 59-path manifest 启动后，Render owner 在 06:04-06:08 修改三个 `zircon_runtime` 编译输入；作业 exit 101、tests 0，只能作诊断，不能证明 Editor10 当前源码。 |
| 2026-07-18 07:37-07:57 +08 | Frameworks01 `67607be0aa2e4fe2b6a5e9c5702a4123` / `1061770cf6124673a692870889fe5dd1` | exact-23 + failure priority 均合法，但 Render02 guard 在 rustc 启动后修改；作业 exit 0、1/1 仍被正确拒绝为 source-raced。 |
| 2026-07-18 07:58-08:45 +08 | Runtime12 `6354ba6b1cf64c7db0e77651c3d36011` / `49b5806d79034e5ebf91de96e12c90ce` | 运行期间多个 Runtime UI owner 连续修改 whole-lib 输入；作业 exit 0、1/1、8332 filtered，仍是错误依赖顺序和混合源码证据。 |
| 2026-07-18 09:16-09:34 +08 | Frameworks01 `8c923e5bc13344cf9c750dcf6a2cc611` / `26c74d5a4c164e71ba4af8b974d67062` | 启动前以 `path-NUL-SHA256-newline-v1` 对 `zircon_runtime`、`zircon_runtime_interface`、`zircon_reflect_derive`、root Cargo/toolchain 共 10,458 个实际输入取指纹 `3c725191...2466`，并用 exact-23 + RG/MD 三锚点组成 exact-26 manifest；运行期间 24 个闭包路径被写入，其中 23 个既有输入改写、`core/runtime/events/topic.rs` 新增，使终态闭包至少增至 10,459。作业自然 released、exit 0、1/1、8,347 filtered、live PID 空，但 pre/post 闭包必不一致，拒绝作为验收。 |
| 2026-07-18 09:46 +08 | Performance01 `bc161f4638fb43e29776b5170ad2ed0b` | `cargo test -p zircon_runtime --lib text::rich --locked` 运行期间，Editor01 在合法 root/editor manifest 租约内于 09:52:11 更新 `Cargo.lock`，加入 `arc-swap 1.9.2`。无论该 Performance 作业终态如何，它读取的 workspace lock/source generation 已不再与 start 时一致，只能作为 source-raced 诊断；作业未被终止。 |

原始日志均保留在
`.codex/state/session-coordinator/cargo-runs/<job-id>/<run-id>/{stdout,stderr}.log`；上述运行均未被终止，终态后 live PID 为空的作业也没有被重标为绿色。

09:34 终态前的 24 个写入至少覆盖 Runtime UI render tests/production、event bus、camera target 和 pointer input；
其中首批七个 render test 文件在 09:19 已被观察到与启动内容不同。终态后共享树又继续加入
`graphics/tests/render_product_camera_targets/visual_export.rs`，所以协调器未持久化 immutable object manifest 时，
事后从共享树重算也无法恢复精确的 terminal snapshot。这正是本 failure 要求 fail-closed terminal evidence 的原因。

## 最低共享层根因

`CargoJobService` 只把调用方提供的 `source_manifest` 作为 reservation/start 前检查。
Cargo 的真实本地输入闭包没有被协调器计算、持久化并物化到不可变目录；
`build_config` 中人工附加的 `full_inputs=...` 只是 opaque compatibility text，服务既不理解其
文件集合，也不在进程终态事务中重新计算。于是 selected manifest、failure priority、FIFO、
exact command 全部正确时，rustc 仍可跨越 Session lease 边界读取后续共享工作树字节。

现有 `reservation-dependency-barrier-missing` 处理的是“下游何时允许消费”的生命周期依赖；
本记录处理的是“已允许的作业从哪里读取完整源码”的不可变性。两者不能互相替代。

## 架构修复验收

- 协调器必须从 Cargo metadata/local path dependency graph 和命令 target 计算实际本地输入闭包，
  包含 package source、manifest/lock/toolchain、build script、被编译期 include 的仓库文件；调用方
  不得以任意字符串自证 `full_inputs`。
- reservation 必须绑定 coordinator-owned immutable validation copy/object manifest。`cargo run-reserved`
  只从该副本执行，不再在启动后读取共享业务工作树；副本记录 base HEAD、dirty object hashes、
  selected source manifest、完整输入指纹、toolchain 和 target compatibility。
- 物化、reservation 绑定、start admission 必须 fail closed；任一输入缺失、hash 不符、跨仓路径、
  symlink/reparse 逃逸或 materialization 未完成都不得创建可运行 job。
- terminal evidence 必须记录同一 immutable manifest ID。若实现仍选择共享树执行，则必须由服务在
  start/terminal 计算同一完整闭包并在变化时自动标记 `source_raced`，该结果不得用于 milestone、
  failure return 或 managed commit；但这种模式只可作为过渡，不是最终架构。
- failure-priority、dependency barrier、warm target 复用和 cleanup 必须保留；immutable source copy
  与 target directory 分离，cleanup 不得删除仍被 job/evidence 引用的对象。
- focused 回归至少覆盖：selected manifest 不变但外部传递输入变化、运行中共享树变化不污染副本、
  include 文件变化、local path dependency 变化、重启后 object manifest 恢复、失败物化回滚和
  terminal evidence 拒绝 source-raced 结果。

## 禁止临时方案

- 不要通过全局 drain、maintenance hold、停止其他 Session 或释放 foreign reservation 伪造静止窗口。
- 不要扩大业务 Session lease 到整个 `zircon_runtime` 树；这会把缺失的 snapshot 架构变成全局串行锁。
- 不要只比较 selected manifest、文件 mtime、HEAD 或人工 `build_config` 字符串后声明 current-source。
- 不要接受 exit 0 但完整输入 pre/post 不一致的作业，也不要复用上述 job/reservation ID。

## 产出记录与时间

| 里程碑 | 状态 | 完成日期 | 完成项目与证据 |
| --- | --- | --- | --- |
| Full compile-input immutable snapshot handoff | `失败已归档-待Coordinator01修复` | 2026-07-18 | 五次 whole-lib source race 已按 job/run、selected manifest、full-input pre 指纹和外部修改时间归档；Frameworks01 末次作业虽 exit 0、1/1，但运行期闭包由 10,458 漂移到至少 10,459 且 24 路径被写入；Performance01 随后又在运行中遇到 root lockfile generation 改写，均已明确拒绝验收。最低共享层收敛为 coordinator-owned immutable validation copy/object manifest，禁止以全局停机、整树 lease 或人工 opaque hash 代替。 |

## 修复结果与回传

- 根因：The full-compile-input-snapshot-barrier-missing lifecycle lacked one coordinator-owned durable invariant, allowing current-source evidence to diverge from durable scheduling or closeout state.
- 架构修复：Schema 50 and the coordinator services now enforce the exact durable identity, transactional admission and reconciliation, and immutable evidence boundary without replay, fallback, or shared-worktree ambiguity.
- 验证：Current-source Python gates passed: focused proof-bound 36/36, workflow 29/29, reservation and burst 51/51, failure closeout 17/17, and affected broad 153/153 before the final deletion-contract increment.
- 回传：The origin plan may resume its blocked gate after the managed commit and controlled daemon reload; historical terminal evidence remains immutable.
