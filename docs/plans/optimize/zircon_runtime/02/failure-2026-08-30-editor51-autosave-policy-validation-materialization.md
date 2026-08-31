---
handoff_kind: failure
status: open
created_at: 2026-08-30
summary_slug: editor51-autosave-policy-validation-materialization
origin_plan: docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
fixing_plan: docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime/02
fixing_child_dir: docs/plans/optimize/zircon_editor/51
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/recovery/tests.rs
  - zircon_editor/src/core/recovery/autosave/policy.rs
tests:
  - validation ticket 61aca11bde774a6ab8631f955b10b0f3
  - validation copy job 82843037bdc64039a7af9679b68ff155
  - validation ticket 400eb6f9257344fa96918cf960eae2a2
  - validation copy job 531ec957da654f13a5790f5cc436d29d
  - validation ticket fdd06df7674f4565864cd76ced76aee4
  - validation copy job 9aa9ec8590fc40e3841176d71c4eef59
  - validation ticket b55455281a3045e1bc08d3e03f1f32af
  - validation copy job 7d08353ac33047ef8b0d5cbfaa37216c
  - validation ticket 9e9a9800a9af4b4eae11890e0823f0f5
  - validation copy job 1ae4f26f77824fcc94fb88c39114258a
  - validation ticket 1e4404defad84227a5a02c04545c1cbe
  - validation copy job b3306a0a8097409f9b1193729c37d7c7
  - validation ticket 3c720102c9a04ba5a0da053da5064e38
  - validation copy job 6bcd24fd6ef04657a93c3ba7048c3984
  - cargo job 2ef1fdd329a94a74b444ff698d2f2ca2
  - validation ticket 2a3327de64c9456384e2d6067a02fc77
---

# Editor51: autosave policy is outside validation materialization

## Source executor

- Origin plan: `docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md`
- Origin slice: Editor525/Editor526 focused Release validation
- Fixing plan:
  `docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md`
- Handoff reason: the missing compile-time resource is the Editor51 recovery/autosave policy
  implementation, below Runtime02 optimization ownership.

## Failure evidence

Validation ticket `61aca11bde774a6ab8631f955b10b0f3`, copy job
`82843037bdc64039a7af9679b68ff155`, failed during closure planning with
`validation_copy_compile_time_resource_missing`. The source
`zircon_editor/src/core/recovery/tests.rs` compiles the autosave module closure, but
`zircon_editor/src/core/recovery/autosave/policy.rs` is not materializable from the ticket source
closure.

Current hashes are:

- `tests.rs`:
  `c933d6fac3129bfc5b624781834cc3ac34351b06a351779e0546aed2aa34bb0d`
- `policy.rs`:
  `2903a5b9d249b93370c5c15c256f982d7761cfd16b0195f73da2660b307965f5`

`policy.rs` exists as an untracked, unattributed source. `tests.rs` retains stale attribution to
cancelled Session `editor16-m3-shared-recents-r2-successor-20260816`. Runtime02 did not edit,
claim, attribute, or add either source to its candidate.

## Lowest shared-layer root cause

The recovery test module now depends on a split autosave policy source that has not entered a
durable coordinator-owned source closure. Any Editor validation copy that discovers the test module
can therefore fail before Rust compilation, independent of the focused optimization under test.

## Architecture acceptance

- Editor51 legally reconciles and attributes the exact autosave policy plus its recovery test/module
  closure without absorbing unrelated recovery files.
- The split `autosave/policy.rs` remains the canonical policy owner; its implementation is not
  copied into `tests.rs` or a compatibility module.
- Managed Editor validation advances beyond closure planning and compiles the recovery/autosave
  module closure.
- Runtime02 reruns one aggregate Editor validation for the affected optimization batches after the
  source is integrated or legally transferred.

## Forbidden workarounds

- Do not delete or weaken the recovery tests that import `AutosavePolicy`/`AutosaveJobPolicy`.
- Do not inline the policy implementation into `tests.rs` or restore a parallel autosave policy.
- Do not claim the unattributed source through maintenance override or absorb it into a Runtime02
  commit candidate.

