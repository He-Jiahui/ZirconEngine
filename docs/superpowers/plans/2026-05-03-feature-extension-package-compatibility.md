# Feature Extension Package Compatibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a compatibility mode where an independent plugin package can provide an optional feature that is projected under another owner plugin.

**Architecture:** Keep owner-plugin optional feature selection as the user-facing model. Add provider package metadata so runtime, native loading, and export can distinguish owner-embedded features from external feature-extension packages.

**Tech Stack:** Rust, serde/TOML manifests, zircon_runtime plugin catalog/export/native loader, zircon_app bootstrap, targeted Cargo validation.

---

## Milestone 1: Core Manifest And Catalog Support

**Files:**
- Modify: `zircon_runtime/src/plugin/package_manifest/plugin_package_kind.rs`
- Modify: `zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs`
- Modify: `zircon_runtime/src/plugin/package_manifest/constructors.rs`
- Modify: `zircon_runtime/src/plugin/package_manifest/mod.rs`
- Modify: `zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_feature_selection.rs`
- Modify: `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs`
- Test: `zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs`

- [x] Add `PluginPackageKind::{Standard, FeatureExtension}` with serde snake_case and default `Standard`.
- [x] Add `package_kind` to `PluginPackageManifest` and constructor helpers.
- [x] Add `provider_package_id` to `ProjectPluginFeatureSelection`, preserving old manifests with `Option<String>`.
- [x] Teach catalog completion to project external `FeatureExtension` package features under their `owner_plugin_id` with `provider_package_id = external package id`.
- [x] Teach feature dependency resolution to require an enabled provider package only when `provider_package_id != owner_plugin_id`.
- [x] Add roundtrip, completion, provider-missing, provider-enabled, duplicate-provider, and internal-owner compatibility tests.

## Milestone 2: Export And Native Loading

**Files:**
- Modify: `zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs`
- Modify: `zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs`
- Modify: `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs`
- Modify: `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_feature_registration_report.rs`
- Test: `zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs`

- [x] Link embedded external feature packages from `zircon_plugins/<provider_package_id>/runtime` when selected as `LibraryEmbed`.
- [x] Select native dynamic provider packages independently when a selected feature has `provider_package_id` and `packaging = NativeDynamic`.
- [x] Preserve `provider_package_id` in generated `ProjectPluginFeatureSelection` code.
- [x] Keep native loader projecting feature-extension package manifests into feature registration reports while avoiding base plugin registration reports for pure feature-extension packages.
- [x] Add export tests for embedded and native external feature providers.

## Milestone 3: Documentation And Validation

**Files:**
- Modify: `docs/engine-architecture/plugin-optional-feature-bundles.md`
- Modify: `.codex/sessions/20260502-1956-plugin-optional-feature-bundles.md`

- [x] Document external feature provider rules, conflict handling, and export/native behavior.
- [x] Run `cargo fmt -p zircon_runtime -p zircon_app -p zircon_editor`.
- [x] Run `cargo metadata --locked --no-deps --format-version 1`.
- [x] Run targeted `zircon_runtime` plugin extension tests with `CARGO_INCREMENTAL=0`.
- [x] Run `git diff --check` on touched files.
