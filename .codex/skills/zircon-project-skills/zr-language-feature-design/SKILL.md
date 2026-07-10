---
name: zr-language-feature-design
description: Design or review `zirconEngine` scripting, module/plugin, runtime, editor/runtime boundary, or performance-sensitive subsystem features by grounding every decision in the current `zircon_*` crates plus the reference engine trees under `dev/` (`UnrealEngine`, `bevy`, `Fyrox`, `godot`, `Graphics`, `Piccolo`, `slint`, and `theatre`). Use when a task needs evidence for which upstream engine should lead, how to divide responsibility across those references, and how to derive coherent validation and intentional divergence.
---

# Zr Language Feature Design

## Overview

Use this skill when `zirconEngine` needs a new scripting, module, runtime, editor/runtime, or closely related subsystem capability and the design must stay inside the combined capability envelope of the current workspace plus the bundled reference engines under `dev/`. Collect upstream evidence first, choose a dominant reference based on subsystem fit, align the change to the current `zirconEngine` plans, and derive complete tests before calling the feature designed or implemented.

## Progressive Disclosure Index

- If the work changes top-level crate ownership, public architecture naming, or runtime/editor boundaries, also apply `../zr-architecture-first-engineering/SKILL.md`.
- Start with `../zr-reference-engine-routing/SKILL.md` if the task spans multiple subsystems or you have not yet chosen the dominant reference engines.
- Start with `references/reference-language-roots.md`.
- Read `references/feature-evidence-and-test-checklist.md` before proposing semantics or tests.
- If the work affects milestone order or acceptance gates, also apply `../layered-milestone-development/SKILL.md`.
- If the work starts from a failing high-level scenario, also apply `../support-first-regression-testing/SKILL.md`.
- If you will edit or validate repository code, also apply `../../zircon-dev/SKILL.md`.
- Use `scripts/search_feature_evidence.py` to search the reference trees before relying on memory.

## Reference Routing Priorities

- Start from the touched `zircon_*` crates first. Reference trees justify or pressure-test the design; they do not replace repository architecture.
- Choose one dominant reference that matches the subsystem's hardest constraint, then add one or more stabilizing references that cover runtime, editor, tooling, or performance gaps.
- Use `dev/UnrealEngine` first for heavyweight system design and future engine-scale features such as Nanite-, Lumen-, Niagara-, Montage-, or large editor-pipeline style work.
- Use `dev/bevy` and `dev/godot` first for common engine infrastructure, Rust-friendly data flow, reflection, asset/resource management, serialization, and recurring day-to-day engine utilities.
- Use `dev/Fyrox` first for program framework, editor/runtime separation, Rust-native engine layering, and subsystem boundaries that must fit the current repository structure.
- Use `dev/Graphics` first for render pipeline structure, SRP-style separation, shader graph or VFX-adjacent pipeline decisions, and other Unity Graphics-derived rendering patterns.
- Use `dev/Piccolo` first for minimal startup flow, baseline engine entry, and smallest-possible end-to-end bootstraps.
- Use `dev/slint` first for editor UI architecture, document-oriented UI patterns, and toolkit-specific integration questions.
- Use `dev/theatre` first for animation authoring, state machine or timeline UX, and editor sequencing workflows.
- Pair `UnrealEngine` or `Graphics` with a Rust-native reference such as `Fyrox`, `bevy`, or the current `zircon_*` crates before landing shared repository abstractions.
- Pair `slint` or `theatre` with `Fyrox`, `godot`, or `UnrealEngine` when the authoring workflow must survive engine/runtime/editor integration.

## Non-Negotiable Rules

- Treat the reference engine trees under `dev/` plus the relevant `zircon_*` crates as the primary source of truth for feature precedent.
- Do not invent `zirconEngine` semantics first and search for justification later.
- For every feature, find comparable behavior in at least two reference engines or engine subsystems. Use three or more when semantics, authoring flow, memory ownership, or performance tradeoffs are non-trivial.
- Do not unblock a new feature by adding one-off branches in shared foundations for specific type names, object names, syntax spellings, or isolated call sites.
- Treat stringly-typed dispatch, type-name checks, and feature-specific condition chains in shared execution paths as a design failure unless they are already the intended abstraction.
- When a feature seems to require a special case, stop and identify which foundational capability is missing: module contract, plugin protocol, host capability boundary, runtime contract, serialization rule, ownership rule, or diagnostic model.
- Do not introduce or normalize non-network `server` naming while designing new architecture. If a reference engine uses that term, translate it into the converged `zircon_runtime::core::manager`, `zircon_runtime::core::framework`, or `zircon_runtime` vocabulary before landing the design.
- Prefer repairing or generalizing the foundational capability so the new feature becomes a normal consumer of it.
- Design upward from a coherent subsystem model, not downward from one failing testcase or one temporary workaround.
- Separate evidence into architecture, API surface, compile-time rules, runtime behavior, editor or authoring flow, error model, serialization or persistence behavior, and performance implications.
- Record intentional divergence explicitly. Diverge only to improve coherence with existing `zirconEngine` plans, implementation cost, or runtime performance.
- Derive tests from upstream tests or upstream behavior, not from one hand-written happy path.
- Boundary tests are mandatory. Stress or extreme tests are mandatory for hot reload, repeated module load, runtime handle management, serialization, deep nesting, repeated transitions, or resource limits.
- Do not mark a feature complete unless the affected workspace layers, runtime behaviors, and project-facing surfaces are addressed at the layers the feature touches.

