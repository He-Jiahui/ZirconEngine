---
related_code:
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests/provider_snapshot.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/source_assertions.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - examples/vampire/zircon-project.toml
  - zircon_plugins/*/plugin.toml
  - zircon_plugins/*/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/runtime_plugin/descriptor.rs
  - zircon_plugins/asset_importers/*/plugin.toml
  - zircon_plugins/opus_importer/plugin.toml
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/registration.rs
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor/package_manifest.rs
  - tools/audit_plugin_structure.py
  - tools/plugin_structure_audits/manifest_schema.py
  - tools/plugin_structure_audits/skeleton.py
  - zircon_plugins/plugin_sdk_examples/editor/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/capability.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extensions.rs
implementation_files:
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests/provider_snapshot.rs
  - tools/audit_plugin_structure.py
  - tools/plugin_structure_audits/manifest_schema.py
  - tools/plugin_structure_audits/skeleton.py
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
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
tests:
  - rustfmt --edition 2021 --check zircon_plugins/first_party_runtime_catalog/src/lib.rs zircon_app/src/entry/first_party_runtime_plugins.rs zircon_app/src/entry/tests/source_assertions.rs
  - app optional-plugin crate fan-out source guard over all current `zircon_plugin_*_runtime` package names parsed from `zircon_plugins/Cargo.toml`
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --format-version 1 --no-deps --locked
  - cargo metadata --format-version 1 --no-deps --locked
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-first-party-catalog-0604 --message-format short --color never
  - cargo check -p zircon_app --bin zircon_runtime --features "target-client,first-party-runtime-plugins,first-party-navigation-runtime-plugin,first-party-zr-vm-language-runtime-plugin,first-party-zr-vm-real-backend" --message-format short --color never with CARGO_TARGET_DIR=E:\cargo-targets\zircon-vampire-app, ZR_VM_RUST_BINDING_LIB_DIR=E:\Git\zr_vm\build\codex-msvc-debug\lib\Debug, PATH including E:\Git\zr_vm\build\codex-msvc-debug\bin\Debug: passed 2026-06-09
  - cargo build -p zircon_app --bin zircon_runtime --features "target-client,first-party-runtime-plugins,first-party-navigation-runtime-plugin,first-party-zr-vm-language-runtime-plugin,first-party-zr-vm-real-backend" --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app, ZR_VM_RUST_BINDING_LIB_DIR=E:\Git\zr_vm\build\codex-msvc-debug\lib\Debug, PATH including E:\Git\zr_vm\build\codex-msvc-debug\bin\Debug: passed 2026-06-09
  - rustfmt --edition 2021 --config skip_children=true --check zircon_plugins/first_party_runtime_catalog/src/lib.rs zircon_plugins/native_dynamic_fixture/native/src/lib.rs: passed 2026-06-22
  - static generated manifest header scan over non-native zircon_plugins/*/plugin.toml: 30/30 passed 2026-06-22
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12_static_plugin_manifest_is_generated --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins12-ds7-0622 --message-format short --color never -- --test-threads=1: timed out after 1200s on 2026-06-22; not counted as passing
  - rustfmt --edition 2021 --config skip_children=true --check zircon_plugins/asset_importers/audio/runtime/src/lib.rs zircon_plugins/asset_importers/texture/runtime/src/lib.rs zircon_plugins/asset_importers/model/runtime/src/registration.rs zircon_plugins/asset_importers/model/runtime/src/tests/registration.rs zircon_plugins/first_party_runtime_catalog/src/lib.rs: passed 2026-06-22
  - static generated manifest header scan over non-native zircon_plugins/**/plugin.toml: 36/36 passed 2026-06-22
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_audio_runtime -p zircon_plugin_asset_importer_texture_runtime -p zircon_plugin_asset_importer_model_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins12-missing-manifests-0622 --message-format short --color never: passed 2026-06-22 with existing zircon_runtime warnings
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12_static_plugin_manifest_is_generated --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins12-missing-manifests-0622 --message-format short --color never -- --test-threads=1: 1 passed, 0 failed on 2026-06-22 with existing zircon_runtime warnings
  - static plugin manifest required-field schema scan over generated and native zircon_plugins/**/plugin.toml: 37/37 passed 2026-06-22
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12_manifest_schema_uniform --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins12-missing-manifests-0622 --message-format short --color never -- --test-threads=1: 1 passed, 0 failed on 2026-06-22 with existing zircon_runtime warnings
  - python tools/audit_plugin_structure.py --json: passed 2026-06-22 with m1_gate_status classified-and-clear, missing_plugin_toml 0, manifest_schema_violations 0, expected_manifest_count 37
  - python tools/audit_plugin_structure.py --json: passed 2026-06-22 with skeleton sample_conformance_status sample-clean, sample_violation_count 0, migration_debt_count 35
  - python -m py_compile tools/audit_plugin_structure.py tools/plugin_structure_audits/__init__.py tools/plugin_structure_audits/manifest_schema.py: passed 2026-06-22
  - python -m py_compile tools/audit_plugin_structure.py tools/plugin_structure_audits/__init__.py tools/plugin_structure_audits/manifest_schema.py tools/plugin_structure_audits/skeleton.py: passed 2026-06-22
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor -p zircon_first_party_runtime_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never: passed 2026-06-22 with existing warning noise
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12_crate_skeleton_conformance --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never -- --test-threads=1 --nocapture: timed out after 900s on 2026-06-22, not counted as passing
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12_ --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins12-missing-manifests-0622 --message-format short --color never -- --test-threads=1: 3 passed, 0 failed on 2026-06-22 with existing zircon_runtime warnings
  - rustfmt --edition 2021 --config skip_children=true --check on catalog/descriptor parity touched Rust files: passed 2026-06-22
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12_feature_enabled_runtime_descriptor_manifest_parity --features "base-runtime-plugins advanced-render-runtime-plugins navigation-runtime-plugin zr-vm-language-runtime-plugin" --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins12-feature-catalog-check-0622 --message-format short --color never -- --test-threads=1: 1 passed, 0 failed on 2026-06-22 with existing zircon_runtime warnings
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12 --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins12-feature-catalog-check-0622 --message-format short --color never -- --test-threads=1: 3 passed, 0 failed, 1 filtered out on 2026-06-22 with existing zircon_runtime warnings
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime navigation_registration_contributes_runtime_module_and_components --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins12-feature-catalog-check-0622 --message-format short --color never -- --test-threads=1: 1 passed, 0 failed on 2026-06-22 with existing zircon_runtime warnings
  - python -m unittest tools.tests.test_plugin_structure_audit_manifest_schema tools.tests.test_plugin_structure_audit_registration: passed 2026-07-03, 16 tests
  - python tools\audit_plugin_structure.py --json: passed 2026-07-03 with manifest_count=37, generated_manifest_count=36, manifest_schema_violations=0, capability_source_mismatches=0, runtime_registration_builder_violation_count=0, dist_capable_plugin_count=39, dist_dependency_boundary_violations=0, retired_ui_asset_files=0
  - read-only first-party catalog snapshot script over zircon_plugins/first_party_runtime_catalog/src/lib.rs and zircon_plugins/**/plugin.toml: passed 2026-07-03 with provider_count=13, static_manifest_count=36, plugin_manifest_count=37, generated_manifest_count=36, native_dynamic_fixture hand-written exception present, snapshot_sha256=e16a68cf81d326342c4694cd3e1a6425e8162d503c1b2c8ca578c8e5b30b54e7
  - rustfmt --edition 2021 --check zircon_plugins\first_party_runtime_catalog\src\lib.rs zircon_plugins\first_party_runtime_catalog\src\tests\provider_snapshot.rs: passed 2026-07-03
  - wsl.exe bash -lc "cd /mnt/e/Git/ZirconEngine && cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog --features base-runtime-plugins,advanced-render-runtime-plugins,navigation-runtime-plugin,zr-vm-language-runtime-plugin --locked --jobs 1 --target-dir /home/hejiahui/zircon-targets/frameworks-first-party-catalog-featured-0703 --message-format short --color never -- --nocapture --test-threads=1": passed 2026-07-03, 8/8 tests, doc-tests 0/0
  - wsl.exe bash -lc "cd /mnt/e/Git/ZirconEngine && cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog --features base-runtime-plugins,advanced-render-runtime-plugins,navigation-runtime-plugin,zr-vm-language-runtime-plugin --locked --jobs 1 --target-dir /home/hejiahui/zircon-targets/frameworks-first-party-catalog-featured-0703 --message-format short --color never feature_enabled_first_party_provider_snapshot_reports_compiled_runtime_plugins -- --nocapture --test-threads=1": passed 2026-07-03, 1/1 tests, 7 filtered
  - wsl.exe bash -lc "cd /mnt/e/Git/ZirconEngine && cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog --no-default-features --locked --jobs 1 --target-dir /home/hejiahui/zircon-targets/frameworks-first-party-catalog-0703 --message-format short --color never catalog_without_provider_features_returns_no_registrations -- --nocapture --test-threads=1": passed 2026-07-03 current closeout rerun, 1/1 tests
  - wsl.exe bash -lc "cd /mnt/e/Git/ZirconEngine && cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog --no-default-features --locked --jobs 1 --target-dir /home/hejiahui/zircon-targets/frameworks-first-party-catalog-0703 --message-format short --color never -- --nocapture --test-threads=1": passed 2026-07-03 current closeout rerun, 7/7 tests, doc-tests 0/0, test runtime 136.39s
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

