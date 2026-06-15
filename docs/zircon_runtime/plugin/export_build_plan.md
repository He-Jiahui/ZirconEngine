---
related_code:
  - zircon_runtime/src/asset/project/manifest.rs
  - zircon_runtime/src/plugin/export_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest/profile_projection.rs
  - zircon_runtime/src/plugin/export_build_plan/export_validate_report.rs
  - zircon_runtime/src/plugin/export_build_plan/library_embed_compile_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/source_template_build_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/native_dynamic_package_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/native_plugin_load_manifest_template.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize.rs
  - zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_manifest.rs
  - zircon_runtime/src/bin/zircon_export_validate/main.rs
  - zircon_runtime/src/bin/zircon_export_validate/args.rs
  - zircon_runtime/src/bin/zircon_export_validate/run.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_native_dynamic.rs
  - zircon_runtime/src/asset/tests/project/manifest.rs
implementation_files:
  - zircon_runtime/src/asset/project/manifest.rs
  - zircon_runtime/src/plugin/export_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest/profile_projection.rs
  - zircon_runtime/src/plugin/export_build_plan/export_validate_report.rs
  - zircon_runtime/src/plugin/export_build_plan/library_embed_compile_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/source_template_build_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/native_dynamic_package_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/native_plugin_load_manifest_template.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize.rs
  - zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_manifest.rs
  - zircon_runtime/src/bin/zircon_export_validate/main.rs
  - zircon_runtime/src/bin/zircon_export_validate/args.rs
  - zircon_runtime/src/bin/zircon_export_validate/run.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_native_dynamic.rs
  - zircon_runtime/src/asset/tests/project/manifest.rs
plan_sources:
  - docs/plans/zircon_plugins/09-export-publishing.md
  - docs/plans/zircon_plugins/index.md
tests:
  - export_profile_map_table_parses_planned_profile_asset_fields
  - profile_with_features_compiles_to_build_plan
  - invalid_plugin_combination_rejected_with_diagnostic
  - validate_report_summarizes_profile_plan_and_fatal_state
  - feature_matrix_links_selected_plugins_only
  - source_template_profile_carries_build_validation_plan
  - native_dynamic_generates_loader_manifest_without_source_template
  - validate_report_exposes_native_dynamic_abi_v3_package_exports
  - loader_manifest_deserializes_abi_v3_contract_fields
  - native_dynamic_materialization_copies_runtime_package_without_source_crates
  - export_wizard_compile_host_path_feeds_platform_bundle_host_input
  - export_wizard_compile_host_path_respects_target_dir_override_and_build_mode
  - export_wizard_default_host_executable_points_to_compile_host_output
doc_type: module-detail
---

# Export Build Plan

`zircon_runtime::plugin::export_build_plan` converts a project manifest and export profile into the
build-time projection used by SourceTemplate, LibraryEmbed, and NativeDynamic packaging paths. It
owns plan-time diagnostics and generated source metadata, but does not run Cargo or package assets
itself.

## Profile Asset Shape

`ProjectManifest` accepts both legacy `[[export_profiles]]` list tables and the planned
`[export_profiles.<name>]` asset form. The map-table form fills `ExportProfile.name` from the table
key when the profile body omits it.

`ExportProfile` keeps the old fields while accepting the 09 plan aliases:

- `platform` aliases `target_platform`, including `windows-x86_64`, `linux-x86_64`, and
  `macos-aarch64` for the current desktop matrix.
- `path` aliases `strategies`; a single value such as `library_embed` becomes a one-entry strategy
  list.
- `mode` records `ExportBuildMode` and serializes under the same public field name.
- `plugins` records profile-level plugin selection as `selected_plugins` internally.
- `features` is a per-owner map of selected optional feature ids.
- `asset_filter` is stored on the profile for the later asset-pack/cook stage.

## Projection Rules

