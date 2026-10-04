---
handoff_kind: failure
status: open
created_at: 2026-07-30
summary_slug: runtime-frame-capture-sibling-module-projection
origin_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
fixing_plan: docs/plans/mvp/03-f2-scene-runtime.md
origin_child_dir: docs/plans/zircon_runtime/render/17
fixing_child_dir: docs/plans/mvp/03
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/runtime_entry_app/surface_present/redraw.rs
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
tests:
  - managed cargo +1.94.1 test -p zircon_app --lib --locked -- entry::runtime_entry_app::surface_present::redraw::tests::frame_capture_projects_to_the_runtime_entry_root_sibling --exact --nocapture --test-threads=1
  - managed cargo +1.94.1 test -p zircon_app --lib --locked -- entry::runtime_entry_app::frame_capture::tests:: --nocapture --test-threads=1
  - managed cargo test -p zircon_runtime --lib graphics::tests::render_product_post_process_full_chain::render_product_post_full_chain_all_effects_on --locked -- --exact --test-threads=1
---

# MVP 03：runtime frame capture sibling module projection

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 来源执行切片：Render17 current-source WGPU full-chain chromatic diagnostic gate
- 修复责任计划：`docs/plans/mvp/03-f2-scene-runtime.md`
- 交接原因：失败位于 MVP F2 当前拥有的 `runtime_entry_app` 产品帧捕获模块边界；Render17 只消费该应用层 crate，不能在渲染计划内改写其模块投影。

## 失败现象与复现证据

2026-07-30 12:25 CST，Render17 受管 job
`d425b47aef704483adab085546417704` 在运行上列 exact gate 前编译当前共享源码，`zircon_app` 自然终止：

```text
error[E0433]: could not find `frame_capture` in `super`
  --> zircon_app/src/entry/runtime_entry_app/surface_present/redraw.rs:148:16
148 -         super::frame_capture::write_runtime_frame_png(
```

`frame_capture` 定义于 `runtime_entry_app` 根模块；`redraw.rs` 位于其 `surface_present` 子模块，因而该路径解析为 `surface_present::frame_capture`，而不是根模块的 sibling。Render17 测试体尚未执行，故本轮不是色彩链路的通过或失败证据。

## 最低共享层根因

MVP F2 新增的产品截图写出调用未从 `surface_present` 正确投影到 `runtime_entry_app::frame_capture`。这是应用层模块所有权/可见性错误，不是 WGPU、Render17 后处理或测试配置错误。

## 架构修复验收

- `surface_present::redraw` 通过唯一的 `runtime_entry_app` 根模块路径调用帧捕获 writer，不创建 alias、重复 writer 或 test-only fallback。
- MVP F2 的聚焦运行时帧捕获测试通过。
- 重跑本 handoff 的原始 Render17 受管 exact gate；它至少必须越过 `zircon_app` 的 E0433 编译边界，之后 Render17 再独立判定色彩结果。

## 禁止临时方案

- 不得在 `surface_present` 复制 `frame_capture`、添加兼容 alias，或关闭首帧 PNG 写出以掩盖编译错误。
- 不得将 Render17 的截图/色彩验收改为跳过 `zircon_app` 编译。

## 修复结果与回传

### 2026-08-27 受管验证前置阻塞

- 以 Coordinator01 failure-cleanup session 发起
  `zircon_app` focused lib-test
  `frame_capture_projects_to_the_runtime_entry_root_sibling`；该 gate 会真实编译
  `zircon_app`，用于直接复核旧 E0433 边界。
- 协调器在 validation ticket/Cargo job 创建前以
  `unmanaged_artifacts_detected` fail-closed。唯一当前路径为
  `D:\ZirconBuilds\tooling15-wave137-runtime-20260827-054615`，其 artifact cleanup
  reservation 于 `2026-08-26T21:55:50.161328+00:00` 仍存在；对应 Tooling15 bootstrap
  进程仍存活，MVP03 未删除、终止或接管该 owner 的产物。
- 本次没有启动 Cargo/rustc，也不构成新的动态 GREEN。HEAD 中仍保留唯一
  `super::super::frame_capture::write_runtime_frame_png(...)` 调用和两条 committed
  source guard；failure 继续保持 `open`，待 artifact governance 恢复后由 FIFO
  重提一次 focused gate。

### 2026-08-24 受管验证前置阻塞

