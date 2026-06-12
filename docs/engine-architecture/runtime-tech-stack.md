---
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime_interface/Cargo.toml
  - zircon_editor/Cargo.toml
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/tests/text_shaper.rs
  - zircon_runtime/src/plugin/export_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/mod.rs
  - zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs
implementation_files:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime_interface/Cargo.toml
  - zircon_editor/Cargo.toml
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/plugin/export_profile.rs
  - zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs
plan_sources:
  - user: 2026-06-12 implement runtime architecture from docs/plans/zircon_runtime/runtime
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - dev/bevy/crates/bevy_app/src/main_schedule.rs
  - dev/bevy/crates/bevy_text/src/lib.rs
  - dev/Fyrox/fyrox-impl/Cargo.toml
tests:
  - zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs
  - cargo test -p zircon_runtime --lib tech_stack --locked -- --nocapture
doc_type: module-detail
---

# Runtime Tech Stack

## Purpose

This document is the runtime-side dependency authority for `zirconEngine`. It separates dependencies that are part of the runtime product surface from editor-only candidates, plugin-owned stacks, and future backlog decisions. Manifest changes that alter these decisions must update this document and the matching source guard in `zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs`.

## Dependency Matrix

| Dependency / area | Current version or state | Owner crate | Feature gate | Upgrade or replacement gate |
|---|---:|---|---|---|
| `winit` | `0.31.0-beta.2` | `zircon_runtime` optional platform window path; `zircon_editor` retained host path | runtime: `platform-winit`; editor: direct dependency | Upgrade only in a dedicated milestone after `0.31` final is available and `ApplicationHandler` API impact is reviewed. |
| `wgpu` / `naga` | `29.0.1` / `29.0.1` | `zircon_runtime::graphics` | default runtime client/editor-host profile through render features | Renderer plan owns upgrades; `zircon_runtime_interface` must stay free of both dependencies. |
| `taffy` | `0.10` | `zircon_runtime::ui::layout` | runtime UI | Replace only behind the runtime layout bridge after editor UI plan sign-off. |
| `glam` | `0.32.1` with `serde` | workspace + interface/runtime/editor consumers | none | Precision and ABI seam decisions stay under runtime foundation docs. |
| `glyphon` | `0.11.0` | runtime render/text submission | runtime UI/render | Current layout backend is still heuristic; glyphon is a render-side/native-text intent, not the active layout shaper. |
| `fontsdf` | `0.5.3` | runtime text/raster policy | runtime UI/render | Stays local to runtime text/raster policy until SDF atlas implementation replaces the heuristic layout path. |
| `image` | `0.25.10` | asset import and texture/image processing | none | Shared importer policy owns format expansion. |
| `gltf` / `tobj` | `1.4.1` / `4.0.3` | runtime asset import and mesh ingest | none | Model-importer plugin work may move behavior outward, but runtime still owns current built-in importer paths. |
| `notify` | `9.0.0-rc.3` | runtime/editor asset watch paths | none | Upgrade only in a dedicated milestone after `9.0` final is available and watcher event compatibility is checked. |
| `rayon` | `1.11.0` | runtime scheduling/asset parallelism | none | Replace only with an execution-policy milestone that covers ECS scheduling and asset worker behavior together. |
| `crossbeam-channel` / `crossbeam-utils` | `0.5.15` / `0.8.21` | runtime channels and worker support | none | Any replacement must preserve current runtime channel facade semantics. |
| `serde`, `serde_json`, `toml`, `ron`, `bincode` | workspace or crate-local pinned versions | manifests, project data, artifact cache, debug/config IO | none | Serialization format changes need explicit migration plans. |
| `libloading` | `0.9.0` | runtime cdylib loading and native dynamic plugin support | none | Dynamic ABI changes are governed by runtime interface convergence and plugin ABI plans. |
| `zstd` | `0.13.3` | runtime/export compression support | none | Archive container choice is still a decision item; do not imply zip/tar support from this dependency. |
| `accesskit` | `0.22.0` optional | runtime accessibility | `accessibility-accesskit` | Upgrade with accessibility DTO compatibility checks. |
| gamepad input | app/runtime input stack | app/runtime input | `input-gamepad`, `gamepad-gilrs` | Browser gamepad remains a separate target path. |
| `zr_vm_rust_binding` / `zr_vm_rust_binding_sys` | external path dependency at `../../zr_vm/...` | runtime script backend | `zr-vm-real-backend` | Current decision is to keep the external checkout. Any move to submodule/vendor/published crate must pair with the empty-argument marshalling fix in the binding version. |

## Corrected Non-Dependencies

The runtime plan previously mentioned several libraries that are not present in the current workspace manifests. These are not runtime dependencies:

