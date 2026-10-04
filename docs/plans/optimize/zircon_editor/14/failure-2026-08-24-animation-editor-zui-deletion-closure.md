---
handoff_kind: failure
status: open
created_at: 2026-08-24
summary_slug: animation-editor-zui-deletion-closure
origin_plan: docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
fixing_plan: docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
origin_child_dir: docs/plans/optimize/zircon_editor/09
fixing_child_dir: docs/plans/optimize/zircon_editor/14
plan_link_mode: child_record_only
failure_scope: cross_plan
related_code:
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.zui
  - zircon_editor/assets/ui/editor/host/animation_graph_body.zui
  - zircon_editor/src/tests/ui/animation_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/boundary/global_material_surface_assets/mod.rs
  - zircon_editor/src/tests/ui/boundary/global_material_surface_assets/contracts.rs
  - zircon_editor/src/tests/ui/boundary/global_material_surface_assets/support.rs
  - zircon_editor/src/tests/ui/boundary/template_assets/mod.rs
  - zircon_editor/src/tests/ui/boundary/template_assets/host_shells.rs
  - zircon_editor/src/ui/layouts/views/animation_editor.rs
  - zircon_editor/src/ui/layouts/views/mod.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/tests/integration_contracts/workbench_animation_editor_shell.rs
tests:
  - cargo +1.94.1 test -p zircon_editor --locked --lib --release core::jobs -- --include-ignored --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --locked --lib --release editor09_ -- --include-ignored --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --locked --lib --release editor10_ -- --include-ignored --nocapture --test-threads=1
---

