---
related_code:
  - zircon_runtime/src/core/framework/gizmos/mod.rs
  - zircon_runtime/src/core/framework/gizmos/buffer.rs
  - zircon_runtime/src/core/framework/gizmos/command.rs
  - zircon_runtime/src/core/framework/gizmos/config.rs
  - zircon_runtime/src/core/framework/gizmos/extract.rs
  - zircon_runtime/src/core/framework/gizmos/retained.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/core/framework/mod.rs
implementation_files:
  - zircon_runtime/src/core/framework/gizmos/mod.rs
  - zircon_runtime/src/core/framework/gizmos/buffer.rs
  - zircon_runtime/src/core/framework/gizmos/command.rs
  - zircon_runtime/src/core/framework/gizmos/config.rs
  - zircon_runtime/src/core/framework/gizmos/extract.rs
  - zircon_runtime/src/core/framework/gizmos/retained.rs
plan_sources:
  - user: 2026-05-08 continue Runtime Picking / Gizmos / Camera / Remote Bevy completion plan
  - .codex/plans/runtime-picking-gizmos-camera-remote-bevy-completion-plan.md
  - docs/superpowers/plans/2026-05-08-runtime-gizmos-m3.md
tests:
  - zircon_runtime/src/tests/gizmos/mod.rs
  - cargo test -p zircon_runtime gizmo --locked
doc_type: module-detail
---

# Runtime Gizmos Framework Contracts

## Purpose

`zircon_runtime::core::framework::gizmos` is the M3 runtime-owned debug geometry contract layer. It lets runtime systems, editor adapters, dev tools, and future plugins record neutral gizmo commands without depending on editor viewport code or graphics renderer internals.

This module is deliberately a framework DTO/helper layer, not a renderer and not editor authoring state. It records immediate commands, retained gizmo assets, and rendering policy metadata, then offers a one-way adapter into the existing `RenderOverlayExtract` scene-gizmo packet. The existing overlay packet remains the graphics-facing output boundary until a later rendering milestone grows dedicated gizmo rendering metadata.

## Reference Evidence

The primary reference is Bevy gizmos:

- `dev/bevy/crates/bevy_gizmos/src/lib.rs` separates plugin/config group registration from immediate and retained drawing APIs.
- `dev/bevy/crates/bevy_gizmos/src/gizmos.rs` defines `GizmoBuffer`, clear contexts, and immediate drawing helpers such as line, ray, linestrip, rect, cube, AABB, and primitive rendering.
- `dev/bevy/crates/bevy_gizmos/src/config.rs` defines enabled state, line width, depth bias, and render layer policy as configuration rather than per-renderer ad hoc flags.
- `dev/bevy/crates/bevy_gizmos/src/retained.rs` keeps static or reused gizmos as retained assets/components instead of pushing every static debug line through an immediate path.

Fyrox is the editor/runtime split cross-check:

- `dev/Fyrox/editor/src/interaction/gizmo/move_gizmo.rs` keeps transform gizmo meshes and interaction as editor-side consumers of scene state.
- `dev/Fyrox/editor/src/scene_viewer/gizmo.rs` isolates viewport orientation gizmos from runtime scene authority.

Zircon intentionally follows Bevy for the neutral immediate/retained/config contract, while following the existing Zircon/Fyrox boundary rule that editor selection, handle state, and transform interaction stay editor-owned until the later cutover milestones.

## Ownership Boundary

The gizmo framework module owns reusable command and configuration contracts only:

- `GizmoCommand` stores neutral drawing requests.
- `GizmoBuffer` stores one immediate command list with a single config.
- `GizmoConfig` stores enabled state, line width, depth bias, render layer, color policy, and screen-scale policy.
- `GizmoAsset` stores a retained command list copied from a buffer.
- `RetainedGizmo` binds a retained asset to transform and config metadata.
- `GizmoOverlayExtractRequest` adapts immediate and retained commands into `SceneGizmoOverlayExtract`.

The module does not own runtime world entities, editor selection, undo, hover state, renderer resources, GPU buffers, or transform gizmo drag math. M4 will build transform gizmo interaction on top of M2 picking plus this M3 drawing contract. M9 will decide how much editor viewport overlay code can be deleted or downgraded to adapters.

## Data Model

The module is folder-backed so each file has one responsibility:

- `command.rs` defines `GizmoCommand` and `GizmoAxis`.
- `config.rs` defines config groups, line config, render-layer, color, and screen-scale policy.
- `buffer.rs` defines immediate-mode `GizmoBuffer` and primitive helper methods.
- `retained.rs` defines `GizmoAsset` and `RetainedGizmo`.
- `extract.rs` defines conversion into current render overlay DTOs.
- `mod.rs` only declares and re-exports the public surface.

