---
handoff_kind: failure
status: open
created_at: 2026-08-24
summary_slug: rhi-surface-allocator-borrow-compile
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/optimize/zircon_runtime/90
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_rhi/src/surface.rs
tests:
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zr_rhi -SkipBuild -LibTests -TestFilter surface_handle -VerboseOutput
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zr_rhi -SkipBuild -LibTests -VerboseOutput
---

# Runtime90: RHI surface allocator mutable-borrow compile blocker

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：P1-10 durable transaction journal current-source managed validation
- 修复责任计划：`docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md`
- 交接原因：失败发生在 editor journal 进入编译前的共享 `zr_rhi` surface handle allocator；Runtime90 明确拥有 RHI surface/device integration，Editor17 不能在上层规避或修改该所有者。

## 失败现象与复现证据

2026-08-24 在 coordinator-managed D: test lane 执行：

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -LibTests -TestFilter journal -VerboseOutput
```

`Cargo build` 和随后 `Cargo test` 均以 exit 101 停止，尚未到达 `zircon_editor`。编译器对
`zircon_runtime/crates/zr_rhi/src/surface.rs` 报告两处 E0499：

- `allocate()` line 233：`SurfaceHandleKind::Session` 同时借用 `state.next_session` 与 `state.active_sessions`；
- `allocate()` line 234：`SurfaceHandleKind::Frame` 同时借用 `state.next_frame` 与 `state.active_frames`。

验证 stdout/stderr 记录在同一 coordinator-managed D: temporary directory；没有 C: 或仓库 `target` Cargo 产物。

## 最低共享层根因

`RenderSurfaceHandleAllocator::allocate()` 在同一个 `MutexGuard<RenderSurfaceHandleAllocatorState>` 上构造
`(&mut next_counter, &mut active_set)` 元组。两个字段逻辑上相互独立，但该 match expression 不能通过 Rust 的同时可变借用检查，导致整个 `zr_rhi` crate 无法编译。当前 `surface.rs` 是 worktree 中未跟踪的 Runtime90 RHI 变更，来源 Editor17 没有其所有权，也没有可接受的上层替代路径。

## 架构修复验收

- Runtime90 在 `surface.rs` 的 allocator owner 内重构 allocation state access，使 session/frame counter 与 active set 的更新在一个锁域内且不依赖非法重叠借用。
- 保持 opaque handle 的 namespace、generation、单调 sequence、overflow fail-closed 与 release 后不可复活不变。
- 为 session 和 frame allocation 的单调性、release 后 stale、overflow/foreign allocator 边界保留或补充 focused tests。
- 先运行 `zr_rhi` focused tests，再重跑上述 locked coordinator-managed `zircon_editor` journal gate。

## 禁止临时方案

- 不得在 Editor17 停用 surface/RHI 依赖、降级 feature、跳过 build，或添加上层 conditional workaround。
- 不得通过复制 allocator、兼容 alias、全局 mutable state 或取消 handle validation 规避借用错误。
- 不得弱化原始 `zircon_editor` journal 验证门。

## 修复结果与回传

The allocator now obtains and advances the selected sequence counter before mutating
the corresponding active set, while both operations remain in the same lock domain.
Focused regressions cover session/frame monotonicity, allocator-local identity,
overflow fail-closed behavior, and foreign-allocator rejection.

Managed job `18086ba7d85f496b8dda823e9e1be17a` ran the surface-handle filter and
released with exit code 0. Managed job `b5522b23945e4c70837b8dacb18b145c`
then ran the complete 78-test `zr_rhi --lib` suite and released with exit code 0;
the original E0499 no longer appears.

Open state: `current-source repair and managed validation green / Runtime90 atomic
integration pending`. Editor17's original product-level journal gate remains owned by
Editor17 and is not claimed by this lower-layer result.
