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
  - zircon_runtime/src/plugin/export_build_plan/materialize/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/archive.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/generated.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/paths.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/native.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/package_lookup.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/copy.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/report.rs
  - zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_manifest.rs
  - zircon_runtime/src/bin/zircon_export_validate/main.rs
  - zircon_runtime/src/bin/zircon_export_validate/args.rs
  - zircon_runtime/src/bin/zircon_export_validate/error.rs
  - zircon_runtime/src/bin/zircon_export_validate/run.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan/catalog_projection.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan/profile_feature_matrix.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform/browser_hosts.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform/release_adapters.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_native_dynamic.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/export_build_plan.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/export_build_plan_platform.rs
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
  - zircon_runtime/src/plugin/export_build_plan/materialize/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/archive.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/generated.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/paths.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/native.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/package_lookup.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/copy.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/report.rs
  - zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_manifest.rs
  - zircon_runtime/src/bin/zircon_export_validate/main.rs
  - zircon_runtime/src/bin/zircon_export_validate/args.rs
  - zircon_runtime/src/bin/zircon_export_validate/error.rs
  - zircon_runtime/src/bin/zircon_export_validate/run.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan/catalog_projection.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan/profile_feature_matrix.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform/browser_hosts.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform/release_adapters.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_native_dynamic.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/export_build_plan.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/export_build_plan_platform.rs
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
  - runtime_15_export_build_plan_tests_are_folder_backed
  - source_template_preserves_builtin_catalog_target_modes_after_manifest_completion
  - source_template_links_rendering_default_owner_features
  - source_template_with_native_dynamic_merges_native_loader_reports
  - runtime_15_export_build_plan_platform_tests_are_folder_backed
  - generated_browser_hosts_instantiate_wasm_exports_and_gate_asset_origins
  - native_dynamic_generates_loader_manifest_without_source_template
  - validate_report_exposes_native_dynamic_abi_v3_package_exports
  - loader_manifest_deserializes_abi_v3_contract_fields
  - native_dynamic_materialization_copies_runtime_package_without_source_crates
  - native_dynamic_zip_archive_materialization_writes_generated_files_and_runtime_payloads
  - native_dynamic_zip_archive_preview_reports_archive_without_writes
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

Runtime 15 F5 export CLI typed errors (`runtime_15_export_cli_typed_errors_static_passed_cargo_deferred`)
keeps that shell thin while giving it a typed internal error boundary. `zircon_export_validate/error.rs`
owns `ExportValidateError` / `ExportValidateResult`; argument usage, report JSON encoding, report
directory creation, and report file writes are no longer transported as `Result<_, String>`.
Project-manifest load and build-plan validation failures still become `ExportValidateReport`
diagnostics, because those are user-facing stage-report fields rather than internal Rust errors.
`review_f5_export_cli_uses_typed_errors_before_cli_boundary` locks the boundary together with the
Pack binary.

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
plan to `python -m tools.zircon_export --stage validate`, so the later `CompileHost` stage can compare
what it executes against the validated plan.

## SourceTemplate Build Validation Plan

`source_template_build_plan.rs` adds the M4 SourceTemplate validation hook. The plan is declarative
like CompileHost: runtime code still does not spawn Cargo, but it records the generated project's
manifest path, stage-local target directory, debug/release profile, and canonical `cargo build`
command. `ExportValidateReport.plan_summary.source_template_build` exposes that command to
`python -m tools.zircon_export --stage source_template`.

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

## Materialization Owner

`materialize/mod.rs` owns the public `ExportBuildPlan::write_generated_files(...)`,
`materialize(...)`, `materialize_with_native_packages(...)`, `preview_materialize(...)`,
`preview_materialize_with_native_packages(...)`, `materialize_zip_archive(...)`, and
`preview_zip_archive(...)` entry points. The implementation is folder-backed:
`archive.rs` writes or previews a ZIP archive, `generated.rs` writes or previews generated files,
`paths.rs` validates materialized relative paths,
`native.rs` orchestrates NativeDynamic package copying and preview, `package_lookup.rs` discovers
package sources under the configured plugin root, `copy.rs` filters copied package payloads and
provides no-write package diagnostics, and `report.rs` writes the per-package ABI report.

Generated export file paths are treated as portable export-relative paths, not host filesystem
paths. The resolver rejects empty paths, absolute/root/prefix paths, `.` or `..` components,
trailing separators, and backslash separators before creating parent directories or writing file
contents. This keeps the directory-first materializer aligned with archive-container path traversal
requirements and is reused for ZIP entry names.

NativeDynamic source discovery and payload copy also use a non-following filesystem boundary.
`package_lookup.rs` only traverses real directories and only reads real `plugin.toml` files, so
symlinked package roots or manifest files cannot stand in for a package under the configured plugin
root. `copy.rs` skips symlinked top-level package entries, resource children, and native artifact
children; top-level skipped payloads are reported as materialization diagnostics instead of being
published into the runtime distribution.

