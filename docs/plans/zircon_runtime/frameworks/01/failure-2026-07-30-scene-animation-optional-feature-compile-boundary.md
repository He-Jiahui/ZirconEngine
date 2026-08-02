---
handoff_kind: failure
status: open
created_at: 2026-07-30
summary_slug: scene-animation-optional-feature-compile-boundary
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
fixing_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
origin_child_dir: docs/plans/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_runtime/frameworks/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/level_system/frame_state.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
  - zircon_runtime/src/scene/tests/level_system_frame_state.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib level_system_constructs_and_replaces_world_without_animation --no-default-features --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_plugin_sdk --lib runtime_registration_builder_hides_module_owner_sequence --locked --jobs 1 -- --nocapture --test-threads=1
  - Runtime animation-enabled LevelSystem clip-event behavior remains covered without a scene-to-animation reverse dependency
---

# Frameworks01: scene animation optional-feature compile boundary

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 来源执行切片：per-World scene-system callback factory SDK forwarding validation
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 交接原因：Frameworks01 owns the declared `scene->animation` reverse dependency and the optional-domain decomposition boundary. Plugin SDK must not make an optional Runtime animation domain mandatory merely to compile its runtime registration builder.

## 失败现象与复现证据

Managed source-bound job `d30d4e49a2aa474f9143efbf547fbff6` /
`5dffe135b5f54b9aa1f3da35ef9a60c0` ran:

```text
cargo +1.94.1 test -p zircon_plugin_sdk --lib runtime_registration_builder_hides_module_owner_sequence --locked --jobs 1 -- --nocapture --test-threads=1
```

It terminated `exit 101` before the requested SDK test binary ran. The SDK's `runtime` feature selects
`zircon_runtime` without its default optional domains, and that dependency compile produced three errors:

- E0432 at `zircon_runtime/src/scene/level_system.rs:6`: unconditional `crate::animation` import while `lib.rs` gates `animation` behind `feature = "animation"`.
- E0432 at `zircon_runtime/src/scene/level_system/frame_state.rs:4`: the same unconditional dependency on the animation clip-event cursor.
- E0689 at `zircon_runtime/src/scene/level_system.rs:314`: `emitted_event_bytes = 0` has no concrete numeric type for `saturating_add(batch.emitted_event_bytes)`.

The lower Runtime callback GREEN job had already executed all four callback tests successfully, so this
record is compile-blocker evidence only and does not reinterpret either callback or SDK behavior.

Managed no-default-features retry `d2bad3c6a3dc40d5860f11d1400003e9` /
`4b6f8c4dc4e542fdac9c5e16fb4aa62e` reached the `zircon_runtime` lib-test compiler but exited
`101` before its requested test executed. It proved two further local boundary leaks: a statement-level
`#[cfg]` at `level_system.rs:181` (E0658) and four animation-pose tests in
`scene/tests/level_system_frame_state.rs` that compile without the `animation` feature (E0599).
It also exposed an independent Frameworks01 M1 test-owner error: `core/runtime/tests/tasks.rs` imports
the now-private `core::framework::render::environment` module (E0603). That file belongs to the active
M1 contracts/kernel test-boundary session and is not repaired by this failure scope.

## 最低共享层根因

`scene::level_system` stores clip-event sampling cursor/state in its always-compiled frame-state owner
and imports `crate::animation` unconditionally. `zircon_runtime::animation` is optional, so a consumer
that legitimately omits the animation domain compiles an invalid scene module. The untyped event-byte
accumulator is a second latent compile error in the same optional code path. The mixed render-extract
source-guard child also compiled three animation-only assertions under every feature selection. Only
those animation-specific cases may carry the feature boundary; its two always-on scene/render
architecture guards must remain compiled without the animation feature.

## 架构修复验收

- Scene's always-compiled LevelSystem and frame-state owners build when Runtime animation is absent.
- Clip-event sampler/cursor state has an explicit animation-enabled owner or scene-facing contract boundary; no direct always-on scene import reaches the optional animation module.
- The event-byte budget preserves its concrete byte-count type and saturating semantics.
- The exact Plugin SDK command above compiles and executes exactly its requested test.
- Animation-enabled clip-event behavior and the original Plugins01 four-test callback GREEN remain passing under managed source-bound validation.

## 禁止临时方案

- Do not force `animation` into the Plugin SDK runtime feature or otherwise broaden consumer feature selection to hide the broken boundary.
- Do not restore a scene-to-animation facade, re-export, alias, or a test-only cfg bypass.
- Do not weaken clip-event budgeting, event continuity, or the SDK test filter.

## 修复结果与回传

2026-08-01 current-source hard cut:

- `level_system.rs` and its frame-state owner now gate every direct optional animation import,
  state field, clip-event cursor, and animation-only method behind `feature = "animation"`; the
  always-compiled LevelSystem/frame snapshot remains available without that domain.
- The final reset call in `replace_world_and_reset_runtime_state` is enclosed by an item-stable cfg
  block instead of a statement-level cfg attribute. This closes the remaining Rust E0658 boundary
  without enabling animation for Plugin SDK consumers.
- The clip-event byte accumulator is explicitly `usize` and retains saturating addition. Animation
  pose/source-guard tests carry the animation cfg, while the two scene/render architecture guards
  remain always compiled.
- The first independent review found that a module-level cfg still hid all five mixed
  `level_source_guards` cases. The cfg now sits only on the three animation behavior tests and their
  parent imports/helper; the snapshot-adapter and inactive-camera architecture guards compile in
  every feature selection. A focused source guard fixes this at `3` animation-only plus `2`
  always-on cases.
- Scoped Rust 1.94.1 rustfmt, diff-check, production feature-boundary source guards, and test cfg
  guards are GREEN. The first independent review was C0/I1/M0 and its Important finding is repaired;
  the fresh exact-scope re-review is C0/I0/M0 Ready. Managed Runtime no-default, Plugin SDK, and
  animation-enabled gates are still required before the failure can return as fixed.

Open state: `implementation_static_and_secondary_review_green_managed_validation_pending`; no SDK or
Runtime Cargo pass is claimed.