- validation ticket：`d1ec49cbdb304826a21bb59a7faccbba`
- validation copy：`a5356e6b9f4742a693f895ec67f2ac37`
- 终态：`failed`，阶段 `closure_planning`，Cargo 未启动。
- 精确外部依赖链：`zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs` 引用了当前副本缺失的 `zircon_editor/assets/ui/editor/animation_editor.zui`；durable error code 为 `validation_copy_compile_time_resource_missing`。
- 本 failure 的两条实现路径仍与 snapshot `2089` 一致；不得把 Editor 资产缺失误记为 frame-capture GREEN。等待该外部 compile-time resource 恢复后，仅按 FIFO 重提同一 focused test。

Open state: `MVP03 source repair complete; managed upward validation pending`; no pass is claimed.

- `surface_present::redraw` now calls the one root-owned writer through
  `super::super::frame_capture::write_runtime_frame_png(...)`; it does not create a
  `surface_present` alias, duplicate writer, or test-only capture fallback.
- Current-source static review confirms the root `frame_capture` module exports the writer and
  the redraw call resolves through the sibling path. The local source guard covers that call
  shape; this is not Cargo evidence.
- The declared managed Render17 exact gate must still rebuild `zircon_app` from a fresh source
  snapshot and run past the previous E0433 boundary. Only its terminal result can determine the
  downstream render outcome or move this canonical handoff to `fixed-*`.

### 2026-08-28 current-source managed replay

- Managed job `09b9495deb684503850deaa5b0bdf774` ran
  `cargo test -p zircon_app --locked --lib frame_capture_projects_to_the_runtime_entry_root_sibling`
  in the retained D-drive pool and released normally with no live process PIDs.
- Cargo rebuilt the current dependency graph and did not reproduce the original
  `surface_present::frame_capture` E0433. Compilation stopped before the focused test executed on
  the separately owned `zircon_runtime_host/src/foreign_output/item_count.rs`: its match over
  `WorldQueryResult` does not yet cover `TransformSnapshot` (E0004).
- This replay proves the prior artifact-governance and validation-copy closure blockers are no
  longer the admission boundary, but it is not a focused GREEN. The canonical frame-capture
  failure remains `open` until the external world-query consumer repair is integrated and the
  same managed test executes successfully; MVP03 does not absorb or patch that owner here.

### 2026-09-11 rolling repair continuation

- Stable fixing Session `failure-roll-01a090ae-mvp03-frame-capture-sibling-r1` registered the
  current three-path scope at baseline epoch `608`: the `surface_present/redraw.rs` caller,
  the root-owned `frame_capture.rs` writer, and this failure record. Current attribution was
  refreshed before and after the validation attempt; no source bytes were changed in this slice.
- The first coordinator static ticket `5b27ca17876d4365a0029e586c005c1f` ended `failed` because
  its checker used a substring negative assertion that also matched the legal
  `super::super::frame_capture` path. Its diagnostic is a validator assertion error, not a Rust
  or source failure, and is retained rather than reused as acceptance evidence.
- Corrected static ticket `358f823ca2c347919ba941221567b502` passed with the same sealed current
  manifest (`7e6e6036568af37f72bf9876c444e7bcd6c2eb520a6fdf589cde181549f68ba2`). The corrected
  guard matches the complete call line, confirms `pub(super) fn write_runtime_frame_png`, and
  rejects aliases/duplicates. This is static evidence only.
- The new managed Cargo request
  `failure-roll-01a090ae-mvp03-cargo-20260911-r1` used the exact focused
  `zircon_app --lib frame_capture_projects_to_the_runtime_entry_root_sibling` command with
  `--locked` and no caller-owned jobs flag. Coordinator admission rejected it before ticket
  creation or Cargo execution with `validation_ticket_external_worktree_dirty` for
  `E:\Git\zr_vm`. Therefore there is no new Cargo exit code or dynamic GREEN. The prior
  `TransformSnapshot` compile failure remains an independently owned upstream blocker; MVP03
  does not patch it.
- Session status is `waiting_validation`, leases are released, and this canonical failure stays
  `open`. No fixed return, closeout commit, review completion, or notification is claimed.

### 2026-09-19 rolling successor source-contract admission

- Successor Session `failure-roll-01a084c8-mvp03-frame-capture-sibling-r2` reclaimed
  the exact caller, root writer, and canonical record under transfer fingerprint
  `2b6793fbfe75d1583d368fa62ed8ecfeee96911617080ee7328235d4ca39df2c` at baseline
  epoch `611`; current source hashes were preserved without edits.
