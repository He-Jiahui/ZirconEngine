---
handoff_kind: failure
status: open
created_at: 2026-08-13
summary_slug: lock-poison-cfg-test-tail-masking
origin_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/runtime/15
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
failure_scope: local
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
  - zircon_runtime/crates/zr_resource/src/event_stream.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_system_texture_generation_owner.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_ui_resource_upload_transaction.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_dispatch_bound_text_model_updates.rs
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/text/08-ime-and-text-input.md
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/vertex_buffer.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/commit_compiled_scene_frame_success.rs
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

- 来源计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 来源执行切片：Runtime15 lock-poison and production-structure guard convergence
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：The false-green parser and every affected structure guard are owned by Runtime15; the global review document identified the defect but is not a numbered execution owner.
- 发现来源：`docs/plans/engine-code-review-findings-2026-06.md`
- 当前阶段：`resolving_failure`

## 失败现象与复现证据

Runtime15 lock-poison 守卫过去以第一次文本 `\n#[cfg(test)]` 为边界截断整个 Rust 文件。只要测试 helper 位于生产 item 之前，后续生产代码就完全不参与扫描。例如 `core/resource/event_stream.rs` 的 test-only `poison_state` 位于 `ResourceEventReceiver` 之前，旧 global gate 无法看见 receiver 及文件剩余生产实现。

## 最低共享层根因

同根问题不只存在于 global gate。`lock_poison_policy` 曾有 14 个 `production_section` 调用，`asset_render_input/asset_pipeline.rs` 还维护了第二个手写 `split("\n#[cfg(test)]\nmod tests")`；三个外置 Runtime15 lock-poison 守卫也各自复制首次 `cfg(test)` 截断。只修 global gate 会保留其余 false-green。

## 架构修复验收

- 统一由一个 lexical/cfg production-view owner 处理 Rust 输入，删除手写 `split` 截断。
- 注释、普通/raw/byte/C raw 字符串、字符字面量与 lifetime 不得伪造 attribute 或直接锁调用。
- `cfg(all/any/not)`、stacked attributes 与 `cfg_attr(not(test), cfg(test))` 必须按 `test=false` 的可满足性判定。
- 只遮蔽可确定边界的 Rust item。字段、enum variant、语句或 match arm 等未完整支持的局部 grammar 必须保守保留，不能吞掉 enclosing item 之后的生产源码。
- `production_section` 保留生产字符串内容，供现有 `lock poisoned` 诊断文本守卫使用；`production_code_view` 额外遮蔽 lexical 字符串与注释，供直接调用扫描使用。
- `zircon_runtime/crates/zr_resource/src/event_stream.rs` 的 test-only poison helper 必须不可见，后续 `ResourceEventReceiver` 必须可见；退役的 `core/resource/event_stream.rs` 必须保持不存在。

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

The RHI WGPU and Script VM guards had the same declaration-only document input
pattern. Their unused document reads are removed; Script VM also no longer
imports the repository reader that served only those reads. Both guards retain
their production-owner, mount, and poison-safe lock assertions. This remains
an open forward repair pending managed Cargo and immutable review.

The screen-space UI text font-ID report owner guard also read seven archived
Runtime15 or module documents without using them in any assertion. Those
declaration-only reads and the now-unused repository-reader import are removed.
The parent/child ownership, narrow font-face query, and production-file budget
assertions remain unchanged. This is static source-scope repair only; the
canonical failure remains open pending managed Cargo and immutable review.

## 2026-08-15 shared-view integration follow-up

The shared lexical/cfg production-view owner is integrated at
`b8fb3bd0a`; all lock-poison consumers continue to route through that single
owner. The last structure-guard-local first-`cfg(test)` split was an unowned
Render01 materialization guard. Its single-file successor replaced the split
with `production_section` and integrated at `f49583ffeb6fa130d59504bbf7b8c87bc34dd7a1`.

A fresh static scan of the entire `structure_convention` guard tree now reports
zero legacy `.split`/`.split_once` calls that take `cfg(test)`. This only
removes the remaining false-green masking route. No source-bound managed Cargo
result or independent immutable review exists yet, so this handoff remains
`open` and no fixed return is authorized.

## 2026-09-09 canonical resource-owner regression repair

