---
name: zr-runtime-interface-convergence
description: Use when designing, reviewing, or refactoring `zirconEngine` entry/module/service boundaries and you need to map `IEntry`/`IManager`/`IDriver`/`IPlugin`/`IEntity`/`IComponent` concepts onto the current Rust contracts, audit whether module crates are converged versus skeleton, or decide where structure drift requires targeted refactoring before more features land.
---

# Zr Runtime Interface Convergence

## Overview

Use this skill to reconcile the roadmap vocabulary in [`基本路线图`](../../../../.cursor/plans/基本路线图.md) and [`全系统重构方案`](../../../../.codex/plans/全系统重构方案.md) with the current Rust workspace. Read the roadmap first, map the request into the interface family, run the structural audit, and only then decide whether the task is a leaf edit, a module-owner convergence task, or a deeper refactor.

## Progressive Disclosure Index

- Read `../../../../.cursor/plans/基本路线图.md` and `../../../../.codex/plans/全系统重构方案.md` first.
- Read `references/interface-family.md` when the task mentions `IEntry`, `IModule`, `IService`, `IDriver`, `IManager`, `IPlugin`, `IObject`, `ISystem`, `IEntity`, or `IComponent`.
- Read `references/structural-audit.md` when deciding whether the current workspace is already structured enough or where refactoring should start.
- Run `python ./.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py` before calling a module crate converged.
- Also apply `../zr-architecture-first-engineering/SKILL.md` when the change affects runtime architecture ownership.
- Also apply `../zr-module-boundary-discipline/SKILL.md` when a touched file or folder is already structurally overloaded.
- Also apply `../../zircon-dev/SKILL.md` when the task will edit Rust workspace code.

## Workflow

1. Anchor the request in the roadmap.
- Treat `基本路线图.md` and `.codex/plans/全系统重构方案.md` as the authority for concept names and layering.
- Do not invent a parallel abstraction family when the roadmap already names the role.

2. Map concept names to Rust contracts.
- Use `references/interface-family.md` to convert `I*` concepts into the current Rust-facing contracts.
- Keep concept names in architecture discussion.
- Keep Rust-style trait and type names in code.

3. Run the structural audit before proposing refactors.
- Use `scripts/audit_runtime_structure.py`.
- Check module descriptor distribution, `stub_module_descriptor` use, module owner coverage, plugin runtime gaps, `zircon_app` static dependencies, and large file hotspots.
- Treat the audit as evidence. Do not guess which crates are converged.

4. Classify the touched crates before acting.
- `converged`: real `EngineModule` owner, non-stub module descriptor, no immediate boundary red flag in the touched surface.
- `skeleton`: still built around `stub_module_descriptor` or equivalent no-op lifecycle wiring.
- `needs-refactor`: owner missing, entry coupling still too static, plugin boundary still shallow, or major production hotspots are accumulating too much responsibility.

5. Choose the smallest convergence move that removes ambiguity.
- Prefer adding or tightening the missing contract first: `EngineEntry`, `EngineModule`, `EngineService`, or ECS semantics traits.
- Do not jump straight to a broad rewrite if one missing boundary explains the drift.
- Keep `IEntity` and `IComponent` as ECS semantics, not as a new OO inheritance tree.

## Non-Negotiable Rules

- Keep `IEntry -> EngineEntry`, `IModule -> EngineModule`, and `IService -> EngineService` aligned with the current workspace crates instead of scattering duplicate abstractions into subsystem crates.
- Treat `EngineService` as the metadata-level runtime contract. Do not force every concrete manager or driver instance into one monolithic base trait when descriptors plus facade traits already carry the real behavior.
- Keep `IDriver` dependency direction strict: driver-like services may depend on other drivers, never on managers.
- Keep `IManager` dependency access descriptor-driven or resolver-driven. Avoid direct long-lived strong coupling across lifecycle boundaries.
- Keep `IPlugin` reserved for the VM/plugin runtime path. Do not use it as a synonym for every dynamically loaded service.
- Keep `IObject`, `ISystem`, `IEntity`, and `IComponent` in `zircon_runtime::scene` as runtime semantics only. Do not use this skill to justify rebuilding ECS data as deep OO class trees.
- If the audit says a crate is `skeleton` or `needs-refactor`, do not call it architecture-complete just because it compiles.

## Red Flags

- A request asks for `IManager` or `IDriver`, but the proposed change only adds another descriptor helper name.
- A module crate exposes `module_descriptor()` but has no real owner type implementing `EngineModule`.
- `zircon_app` keeps accumulating subsystem-specific bootstrap knowledge instead of converging through `EngineEntry`.
- `zircon_runtime::script` is treated as if plugin lifecycle were already fully wired through `PluginDescriptor` and `resolve_plugin`, even when audit output says otherwise.
- `IEntity` or `IComponent` proposals introduce behavior inheritance instead of ECS identity/data semantics.

All of these mean the convergence work is not done yet.

## Quick Use

```powershell
python .\.codex\skills\zircon-project-skills\zr-runtime-interface-convergence\scripts\audit_runtime_structure.py
python .\.codex\skills\zircon-project-skills\zr-runtime-interface-convergence\scripts\audit_runtime_structure.py --json
```

Report at least:
- the touched interface family members,
- the current crate classifications,
- the concrete audit evidence behind the classification,
- the smallest refactor or convergence step that should land next.
