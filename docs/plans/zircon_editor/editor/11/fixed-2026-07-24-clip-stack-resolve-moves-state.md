---
handoff_kind: fixed
status: fixed
created_at: 2026-07-23
summary_slug: clip-stack-resolve-moves-state
origin_plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
fixing_plan: docs/plans/zircon_editor/editor_layout/21-gpu-submission-and-draw-pipeline.md
origin_child_dir: docs/plans/zircon_editor/editor/11
fixing_child_dir: docs/plans/zircon_editor/editor_layout/21
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/surface/render/batch/clip.rs
  - zircon_runtime_interface/src/ui/surface/render/batch/tests.rs
tests:
  - cargo test -p zircon_runtime_interface --lib clip_stack_intersects_nested_axis_aligned_scissors --locked --jobs 1 -- --test-threads=1
resolved_at: 2026-07-24
---


# Layout21：clip stack resolve 移动状态后再次使用

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/11-serialization-and-versioning.md`
- 来源执行切片：current canonical text focused managed gate
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/21-gpu-submission-and-draw-pipeline.md`
- 交接原因：最低错误位于 Layout21 的 render batch clip stack 所有权边界，不属于 serialization。

## 失败现象与复现证据

Editor11 managed job `5e6a433aa91f41e485c879a156bb09af` / run
`65496a78b5034a65b52a19ed407e2343` 在编译 `zircon_runtime_interface`
lib tests 时 exit 101、目标测试 0。唯一错误是
`ui/surface/render/batch/tests.rs:123` E0382：`resolve(nested)` 消耗
`UiClipState`，随后 `pop() == Some(nested)` 又使用该值。

## 最低共享层根因

`UiClipStack::resolve` 只执行只读 identity lookup，却错误取得 `UiClipState`
所有权。调用者因此必须丢弃仍用于 stack/pop 验证的状态，所有权契约比行为所需更宽。

## 架构修复验收

- `resolve` 只借用查询状态，不 clone、不消耗调用者所有权。
- nested 和 disjoint clip stack 回归均通过。
- 原 Editor11 focused gate 重新编译并执行目标测试。

## 禁止临时方案

- 不允许在调用点 clone `UiClipState` 掩盖错误所有权契约。
- 不允许弱化 pop/identity 断言、增加兼容 overload 或 test-only bypass。

## 修复结果与回传

- 根因：UiClipStack::resolve consumed UiClipState even though resolution is an identity lookup, so the caller could not verify the same state after pop.
- 架构修复：Hard-cut resolve to borrow &UiClipState and return a borrowed match; callers retain ownership, with no clone, overload, compatibility shim, or weakened assertion.
- 验证：Managed job 38b9f0a394b2408f903451f487e63f8b / run a1e27c4eda524f0d9f63462718b1af07 passed clip_stack_ 2/2. Origin Editor11 final interface job 3b3ef168188d4506b7dbab69c5e9cafe / run 55fdde02f7124283838cc6f61bb1a6e5 passed serialization 50/50.
- 回传：Layout21 borrowing contract is fixed and the originating Editor11 current serialization gate compiles and passes; return the cross-plan failure for atomic closeout.
