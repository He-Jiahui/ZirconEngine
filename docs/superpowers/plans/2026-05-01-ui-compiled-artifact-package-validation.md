# M16 UI Compiled Artifact And Package Validation Implementation Plan

## Summary
Complete the first M16 productization gap by turning the landed M7/M10/M12/M13 UI Asset foundations into a package-facing compiled artifact pipeline: header, dependency manifest, runtime/editor package profile, validation report, deterministic binary artifact envelope, cache record, and package manifest writer/importer.

V1 intentionally kept the validation report metadata-only. The back-half landed under the same runtime package owner with binary artifact behavior plus cache-record and package-manifest contract surfaces without changing V1 validation semantics. The runtime-interface hard cutover later moved the cache-record/package-manifest DTO declarations to `zircon_runtime_interface::ui::template` while runtime retained the compiler/artifact builder behavior. Remaining work is cross-process cache storage plus resolver/runtime-loader/editor resource UX, not the artifact envelope or manifest surface.

## Back-Half Addendum
- Add the M16 back-half under the same runtime package owner without changing V1 report authority.
- Expose a deterministic runtime compiled artifact binary envelope with magic, schema version, payload length, and a typed UTF-8 TOML payload. After the runtime-interface cutover, this runtime payload wrapper is named `UiRuntimeCompiledAssetArtifact`; `UiCompiledAssetArtifact` is reserved for the interface-owned neutral `{ report, bytes }` DTO.
- Expose `UiCompiledAssetCacheRecord` so persistent cache indexes reuse the exact M12 `UiCompileCacheKey`, derived invalidation snapshot, artifact fingerprint, and byte length.
- Expose `UiCompiledAssetPackageManifest` TOML writer/importer that records the header, dependency manifest, cache record, and artifact entry.
- Consume M15 explicit `imports.resources` through `UiCompiledAssetDependencyManifest.resource_dependencies`; do not add a second resource classification surface.
- Keep resource resolver IO, runtime loader backends, graphics/RHI resource consumption, M14 localization, and M21 host action policy out of this M16 back-half.

## Key Changes
- Add the M16 package validation owner under the existing `zircon_runtime` UI asset compiler boundary.
- Expose `UiCompiledAssetHeader`, `UiCompiledAssetDependencyManifest`, `UiCompiledAssetPackageValidationReport`, and `UiCompiledAssetPackageProfile::{Runtime, Editor}`.
- Add `UiDocumentCompiler::validate_package(...)`, reusing the existing document compiler, component contract validation, descriptor registry revision, compile cache key, and invalidation fingerprints.
- Include source schema version, compiler/package schema version, descriptor registry revision, component contract revision, root document fingerprint, and compile cache key summary in the header.
- Emit deterministic widget/style dependency manifests so M12 invalidation/cache inputs become visible to future package diagnostics and M15 resource manifests.
- Emit runtime/editor retained/stripped package sections as report metadata only.

## Testing Stage
- Add focused `asset_package_validation` runtime tests for header contents, dependency ordering/fingerprints, runtime/editor profile sections, compiler state immutability, and existing compiler error propagation.
- Add focused `asset_package_validation` runtime tests for resource dependency manifest rows, binary artifact roundtrip determinism, cache record reuse of M12 inputs, and package manifest writer/importer roundtrip.
- Re-run the focused foundation regression gates: `asset_component_contract`, `asset_compile_cache`, `asset_invalidation`, `component_catalog`, and `ui::tests::asset`.
- Run a scoped `cargo check -p zircon_runtime --lib --locked --jobs 1` before closeout.

## Documentation
- Update the UI Asset docs under `docs/ui-and-layout` with the M16 report/artifact/cache-record/package-manifest behavior, related runtime owner files, and validation evidence.
- Update `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md` so M16 reads as partial-to-mostly-complete after validation evidence is available, while keeping cross-process cache storage, resolver/runtime-loader/editor resource UX, and broad workspace green outside this closeout.

## Assumptions
- M14 localization, M21 action safety/host policy, resource resolver IO, runtime loader backends, and graphics/RHI resource consumption remain out of scope.
- M15 media/font/resource dependencies are in scope only through the accepted runtime typed collector and compiled dependency manifest surface; this slice must not implement a parallel resource resolver or loader behavior.
- Current active sessions own plugin renderer, runtime UI showcase/schema panel, and runtime UI data-source areas; this M16 slice avoids those files.

## Accepted Evidence
- Runtime owner files: `zircon_runtime/src/ui/template/asset/compiler/package/{artifact.rs,header.rs,manifest.rs,package_manifest.rs,report.rs,validate.rs}` plus `zircon_runtime/src/ui/template/asset/compiler/cache/{cache_key.rs,compile_cache.rs}` and `zircon_runtime/src/ui/tests/asset_package_validation.rs`; neutral cache-record/package-manifest DTO declarations live under `zircon_runtime_interface/src/ui/template/asset/compiler/**` after the runtime-interface cutover.
- `UiRuntimeCompiledAssetArtifact` writes magic `ZRUIA016`, little-endian artifact schema version, little-endian payload length, and UTF-8 TOML payload for `UiRuntimeCompiledAssetArtifact { report, compiled: UiTemplateInstance }`.
- `UiCompiledAssetCacheRecord` stores the M12 cache key, invalidation snapshot, artifact fingerprint, and byte length for future persistent cache indexing; it is not itself a cross-process cache store.
- `UiCompiledAssetPackageManifest` TOML writer/importer records the header, dependency manifest, cache record, and artifact entry, and preserves M15 resource dependency rows from the single package-input manifest surface.
- `cargo test -p zircon_runtime --lib asset_package_validation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed with `9 passed; 0 failed`, covering resource dependency rows, binary artifact envelope roundtrip, cache record, package manifest writer/importer, runtime/editor profile sections, compiler error propagation, cache-input stability, and contract-precondition ordering.
- `cargo test -p zircon_runtime --lib asset_compile_cache --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed with `9 passed; 0 failed`.
- `cargo test -p zircon_runtime --lib asset_invalidation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed with `6 passed; 0 failed`.
- `cargo test -p zircon_runtime --lib asset_resource_refs --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed with `9 passed; 0 failed`.
- `cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed with `86 passed; 0 failed`.
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed with unrelated graphics/plugin warnings; broad workspace green is not claimed.