The M3 command set covers line, ray, linestrip, rect, circle, sphere, cube, AABB, and axis helpers. Commands carry their own color, while `GizmoConfig.color_policy` can keep command color, override it, or multiply it during extraction. `GizmoConfig.enabled` is honored both by the buffer push path and by extraction, so disabled groups do not grow new immediate commands and stale enabled commands can be suppressed at output time.

`GizmoAsset::from_buffer` copies command data out of an immediate buffer. That means retained gizmos do not share mutable command storage with the buffer that created them. `RetainedGizmo` adds a transform and config to that retained command list so static debug geometry can be reused across frames without going through the immediate builder each time.

## Overlay Extraction

`extract_gizmo_overlay` accepts immediate buffers and retained gizmos in the order supplied by the request. It converts supported commands to the current `SceneGizmoOverlayExtract` line list. This is a compatibility adapter to the existing renderer-facing overlay packet, not the long-term renderer implementation.

Line-like commands convert directly:

- `Line` becomes one `OverlayLineSegment`.
- `Ray` becomes one line from `start` to `start + vector`.
- `LineStrip` becomes adjacent line segments.

Shape commands are approximated as line segments:

- `Rect` uses four local XY-plane edges.
- `Circle` uses a named local segment count.
- `Sphere` uses three great circles.
- `Cube` and `Aabb` use twelve box edges.
- `Axis` uses one axis-aligned segment.

The adapter preserves selected state, owner, and scene-gizmo kind because those already exist on `SceneGizmoOverlayExtract`. It records render-layer, depth-bias, and screen-scale policies in gizmo config, but the current render overlay DTO cannot yet carry all of those values. That limitation is documented here instead of hidden in renderer-specific behavior; a later graphics milestone can extend overlay DTOs or add a dedicated gizmo render packet.

## Intentional Divergence

Bevy exposes `Gizmos` as an ECS `SystemParam` with schedule-specific clear contexts and renderer-managed gizmo assets. Zircon M3 keeps the API as plain Rust DTOs and helper methods because `zircon_runtime::core::framework` is the shared contract spine, not the scheduler or renderer. Clear timing is explicit through `GizmoBuffer::clear`; future runtime scheduling can own main/fixed context storage without changing command semantics.

Bevy supports many primitive builders and renderer-specific mesh generation. Zircon M3 starts with the command set required by the plan and maps shapes to existing overlay line DTOs. This avoids expanding graphics/render pass scope before transform gizmos and editor cutover are ready.

## Test Coverage

`zircon_runtime/src/tests/gizmos/mod.rs` covers:

- immediate command ordering and clearing,
- disabled buffers not accumulating commands,
- config defaults for enabled state, line width, depth bias, render layer, color policy, and screen-scale policy,
- retained assets copying buffer commands without sharing mutable state,
- line/ray/linestrip extraction order,
- disabled extraction and color-policy override behavior,
- appending extracted gizmos into an existing `RenderOverlayExtract` packet,
- shape command extraction coverage for rect, circle, sphere, cube, AABB, and axis,
- mixed immediate and retained extraction order with retained transform application.

Milestone testing evidence:

- 2026-05-08: `rustfmt --edition 2021 --check "zircon_runtime/src/core/framework/mod.rs" "zircon_runtime/src/core/framework/gizmos/mod.rs" "zircon_runtime/src/core/framework/gizmos/command.rs" "zircon_runtime/src/core/framework/gizmos/config.rs" "zircon_runtime/src/core/framework/gizmos/buffer.rs" "zircon_runtime/src/core/framework/gizmos/retained.rs" "zircon_runtime/src/core/framework/gizmos/extract.rs" "zircon_runtime/src/tests/mod.rs" "zircon_runtime/src/tests/gizmos/mod.rs"` passed with no output.
- 2026-05-08: `cargo test -p zircon_runtime gizmo --locked --target-dir "E:\cargo-targets\zircon-runtime-gizmos-m3" --message-format short --color never` passed. The filtered run reported 13 matching `src/lib.rs` tests passed, 2 matching `m1_runtime_editor_boundary_contract` tests passed, 0 failures, and 1071 filtered out in the main test binary. Existing warning noise remained in unrelated render/UI/native-loader code and workspace `winit` feature configuration; no M3 gizmo warning remained after the append adapter test was added.
