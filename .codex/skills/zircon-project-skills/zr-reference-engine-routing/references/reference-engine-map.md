# Reference Engine Map

Use this map after reading the parent skill. Start from the touched `zircon_*` crates, then choose a primary reference and one or more secondary references.

## Quick Matrix

| Task shape | Primary reference | Secondary references | Why |
| --- | --- | --- | --- |
| Heavyweight engine systems, flagship rendering or simulation features, large editor pipelines, future Nanite/Lumen/Niagara/Montage-scale work | `dev/UnrealEngine` | `dev/Graphics`, `dev/Fyrox`, current `zircon_*` crates | Use Unreal for system shape and engine-scale constraints, then convert the landing zone to repository-compatible Rust abstractions. |
| Common engine infrastructure, asset/resource flows, reflection, serialization, ECS-ish scheduling, reusable support behavior | `dev/bevy`, `dev/godot` | `dev/Fyrox`, current `zircon_*` crates | Use Bevy and Godot for everyday infrastructure and data flow patterns that recur across many systems. |
| Program framework, editor/runtime split, Rust-native engine layering, subsystem boundaries that should feel close to current repository structure | `dev/Fyrox` | `dev/bevy`, `dev/godot`, current `zircon_*` crates | Use Fyrox when the task must land cleanly in a Rust engine architecture with a real editor/runtime separation. |
| Render pipeline, SRP-style passes, renderer features, shader graph, VFX graph, and Unity Graphics-derived rendering architecture | `dev/Graphics` | `dev/UnrealEngine`, `dev/bevy`, `zircon_runtime/src/graphics` | Use Graphics for pipeline structure and rendering feature placement, then pair it with an engine-scale or Rust-native reference before finalizing shared abstractions. |
| Minimal startup path, smallest engine bootstrap, baseline runtime entry, first executable host flow | `dev/Piccolo` | `dev/Fyrox`, `dev/bevy`, current `zircon_*` crates | Use Piccolo to keep the initial path minimal, then switch to Fyrox or Bevy once the bootstrap grows into a framework concern. |
| Editor shell, declarative UI, panel layout, data-binding surfaces, document-oriented tooling UI | `dev/slint` | `dev/Fyrox`, `dev/godot`, `dev/UnrealEngine` | Use Slint for toolkit-specific UI decisions, but keep engine/runtime/editor constraints honest with a second reference. |
| Timeline editing, sequencing, animation authoring, state-machine-adjacent tooling, studio or workbench UX | `dev/theatre` | `dev/UnrealEngine`, `dev/slint`, `dev/Fyrox` | Use Theatre for time-based authoring workflows, then stabilize the model with an engine-scale or Rust-native editor reference. |

## Practical Routing Rules

- Start with repository truth:
  - Read the touched `zircon_*` crates first.
  - Record which crate or subsystem owns the problem before pulling in external precedent.
- Pick a primary reference by the dominant constraint:
  - System scale or flagship feature family: `UnrealEngine`
  - Common infrastructure and recurring support code: `bevy` plus `godot`
  - Rust-native framework and editor/runtime host shape: `Fyrox`
  - Render pipeline architecture: `Graphics`
  - Minimal bootstrap or smallest entry path: `Piccolo`
  - Editor UI and document-style tooling: `slint`
  - Timeline and animation authoring: `theatre`
- Add secondary references to close missing constraints:
  - Add `Fyrox`, `bevy`, or current `zircon_*` crates when the primary reference is not Rust-native.
  - Add `UnrealEngine` when the feature may grow into a heavyweight, long-lived engine system.
  - Add `Graphics` whenever rendering pipeline boundaries or authoring tooling interact with rendering.
  - Add `slint` or `theatre` when user-facing tooling is the hard part, not just runtime semantics.

## Recommended Pairings

- `UnrealEngine` + `Fyrox`:
  - Use when the target system is heavyweight but the landing zone must fit a Rust-native engine/editor split.
