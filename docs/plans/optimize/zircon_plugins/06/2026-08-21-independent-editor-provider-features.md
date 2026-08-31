---
plan: docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
task: FP-CATALOG acceptance 10 - independent editor provider feature closure
status: static_validation_complete_release_batch_queued
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
- Integration Session: `root-runtime-interface03-activate-link-failure-20260831`;
  ownership apply `6fa932b4c1cc43af9c40453a13c6f10b`, preview
  `519ad86b11474e9fa9e9a1d9a37d5b7d`, fingerprint
  `1c9e6a8501368ce12bd15a0a667836cb9cc29e7e8238ceac843747a420a86ed7`.
- Current source SHA-256:
  `Cargo.toml=238685EFAF961EAA23D1217827BC7AD3EAE2C8E57A075920A8201A6025AA53B3`,
  `entry/mod.rs=82C4B6F4B85894CDAF7CED9BF4E10C5557C88ACF501FE99B1D0C62E5CFB96CBF`,
  `first_party_editor_plugins.rs=E5C8EAAA88CD2D2D5F720674212EA720975D528D2180652A487AAAA1BE33BDF5`.
- Deterministic model manifest SHA-256:
  `23D42492111C14269A52DCBE55D858E211E5A07B726BCD17EEE155B23AB186F8`.
  Each provider configuration executes exactly `21,504` manifest projections;
  both configurations execute `43,008`, preserving one registration per
  projection. Neural-only registration changes `0 -> 1`; Navigation remains
  `1 -> 1`; the empty fallback branch changes `1 -> 0`.
- Focused source/model/validator contract passed locally `9/9`; Python and
  PowerShell syntax, exact Rustfmt, and scoped diff checks are green. Managed
  static ticket: `d4c2e015d961432fb312b45bdd3ecd29`.
- The Neural-only and Navigation-only release configurations are queued
  together in ticket `0634d4a2041d46a4acfa4692f9c47d78`, using validator
  SHA-256 `46690484B0D0057D770262183CDECA886D11E50B95113A6CF54B4F36D878F169`.
  Their measured P50/P95 remain pending; no performance pass is claimed before
  that coordinator receipt becomes terminal.
