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
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zr_rhi_wgpu -SkipBuild -LibTests -TestFilter tests::surface_lifecycle:: -VerboseOutput
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -LibTests -TestFilter journal -VerboseOutput
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

## 2026-09-19 rolling successor formal source binding

- Successor Session `failure-roll-01a084c8-runtime90-surface-r4` reclaimed the canonical
  failure record and `zircon_runtime/crates/zr_rhi/src/surface.rs` through coordinator transfer
  fingerprint `b51e4ec9b76001ef855b4ed41e0ee28a59d917991718c1d2fe732bc42a6e0b95` at baseline
  epoch `611`; no source bytes changed during attribution.
- A successor Windows source-contract ticket will assert the single-lock allocation sequence,
  checked overflow/stale handling, and all focused session/frame allocator regressions. It
  defers fresh current-source Cargo, Editor17's originating journal gate, independent C/I/M
  review, fixed return and closeout; the two prior managed GREEN jobs remain supporting
  evidence only.
- Request `failure-roll-01a084c8-runtime90-surface-20260919-r1` admitted ticket
  `4b75d0e349b547be8b9c85fabe0e4566` with sealed manifest hash
  `4d7cf0ceca398ca3d4063eb95f3dc7485e2968563e03a4fe163c806f6092af0d`; status is `queued`
  pending the coordinator terminal result.

## 2026-09-19 corrected source-contract ticket terminal evidence

- The coordinator terminalized ticket `4b75d0e349b547be8b9c85fabe0e4566` as
  `passed` at `2026-09-19T05:42:13.094075Z` (run id equal to the ticket id).
- Managed validation job `c76fba5f49e8493bb255bcc7b3bb985c` exited `0`; the
  terminal stdout marker was
  `RUNTIME90_RHI_SURFACE_ALLOCATOR_SOURCE_CONTRACT_PARSE_PASS`. The coordinator
  recorded an empty stderr tail and completed cleanup (`event 10860`).
- This is a current-source static contract pass only. Fresh managed `zr_rhi`
  Cargo validation, the Editor17 originating journal gate, independent
  Critical/Important/Moderate review, canonical fixed return, and failure
  closeout remain pending. The external `E:\Git\zr_vm` dirty-worktree blocker
  remains unchanged.

## 2026-09-21 independent source review receipt

- Reviewer Session `review-runtime90-surface-r4` inspected the current owned
  `zircon_runtime/crates/zr_rhi/src/surface.rs` without editing it. The source
  remains byte-identical to the sealed static ticket manifest at SHA-256
  `0de95d13171efc2b1d5e2fc6a0ce308e108576e0484497879f313d5983e3b996`.
- The review re-ran `rustfmt +1.94.1 --edition 2021 --config
  skip_children=true --check` and scoped `git diff --check`; both passed with
  markers `RUNTIME90_RUSTFMT_PASS` and `RUNTIME90_DIFF_CHECK_PASS`.
- The independent source probe passed as
  `RUNTIME90_RHI_SURFACE_ALLOCATOR_REVIEW_PASS`. It verified that allocation
  advances the selected counter and publishes into the matching active set
  within one lock domain, uses checked overflow handling, and contains no
  overlapping `(&mut next_counter, &mut active_set)` borrow. It also verified
  the focused session/frame monotonicity, stale-after-release,
  overflow-without-publication, bounded terminal-history, owner-validation,
  and foreign-allocator regression contracts.
- Independent review result: **Critical=0 / Important=0 / Moderate=0**. No
  Editor17 or foreign RHI source was absorbed.
- This receipt does not promote the two historical green jobs to current
  acceptance. Fresh managed `zr_rhi` Cargo validation and the Editor17
  originating journal gate remain pending because external `E:\Git\zr_vm` is
  dirty. Canonical `fixed-*` return, closeout, and WeCom notification remain
  pending until matching source-bound dynamic evidence exists.

## 2026-09-28 acceptance command correction

The `tests` field now includes the direct WGPU surface lifecycle consumer and
the originating Editor17 `journal` gate alongside the two original lower RHI
commands. This only corrects the executable acceptance scope. The filters must
execute nonzero target tests under managed Windows `--locked` validation; no
new Cargo pass, failure return, closeout, or notification is claimed here.
