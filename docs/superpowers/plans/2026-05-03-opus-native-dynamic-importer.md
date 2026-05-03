# Opus NativeDynamic Importer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a split Opus audio importer plugin package that owns the `.opus` NativeDynamic/libopus importer contract and stable missing-backend diagnostics.

**Architecture:** Keep Opus as a leaf importer package under `zircon_plugins/opus_importer/runtime`, separate from `audio_importer` WAV/Symphonia decoding and from `sound` mixer/runtime work. The linked Rust package uses `LibraryEmbed`, registers a `Sound` importer descriptor for `.opus`, requires the NativeDynamic asset import capability for the future libopus backend, and wins registry selection over the older diagnostic Opus row when both packages are installed.

**Tech Stack:** Rust 2021, `zircon_runtime` asset importer registry, `RuntimeExtensionRegistry`, `PluginPackageManifest`, NativeDynamic `ZRIMP001`/`ZRIMO001` importer envelope, Cargo workspace `zircon_plugins/Cargo.toml`.

---

## Current Baseline

- Repository policy requires working on existing `main`, no worktree or feature branch.
- `zircon_plugins/audio_importer/runtime/src/lib.rs` currently registers real WAV and Symphonia-backed codec importers plus diagnostic `audio_importer.opus` at priority `80`.
- `zircon_plugins/asset_importers/audio/runtime/src/lib.rs` is a family manifest package and does not decode audio.
- `zircon_runtime/src/asset/importer/native.rs` owns the existing NativeDynamic importer envelope and response validation path.
- `docs/zircon_runtime/asset/importer.md` and `docs/zircon_plugins/asset_importers/runtime-skeletons.md` currently describe Opus as a remaining NativeDynamic/libopus gap.
- A fresh coordination scan found active sound mixer graph work. Do not edit `zircon_plugins/sound/**` or `zircon_runtime/src/core/framework/sound/**` for this plan.

## File Structure

- Create `zircon_plugins/opus_importer/runtime/Cargo.toml`: package manifest for the split Opus runtime importer crate.
- Create `zircon_plugins/opus_importer/runtime/src/lib.rs`: crate root containing descriptor constants, package/module manifests, runtime selection, registration, missing-backend importer shim, and package-local tests. Keep this file focused; do not add actual libopus decoding here.
- Modify `zircon_plugins/Cargo.toml`: add `opus_importer/runtime` as a workspace member near other importer packages.
- Modify `docs/zircon_runtime/asset/importer.md`: update built-in/importer coverage and validation evidence for the Opus split package.
- Modify `docs/zircon_plugins/asset_importers/runtime-skeletons.md`: update split package and legacy family descriptions for Opus.
- Modify `.codex/sessions/20260503-1940-importer-gap-design.md`: update current step/checks after implementation and validation.

## Milestone 1: Split Opus Importer Package

### Goal

Introduce the split Opus importer package and prove its manifest, registration, selection priority, and deterministic missing-backend behavior without touching playback or sound framework code.

### In-Scope Behaviors

- `opus_importer` package manifest declares package id, runtime crate, supported target modes, supported platforms, module manifest, and its linked Rust plugin/Opus slot capabilities.
- Runtime registration contributes one module and one `.opus` asset importer descriptor.
- The Opus descriptor outputs `AssetKind::Sound`, uses importer version `1`, advertises extension `opus`, requires `runtime.asset.importer.audio.opus` and `runtime.asset.importer.native`, and has priority higher than `audio_importer.opus` priority `80`.
- Importing `.opus` without an installed native backend fails with a stable diagnostic string that mentions NativeDynamic/libopus.
- A registry containing both `audio_importer` and `opus_importer` chooses the split Opus importer for `.opus`.
- Documentation records that the split package owns the Opus slot while real decoding still requires the NativeDynamic/libopus backend.

### Dependencies

- Existing `zircon_runtime::asset::{AssetImporterDescriptor, DiagnosticOnlyAssetImporter, FunctionAssetImporter or AssetImporterHandler, AssetKind}` registry contracts.
- Existing `zircon_runtime::plugin::{PluginPackageManifest, PluginModuleManifest, RuntimeExtensionRegistry, RuntimePluginRegistrationReport}` manifest/registration contracts.
- Existing `audio_importer.opus` diagnostic row remains available as the lower-priority fallback.

### Implementation Slices