The shared lexical/cfg owner is present at its integrated hash
`09b4ea15f8d38e4b9962e5da6f2c98d8da314b527c52340ccca2f7648a4d7553`.
The real event-stream owner has since moved to
`zircon_runtime/crates/zr_resource/src/event_stream.rs`, hash
`8b72d3e7a501d01f58c0df7db715de0ada40663a32f45eb6cae9330bec723a7a`.
Its test-only `poison_state` still precedes `ResourceEventReceiver`, so it
remains the production regression input required by this lifecycle. Both
hashes match immutable input `runtime15-registration-input-3257-20260908`,
manifest `86719cb85fc2f0b13f168beea45a0271dade59f04cfbb1aea23ac279d2c913c2`.

Managed Windows static/no-default/locked job
`0da2ccdc294b4f8e82005ff7ca84ad2e` executed
`tests::runtime_absorption::structure_convention::lock_poison_policy::core_runtime::global_gate::runtime_15_production_view_preserves_items_after_test_only_helpers`:
0 passed, 1 failed, 0 ignored, 6849 filtered out. It reached the old path read
and failed with Windows OS error 2, before the real event-stream and final
diagnostic-string assertions. Receipt and complete RED output are retained in
`results/runtime15-event-owner-red-3257.{json,log}` under that input.

Fixing Session `failure-roll-01a07160-runtime15` acquired only this record and
`global_gate.rs` from terminal owners through fingerprint
`50650d676a331867036ba3621e14a9f1c26a7880907b17fa03cfa3134318a17c`.
Both preimages matched HEAD; snapshot 3261 preserves the source hash
`322679e47b2780456514023b850d7397f6b6928a9fb40825d4c7b2b71dc452d1`
and complete record history. The guard now reads the existing canonical
crate owner with the shared repository reader and asserts the retired path
is absent. Every original lexical/cfg, real event-stream, receiver-visibility,
lock-call and diagnostic-string assertion remains. No parser or production
event behavior changed, and no compatibility owner was added.

The active related-code field and acceptance path now name the canonical
resource owner; historical paths above remain original evidence. Current state:
`event_owner_consumer_repaired_managed_regressions_and_review_pending`.
Complete lower regression and upward acceptance, formal binding, independent
C0/I0/M0 review, return and closeout remain required. External zr_vm is skipped.

Source snapshot 3263 freezes the event-stream consumer correction at
`72e49d6a28ab67b8c7b27ee36825601d3d78132311655793ebd9eca08b4adbad`.
Its derived input `runtime15-event-owner-3263-20260909`, manifest
`a242dc7ec07c6c38ca4c7689b5cd78def4411472dd48e559d5d3ff701d98b151`,
changed only that guard. Managed lower-support job
`3ee45d339eba42bfa75f7e01ebd0d2d5` executed both `rust_source_view` tests:
1 passed, 1 failed, 0 ignored. Lexical/cfg masking passed; the recursive
shared-owner test found three real first-cfg truncation consumers in
`render_system_texture_generation_owner.rs`,
`render_ui_resource_upload_transaction.rs` and
`ui_dispatch_bound_text_model_updates.rs`. Their current bytes match that
sealed input. This supersedes the historical zero-residual count; the
complete failure list is in `results/runtime15-production-view-support-3263.{json,log}`.

All three files were HEAD-identical with no attribution. Audited transfer
`d30e5ca0b2b678e54a2bdc4d640181d4c2cbfe940347674e934e5a4a19195425`
and preimage snapshot 3268 preserve their provenance. Five truncation calls
now use the existing `production_section` owner, preserving production
strings while masking test-only items and comments. Each field, transaction,
resource-count and ordering assertion remains; no production source changed.
The shared-owner regression and the three affected consumer gates require
fresh execution after this same-root correction.

Lower producer evidence can be reused: all 84 `zr_resource` files in this
input are byte-identical to `frameworks01-readiness-payload-3237-20260908`.
Managed job `74f5113895dd40d78b1dcf7d1bf8a9fd` on that input passed 227
library tests with 0 failures and 10 ignored, including 26 actually executed
event-stream tests. This supports the unchanged producer only; it does not
accept the Runtime15 consumer or the remaining broader failure obligations.

