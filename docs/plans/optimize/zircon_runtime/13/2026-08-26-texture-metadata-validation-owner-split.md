# Runtime 13 texture metadata validation owner split

## Scope

- Target: `zircon_runtime/src/core/framework/render/image/metadata_validation.rs`.
- Baseline: clean 723-line tracked owner containing the public diagnostic contract, metadata rule route, container-format classification tables, and inline tests.
- Priority sources: render plan 13, the engine structure convention/review findings, Runtime texture review 92, and Editor texture review 109.
- This slice changes ownership only. It does not change a validation rule, format whitelist, diagnostic string, public API, importer artifact, runtime upload path, or texture algorithm.

## Architecture review before optimization

The current validator is a useful narrow metadata guard, but it is not a complete texture schema authority. It accepts a free-form `&str` format token and locally classifies DDS, KTX, KTX2, ASTC, float-family, sRGB-family, and runtime-mip support. It cannot prove the distinction between requested compression and the actual immutable artifact, nor can it qualify a format by platform capability, build recipe, artifact generation, residency state, or runtime install receipt.

The primary local Unreal references were `Engine/Classes/Engine/Texture.h`, `Developer/TextureCompressor/Public/TextureCompressorModule.h`, `Developer/TextureBuildUtilities/Public/TextureBuildUtilities.h`, and `Runtime/Engine/Public/TextureResource.h`. They separate authored texture settings, build/compression settings and services, build metadata, and runtime RHI resources. Zircon should preserve the same direction of separation instead of expanding the string validator into a build or runtime-resource facade.

Runtime review 92 and Editor review 109 already identify the structural target: a unique texture schema/build/artifact/upload/residency authority, with a versioned source recipe, platform-qualified immutable artifact, bulk mip/page identity, and generation-qualified runtime install. Those P0/P1 items remain open. This owner split only makes the current metadata guard replaceable and auditable.

## Implemented layout

| Owner | Responsibility | Current lines |
|---|---|---:|
| `metadata_validation.rs` | Public diagnostics and unchanged metadata rule route | 173 |
| `metadata_validation/format.rs` | DDS/KTX/KTX2/ASTC and scalar format classification | 162 |
| `metadata_validation/tests.rs` | Existing metadata behavior tests | 394 |

The public `TextureMetadataDiagnosticSeverity`, `TextureMetadataDiagnostic`, and `validate_texture_metadata` paths remain unchanged. The facade imports only four private classification operations from the format owner.

## Preserved invariants

- All 13 original production function definitions remain present.
- All 61 production string literals match the baseline as a multiset with zero difference.
- All 18 original test function names remain present with no added or removed behavior test.
- Rule order, error/warning severity, and diagnostic text are unchanged.
- DDS DXGI values, KTX GL internal formats, KTX2 Vulkan formats, ASTC 2D block whitelist, HDR float-family formats, sRGB-family formats, runtime mip storage formats, and compression labels are unchanged.
- The facade contains no DDS, KTX, KTX2, ASTC, or DXGI table token after the split.

## Current evidence and status

- Scoped `rustfmt --edition 2021` completed for all three Rust owners.
- Static migration comparison retained functions `13/13`, production string literals `61/61`, and tests `18/18`, all with zero difference.
- Root size changed from 723 to 173 lines; all three owners are below 400 lines.
- Scoped whitespace/conflict scans and tracked-root `git diff --check` passed; Git emitted only the repository's LF/CRLF checkout notice.
- Managed Cargo and behavior validation were not requested while bypassing the shared validation blocker.
- Status is `render_plan13_texture_metadata_validation_owner_split_implemented_static_passed_managed_validation_deferred_algorithm_unchanged`.

## Required structural and performance follow-up

Before changing format classification or claiming a performance improvement, profile representative import/cook and runtime-install workloads by source/container/platform family. Capture validation invocations and bytes, parse/classification CPU p50/p95/p99, allocations, build-cache hit ratio, requested-versus-actual format, artifact bytes, upload bytes, first-resident latency, replacement latency, residency churn, and render-thread/queue wait. Compare cold/warm DDS, KTX/KTX2, ASTC, HDR/EXR, compressed mip-chain, reload, device-loss, and budget-pressure cases against the retained implementation and an Unreal reference workload of equivalent content and quality.

The next structural implementation must not add another string table. It must converge on the Runtime 92 service boundary and the Editor 109 source/recipe/artifact/install chain, then hard-cut consumers to the typed authority. No CPU, GPU, memory, latency, energy, power, or Unreal-parity improvement is claimed in this record.
