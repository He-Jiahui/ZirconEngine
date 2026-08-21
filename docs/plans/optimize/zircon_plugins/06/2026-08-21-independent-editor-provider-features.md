---
plan: docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
task: FP-CATALOG acceptance 10 - independent editor provider feature closure
status: validation_pending
date: 2026-08-21
owned_code:
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/mod.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
tests:
  - app_composition_projects_selected_navigation_editor_provider
  - app_composition_projects_selected_neural_editor_provider
validator: .codex/state/session-coordinator/cargo-runs/zircon-validation-plugins06-independent-editor-providers.ps1
---

# Plugins06 independent editor provider feature closure

## Problem

`zircon_app` delegated editor provider registration only when
`first-party-navigation-editor-plugin` was enabled. A build that linked the Neural editor
provider without Navigation therefore compiled the fallback branch and returned no registrations,
even though `zircon_first_party_editor_catalog` contained a valid Neural provider.

The `target-editor-host` feature currently enables both providers, which hid the defect from the
standard product build and made the Neural-only combination impossible to validate at the App
composition boundary.

## Change

- Added provider-neutral `first-party-editor-catalog`, which owns the Editor/catalog dependencies.
- Made Navigation and Neural editor provider features compose through that shared feature.
- Gated the App catalog adapter on the shared feature rather than on Navigation or the full Editor
  product target.
- Removed the empty fallback branch. Every linked provider combination now delegates to the same
  catalog resolver.
- Kept `target-editor-host` behavior unchanged: it still enables both existing editor providers.

The design follows Bevy's separation between plugin-group composition and individual feature-gated
members (`dev/bevy/crates/bevy_app/src/plugin_group.rs`) and Fyrox's explicit distinction between
statically and dynamically available plugin containers
(`dev/Fyrox/fyrox-impl/src/plugin/mod.rs`). Zircon deliberately keeps its existing catalog and
registration report types rather than importing either engine's container model.

## Acceptance

One coordinator batch must build and run the App library twice with `--no-default-features`:

1. `first-party-neural-editor-plugin` only resolves exactly one Neural registration.
2. `first-party-navigation-editor-plugin` only resolves exactly one Navigation registration.
3. Each configuration executes 21 release samples of 1,024 real manifest-to-registration
   projections.
4. Each configuration emits nearest-rank P50/P95, retains one registration per iteration, and
   keeps P95 at or below 250,000 microseconds.
5. The existing full `target-editor-host` feature composition remains unchanged.

## Evidence

- The validator parsed with zero PowerShell errors.
- Its pre-implementation static gate failed on the missing provider-neutral feature, establishing
  the regression before the production edit.
- Focused `rustfmt --check` and `git diff --check` pass.
- Cargo behavior tests and measured P50/P95 are pending the serialized Runtime root-workspace
  aggregate. No performance pass is claimed before that coordinator receipt becomes terminal.