Managed job `f4938ab737d24200b7fc4a4c65d0d39f` on the 3263 input
subsequently completed both `global_gate` tests: 2 passed, 0 failed,
0 ignored, 6848 filtered out. The repaired real event-stream regression
reached every remaining assertion, and the Runtime production lock scan
passed. The complete receipt and names are
`results/runtime15-event-owner-and-global-gate-3263.{json,log}`. This confirms
the event-owner repair and unchanged global production lock behavior, while
the three newly reported consumers still require their own verification.

Source snapshot 3269 freezes the three consumer corrections. Its intermediate
input `runtime15-production-view-3269-20260909`, manifest
`282ccef395eb2a007ee0e919e3c0d42f334caf24f9ced0f7faf1908886f448b1`,
has not been claimed as a passing validation. Preflight found that the bound-text
guard also reads two stale aggregate documents and one missing Text08 plan.
All three current documents are HEAD-identical and have no active owner;
transfer fingerprint
`3f49a60bc9ad10bc022929d682d8a2633517b4b84f6cdebe204f1d71f9a0ef33`
and snapshot 3271 freeze their unchanged content for the final input. They
are now explicit `related_code` inputs, without editing their wording or
relaxing the guard's assertions. The final support and three consumer runs
will use this complete input. No source/evidence review or closeout is claimed
for the current four-file Runtime15 increment yet.

## 2026-09-09 complete-input managed consumer results

Final input `runtime15-production-view-3271-20260909` contains 10,966 files,
manifest `532caeb7e8cda84b9b5e5a05ad099875a958a8f9a87803aa55ff683a595712b7`.
It derives from 3269 with exactly the three unchanged documents in snapshot
3271. All four guard corrections remain at their 3263/3269 hashes; no shared
parser, helper or production source changed during these runs. The following
Windows static/no-default/locked jobs actually executed the selected tests:

- `36ec218040ff40f38c449ee3d5709258`: both `rust_source_view` regressions,
  2 passed, 0 failed, 0 ignored, 6848 filtered out. This includes the lexical/cfg
  regression and the complete recursive scan that previously named three
  first-cfg truncation consumers. Artifact: `results/runtime15-production-view-support-3271.{json,log}`.
- `c7fe4f46798d42ba8752740907fa733e`: the exact system-texture generation
  consumer, 1 passed, 0 failed, 0 ignored, 6849 filtered out. All three changed
  production-view reads and the retained resource/count/order assertions ran.
  Artifact: `results/runtime15-system-texture-view-3271.{json,log}`.
- `c25d486342cf48d2bc26b76a8767fa1d`: the exact UI-upload consumer,
  0 passed, 1 failed, 0 ignored, 6849 filtered out. Line 40 expects six neutral
  buffer-upload sites but finds seven. The SDF vertex owner now has both full
  and range upload paths, at lines 49 and 101, hash
  `de682e8be261ef548a42337c2af83192212f9f73532edc2c6744642a18f99ed3`.
  Both are production `WgpuBufferUpload::from_bytes` calls. This failure occurs
  before the changed text production-view assertion; that assertion is not
  claimed as executed. Runtime15 retains this separate current-contract repair
  obligation. Artifact: `results/runtime15-ui-upload-view-3271.{json,log}`.
- `6b9d00a92d864c57b63c3a07f0082178`: the exact bound-text consumer,
  0 passed, 1 failed, 0 ignored, 6849 filtered out. Its changed profile view and
  forbidden-field assertions ran before line 151 rejected the 833-line input
  manager against the unchanged 800-line budget. The subsequent document
  status cohort was not reached. The aggregate structure document also lacks
  the two literal status strings, a static finding only. Artifact:
  `results/runtime15-bound-text-view-3271.{json,log}`.

The production input-manager ownership/budget failure is handed to Runtime09
in [its canonical record](../09/failure-2026-09-09-input-manager-bound-text-owner-budget.md).
No budget or production assertion was relaxed, no hidden fallback restored,
and neither failed consumer is recorded as GREEN. Source/evidence review is
requested for this bounded four-file increment; whole-lifecycle acceptance,
formal binding, canonical return, closeout commit and WeCom remain pending.

The user-designated independent task `01a07063-6f03-7803-a12d-13ea015ca645`
reviewed the four source corrections and both snapshot-3273 records, reporting
source C0/I0/M0 and evidence C0/I0/M0 with no reviewed-path ownership conflict.
Report: `.codex/tmp/runtime15-production-view-3269-review-20260909-result.txt`.
It verified current, attribution and ObjectStore hashes and distinguished the
three passing jobs from both failed consumers. This is bounded source/evidence
review, not full failure acceptance or formal closeout-review admission. The
separate archived-reviewer lifecycle gate remains open; no failed command was
resubmitted and no new commit or WeCom result is claimed here.

