---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: scene-binding-generations-visibility
origin_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/zircon_runtime/runtime/11
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/world/compiled_binding/mod.rs
  - zircon_runtime/src/scene/world/compiled_binding/generation.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
tests:
  - cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1
---

# Runtime08: SceneBindingGenerations visibility

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 来源执行切片：Runtime11 bounded operation service current-source compile gate
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：Runtime08 owns the generation-authoritative compiled scene binding boundary under `scene::world`; Runtime11 has no authority to change its type visibility or re-export topology.
- 生命周期键: `scene-binding-generations-visibility`

## 失败现象与复现证据

Managed Windows job `792f8a5fa4f146b4952847d756c99778` / run `fec70895dfe34049b36e3a306d3e696e` executed `cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1` and naturally terminated `exit 101`. Its raw stderr reported one `E0365` at `scene/world/compiled_binding/mod.rs:4` and two `E0603` imports in `scene/world/world.rs:5` and `scene/world/bootstrap.rs:7`: `compiled_binding` re-exports `SceneBindingGenerations` to its parent while `generation.rs` declares the type only `pub(super)`, so the parent `scene::world` consumers cannot name it.

## 最低共享层根因

The defining type's visibility is narrower than the intended `scene::world` internal re-export boundary. The compiled-binding module and its parent consumers disagree about ownership scope; this is a Rust module contract error, not a Runtime11 operation failure.

## 架构修复验收

- Runtime08 makes `SceneBindingGenerations` visible exactly throughout the owning `scene::world` boundary and keeps the compiled-binding re-export consistent with that scope.
- No consumer outside the intended world boundary gains access, and `world.rs` plus `bootstrap.rs` retain one generation authority rather than duplicating binding state.
- The original `cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1` passes after the Runtime11 operation retry has also cleared its independent `E0499` repair.

## 禁止临时方案

- Do not widen the type to a crate/root public API, add an alias, duplicate the generation state, or route callers around the compiled-binding owner.
- Do not remove the world/bootstrap imports or gate them out merely to hide the visibility violation.

## 修复结果与回传

- `SceneBindingGenerations` now uses `pub(in crate::scene::world)`. Its fields and mutation methods remain private to `compiled_binding`, and the parent re-export remains `pub(super)`; no public alias or compatibility path was introduced.
- Independent review accepted that visibility boundary, then found a lower Runtime08 correctness defect: removing and reinserting a root with the same `EntityId` did not advance that root's binding generation, so a retained descendant-name index or compiled property target could be reported current for a different entity lifetime.
- Two focused regressions now cover same-ID root reuse for both compiled binding types. The first managed reservation `1bafa88d371049d29b2eb350c025e4f2` was atomically rejected before process spawn because concurrent owner additions changed four bound files; job `b76298a7fa9b4ec8b8771e5fd2dc0280` was released with no Cargo PID and supplies no test result. Reservation `db5d97f8ca3046599d91d432ea29d56b` was then explicitly released unbound when the repository milestone-first policy moved unit execution to the completed implementation boundary; it also supplies no test result.
- Removal now advances the removed entity's tombstone generation together with its previous ancestor chain. Insertion starts generation advancement at the inserted entity and follows its current hierarchy, so root and descendant identity reuse both invalidate retained bindings without a second identity store or compatibility API.
- Independent review of snapshot `1273` completed `Critical 0 / Important 0 / Moderate 0 / Minor 0, Ready`; both start and end previews reported exact12 zero drift.

Resolving state: implementation, focused regressions, and independent review are complete. The owner must obtain the current-source managed GREEN, commit the exact owner scope, and only then return the fixed lifecycle to Runtime11.