Plugins 12 adds `plugins_12_static_plugin_manifest_is_generated` in this crate because the catalog already owns first-party runtime provider fan-out. The guard requires every covered non-native `plugin.toml` to carry the `@generated` header, including the nested `asset_importers/*` family manifests and `opus_importer`; compares feature-enabled static runtime manifest slices against the corresponding `package_manifest()` descriptor output, including `supported_platforms`; parses multiline TOML arrays used by real plugin manifests; and checks that `native_dynamic_fixture` embeds its single hand-written root manifest through `include_str!` instead of carrying an inline duplicate.

Plugins 12 also adds `plugins_12_manifest_schema_uniform` and `plugins_12_manifest_schema_uniform_audit_report_is_clean` as T4 guards. The direct guard checks the 36 generated non-native manifests plus the native hand-written manifest for the fixed required schema fields and module fields. The audit-report guard runs `tools/audit_plugin_structure.py --json` and checks the machine-readable M1 fields: `missing_plugin_toml = 0`, `manifest_schema_violations = 0`, `expected_manifest_count = 37`, and `m1_gate_status = classified-and-clear`.

Plugins 12 now also has the explicit feature-gated guard `plugins_12_feature_enabled_runtime_descriptor_manifest_parity`. When the catalog is built with `base-runtime-plugins`, `advanced-render-runtime-plugins`, `navigation-runtime-plugin`, and `zr-vm-language-runtime-plugin`, it compares every linked provider descriptor manifest against the generated static manifest for category, maturity, targets, platforms, capabilities, default packaging, and runtime modules. This closes descriptor/static manifest parity for linked first-party runtime catalog providers; full capability four-source convergence remains the Plugins 12 M4 audit.