## 2026-09-09 UI upload-site contract continuation

After preserving the complete 0/1 result, Runtime15 corrected the independent
upload-site count in source snapshot 3280. The preimage is the 3269 hash
`086ba40e1fc466d3a5cae6b93fe253d1c3f8193158ee44618ebb2de9fcd8150f`;
the new `render_ui_resource_upload_transaction.rs` hash is
`4e271893c7b92c47df17f3b8a2c331959297683e1ddf68aece42b7d7959da0ec`.
Each of the six buffer owners is now checked independently using the shared
production code view: five owners have one neutral upload site each, and the
SDF vertex owner has its full and range sites. Each owner still rejects direct
`queue.write_buffer`; an extra site in one owner cannot compensate for a missing
site in another. The remaining transaction, retry, text and submission-order
assertions are unchanged.

All ten production input files read by this guard match input
`runtime07-eventbus-guards-3277-20260909`. The SDF full/range implementation and
all other production owners remain unchanged. Rustfmt and whitespace checks
passed. The new immutable input, actual full guard result and fresh review are
pending; the preceding C0 review covers only the earlier four-file snapshot,
not this later upload-count correction. Runtime09's manager-budget handoff
continues independently.

## 2026-09-09 UI submission owner continuation

The initial count repair ran on `runtime15-ui-upload-3280-20260909`, manifest
`fb5b62ac0dfd2b956d38dd7671360e7dabcf560768fb478d329a7e983bf54c01`.
Job `d0e016e95c484e57b4e61295750b6149` executed the exact UI guard:
0 passed, 1 failed, 0 ignored, 6849 filtered out. All per-owner upload counts,
transaction and text checks ran; the next failure expected the direct backend
receiver and submit method on the same line. Artifact:
`results/runtime15-ui-upload-view-3280.{json,log}`.

Snapshot 3282 corrected that formatting-sensitive receiver anchor and applied
the shared `production_code_view` to both submission paths, hash
`caf387cd62fa8c1956419583b374fe180ab6c0d55baf8f53ee9b30547a908e5d`.
Its input `runtime15-ui-upload-3282-20260909` has manifest
`bd0a27003ded9b3acb85a59cf3ac28f7fd4251a710770c8b34ca8d2c34d05d77`.
Job `906233c93581424da894fd33d7c5b1de` again executed the exact guard:
0 passed, 1 failed, 0 ignored, 6849 filtered out. It reached direct submitted
cubemap settlement and found the old inline call had moved into
`commit_scene_environment_frame`. The later compiled-path cohort did not run.
Artifact: `results/runtime15-ui-upload-view-3282.{json,log}`.

Source snapshot 3287, hash
`533977281aad7be8acd68a1e1b7efeeb538ae2b63303a767102105917314aeed`,
follows the current production call chain. The direct path must call its
environment-frame owner after submission and before fallible scene-ticket
validation, and that owner must contain the real cubemap commit. The compiled
path must hand prepared UI uploads to `commit_compiled_scene_frame_success`
after ticket validation; the child must commit every prepared upload and retain
the no-renderer empty-list assertion. Both newly read source owners already
match the sealed input, at hashes `2628964e5e21b9e50b9a38c8df140104b9792157054c5c205bc4c3253651d115`
and `659e5d1c35d3ea8d8718a39ce1f3bd12bd4d670b69985c9e64b6d6ed9dcbbd31`.
No production renderer, shared parser or transaction behavior changed.
Rustfmt passes for the current guard. The final managed run and fresh bounded
review remain pending; none of the preceding failed jobs is called GREEN.

## 2026-09-09 bound-text document contract continuation

