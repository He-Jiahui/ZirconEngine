---
related_code:
  - zircon_plugins/plugin_sdk_examples/plugin.toml
  - zircon_plugins/plugin_sdk_examples/editor/Cargo.toml
  - zircon_plugins/plugin_sdk_examples/editor/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/capability.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extensions.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extension_ids.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/tests.rs
  - zircon_plugins/plugin_sdk/src/manifest/package_builder.rs
  - zircon_plugins/plugin_sdk/src/editor.rs
  - tools/plugin_structure_audits/skeleton.py
implementation_files:
  - zircon_plugins/plugin_sdk_examples/editor/Cargo.toml
  - zircon_plugins/plugin_sdk_examples/editor/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/capability.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extensions.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extension_ids.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/tests.rs
plan_sources:
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - rustfmt --edition 2021 --config skip_children=true --check zircon_plugins/plugin_sdk_examples/editor/src/lib.rs zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs zircon_plugins/plugin_sdk_examples/editor/src/capability.rs zircon_plugins/plugin_sdk_examples/editor/src/extensions.rs zircon_plugins/plugin_sdk_examples/editor/src/extension_ids.rs zircon_plugins/plugin_sdk_examples/editor/src/tests.rs: passed 2026-06-22
  - python tools/audit_plugin_structure.py --json: skeleton sample_conformance_status=sample-clean, sample_expected_count=1, sample_violation_count=0 on 2026-06-22
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor -p zircon_first_party_runtime_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never: passed 2026-06-22 with existing warning noise
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never -- --test-threads=1: timed out after 1200s on 2026-06-22, not counted as passing
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-editor-sdk-m2-0622 --message-format short --color never: passed 2026-06-22 with existing zircon_runtime/zircon_editor warnings
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-editor-sdk-m2-0622 --message-format short --color never -- --test-threads=1: timed out after 300s on 2026-06-22, no test binary, not counted as passing
doc_type: module-detail
status: in_progress
---

# Plugin SDK Examples Editor

`zircon_plugin_sdk_examples_editor` is the first Plugins 12 M2/T2 skeleton-conformance sample. It demonstrates how an editor plugin crate should keep its root façade thin while moving ownership into named modules.

## Module Ownership

- `src/lib.rs` is the public façade. It declares modules and re-exports stable constants, plugin constructors, and report helpers.
- `src/capability.rs` owns the editor capability constants for the example package.
- `src/extension_ids.rs` owns view, importer, template, asset-kind, and component identifiers used by the sample extension registrations.
- `src/extensions.rs` owns `EditorExtensionRegistry` mutation: example window, menu operations, model importer, asset editor, UI templates, asset creation template, and inspector customization.
- `src/plugin.rs` owns the editor plugin types, consumes `zircon_plugin_sdk::editor::authoring_plugin!` for the primary editor plugin, and keeps fixture-specific extension registration forwarding plus registration report helpers.
- `src/tests.rs` owns behavior tests for contributed editor extensions and SDK fixture metadata.

## SDK Boundary

The sample uses `zircon_plugin_sdk::editor::authoring_plugin!` to generate the primary editor plugin struct, descriptor access, manifest projection, capability list, and registration report helper. It does not manually add the editor module: `zircon_editor::EditorPlugin::package_manifest(...)` attaches that module from the descriptor owner, preventing duplicate module declarations.

`editor/Cargo.toml` now uses workspace dependency inheritance for the core path dependencies:

- `zircon_editor.workspace = true`
- `zircon_plugin_sdk = { workspace = true, features = ["editor"] }`
- `zircon_runtime.workspace = true`

## Guard Contract

`plugin_structure_audits::skeleton` treats `plugin_sdk_examples` as the first blessed sample root. The M2/T2 gate requires:

- `sample_conformance_status = sample-clean`
- `sample_expected_count = 1`
- `sample_conforming_count = 1`
- `sample_violation_count = 0`
- `sample_workspace_dependency_status = sample-workspace-deps-clean`
- `sample_workspace_dependency_violation_count = 0`

Other plugin roots remain migration debt until Plugins 12 M5 touch-it-conform-it slices move them into the same skeleton.
