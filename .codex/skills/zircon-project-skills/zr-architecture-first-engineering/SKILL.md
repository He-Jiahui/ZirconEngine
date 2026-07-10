---
name: zr-architecture-first-engineering
description: Use when designing, refactoring, or implementing `zirconEngine` systems, subsystems, or cross-crate features that must define architecture before code, stay aligned with `.codex/plans/全系统重构方案.md` plus `.codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md`, converge to the fixed `zircon_app`/`zircon_runtime`/`zircon_editor` package shape plus the runtime-internal `core/{runtime,framework,manager,math,resource}` spine, and reject shallow one-off implementations.
---

# Zr Architecture-First Engineering

## Overview

Keep `zirconEngine` work inside an explicit engine architecture instead of solving requests with direct concrete implementations. Before code, decide the owning boundary, lifecycle path, facade surface, data model, and validation shape. If the easiest path is a one-off branch or a thin wrapper around one feature, deepen the architecture first.

## Progressive Disclosure Index

- Start with `references/system-architecture-baseline.md`.
- Read `.codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md` whenever the task touches runtime absorption, editor or scene ownership, crate naming, or cross-crate convergence.
- Read `.codex/plans/全系统重构方案.md` as the wider authority roadmap for the current engine target.
- If the work needs mainstream engine precedent, also apply `../zr-language-feature-design/SKILL.md`.
- If the work spans multiple layers or milestones, also apply `../layered-milestone-development/SKILL.md`.
- If the work edits or validates repository code, also apply `../../zircon-dev/SKILL.md`.

## Non-Negotiable Rules

- Do not implement a new subsystem or feature until its target abstraction is explicit: owner crate, module boundary, descriptors, facade surface, data model, lifecycle mode, config surface, event hooks, ECS authority, schedule stage, or plugin contract as applicable.
- Treat `.codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md` as the binding convergence rule for top-level runtime/editor architecture unless the user explicitly replaces it.
- Keep the public architecture on the fixed three-package root:
  - `zircon_app`: process entry, profile selection, and main-loop host
  - `zircon_runtime`: runtime absorption layer and home of the internal `core/{runtime,framework,manager,math,resource}` spine
  - `zircon_editor`: editor host and authoring logic
- Inside `zircon_runtime::core`, keep the internal role split explicit:
  - `runtime`: lifecycle, dependency, registration, and scheduling kernel
  - `manager`: service-name, resolver, handle, activation, and access-entry layer
  - `framework`: pure framework contracts and shared DTOs only
  - `math` / `resource`: canonical shared foundations
- Do not bypass the convergence spine with direct upper-layer construction or concrete cross-crate coupling. `zircon_app` hosts, `zircon_runtime::core::runtime` schedules, `zircon_runtime::core::manager` exposes access, `zircon_runtime::core::framework` defines contracts, and `zircon_runtime` plus `zircon_editor` consume those contracts.
- Reserve `server` naming for real network or service-host semantics only. Treat non-network `*server*` crate names, traits, or handles as migration debt to remove instead of a valid design direction.
- Do not keep compatibility shims, alias crates, legacy-path re-exports, or temporary bridge modules once a migration step is complete. Cut directly to the converged structure at each stage.
- If an in-flight task is still operating inside a non-compliant crate, folder, or boundary that the convergence plan already says should move, pause feature work and migrate that work into the compliant destination first. Resume the original task only after the ownership path is corrected.
- When an implementation feels "simple", verify that the simplicity comes from an already-strong abstraction. If it comes from skipping one, redesign first.
- Design new boundaries for future scale, not only today's feature count. If the crate or folder shape would need a structural rewrite after several sibling systems land, deepen it now.
- Do not add one-off branches, type-name checks, or ad hoc shared-service lookups in foundations when the real issue is a missing generalized capability.
- Prefer reusable framework slots over direct leaf implementations: descriptors, drivers, managers, stable handles, systems, manifests, config objects, command surfaces, and extract pipelines.
- Upper layers may consume facades, handles, descriptors, and ECS queries. They should not own lower-layer runtime objects directly.
- If a cross-workspace refactor activates one crate as a real module, give sibling `zircon_*` crates matching lifecycle skeletons or stub integration instead of leaving placeholder crates behind.
- Direct implementation is allowed only for leaf details fully contained inside an approved abstraction and invisible to cross-crate architecture.
- Record any deliberate divergence from mainstream engines or from the roadmap, and justify it before treating the design as settled.