- Current-source static request
  `failure-roll-01a084c8-mvp03-frame-capture-sibling-20260919-r1` admitted ticket
  `5eaa2054cfb24d26adc4d2ddcc9c414b` with sealed manifest hash
  `499a733a2bacca8be78e762f35ef4e7b98d5a7cf4ed9c4f1cc7320fa8617aade`; status is
  `queued` pending the coordinator terminal result. The checker asserts the exact
  `super::super::frame_capture` call and root writer export while rejecting aliases.
- This ticket is static-only. Fresh managed `zircon_app` Cargo, the Render17
  originating gate, independent C/I/M review, fixed return, closeout, and the
  external `E:\Git\zr_vm` clean-worktree prerequisite remain pending.

### 2026-09-19 static checker correction

- The successor static ticket `5eaa2054cfb24d26adc4d2ddcc9c414b` reached the
  coordinator terminal state `failed` at `2026-09-19T06:46:20.191550Z` after
  running job `12394280b048455ba4e25ac4b89a783d`.
- The source assertions for the exact sibling call and root writer export passed,
  but the checker then used a substring negative assertion for
  `super::frame_capture::write_runtime_frame_png(`. That substring also occurs
  inside the legal `super::super::frame_capture::write_runtime_frame_png(`
  path, so the failure is a coordinator checker defect (`AssertionError`), not
  a Rust/source failure or dynamic acceptance result. The ticket is excluded
  from failure-cache reuse (`failureCacheExcluded=true`).
- No source bytes changed. A corrected line-based checker must be submitted
  against a fresh manifest; the managed `zircon_app` Cargo gate, Render17
  originating gate, independent C/I/M review, fixed return, closeout, and the
  external `E:\Git\zr_vm` clean-worktree prerequisite remain pending.

### 2026-09-19 corrected static terminal result

- Corrected ticket `a003782cf27b4f36b622c2ac95a00b93` passed at
  `2026-09-19T06:53:27.651636Z` in job
  `8ded1dc1c4c64fa9934587571e7bf0d6`, with exit code `0` and stdout marker
  `MVP03_FRAME_CAPTURE_SIBLING_CURRENT_SOURCE_CONTRACT_PASS`. Cleanup event
  `10887` completed successfully.
- The line-based checker now distinguishes the legal
  `super::super::frame_capture::write_runtime_frame_png` route from shorter
  aliases; this is static source-contract evidence only. It supersedes the
  failed substring-checker ticket `5eaa2054cfb24d26adc4d2ddcc9c414b` and does
  not add Cargo or Render17 acceptance.
- Fresh managed `zircon_app` Cargo, the Render17 originating gate, independent
  C/I/M review, fixed return, closeout, and the external `E:\\Git\\zr_vm`
  clean-worktree prerequisite remain pending; the canonical failure stays
  `open`.

### 2026-09-20 independent source review r2

The independent reviewer inspected the current `runtime_entry_app` module
projection and its writer. `surface_present::redraw` has exactly one
`super::super::frame_capture::write_runtime_frame_png` call, resolving to the
root-owned sibling without an alias or duplicate writer. The call remains
after a successful native or reference presentation and before the presented
frame counter advances. The writer preserves the explicit RGBA length check,
staging-file reservation, buffered flush, filesystem sync, atomic commit, and
cleanup-on-error paths; no test-only fallback was added.

Read-only checks:

- `rustfmt +1.94.1 --edition 2021 --config skip_children=true --check` over
  `frame_capture.rs` and `surface_present/redraw.rs`:
  `MVP03_RUSTFMT_PASS paths=2`.
- `git diff --check` over those paths: pass.

Current source hashes inspected:

```text
zircon_app/src/entry/runtime_entry_app/frame_capture.rs afeebd2a08d95f29e54d9351fffeccca9c76ea3e86848cae177c032a6bbf7ac7
zircon_app/src/entry/runtime_entry_app/surface_present/redraw.rs 9cd337a68a21c8e8f34c4fdbbd6f6fdd0c10a72823f32f287c04f578f9558583
```

Independent review result: `Critical=0 Important=0 Moderate=0`. No managed
`zircon_app` Cargo, Render17 WGPU, screenshot/PNG, canonical fixed return, or
closeout result is inferred from this source review.

### 2026-09-25 current-source rolling reconciliation

- Current source hashes still match the sealed sibling projection:
  `zircon_app/src/entry/runtime_entry_app/frame_capture.rs`
  `afeebd2a08d95f29e54d9351fffeccca9c76ea3e86848cae177c032a6bbf7ac7` and
  `zircon_app/src/entry/runtime_entry_app/surface_present/redraw.rs`
  `9cd337a68a21c8e8f34c4fdbbd6f6fdd0c10a72823f32f287c04f578f9558583`.
  The working tree is clean for both paths; this successor made no source edit.
