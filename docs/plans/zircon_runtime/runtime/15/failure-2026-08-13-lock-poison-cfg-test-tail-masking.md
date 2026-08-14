---
handoff_kind: failure
status: open
created_at: 2026-08-13
summary_slug: lock-poison-cfg-test-tail-masking
origin_plan: docs/plans/engine-code-review-findings-2026-06.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/runtime
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/rust_source_view.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/support.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/global_gate.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input/asset_pipeline.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services/dynamic_scene.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_ui_text_font_id_report.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/native_live_host_lock_poison.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/rhi_wgpu_lock_poison.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/script_vm_lock_poison.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_pipeline_shadow_graph_contracts.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/sources.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/depth_prepass_pure_depth_product_migration.rs
  - zircon_runtime/src/core/resource/event_stream.rs
tests:
  - runtime_15_rust_production_view_rejects_lexical_and_test_only_false_positives
  - runtime_15_structure_guards_share_the_rust_production_view_owner
  - runtime_15_production_view_preserves_items_after_test_only_helpers
  - runtime_15_production_sources_do_not_directly_unwrap_mutex_locks
  - runtime_15_screen_space_ui_text_font_id_report_is_child_owner
  - runtime_15_depth_prepass_pure_depth_product_migration_is_wired
---

# Runtime15 lock-poison cfg-test tail masking

## 来源执行者

- 来源计划：`docs/plans/engine-code-review-findings-2026-06.md`
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 当前阶段：`resolving_failure`

## 失败现象与最低根因

Runtime15 lock-poison 守卫过去以第一次文本 `\n#[cfg(test)]` 为边界截断整个 Rust 文件。只要测试 helper 位于生产 item 之前，后续生产代码就完全不参与扫描。例如 `core/resource/event_stream.rs` 的 test-only `poison_state` 位于 `ResourceEventReceiver` 之前，旧 global gate 无法看见 receiver 及文件剩余生产实现。

同根问题不只存在于 global gate。`lock_poison_policy` 曾有 14 个 `production_section` 调用，`asset_render_input/asset_pipeline.rs` 还维护了第二个手写 `split("\n#[cfg(test)]\nmod tests")`；三个外置 Runtime15 lock-poison 守卫也各自复制首次 `cfg(test)` 截断。只修 global gate 会保留其余 false-green。

## 架构修复合同

- 统一由一个 lexical/cfg production-view owner 处理 Rust 输入，删除手写 `split` 截断。
- 注释、普通/raw/byte/C raw 字符串、字符字面量与 lifetime 不得伪造 attribute 或直接锁调用。
- `cfg(all/any/not)`、stacked attributes 与 `cfg_attr(not(test), cfg(test))` 必须按 `test=false` 的可满足性判定。
- 只遮蔽可确定边界的 Rust item。字段、enum variant、语句或 match arm 等未完整支持的局部 grammar 必须保守保留，不能吞掉 enclosing item 之后的生产源码。
- `production_section` 保留生产字符串内容，供现有 `lock poisoned` 诊断文本守卫使用；`production_code_view` 额外遮蔽 lexical 字符串与注释，供直接调用扫描使用。
- `core/resource/event_stream.rs` 的 test-only poison helper 必须不可见，后续 `ResourceEventReceiver` 必须可见。

## 当前候选与证据

最初 exact4 私有 parser 候选 fingerprint `574d8aa882e5e1ec4e3abe1daf28b698b0ea2bc851c433237d2a80f6d0aa0976` 已废弃，不得提交。当前共享候选把 lexer/cfg production view 提升到 `structure_convention/rust_source_view.rs` 唯一 owner，lock-poison 与 Text font-id report 守卫共同消费，不保留私有复制。exact15 已在受管 Session 中取得 audited scope/lease；fresh immutable review、coordinator 受管 Cargo 与 atomic commit 完成前不得验收。

Rust 1.94.1 直接编译并执行候选 `support.rs` 的独立 harness 通过：`candidate_support=direct synthetic_lines=13 event_lines=611 violations=0`。同一 exact4 测试模块树由 Rust 1.94.1 `rustc --test` 编译成功。核心测试直接执行为 2/2 GREEN：

- `runtime_15_production_view_preserves_items_after_test_only_helpers`
- `runtime_15_production_sources_do_not_directly_unwrap_mutex_locks`

共享 owner 的独立回归和 Text guard harness 也由 Rust 1.94.1 编译并分别 1/1 GREEN：

- `runtime_15_rust_production_view_rejects_lexical_and_test_only_false_positives`
- `runtime_15_screen_space_ui_text_font_id_report_is_child_owner`

候选扩展还把 native live host、RHI WGPU 与 script VM 三个外置 lock-poison 守卫迁移到共享 production view；direct harness 暴露的 native registry 与 deterministic RHI owner 旧字符串锚点同步更新为当前生产符号，不恢复旧容器或旧测试名。

