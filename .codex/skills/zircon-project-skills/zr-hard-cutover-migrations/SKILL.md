---
name: zr-hard-cutover-migrations
description: Use when renaming, relocating, absorbing, or converging `zirconEngine` crates, modules, folders, or public APIs and the old path is not meant to survive; especially when there is temptation to leave `pub use` re-exports, compatibility modules, shim traits, facade wrappers, bridge folders, or stale `use` sites behind during migration.
---

# Zr Hard Cutover Migrations

## Overview

Land `zirconEngine` migrations as direct cutovers instead of coexistence phases. Once a new owner path is chosen, move every consumer to it in the same change and delete the superseded surface instead of preserving old imports behind forwarding layers.

## Progressive Disclosure Index

- Also apply `../zr-architecture-first-engineering/SKILL.md` when the migration changes crate ownership, subsystem authority, or public architecture names.
- Also apply `../zr-module-boundary-discipline/SKILL.md` when the migration changes folder layout, module seams, or root wiring files.
- Also apply `../../zircon-dev/SKILL.md` when the task edits Rust workspace code or needs validation guidance.

## Non-Negotiable Rules

- Do not add or preserve `pub use` re-exports whose job is to keep an old migration path alive.
- Do not create compatibility modules, shim traits, alias files, facade wrappers, bridge folders, or forwarding `mod.rs` files just to avoid changing callers.
- After moving a type, function, module, or file, update every reachable `use` site, fully qualified path, test, doc assertion, and structural check that still references the old location.
- Delete superseded files, directories, exports, and stale tests in the same change once the last intended consumer moves.
- Treat root wiring files as structural only. If a migration leaves a root file acting as a forwarding shell, remove the shell instead of polishing it.
- Prefer immediate compile failures in stale consumers over hiding drift behind a wrapper. Broken imports are evidence that more migration work remains.
- Update path-based tests to the new home. A test that still points at the old directory is migration debt, not a reason to keep the old directory.
- If a curated public surface already exists for design reasons, do not use this skill to widen it. This skill is only about rejecting migration-only forwarding layers.

## Hard-Cutover Workflow

1. Name the old owner path and the target owner path before editing.
2. Move the declaration or implementation into the target path first, then make consumers compile against that destination instead of reintroducing the old path.
3. Search the touched workspace for the old path, old symbol name, likely alias names, and migration-smell words such as `compat`, `shim`, `facade`, `bridge`, and `legacy`.
4. Rewrite all intentional consumers to the new direct path. Do not stop after one crate or one test target builds.
5. Remove migration-only forwarding layers immediately after the direct consumers are updated.
6. Re-run the searches. Any remaining live hit in the touched subsystem is a blocker unless it is clearly historical test data or a deliberate non-migration API.
7. Validate the narrowed package or workspace. Remaining failures should now expose real stale callers or pre-existing unrelated breakage, not preserved compatibility glue.

## Search Checklist

- old module path
- old type or function name
- new file name accidentally re-exported from old root
- `pub use`
- `compat`
- `shim`
- `facade`
- `bridge`
- `legacy`

## Red Flags

- "Leave a `pub use` for now so the rest compiles."
- "Add a thin facade and clean it up later."
- "Keep the old folder because tests still point there."
- "Move the implementation now; fix imports in a later pass."
- "Bridge module", "compat module", or "legacy alias" appears in the new diff.
- A root file becomes a forwarding layer instead of a structural boundary.

All of these mean the migration is not complete. Continue cutting over until the old surface is gone.

## Completion Standard

- Callers import the new path directly.
- The touched subsystem keeps no migration-only `pub use`, facade, shim, bridge, or alias module.
- Obsolete files and directories from the old path are deleted or proven to still own a real non-migration responsibility.
- Tests, docs, and structural assertions point at the new location.
- Validation has been run, and any remaining failures are reported as unrelated pre-existing issues or newly exposed stale consumers, not hidden behind compatibility glue.
