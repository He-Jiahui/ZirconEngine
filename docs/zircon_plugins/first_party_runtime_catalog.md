---
related_code:
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/source_assertions.rs
implementation_files:
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/source_assertions.rs
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - rustfmt --edition 2021 --check zircon_plugins/first_party_runtime_catalog/src/lib.rs zircon_app/src/entry/first_party_runtime_plugins.rs zircon_app/src/entry/tests/source_assertions.rs
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --format-version 1 --no-deps --locked
  - cargo metadata --format-version 1 --no-deps --locked
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-first-party-catalog-0604 --message-format short --color never
doc_type: module-detail
---

# First-Party Runtime Catalog

## Purpose

`zircon_first_party_runtime_catalog` is the linked provider catalog for first-party runtime plugins. It centralizes the optional Rust crate fan-out that used to live in `zircon_app`.

`zircon_app` still owns process entry, target/profile choice, and render-profile projection. The app then delegates manifest-to-provider selection to this catalog. That keeps the process host from directly knowing every optional runtime plugin implementation crate while preserving the existing profile bootstrap behavior.

## Boundary

The catalog lives in the plugin workspace because it depends on concrete first-party provider crates under `zircon_plugins/*/runtime`. `zircon_runtime` must not depend on those implementation crates. The runtime-owned contract remains `RuntimePluginRegistrationReport`, `ProjectPluginManifest`, `RuntimePluginId`, and the runtime module assembly helpers that consume registration reports.

This mirrors the current engine split:

- `zircon_runtime` owns plugin ids, manifests, descriptors, registration reports, availability reports, and module assembly.
- `zircon_plugins/*/runtime` owns concrete first-party provider implementations.
- `zircon_first_party_runtime_catalog` maps selected runtime plugin ids to compiled provider registration reports.
- `zircon_app` projects config and calls the catalog through its entry helper.

## Feature Groups

- `base-runtime-plugins` links AI, Sound, Texture, Net, Particles, Animation, and Rendering providers.
- `advanced-render-runtime-plugins` links Virtual Geometry, Hybrid GI, and Solari providers.
- `navigation-runtime-plugin` links the Navigation provider separately so native/Recast-oriented validation can remain explicit.

The app-facing feature names remain stable:

- `first-party-runtime-plugins`
- `first-party-advanced-render-runtime-plugins`
- `first-party-navigation-runtime-plugin`

Each app feature now enables the catalog plus the matching catalog feature instead of directly naming individual `zircon_plugin_*_runtime` crates.

## Regression Guard

`zircon_app/src/entry/tests/source_assertions.rs` now checks that:

- `zircon_app/Cargo.toml` depends on `zircon_first_party_runtime_catalog`;
- app features do not mention individual first-party `zircon_plugin_*_runtime` crates;
- `zircon_app/src/entry/first_party_runtime_plugins.rs` delegates provider collection to the catalog instead of calling concrete `plugin_registration()` functions directly.

This is a structural guard, not a replacement for profile bootstrap tests. Provider behavior still needs feature-enabled app/profile validation at M2 milestone boundaries.
