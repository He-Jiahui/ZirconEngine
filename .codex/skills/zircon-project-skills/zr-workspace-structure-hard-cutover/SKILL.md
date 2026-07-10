---
name: zr-workspace-structure-hard-cutover
description: Use when refactoring or reviewing `zirconEngine` workspace structure, crate roots, owner paths, or folder layouts and the old path is not meant to survive; especially when enforcing the fixed `zircon_app`/`zircon_runtime`/`zircon_editor` shape, shrinking root public surfaces, deleting compat-style structure drift, or standardizing docs after a hard cutover.
---

# ZR Workspace Structure Hard Cutover

## Overview

Use this skill to keep workspace-wide refactors destructive and converged. The target structure is fixed first; stale owners, transition folders, and migration-only root exports are deleted instead of preserved.

## Read First

- `docs/engine-architecture/workspace-ownership-cutover-map.md`
- `docs/engine-architecture/workspace-root-rules-and-hard-cutover.md`
- `docs/editor-and-tooling/editor-structure-hard-cutover-rules.md`
- `docs/assets-and-rendering/runtime-surface-and-assets-rules.md`

Also apply:

- `zr-hard-cutover-migrations`
- `zr-module-boundary-discipline`

## Non-Negotiable Rules

- Workspace shape is fixed:
  - `zircon_app` = thin entry host only
  - `zircon_runtime` = runtime absorption and stable runtime-facing contracts
  - `zircon_editor` = `core/scene/ui` authoring tree
- Root files stay structural. No mixed-domain logic in `lib.rs`, `mod.rs`, or similar entry files.
- Do not keep migration-only `pub use`, alias modules, shim folders, `compat`, `legacy`, or empty bridge/service shells.
- Move every intended caller to the new owner path in the same change, then delete the old path.
- `zircon_editor::ui::host` owns editor host orchestration; `ui::slint_host` is glue only; `ui::asset_editor` stays folder-backed.
- `zircon_editor::ui` may keep curated reflection/activity exports, but binding, asset-editor, template, and control specialists must be referenced through their owner modules instead of the `ui` root.
- `zircon_editor/src/lib.rs` may re-export `EditorModule`, but must not own the `EditorModule` type or `EngineModule` implementation; that ownership lives under `ui/host/module.rs`.
- Runtime root test folders such as `zircon_runtime/src/tests/{runtime_absorption,extensions,ui_boundary}` follow the same rule: their `mod.rs` files stay navigational only, while assertions live in focused child files.
- `zircon_runtime::ui::dispatch` and `zircon_runtime::ui::surface` stay folder-backed namespace owners; their root `mod.rs` files may declare child modules and curated exports, but pointer/navigation/render/surface behavior must live below them.
- `zircon_runtime::platform` and `zircon_runtime::extensions::{navigation,net,particles,sound,texture}` must follow the same rule: root `mod.rs` files may re-export the public surface, but module/config/service ownership and descriptor-builder logic must live in child files such as `module.rs`, `config.rs`, `service_types.rs`, or `registration.rs`.
- Runtime production `.ui.toml` resources belong in crate `assets/`, never in `src/`.

## Completion Standard

- Old owner-path searches are clean in the touched subsystem.
- Docs and skills mention only converged paths.
- Public roots are narrowed; specialist types are accessed through owner modules.
- Remaining failures, if any, are clearly unrelated active-workstream issues rather than migration residue.