The preview entry points return the same `ExportMaterializeReport` shape without creating
directories, writing generated files, copying package payloads, or writing package reports. In
preview mode, `generated_files` and `copied_packages` are planned output paths, while diagnostics
come from the same generated-path resolver, NativeDynamic source lookup, duplicate output-directory
check, native artifact scan, and symlink boundary used by the mutating materializer.

Mutating materialization now preflights `ExportBuildPlan::effective_fatal_diagnostics()` before
performing filesystem writes. `materialize(...)` returns an empty generated/copy set plus a blocking
diagnostic when fatal diagnostics are present, `materialize_with_native_packages(...)` skips
NativeDynamic package copy in that state, and direct `write_generated_files(...)` calls no-op rather
than bypassing the gate. Preview entry points intentionally still list planned paths for editor
preflight and validation UI.

ZIP archive materialization is explicit and separate from directory materialization.
`materialize_zip_archive(plugin_root, archive_path)` writes generated files, copied NativeDynamic
payload files, and each `native_dynamic_package.toml` report into a single archive. Generated file
entries are path-sorted, NativeDynamic payload entries use the same copy eligibility rules as
directory materialization, source crates stay excluded, and `ExportMaterializeReport.archive_file`
records the produced archive path. `preview_zip_archive(...)` returns planned generated/copied rows
and diagnostics without creating the archive or parent directory. Fatal plans return a blocked
archive report and do not create the ZIP.

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
- `source_template_preserves_builtin_catalog_target_modes_after_manifest_completion`,
  `source_template_links_rendering_default_owner_features`, and
  `source_template_with_native_dynamic_merges_native_loader_reports` cover the Runtime 15 M3
  catalog-projection child owner: builtin catalog completion, render default feature providers, and
  SourceTemplate plus NativeDynamic bootstrap projection stay isolated in
  `tests/plugin_extensions/export_build_plan/catalog_projection.rs`.
- `native_dynamic_generates_loader_manifest_without_source_template` covers M5-T1 loader manifest
  generation with `package_report` and ABI v3 contract fields.
- `validate_report_exposes_native_dynamic_abi_v3_package_exports` covers Validate report exposure of
  structured native package exports while preserving the old package-id list.
- `loader_manifest_deserializes_abi_v3_contract_fields` covers deserialization of the optional
  loader manifest ABI contract.
- `native_dynamic_materialization_copies_runtime_package_without_source_crates` covers copied native
  package layout plus the generated per-package `native_dynamic_package.toml`.
- `native_dynamic_zip_archive_materialization_writes_generated_files_and_runtime_payloads` covers
  ZIP archive generation, native loader manifest entries, copied native payloads, package report
  entries, and source-crate exclusion.
- `native_dynamic_zip_archive_preview_reports_archive_without_writes` covers no-write ZIP archive
  preview reporting.
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

2026-06-20 materialization owner split update: `materialize.rs` was hard-cut to
`materialize/{mod,generated,paths,native,package_lookup,copy,report}.rs`. The public
`ExportBuildPlan` materialization methods and report shape remain unchanged, but generated-file
targets now pass through a shared export-relative path resolver before disk writes. Validation for
this slice covered rustfmt check, old flat-file absence, conflict-marker scan, stale old-path scan,
trailing-whitespace scan, and path-scoped `git diff --check`; Cargo and focused behavior tests are
deferred under the implementation-first direction.

2026-06-24 Runtime 15 M3 export build plan test folder split
(`runtime_15_export_build_plan_tests_folder_split_static_passed_cargo_deferred`):
`export_build_plan.rs` now mounts `export_build_plan/catalog_projection.rs`, keeping the
SourceTemplate/LibraryEmbed export-plan fixtures folder-backed. The parent is 723 lines and retains
11 tests plus shared helpers; the child is 263 lines and owns 5 builtin catalog, render feature, and
native merge projection tests.
`runtime_15_export_build_plan_tests_are_folder_backed` locks the parent mount, moved-test
non-regression, total 16-test preservation, per-owner line budgets, and cross-document/status
anchors. Cargo remains deferred under the Runtime 15 implementation-slice cadence and is not
claimed as passing.

2026-07-01 Runtime 15 M3 export build plan profile feature matrix test child-owner split
(`runtime_15_export_build_plan_profile_feature_matrix_tests_child_owner_split_static_passed_cargo_deferred`):
`export_build_plan.rs` now also mounts `export_build_plan/profile_feature_matrix.rs`, moving the
profile feature TOML projection, invalid required-plugin combination, validate-report summary, and
LibraryEmbed feature-matrix tests out of the parent. The parent is 423 lines and retains 7 tests
plus shared helpers; `catalog_projection.rs` is 243 lines with 5 tests, and
`profile_feature_matrix.rs` is 268 lines with 4 tests. The existing
`runtime_15_export_build_plan_tests_are_folder_backed` guard now locks both child mounts,
representative moved-test non-regression, total 16-test preservation, per-owner line budgets, and
cross-document/status anchors. Cargo remains deferred while external cargo/rustc lanes are active
and is not claimed as passing.