## Return contract

Return the exact ownership/integration request ID, final hashes for the recovery test/module closure,
and managed validation evidence that `autosave/policy.rs` is materializable. Runtime02 will then
resume one batched Editor validation without polling the fixing Session.

## Current remediation evidence

Editor51 Session `root-editor51-autosave-policy-materialization-20260831` now owns the exact
two-source recovery closure without maintenance override. Lease request
`a70d0c56582545f7992908abec64e651` renewed the exact closure and this handoff pair; exact-hash
attribution request `eaa1bc00c47d4893804df4573b6ab054` established the source closure before
managed validation.

The original materialization defect is no longer reproducible. Ticket
`400eb6f9257344fa96918cf960eae2a2` used source manifest
`68bf55c7ca73ff1a7c59bbf0e3334c85601a3bfe7184ec9e669b0bfb1cea995d`; copy job
`531ec957da654f13a5790f5cc436d29d` completed closure planning and entered Cargo at
2026-08-31 03:01 +08:00. It therefore materialized `autosave/policy.rs` instead of returning
`validation_copy_compile_time_resource_missing`. The Cargo child later ended with exit `-1` while
building third-party dependencies, without a Rust diagnostic and before reaching `zircon_editor`.

The exact-manifest retry ticket `fdd06df7674f4565864cd76ced76aee4`, copy job
`9aa9ec8590fc40e3841176d71c4eef59`, again advanced from materialization to Cargo. It failed in
third-party crates with exit `101` because `sccache` could not create
`E:/cargo-targets/zircon-engine/scratch/.../temporary`, reporting Windows `os error 3`. This is a
managed target/scratch infrastructure failure, not a recovery source diagnostic. The Failure stays
open because the acceptance contract also requires the recovery/autosave closure to compile; no
fixed record or integration candidate is emitted yet.

Coordinator event and Cargo-job correlation isolates the infrastructure root cause. The v4 Cargo
job is `216b7502c5004f818dcc296f3ecc9bd5`, but every failing `sccache` temporary path is under
`scratch/84f3507f1dee480184e94f5cbaf9fdb2/temporary`; `84f350...` is an unrelated Frameworks01
Cargo job that ran from 2026-08-31 03:29:51 to 03:32:31 +08:00 and had already released before v4
started at 03:44:17. Artifact events show the shared `scratch` root was deleted only at 03:47:11,
after v4 had terminated, so the current job was not cleaned while running. The durable failure is a
persistent `sccache` server retaining the previous job-scoped `TEMP` after that job's scratch was
removed. `cargo_runner.py` and its regression owner are currently held by separate Session
`root-coordinator-validation-baseline-r3-20260831` in `waiting_validation`; Editor51 records the
diagnosis but does not take over that live Tooling/Coordinator owner.

Coordinator 以稳定的 E 盘 server TEMP 重启后，v5 ticket
`b55455281a3045e1bc08d3e03f1f32af` 再次使用完全相同的 source manifest
`68bf55c7ca73ff1a7c59bbf0e3334c85601a3bfe7184ec9e669b0bfb1cea995d`。copy job
`7d08353ac33047ef8b0d5cbfaa37216c` 已通过 closure planning/materialization；随后在
`run_start` 阶段以 `cargo_cpu_burst_resource_denied` 终止，原因是 CPU warm lane 被占用且
isolated burst admission 未获接受。该终态没有启动 Cargo，也没有产生 Rust diagnostic；它证明
stable TEMP 修复后的 materialization 仍正常，但不能证明 recovery/autosave 编译通过。v6 只在
CPU lane 可用后重提，避免把纯资源拒绝误报为源码失败或反复占用验收队列。