- The exact focused Cargo command remains the frontmatter gate. The prior
  managed replay reached Cargo but stopped before test execution at the foreign
  `WorldQueryResult::TransformSnapshot` exhaustiveness error; the external
  `E:\\Git\\zr_vm` dirty-worktree admission blocker also remains recorded. No
  historical static ticket or Render17 result is reused as a dynamic pass.
- Independent review C/I/M `0/0/0`, the Render17 upward gate, fresh managed
  `zircon_app` execution, fixed return, and closeout remain pending; this
  canonical failure stays `open`.

### 2026-09-25 independent static review receipt

- Reviewer `/root/review_editor03_gizmo_private` re-read snapshot 3798 at
  document SHA-256 `f4d65ac21549e40200f9e143af9d6cf0f8e4dabbf08f4e1704d1b940d74ce379`.
  Both source hashes match and the working tree is clean for the two owned
  paths. The production tree has exactly one sibling writer call
  `super::super::frame_capture::write_runtime_frame_png`; the root writer is
  `pub(super)`, with no alias, duplicate, or fallback route.
- Review result: Critical/Important/Moderate = `0/0/0`. The foreign
  `WorldQueryResult::TransformSnapshot` compile blocker and external
  `E:\\Git\\zr_vm` admission blocker remain accurate. Static checker, rustfmt,
  and historical Cargo replay are not current managed Render17 acceptance; no
  Cargo command was run.


### 2026-09-27 current-source acceptance-command reconciliation

- The frontmatter now lists the direct `zircon_app` sibling-projection regression
  and the root PNG writer suite before the preserved original Render17 exact gate.
  The mounted projection test is
  `entry::runtime_entry_app::surface_present::redraw::tests::frame_capture_projects_to_the_runtime_entry_root_sibling`.
  The writer filter is
  `entry::runtime_entry_app::frame_capture::tests::`; current source declares seven
  tests under that module, including RGBA PNG roundtrip, invalid payload,
  staging cleanup, atomic replacement, and flush/sync error propagation. This
  source inventory is not a test-run count or a dynamic pass. Each managed result
  must show that its intended test or all seven writer tests actually executed.
- The module chain is `lib.rs -> entry/mod.rs -> runtime_entry_app/mod.rs ->
  surface_present/mod.rs -> redraw.rs` for the caller, and the root
  `runtime_entry_app/mod.rs -> frame_capture.rs` for the writer. Default App
  `target-client` features include `default-platform -> platform-winit`, which
  admits the runtime-entry module. The corrected caller and writer still match
  the source hashes recorded above; this reconciliation does not change them.
- The historical Render17 command targets only `zircon_runtime --lib`; the
  current Runtime manifest has no dependency on `zircon_app`. Its execution is
  required for the original downstream reproduction, while the separate App
  gate must compile the actual caller and writer to prove that the former E0433
  boundary is resolved. The prior job's broader compilation is preserved as
  historical evidence and is not attributed to that single Runtime command.
- Current HEAD `bc02eefafead65dbf5050482110e8175250a5e77` already contains
  `WorldQueryResult::TransformSnapshot { .. } => 1` at
  `zircon_runtime_host/src/foreign_output/item_count.rs:90`. Its current bytes are
  SHA-256 `7e643d75bc34a77a5a596a59351184ad679a10197b613a7c7255b7d00f66e322`,
  and the file is clean against HEAD. The August replay's E0004 remains a
  historical RED; it is not an unresolved current-source omission. This static
  observation does not prove a fresh Cargo compile or consumer regression pass.
- Fresh direct App regression, the seven PNG writer tests, and the originating
  Render17 exact gate remain dynamically pending. The F2 plan's real product PNG,
  input, and two-lifecycle gates also retain their own pending status; encoder
  tests cannot substitute for those product results. No old ticket's terminal
  state is changed or inferred in this reconciliation, and no request was
  resubmitted. The external whole-worktree archive and Cargo network admission
  prerequisites require their own verified resolution before new managed Cargo.
- Managed Windows validation keeps `--locked` and freezes the actual relevant
  source, tests, configuration, and external-worktree inventory. Compiler
  products and caches must physically be under drive-root
  `D:\cargo-targets`, `E:\cargo-targets`, or `F:\cargo-targets`; the coordinator
  supplies the approved location. Repository-local targets, nested lookalike
  roots, aliases, `C:`, and legacy `ZirconBuilds` paths are not acceptance storage.
- This record remains `open`; no dynamic GREEN, canonical fixed return,
  closeout, commit, notification, or completion follows from this reconciliation.
