---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: scene-viewport-surface-projection-drift
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/zircon_runtime/text/01
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_surface/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/surface.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --test runtime_text_multilingual_product_framebuffer export_runtime_multilingual_text_product_framebuffer_png --locked --jobs 1 --color never -- --ignored --exact --test-threads=1
---

# Render17: SceneViewportSurface projection drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 来源执行切片：current-source Windows WGPU multilingual product framebuffer proof.
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 交接原因：viewport surface 的定义、scene-to-framework handoff 与 graphics facade projection 均落在 Render17 注册的 render-framework/scene-core 边界；Text01 只消费该框架以验证真实文本帧。

## 失败现象与复现证据

Managed Windows GPU job `9df4e4c92b2240d0b327178e751c0c60`, run `118117e4c9ea4313a7a94dff49ebfee6`, completed with exit `101` before the Text01 test body ran. The product command above reached `zircon_runtime` compilation and reported:

- `graphics/mod.rs:81` E0432: `scene::SceneViewportSurface` is not projected by `graphics::scene`, although `scene_renderer` defines it.
- `graphics/runtime/render_framework/viewport_surface/viewport_surface.rs:28` E0308: `ViewportRecord::bind_surface` accepts backend `ViewportSurface`, while `SceneRenderer::create_viewport_surface` returns wrapper `SceneViewportSurface`.

The same run also has independent Runtime11 E0499 at `operation/service.rs:237`; this handoff owns only the Render17 viewport-surface failures. Text01 did not claim or modify Render17 source paths.

## 最低共享层根因

The viewport-surface hard cut stopped halfway through the scene/render-framework boundary. `SceneRenderer` exposes a scene-owned wrapper, the framework record still stores the backend surface, and the scene facade lacks the re-export required by the graphics facade. These are one ownership/projection migration, not Text01 renderer or test failures.

## 架构修复验收

- Establish one explicit, consuming scene-to-framework surface handoff that preserves `ViewportRecord`'s single stored surface owner; do not retain parallel wrapper storage.
- Complete the required scene facade projection so the existing `graphics` facade only re-exports a reachable contract type.
- Add a focused Render17 regression that exercises viewport bind/unbind through the normal framework path.
- Rerun the original Text01 WGPU product command after the Render17 focused gate and Runtime11 lower compile repair return; it must reach the test body before it can be considered Text01 evidence.

## 禁止临时方案

- Do not add a type alias, compatibility re-export, call-site cast, or duplicate viewport-surface state merely to satisfy one import.
- Do not weaken the Text01 product test, skip its native atlas path, or treat its pre-test compile exit as screenshot evidence.


## 修复结果与回传

Open state: `SceneViewportSurface`已通过`graphics::scene`/`graphics` facade投影，并以consuming `into_backend_surface`交给唯一`ViewportRecord` surface owner；bind/unbind结构契约回归已落地。待受管current-source framework gate及原Text01 WGPU产品命令复跑后回传`fixed-*`，当前不声明pass。
