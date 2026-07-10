# Reference Engine Roots

Use these repository-local trees as the primary evidence base for `zirconEngine` scripting, module, and runtime feature design. Search source and tests together whenever possible.

## ZirconEngine

- Source roots:
  - `zircon_app`
  - `zircon_runtime`
  - `zircon_runtime/src/core/runtime`
  - `zircon_runtime/src/core/manager`
  - `zircon_runtime/src/core/framework`
  - `zircon_runtime/src/engine_module`
  - `zircon_editor`
- Test roots:
  - inline `#[cfg(test)]` blocks in the touched `zircon_*` crates
  - `.github/workflows/ci.yml` for the canonical build/test shape
- Use first for: current runtime contracts, module/plugin descriptors, service-name and resolver handles, framework DTOs, runtime absorption seams, editor/runtime integration, and actual repository naming.
- Good search hints: `ModuleDescriptor`, `PluginDescriptor`, `ManagerResolver`, `RenderFramework`, `EntryProfile`, `builtin_runtime_modules`, `hot_reload`, `register`, `with_driver`, `with_manager`

## Unreal Engine

- Source roots:
  - `dev/UnrealEngine/Engine/Source/Runtime`
  - `dev/UnrealEngine/Engine/Source/Editor`
  - `dev/UnrealEngine/Engine/Source/Developer`
  - `dev/UnrealEngine/Engine/Source/Programs`
- Test roots:
  - nearby automation, spec, and validation code under the touched `Engine/Source` modules
  - `dev/UnrealEngine/Samples`
  - `dev/UnrealEngine/Templates`
- Use first for: heavyweight engine systems, engine/editor pipelines, reflection-driven frameworks, asset streaming, and future Nanite, Lumen, Niagara, Montage, or other engine-scale feature families.
- Good search hints: `Subsystem`, `Module`, `Asset`, `Render`, `Animation`, `Niagara`, `Timeline`, `Montage`, `Automation`, `Spec`

## bevy

- Source roots:
  - `dev/bevy/crates`
  - `dev/bevy/src`
- Test roots:
  - `dev/bevy/tests`
  - `dev/bevy/tests-integration`
  - `dev/bevy/examples`
- Use first for: Rust-native engine infrastructure, ECS or app wiring, schedules, reflection, asset management, serialization-friendly data flow, and common engine utilities.
- Good search hints: `Plugin`, `App`, `Schedule`, `Resource`, `SystemSet`, `Reflect`, `Asset`, `States`

## Fyrox

- Source roots:
  - `dev/Fyrox/fyrox`
  - `dev/Fyrox/fyrox-graphics`
  - `dev/Fyrox/fyrox-resource`
  - `dev/Fyrox/fyrox-scripts`
- Test roots:
  - inline tests in the touched Fyrox crates
  - `dev/Fyrox/editor`
- Use first for: Rust-native engine architecture, editor/runtime separation, resource systems, scripting hooks, and subsystem-oriented crate boundaries.
- Good search hints: `script`, `resource`, `plugin`, `scene`, `handle`, `register`, `editor`

## Godot

- Source roots:
  - `dev/godot/core`
  - `dev/godot/scene`
  - `dev/godot/modules`
  - `dev/godot/editor`
- Test roots:
  - `dev/godot/tests`
  - `dev/godot/modules/gdscript/tests`
- Use first for: editor/runtime contracts, scene graph behavior, scripting/runtime interplay, serialization, and engine-scale integration patterns.
- Good search hints: `script`, `module`, `resource`, `reload`, `SceneTree`, `Variant`, `Editor`, `test_`

## Graphics

- Source roots:
  - `dev/Graphics/Packages/com.unity.render-pipelines.core`
  - `dev/Graphics/Packages/com.unity.render-pipelines.universal`
  - `dev/Graphics/Packages/com.unity.render-pipelines.high-definition`
  - `dev/Graphics/Packages/com.unity.shadergraph`
  - `dev/Graphics/Packages/com.unity.visualeffectgraph`
  - `dev/Graphics/com.unity.postprocessing`
- Test roots:
  - `dev/Graphics/Tests`
  - `dev/Graphics/TestProjects`
  - package-local tests near the touched render pipeline code
- Use first for: SRP pipeline boundaries, render passes, renderer features, shader graph or VFX graph integration, and Unity Graphics-derived rendering patterns.
- Good search hints: `RenderPipeline`, `RendererFeature`, `RenderGraph`, `Pass`, `Volume`, `ShaderGraph`, `VFX`

## Piccolo

- Source roots:
  - `dev/Piccolo/engine/source`
- Test roots:
  - nearby validation code and examples under `dev/Piccolo/engine/source`
- Use first for: minimal runtime or framework layering, baseline engine entry, hot-reload-adjacent behavior, asset and module coordination, and smallest-possible engine architecture tradeoffs.
- Good search hints: `module`, `runtime`, `reload`, `asset`, `component`, `reflect`, `serialize`

## slint

- Source roots:
  - `dev/slint/api`
  - `dev/slint/internal`
  - `dev/slint/ui-libraries`
- Test roots:
  - `dev/slint/tests`
  - `dev/slint/examples`
  - `dev/slint/demos`
- Use first for: toolkit-specific UI architecture, declarative UI and runtime bridge questions, editor shell layouts, document-style panels, and UI data binding boundaries.
- Good search hints: `component`, `callback`, `model`, `property`, `interpreter`, `window`, `binding`

## theatre

- Source roots:
  - `dev/theatre/packages/core`
  - `dev/theatre/packages/studio`
  - `dev/theatre/packages/react`
  - `dev/theatre/packages/utils`
- Test roots:
  - `dev/theatre/compat-tests`
  - `dev/theatre/examples`
  - nearby tests under `dev/theatre/packages`
- Use first for: timeline editing, animation state or sequence authoring, studio or workbench patterns, and sequencing UX.
- Good search hints: `timeline`, `sequence`, `sheet`, `studio`, `state`, `keyframe`, `scrub`, `track`

## Search Strategy

- Start with the nearest execution model, then add at least one contrasting engine or subsystem to avoid cargo-culting a single implementation.
- Start from the touched `zircon_*` crates before choosing external references.
- Search tests and implementation together. If you only found source code, keep looking for regression tests.
- Prefer small, directly relevant files over massive top-level sweeps.
- For semantics that combine editor/runtime, module/runtime, or scripting/runtime behavior, pair a reference engine with the current `zircon_*` crates.
- When a feature touches scripting runtime or host capability boundaries, include at least one runtime-focused tree such as `godot`, `Piccolo`, or `bevy` even if the API inspiration comes from elsewhere.
- When a feature touches render pipelines, pair `Graphics` with either `UnrealEngine`, `bevy`, or the touched `zircon_runtime/src/graphics` code before choosing abstractions.
- When a feature touches editor UI or authoring workflows, pair `slint` or `theatre` with `Fyrox`, `godot`, or `UnrealEngine` so the UI flow does not drift away from engine/runtime constraints.
- When a feature is engine-scale or likely to become a flagship system, consult `UnrealEngine` early even if implementation details will land closer to `Fyrox`, `bevy`, or the current repository.
