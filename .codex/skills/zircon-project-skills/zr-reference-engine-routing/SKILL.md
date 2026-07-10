---
name: zr-reference-engine-routing
description: Use when a `zirconEngine` task needs repository-local reference selection across `dev/UnrealEngine`, `bevy`, `Fyrox`, `godot`, `Graphics`, `Piccolo`, `slint`, or `theatre`, especially before designing or implementing complex systems, core infrastructure, rendering, editor UI, startup flow, animation tooling, or new module and directory layouts that should align with mature engine source structure first.
---

# Zr Reference Engine Routing

## Overview

Use this skill to decide which `dev/` reference trees should lead a `zirconEngine` task before deeper design or implementation work begins. Start from the touched `zircon_*` crates, choose a primary reference that matches the dominant problem, inspect the reference tree's directory shape before inventing local module layout, add secondary references for missing constraints, and then hand the task off to the deeper execution skill that fits the subsystem.

## Progressive Disclosure Index

- Read `references/reference-engine-map.md` for the detailed routing matrix.
- If the task is mainly scripting, module/plugin, runtime, editor/runtime, or evidence-driven subsystem design, also apply `../zr-language-feature-design/SKILL.md`.
- If the task is mainly system architecture, subsystem ownership, or long-horizon engine direction, also apply `../zr-architecture-first-engineering/SKILL.md`.
- If the task changes milestone order or needs bottom-up execution gates, also apply `../layered-milestone-development/SKILL.md`.
- If the task will edit or validate repository code, also apply `../../zircon-dev/SKILL.md`.

## Core Routing Rules

- Start from the touched `zircon_*` crates before choosing any external engine reference.
- Pick one primary reference that matches the task's hardest constraint: system scale, infrastructure shape, framework layering, rendering pipeline, minimal bootstrap, UI toolkit behavior, or animation tooling.
- Add at least one secondary reference whenever runtime semantics, editor workflows, rendering, or authoring surfaces cross subsystem boundaries.
- When the task creates or reshapes a module tree, inspect the reference directory layout first. Match subsystem boundaries, file roles, and folder seams before translating the design into Rust code.
- Use `UnrealEngine` to lead heavyweight feature families and engine-scale systems. Stabilize the landing zone with `Fyrox`, `bevy`, or the current repository before shaping shared Rust abstractions.
- Use `bevy` and `godot` to lead common infrastructure and reusable support behavior. Bring in `Fyrox` when the same work must align with the current editor/runtime split.
- Use `Graphics` to lead SRP-style rendering decisions. Pair it with `UnrealEngine` for large-system tradeoffs or with `bevy` or `Fyrox` for Rust-facing landing zones.
- Treat `dev/Graphics` as the repository's Unity Graphics-style reference for package-oriented rendering architecture and tooling layout.
- Use `Piccolo` to lead minimal startup and baseline engine entry work. Switch to `Fyrox` or `bevy` once the task grows beyond a small bootstrap path.
- Use `slint` and `theatre` to lead editor-facing authoring experience. Pair them with `Fyrox`, `godot`, or `UnrealEngine` when the UX must remain coherent with engine/runtime constraints.
- When in doubt, prefer a three-part stack: current `zircon_*` code for repository truth, one dominant external reference for the core idea, and one contrasting reference to expose missing edge cases.
- Bias toward folder and crate seams that survive future Unreal-scale growth. Avoid flat or feature-local layouts that would need to be torn apart once the subsystem expands.

## Quick Route Summary

- Heavyweight gameplay, rendering, simulation, streaming, or flagship engine systems: lead with `UnrealEngine`.
- Common engine infrastructure, data flow, reflection, serialization, resources, ECS-like scheduling, and recurring utilities: lead with `bevy` and `godot`.
- Program framework, editor/runtime separation, Rust-native subsystem layering, and current repository fit: lead with `Fyrox`.
- Render pipeline, SRP-style pass layout, shader graph, VFX graph, and Unity Graphics-derived architecture: lead with `Graphics`.
- Minimal engine bootstrap, smallest possible entry flow, and baseline runtime startup: lead with `Piccolo`.
- Editor UI shell, panel layout, declarative UI bindings, and document-oriented tooling: lead with `slint`.
- Timeline, sequencing, animation authoring, state-machine-adjacent tooling, and studio-style timeline UX: lead with `theatre`.

## Structure-First Workflow

1. Read the touched `zircon_*` crates and name the owning subsystem.
2. Open the matching directories in the primary and secondary references. Compare how they split runtime, editor, servers, packages or crates, declarations, and behavior families.
3. Sketch the target `zirconEngine` path layout before writing implementation code: crate, folder, declaration files, behavior files, and minimal root wiring files.
4. Only then hand off to `zr-language-feature-design`, `zr-architecture-first-engineering`, or `zircon-dev` for the deeper work.

## Escalation Rules

- Route to `zr-language-feature-design` when the task needs concrete cross-engine evidence, semantic design, and validation planning.
- Route to `zr-architecture-first-engineering` when the task is mostly about subsystem ownership, layering, or long-term engine architecture.
- Route to `layered-milestone-development` when the work must be ordered bottom-up across shared foundations before upper-layer features.
- Route to `zircon-dev` when the reference decision is made and repository code or validation work must begin.