`from_project_manifest/profile_projection.rs` runs after builtin catalog completion and before the
existing export build-plan dependency/link logic.

The projection is a profile-level view over the project plugin manifest:

- when `plugins` is non-empty, plugins not listed by the profile are disabled in the projected
  manifest;
- features belonging to disabled plugins are disabled as well so they cannot leak blocked-feature
  diagnostics through the dependency resolver;
- `features.<plugin>` narrows that owner's enabled feature set and accepts either short ids
  (`http`) or fully-qualified ids (`net.http`);
- selected plugin rows retain catalog-filled crate names, target modes, packaging defaults, and
  optional-feature metadata.

Diagnostics are produced before projection so required project intent is not lost. Unknown selected
plugins, unknown selected features, features selected for an unselected plugin, and profile removal
of a required plugin or required feature are reported as fatal export-plan diagnostics.

## Validate Report

`export_validate_report.rs` is the shared report DTO consumed by the M1 `zircon_export` CLI. It
keeps the runtime crate as the owner of export diagnostics while allowing process-level tools to
serialize the result without reimplementing validation.

The shared `ExportPipelineStage` enum now mirrors the full export CLI pipeline:
`Validate`, `CompileHost`, `SourceTemplate`, `CookAssets`, `Pack`, `PlatformBundle`, and `Report`.
Only the Validate binary writes `ExportValidateReport` today, but editor tooling and later report
aggregation can use the same stage identities instead of maintaining a separate editor-only enum.

`ExportValidateReport::from_build_plan(...)` records:

- `stage = Validate`, the project manifest path, profile name, stage output directory, profile
  presence, and fatal state.
- de-duplicated `diagnostics` and `effective_fatal_diagnostics`.
- a profile summary containing target mode, target platform, build mode, strategies, selected
  plugins, selected features, and asset filter.
- a plan summary containing enabled runtime plugins, linked runtime crates, native dynamic
  packages, NativeDynamic ABI v3 package exports, generated file metadata plus generated contents
  for SourceTemplate materialization, optional LibraryEmbed/SourceTemplate build plans, and runtime
  plugin availability.

The `zircon_export_validate` binary is a thin shell around this DTO. It loads `ProjectManifest`,
calls `ExportBuildPlan::from_project_manifest`, writes optional `report.json`, prints the same JSON
to stdout, and exits with code `2` when the report is fatal.

## LibraryEmbed CompileHost Plan

`library_embed_compile_plan.rs` lifts the M2 LibraryEmbed path from an implicit linked-crate list to
an explicit `CompileHost` plan. The plan is still declarative: it does not run Cargo from
`zircon_runtime`, but it tells the external CLI exactly which host package, binary, feature set,
target directory, and plugin crates should be used.

For a client-runtime profile the command model is:

```text
cargo build -p zircon_app --bin zircon_runtime --no-default-features --features target-client --target-dir stages/compile_host/target
```

Release profiles append `--release`. Server profiles use `target-server`, while editor-host profiles
use `zircon_editor` plus `target-editor-host`. Each linked crate row records whether it is a runtime
plugin registration or a runtime feature registration, preserving provider package ids for external
feature providers.

`LibraryEmbedCompileHostPlan::binary_for_target_mode(...)` and
`cargo_profile_for_build_mode(...)` expose the same mapping used by the declarative plan. Editor
export wizard code uses these helpers to derive the PlatformBundle handoff path
`<out>/stages/compile_host/target/<cargo-profile>/<binary>[.exe]` without copying a separate
runtime-target-mode table or falling back to a placeholder executable name. The file still belongs
to the external CompileHost stage; this runtime module only owns the validated planning rules.

`ExportBuildPlan.library_embed_compile_host` is populated only when the profile enables
`LibraryEmbed`. `ExportValidateReport.plan_summary.library_embed_compile_host` exposes the same
plan to `python -m zircon_export --stage validate`, so the later `CompileHost` stage can compare
what it executes against the validated plan.

