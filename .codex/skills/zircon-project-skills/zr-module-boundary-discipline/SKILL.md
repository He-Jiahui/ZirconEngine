---
name: zr-module-boundary-discipline
description: Use when designing, refactoring, or extending `zirconEngine` Rust modules and a file or directory is starting to mix multiple feature domains, declarations, and logic; especially when binding/root modules are accumulating behavior, similar modules are being added flat, declaration files are collecting implementation details, or subsystem boundaries need stricter folder-backed separation that can survive future engine-scale growth.
---

# Zr Module Boundary Discipline

## Overview

Apply this skill to keep `zirconEngine` source trees aggressively modular: one responsibility per file, one subsystem per folder, and root wiring files that stay structural instead of becoming umbrella implementations. Keep `.rs` files concise and independent, keep declaration files minimal, and choose folder shapes that can scale long before the codebase reaches Unreal-like complexity. Use it when line count is not the real problem yet, but boundary drift already is.

This skill exists because files such as `zircon_editor/src/ui/binding.rs` can stay under a crude "large file" threshold and still be architecturally wrong by mixing asset, inspector, dock, viewport, routing, parsing, and binding concerns into one surface.

## Progressive Disclosure Index

- Read `references/binding-rs-anti-pattern.md` when a mixed editor/UI binding module needs a concrete decomposition target.
- Read `../../zircon-dev/workflow/structure/root-entry-files.md` when `binding.rs`, `lib.rs`, `main.rs`, or `mod.rs` is becoming an implementation file instead of a structural boundary.
- Also apply `../modularize-large-files/SKILL.md` when the touched file is near or above the repository size warning threshold.
- Also apply `../zr-reference-engine-routing/SKILL.md` before inventing a new folder or module layout that should align with reference engine source trees.
- Also apply `../../zircon-dev/SKILL.md` when the task will edit Rust workspace code and needs repository-wide workflow or validation rules.

## Non-Negotiable Rules

- One source file may own one coherent responsibility only. Do not let "same general area" justify multiple domains in one file.
- Keep each `.rs` file as small and self-sufficient as the responsibility allows. If the next helper, `impl`, or match arm would introduce a second behavior family, split first.
- Do not colocate multiple top-level enum, struct, trait, or error declarations in one Rust file when they represent different concepts. Give each declaration its own file.
- Group related files under a folder-backed subtree. Do not keep similar modules flat at the crate root or subsystem root once they clearly belong to one domain.
- Keep `binding.rs`, `lib.rs`, `main.rs`, and `mod.rs` structural. They may declare child modules, re-export public items, and expose minimal entry wiring. They may not become long-term homes for parsing, conversion, routing, command-specific behavior, or mixed helper logic.
- Treat each domain boundary as visible in the path. If a reader must open the file to learn whether code is about assets, docking, inspector, or viewport behavior, the tree is too vague.
- Split declarations from behavior. Declaration files define the type. Behavior files implement one behavior family such as `encode`, `decode`, `parse`, `route`, `snapshot`, or `mutation`.
- Keep declaration files minimal. A declaration file may contain derives, fields, narrow invariants, and only the smallest constructor or accessor helpers needed to keep the type readable. Move parsing, formatting, routing, conversion, serialization, and non-trivial `impl` families out into behavior files.
- Prefer adding a new file under an existing domain folder over appending a new section to an umbrella file.
- Bias toward early subdivision when a subsystem is likely to grow. If the shape would need a structural rewrite after 5-10 sibling commands, modes, or backends, deepen the tree now.

## File Role Rules

### Root wiring files

Use files such as `binding.rs`, `lib.rs`, `main.rs`, and folder `mod.rs` files only for:

- `mod ...;`
- `pub use ...;`
- minimal type aliases
- narrow construction or pass-through entry points

Move everything else down one level. If a root wiring file starts matching on symbols, parsing payloads, mutating state, or converting domain values, it is already over the line.

Use `../../zircon-dev/workflow/structure/root-entry-files.md` for the hard per-file allowlist, denylist, red lines, and target directory shapes.

### Declaration files

Use one file per top-level declaration:

- one enum file
- one struct file
- one trait file
- one error file

Name the file after the declaration or the narrow concept it owns: `asset_command.rs`, `viewport_command.rs`, `editor_ui_binding.rs`, `inspector_field_change.rs`, `editor_ui_binding_error.rs`.

If a declaration needs more than trivial methods, keep the type in its own file and move the extra behavior into sibling behavior files under the same domain folder.

### Behavior files

Use separate files for behavior families that would otherwise bloat a declaration file:

- `encode.rs`
- `decode.rs`
- `parse.rs`
- `route.rs`
- `format.rs`
- `snapshot.rs`
- `mutation.rs`

One behavior file may contain several tightly related helpers, but only for one domain and one behavior family.

### Domain folders

Create a folder as soon as several files belong to one subsystem:

- `asset/`
- `viewport/`
- `dock/`
- `inspector/`
- `binding/`

Do not leave these as flat siblings when the folder name is already obvious.

## Folder-First Triggers

Convert a file into a folder-backed subtree immediately when any of these are true:

- the file now mentions more than one feature domain,
- the module contains both public API wiring and internal logic,
- declarations and implementations are both growing,
- similar file names would otherwise accumulate flat at one directory level,
- the next edit would add "one more section" instead of one more module.

Do not wait for 1000 lines. A 300-line mixed-responsibility file is already a design failure.

## Reference-Shaped Layout Check

Before creating a new module tree, compare the intended boundary with the closest matching paths in `dev/UnrealEngine`, `dev/godot`, `dev/bevy`, `dev/Fyrox`, or `dev/Graphics` as routed by `zr-reference-engine-routing`.

- Copy the boundary idea, not the language syntax: runtime vs editor splits, subsystem folders, package or crate seams, and behavior-family separation.
- Prefer the layout that still reads clearly after 10x more declarations, commands, or backends.
- If the reference tree already separates declaration, routing, serialization, authoring, or runtime behavior, do not recombine them in one local `.rs` file.
- Treat "good enough for this feature" as a red flag when the subsystem is expected to keep growing.

## Refactor Workflow

1. List the distinct domains currently mixed together.
2. Choose the domain folders first. Paths should reveal subsystem ownership before implementation details.
3. Move each top-level declaration into its own file.
4. Move behavior into narrow files grouped by behavior family inside the owning domain folder.
5. Reduce the original root file to wiring only: child-module declarations, re-exports, and the smallest public entry points that still make sense.
6. Re-check imports and call sites. If callers still need to understand internal layout details, keep refining the public surface.

## Red Flags

- `binding.rs` contains asset, inspector, dock, viewport, and router logic together.
- `lib.rs`, `main.rs`, or `mod.rs` becomes the easiest place to hide “temporary” real behavior.
- A file name becomes a fake umbrella such as `binding.rs`, `editor.rs`, `runtime.rs`, or `helpers.rs`.
- New files keep appearing flat with repeated prefixes instead of under a folder.
- A top-level file defines several enums and several structs because they are "all commands."
- Parsing helpers, string-name mappings, encoding, and routing all live beside the declaration.
- A root file needs scrolling to discover what subsystems it secretly owns.

All of these mean the boundary is already wrong. Split first, then continue the feature.

## Completion Standard

Do not call a reorganization complete unless all of the following are true:

- the path alone tells a reviewer which subsystem owns the code,
- root wiring files stay short and mostly declarative,
- each major enum/struct/error declaration lives in its own file,
- implementation helpers are grouped by one domain and one behavior family,
- adding a new feature to one subsystem no longer requires reopening an unrelated umbrella file.
