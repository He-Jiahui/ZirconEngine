# Plugins05 borrowed shader source validation optimization record

- Date: 2026-08-21
- Owner plan: `docs/plans/optimize/zircon_plugins/05-shader-wgsl-family-importer-compiler-artifact-native-product-integration-review.md`
- Finding: `P1-13`
- Status: `validation_pending`

## Scope

- Add an allocation-free UTF-8 view over `AssetImportContext::source_bytes`.
- Parse and validate WGSL/GLSL through the borrowed view in both shader importer providers.
- Delay creation of the final owned shader source until Naga parsing and validation succeed.
- Keep `source_text()` as the compatible owned convenience API for existing importers.

## Contract

- `source_str()` returns a slice backed by the context byte buffer and performs no allocation for valid UTF-8.
- Invalid UTF-8 retains the existing `AssetImportError::SourceTextDecode` and owned `FromUtf8Error` source chain.
- Accepted shader assets still own their source and emitted WGSL strings with unchanged contents.
- Shader parsing, diagnostics, entry-point projection, validation flags, and capability policy are unchanged.

## Performance Gate

- The release workload parses a 1 MiB valid-UTF-8 but syntactically invalid WGSL source 32 times per sample.
- Owned validation clones 33,554,432 source bytes per sample; borrowed validation clones zero, a deterministic 100% reduction.
- The gate uses 21 alternating owned/borrowed sample pairs and nearest-rank P95.
- Borrowed validation P95 must be no more than 85% of owned validation P95. Measured timings remain pending the grouped coordinator validation.

## Validation

- The context regression checks that the borrowed string pointer and length exactly match `source_bytes`.
- The WGSL provider regression checks that invalid borrowed source still produces a URI-qualified shader validation error.
- Existing WGSL and GLSL accepted-asset tests retain source language, entry point, and emitted WGSL assertions.
- The release performance marker is `PERF-MVP-PLUGINS05-BORROWED-SHADER-SOURCE`.
- Cargo compilation, behavior tests, and release measurements are queued in the multi-task Plugins aggregate; no standalone Cargo run is claimed here.

## Remaining Plan Work

- This slice removes the eager source clone from validation failures in both current shader providers.
- Accepted assets still require owned source strings under the existing `ShaderAsset` schema; shared immutable source storage is separate architecture work.
- Target capability binding, stage ambiguity, reflected pipeline layouts, structured diagnostics, and source/dist behavior equivalence remain open Plugins05 milestones.