- [ ] **Slice 1: Add workspace member and crate manifest**

  Modify `zircon_plugins/Cargo.toml` by adding the new member immediately after `audio_importer/runtime`:

  ```toml
  members = [
      "editor_support",
      "gltf_importer/runtime",
      "obj_importer/runtime",
      "texture_importer/runtime",
      "audio_importer/runtime",
      "opus_importer/runtime",
      "shader_wgsl_importer/runtime",
      "ui_document_importer/runtime",
      "asset_importers/model/runtime",
  ]
  ```

  Create `zircon_plugins/opus_importer/runtime/Cargo.toml`:

  ```toml
  [package]
  name = "zircon_plugin_opus_importer_runtime"
  version.workspace = true
  edition.workspace = true
  license.workspace = true
  description = "Opus audio asset importer runtime plugin package for Zircon."

  [dependencies]
  zircon_runtime = { path = "../../../zircon_runtime", default-features = false }

  [dev-dependencies]
  zircon_plugin_audio_importer_runtime = { path = "../../audio_importer/runtime" }
  ```

- [ ] **Slice 2: Add package constants, manifests, and descriptor**

  Create `zircon_plugins/opus_importer/runtime/src/lib.rs` with this initial structure:

  ```rust
  use zircon_runtime::asset::{
      AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
      DiagnosticOnlyAssetImporter,
  };
  use zircon_runtime::core::ModuleDescriptor;
  use zircon_runtime::{
      plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginModuleManifest,
      plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
      plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError,
      plugin::RuntimePluginRegistrationReport, RuntimeTargetMode,
  };

  pub const PLUGIN_ID: &str = "opus_importer";
  pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_opus_importer_runtime";
  pub const MODULE_NAME: &str = "OpusImporterModule";
  pub const OPUS_IMPORTER_ID: &str = "opus_importer.opus";
  pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.opus_importer";
  pub const OPUS_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.audio.opus";
  pub const NATIVE_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.native";
  pub const OPUS_IMPORTER_PRIORITY: i32 = 130;
  const MISSING_BACKEND_DIAGNOSTIC: &str =
      "opus import requires a NativeDynamic libopus backend";

  pub fn runtime_capabilities() -> &'static [&'static str] {
      &[
          RUNTIME_CAPABILITY,
          OPUS_IMPORTER_CAPABILITY,
      ]
  }

  pub fn supported_targets() -> [RuntimeTargetMode; 2] {
      [
          RuntimeTargetMode::ClientRuntime,
          RuntimeTargetMode::EditorHost,
      ]
  }

  pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
      [
          ExportTargetPlatform::Windows,
          ExportTargetPlatform::Linux,
          ExportTargetPlatform::Macos,
      ]
  }

  pub fn module_descriptor() -> ModuleDescriptor {
      ModuleDescriptor::new(MODULE_NAME, "Opus audio importer plugin")
  }

  pub fn asset_importer_descriptor() -> AssetImporterDescriptor {
      AssetImporterDescriptor::new(OPUS_IMPORTER_ID, PLUGIN_ID, AssetKind::Sound, 1)
          .with_priority(OPUS_IMPORTER_PRIORITY)
          .with_source_extensions(["opus"])
          .with_required_capabilities([OPUS_IMPORTER_CAPABILITY, NATIVE_IMPORTER_CAPABILITY])
  }

  pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
      vec![asset_importer_descriptor()]
  }

  pub fn package_manifest() -> PluginPackageManifest {
      let mut manifest = PluginPackageManifest::new(PLUGIN_ID, "Opus Audio Importer")
          .with_category("asset_importer")
          .with_supported_targets(supported_targets())
          .with_supported_platforms(supported_platforms())
          .with_capabilities(runtime_capabilities().iter().copied())
          .with_runtime_module(runtime_module_manifest());
      for importer in asset_importer_descriptors() {
          manifest = manifest.with_asset_importer(importer);
      }
      manifest
  }

  pub fn runtime_module_manifest() -> PluginModuleManifest {
      PluginModuleManifest::runtime("opus_importer.runtime", RUNTIME_CRATE_NAME)
          .with_target_modes(supported_targets())
          .with_capabilities(runtime_capabilities().iter().copied())
  }

  pub fn runtime_selection() -> ProjectPluginSelection {
      ProjectPluginSelection {
          id: PLUGIN_ID.to_string(),
          enabled: true,
          required: false,
          target_modes: supported_targets().to_vec(),
          packaging: ExportPackagingStrategy::LibraryEmbed,
          runtime_crate: Some(RUNTIME_CRATE_NAME.to_string()),
          editor_crate: None,
          features: Vec::new(),
      }
  }
  ```