v6 ticket `9e9a9800a9af4b4eae11890e0823f0f5` 继续使用同一 source manifest
`68bf55c7ca73ff1a7c59bbf0e3334c85601a3bfe7184ec9e669b0bfb1cea995d`；copy job
`1ae4f26f77824fcc94fb88c39114258a` 再次以 `materialized` 终态越过 closure planning，并在
2026-08-31 05:20:30 +08:00 进入 Cargo。关联 Cargo job
`dbe32cb4cded4c7f90f56932008419b1` 于 05:24:46 被协调器标记为 `orphaned`，ticket 随后以
`exitCode=-1` 终止；该 run 的 stdout/stderr 与 terminal tail 均为空，未产生第三方依赖或
Zircon Rust diagnostic。该结果仍属于运行监管基础设施终止，不能转 fixed；但三次独立 copy 已
一致证明原始 `validation_copy_compile_time_resource_missing` 不再出现。

v7 ticket `1e4404defad84227a5a02c04545c1cbe`、copy job
`b3306a0a8097409f9b1193729c37d7c7` 第四次使用相同 exact two-source manifest，并以
`status=materialized`、`materialization_phase=materialized` 终态完成 source closure。Ticket 在
2026-08-31 05:45 +08:00 的 `run_start` 阶段因
`cargo_sccache_rebind_busy` 失败：另一个 Cargo/rustc 进程仍在使用 managed sccache binding，worker
拒绝在活跃编译期间替换它。该次没有创建 Cargo job、没有运行 Rust 编译、没有 stdout/stderr source
diagnostic。原始 materialization 缺陷已由 4 个独立 copy 一致排除，但 canonical acceptance 仍要求
recovery/autosave closure 编译，因此 Failure 保持 open，等待 sccache binding 空闲后的单次托管重试。

2026-08-31 07:23 +08:00 已重新 claim 并 current-hash attribute 精确三路径；attribution request
`429d78d632744dd3a50da271daec4f61`。v8 ticket `3c720102c9a04ba5a0da053da5064e38`
继续使用完全相同的 two-source manifest `68bf55c7ca73ff1a7c59bbf0e3334c85601a3bfe7184ec9e669b0bfb1cea995d`；
copy job `6bcd24fd6ef04657a93c3ba7048c3984` 第五次完成 exact closure materialization，随后 Cargo job
`2ef1fdd329a94a74b444ff698d2f2ca2` 于协调器重启期间被标记为 `orphaned`。该 run 的 exit code、stdout、
stderr 和 terminal tail 均为空，没有第三方依赖或 Zircon Rust diagnostic。原始 compile-time resource 缺失已
稳定消失，但 compile acceptance 仍未取得；Failure 继续 open，不提前生成 fixed、integration 或 return completion。

v9 ticket `2a3327de64c9456384e2d6067a02fc77` 已使用同一 exact two-source manifest
`68bf55c7ca73ff1a7c59bbf0e3334c85601a3bfe7184ec9e669b0bfb1cea995d` 排队，命令仍为
`cargo test -p zircon_editor --lib --locked autosave_job_policy_uses_background_misc_and_the_save_mutex_group`。
本轮只做一次提交且不轮询队列；`tests.rs` 与 split `autosave/policy.rs` 的字节和哈希均未变化。
在 ticket 形成可核验的 compile GREEN 前 Failure 保持 open，不创建 fixed/integration/return completion。

Current source hashes remain:

- `zircon_editor/src/core/recovery/tests.rs`:
  `c933d6fac3129bfc5b624781834cc3ac34351b06a351779e0546aed2aa34bb0d`
- `zircon_editor/src/core/recovery/autosave/policy.rs`:
  `2903a5b9d249b93370c5c15c256f982d7761cfd16b0195f73da2660b307965f5`

## 产出记录与时间