- `UnrealEngine` + `Graphics`:
  - Use when a rendering or VFX system needs both engine-scale precedent and concrete SRP-style pipeline architecture.
- `bevy` + `godot`:
  - Use when the task is common infrastructure and you need both a Rust-native pattern and a mature engine cross-check.
- `Graphics` + `bevy`:
  - Use when render pipeline ideas must land in Rust-friendly crate boundaries and render scheduling.
- `slint` + `Fyrox`:
  - Use when editor shell UI must remain coherent with the runtime or editor host structure.
- `theatre` + `UnrealEngine`:
  - Use when animation tooling needs both modern sequencing UX and heavyweight engine authoring expectations.
- `Piccolo` + `Fyrox`:
  - Use when a minimal bootstrap path is growing into a longer-lived framework decision.

## Directory Topology Cues

When the task creates a new module, folder, or crate layout, inspect these top-level shapes before deciding local structure:

- `dev/UnrealEngine/Engine/Source`: `Runtime/`, `Editor/`, `Programs/`, `Developer/`, `ThirdParty/`
  - Use this tree to check heavyweight subsystem ownership, editor/runtime separation, and whether a concern deserves its own long-lived subsystem folder instead of one local module file.
- `dev/godot`: `core/`, `scene/`, `servers/`, `editor/`, `platform/`, `modules/`
  - Use this tree to separate engine foundations, scene-facing APIs, server/runtime services, editor-only behavior, platform details, and optional modules.
- `dev/bevy`: `crates/`, `src/`, `examples/`, `tests/`, `tools/`
  - Use this tree to bias toward narrow crate boundaries, support tooling outside production modules, and explicit testing or example surfaces instead of bloated crate roots.
- `dev/Fyrox`: `editor/` plus many narrow `fyrox-*` crates
  - Use this tree to keep Rust-native subsystem boundaries explicit, split reusable foundations into dedicated crates, and preserve a real editor/runtime divide.
- `dev/Graphics`: `Packages/`, `Templates/`, `Tests/`, `Tools/`
  - Use this tree as the Unity Graphics-style reference for package-oriented rendering architecture, support tools, templates, and validation surfaces that should not be mixed into runtime implementation folders.

Do not copy these trees mechanically. Use them to decide whether `zirconEngine` needs a crate seam, subsystem folder, editor-only branch, tooling folder, or a minimal runtime path before adding code.

## Module Creation Checklist

- Identify the owning `zircon_*` crate first.
- Choose one primary reference for subsystem shape and one secondary reference for missing constraints.
- Compare directory layout first, not just type names or APIs.
- Prefer boundaries that still make sense after major subsystem growth.
- Translate the boundary into minimal Rust wiring files plus narrow declaration and behavior files.

## Avoid These Shortcuts

- Do not use `UnrealEngine` alone to justify Rust crate boundaries or API ergonomics.
- Do not use `bevy` alone for editor-heavy authoring workflows when UI, panels, or sequencing dominate the problem.
- Do not use `slint` alone for runtime semantics, asset lifetime, or module/plugin contracts.
- Do not use `theatre` alone for animation runtime ownership or engine integration rules.
- Do not use `Piccolo` alone once the task crosses into editor, rendering, asset, or multi-module system design.
- Do not use `Graphics` alone for non-rendering engine architecture.

## Future Heavy-Engine Direction

- Bias toward `UnrealEngine` for future heavyweight feature families and recognizable engine systems.
- Bias toward `Graphics` for render pipeline implementation patterns, especially SRP-style decomposition and rendering feature placement.
- Keep `bevy`, `godot`, and `Fyrox` in the loop so heavy-engine ideas still land as maintainable Rust crate boundaries instead of direct source transplantation.

## Handoff

- After selecting the references, move to the execution skill that matches the real work:
  - `zr-language-feature-design` for evidence-driven feature semantics and test derivation
  - `zr-architecture-first-engineering` for subsystem architecture and ownership
  - `layered-milestone-development` for ordered milestone execution
  - `zircon-dev` for code edits and validation