## SourceTemplate Build Validation Plan

`source_template_build_plan.rs` adds the M4 SourceTemplate validation hook. The plan is declarative
like CompileHost: runtime code still does not spawn Cargo, but it records the generated project's
manifest path, stage-local target directory, debug/release profile, and canonical `cargo build`
command. `ExportValidateReport.plan_summary.source_template_build` exposes that command to
`python -m zircon_export --stage source_template`.

SourceTemplate and LibraryEmbed now share the linked-Rust dependency projection. A SourceTemplate-only
profile therefore still receives external runtime plugin and feature crates in the generated
`Cargo.toml`; it no longer needs to also select LibraryEmbed just to make generated registration
calls compile.

## NativeDynamic ABI v3 Package Plan

`native_dynamic_package_plan.rs` owns package-id de-duplication, sanitized package directories, and
the export-time ABI v3 package contract. `ExportBuildPlan.native_dynamic_packages` remains as the
string-list compatibility field, while `native_dynamic_package_exports` carries each package's id,
directory, staged path, manifest path, and ABI contract.

The generated `plugins/native_plugins.toml` now writes `package_report` and `[plugins.abi]` for
each `[[plugins]]` row. `NativePluginLoadManifestEntry` can deserialize those optional fields
without changing existing discovery/load behavior, so older loader paths can ignore the package
contract while tooling can audit it.

`materialize_with_native_packages(...)` writes a `native_dynamic_package.toml` report beside each
copied native package. The materializer also falls back to deriving this report from the old
`native_dynamic_packages` string list, which keeps deserialized older build plans usable.

## Current Coverage

The M1 export profile slice adds four focused regressions:

- `export_profile_map_table_parses_planned_profile_asset_fields` covers TOML map-table parsing and
  plan-field aliases.
- `profile_with_features_compiles_to_build_plan` covers profile-level plugin and feature trimming
  into linked runtime crates.
- `invalid_plugin_combination_rejected_with_diagnostic` covers fatal diagnostics when a profile
  excludes a required plugin.
- `validate_report_summarizes_profile_plan_and_fatal_state` covers the shared Validate report
  shape used by the CLI.
- `feature_matrix_links_selected_plugins_only` covers M2-T1 LibraryEmbed CompileHost planning:
  selected plugins/features produce linked crate rows, unselected plugins/features are trimmed, and
  the planned command targets `zircon_app` with the expected runtime feature and release flag.
- `source_template_profile_carries_build_validation_plan` covers M4-T1 SourceTemplate planning:
  SourceTemplate-only profiles link selected runtime crates, carry a generated-project build
  command, and expose that command in the Validate report.
- `native_dynamic_generates_loader_manifest_without_source_template` covers M5-T1 loader manifest
  generation with `package_report` and ABI v3 contract fields.
- `validate_report_exposes_native_dynamic_abi_v3_package_exports` covers Validate report exposure of
  structured native package exports while preserving the old package-id list.
- `loader_manifest_deserializes_abi_v3_contract_fields` covers deserialization of the optional
  loader manifest ABI contract.
- `native_dynamic_materialization_copies_runtime_package_without_source_crates` covers copied native
  package layout plus the generated per-package `native_dynamic_package.toml`.
- `export_pipeline_stage_parser_accepts_cli_and_report_stage_names` in the desktop export editor
  plugin covers the public seven-stage enum from the editor-facing stream parser.
- `export_wizard_compile_host_path_feeds_platform_bundle_host_input`,
  `export_wizard_compile_host_path_respects_target_dir_override_and_build_mode`, and
  `export_wizard_default_host_executable_points_to_compile_host_output` cover the editor-side reuse
  of the runtime CompileHost binary/profile rules for PlatformBundle host handoff paths.