- [ ] **Slice 3: Add registration and deterministic missing-backend import path**

  Continue `zircon_plugins/opus_importer/runtime/src/lib.rs` with registration and import behavior:

  ```rust
  pub fn plugin_registration() -> RuntimePluginRegistrationReport {
      let mut extensions = RuntimeExtensionRegistry::default();
      let mut diagnostics = Vec::new();
      if let Err(error) = register_runtime_extensions(&mut extensions) {
          diagnostics.push(error.to_string());
      }
      RuntimePluginRegistrationReport {
          package_manifest: package_manifest(),
          project_selection: runtime_selection(),
          extensions,
          diagnostics,
      }
  }

  pub fn register_runtime_extensions(
      registry: &mut RuntimeExtensionRegistry,
  ) -> Result<(), RuntimeExtensionRegistryError> {
      registry.register_module(module_descriptor())?;
      registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
          asset_importer_descriptor(),
          MISSING_BACKEND_DIAGNOSTIC,
      ))?;
      Ok(())
  }

  pub fn import_opus_missing_backend(
      context: &AssetImportContext,
  ) -> Result<AssetImportOutcome, AssetImportError> {
      Err(AssetImportError::UnsupportedFormat(format!(
          "decode opus {}: {MISSING_BACKEND_DIAGNOSTIC}",
          context.source_path.display()
      )))
  }
  ```

  If `import_opus_missing_backend` is unused after using `DiagnosticOnlyAssetImporter`, remove it before validation. Do not keep unused code only to describe future libopus behavior.