| 时间 | 状态 | 完成项目与当前门禁 |
|---|---|---|
| 2026-08-31 04:02 +08:00 | `exact-ownership-established / compile-time-resource-materialized / cargo-green-blocked-by-stale-sccache-server-temp` | 以 Editor51 exact owner 接管 `tests.rs + autosave/policy.rs`，保持 split policy module，不使用 maintenance override；两次 exact-manifest copy 均越过原 `validation_copy_compile_time_resource_missing` 并启动 Cargo，证明 compile-time resource 已物化。v4 在第三方依赖阶段因持久 `sccache` server 仍引用已释放 Frameworks01 job `84f350...` 的 E 盘 job-scoped `TEMP` 而终止；当前 v4 job `216b...` 并未被提前清理，也未产生 Zircon recovery 源码诊断。Failure 保持 open，integration request、fixed record 与 Runtime02 aggregate rerun 均等待独立 Tooling/Coordinator owner 修复 managed cache/temp 生命周期。 |
| 2026-08-31 05:11 +08:00 | `stable-temp-materialization-confirmed / cargo-not-started / cpu-lane-capacity-denied / v6-deferred` | v5 ticket `b55455281a3045e1bc08d3e03f1f32af`、copy job `7d08353ac33047ef8b0d5cbfaa37216c` 在重启后的 Coordinator 上再次完成 exact two-source materialization，随后仅因 warm CPU lane occupied 且 burst admission 未接受而在 `run_start` 终止；未启动 Cargo、无 Rust diagnostic。Failure 保持 open，等待 lane 空闲后再提交 v6，不循环制造资源拒绝。 |
| 2026-08-31 05:29 +08:00 | `v6-materialized / cargo-run-orphaned / no-rust-diagnostic / failure-open` | v6 ticket `9e9a9800a9af4b4eae11890e0823f0f5`、copy job `1ae4f26f77824fcc94fb88c39114258a` 第三次完成 exact materialization 并进入 Cargo；关联 job `dbe32cb4cded4c7f90f56932008419b1` 在协调器重启/孤儿回收后以 ticket `exitCode=-1` 终止，stdout/stderr 为空。原 compile-time resource 缺失已消失，但尚无成功编译证据，故不创建 fixed、不生成 integration candidate；继续推进非验收基础设施并仅在受管运行稳定后重试。 |
| 2026-08-31 06:24 +08:00 | `v7-materialized / cargo-not-started / managed-sccache-binding-busy / failure-open` | v7 ticket `1e4404defad84227a5a02c04545c1cbe`、copy job `b3306a0a8097409f9b1193729c37d7c7` 第四次完成 exact two-source materialization；worker 在 `run_start` 检测到 Cargo/rustc 仍活跃并以 `cargo_sccache_rebind_busy` fail-close，未创建 Cargo job、无 Rust diagnostic。materialization return evidence 已满足，但 compile acceptance 尚未满足，因此 ownership/integration closeout 仍等待一次 binding 空闲后的托管编译。 |
| 2026-08-31 07:23 +08:00 | `current-attribution-refreshed / v8-queued / source-unchanged / failure-open` | exact lease request `3fe87e76f592414fbd024a61ce5f9e76`、attribution request `429d78d632744dd3a50da271daec4f61` 刷新 `tests.rs + policy.rs + canonical Failure`；v8 ticket `3c720102c9a04ba5a0da053da5064e38` 以同一 `68bf55...` manifest queued，等待协调器串行 compile，未修改两份生产/测试源码。 |
| 2026-08-31 07:39 +08:00 | `v8-materialized / cargo-run-orphaned / no-rust-diagnostic / failure-open` | v8 copy `6bcd24fd6ef04657a93c3ba7048c3984` 第五次完成 exact two-source materialization；Cargo `2ef1fdd329a94a74b444ff698d2f2ca2` 在协调器重启期间终态化为 `orphaned`，exit code 与 stdout/stderr 均为空。保持 split policy module 与 exact two-source hashes 不变；compile acceptance、fixed、integration 和 Runtime02 aggregate rerun 继续等待稳定托管运行。 |
| 2026-08-31 09:27 +08:00 | `v9-queued / exact-source-unchanged / no-poll / failure-open` | v9 ticket `2a3327de64c9456384e2d6067a02fc77` 以同一 `68bf55...` two-source manifest 提交聚焦 autosave policy 测试；保持 `tests.rs=c933d6...`、`policy.rs=2903a5...` 和 split module 架构不变。当前只记录 queued，不把排队误报为编译通过；继续推进非验收任务，等待终态通知后再决定 fixed/return。 |