2026-06-14 validation: `rustfmt --edition 2021 --check`, conflict-marker scan, and
`git diff --check` passed for the touched export profile/build-plan/project-manifest files; diff
check only reported LF/CRLF notices. `cargo check -p zircon_runtime --lib --no-default-features
--features core-min --offline --jobs 1` and focused `cargo test` attempts timed out during current
runtime compilation pressure before returning Rust diagnostics, so no Cargo pass is claimed for
this slice yet.

2026-06-14 M1-T3 update: the Validate CLI/report slice adds Python smoke coverage (`py_compile`,
`python -m zircon_export --help`, and dry-run Validate command rendering) plus Rust formatting/static
checks for the report and validator sources. The M1 testing stage passed
`cargo check -p zircon_runtime --bin zircon_export_validate --locked --offline --jobs 1
--target-dir D:\cargo-targets\zircon-export-m1-validate-0614`, and a real
`python -m zircon_export --profile windows-release --stage validate` run wrote a non-fatal
`report.json` containing the expected rendering and net feature linked crates. Focused
`cargo test -p zircon_runtime --lib profile_with_features_compiles_to_build_plan --locked --offline`
is still blocked before target-test execution by unrelated UI test compile drift in
`zircon_runtime/src/ui/tests/runtime_input_reply_routes/table_pointer_routes.rs` (`capture_started`
and `capture_released` fields are missing from `UiInputDispatchDiagnostics`).

2026-06-14 M2-T1 update: `rustfmt --edition 2021 --check`, `git diff --check`, conflict-marker
scan, and `python -m zircon_export ... --dry-run` passed after adding the LibraryEmbed CompileHost
plan. `cargo check -p zircon_runtime --bin zircon_export_validate --locked --offline --jobs 1
--target-dir D:\cargo-targets\zircon-export-m1-validate-0614` also passed with existing runtime
warnings, proving the Validate binary still compiles with the new compile-plan DTO. The focused
runtime lib-test remains blocked by the unrelated UI compile drift recorded above.

2026-06-14 M4-T1 update: SourceTemplate planning now emits
`source_template_build` and Validate report generated-file rows include `contents` so the external
CLI can materialize the generated project without duplicating Rust template logic. `rustfmt
--edition 2021 --check` and Python SourceTemplate stage tests passed. A scoped
`cargo check -p zircon_runtime --bin zircon_export_validate --locked --offline --jobs 1
--target-dir D:\cargo-targets\zircon-export-m4-source-template-0614` attempt timed out after 244
seconds without Rust diagnostics, so no new Cargo pass is claimed for this slice.

2026-06-14 M5-T1 update: NativeDynamic planning now emits structured ABI v3 package exports,
loader-manifest ABI rows, and per-package `native_dynamic_package.toml` reports. `rustfmt
--edition 2021 --check`, `git diff --check`, conflict-marker scan, trailing-whitespace scan, and a
Python TOML parse check passed for the touched M5 files. A scoped
`cargo check -p zircon_runtime --bin zircon_export_validate --locked --offline --jobs 1
--target-dir D:\cargo-targets\zircon-export-m5-native-dynamic-0614` passed with existing warning
noise, and lockfiles stayed unchanged.

2026-06-14 M6 follow-up update: `ExportPipelineStage` now carries the full seven-stage
pipeline for editor progress parsing. A scoped
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-0614`
passed with existing warnings, proving the extended enum compiles through the editor plugin
consumer. The focused editor plugin `cargo test ... export_wizard ...` command timed out after
604 seconds before target test results.

2026-06-15 M6 handoff update: the CompileHost plan exposes binary/profile lookup helpers and the
editor wizard now projects those rules into the default PlatformBundle `--host-executable` path.
`rustfmt --edition 2021` was applied to the touched runtime/editor files. Scoped validation passed
with existing warnings for
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
and
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`.
Focused `cargo test -p zircon_editor export_wizard_compile_host_path --lib ...` timed out twice
during lib-test compilation without target output; no focused handoff test pass is claimed.
