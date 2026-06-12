---
related_code:
  - zircon_editor/Cargo.toml
  - zircon_editor/src/ui/host
  - zircon_editor/src/ui/retained_host
  - zircon_runtime/Cargo.toml
  - zircon_runtime_interface/Cargo.toml
  - zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs
implementation_files:
  - zircon_editor/Cargo.toml
  - zircon_editor/src/ui/host
  - zircon_editor/src/ui/retained_host
  - zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - docs/engine-architecture/runtime-tech-stack.md
tests:
  - zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs
  - editor_only_dependency_candidates_have_editor_backlog_owner
doc_type: module-detail
---

# Runtime Editor-Only Dependency Backlog

## Purpose

This backlog records libraries that were previously claimed as runtime dependencies but are actually editor-host conveniences. They must not be added to `zircon_runtime` or `zircon_runtime_interface`. If they are introduced, the implementation belongs under `zircon_editor` and must be validated through the editor UI/tooling plans.

## Candidate Table

| Candidate | Editor need | Expected owner | Runtime rule |
|---|---|---|---|
| `fontdue` | Current retained-host text measurement/raster fallback in `zircon_editor/src/ui/retained_host/host_contract/painter/text.rs`. | `zircon_editor/src/ui/retained_host` owns the existing fallback; migration to runtime UI text/glyphon/SDF belongs to the editor UI text plan. | Do not add to `zircon_runtime`. Keep as temporary editor-only debt until retained-host text rendering consumes the runtime UI text stack. |
| `rfd` | Native file and folder dialogs for opening projects, importing external assets, choosing export directories, and selecting package destinations. | `zircon_editor/src/ui/host` owns the command surface; `zircon_editor/src/ui/retained_host` may adapt it to the retained window shell. | Do not add to `zircon_runtime` or `zircon_runtime_interface`. Runtime export and asset APIs should receive resolved paths/locators, not drive native dialogs. |
| `arboard` | Clipboard integration for editor text fields, inspector values, asset paths, node names, and future graph editors. | `zircon_editor/src/ui/host` owns editor commands and diagnostics; retained-host adapters may translate platform clipboard events. | Do not add to runtime. Runtime input/dispatch contracts may carry clipboard intent/results as DTOs only if a future UI plan needs them. |

## Admission Gate

Before `rfd` or `arboard` enters `zircon_editor/Cargo.toml`, the editor plan must define:

- The command or host-service facade that owns the feature.
- How headless tests bypass native OS dialogs or clipboard state.
- How failure diagnostics surface to editor panes without leaking OS details into runtime contracts.
- Which runtime/interface DTOs, if any, are needed for requests or results.

Before `fontdue` is removed, the editor plan must define the retained-host text replacement path and prove that text measurement, glyph cache/raster policy, and repaint invalidation use the runtime UI text boundary or an explicitly editor-owned replacement. The runtime-side guard `fontdue_editor_retained_host_dependency_has_migration_owner` keeps the current debt visible while rejecting any runtime manifest dependency on `fontdue`.

The runtime-side guard `editor_only_dependency_candidates_have_editor_backlog_owner` keeps this backlog linked from the runtime dependency authority and rejects silent `rfd` / `arboard` manifest additions.
