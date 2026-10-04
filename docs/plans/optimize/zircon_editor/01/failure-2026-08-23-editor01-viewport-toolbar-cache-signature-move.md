---
handoff_kind: failure
status: open
failure_scope: cross_plan
created_at: 2026-08-23
summary_slug: editor01-viewport-toolbar-cache-signature-move
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/optimize/zircon_editor/01
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/surface_frame_cache.rs
---

# Editor01 viewport-toolbar cache signature move: validation failure handoff

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M6 current-source Editor bundle and real WGPU visual acceptance
- 修复责任计划：`docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md`
- 交接原因：Editor01 owns the newly added viewport-toolbar surface-frame cache and its
  performance contract.

## 失败现象与复现证据

- Managed command: `tools/build/build-editor.ps1 -TargetDir
  D:\cargo-targets\zircon-engine\ui12\bundle-current-bee4c707-20260822 -OutputDirectory
  D:\ZirconBuilds\ui12-editor-aa-current-bee4c707-20260822`.
- Managed Job: `95421ec3365b4a6b9223b3a0647f1374`; released with exit code 1 at
  2026-08-23 02:17:54 +08:00. No final bundle was published.
- Current-source diagnostic: `E0507` at
  `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/surface_frame_cache.rs:78`.
  `SurfaceFrameSignature::with_hit_control_ids(self, ...)` attempts to move
  `cached.signature` out of an existing `&mut CachedSurfaceFrame`.
- Coordinator ownership preview identifies source owner
  `optimize-editor01-ui-profile-visual-cache-r1-a9220896-20260822` as active and executable.
  The file is currently untracked and is not in UI12's write scope, so UI12 did not modify it.
- The preceding managed Editor build reached and linked current-source `zircon_editor.exe`; this
  single later source regression is now the current bundle blocker.
- Editor01 updated the cache with a `hit_route_key` fast path at 2026-08-23 02:47-02:50 +08:00,
  moving the same consuming call to current-source line 99. The branch still evaluates
  `cached.signature.with_hit_control_ids(...)` before assigning the returned signature back, so
  the `E0507` remains present; the new hit-route work does not close this handoff.

## 最低共享层根因

The remap branch already holds a mutable cache entry, but its helper consumes the complete
signature and the branch then replaces the whole map entry. Ownership and cache-update authority
are therefore misaligned: an in-place cache update is expressed as a move out of a borrowed field.

## 架构修复验收

- Update the existing mutable cache entry in place: replace mapped hit-control IDs on
  `cached.signature`, rebuild the frame from that signature, then assign `cached.frame` and
  `cached.last_used_generation`.
- Do not clone the complete `SurfaceFrameSignature` or its node vector on this remap hot path.
- Preserve the existing cache-hit and cache-reproject counters and returned `Arc<UiSurfaceFrame>`
  behavior.
- Add or retain a focused regression that exercises a same-size frame whose mapped hit-control IDs
  change and verifies one reproject followed by a cache hit.
- Re-run the persistent-target managed Editor bundle build past this `E0507`.

## 禁止临时方案

- Do not add `cached.signature.clone()` merely to satisfy the borrow checker.
- Do not bypass the cache, disable hit-control remapping, or move the cache into UI12 code.

## 修复结果与回传

Open state: awaiting the active Editor01 owner repair and current-source managed validation.

## 2026-09-11 rolling repair admission

- The current source was rechecked at `c37155ba304740b3762b20585f77fb53a6da47fb` under
  session `failure-roll-01a084c8-editor01-viewport-toolbar-cache`; the lease covers this
  failure record and `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/surface_frame_cache.rs`.
- Snapshot `3409` sealed the failure record and source. The source hash is
  `f046194d51347229dfe09ce342b4ce9bf0008eae4495bca611b353354761c771`; the failure record
  hash is `b69094d4518925f632f22c1ebfe5ca9a57f5ff3a559426095b4ada3089d8df14`.
- Direct managed validation request
  `editor01-viewport-toolbar-cache-20260911-r1` was submitted with
  `cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked`.
  The coordinator returned `internal_error` with correlation/request id
  `17080839e7bc42218f34fdbc46b7feb5`.
