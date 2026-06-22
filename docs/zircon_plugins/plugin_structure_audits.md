---
related_code:
  - audit_plugin_structure.py
  - plugin_structure_audits/__init__.py
  - plugin_structure_audits/manifest_schema.py
  - plugin_structure_audits/skeleton.py
  - plugin_structure_audits/registration.py
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/**/plugin.toml
  - zircon_plugins/plugin_sdk_examples/editor/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/capability.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extensions.rs
  - zircon_plugins/asset_importers/data/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/model/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/shader/runtime/src/plugin.rs
  - zircon_plugins/gltf_importer/runtime/src/plugin.rs
  - zircon_plugins/obj_importer/runtime/src/plugin.rs
  - zircon_plugins/texture_importer/runtime/src/plugin.rs
  - zircon_plugins/audio_importer/runtime/src/plugin.rs
  - zircon_plugins/opus_importer/runtime/src/plugin.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/plugin.rs
  - zircon_plugins/ui_document_importer/runtime/src/plugin.rs
implementation_files:
  - audit_plugin_structure.py
  - plugin_structure_audits/__init__.py
  - plugin_structure_audits/manifest_schema.py
  - plugin_structure_audits/skeleton.py
  - plugin_structure_audits/registration.py
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/capability.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extensions.rs
  - zircon_plugins/gltf_importer/runtime/src/plugin.rs
  - zircon_plugins/obj_importer/runtime/src/plugin.rs
  - zircon_plugins/texture_importer/runtime/src/plugin.rs
  - zircon_plugins/audio_importer/runtime/src/plugin.rs
  - zircon_plugins/opus_importer/runtime/src/plugin.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/plugin.rs
  - zircon_plugins/ui_document_importer/runtime/src/plugin.rs
plan_sources:
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python audit_plugin_structure.py --json: classified-and-clear, missing_plugin_toml=0, manifest_schema_violations=0, expected_manifest_count=37
  - python audit_plugin_structure.py --json: sample_conformance_status=sample-clean, sample_expected_count=1, sample_violation_count=0, migration_debt_count=35, migration_debt_details_truncated=true on 2026-06-22
  - python -m py_compile audit_plugin_structure.py plugin_structure_audits/__init__.py plugin_structure_audits/manifest_schema.py plugin_structure_audits/skeleton.py: passed 2026-06-22
  - python -m py_compile audit_plugin_structure.py plugin_structure_audits/__init__.py plugin_structure_audits/manifest_schema.py plugin_structure_audits/skeleton.py plugin_structure_audits/registration.py: passed 2026-06-23
  - python audit_plugin_structure.py --json: registration_conformance.m3_t1_gate_status=family-single-entry-clean, asset_importer_family_free_function_registration_sites=0, asset_importer_family_registration_owner_files=0 on 2026-06-23
  - python audit_plugin_structure.py --json: registration_conformance.m3_split_importer_gate_status=split-importer-single-entry-clean, split_importer_free_function_registration_sites=0, split_importer_registration_owner_files=0, m3_importer_gate_status=importer-single-entry-clean on 2026-06-23
  - rustfmt --edition 2021 --config skip_children=true --check zircon_plugins/plugin_sdk_examples/editor/src/*.rs zircon_plugins/first_party_runtime_catalog/src/lib.rs: passed 2026-06-22
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor -p zircon_first_party_runtime_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never: passed 2026-06-22 with existing warning noise
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12_crate_skeleton_conformance --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never -- --test-threads=1 --nocapture: timed out after 900s on 2026-06-22, not counted as passing
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12 --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins12-missing-manifests-0622 --message-format short --color never -- --test-threads=1: 3 passed, 0 failed on 2026-06-22 with existing zircon_runtime warnings
doc_type: module-detail
---

# Plugin Structure Audits

`audit_plugin_structure.py` is the Plugins 12 structure-audit entry point. It mirrors the runtime audit pattern with a plugin-focused audit package under `plugin_structure_audits/`.

The first landed audit is `plugin_manifest_schema_uniform`. It derives expected plugin roots from `zircon_plugins/Cargo.toml`, skips workspace support crates that are not plugins (`editor_support`, `first_party_runtime_catalog`, and `plugin_sdk`), folds `features/*` members back to their parent plugin, and treats `asset_importers/<kind>/runtime` as separate importer plugin roots. The current expected manifest set is 37 roots.

Plugins 12 M2/T2 adds `skeleton_conformance`. It uses the same expected plugin root set, checks the first blessed sample root `plugin_sdk_examples`, and classifies non-sample violations as migration debt instead of failing the M2 sample gate.

Plugins 12 M3/T1 adds `registration_conformance` for both importer tracks. The first slice covered the `asset_importers/*` family; the follow-up extends the same scan to root-level split importer packages. It scans runtime source files outside tests for public `pub fn register(...)` free functions and for `runtime/src/registration.rs` owner files.

The audit reports:

- `missing_plugin_toml`
- `manifest_schema_violations`
- `generated_manifest_header_violations`
- `skeleton_conformance.sample_conformance_status`
- `skeleton_conformance.sample_roots`
- `skeleton_conformance.migration_debt_roots`
- `plugin_skeleton_gate.m2_gate_status`
- `registration_conformance.m3_t1_gate_status`
- `registration_conformance.asset_importer_family_free_function_registration_sites`
- `registration_conformance.asset_importer_family_registration_owner_files`
- `registration_conformance.m3_split_importer_gate_status`
- `registration_conformance.split_importer_free_function_registration_sites`
- `registration_conformance.split_importer_registration_owner_files`
- `registration_conformance.m3_importer_gate_status`
- `capability_source_mismatches = "pending"`

The capability field is intentionally pending because capability four-source convergence belongs to Plugins 12 M4. M1/T4 only claims `missing_plugin_toml = 0` and `manifest_schema_violations = 0`. M2/T2 only claims that the blessed sample is clean; the current 35 migration-debt roots remain scheduled for M5 touch-it-conform-it.

`skeleton_conformance.migration_debt_details` is intentionally capped to the first 64 detail rows and paired with `migration_debt_detail_count` plus `migration_debt_details_truncated`. Counts and root names remain complete, while the JSON stays usable for command-line validation.

The first-party runtime catalog carries a separate feature-enabled descriptor/static manifest parity guard for linked runtime providers. That guard catches category, maturity, target, platform, capability, default packaging, and runtime-module drift between Rust descriptors and generated static manifests, but it does not turn `capability_source_mismatches` into a full plugin-wide four-source audit.

The current M3/T1 registration gates are clean for the importer scopes covered so far: `asset_importers/{data,model,shader}/runtime/src/plugin.rs` are the family trait-backed entries, split importers now use `runtime/src/plugin.rs` entries, importer free-function registration sites are zero, and there are no non-test `runtime/src/registration.rs` owner files in either importer track. This does not close full skeleton conformance or capability single-source migration.

`zircon_first_party_runtime_catalog::tests::plugins_12_manifest_schema_uniform_audit_report_is_clean` runs the script in JSON mode and asserts the M1 gate fields stay clean. `plugins_12_crate_skeleton_conformance` consumes the same report and asserts `plugin_sdk_examples` is the clean sample while migration debt is explicitly counted. The catalog keeps the Rust-only manifest parser tests as a second guard for descriptor parity and native manifest single-source checks.
