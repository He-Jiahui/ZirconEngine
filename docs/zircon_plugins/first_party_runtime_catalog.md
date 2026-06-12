---
related_code:
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/source_assertions.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - examples/vampire/zircon-project.toml
implementation_files:
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/source_assertions.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - examples/vampire/zircon-project.toml
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - rustfmt --edition 2021 --check zircon_plugins/first_party_runtime_catalog/src/lib.rs zircon_app/src/entry/first_party_runtime_plugins.rs zircon_app/src/entry/tests/source_assertions.rs
  - app optional-plugin crate fan-out source guard over all current `zircon_plugin_*_runtime` package names parsed from `zircon_plugins/Cargo.toml`
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --format-version 1 --no-deps --locked
  - cargo metadata --format-version 1 --no-deps --locked
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-first-party-catalog-0604 --message-format short --color never
  - cargo check -p zircon_app --bin zircon_runtime --features "target-client,first-party-runtime-plugins,first-party-navigation-runtime-plugin,first-party-zr-vm-language-runtime-plugin,first-party-zr-vm-real-backend" --message-format short --color never with CARGO_TARGET_DIR=E:\cargo-targets\zircon-vampire-app, ZR_VM_RUST_BINDING_LIB_DIR=E:\Git\zr_vm\build\codex-msvc-debug\lib\Debug, PATH including E:\Git\zr_vm\build\codex-msvc-debug\bin\Debug: passed 2026-06-09
  - cargo build -p zircon_app --bin zircon_runtime --features "target-client,first-party-runtime-plugins,first-party-navigation-runtime-plugin,first-party-zr-vm-language-runtime-plugin,first-party-zr-vm-real-backend" --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app, ZR_VM_RUST_BINDING_LIB_DIR=E:\Git\zr_vm\build\codex-msvc-debug\lib\Debug, PATH including E:\Git\zr_vm\build\codex-msvc-debug\bin\Debug: passed 2026-06-09
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

- `base-runtime-plugins` links AI, Sound, Texture, glTF Importer, Net, Particles, Animation, and Rendering providers.
- `advanced-render-runtime-plugins` links Virtual Geometry, Hybrid GI, and Solari providers.
- `navigation-runtime-plugin` links the Navigation provider separately so native/Recast-oriented validation can remain explicit.
- `zr-vm-language-runtime-plugin` links the ZrVM language provider.
- `zr-vm-real-backend` enables the ZrVM provider plus its `real-zr-vm` native binding feature.

The app-facing feature names remain stable:

- `first-party-runtime-plugins`
- `first-party-advanced-render-runtime-plugins`
- `first-party-navigation-runtime-plugin`
- `first-party-zr-vm-language-runtime-plugin`
- `first-party-zr-vm-real-backend`

Each app feature now enables the catalog plus the matching catalog feature instead of directly naming individual `zircon_plugin_*_runtime` crates.

The Vampire runtime example uses this feature set together with `first-party-runtime-plugins` so rendering/texture/animation/glTF-import providers, navigation, and the ZrVM language runtime can be linked into the standalone `zircon_runtime` app binary.

The dynamic runtime executable still creates sessions through `zircon_runtime.dll`; app-linked catalog registrations do not automatically cross the ABI. For that reason the runtime default asset importer now carries built-in glTF/GLB, common image, and text-data importers for simple standalone project startup. The catalog glTF provider remains the first-party plugin-registration path for static/catalog-driven hosts and can override the built-in matcher when installed with higher priority.

## Regression Guard

`zircon_app/src/entry/tests/source_assertions.rs` now checks that:

- `zircon_app/Cargo.toml` depends on `zircon_first_party_runtime_catalog`;
- app features do not mention any current first-party `zircon_plugin_*_runtime` package from the plugin workspace, including importers and feature-provider runtime packages;
- `zircon_app/src/entry/first_party_runtime_plugins.rs` delegates provider collection to the catalog instead of calling concrete `plugin_registration()` functions directly.

This is a structural guard, not a replacement for profile bootstrap tests. Provider behavior still needs feature-enabled app/profile validation at M2 milestone boundaries.