After the Runtime09 parser extraction, job
`367b263ba0aa4a668dc2a7ab61a05030` ran the exact bound-text guard on
`runtime09-input-manager-3285-20260909`, manifest
`ef17397ec086dfdb9d2d93c90b8ccca4e44ba0c63700ee0597565d6cc53bd788`.
Result: 0 passed, 1 failed, 0 ignored, 6849 filtered out. All production-view,
source-contract and owner-size assertions ran, including the 758-line manager.
The first document cohort then failed because the structure convention uses
`runtime_09_15_focused_bound_model_update_owner_implemented_static_unvalidated`
and a pending-store zeroization status, not the two status identifiers shared
by review findings and the Text08 plan. Artifact:
`results/runtime09-bound-text-view-3285.{json,log}`. Neither of the two later
document cohorts is claimed as dynamically reached in this failing run.

Snapshot 3291 fixes only this document-contract mismatch in
`ui_dispatch_bound_text_model_updates.rs`, hash
`e004124ea5785f352b24f4f00870e7cb71290cd614c483554b6347198ebbc316`.
The structure convention must retain its exact current owner status, its
explicit Surface-owned secure-pending text contract, pending-store zeroization
status and the still-open persistent secure-document finding. Review findings
and Text08 retain both original required identifiers. No document wording,
production guard, owner budget or pending acceptance status is changed.
Intermediate snapshot 3290 had only the draft document-guard correction and
was not submitted as a validation input; snapshot 3291 supersedes it.

The complete UI input `runtime15-ui-upload-3287-20260909`, manifest
`7198c31a6fb989086bd536d8b1286a7237b7849cc343deab0aae0d2bc9297320`,
includes Runtime09's two source overlays and the corrected UI-upload guard.
It was sealed only after source derivation finished; a premature local wrapper
call failed before job submission because `snapshot.json` did not yet exist.
No managed request was accepted or replayed for that local preflight failure.
The next complete input adds only snapshot 3291 for both exact consumer runs.

The combined input `runtime15-consumer-guards-3291-20260909` is now sealed:
10,967 files, manifest
`564214103c8dfbbf4cd93a0063676b65f68e8e89c403808f7064a0b54e79a4ba`.
The first UI-upload submission, label `runtime15-ui-upload-view-3291`, was
rejected before any job or test was accepted: `cargo_reuse_pool_busy` named
owner `41500d963f7241009fc783e0d2d8724a`. Its JSON has empty jobs and tests;
the rejection is not a queued validation or a failed source test. That owner
has since returned its terminal UI-feature compile failure. The distinct
`runtime15-ui-upload-view-3291-r2` command was submitted once after that result
boundary; its terminal result is pending. No accepted request was replayed.

Both final exact consumer commands have now completed on that same immutable
input, with their receipts and unchanged source-manifest hashes verified:

- UI-upload job `28d1c6a25bca4549b3f69eb9ecb0b089`: 1 passed, 0 failed,
  0 ignored, 6849 filtered out. Every upload-owner, transaction, direct-submit
  and compiled-submit assertion ran. Artifact:
  `results/runtime15-ui-upload-view-3291-r2.{json,log}`.
- Bound-text job `05374c70f4e34c8887ffeb4aea08bd77`: 1 passed, 0 failed,
  0 ignored, 6849 filtered out. All production, size and three document-cohort
  assertions ran. Artifact: `results/runtime15-bound-text-view-3291.{json,log}`.

Both are native Windows, static, no-default-feature managed lib-test commands
with locked dependency resolution. No serial harness flag was requested or
inferred. The final UI and bound-text guard hashes remain snapshot 3287
`533977281aad7be8acd68a1e1b7efeeb538ae2b63303a767102105917314aeed`
and snapshot 3291
`e004124ea5785f352b24f4f00870e7cb71290cd614c483554b6347198ebbc316`.
These results close the two previously failing structure-consumer checks, not
the separate feature-enabled Runtime09 behavior requirement or whole-lifecycle
acceptance. Fresh bounded review and formal closeout admission remain pending.

The fresh bounded review payload is retained at
`.codex/tmp/runtime15-consumer-guards-3291-review-20260909.txt` and binds both
final source snapshots, both passing jobs and evidence snapshot 3297. At
2026-09-08T18:56:43Z the native resume of the designated task
`01a07063-6f03-7803-a12d-13ea015ca645` failed before review with
`thread-store conflict: already has an active writer` (JSON-RPC -32600).
The process terminated with exit 1, the JSONL is empty and no review result
file was produced. This is a suspended review request, not C0 evidence.
No replacement task or takeover was created; independent texture-descriptor
failure repair continued. The earlier 3269 review does not cover these final
consumer corrections.
