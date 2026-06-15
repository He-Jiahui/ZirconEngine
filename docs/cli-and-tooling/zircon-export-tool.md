---
related_code:
  - tools/zircon_export/__init__.py
  - tools/zircon_export/__main__.py
  - tools/zircon_export/cli.py
  - tools/zircon_export/cook_assets.py
  - tools/zircon_export/native_build.py
  - tools/zircon_export/native_dynamic.py
  - tools/zircon_export/native_signing.py
  - tools/zircon_export/pipeline_report.py
  - tools/zircon_export/pipeline_stages.py
  - tools/zircon_export/source_template.py
  - tools/zircon_export/tests/test_platform_bundle_delta.py
  - tools/zircon_export/tests/test_templates.py
  - tools/zircon_export/tests/test_native_dynamic.py
  - export-templates/windows-x86_64-library_embed-debug/template.toml
  - export-templates/linux-x86_64-library_embed-debug/template.toml
  - export-templates/macos-aarch64-library_embed-debug/template.toml
  - zircon_export/__init__.py
  - zircon_export/__main__.py
  - zircon_runtime/src/bin/zircon_export_validate/main.rs
  - zircon_runtime/src/bin/zircon_export_validate/args.rs
  - zircon_runtime/src/bin/zircon_export_validate/run.rs
  - zircon_runtime/src/bin/zircon_export_pack/main.rs
  - zircon_runtime/src/bin/zircon_export_pack/args.rs
  - zircon_runtime/src/bin/zircon_export_pack/manifest.rs
  - zircon_runtime/src/bin/zircon_export_pack/run.rs
  - zircon_runtime/src/asset/pack/delta.rs
  - zircon_runtime/src/plugin/export_build_plan/export_validate_report.rs
  - zircon_runtime/src/plugin/export_build_plan/native_dynamic_package_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/native_plugin_load_manifest_template.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_manifest.rs
  - zircon_runtime/src/asset/pack/trim.rs
  - zircon_runtime/src/asset/pack/writer.rs
  - zircon_runtime/src/plugin/export_build_plan/mod.rs
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/Cargo.toml
implementation_files:
  - tools/zircon_export/cli.py
  - tools/zircon_export/cook_assets.py
  - tools/zircon_export/native_build.py
  - tools/zircon_export/native_dynamic.py
  - tools/zircon_export/native_signing.py
  - tools/zircon_export/pipeline_report.py
  - tools/zircon_export/pipeline_stages.py
  - tools/zircon_export/source_template.py
  - tools/zircon_export/tests/test_platform_bundle_delta.py
  - tools/zircon_export/tests/test_templates.py
  - tools/zircon_export/tests/test_native_dynamic.py
  - export-templates/windows-x86_64-library_embed-debug/template.toml
  - export-templates/windows-x86_64-library_embed-debug/bin/zircon_runtime.host-placeholder
  - export-templates/linux-x86_64-library_embed-debug/template.toml
  - export-templates/linux-x86_64-library_embed-debug/bin/zircon_runtime.host-placeholder
  - export-templates/macos-aarch64-library_embed-debug/template.toml
  - export-templates/macos-aarch64-library_embed-debug/bin/zircon_runtime.host-placeholder
  - export-templates/macos-aarch64-library_embed-debug/platform/macos/Info.plist
  - zircon_export/__main__.py
  - zircon_runtime/src/bin/zircon_export_validate/main.rs
  - zircon_runtime/src/bin/zircon_export_validate/args.rs
  - zircon_runtime/src/bin/zircon_export_validate/run.rs
  - zircon_runtime/src/bin/zircon_export_pack/main.rs
  - zircon_runtime/src/bin/zircon_export_pack/args.rs
  - zircon_runtime/src/bin/zircon_export_pack/manifest.rs
  - zircon_runtime/src/bin/zircon_export_pack/run.rs
  - zircon_runtime/src/asset/pack/delta.rs
  - zircon_runtime/src/plugin/export_build_plan/export_validate_report.rs
  - zircon_runtime/src/plugin/export_build_plan/native_dynamic_package_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/native_plugin_load_manifest_template.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_manifest.rs
  - zircon_runtime/src/asset/pack/trim.rs
  - zircon_runtime/src/asset/pack/writer.rs
  - zircon_runtime/Cargo.toml
plan_sources:
  - docs/plans/zircon_plugins/09-export-publishing.md