Plugins 12 M2/T2 adds `plugins_12_crate_skeleton_conformance`. The guard runs the Python audit report and asserts the blessed `plugin_sdk_examples` sample root is clean: one expected sample, one conforming sample, zero sample violations, and `plugin_skeleton_gate.m2_gate_status = sample-clean-migration-debt-present`. The remaining migration-debt roots are tracked by count and root list for M5 touch-it-conform-it, not treated as a failure of this sample gate.

Runtime 06 / F8 now requires first-party runtime plugin descriptor production files to use the
`RuntimePluginDescriptor::builder(...).build()` construction path. The migrated production set is
`ai`, `animation`, `hybrid_gi`, `navigation`, `net`, `particles`, `physics`, `prefab_tools`,
`rendering`, `solari`, `sound`, `terrain`, `texture`, `tilemap_2d`, `virtual_geometry`, and
`zr_vm_language`: first-party runtime plugin descriptor production files 16/16. The structure guard
`review_f8_first_party_runtime_plugin_descriptors_use_builder` rejects old
`RuntimePluginDescriptor::new(` in those production owners while allowing runtime/plugin extension
test fixtures to migrate in a separate slice. Status:
`runtime_plugin_descriptor_first_party_builder_migration_coremin_check_passed`; RuntimePluginDescriptor
test fixture migration remains pending.

Frameworks 02 M3 adds a low-interference catalog/source snapshot while runtime/editor Cargo lanes are
busy. The static provider match order in `first_party_registration_for_runtime_plugin(...)` is:
`Ai`, `Sound`, `Texture`, `Net`, `Navigation`, `Particles`, `Animation`, `Rendering`,
`GltfImporter`, `VirtualGeometry`, `HybridGi`, `Solari`, `ZrVmLanguage`. The same snapshot verifies
that the catalog test array covers 36 generated non-native manifests, the plugin tree still contains
37 manifests total, and `native_dynamic_fixture` remains the single hand-written native manifest
exception. Snapshot SHA256:
`e16a68cf81d326342c4694cd3e1a6425e8162d503c1b2c8ca578c8e5b30b54e7`. This evidence does not replace
feature-enabled catalog Cargo tests or full plugin workspace build/test.

Frameworks 02 M3 now also has feature-enabled provider execution coverage in
`src/tests/provider_snapshot.rs`. The test is mounted from the crate test module only when one of
the provider feature groups is active, then constructs a client-runtime project manifest and calls
`first_party_runtime_plugin_registrations_for_manifest(...)`. This executes the compiled
first-party provider `plugin_registration()` functions, asserts the 13-provider package order,
checks `ProjectPluginSelection.id` against the returned package manifest id, and requires every
registration report to carry an empty diagnostic list.

The snapshot owner is deliberately outside `src/lib.rs`: adding it inline pushed the crate root to
roughly 1090 lines, so the test was split to keep the root structural. After the split,
`src/lib.rs` is 920 lines and `src/tests/provider_snapshot.rs` is 126 lines. Fresh WSL validation
passed the feature-enabled catalog crate gate 8/8 with doc-tests 0/0, including the provider
snapshot. The current WSL no-default closeout reruns also pass: the focused
`catalog_without_provider_features_returns_no_registrations` regression passes 1/1, and the full
no-default crate passes 7/7 with doc-tests 0/0. The earlier `sdf_fallback.rs` `E0689` compile
blocker was removed by the active runtime text/SDF owner; this catalog session did not edit that
file. This closes first-party provider execution and current no-default catalog regression for this
catalog gate only; it does not claim full
`zircon_plugins` workspace build/test, host `NativePluginLoader`, public wrapper artifact build, or
runtime/editor integration.