- [ ] **Slice 4: Add package-local tests**

  Add this `#[cfg(test)]` module to `zircon_plugins/opus_importer/runtime/src/lib.rs`:

  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use zircon_runtime::asset::{AssetImportContext, AssetImporterRegistry, AssetUri};

      #[test]
      fn package_declares_opus_native_dynamic_importer() {
          let manifest = package_manifest();

          assert_eq!(manifest.id, PLUGIN_ID);
          assert!(manifest
              .capabilities
              .contains(&RUNTIME_CAPABILITY.to_string()));
          assert!(manifest
              .capabilities
              .contains(&OPUS_IMPORTER_CAPABILITY.to_string()));
          assert!(!manifest
              .capabilities
              .contains(&NATIVE_IMPORTER_CAPABILITY.to_string()));
          assert!(manifest.default_packaging.contains(&ExportPackagingStrategy::LibraryEmbed));
          assert!(!manifest.default_packaging.contains(&ExportPackagingStrategy::NativeDynamic));
          assert_eq!(manifest.asset_importers.len(), 1);
          let importer = &manifest.asset_importers[0];
          assert_eq!(importer.id, OPUS_IMPORTER_ID);
          assert_eq!(importer.output_kind, AssetKind::Sound);
          assert_eq!(importer.importer_version, 1);
          assert!(importer.source_extensions.contains(&"opus".to_string()));
          assert!(importer
              .required_capabilities
              .contains(&OPUS_IMPORTER_CAPABILITY.to_string()));
          assert!(importer
              .required_capabilities
              .contains(&NATIVE_IMPORTER_CAPABILITY.to_string()));
          let selection = runtime_selection();
          assert_eq!(selection.packaging, ExportPackagingStrategy::LibraryEmbed);
      }

      #[test]
      fn registration_contributes_module_and_opus_importer() {
          let report = plugin_registration();

          assert!(report.is_success(), "{:?}", report.diagnostics);
          assert!(report
              .extensions
              .modules()
              .iter()
              .any(|module| module.name == MODULE_NAME));
          assert_eq!(report.extensions.asset_importers().descriptors().len(), 1);
          assert_eq!(
              report.extensions.asset_importers().descriptors()[0].id,
              OPUS_IMPORTER_ID
          );
      }

      #[test]
      fn opus_importer_wins_over_audio_package_diagnostic_row() {
          let audio_report = zircon_plugin_audio_importer_runtime::plugin_registration();
          let opus_report = plugin_registration();
          let mut registry = AssetImporterRegistry::default();

          for importer in audio_report.extensions.asset_importers().handlers() {
              registry.register_arc(importer.clone()).unwrap();
          }
          for importer in opus_report.extensions.asset_importers().handlers() {
              registry.register_arc(importer.clone()).unwrap();
          }

          let selected = registry.select(std::path::Path::new("voice.opus")).unwrap();

          assert_eq!(selected.descriptor().id, OPUS_IMPORTER_ID);
          assert!(selected.descriptor().priority > 80);
      }

      #[test]
      fn missing_native_backend_reports_stable_opus_diagnostic() {
          let report = plugin_registration();
          let importer = report
              .extensions
              .asset_importers()
              .select(std::path::Path::new("voice.opus"))
              .unwrap();
          let context = AssetImportContext::new(
              "voice.opus".into(),
              AssetUri::parse("res://audio/voice.opus").unwrap(),
              b"not a real opus stream".to_vec(),
              Default::default(),
          );

          let error = importer.import(&context).unwrap_err();

          assert!(error.to_string().contains("NativeDynamic libopus backend"));
      }
  }
  ```

  If `AssetImporterRegistry` does not expose `handlers()` or `register_arc(...)`, inspect `zircon_runtime/src/asset/importer/registry.rs` and adjust only the test setup to use the existing public registry transfer API. Keep the behavior assertion the same: a registry with both packages selects `OPUS_IMPORTER_ID` for `voice.opus`.

- [ ] **Slice 5: Update documentation**

  In `docs/zircon_runtime/asset/importer.md`, update the coverage paragraph around the current audio importer text. The replacement should preserve the surrounding texture/shader/UI descriptions and include this content:

  ```markdown
  The split `audio_importer` package decodes WAV directly and decodes MP3/OGG/Vorbis/FLAC/AIFF/AIF through Symphonia into `SoundAsset` f32 PCM. Opus now has a split `opus_importer` package that owns the `.opus` `SoundAsset` importer slot and NativeDynamic/libopus command contract; importing still requires an installed native backend, and missing backend cases remain stable importer errors.
  ```

  Update the heavy/toolchain gap paragraph so Opus is no longer listed as a generic missing format. It should say:

  ```markdown
  Heavy or toolchain-backed formats are registered as diagnostic importers until a plugin backend is installed. This includes FBX/DAE/3DS/USD-family model containers, cubemap/DXGI texture authoring formats, and HLSL/CG/FX shader toolchains. The Opus split package uses the same diagnostic path when its NativeDynamic/libopus backend is absent.
  ```

  In `docs/zircon_plugins/asset_importers/runtime-skeletons.md`, update the split package section to include:

  ```markdown
  `opus_importer` declares the `.opus` audio importer as a split package. It owns the `SoundAsset` importer descriptor and NativeDynamic/libopus command contract, registers ahead of the old audio-family diagnostic row, and reports a stable missing-backend diagnostic until a native libopus backend is installed.
  ```

  Update the legacy `audio` sentence so it no longer implies Opus belongs inside the audio family implementation:

  ```markdown
  `audio` declares WAV plus optional codec-backed formats such as MP3, OGG, FLAC, and AIFF; Opus is now represented by the split `opus_importer` package.
  ```

- [ ] **Slice 6: Update coordination note**

  Update `.codex/sessions/20260503-1940-importer-gap-design.md`:

  ```markdown
  ## Current Step
  - Implementing the approved Opus split importer package design from `docs/superpowers/specs/2026-05-03-opus-native-dynamic-importer-design.md` and `docs/superpowers/plans/2026-05-03-opus-native-dynamic-importer.md`.

  ## Checks / Failing Signals
  - Fresh coordination scan found active sound mixer graph work in `zircon_plugins::sound::{runtime,editor}` and `zircon_runtime::core::framework::sound`; this session avoids mixer graph, DSP, output-device, and sound framework internals unless explicitly reassigned.
  - Validation pending for `zircon_plugin_opus_importer_runtime`.
  ```

### Lightweight Checks

- During implementation, run `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_opus_importer_runtime --lib --locked --jobs 1` only if the new crate does not typecheck clearly from editor diagnostics or if registry API names need confirmation.
- Do not run full workspace tests before the milestone testing stage.

### Testing Stage

Run these commands from `E:\Git\ZirconEngine` after all implementation slices are complete:

```powershell
cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_opus_importer_runtime --check
cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_opus_importer_runtime --lib --locked --jobs 1
cargo metadata --manifest-path zircon_plugins\Cargo.toml --locked --no-deps --format-version 1
git diff --check
```

If the package test fails because the registry transfer API differs from the planned code, fix the test setup against the actual public registry API and rerun the package test. If validation fails in unrelated crates before reaching `zircon_plugin_opus_importer_runtime`, record the exact external blocker in `.codex/sessions/20260503-1940-importer-gap-design.md` and in the final report.

### Exit Evidence

- `zircon_plugin_opus_importer_runtime` package tests pass or are blocked only by a documented unrelated workspace issue.
- Cargo metadata for `zircon_plugins/Cargo.toml` passes with `--locked`.
- Docs describe Opus as split NativeDynamic/libopus package contract, not as an unowned missing gap.
- No files under `zircon_plugins/sound/**` or `zircon_runtime/src/core/framework/sound/**` are modified.

## Self-Review Notes

- Spec coverage: package boundary is covered by Slices 1-3, tests by Slice 4 and Testing Stage, docs by Slice 5, coordination by Slice 6, and validation by Testing Stage.
- Placeholder scan: no `TBD`, `TODO`, or unspecified edge handling remains.
- Type consistency risk: the only intentionally conditional API is registry handler transfer in Slice 4; the plan names the fallback action and preserves the same behavior assertion.