## Workflow

1. Anchor the task in the current engine shape.
- Read `.codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md`, `.codex/plans/全系统重构方案.md`, and the affected crates.
- State which fixed role owns the work: app host, runtime-internal core kernel, runtime-internal manager access layer, runtime-internal framework contract layer, runtime absorption layer, editor authoring host, supporting runtime module, or leaf subsystem.

2. Extract the missing framework before behavior.
- List the abstractions the feature needs before code exists.
- For every new capability, decide whether it belongs in a descriptor, trait, handle, config type, lifecycle state, event channel, ECS component, system stage, command surface, or plugin protocol.
- If none are needed, explicitly prove the work is a leaf detail inside an existing abstraction.

3. Check mainstream engine alignment before committing to the design.
- Compare the subsystem shape against mature engine patterns and the current reference engines.
- Compare directory and module topology too, not only API surfaces. Favor the folder, crate, and editor/runtime splits that can absorb long-term growth without reorganization.
- Reuse terminology and boundaries that keep `zirconEngine` recognizable as a full engine architecture, not a demo-specific code path.
- If the planned abstraction is thinner or more special-cased than mainstream precedent, deepen it first.

4. Run the architecture depth test.
- Can another feature reuse the same boundary without editing shared code again?
- Does the upper layer depend on a facade or contract instead of a concrete implementation?
- Does the design preserve lifecycle ordering, hot reload expectations, ECS ownership, and extract/update separation where relevant?
- Does the design keep runtime-world authority in `zircon_runtime` and authoring-state authority in `zircon_editor` instead of mixing them?
- Does the naming stay inside the converged crate vocabulary instead of reviving non-network `server` terminology?
- Would the path and boundary still make sense after 10x more modules, commands, or authoring surfaces appear?
- If any answer is no, redesign before implementation.

5. Implement from framework inward.
- Add or revise the boundary objects first.
- Wire registration, descriptors, lifecycle, config, no-op or stub paths, and validation hooks before feature-specific behavior.
- Only then land the concrete behavior as a normal consumer of the framework.

6. Validate architecture, not just behavior.
- Verify that crates register and shut down through the intended runtime path.
- Verify that editor, runtime, and plugin consumers reach the feature through the planned facade or contract.
- Add tests for lifecycle, dependency ordering, extraction, serialization, state migration, and failure boundaries when those layers are touched.

## Architecture Red Flags

- A new feature starts with "just call this concrete type from editor, app, or runtime."
- Shared code needs a special branch for one feature name, one type name, or one call site.
- A crate gains user-visible behavior without module descriptors or lifecycle entry.
- A refactor keeps both the old path and the new path alive with re-exports or alias crates for now.
- A new non-network crate, trait, or handle adopts `server` naming.
- Ongoing feature work keeps accumulating inside a directory that the convergence plan already marks as the wrong home.
- Editor state becomes the owner of runtime world data instead of consuming runtime world or manager surfaces.
- A plugin feature requires direct Rust object sharing instead of stable host handles and protocol state.
- The change works only because untouched sibling crates stay as placeholders.

## Quick Use

Before coding, write a short architecture note in the task:
- target roadmap section
- owner fixed role and crate boundary
- required descriptors, facades, or contracts
- mainstream engine precedents checked
- reason the abstraction depth is sufficient
- validation layers that must pass