2026-06-24 Runtime 15 M3 export build plan platform test folder split
(`runtime_15_export_build_plan_platform_tests_folder_split_static_passed_cargo_deferred`):
`export_build_plan_platform.rs` now mounts `export_build_plan_platform/browser_hosts.rs`, keeping
browser-specific WebGPU/WASM host glue checks out of the platform policy parent. The parent is 780
lines and retains 9 tests; the child is 69 lines and owns the WebAssembly export and allowed
asset-origin gate test. `runtime_15_export_build_plan_platform_tests_are_folder_backed` locks the
parent mount, moved-test non-regression, total 10-test preservation, per-owner line budgets, and
cross-document/status anchors. Cargo remains deferred under the Runtime 15 implementation-slice
cadence and is not claimed as passing.

2026-06-20 NativeDynamic materialization symlink boundary update: package discovery now avoids
symlinked directories and symlinked `plugin.toml` files, and payload copy skips symlinked package,
resource, and native artifact entries. Top-level skipped payloads are surfaced as diagnostics so
export reports can explain omitted files while still avoiding symlink traversal. Validation covered
rustfmt check, static scans for the new non-following helpers, conflict-marker scan,
trailing-whitespace scan, and path-scoped `git diff --check`; Cargo and focused behavior tests remain
deferred under the implementation-first direction.

2026-06-20 materialization preview update: `ExportBuildPlan::preview_materialize(...)` and
`preview_materialize_with_native_packages(...)` now expose a dry-run report that lists planned
generated file paths and planned copied package directories while reusing the same path resolver,
package lookup, duplicate-output, native-artifact, and symlink diagnostics as the mutating
materializer. Validation covered rustfmt check plus static scans proving preview helpers are present
and that filesystem writes/copies remain in the mutating materializer leaves; Cargo and focused
behavior tests remain deferred under the implementation-first direction.

2026-06-20 materialization fatal gate update: mutating materialization now blocks before generated
file writes or NativeDynamic package copies when the effective fatal diagnostic list is non-empty.
The report preserves fatal diagnostics, adds an explicit materialization-blocked diagnostic, and
returns no generated or copied paths; preview remains available as a no-write planning surface.
Validation covered rustfmt check plus static scans for the fatal gate, write/copy call sites,
conflict markers, trailing whitespace, and path-scoped diff checks; Cargo and focused behavior tests
remain deferred under the implementation-first direction.

2026-06-20 ZIP archive materialization update: `materialize/archive.rs` now owns
`ExportBuildPlan::materialize_zip_archive(...)` and `preview_zip_archive(...)`. The runtime manifest
admits only `zip = { version = "9.0.0-pre2", default-features = false, features = ["deflate-flate2"] }`
for this materializer; `tar` remains absent. `ExportMaterializeReport` now carries `archive_file`,
and directory materialization leaves it as `None`. Validation for this implementation slice covered
rustfmt check, direct tech-stack boundary audit, and static path/guard scans; Cargo and focused
behavior tests remain deferred under the implementation-first direction.

2026-06-14 M1-T3 update: the Validate CLI/report slice adds Python smoke coverage (`py_compile`,
`python -m tools.zircon_export --help`, and dry-run Validate command rendering) plus Rust formatting/static
checks for the report and validator sources. The M1 testing stage passed
`cargo check -p zircon_runtime --bin zircon_export_validate --locked --offline --jobs 1
--target-dir D:\cargo-targets\zircon-export-m1-validate-0614`, and a real
`python -m tools.zircon_export --profile windows-release --stage validate` run wrote a non-fatal
`report.json` containing the expected rendering and net feature linked crates. Focused
`cargo test -p zircon_runtime --lib profile_with_features_compiles_to_build_plan --locked --offline`
is still blocked before target-test execution by unrelated UI test compile drift in
`zircon_runtime/src/ui/tests/runtime_input_reply_routes/table_pointer_routes.rs` (`capture_started`
and `capture_released` fields are missing from `UiInputDispatchDiagnostics`).

2026-06-14 M2-T1 update: `rustfmt --edition 2021 --check`, `git diff --check`, conflict-marker
scan, and `python -m tools.zircon_export ... --dry-run` passed after adding the LibraryEmbed CompileHost
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

2026-07-01 Runtime 15 M3 export build plan platform release-adapter test child-owner split
(`runtime_15_export_build_plan_platform_release_adapter_tests_child_owner_split_static_passed_cargo_deferred`):
`export_build_plan_platform.rs` now mounts `export_build_plan_platform/release_adapters.rs`
alongside the existing `export_build_plan_platform/browser_hosts.rs`. The new child owns signing,
store-upload, and CDN upload release adapter validation while the parent keeps platform policy,
host scaffold, package manifest, callback adapter, binding/resource glue, and child mounts.
`runtime_15_export_build_plan_platform_tests_are_folder_backed` locks both child owners,
moved-test non-regression, total 10-test preservation, per-owner line budgets, and
cross-document/status anchors. Cargo remains deferred under the Runtime 15 implementation-slice
cadence and is not claimed as passing.
Scoped rustfmt passed, standalone structure exacts plus production/test global budget exacts passed
3/3, and standalone plan-status passed 42/42.
