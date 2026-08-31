# Plugins05 borrowed shader source validation optimization record

- Date: 2026-08-21
- Owner plan: `docs/plans/optimize/zircon_plugins/05-shader-wgsl-family-importer-compiler-artifact-native-product-integration-review.md`
- Finding: `P1-13`
- Status: `validation_pending`

## Current-source convergence receipt

- Ownership transfer preview request: `af06e1b6b96c48f98b453a8c94268707`.
- Ownership transfer apply request: `173b7dcccf4a49feb58f286384a2f1ff`.
- Applied fingerprint: `62d56b3b49fa395ad43df192d535eb75b4f6ec921fd540846a66cb05062a0d13`.
- Current session: `root-runtime-interface03-activate-link-failure-20260831`.
- Shared model: `tools/plugins05_borrowed_shader_pressure.py`, source manifest `5535B931CDBBF0A765AE72E86B5314629951115ABC606394C751599448739C4B`.
- Current source hashes: Runtime `contract.rs` `CCB2124380774DF2165628326631ACC6F03C94ED20EF93C22ADCCD2840256F0A`; WGSL provider `83BA4E135FF1D7CFEE6D9DA720C0CC275A867EBA75083A83BFEE88D1EBD2DEBF`; shader-family provider `BB916C5BAB2DC0647C5B12EEDA23F66CD1EDEFEAB43EDED01261F613D7A70418`; model `B3C81611D0741C86D72EA6C06B5BC0B598E9EE904D18B42FD18028B4E11E80FB`.
- Local focused model/source contracts: 7/7 passed; scoped Rust 1.94.1 formatting and `git diff --check` passed.
- Static/model ticket: `d6d2ce41bb0d4b378f29be699b213201` (queued, 7 Python tests).
- Release performance ticket: `587d3d9d75f64684b221a89051a7a590` (queued; exact ignored WGSL benchmark with the Runtime contract overlay).

The current-source model is structural evidence, not wall-clock timing. Across 32 syntactically invalid parses of a 1 MiB valid-UTF-8 source, owned validation performs `33,554,432` source-clone bytes and 32 source-clone allocations while borrowed validation performs zero; both retain 32 UTF-8 views and 32 parse attempts. The exact ignored 21-pair WGSL release benchmark remains authoritative for timing and must satisfy borrowed P95 `<= 85%` of owned before integration or WeCom publication.

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
