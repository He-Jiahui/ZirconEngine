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

- Managed command: `tools/build-editor.ps1 -TargetDir
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