- Read-only coordinator inspection at 2026-09-11 07:25 +08:00 found that request still
  `accepted`, with no `validation_ticket_requests`, no `validation_tickets`, and no
  terminal response or error persisted. This is an interrupted coordinator command, not
  dynamic Cargo evidence; no pass, upward acceptance, review, fixed return, or closeout is
  claimed. The item remains waiting for coordinator request reconciliation and a fresh
admitted validation after the external validation state is healthy.

## 2026-09-12 local ownership repair

The current owner repair is now present in
`surface_frame_cache.rs`: same-size route changes mutate the borrowed
`SurfaceFrameSignature` in place and rebuild the published frame only when a
mapped hit-control ID changes. The route-key update reuses the cache entry's
owned vector and string capacities, and initial signature construction reserves
the projection upper bound. Focused in-file regressions cover the remap and
route-key contracts; the batched Editor cache/pointer contract run passed
`43/43` in `0.813s`.

The refreshed Runtime/Editor performance-plus-pressure batch loaded `536`
modules and passed `1993/1993` in `12.193s` after the Runtime712 summary extension. This local evidence resolves the
previously described move-out shape but does not close the handoff. The record
remains open for the owner-attributed managed Cargo build and current-source
Editor Release/product measurements; no coordinator state was polled or
re-submitted during this repair.

Runtime714 subsequently added the defensive unordered-line text-decoration fallback and shared
the touched source map with caret projection in the Runtime Interface path. The current combined
static batch remains `1993/1993` across 536 modules in `12.015s`; a later single-invocation rerun
after this cache change passed in `8.398s`. This does not alter the handoff's managed validation
requirement.

## 2026-09-19 rolling successor source reconciliation

- Successor Session `failure-roll-01a084c8-editor01-toolbar-cache-r2` reclaimed the
  cache source and canonical record at baseline epoch `611`. Ownership transfer
  fingerprint: `8597e6f93875bbf821a01d2071ec7bff8e3185e36f175c653b1cb4c646728e75`.
- Current source mutates `SurfaceFrameSignature` in place through
  `remap_hit_control_ids`, updates retained route-key allocations, and rebuilds
  the frame only when mapping changes. Current working file also contains the
  owner's focused capacity/remap tests; no bytes were reverted or attributed
  beyond scope.
- Fresh managed Editor Cargo/build, current-source independent review binding,
  fixed return, closeout, and clean `E:\Git\zr_vm` remain pending. No dynamic
  acceptance is inferred from prior local `43/43` or `1993/1993` evidence.

## 2026-09-19 corrected static source-contract ticket

- Current-source checker executed locally with the exact managed command payload and
  passed `EDITOR01_VIEWPORT_TOOLBAR_CACHE_IN_PLACE_REMAP_CURRENT_SOURCE_CONTRACT_PASS`.
- Managed validation ticket `42bd56b9d978415890ecc122709dae27` was admitted and is
  queued under request `failure-roll-01a084c8-editor01-toolbar-cache-20260919-r1`.
  The ticket binds the current failure-record hash
  `8d19a28047fca249524e7c225429e0d75bdf14e2770db087c7ca811eca105c8e` and source hash
  `4416a22121dff46c060a822f2d36af653f8f2f7ee2d8af0439abf5cf30511500`.
- The ticket is static-only (`staticParseOnly=true`, `upwardAcceptance=false`) and
  records the external `E:\Git\zr_vm` dirty-worktree blocker. It does not establish
  the required dynamic Editor Cargo/build, independent C/I/M review, fixed return, or
  failure closeout.

- Terminal result: ticket `42bd56b9d978415890ecc122709dae27` reached `passed` at
  `2026-09-19T07:55:09.424445Z`; the managed Python contract emitted
  `EDITOR01_VIEWPORT_TOOLBAR_CACHE_IN_PLACE_REMAP_CURRENT_SOURCE_CONTRACT_PASS`.
  This is source-contract evidence only and leaves the dynamic and closeout gates open.