| Name | Current decision | Owner or follow-up |
|---|---|---|
| `cosmic-text` | Not introduced. The current text layout backend is `UiHeuristicTextShaper`; glyphon is only the native render/backend intent. | Future complex text demand may introduce cosmic-text through `UiTextShaper`, not by bypassing that trait. |
| `kira` | Not introduced. Sound runtime uses the existing plugin-owned stack, currently based on `cpal` and custom mixer/DSP/HRTF/occlusion paths. | Sound plugin plan owns audio backend decisions. |
| `zip` / `tar` | Not introduced. Export packaging currently has strategy contracts but no archive-container dependency. Runtime 01 M3.2 selects ZIP as the future archive container, but no manifest dependency is allowed until archive materialization lands. | Export build-plan owner must add the dependency and guard change in the same implementation slice. |
| `fontdue` | Not introduced in runtime. It remains a temporary `zircon_editor` retained-host text fallback. | Tracked in [Runtime Editor-Only Dependency Backlog](../editor-and-tooling/runtime-editor-only-dependency-backlog.md); remove or replace under the editor UI text plan once retained-host text rendering consumes runtime UI text/glyphon/SDF. |
| `rfd` | Not introduced in runtime. | Editor-only file-dialog candidate tracked in [Runtime Editor-Only Dependency Backlog](../editor-and-tooling/runtime-editor-only-dependency-backlog.md); do not add to runtime. |
| `arboard` | Not introduced in runtime. | Editor-only clipboard candidate tracked in [Runtime Editor-Only Dependency Backlog](../editor-and-tooling/runtime-editor-only-dependency-backlog.md); do not add to runtime. |

## Prerelease Version Governance

`winit 0.31.0-beta.2` and `notify 9.0.0-rc.3` remain intentionally pinned. They are allowed because they are already integrated and because replacing them without a targeted migration would touch platform/application lifecycle code and watcher behavior across runtime, app, and editor.

Upgrade gates:

1. `winit`: wait for `0.31` final, then verify `ApplicationHandler` and platform feature behavior in a dedicated milestone before changing the workspace dependency.
2. `notify`: wait for `9.0` final, then rerun asset watcher and UI hot-reload watch invalidation coverage before changing the workspace dependency.
3. Any silent manifest bump without this document and `tech_stack_dependency_guard.rs` changing together is invalid.

## External ZrVM Path Dependency

The current decision is option A from the runtime 01 plan: keep `../../zr_vm` as an external checkout and gate it behind `zr-vm-real-backend`. This keeps the default runtime build independent from a local ZrVM checkout while preserving the real backend for explicit validation.

The path dependency is not only a clone-layout issue. The runtime real-backend contract depends on a paired binding version that represents empty export argument lists as a valid non-null pointer with length `0`. Moving the dependency to a submodule, vendored crate, or published crate must include that binding fix as a version gate.

Required local layout for the real backend:

```text
E:/Git/ZirconEngine
E:/Git/zr_vm
```

## Export Archive Decision

The current `ExportPackagingStrategy` enum is not an archive-container enum. It describes how project/plugin code is materialized: `SourceTemplate`, `LibraryEmbed`, and `NativeDynamic`. The current export path remains directory-first and does not produce a single archive file.

Runtime 01 M3.2 selects ZIP as the future desktop/editor archive container. The reasons are cross-platform user tooling, Windows Explorer/macOS Finder/Linux desktop compatibility, existing editor-export expectations around a single distributable file, and a lower support burden than a custom container. `tar + zstd` remains a possible CI/server artifact format later, but it is not the primary runtime export container. A custom container is rejected for V1 because it would require custom inspection, extraction, and failure-recovery tooling before the runtime package format itself is stable.

This decision does not change manifests today. Neither `zip` nor `tar` may enter workspace manifests until export materialization grows an explicit archive step, deterministic path/timestamp normalization, path traversal checks, and validation coverage under the [Runtime/Editor Pluginized Export](./runtime-editor-pluginized-export.md) / export build-plan owner.

## Text Stack Boundary

Runtime text currently has three separate responsibilities:

| Layer | Current owner | Current state |
|---|---|---|
| Layout and measurement | `zircon_runtime::ui::text::UiTextShaper` | Active backend is heuristic; `heuristic_text_shaper_matches_public_layout_entrypoint` and `text_shaper_stack_uses_current_heuristic_backend_until_font_backends_land` lock that behavior. |
| Font/raster policy | `zircon_runtime::ui::text` | Font registry and raster policy exist; SDF/native layout backends are not connected yet. |
| GPU/native submission | runtime graphics/UI render paths with `glyphon` | Render-side dependency exists; layout backend remains heuristic until a future text milestone swaps the `UiTextShaper` implementation. |

`cosmic-text`, Parley, Swash, or HarfBuzz may only enter through a replacement implementation of `UiTextShaper`. They must not duplicate public text layout entry points or bypass the existing `UiResolvedTextLayout` contract.

## Interface And Editor Dependency Boundary

`zircon_runtime_interface` is a DTO/ABI crate. Its manifest must remain free of `wgpu` and `winit`. `zircon_editor` is allowed to keep a direct `winit` dependency for the retained host and `softbuffer` self-drawn shell, but it must remain free of `wgpu` unless the editor UI plan explicitly changes renderer ownership.

The editor-only candidates `fontdue`, `winit`, `softbuffer`, future `rfd`, and future `arboard` are not runtime dependency claims. `fontdue`, `rfd`, and `arboard` are tracked in the [Runtime Editor-Only Dependency Backlog](../editor-and-tooling/runtime-editor-only-dependency-backlog.md); all of these are editor-host concerns and should be moved or removed only under the editor plan.
