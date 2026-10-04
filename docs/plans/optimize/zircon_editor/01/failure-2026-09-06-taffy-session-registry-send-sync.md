---
handoff_kind: failure
status: open
created_at: 2026-09-06
summary_slug: taffy-session-registry-send-sync
plan_link_mode: child_record_only
origin_plan: docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
fixing_plan: docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
origin_child_dir: docs/plans/optimize/zircon_app/08
fixing_child_dir: docs/plans/optimize/zircon_editor/01
related_code:
  - zircon_runtime/src/ui/layout/taffy_bridge/product_cache.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/compute.rs
  - zircon_runtime/src/ui/layout/pass/slot.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
  - zircon_runtime/src/dynamic_api/session/registry/tests.rs
  - zircon_runtime/src/ui/tests/taffy_layout_diagnostics.rs
tests:
  - managed target-client Runtime check with dev-dynamic linking and --lib
---

# Editor01: retained Taffy product session-registry Send/Sync boundary

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md`
- 来源执行切片：App08 client Runtime development-DLL validation
- 修复责任计划：`docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md`
- 交接原因：the client check reaches the retained Taffy layout product owned by Editor01 and then fails at the dynamic session registry's thread-safety boundary.

Implementation records: [App08 compiler reuse](../../zircon_app/08/2026-08-30-runtime-artifact-reuse-compact-validation.md) and [Editor01 retained Taffy product](2026-08-30-runtime-taffy-retained-parent-product.md).

## 失败现象与复现证据

The managed Windows check exits during `cargo check`, before code generation, linking, or tests, with `E0277` because the session registry's global `Mutex` requires `SessionRegistry: Send`, while the retained `TaffyTree<()>` path contains Taffy's `CompactLength` pointer representation. The latest receipt is job `71fbe8f2e5f04f41a8b65e23a9282b48`: queue 22.60 s, source synchronization 176.99 s, check 497.97 s, zero compile/link and test seconds.

Reproduce through the managed entry:

```powershell
& .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -RepoRoot E:\Git\ZirconEngine -Package zircon_runtime -Features target-client -NoDefaultFeatures -LibTests -CheckOnly -LinkMode dev-dynamic -StorageMode reuse
```

## 最低共享层根因

The retained per-parent Taffy product is stored in the live session-owned layout path, but its `TaffyTree<()>` is not accepted by the dynamic session registry's `Send + Sync` boundary. The ownership and synchronization model must be repaired at the retained layout/session boundary before client or Editor acceptance can resume.

## 架构修复验收

- Keep retained Taffy ownership explicit and make the live session boundary satisfy its actual thread-safety contract.
- Run focused retained-layout tests and the Runtime dynamic API registry tests.
- Rerun the exact managed target-client check, then the Editor target-editor-host check and focused tests.

## 禁止临时方案

- Do not add `unsafe impl`, Editor-only aliases, compatibility shims, test-only exports, or call-site exceptions without proving the owning synchronization invariant.
- Do not exclude the retained layout modules or weaken the client check.

## 修复结果与回传

Open state: `待修复`; no client, Editor, DLL-reuse, or performance acceptance is claimed.

The initial marker removed the Taffy/global registry `E0277` in managed job
`54a57a1c8eae41bb8c6627b203437db1` (sealed input
`235ccf8a63bc9385aadd1d959f1b5dc658a45e97f5ccc6ca2cb1b05befac3456`).
That check still failed with 508 coded source errors and ran no tests.

The reviewed patch marks only private `TaffyParentProduct` as `Send`, leaving its
cache to derive the auto trait from its fields. The product owns `TaffyTree<()>`
and numeric styles constructed by the bridge; it cannot receive external calc
handles. This invariant does not rely on Cargo feature unification keeping calc
disabled. Scratch styles remain thread-local. Local Bevy's `UiTree` in
`dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs` uses the same no-calc invariant.
The bridge's missing `TaffySize` import is restored. The registry drain test moves
only `ZrStatusCode` across its thread boundary, leaving the C ABI pointer-bearing
`ZrStatus` contract unchanged.

`retained_taffy_surface_moves_between_threads_and_reuses_layout` warms a real
surface, moves it to another thread, checks unchanged geometry and tree reuse,
resizes it, and drops it on the receiving thread. The regression and original
client/Editor commands must pass before this handoff can be returned as fixed.

The focused managed rerun `9e45fc19097d4bfb8267d1647e40cc2a` checked input
`2ecb063d569271df1206014820f56e391b921ee49607482893247d62e0c8b6c4` and reported no
errors in the changed Taffy/registry files. It still failed with 503 total source
errors before the test could execute: queue 12.21 s, synchronization 146.02 s,
check 346.36 s, compile/link and test 0 s. Log:
`.codex/tmp/app08-taffy-client-20260906-r2.log`. A separate read-only review found
no actionable issue in the numeric-style ownership proof or regression. This is
compile-diagnostic progress only; the handoff remains open for execution evidence.