# Editor 14: animation_editor.zui 删除迁移未闭合

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md`
- 来源执行切片：Editor 09 后台作业热路径批量验证的 validation-copy closure planning gate；同一缺口也会阻断 Editor 10 及其他 `zircon_editor --lib` 批次。
- 修复责任计划：`docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md`
- 交接原因：缺失资源及其全部引用都属于动画编辑器布局、模板与产品入口迁移边界，最低共享责任不在 Editor 09 作业系统或 Editor 10 通知系统。

## 失败现象与复现证据

协调器票据 `d79991c986e84551837f2ab8642f3d06` 与 `0b8cc9f7a8d949ada04e863c84b26d6e` 均在 Cargo 启动前失败，阶段为 `closure_planning`，错误码为 `validation_copy_compile_time_resource_missing`。缺失路径是 `zircon_editor/assets/ui/editor/animation_editor.zui`，发现来源是 `zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs` 的编译期 `include_str!`。

原始失败快照把该受版本控制资源标记为删除，并列出五处编译期或产品引用。当前滚动快照已由其他已归档迁移工作收束其中的 bootstrap 与 global-material inventory；剩余可复现的悬空产品/契约引用是布局投影、Workbench integration contract，以及 template inventory 中仍要求旧路径的断言：

- `zircon_editor/src/ui/layouts/views/animation_editor.rs`
- `zircon_editor/tests/integration_contracts/workbench_animation_editor_shell.rs`
- `zircon_editor/src/tests/ui/boundary/template_assets/host_shells.rs`

因此原始 Editor 09 和相关 Editor 10 命令仍不能回溯表述为测试失败或测试通过；本次修复必须重新提交 focused asset/template validation 后才能升级状态。

## 最低共享层根因

Editor 14 正在进行动画工作台 ZUI 资产迁移，但删除旧 `animation_editor.zui` 的 hard cutover 尚未同时收束产品布局常量与 integration/template contracts。当前 canonical owner 是 sequence 与 graph 两个 host body 资产；validation-copy 正确地拒绝构造包含悬空编译期资源的验证快照。

最低已证实边界是“动画编辑器布局资源的 canonical owner 与全部消费者未原子迁移”；未对更深层的目标布局方案作推断。

## 架构修复验收

- 明确并实现唯一 canonical 动画编辑器布局资产 owner；若旧资产确实应删除，全部消费者必须在同一 hard cutover 中迁移到新 owner。
- 仓库中不再存在指向缺失 `animation_editor.zui` 的 `include_str!`、产品路径常量、模板 inventory 或 integration contract。
- 通过协调器重新提交 Editor 14 的 focused asset/template validation，并确认 validation-copy closure planning 与 Cargo 测试均通过。
- 重新运行上列 Editor 09 与 Editor 10 原始批量命令，确认上层性能计划 gate 可以恢复。

## 禁止临时方案

- 不得仅为绕过 validation-copy 而恢复已废弃的旧资产、复制一份同名资产或引入 alias、compatibility shim、silent fallback、test-only bypass、call-site exception。
- 不得削弱 compile-time resource 检查、模板 inventory、integration contract 或计划验收标准来隐藏悬空引用。
- 不得把 Editor 14 的动画资产迁移实现混入 Editor 09/10 性能优化提交。

## 修复结果与回传

Open state: `待受管 Cargo 验证`; no fixed return or closeout is claimed.

当前修复快照（2026-09-12，baseline epoch 608）：

- `tools/tests/test_editor14_animation_zui_hard_cut.py` 受管静态票据 `c4ea72ba6a4542b395c89186a298d3f2` 已真实执行并通过 `2 passed`；它覆盖旧资源无 live source 引用、sequence/graph 双 owner 路由和 host mount contract。
- `zircon_editor/src/ui/layouts/views/animation_editor.rs` 已按 pane kind 分别投影 `host/animation_sequence_body.zui` 与 `host/animation_graph_body.zui`；Workbench integration/template inventory 已同步。
- Editor 14 的锁定 Cargo check 票据在 admission 阶段被协调器以 `validation_ticket_external_worktree_dirty` 拒绝，阻塞源为外部 `E:\Git\zr_vm` 的未提交工作树；因此尚未形成 Cargo 编译或原始 Editor 09/10 上行测试证据。

待外部依赖清洁后，必须用同一当前源码快照重新提交 focused Cargo check、integration contract 和原始 Editor 09/10 命令；在此之前保持 failure open。

## 2026-09-20 successor reconciliation and review

- Successor `failure-roll-01a084c8-editor14-animation-zui-closure-r2` was admitted at
  baseline epoch 611. The archived r1 ownership was transferred through coordinator
  preview/apply fingerprints `9d52d535dd18c722bfd225bf5f1f56f5927da08f458ca72bf364931308ed7b1d`
  (eight source/record paths) and `e1ea1fbc1cebf7b56d6bb4f918a54dfe63ab94d30044e89f172fba8fef38d0b9`
  (the two canonical host assets). Current leases cover the exact twelve-path scope;
  no source bytes were reverted or reconstructed.
- Current source static regression `python tools/tests/test_editor14_animation_zui_hard_cut.py`
  executed the two named tests and reported `2 passed`. The focused owned Rust files
  (`animation_editor.rs`, bootstrap assets, template inventory, and integration contract)
  pass Rust 1.94.1 rustfmt check; scoped `git diff --check` passes. The two layout projection
  files contain the canonical sequence/graph dispatch and no live `animation_editor.zui`
  reference.
- Independent review of the owned migration boundary is C0/I0/M0 for the canonical asset
  routing, pane-kind dispatch, and contract assertions. An out-of-scope handoff is retained:
  `zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection/document_leaves.rs`
  and related unregistered worktree changes are foreign to this lifecycle; they are not edited,
  attributed, or treated as Editor 14 proof. Any Cargo snapshot that requires those bytes must
  first receive an owner transfer or a clean owner-provided revision.
- A fresh managed focused command was submitted as cargo job
  `c858da1430ae4b0c80ac183fd94d6e90` under coordinator validation session
  `validate-matrix:01a084c8-607d-70d2-bed6-d6254c1241b7:successor:0e1083c68d134947a62fdbd700cc98e9`:
  `validate-matrix.ps1 -Package zircon_editor -LibTests -TestFilter integration_contracts::workbench_animation_editor_shell::animation_editor_shell_uses_canonical_sequence_and_graph_templates -VerboseOutput`.
  The coordinator supervisor exited `1` at `2026-09-20T20:53:42Z` and released the retained
  target at `20:53:51Z`; no `validation_tickets`, `validation_copies`, `cargo_job_runs`, or
  stdout/stderr diagnostic artifact was recorded. This is supervisor-level admission/runner
  evidence only, not a test result, so no Cargo pass or failure reproduction is claimed. The
  original Editor 09/10 release gates and the upward product gate remain pending. Keep this
  failure open until a clean owner-provided snapshot reaches the managed result, independent
  review binding, return, fixed artifact, and closeout stages.

## 2026-09-29 view-surface inventory correction

- Current `.zui` parsing finds 41 view surfaces under the test's editor asset roots. The sequence
  and graph host bodies are present; the retired `animation_editor.zui` is absent. The separate
  `global_material_surface_assets/contracts.rs` count assertion still required 42, so its
  current-source test would fail even after the missing-resource closure is repaired.
- Successor Session `failure-roll-01a0df1a-editor14-zui-closure-r3` took the exact failure record
  and global-material contract module from their archived owners. The contract now pins all 41
  view paths and retains every per-asset conformance check, so an equal-count substitution also
  fails. This corrects the linked inventory and assertion only. The focused Editor14 Cargo,
  original Editor09/10 batches,
  independent review, return, and closeout remain open.