最终二审继续发现 shadow mesh pipeline 与 depth-prepass 两个 production 结构守卫的同根截断；候选把它们迁移到 shared production view，并新增递归结构守卫，要求整个 Runtime15 `structure_convention` 测试树不再复制首次 `cfg(test)` split。迁移后的 direct harness 进一步暴露两个旧锚点：shadow GPU execution 已拆到 `gpu/mesh_recording.rs`，source inventory 现显式读取该 child owner；DepthPrepass 的正向字符串断言使用保留字符串的 `production_section`，禁止项继续使用 `production_code_view`，且不再要求已退出该局部合同的全局 review-findings 状态镜像。

exact4 harness 的其余细分门有 1 个 GREEN、4 个在进入 production-view 断言前因 foreign dirty 生产锚点漂移失败。漂移属于当前 Asset、dynamic session 与 dynamic-scene owner 的共享迁移，不在本 exact4 中修补，也不作为本 failure 的 source RED/GREEN：

- `asset/pipeline/worker_pool.rs` 的 helper shape 已由当前 foreign 改动演进。
- `project_asset_manager/runtime.rs` 的 change subscriber 类型已演进为 typed subscriber。
- `dynamic_api/session/ffi.rs` 的 registry import 已由当前 foreign 改动重排并扩展。
- `scene/dynamic_scene/spawn_task/task.rs` 的 multi-line import 已由当前 foreign rustfmt 重排。

## 禁止临时方案

不得恢复按第一个 `cfg(test)` 截断，不得只修 global gate 而保留细分门旧 helper，不得用 ignore/allow-failure 掩盖 parser 漏扫，也不得抢改 foreign 生产 owner 以迁就字符串锚点。

## 修复结果与回传

Open state: `共享 exact15 source candidate、scope lease 与 Rust1.94.1 direct harness 已完成；fresh immutable exact review、managed Cargo、atomic commit/fixed return 尚未完成。因此当前不得写为 fixed。`

## 2026-08-14 forward repair and static evidence

The shared production-view candidate still rebuilt `production_section` from raw source after it had found cfg-test spans. That preserved required production diagnostic strings, but also preserved comments, so a comment-only `lock poisoned` phrase could create a false-positive lock-poison gate failure. `production_section` now masks cfg-test spans from the comment-free lexical view instead: comments remain invisible while ordinary, raw, byte, and C-raw production strings remain available to diagnostic guards.

`lock_poison_policy/core_runtime/global_gate.rs` now carries the regression assertion for that distinction. A temporary parent-module harness compiled with Rust 1.94.1 and executed outside the Cargo lane on the D: target: 3 passed, 0 failed (`runtime_15_rust_production_view_rejects_lexical_and_test_only_false_positives`, `runtime_15_structure_guards_share_the_rust_production_view_owner`, and the new comment/string regression). `rustfmt --check` passed for the two changed guard files; scoped `git diff --check` has no whitespace defect beyond the repository's LF/CRLF notice.

This is forward repair evidence only. Managed Cargo, fresh immutable review, atomic commit, and the canonical fixed return remain pending; this failure remains open.

## 2026-08-14 integration-ready static review

The exact Runtime15 candidate now has 14 leased paths: the shared
`rust_source_view` owner, its parent mount, nine lock/production consumers,
two shader/depth consumers, and this handoff. A Rust 1.94.1 standalone harness
compiled only that shared owner on `D:\\ZirconBuilds` and executed both of its
regressions: `2 passed; 0 failed`. It covers lexical comments and strings,
`cfg`/`cfg_attr` test-only item masking, conservative retention of unsupported
local grammar, the `event_stream` test-helper ordering regression, and a full
structure-guard scan for the retired manual `cfg(test)` split pattern.

The exact-source `rustfmt --check --config skip_children=true` and
`git diff --check` pass. The current tree reports exactly two shared-view
definitions (both in `rust_source_view.rs`) and zero legacy manual
`cfg(test)` splits under `structure_convention`. A non-recursive root format
check still discovers import-order drift in four unowned
`runtime_dead_code` files; they are outside this manifest and are not absorbed
by this repair.

This establishes a HEAD-integratable support snapshot only. The source-bound
managed `zircon_runtime --lib` validation, fresh immutable review, and failure
return remain required, so the handoff stays `open`.

## 2026-08-14 native-live-host input hard cut

`native_live_host_lock_poison.rs` no longer reads six declaration-only Runtime15
archive/module documents and its local repository reader was removed with them.
It continues to read the live bridge-method source and structure-convention
mount, then checks poison recovery and direct-lock safety through the shared
production view. This removes unrelated document resources from the Runtime
lib-test compile input without weakening the lock-poison contract. Rustfmt,
scoped diff, and exact source checks passed for this source edit; no Cargo
result or fixed return is claimed.

The lock-poison global gate likewise no longer reads five declaration-only
Runtime15 parent/index/review/structure/module documents. It continues to walk
all production Rust source and retains the lexical, cfg, event-stream ordering,
and diagnostic-string regressions for the shared production view.

## 2026-08-15 asset guard input narrowing

The two Asset lock-poison guards had each read four historical Runtime15 output
records plus two module documents without using any of those values in an
assertion. Those declaration-only reads are removed. The guards continue to
read and assert against the ProjectAssetManager, AssetWorkerPool, and service
contract production sources; poison-recovery and direct-lock assertions remain
unchanged. This is static source-scope repair only: managed Cargo, immutable
review, and the canonical failure return remain pending.