## Workflow

1. Anchor the feature in the current `zirconEngine` roadmap.
- Read the milestone plan under `.codex/plans/`.
- Decide which crate or subsystem owns the feature now and which follow-on work belongs later.

2. Build a cross-engine evidence matrix.
- Search the relevant reference engine trees and current workspace crates before designing.
- Prefer source files and tests over vague recollection.
- Capture the exact files and the specific behavior they justify.
- Look for both the feature and its failure modes.
- Pick one dominant reference and at least one stabilizing reference. If the dominant reference is `UnrealEngine`, `Graphics`, `slint`, or `theatre`, add a Rust-native or repository-local cross-check before finalizing the abstraction.

3. Challenge the current foundations before designing the feature surface.
- Ask whether `zircon_runtime::core::{runtime,manager,framework}`, `zircon_runtime::engine_module`, `zircon_runtime`, `zircon_editor`, the runtime script/plugin path, or the target subsystem boundary is too weak or too narrow.
- If the only obvious implementation path is a feature-specific branch in shared code, treat that as a signal to redesign the lower layer first.
- Prefer capability-driven hooks and general contracts over checking concrete type names or special spelling cases.
- Reject temporary fixes in shared foundations unless they are explicitly marked as a short-lived debugging probe and removed before completion.

4. Choose the `zirconEngine` design from the shared core.
- Extract the common semantic core across the reference projects.
- Identify which reference project offers the closest execution model for the target subsystem.
- Identify which reference engine provides the best authoring, rendering, or framework precedent when the execution model alone is not enough.
- Keep traits, descriptors, serialization, diagnostics, host capabilities, and runtime objects consistent with the current repository shape.
- If the reference projects disagree, prefer the model that best matches `zirconEngine`'s existing architecture and milestone priorities.

5. Design the tests before or alongside the implementation.
- Translate upstream coverage into `zirconEngine` layers:
  - descriptor, manifest, or validation logic
  - module/plugin/service registration behavior
  - runtime semantics and structured errors
  - editor/runtime, asset, or host-facing integration behavior when applicable
  - authoring workflow behavior when the feature exposes tooling or timeline surfaces
  - serialization or artifact checks when persisted output matters
  - boundary, stress, and regression tests
- Mirror upstream negative tests, not just successful examples.
- Add limit tests for size, depth, overflow, missing handles, repeated load or reload, or long-running behavior when the feature can fail there.

6. Implement from lower shared support upward.
- Fix shared support layers before patching top-level behavior.
- Re-check module loading, descriptor registration, host capability surfaces, runtime ownership, or serialization helpers before forcing a high-level pass.

7. Validate and report with evidence.
- State which upstream files informed the design.
- State which foundational capability was confirmed sufficient or which foundational layer was changed instead of adding a special case.
- State which `zirconEngine` layers gained coverage and which gaps remain.
- Call out any deliberate divergence from the reference projects and why it is acceptable.

## Evidence Quality Bar

- Strong: implementation files plus tests from at least two reference engines or engine subsystems.
- Medium: implementation or tests from one reference engine plus corroborating docs or code from another.
- Weak: examples, docs, or memory only. Use only for initial direction, not final justification.

If there is no close precedent, narrow the feature to the smallest coherent subset and say that explicitly before implementing.

## Quick Use

From the repository root, search likely precedents:

```powershell
python .\.codex\skills\zircon-project-skills\zr-language-feature-design\scripts\search_feature_evidence.py "module|plugin|runtime" --languages zircon,fyrox,godot,bevy
python .\.codex\skills\zircon-project-skills\zr-language-feature-design\scripts\search_feature_evidence.py "timeline|sequence|state" --languages theatre,unreal
python .\.codex\skills\zircon-project-skills\zr-language-feature-design\scripts\search_feature_evidence.py "render|pipeline|pass" --languages graphics,unreal,bevy
```

For every proposal or implementation, report:
- the target milestone
- the reference projects and files consulted
- the foundational capability that carries the feature
- the chosen `zirconEngine` behavior and any divergence
- the planned or added validation, runtime, integration, boundary, and stress tests