tests:
  - python -m py_compile tools/zircon_export/__init__.py tools/zircon_export/__main__.py tools/zircon_export/cli.py tools/zircon_export/cook_assets.py tools/zircon_export/native_build.py tools/zircon_export/native_dynamic.py tools/zircon_export/native_signing.py tools/zircon_export/pipeline_report.py tools/zircon_export/pipeline_stages.py tools/zircon_export/source_template.py zircon_export/__init__.py zircon_export/__main__.py tools/zircon_export/tests/test_templates.py tools/zircon_export/tests/test_native_dynamic.py tools/zircon_export/tests/test_platform_bundle_delta.py
  - python -m zircon_export --help
  - python -m unittest tools.zircon_export.tests.test_templates
  - python -m unittest tools.zircon_export.tests.test_native_dynamic
  - python -m unittest tools.zircon_export.tests.test_platform_bundle_delta
  - test_native_dynamic_stage_writes_package_export_report
  - test_native_dynamic_stage_materializes_package_and_loader_manifest
  - test_native_dynamic_stage_reports_materialized_file_manifest
  - test_native_dynamic_package_report_records_package_payload_hash
  - test_native_dynamic_stage_removes_stale_unselected_packages
  - test_native_dynamic_stage_filters_artifacts_by_target_platform
  - test_native_dynamic_stage_requires_platform_loadable_artifact
  - test_native_dynamic_stage_copies_macos_dsym_bundle
  - test_native_dynamic_stage_reports_package_loadable_artifacts
  - test_native_dynamic_payload_summary_keeps_loadable_artifact_audit
  - test_native_dynamic_payload_summary_rejects_malformed_package_audit
  - test_native_dynamic_payload_summary_rejects_loadable_artifact_not_in_manifest
  - test_native_dynamic_stage_reports_native_cdylib_build_plan
  - test_native_dynamic_build_plan_records_cargo_features
  - test_native_dynamic_build_executes_plan_and_stages_cdylib
  - test_native_dynamic_signs_loadable_artifact_before_manifest_hash
  - test_native_dynamic_notarization_runs_after_signing_before_manifest_hash
  - test_native_dynamic_notarization_profile_rejects_platform_mismatch
  - test_native_dynamic_signing_failure_cleans_staged_payload
  - test_native_dynamic_stage_removes_partial_package_on_artifact_filter_fatal
  - test_native_dynamic_stage_removes_all_packages_when_any_package_fails
  - test_native_dynamic_stage_rejects_inconsistent_package_paths
  - test_native_dynamic_stage_rejects_inconsistent_package_report_path
  - test_native_dynamic_stage_derives_missing_package_report_path
  - test_native_dynamic_stage_accepts_sanitized_package_directory
  - test_native_dynamic_stage_rejects_package_directory_id_mismatch
  - test_native_dynamic_stage_rejects_duplicate_package_ids
  - test_native_dynamic_stage_rejects_source_manifest_id_mismatch
  - test_native_dynamic_stage_rejects_source_manifest_parse_error
  - test_native_dynamic_stage_rejects_source_manifest_missing_id
  - test_native_dynamic_stage_rejects_duplicate_recursive_package_sources
  - test_native_dynamic_stage_rejects_non_v3_abi_version
  - test_native_dynamic_stage_rejects_wrong_v3_descriptor_symbol
  - test_native_dynamic_stage_rejects_unselected_package_export
  - test_native_dynamic_stage_rejects_duplicate_selected_package_ids
  - test_native_dynamic_stage_rejects_missing_selected_package_export
  - test_native_dynamic_stage_reports_missing_package_source_fatal
  - test_platform_bundle_copies_native_dynamic_plugins_dir
  - test_pipeline_platform_bundle_uses_native_dynamic_report_plugins
  - test_pipeline_platform_bundle_preserves_native_dynamic_payload_hash
  - test_pipeline_platform_bundle_rejects_stale_native_dynamic_payload_hash
  - test_report_stage_projects_native_dynamic_release_audit
  - test_report_stage_requires_native_dynamic_for_native_dynamic_profile
  - test_pipeline_from_validate_uses_native_dynamic_profile_stages
  - native_dynamic_only_profile_carries_minimal_compile_host_plan
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-plugin-native-dynamic-host-plan-check-0615 --message-format short --color never
  - native_loader_loads_real_fixture_from_export_load_manifest_payload
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-plugin-native-dynamic-loader-manifest-check-0615 --message-format short --color never
  - cargo test -p zircon_runtime --lib native_loader_loads_real_fixture_from_export_load_manifest_payload --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-plugin-native-dynamic-loader-manifest-check-0615 --message-format short --color never -- --exact --test-threads=1 --nocapture
  - python -m zircon_export --profile windows-release --out D:/zircon-export-native-dynamic-path-smoke --resume-from native_dynamic --dry-run
  - python -m zircon_export --profile windows-release --out D:/zircon-export-native-dynamic-path-smoke --stage native_dynamic
  - python -m zircon_export --profile windows-release --out D:/zircon-export-native-dynamic-path-smoke --stage report --pretty
  - python -m zircon_export --profile windows-release --repo-root D:/zircon-export-native-dynamic-materialize-smoke/repo --out D:/zircon-export-native-dynamic-materialize-smoke/out --resume-from native_dynamic --dry-run
  - python -m zircon_export --profile windows-release --repo-root D:/zircon-export-native-dynamic-materialize-smoke/repo --out D:/zircon-export-native-dynamic-materialize-smoke/out --stage native_dynamic
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (file_manifest/content_hash smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (native_dynamic_package.toml payload smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (stale package cleanup smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (target-platform artifact filtering smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (loadable artifact gate smoke)
  - python -m zircon_export --profile macos-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (macOS dSYM bundle copy smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (partial package cleanup smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (fatal stage atomic cleanup smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (inconsistent package path gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (inconsistent package_report gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (derived package_report gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (package directory/package_id gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (duplicate package_id gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (source manifest id mismatch smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (source manifest parse error smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (duplicate recursive source manifest smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (ABI version gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (ABI v3 contract value gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (selection/export consistency gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (duplicate selected package_id smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (fatal package materialization leaves no loader manifest smoke)
  - python -m zircon_export --profile native-dynamic-fixture-smoke --repo-root E:/Git/ZirconEngine --out D:/zircon-native-dynamic-real-fixture-smoke-0615/out --stage native_dynamic --native-dynamic-build --offline --pretty
  - python -m zircon_export --profile native-dynamic-fixture-v2-smoke --repo-root E:/Git/ZirconEngine --out D:/zircon-native-dynamic-real-fixture-v2-smoke-0615/out --stage native_dynamic --native-dynamic-build --native-dynamic-build-feature abi_v2_only --offline --pretty
  - python -m zircon_export --profile native-dynamic-fixture-release-smoke --repo-root E:/Git/ZirconEngine --out D:/zircon-native-dynamic-real-fixture-release-smoke-0615/out --stage native_dynamic --native-dynamic-build --offline --pretty
  - python -m zircon_export --profile native-dynamic-fixture-release-v2-smoke --repo-root E:/Git/ZirconEngine --out D:/zircon-native-dynamic-real-fixture-release-v2-smoke-0615/out --stage native_dynamic --native-dynamic-build --native-dynamic-build-feature abi_v2_only --offline --pretty
  - python -m zircon_export --profile windows-release --repo-root D:/zircon-export-native-dynamic-materialize-smoke/repo --out D:/zircon-export-native-dynamic-materialize-smoke/out --stage report --pretty
  - python -m zircon_export --profile windows-release --out D:/zircon-export-platform-native-plugins-smoke/out --resume-from platform_bundle
  - python -m zircon_export --profile windows-release --out <temp>/out --resume-from platform_bundle (NativeDynamic payload hash smoke)
  - python -m zircon_export --profile windows-release --out <temp>/out --resume-from platform_bundle (stale NativeDynamic payload hash smoke, expected exit code 2)
  - test_report_stage_uses_source_template_profile_requirements
  - test_report_stage_requires_source_template_for_source_template_profile
  - test_report_stage_ignores_profile_mismatch_validate_strategies
  - test_pipeline_from_validate_uses_source_template_profile_stages
  - python -m zircon_export --profile windows-release --out D:/zircon-export-source-template-path-smoke --resume-from source_template --dry-run
  - test_pack_reports_missing_asset_manifest_before_packer
  - python -m zircon_export --profile windows-release --out D:/zircon-export-pack-missing-manifest-smoke --stage pack --pretty (expected exit code 2)
  - python -m zircon_export --profile windows-release --out D:/zircon-export-resume-smoke --resume-from pack --dry-run
  - python -m zircon_export --profile windows-release --project zircon-project.toml --out D:/zircon-export-m1-smoke --stage validate --dry-run --offline --target-dir D:/cargo-targets/zircon-export-validate-cli-0614
  - python -m zircon_export --profile windows-release --out D:/zircon-export-compile-host-dryrun --stage compile_host --dry-run --offline
  - python -m zircon_export --profile windows-release --out D:/zircon-export-source-template-dryrun --stage source_template --dry-run --offline
  - python -m zircon_export --profile windows-release --out D:/zircon-export-m2-smoke --stage pack --asset-manifest D:/zircon-export-m2-smoke/assets/assets.json --determinism-check --offline --target-dir D:/cargo-targets/zircon-export-m1-validate-0614
  - cargo check -p zircon_runtime --bin zircon_export_pack --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-export-pack-profile-0615 --message-format short --color never
  - python -m zircon_export --profile windows-release --out D:/zircon-export-pack-profile-smoke/out --stage pack --asset-manifest D:/zircon-export-pack-profile-smoke/assets/assets.json --target-dir D:/cargo-targets/zircon-export-pack-profile-0615 --offline --pretty
  - python -m zircon_export --profile windows-release --project D:/zircon-export-cook-project-smoke/project/zircon-project.toml --out D:/zircon-export-cook-project-smoke/out --stage cook_assets --asset-filter shipping --pretty
  - python -m zircon_export --profile windows-release --out D:/zircon-export-m3-template-smoke --stage platform_bundle --pack-file D:/zircon-export-m3-template-smoke/inputs/assets.zrpack --template-dir export-templates/windows-x86_64-library_embed-debug --target-platform windows-x86_64
  - python -m zircon_export --profile linux-release --out D:/zircon-export-template-root-smoke --stage platform_bundle --pack-file D:/zircon-export-template-root-smoke/inputs/assets.zrpack --template-root export-templates --target-platform linux-x86_64
  - validate_report_summarizes_profile_plan_and_fatal_state
  - native_dynamic_generates_loader_manifest_without_source_template
  - validate_report_exposes_native_dynamic_abi_v3_package_exports
  - loader_manifest_deserializes_abi_v3_contract_fields
  - native_dynamic_materialization_copies_runtime_package_without_source_crates
  - delta_pack_contains_only_changed_chunks
  - test_pack_delta_args_are_forwarded_to_packer
  - delta_pack_applies_to_base_pack
  - delta_pack_rejects_wrong_base_manifest
  - test_pack_command_forwards_profile_to_packer
  - test_pipeline_pack_uses_cook_assets_report_manifest
  - test_pipeline_cook_assets_uses_validate_report_asset_filter
  - test_cook_assets_preserves_manifest_asset_filter_over_pipeline_default
  - test_cook_assets_derives_project_default_scene_without_manifest
  - test_cook_assets_reports_missing_project_default_scene_source
  - deterministic_pack_double_run_byte_identical
  - template_version_mismatch_rejected
  - test_template_rejects_aliasing_file_and_host_paths
  - test_omitting_stage_runs_main_pipeline_from_validate
  - test_pipeline_platform_bundle_uses_compile_host_report_host
  - test_pipeline_platform_bundle_uses_pack_report_pack_path
  - test_pipeline_platform_bundle_uses_pack_report_delta_pack_path
  - test_template_delta_pack_path_controls_bundle_location
  - test_checked_in_windows_template_routes_delta_pack_path
  - test_template_root_skips_malformed_template_manifest
  - test_platform_bundle_failure_cleans_previous_profile_bundle
  - test_report_stage_rejects_unverified_delta_pack
doc_type: workflow-detail
---

# Zircon Export Tool

`python -m zircon_export` is the staged export pipeline entry point for project-level release
builds. M1 implemented `Validate`; M2 added the first executable asset `Pack` stage and a
`PlatformBundle` staging shell. M3-T1 adds the first `export-template` package contract and version
lock validation. The CLI now also has a `SourceTemplate` generated-project stage, a
`NativeDynamic` package-export report stage, a `CookAssets` handoff stage that normalizes cooked
asset manifests into the standard stage directory before `Pack` consumes them, and a final `Report`
stage that aggregates per-stage JSON into the release-level pipeline report. The main stage machine
can be resumed with `--resume-from <stage>` after a failed or interrupted export. Real host
compilation and full importer-driven asset cooking are still follow-up work. NativeDynamic now
reports the exact cdylib Cargo commands for selected package crates and can execute/copy those built
artifacts only when `--native-dynamic-build` is explicitly requested.

## Ownership

The Python package under `tools/zircon_export` owns process orchestration, output layout, resume
execution, and command construction. It does not duplicate plugin dependency validation or platform
policy checks. Those decisions stay in `zircon_runtime::plugin::export_build_plan` so editor UI,
CLI, CI, and future build stages all consume the same diagnostics.

`zircon_export/__main__.py` is a thin top-level wrapper so the plan command works directly from the
repository root:

```powershell
python -m zircon_export --profile windows-release --project zircon-project.toml --out D:\zircon-export
```

Omitting `--stage` runs the main pipeline from `validate` through `report`. Passing `--stage
<stage>` keeps the single-stage debugging/CI behavior, and `--resume-from <stage>` starts the same
main pipeline at a later persisted stage.

## Validate Stage

The stage output is:

```text
<out>/
  stages/
    validate/
      report.json
```

When no prebuilt validator is supplied, the Python stage runs:

```text
cargo run -p zircon_runtime --bin zircon_export_validate --locked -- <validator-args>
```

The validator binary is deliberately small. It parses `--project`, `--profile`, optional
`--report`, optional `--stage-output`, and `--pretty`; then it loads `ProjectManifest` and calls
`ExportBuildPlan::from_project_manifest`. A profile or manifest failure still produces JSON with
`fatal = true` and a non-zero exit code, which lets CI and the future editor shell show the same
report shape for both success and failure.

The M1 report contains:

- `stage = "Validate"`, `project_manifest`, `profile`, `stage_output`, `profile_found`, and
  `fatal`.
- `diagnostics` and `fatal_diagnostics` from the effective build plan.
- `profile_summary` with target mode, target platform, build mode, strategies, selected plugins,
  selected features, and asset filter.
- `plan_summary` with enabled runtime plugins, linked runtime crates, native dynamic packages,
  NativeDynamic ABI v3 package exports, generated-file metadata and contents, the LibraryEmbed
  `CompileHost` plan when present, the SourceTemplate generated-project build plan when present, and
  runtime plugin availability categories.

For M2-T1, the `CompileHost` plan is included in the Validate report. The CLI now has an executable
`compile_host` stage that consumes this report, rewrites the planned target directory beneath the
current export output, appends `--locked` by default, passes through `--offline`, and records the
selected command and expected host executable in `<out>/stages/compile_host/report.json`. When
`--target-dir` is supplied, both the Cargo command and reported `host_executable` are derived from
that explicit directory so PlatformBundle consumes the same host path Cargo produced.

NativeDynamic-only profiles also receive the same minimal host plan through the existing
`plan_summary.library_embed_compile_host` field. The plan provides a CompileHost boundary for the
final bundle, while NativeDynamic packages remain loadable plugin payloads and are not linked into
the host's `linked_runtime_crates` list.

The stage intentionally depends on `Validate` output instead of re-running profile dependency logic
in Python. If the Validate report is fatal, missing, for another profile, or lacks a
`library_embed_compile_host` plan, CompileHost returns a fatal report before invoking Cargo.

## SourceTemplate Stage

`SourceTemplate` consumes the Validate report's generated-file rows and
`plan_summary.source_template_build` command. It materializes the generated Cargo project under:

```text
<out>/
  stages/
    source_template/
      project/
        Cargo.toml
        src/...
      report.json
```

Because the generated `Cargo.toml` is authored by the Rust build-plan templates, Python does not
reconstruct project dependencies. It only writes the generated files, rewrites local `zircon_*`
path dependencies from template-relative paths to absolute workspace paths for the current
`--repo-root`, and records the validated cargo build command in `report.json`.

Materialization diagnostics that make the generated project untrustworthy are fatal: invalid or
escaping generated-file paths, generated-file rows without contents, a missing generated
`Cargo.toml`, or missing rewritten local `zircon_*` dependency paths. These diagnostics stop the
optional build step before Cargo is invoked. The deliberate "build validation skipped" diagnostic
remains non-fatal when `--source-template-build` is not supplied.

By default this stage materializes the project and skips Cargo execution. Passing
`--source-template-build` executes the validated `cargo build --manifest-path <project>/Cargo.toml`
command, with `--locked` enabled by default and `--offline` forwarded when requested. This keeps the
stage usable during current workspace compile drift while preserving the real build-validation hook
for CI and later clean runs.

When Validate report `profile_summary.strategies` contains `source_template`, the main pipeline now
includes this stage. A SourceTemplate-only profile runs `Validate -> SourceTemplate -> Report`; a
hybrid SourceTemplate + LibraryEmbed profile runs SourceTemplate first and then the LibraryEmbed
host/assets/bundle stages. This keeps the first-class `python -m zircon_export --profile <name>`
entry point aligned with the profile path instead of requiring a manual `--stage source_template`
detour.

## NativeDynamic Package Manifest

Validate report keeps both native dynamic views: `native_dynamic_packages` is the compatibility
package-id list, and `native_dynamic_package_exports` is the structured ABI v3 package export table.
Each export row records the package id, output directory derived from the Rust build plan's
`package_id` sanitization rule, package path, package manifest path, and ABI v3 contract fields used
by the native loader/tooling boundary. The Python stage derives `package_report =
"<path>/native_dynamic_package.toml"` when the Validate report omits it, matching the current Rust
`NativeDynamicPackageExportPlan` shape; if a report supplies `package_report`, the stage validates it
against that derived path.

The Rust build plan generates `plugins/native_plugins.toml` with `id`, `path`, `manifest`,
`package_report`, and `[plugins.abi]` for each native package. When native packages are materialized,
each copied package receives a `native_dynamic_package.toml` report with the same ABI v3 descriptor,
entry, host function table, behavior, snapshot, and bridge method table contract. The package report
also carries `[payload]` with a package-local file count, content hash, and `[[payload.files]]`
entries for the release-facing files copied into that package, excluding the generated package report
itself.

The CLI `native_dynamic` stage consumes the Validate report, finds the selected native packages
under `<repo-root>/zircon_plugins` by matching `plugin.toml` ids, and writes:

```text
<out>/
  stages/
    native_dynamic/
      plugins/
        native_plugins.toml
        <package>/
          plugin.toml
          native/...
          resources/...
          native_dynamic_package.toml
      report.json
```

Before materializing packages, the stage recreates its owned `<out>/stages/native_dynamic/plugins/`
directory. This makes repeated exports deterministic: a package removed from the active profile
cannot remain in the staged payload, loader manifest, file manifest, or content hash from an earlier
run.

The stage also reads `profile_summary.target_platform` from the Validate report when present and
filters copied native artifacts by platform family. Windows packages copy `.dll` and `.pdb`, Linux
packages copy `.so` and `.dbg`, and macOS packages copy `.dylib` plus `.dSYM`/`.dsym` debug symbol
bundles. macOS debug bundles are copied recursively from `native/` and their nested files are listed
in the staged file manifest. If a legacy or unknown Validate report has no recognizable target
platform, the stage falls back to the full native artifact extension set so older reports remain
diagnosable instead of failing before the package shape can be inspected. Debug symbol files or
directories may accompany a package, but they do not satisfy the loadable library requirement by
themselves: each materialized package must include at least one `.dll`, `.so`, or `.dylib` selected by
the target platform.

The `native_dynamic_package_exports` table is validated before any package payload is materialized.
The selected package list and structured export table must match exactly: every
`native_dynamic_packages` id needs one export row, every export row must be selected, and selected
package ids may not repeat. Each `package_id` in the export table may also appear only once. Each
row must also be internally consistent: `directory` must equal the sanitized package id that the
Rust build plan would generate (`animation.fx` becomes `animation_fx`), and for
`package_id = "animation"` / `directory = "animation"` the stage accepts only `path = "plugins/animation"`,
`manifest = "plugins/animation/plugin.toml"`, and
derived or supplied `package_report = "plugins/animation/native_dynamic_package.toml"`. Unselected
export rows, missing export rows, duplicate `package_id` values, mismatched `directory`, or
mismatched `path`/`manifest`/`package_report` entries are fatal and prevent
`plugins/native_plugins.toml` from being written, so the loader manifest cannot point at a different
package location than the staged payload or contain rows that disagree with the profile's selected
plugin set.

NativeDynamic also treats the ABI v3 contract as a hard publishing boundary: each
`native_dynamic_package_exports` row must carry `abi.abi_version = 3`, and every ABI contract string
must match the Rust build-plan generator's fixed v3 values such as
`zircon_native_plugin_descriptor_v3`, `NativePluginAbiV3`, and
`NativePluginBridgeMethodTableV3`. Older ABI versions, future ABI versions, or mismatched v3 contract
names are fatal before package materialization, so an incompatible loader contract cannot be written
into `plugins/native_plugins.toml`.

The stage report records the Validate report path, `native_dynamic_packages`, the full
`native_dynamic_package_exports` table, `package_count`, `loader_manifest`, each materialized
package source/destination/report path, each package's stage-relative `loadable_artifacts` plus
`loadable_artifact_count`, and a `native_build_plan`. The build plan reads the selected source
package `plugin.toml` module crate names, matches them against `cdylib` members declared in
`<repo-root>/zircon_plugins/Cargo.toml`, derives the Cargo profile from Validate
`profile_summary.build_mode`, and records the target directory, exact `cargo build` command
(`--manifest-path`, `-p`, `--target-dir`, lock/offline/release flags), and platform-specific
expected loadable artifact path for every matched package. The default target directory remains
`<out>/stages/native_dynamic/target`, but an explicit `--target-dir` feeds the native cdylib build
plan, execution command, and expected loadable artifact path. Repeated
`--native-dynamic-build-feature <feature>` values are normalized, deduplicated, recorded in
`native_build_plan.build_features` and each package plan's `features`, and appended to the Cargo
command as `--features <comma-separated features>`.

By default this plan is deliberately non-executing: missing workspace metadata is reported inside
`native_build_plan.diagnostics`, while package materialization still consumes existing artifacts
under each package's `native/` directory. Passing `--native-dynamic-build` turns the plan into an
execution gate. In that mode the stage may materialize package metadata/resources before source
`native/` artifacts exist, runs each planned Cargo command from `--repo-root`, copies the expected
`.dll`/`.so`/`.dylib` into the staged package `native/` directory, copies adjacent `.pdb`/`.dbg` or
`.dSYM` sidecars when present, then writes `native_dynamic_package.toml` after the built artifact is
part of the package payload. The stage report records this as `native_build_execution` with per
package command output, exit code, expected artifact, copied artifact, copied sidecars, and fatal
diagnostics.

NativeDynamic can also run an explicit external signing command before package reports and the
stage-level manifest/hash are written. Passing `--native-dynamic-sign-command <program>` enables the
hook; repeated `--native-dynamic-sign-arg <arg>` values are appended to the signer command and may
use `{artifact}`, `{package_id}`, `{package_dir}`, `{target_platform}`, and `{signing_profile}`
placeholders. `--native-dynamic-sign-profile <name>` records the selected signing profile and passes
that value to the external signer. Repeated `--native-dynamic-sign-platform <prefix>` entries form a
platform gate, so a profile declared for `windows` is accepted for `windows-x86_64` and rejected for
`macos-*` targets before any signer process starts. The stage executes the signer once per staged
loadable artifact selected by the target platform. It records the profile, target platform, allowed
platforms, platform-gate decision, expanded command, stdout/stderr, exit code, and before/after
SHA-256 values in `native_signing`. A platform mismatch, non-zero signer exit code, or a signer that
removes the loadable file is fatal and clears the owned `plugins/` payload. This mirrors Godot's
separation between copying shared objects and code-signing them before final bundle metadata is
sealed, while keeping Zircon's current implementation tool-agnostic instead of baking in platform
certificate stores.

After signing, NativeDynamic can run an explicit external notarization or platform post-processing
command before package reports and the stage-level manifest/hash are written. Passing
`--native-dynamic-notarize-command <program>` enables the hook; repeated
`--native-dynamic-notarize-arg <arg>` values are appended to the command and may use `{artifact}`,
`{package_id}`, `{package_dir}`, `{target_platform}`, `{signing_profile}`, and
`{notarization_profile}` placeholders. `--native-dynamic-notarize-profile <name>` records the audit
profile and passes it to the external command. Repeated `--native-dynamic-notarize-platform <prefix>`
entries form a platform gate before the command starts. The stage records the profile, target
platform, allowed platforms, platform-gate decision, expanded command, stdout/stderr, exit code, and
before/after SHA-256 values in `native_notarization`. A platform mismatch, non-zero command exit
code, or a command that removes the loadable file is fatal and clears the owned `plugins/` payload.
This is an external command boundary for future Windows/macOS/Linux platform services; it does not
integrate OS certificate stores, notary accounts, ticket stapling, or platform package repositories
by itself.

The downstream payload summary keeps the package audit and rejects malformed `materialized_packages`
rows instead of silently dropping per-package loadable-library evidence. It also cross-checks every
package `loadable_artifacts` path against the stage `file_manifest`, so a report cannot claim a
loadable library that is absent from the staged payload. The package-prefix check uses the
materialized package `destination` relative to the staged `plugins/` directory, not the raw
`package_id`, so package ids that sanitize to a different output directory such as
`animation.fx -> plugins/animation_fx` remain valid.
On non-fatal exports it also records a sorted `file_manifest` of staged `plugins/` files.
Each file entry contains the stage-relative path, byte length, and sha256; the top-level
`content_hash` is derived from those entries so downstream tooling can verify the exact
NativeDynamic package payload before copying or publishing it. Package materialization copies only the release-facing
`plugin.toml`, target-platform native artifacts under `native/`, and resource directories named
`assets`, `asset`, `resources`, or `resource`; source crates and unrelated development files are not
copied into the stage output. When a direct package directory exists, its source `plugin.toml` is
authoritative for that selected package: parse errors, a missing/non-string `id`, or an id mismatch
are reported directly, and the stage does not fall back to a broader search that could silently pick
another package. When the direct package directory is absent and recursive search finds more than one
`plugin.toml` with the selected id, the stage reports a duplicate source fatal instead of choosing an
arbitrary directory. NativeDynamic treats the staged
`plugins/` payload as atomic: if any
selected package cannot be found, lacks usable source native artifacts in the default non-build mode,
has invalid cdylib plan metadata in build mode, fails Cargo execution, or does not produce its
expected loadable artifact, or fails the configured signing/notarization command, the stage writes a fatal report,
keeps `loader_manifest = null`, does not write `plugins/native_plugins.toml`, clears the owned staged
`plugins/` payload so no successful package remains publishable beside the failed package, sets
`payload_cleaned = true` with `cleanup_reason = "fatal_diagnostics"`, and returns exit code `2`.
Successful NativeDynamic reports keep `payload_cleaned = false` and `cleanup_reason = null`.

This is still an M5 NativeDynamic export slice. Optional cdylib build/copy is implemented behind
`--native-dynamic-build`, and external signer execution plus signing-profile audit/gating is
implemented behind `--native-dynamic-sign-command`; external notarization/post-processing execution
plus profile audit/gating is implemented behind `--native-dynamic-notarize-command`;
platform-native certificate-store integration, real notary service/ticket workflows, runtime
hot-update end-to-end invocation, and the linux/macos cross-platform real fixture matrix remain
follow-up work. The checked-in `native_dynamic_fixture` currently has one
debug NativeDynamic stage smoke with `--native-dynamic-build --offline` and one debug ABI v2 feature
smoke with `--native-dynamic-build-feature abi_v2_only`, plus matching release-mode smokes for both
variants; all four Windows local rows build and stage the cdylib payload.

## CookAssets Stage

`CookAssets` currently owns the pipeline handoff between future importer-driven asset cooking and
the existing pack writer. It consumes the same cooked asset manifest shape that `Pack` already
understands, validates the basic JSON contract, rewrites relative `source` paths to absolute paths
based on the source manifest directory, and writes:

```text
<out>/
  stages/
    cook_assets/
      assets.json
      report.json
```

The stage is intentionally not a real importer yet. It does not scan project assets, invoke scene or
texture importers, build dependency graphs, or transform bytes. Those behaviors still belong to the
future real CookAssets implementation. The current stage exists so `Validate -> CompileHost ->
CookAssets -> Pack -> PlatformBundle` has a stable file boundary and so `Pack` no longer needs a
manual manifest path once CookAssets has run.

When no `--asset-manifest` is supplied, CookAssets now has a conservative project fallback. It reads
`--project`, resolves `default_scene = "res://..."` to `<project>/assets/...`, and writes a minimal
cooked manifest with that scene as the only root and only asset. If the main pipeline supplied a
profile `asset_filter`, the fallback entry receives the same label so the entry is not cut by the
temporary label filter. This is only a project-entry boundary; dependency expansion remains future
importer work.

CookAssets also checks any declared `source` path after normalization. Missing source files make the
stage fatal and prevent `<out>/stages/cook_assets/assets.json` from being written, so invalid project
entry manifests fail before Pack tries to read bytes.

When `Pack` is launched by the main pipeline or by `--resume-from pack`, the runner first reads a
non-fatal matching-profile `<out>/stages/cook_assets/report.json` and uses its
`cooked_asset_manifest` path as the Pack manifest input. An explicit `--asset-manifest` still wins,
and standalone `--stage pack` keeps the fixed default `<out>/stages/cook_assets/assets.json`.

CookAssets also accepts `--asset-filter <label>` as a default profile filter. The main pipeline
fills that option from Validate report `profile_summary.asset_filter` when present. The default is
written into the staged cooked manifest only when the source manifest does not already declare
`asset_filter`, so explicitly cooked manifests keep their own filter decision.

The manifest shape is:

```json
{
  "roots": ["scenes/main.zscene"],
  "asset_filter": "shipping",
  "assets": [
    {
      "path": "scenes/main.zscene",
      "source": "scenes/main.zscene",
      "dependencies": ["textures/hero.png"],
      "labels": ["shipping"]
    },
    {
      "path": "textures/hero.png",
      "source": "textures/hero.png",
      "labels": ["shipping"]
    }
  ]
}
```

## Pack Stage

`Pack` consumes a cooked asset manifest rather than reading `zircon-project.toml` directly. If
`--asset-manifest` is supplied, that explicit path is used. Otherwise `Pack` defaults to
`<out>/stages/cook_assets/assets.json`, the standard output from the CookAssets handoff stage.

In non-dry-run mode, the Python stage preflights that manifest path before invoking the Rust packer.
If the path does not exist or is not a file, the stage writes `<out>/stages/pack/report.json` with
`fatal=true`, an empty trim report, and a concrete diagnostic, then returns exit code `2`. This keeps
resume and final Report aggregation on the normal stage-report path instead of failing with no Pack
report. Dry-run still only prints the selected command and input paths.

The Python stage calls the Rust `zircon_export_pack` binary with the export `--profile`, which runs
the `ZrPackTrimPlanner`, passes included assets to `ZrPackWriter`, writes
`<out>/stages/pack/assets.zrpack`, and emits `<out>/stages/pack/report.json`. The report includes
the same `profile` field used by other stages, so downstream pipeline handoff can reject mismatched
reports when profile metadata is present. `--determinism-check` writes the pack in memory a second
time and fails the stage if the bytes differ. Missing dependencies, duplicate trim inputs, and
writer errors are fatal.

For M5-T2, Pack can also produce a delta package by passing `--previous-pack <old.zrpack>` and
`--delta-pack <delta.zrpd>` together. The Rust packer reads the old and newly written full pack,
computes the chunk-hash difference, writes only target chunks missing from the old pack into the
`ZRPD` delta file, and records `delta_manifest`, delta asset/chunk counts, removed assets, and reused
assets in the pack report. This is the byte-package/report layer only; runtime application of a
NativeDynamic hot update remains a later slice.

## PlatformBundle Stage

`PlatformBundle` currently creates:

```text
<out>/
  bundle/
    <profile>/
      assets.zrpack
      bundle.json
  stages/
    platform_bundle/
      report.json
```

When `--host-executable` is supplied, the executable is copied beside the pack. Without a host
executable, the stage writes an assets-only bundle directory but returns fatal status; this keeps the
M2 report honest until CompileHost can produce the actual runtime/editor executable.

When PlatformBundle is launched as part of the main pipeline or through `--resume-from
platform_bundle`, the pipeline runner reads `<out>/stages/compile_host/report.json` and uses its
non-fatal `host_executable` field as the default host input. Explicit `--host-executable` still wins,
and standalone `--stage platform_bundle` keeps requiring callers to pass a host or template source
directly.

The same pipeline-only defaulting now reads `<out>/stages/pack/report.json` when `--pack-file` is
not explicit. A non-fatal report with a `pack` path becomes the PlatformBundle pack input, which
keeps resumed exports aligned with custom Pack output paths. Standalone `--stage platform_bundle`
continues to use the default `<out>/stages/pack/assets.zrpack` or an explicit `--pack-file`.

Each non-dry-run PlatformBundle execution recreates the current profile bundle directory before
validation and materialization. If the stage becomes fatal before or during materialization, the
profile bundle directory is removed and no final `bundle.json` or template-provided manifest is
written; callers should inspect only the stage report for that failed attempt.

M3-T1 adds optional `--template-dir <dir>` support. The directory must contain `template.toml`; when
valid, `paths.host_executable` can provide the host executable path so callers do not need to pass
`--host-executable` separately. M3-T2 adds `--template-root <dir>` for local template repositories:
when `--template-dir` is omitted, PlatformBundle scans direct child packages and selects the single
template matching the requested profile, target platform, engine version, and format version.
Template validation runs before copying bundle contents and records a `template` object in both
`bundle.json` and `<out>/stages/platform_bundle/report.json`. Template-root selection also records
`template_resolution` with candidates and diagnostics. A mismatch returns exit code `2` and skips
host/pack copying.

The current `template.toml` format is:

```toml
format_version = 1
template_id = "windows-x86_64-library_embed-debug"
engine_version = "0.1.0"
target_platform = "windows-x86_64"
host_kind = "desktop"
resource_strategy = "filesystem_bundle"
plugin_strategy = "native_dynamic_allowed"
bundle_format = "directory"
compatible_profiles = ["windows-release"]
content_hash = "<sha256 over sorted file path + file sha256 rows>"

[paths]
host_executable = "bin/zircon_runtime.host-placeholder"

[bundle]
host_path = "ZirconRuntime"
pack_path = "data/assets.zrpack"
delta_pack_path = "patches/assets.delta.zrpd"
manifest_path = "zircon-export.json"

[[files]]
path = "bin/zircon_runtime.host-placeholder"
bundle_path = "ZirconRuntime"
purpose = "M3-T1 placeholder host path for template contract validation"
sha256 = "<file sha256>"
```

`format_version` is locked to `1`. `engine_version` defaults to `[workspace.package].version` from
the root `Cargo.toml` unless `--engine-version` is supplied. Target platform is taken from
`<out>/stages/validate/report.json` when available, or from `--target-platform`. The CLI also
verifies `paths.host_executable` and each `[[files]].path` are safe relative paths: no absolute
path, empty segment, `.`, or `..`. It then checks that the declared host path stays inside the
template directory, is present in `[[files]]`, and matches its declared SHA-256 digest and
aggregate `content_hash`.

M3-T2 extends the same contract with template-driven bundle layout. `[bundle]` can declare
`root`, `host_path`, `pack_path`, and `manifest_path`, and each `[[files]]` entry can declare the
destination `bundle_path`. All bundle paths must be relative and stay inside the profile bundle
directory. The checked-in Linux fixture materializes a directory bundle as
`ZirconRuntime`, `data/assets.zrpack`, and `zircon-export.json`; the macOS fixture materializes
`ZirconRuntime.app/Contents/MacOS/ZirconRuntime`,
`ZirconRuntime.app/Contents/Resources/assets.zrpack`,
`ZirconRuntime.app/Contents/Info.plist`, and
`ZirconRuntime.app/Contents/Resources/zircon-export.json`. The host files are still placeholders;
real runnable platform templates remain gated on CompileHost/CI artifacts.

When `--native-plugins-dir` is supplied, PlatformBundle copies that staged NativeDynamic
`plugins/` directory into `<bundle-root>/plugins`. In the main pipeline this option is filled from
`<out>/stages/native_dynamic/report.json` when the report is non-fatal and matches the requested
profile, so hybrid LibraryEmbed + NativeDynamic exports keep native plugin packages in the final
bundle without a manual path. When that same NativeDynamic report contains `content_hash` and
`file_manifest`, and a well-formed `materialized_packages` list, PlatformBundle preserves the
payload audit as `native_plugins_payload` in both `bundle.json` and the PlatformBundle stage report,
including the stage report path, source path, final bundle path, file count, file manifest, content
hash, package count, per-package loadable artifact lists, and stable `native_signing` /
`native_notarization` audit summaries when present. Those operation summaries intentionally carry
only stable header fields such as `enabled`, `profile`, `target_platform`, `allowed_platforms`,
`platform_allowed`, `fatal`, and `package_count`; the full per-artifact command/stdout/stderr/hash
evidence remains in the NativeDynamic stage report. Before copying, PlatformBundle
recomputes the current staged `plugins/` directory hash and rejects the bundle if it no longer
matches the NativeDynamic report `content_hash`; malformed package audit rows also become stage
diagnostics instead of a partial payload summary.

## Report Stage

`Report` is the final pipeline aggregation stage. It reads Validate first, then derives required
stage reports from Validate report `profile_summary.strategies`.

For a LibraryEmbed profile, it requires:

```text
<out>/stages/validate/report.json
<out>/stages/compile_host/report.json
<out>/stages/cook_assets/report.json
<out>/stages/pack/report.json
<out>/stages/platform_bundle/report.json
```

For a SourceTemplate profile, it requires:

```text
<out>/stages/validate/report.json
<out>/stages/source_template/report.json
```

For a NativeDynamic profile, it requires:

```text
<out>/stages/validate/report.json
<out>/stages/native_dynamic/report.json
<out>/stages/compile_host/report.json
<out>/stages/cook_assets/report.json
<out>/stages/pack/report.json
<out>/stages/platform_bundle/report.json
```

The stage writes both `<out>/stages/report/report.json` and the release-level `<out>/report.json`.
The aggregate report records missing stages, fatal stages, each source report path, each stage's
diagnostics, and the embedded raw stage report. Missing required reports, malformed JSON, profile
mismatch, or any stage with `fatal = true` makes the final report fatal and returns exit code `2`.
`source_template` and `native_dynamic` are read only when the Validate strategies request them. Stale
reports for non-selected strategies can remain under `<out>/stages/`, but they do not enter the
current pipeline report and cannot make the current export fatal.

## Resume Flow

Omitting `--stage` runs the main export stage machine from `validate` through `report`. After
Validate succeeds, the runner reads `profile_summary.strategies` from
`<out>/stages/validate/report.json` and selects the remaining stages for the requested path. This is
the plan-level release command used by local export and CI orchestration:

```powershell
python -m zircon_export --profile windows-release --project zircon-project.toml --out D:\zircon-export
```

`--resume-from <stage>` now runs the main export stage machine from the selected stage through
`report`. Resume also uses the Validate report strategy list when available, so
`--resume-from source_template` on a SourceTemplate profile continues with `source_template,report`,
`--resume-from native_dynamic` on a NativeDynamic profile continues with
`native_dynamic,compile_host,cook_assets,pack,platform_bundle,report`,
while `--resume-from pack` on a LibraryEmbed profile continues with `pack,platform_bundle,report`.

If the Validate report is present and the requested resume stage is outside the selected strategy
set, the runner skips directly to `report`; stale or manually requested strategy stages are not
replayed for the current profile.

```text
validate -> [source_template] -> [native_dynamic] -> [compile_host -> cook_assets -> pack -> platform_bundle] -> report
```

This option is for pipeline recovery and cannot be combined with `--stage`, which remains the
single-stage debug and CI entry point. The runner stops at the first non-zero stage exit code and
does not synthesize later reports, so a failed `platform_bundle` resume does not accidentally write a
final `<out>/report.json`.

`source_template` can still be run as a standalone debug stage with `--stage source_template`, but it
is no longer excluded from the main pipeline when the selected profile requests the SourceTemplate
strategy.

## Command Surface

Useful commands:

```powershell
python -m zircon_export --help
python -m zircon_export --profile windows-release --project zircon-project.toml --out D:\zircon-export --stage validate
python -m zircon_export --profile windows-release --out D:\zircon-export --stage validate --dry-run --offline --target-dir D:\cargo-targets\zircon-export-validate-cli-0614
python -m zircon_export --profile windows-release --out D:\zircon-export --stage compile_host --offline
python -m zircon_export --profile windows-release --out D:\zircon-export --stage source_template --offline
python -m zircon_export --profile windows-release --out D:\zircon-export --stage source_template --source-template-build --offline
python -m zircon_export --profile windows-release --out D:\zircon-export --stage native_dynamic
python -m zircon_export --profile windows-release --out D:\zircon-export --stage native_dynamic --native-dynamic-build --offline
python -m zircon_export --profile windows-release --out D:\zircon-export --stage native_dynamic --native-dynamic-build --native-dynamic-build-feature abi_v2_only --offline
python -m zircon_export --profile windows-release --out D:\zircon-export --stage native_dynamic --native-dynamic-sign-command D:\tools\sign-native.exe --native-dynamic-sign-arg "{artifact}"
python -m zircon_export --profile windows-release --out D:\zircon-export --stage native_dynamic --native-dynamic-notarize-command D:\tools\notarize-native.exe --native-dynamic-notarize-arg "{artifact}" --native-dynamic-notarize-profile windows-attestation --native-dynamic-notarize-platform windows
python -m zircon_export --profile windows-release --out D:\zircon-export --stage cook_assets --asset-manifest D:\zircon-export\assets\assets.json
python -m zircon_export --profile windows-release --out D:\zircon-export --stage pack --determinism-check
python -m zircon_export --profile windows-release --out D:\zircon-export --stage pack --previous-pack D:\zircon-export\previous\assets.zrpack --delta-pack D:\zircon-export\stages\pack\assets.delta.zrpd
python -m zircon_export --profile windows-release --out D:\zircon-export --stage platform_bundle --host-executable D:\zircon-export\stages\compile_host\zircon_runtime.exe
python -m zircon_export --profile windows-release --out D:\zircon-export --stage platform_bundle --host-executable D:\zircon-export\stages\compile_host\zircon_runtime.exe --native-plugins-dir D:\zircon-export\stages\native_dynamic\plugins
python -m zircon_export --profile windows-release --out D:\zircon-export --stage platform_bundle --pack-file D:\zircon-export\stages\pack\assets.zrpack --template-dir export-templates\windows-x86_64-library_embed-debug --target-platform windows-x86_64
python -m zircon_export --profile linux-release --out D:\zircon-export --stage platform_bundle --pack-file D:\zircon-export\stages\pack\assets.zrpack --template-root export-templates --target-platform linux-x86_64
python -m zircon_export --profile windows-release --out D:\zircon-export --stage report
python -m zircon_export --profile windows-release --out D:\zircon-export --resume-from pack
```

`--validator <path>` lets callers use a prebuilt `zircon_export_validate` executable and skip
`cargo run`. `--packer <path>` does the same for `zircon_export_pack`. `--asset-manifest <path>` is
the CookAssets source manifest and remains an explicit Pack input override when needed.
`--previous-pack` and `--delta-pack` enable M5-T2 delta package output for Pack and must be supplied
together.
`--template-dir <path>` makes PlatformBundle consume one export-template package.
`--template-root <path>` makes PlatformBundle resolve one matching package from a local template
repository when `--template-dir` is omitted. `--engine-version` and `--target-platform` can override
the values used for template compatibility checks.
`--native-plugins-dir <path>` copies a NativeDynamic stage `plugins/` directory into the final
PlatformBundle output; main/resume pipeline execution fills it from a non-fatal
`<out>/stages/native_dynamic/report.json` when present. If no matching NativeDynamic stage report is
available for an explicit directory, PlatformBundle still records a directory-level
`native_plugins_payload` snapshot with content hash, file manifest, and package/loadable-artifact
summary. That explicit-directory snapshot reports final bundle logical paths under `plugins/...`
even when the source directory has a different local name. A matching malformed or stale
NativeDynamic report remains fatal instead of being silently replaced by a directory snapshot.
`--source-template-build` makes the SourceTemplate stage execute the generated project's Cargo build
instead of only materializing files. `--native-dynamic-build` makes the NativeDynamic stage execute
its cdylib Cargo build plan and copy the built loadable artifacts into staged plugin packages;
without it, NativeDynamic only consumes existing package `native/` artifacts. Repeat
`--native-dynamic-build-feature` to pass Cargo features such as `abi_v2_only` into the native cdylib
build plan and execution command.
`--native-dynamic-sign-command` enables an external signer for staged NativeDynamic loadable artifacts;
repeat `--native-dynamic-sign-arg` for signer arguments and use placeholders such as `{artifact}` and
`{target_platform}` when the signer needs artifact-specific values. Add
`--native-dynamic-sign-profile` to record and pass a profile label, and repeat
`--native-dynamic-sign-platform` to restrict that profile to target-platform prefixes before the
external signer is launched. `--dry-run` prints the exact stage command or bundle inputs without
creating stage output. Cargo commands use `--locked` by default;
`--native-dynamic-notarize-command` enables an external notarization or platform post-processing
command after signing and before package reports/manifests are sealed; repeat
`--native-dynamic-notarize-arg` for arguments, use `--native-dynamic-notarize-profile` to record a
profile label, and repeat `--native-dynamic-notarize-platform` to gate target-platform prefixes
before the command is launched.
`--no-locked` exists only for explicit lockfile work.
`--resume-from <stage>` replays the main pipeline from a persisted stage directory and is mutually
exclusive with `--stage`.

## Future Stages

The Python stage enum currently exposes `Validate`, `CompileHost`, `SourceTemplate`,
`NativeDynamic`, `CookAssets`, `Pack`, `PlatformBundle`, and `Report`; the resumable main pipeline
selects SourceTemplate, NativeDynamic, and LibraryEmbed stage groups from Validate strategies. Later
work should replace the CookAssets handoff with real importer-driven cooking, extend NativeDynamic
beyond optional cdylib build/copy plus external signer/notarization profile command execution into
platform-native certificate-store signing, real notary service/ticket workflows, runtime hot-update
end-to-end invocation, and the
linux/macos cross-platform real fixture matrix, and expand the final report with
launch-smoke/performance evidence without moving plan validation out of `zircon_runtime`.
Each stage should continue writing beneath
`<out>/stages/<stage>/` so failures are resumable and editor UI can stream a stable pipeline model.

## Test Coverage

M1 adds `validate_report_summarizes_profile_plan_and_fatal_state` to prove the shared runtime report
summarizes profile fields, plan crate links, and fatal state. Python smoke coverage checks module
compilation, `--help`, and a dry-run Validate command.

M2-T1 adds `feature_matrix_links_selected_plugins_only`, which verifies that a LibraryEmbed profile
projects selected plugins and selected feature crates into the CompileHost plan while trimming
unselected plugin and optional feature crates.

The Python CompileHost dry-run coverage verifies that the CLI consumes
`plan_summary.library_embed_compile_host`, rewrites `--target-dir` to
`<out>/stages/compile_host/target`, appends Cargo lock/offline flags, computes the expected host
executable path, and rejects profile mismatches before invoking Cargo. It now also includes
`test_compile_host_report_respects_target_dir_override`, which verifies a custom `--target-dir`
feeds both Cargo execution and the report `host_executable` handoff path. A real Cargo CompileHost
run is not claimed yet because current runtime/UI compile drift is still being tracked separately.

The Python SourceTemplate coverage verifies that the CLI consumes Validate report generated-file
contents, rewrites the generated project's `--manifest-path` and `--target-dir`, materializes
`<out>/stages/source_template/project`, rewrites local `zircon_*` path dependencies to the current
workspace root, and records a non-fatal report when build execution is intentionally skipped.
`test_source_template_stage_marks_invalid_generated_file_fatal` verifies that invalid generated
paths fail the SourceTemplate report instead of producing an incomplete successful project.
The path-aware pipeline coverage adds `test_pipeline_from_validate_uses_source_template_profile_stages`,
`test_report_stage_uses_source_template_profile_requirements`, and
`test_report_stage_requires_source_template_for_source_template_profile`. Together they verify that
Validate strategies drive both execution order and final report requirements for SourceTemplate-only
profiles. `test_report_stage_ignores_stale_strategy_reports` keeps the final Report stage isolated
from stale `source_template` or `native_dynamic` reports when the current Validate report requests
only `library_embed`.

The Python NativeDynamic path coverage adds `test_native_dynamic_stage_writes_package_export_report`,
`test_native_dynamic_stage_materializes_package_and_loader_manifest`,
`test_native_dynamic_stage_reports_materialized_file_manifest`,
`test_native_dynamic_package_report_records_package_payload_hash`,
`test_native_dynamic_stage_removes_stale_unselected_packages`,
`test_native_dynamic_stage_filters_artifacts_by_target_platform`,
`test_native_dynamic_stage_requires_platform_loadable_artifact`,
`test_native_dynamic_stage_copies_macos_dsym_bundle`,
`test_native_dynamic_stage_reports_package_loadable_artifacts`,
`test_native_dynamic_payload_summary_keeps_loadable_artifact_audit`,
`test_native_dynamic_payload_summary_rejects_malformed_package_audit`,
`test_native_dynamic_payload_summary_rejects_loadable_artifact_not_in_manifest`,
`test_native_dynamic_stage_reports_native_cdylib_build_plan`,
`test_native_dynamic_build_executes_plan_and_stages_cdylib`,
`test_native_dynamic_signs_loadable_artifact_before_manifest_hash`,
`test_native_dynamic_signing_failure_cleans_staged_payload`,
`test_native_dynamic_payload_summary_accepts_sanitized_package_directory`,
`test_native_dynamic_stage_removes_partial_package_on_artifact_filter_fatal`,
`test_native_dynamic_stage_removes_all_packages_when_any_package_fails`,
`test_native_dynamic_stage_rejects_inconsistent_package_paths`,
`test_native_dynamic_stage_rejects_inconsistent_package_report_path`,
`test_native_dynamic_stage_derives_missing_package_report_path`,
`test_native_dynamic_stage_accepts_sanitized_package_directory`,
`test_native_dynamic_stage_rejects_package_directory_id_mismatch`,
`test_native_dynamic_stage_rejects_duplicate_package_ids`,
`test_native_dynamic_stage_rejects_source_manifest_id_mismatch`,
`test_native_dynamic_stage_rejects_source_manifest_parse_error`,
`test_native_dynamic_stage_rejects_source_manifest_missing_id`,
`test_native_dynamic_stage_rejects_duplicate_recursive_package_sources`,
`test_native_dynamic_stage_rejects_non_v3_abi_version`,
`test_native_dynamic_stage_rejects_wrong_v3_descriptor_symbol`,
`test_native_dynamic_stage_rejects_unselected_package_export`,
`test_native_dynamic_stage_rejects_duplicate_selected_package_ids`,
`test_native_dynamic_stage_rejects_missing_selected_package_export`,
`test_native_dynamic_stage_reports_missing_package_source_fatal`,
`test_report_stage_requires_native_dynamic_for_native_dynamic_profile`, and
`test_pipeline_from_validate_uses_native_dynamic_profile_stages`. Together they verify that a
NativeDynamic profile runs
`NativeDynamic -> CompileHost -> CookAssets -> Pack -> PlatformBundle -> Report`, that final Report
requires the `native_dynamic` stage report plus the downstream bundle stage reports, that the stage
preserves the ABI v3 package export table from the Validate report, that package files and
`plugins/native_plugins.toml` are materialized, that staged files are reported with deterministic
paths, byte lengths, sha256 values, and a top-level
`content_hash`, that each package's loadable `.dll`/`.so`/`.dylib` files stay visible in
`materialized_packages[]` and payload summaries, that malformed package audit rows are rejected,
that claimed loadable artifact paths must exist in the stage `file_manifest`,
that the loadable-artifact prefix check follows sanitized output directories rather than raw ids,
that the report includes a cdylib Cargo build plan for selected package crates, that
repeated `--native-dynamic-build-feature` values are normalized into the build plan and Cargo
command, that `test_native_dynamic_build_plan_respects_target_dir_override` keeps custom
`--target-dir` values aligned across the native build plan, command, and expected loadable path, that
`--native-dynamic-build` can execute that plan and stage the built loadable artifact into the package
before package payload reports and file manifests are written, that an explicit signer command can
mutate staged loadable artifacts before package payload reports and file manifests are sealed, that
the signing report records before/after hashes and command execution, and that signing failures clean
the staged payload atomically, that an explicit notarization/post-processing command runs after
signing but before package payload reports and file manifests are sealed, that the notarization
report records before/after hashes and command execution, and that notarization platform mismatches
clean the staged payload atomically,
that each `native_dynamic_package.toml` records its package-local payload files and hash, that stale
unselected package directories are removed before the new payload manifest is computed, that
platform-specific packages do not copy foreign-platform dynamic library/debug symbol artifacts, that
platform debug symbols cannot replace a loadable native library, that package-level stage reports
expose loadable artifact paths/counts for audit, that macOS `.dSYM`
debug symbol bundles are copied recursively beside `.dylib` artifacts and included in deterministic
file manifests, that packages failing artifact filtering are removed instead of remaining as partial
payload directories, that any
package failure clears the whole staged `plugins/` payload instead of
leaving partially successful packages publishable, that fatal cleanup is reflected in the stage report
with `payload_cleaned` and `cleanup_reason`, that missing Rust-side `package_report` fields are
derived before writing the loader manifest, that sanitized package directories are accepted, that
package directories which do not match their `package_id`-derived value are rejected, and that
inconsistent package `path`/`manifest`/`package_report` rows are rejected before writing the loader manifest,
that duplicate `package_id` entries are
rejected before writing the loader manifest, that direct source package manifest parse errors,
missing ids, and id mismatches are specific fatal diagnostics, that recursive package source search
rejects multiple `plugin.toml` matches for the same selected package id, that non-v3 ABI package
exports and mismatched ABI v3 contract names are rejected before materialization, that the selected package list and
export table must match exactly, that duplicate selected package ids are rejected before
materialization, and that a missing selected package is fatal without leaving
`plugins/native_plugins.toml` behind.

The NativeDynamic M5-T1 coverage verifies that Validate report exposes
`native_dynamic_package_exports`, that generated loader manifests deserialize optional ABI v3
contract fields, and that materialized native package directories include
`native_dynamic_package.toml`. The scoped validator Cargo check passed under
`D:\cargo-targets\zircon-export-m5-native-dynamic-0614` with existing warning noise.
`native_dynamic_only_profile_carries_minimal_compile_host_plan` verifies that a NativeDynamic-only
profile carries a minimal CompileHost plan for the final host while keeping dynamic packages out of
linked runtime crates; the scoped `cargo check -p zircon_runtime --lib --no-default-features
--features core-min` passed under
`D:\cargo-targets\zircon-plugin-native-dynamic-host-plan-check-0615`, but the focused Rust test
timed out during lib-test compilation and is not claimed as passing.
`native_loader_loads_real_fixture_from_export_load_manifest_payload` constructs a stage-style
`plugins/native_plugins.toml` payload with the real `native_dynamic_fixture` cdylib, package
`plugin.toml`, and `native_dynamic_package.toml`, then loads it through
`NativePluginLoader.load_all_from_load_manifest(...)` and asserts the ABI v3 runtime/editor entry
reports. The scoped core-min Cargo check passed under
`D:\cargo-targets\zircon-plugin-native-dynamic-loader-manifest-check-0615`; the exact focused test
timed out after 904 seconds during lib-test execution, so this guard is type-checked but not claimed
as a passing focused runtime test.

The Python CookAssets coverage verifies that `--stage cook_assets` writes
`<out>/stages/cook_assets/assets.json` and `report.json`, preserves the cooked manifest shape, and
normalizes relative asset `source` paths before Pack reads the staged file. The Pack dry-run coverage
also verifies that omitting `--asset-manifest` selects the CookAssets default path.
The profile-filter handoff coverage verifies that main-pipeline CookAssets can receive
`profile_summary.asset_filter` from Validate report and that a manifest-declared `asset_filter`
still takes priority.
The project fallback coverage adds `test_cook_assets_derives_project_default_scene_without_manifest`,
confirming that a `zircon-project.toml` with `default_scene = "res://..."` can generate the minimal
CookAssets staged manifest when no explicit cooked manifest is available.
`test_cook_assets_reports_missing_project_default_scene_source` keeps missing fallback sources fatal
at CookAssets instead of letting them leak into Pack.

M5-T2 adds writer-level `delta_pack_contains_only_changed_chunks` for the `ZRPD` chunk-diff package
and Python dry-run coverage `test_pack_delta_args_are_forwarded_to_packer` for `--previous-pack` /
`--delta-pack` pass-through. `test_pipeline_platform_bundle_uses_pack_report_delta_pack_path` extends
the handoff: a non-fatal Pack report `delta_pack` path is now defaulted into PlatformBundle, copied
beside the full `.zrpack`, and recorded in both PlatformBundle `report.json` and `bundle.json`.
`test_template_delta_pack_path_controls_bundle_location` verifies that platform templates can route
the copied `.zrpd` through `[bundle].delta_pack_path`, matching the existing host/full-pack path
customization contract.
`test_checked_in_windows_template_routes_delta_pack_path` keeps the checked-in Windows export
template on that same contract, while the Linux and macOS fixtures now declare platform-specific
delta package locations as part of their template bundle metadata.
The runtime pack layer now also has `delta_pack_applies_to_base_pack` and
`delta_pack_rejects_wrong_base_manifest` coverage for the next lower-level hot-update primitive:
`ZrPackDeltaReader::apply_to_base` rebuilds the target full pack from a matching base pack plus
delta payload and refuses mismatched base manifests before any reconstructed bytes are accepted.
The `zircon_export_pack` binary uses that primitive as a writer self-check and reports
`delta_apply_verified`; requested delta output is fatal unless applying the written `.zrpd` to the
previous pack reconstructs the target `.zrpack` bytes.
`test_report_stage_rejects_unverified_delta_pack` keeps the final Report aggregator on the same
contract: a Pack report that contains `delta_pack` but does not carry `delta_apply_verified = true`
marks the pipeline report fatal, even if the Pack stage did not mark itself fatal.
`cargo check -p zircon_runtime --bin zircon_export_pack --locked --offline --jobs 1 --target-dir
D:\cargo-targets\zircon-export-m5-native-dynamic-0614` passed with existing warning noise. The
focused lib-test command for the delta test timed out during lib-test compilation before running the
target test.

The Pack profile handoff coverage adds `test_pack_command_forwards_profile_to_packer`, confirming
that the Python stage supplies `--profile <name>` to the Rust packer. The scoped packer check under
`D:\cargo-targets\zircon-export-pack-profile-0615` confirms the Rust `PackArgs` parser and
`ExportPackReport.profile` field compile with the real binary target. A real Pack smoke using the
same target directory confirms that the Python CLI forwards `windows-release` into the Rust packer
and that `<out>/stages/pack/report.json` writes `profile=windows-release`, `fatal=false`, one asset,
and one chunk.

The pipeline handoff coverage adds `test_pipeline_pack_uses_cook_assets_report_manifest`, confirming
that resume/main-pipeline Pack execution consumes CookAssets' `cooked_asset_manifest` report field
when the user did not supply `--asset-manifest`.

`test_pack_reports_missing_asset_manifest_before_packer` confirms Pack writes a fatal stage report
and does not invoke the packer when the cooked manifest is missing. This locks the failure path used
by resume, final Report aggregation, and the editor wizard.

The Python Report coverage verifies that `--stage report` aggregates all required upstream stage
reports into `<out>/report.json` and `<out>/stages/report/report.json`, includes
`source_template` when present, allows it to be absent for non-SourceTemplate releases, and marks a
missing required upstream stage fatal with a concrete diagnostic.
`test_report_stage_ignores_profile_mismatch_validate_strategies` keeps a wrong-profile Validate
report from selecting the current profile's required stages: the mismatched report remains fatal
evidence in `stages[]`, but its stale `profile_summary.strategies` are not used to aggregate
strategy-specific stage reports.
For NativeDynamic releases, `test_report_stage_projects_native_dynamic_release_audit` also keeps the
final pipeline report projecting PlatformBundle's stable `native_plugins_payload` summary at the
top level, so Hub/editor readers do not have to parse the nested PlatformBundle stage entry to find
payload hash, package count, signing, or notarization state.

The Python resume coverage verifies that `--resume-from pack` dry-runs Pack, PlatformBundle, and
Report without replaying earlier stages, that explicit `--stage` and `--resume-from` are rejected
together, that a fatal PlatformBundle resume stops before writing the final pipeline report, and
that `test_resume_from_ignores_stage_outside_validated_strategy` skips stale strategy stages when
the Validate report says the current profile does not request them.
The same coverage now locks the default command surface: omitting `--stage` dispatches to the main
pipeline starting at Validate instead of silently running only the Validate stage.
It also verifies that a pipeline PlatformBundle run consumes CompileHost's staged
`host_executable` report field plus Pack's `pack` and optional `delta_pack` report fields before
aggregating the final report, so custom stage output paths survive resume execution and hot-update
artifacts are not stranded in the Pack stage.
`test_platform_bundle_failure_cleans_previous_profile_bundle` covers repeat execution after a
successful bundle: when a later PlatformBundle run fails, stale host, pack, and final bundle
manifest files from the previous run are removed instead of remaining publishable beside a fatal
stage report. The missing-template-match branch also asserts that pre-materialization fatal paths
leave no empty profile bundle directory and no `bundle_manifest` path in the stage report.

M2-T4 adds `deterministic_pack_double_run_byte_identical` for the writer-level byte guarantee and a
real CLI Pack smoke with `--determinism-check`. The smoke passed on 2026-06-14 with
`fatal=false`, two included assets, one trimmed unused/editor-only asset, and a successful
`deterministic pack double-run byte comparison passed` diagnostic. PlatformBundle has a stage report
and directory layout, but startup-to-first-frame validation remains deferred until CompileHost and
CookAssets can feed it real host and cooked asset outputs.

M3-T1 adds `tools.zircon_export.tests.test_templates`. The plan-named
`template_version_mismatch_rejected` check mutates a copied template manifest to
`format_version = 999` and asserts a fatal validation report. The valid-template check proves the
checked-in `export-templates/windows-x86_64-library_embed-debug/template.toml` resolves its declared
host path and computed content hash. `test_template_rejects_aliasing_file_and_host_paths` mutates a
copied template to use `bin/./zircon_runtime.host-placeholder` for both host and file paths, then
recomputes the aggregate content hash; validation must still fail so path aliases cannot become
part of a version-locked template package.

M3-T2 layout coverage in the same Python test module checks the signed-in
`linux-x86_64-library_embed-debug` and `macos-aarch64-library_embed-debug` fixtures. It verifies the
Linux directory output paths and the macOS `.app/Contents` output paths, including template-file copy
for `Info.plist`. These are layout tests only, not executable launch tests.

M3-T2 template-root coverage verifies that `--template-root export-templates --target-platform
linux-x86_64` resolves the checked-in Linux template, records `template_resolution`, materializes the
Linux directory layout, and returns a fatal report when no compatible profile/platform template is
found. `test_template_root_skips_invalid_matching_template_candidate` keeps template-root resolution
from letting a corrupted template package shadow a valid package for the same profile/platform:
matching candidates are fully validated before selection, invalid matches are recorded under
`template_resolution.skipped_candidates`, and only valid candidates participate in the duplicate
candidate check. `test_template_root_skips_malformed_template_manifest` applies the same audit path
to a child template whose `template.toml` cannot be parsed; a malformed package is visible in
`skipped_candidates` but does not block a separate valid template from being selected.

PlatformBundle NativeDynamic handoff coverage adds
`test_platform_bundle_copies_native_dynamic_plugins_dir` and
`test_pipeline_platform_bundle_uses_native_dynamic_report_plugins`. Together they verify that an
explicit `--native-plugins-dir` is copied into the final bundle `plugins/` directory and that
pipeline execution can default that input from a non-fatal NativeDynamic stage report.
The explicit-directory path now also records a directory-level `native_plugins_payload` with the
current content hash, file manifest, package count, and loadable artifact audit in both the
PlatformBundle stage report and `bundle.json`. After PlatformBundle copies the plugins payload, the
final payload's `materialized_packages` summary rewrites package `destination` and `package_report`
paths to the final `bundle/plugins/...` locations while keeping `source` and `stage_report` as
upstream provenance. When a NativeDynamic payload is present, PlatformBundle recreates the final
`bundle/plugins/` directory immediately before copying that payload, so template-provided files under
the same directory cannot remain as untracked release plugins; template file report entries targeting
that removed directory are also dropped so `template_files` only describes files still present in the
final bundle. `test_platform_bundle_rejects_malformed_native_dynamic_report` keeps a matching
malformed NativeDynamic report fatal so damaged stage evidence cannot be bypassed by an explicit
directory argument.
`test_platform_bundle_explicit_native_dir_uses_bundle_plugin_paths` verifies that a manual source
directory such as `manual-native-payload/` still produces `plugins/...` payload paths and matching
loadable artifact paths, so hashes and release audit metadata describe the final bundle layout
rather than a caller's temporary directory name.
`test_platform_bundle_native_plugins_replaces_template_plugins_dir` verifies that template files
copied into `plugins/` are removed before the NativeDynamic payload is copied, keeping the final
plugin directory and `template_files` report aligned with `native_plugins_payload.file_manifest`.
`test_pipeline_platform_bundle_preserves_native_dynamic_payload_hash` extends the handoff so the
final PlatformBundle report and `bundle.json` retain the staged NativeDynamic payload hash, file
manifest metadata, per-package loadable artifact audit, and stable signing/notarization operation
summaries. `test_pipeline_platform_bundle_rejects_stale_native_dynamic_payload_hash` keeps
a mutated staged plugin payload fatal before PlatformBundle copies it into the final bundle.
The final Report stage now mirrors that PlatformBundle `native_plugins_payload` into the top-level
pipeline report only when the PlatformBundle stage is non-fatal, while the full per-stage evidence
remains available under `stages[]`. `test_report_stage_does_not_project_fatal_platform_bundle_payload`
keeps failed bundles from exposing a top-level NativeDynamic payload that Hub/editor readers might
mistake for a consumable release audit. `test_report_stage_does_not_project_profile_mismatch_platform_bundle_payload`
applies the same guard when a PlatformBundle report belongs to another profile; profile mismatches
mark the stage wrapper fatal before any top-level release payload is projected.

After M2-T1, the Validate binary still passes:

```powershell
cargo check -p zircon_runtime --bin zircon_export_validate --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m1-validate-0614
```

Focused `zircon_runtime` lib tests remain blocked by unrelated UI test compile drift, so the
CompileHost matrix tests are written but not fully executed through Cargo. The NativeDynamic-only
minimal-host Rust test also timed out during lib-test compilation; the production `core-min` library
check for the touched export build-plan code passes.

The real NativeDynamic fixture smoke used the checked-in `native_dynamic_fixture` package:

```powershell
python -m zircon_export --profile native-dynamic-fixture-smoke --repo-root E:\Git\ZirconEngine --out D:\zircon-native-dynamic-real-fixture-smoke-0615\out --stage native_dynamic --native-dynamic-build --offline --pretty
```

It returned `fatal=false`, built one cdylib package with Cargo exit code `0`, and staged the
fixture `.dll` plus `.pdb` sidecar into the NativeDynamic payload.

The ABI v2 fixture feature smoke used the same package with the `abi_v2_only` Cargo feature:

```powershell
python -m zircon_export --profile native-dynamic-fixture-v2-smoke --repo-root E:\Git\ZirconEngine --out D:\zircon-native-dynamic-real-fixture-v2-smoke-0615\out --stage native_dynamic --native-dynamic-build --native-dynamic-build-feature abi_v2_only --offline --pretty
```

It returned `fatal=false`, wrote `native_build_plan.build_features = ["abi_v2_only"]`, built the
cdylib with Cargo exit code `0`, and staged the fixture `.dll` plus `.pdb` sidecar. This proves the
feature-matrix hook and ABI v2 fixture build path, not runtime loading of the fallback ABI.

The same checked-in fixture also has release-mode stage smokes:

```powershell
python -m zircon_export --profile native-dynamic-fixture-release-smoke --repo-root E:\Git\ZirconEngine --out D:\zircon-native-dynamic-real-fixture-release-smoke-0615\out --stage native_dynamic --native-dynamic-build --offline --pretty
python -m zircon_export --profile native-dynamic-fixture-release-v2-smoke --repo-root E:\Git\ZirconEngine --out D:\zircon-native-dynamic-real-fixture-release-v2-smoke-0615\out --stage native_dynamic --native-dynamic-build --native-dynamic-build-feature abi_v2_only --offline --pretty
```

Both returned `fatal=false`, wrote `cargo_profile = "release"`, completed Cargo with exit code `0`,
and staged the fixture `.dll` plus `.pdb` sidecar. Taken together, the local Windows fixture matrix
now covers debug/release and default/`abi_v2_only` build variants. Linux/macOS cross-platform
fixture execution and runtime load/startup evidence remain pending.

The M1 testing stage passed the validator binary check and a real CLI Validate run:

```powershell
cargo check -p zircon_runtime --bin zircon_export_validate --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m1-validate-0614
python -m zircon_export --profile windows-release --project D:\zircon-export-m1-smoke\project\zircon-project.toml --out D:\zircon-export-m1-smoke\run-valid --stage validate --offline --target-dir D:\cargo-targets\zircon-export-m1-validate-0614
```

The real smoke wrote `<out>/stages/validate/report.json`, returned non-fatal JSON, and confirmed the
report contained selected linked crates such as `zircon_plugin_rendering_runtime` and
`zircon_plugin_net_http_runtime`. Focused runtime lib-test execution is still blocked by unrelated
UI test compile drift in `table_pointer_routes.rs`, so no `cargo test -p zircon_runtime --lib`
pass is claimed yet.

The M2 Pack smoke used:

```powershell
python -m zircon_export --profile windows-release --out D:\zircon-export-m2-smoke --stage pack --asset-manifest D:\zircon-export-m2-smoke\assets\assets.json --determinism-check --offline --target-dir D:\cargo-targets\zircon-export-m1-validate-0614 --pretty
```

It wrote `<out>/stages/pack/assets.zrpack` and `<out>/stages/pack/report.json`. The report returned
`fatal=false`, included `scenes/main.zscene` and `textures/hero.png`, trimmed
`textures/unused.png`, and set `deterministic_double_run=true`.

The CookAssets project fallback smoke used:

```powershell
python -m zircon_export --profile windows-release --project D:\zircon-export-cook-project-smoke\project\zircon-project.toml --out D:\zircon-export-cook-project-smoke\out --stage cook_assets --asset-filter shipping --pretty
```

It returned `fatal=false`, `generated_from_project=true`, and one staged root
`scenes/main.scene.toml`. The staged manifest stores the resolved source path under the project
`assets/` directory and labels the entry `shipping` because the temporary fallback received
`--asset-filter shipping`.

The Pack profile smoke used:

```powershell
python -m zircon_export --profile windows-release --out D:\zircon-export-pack-profile-smoke\out --stage pack --asset-manifest D:\zircon-export-pack-profile-smoke\assets\assets.json --target-dir D:\cargo-targets\zircon-export-pack-profile-0615 --offline --pretty
```

It returned `fatal=false`, wrote `profile=windows-release` in the Pack report, and packed one asset
into one chunk through the real `zircon_export_pack` binary.

The Pack missing-manifest smoke used the default CookAssets handoff path without creating
`<out>/stages/cook_assets/assets.json`:

```powershell
python -m zircon_export --profile windows-release --out D:\zircon-export-pack-missing-manifest-smoke --stage pack --pretty
```

It returned exit code `2`, wrote `<out>/stages/pack/report.json`, and recorded `fatal=true`,
`profile=windows-release`, and a diagnostic telling the caller to run CookAssets first or pass
`--asset-manifest`.

The M3-T1 template smoke used a placeholder pack and the checked-in template package:

```powershell
python -m zircon_export --profile windows-release --out D:\zircon-export-m3-template-smoke --stage platform_bundle --pack-file D:\zircon-export-m3-template-smoke\inputs\assets.zrpack --template-dir E:\Git\ZirconEngine\export-templates\windows-x86_64-library_embed-debug --target-platform windows-x86_64
```

It returned `fatal=false`, copied the template-declared host placeholder and pack into
`bundle/windows-release`, and wrote a report containing the validated template manifest, file hash,
and computed aggregate `content_hash`. A second smoke with `format_version = 999` returned exit code
`2`, recorded `template format_version 999 is not supported; expected 1`, and skipped bundle copy.

The M3-T2 template-root smoke used the checked-in template repository:

```powershell
python -m zircon_export --profile linux-release --out D:\zircon-export-template-root-smoke --stage platform_bundle --pack-file D:\zircon-export-template-root-smoke\inputs\assets.zrpack --template-root E:\Git\ZirconEngine\export-templates --target-platform linux-x86_64
```

It returned `fatal=false`, resolved `linux-x86_64-library_embed-debug`, wrote
`template_resolution`, and materialized `bundle/linux-release/ZirconRuntime`,
`bundle/linux-release/data/assets.zrpack`, and `bundle/linux-release/zircon-export.json`.
