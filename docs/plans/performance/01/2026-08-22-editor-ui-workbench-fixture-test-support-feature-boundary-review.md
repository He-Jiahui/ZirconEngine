---
title: Editor workbench fixture test-support feature boundary performance review
date: 2026-08-22
module: zircon_editor/src/ui/workbench/fixture
priority: MVP-P2 build and test infrastructure
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate development automation test guards
---

# Goal

Keep shared workbench fixtures available to unit and integration-contract tests without compiling
fixture parsers, DTOs and embedded JSON into the default editor product library. Test support must
have an explicit feature/build boundary and must not become a public runtime subsystem.

## Reviewed source

- folder: `zircon_editor/src/ui/workbench/fixture`
- Rust files: 14/14
- lines: 258
- bytes: 8,546
- embedded workbench JSON bytes: 8,985
- joined source-bytes SHA256:
  `ad919944885edae59b5c2ce29a706703c7ffbe985c7d72f8a451fea54173ba55`
- owning commit before review: `08094b9b9e17f6c80372e15c17b01204038b305b`

| File | Lines | SHA256 |
| --- | ---: | --- |
| `constants.rs` | 8 | `1a774bf405f7be63910af7a35784af8f63360d7a2a88179f3806d56f9a2fedfd` |
| `default_preview_fixture.rs` | 25 | `710705949f8e9d6567cc06476826e8e4185c9cf59ae044b97d456c2f4fd2e7dd` |
| `ensure_ui_asset_descriptor.rs` | 28 | `55a6b1022b8b85251374993daa9a00c95ae71e9d600846ee5bfda5191c76300a` |
| `mod.rs` | 22 | `4373146388a65b5bb786205aaa0af6cb83d001a2fd64e0974d4b0e53548555a1` |
| `preview_editor_data_into_snapshot.rs` | 54 | `76f1216ae7d5efc72f48734e4fc82eaaad87abd3eb35726681c899d8480f0dc6` |
| `preview_editor_data.rs` | 20 | `5ce56c973d790da79effd4f4f3abc81e6686820a5a896704e6e2dbaefcdabd62` |
| `preview_fixture_build_chrome.rs` | 15 | `14cbc3f0c84ecf197b66a48cae81aefb90b7efaaa708b6ea11d7ee9f19e1cfcd` |
| `preview_fixture.rs` | 12 | `eb9aa51de276bd0cd0da0c85a52b7271c5457d22f8c084b84dc4a080ad481875` |
| `preview_gizmo_axis_into_gizmo_axis.rs` | 13 | `1f72cf2dbd856034a5c2aad6938e933addb282456e554510d64069e624734ce7` |
| `preview_gizmo_axis.rs` | 8 | `8353c0c1aca1215e3f86530e2ca277d0b99114034631721bfef52a6909cf31aa` |
| `preview_inspector_into_snapshot.rs` | 16 | `a471f634b71223bcd875bc89d5cf04001d51ae4efcf0fea9def887c66a91757c` |
| `preview_inspector.rs` | 15 | `23a2f75d1d098ef493062fe42db04f497c2310816cb0dc2aab108fe7b3beb0b9` |
| `preview_scene_entry_into_snapshot.rs` | 13 | `9b341ba85233e08f035fa1c0d512dbac464e2d632d9cc2808fe7393892c4b37e` |
| `preview_scene_entry.rs` | 9 | `f235eb27a5778ae463c314cf7505b89f8832f801099c55734555717266384b75` |

All files were read in full. Ninety-six `default_preview_fixture` references were classified: the
definition/export plus unit, integration-contract, screenshot and test-only module call sites. The
few references located inside production source files are enclosed by `#[cfg(test)]`. No editor
startup, frame, plugin or host product path calls the fixture.

## Result

### No product runtime hotspot

Fixture parsing, descriptor completion and `PreviewFixture::build_chrome` execute only in tests.
Their linear JSON parsing and small-vector clones are not an editor frame bottleneck and must not be
used to justify a product cache or new global fixture singleton. Optimizing these helpers ahead of
MVP product paths would be the wrong priority.

### P2: test support is compiled into the default product library

`ui/workbench/mod.rs` unconditionally declares `pub mod fixture`. As a result, the default editor
library compiles all 14 fixture files and their serde projection code, and `include_str!` embeds four
JSON files totaling 8,985 bytes, despite zero product consumers.

The manifest already provides an empty `integration-contracts` feature and the external
`integration_contracts` test target requires it. Unit tests have `cfg(test)`. Therefore the existing
build contract can gate the module with:

`#[cfg(any(test, feature = "integration-contracts"))] pub mod fixture;`

This is a hard ownership boundary, not a runtime compatibility shim. It should remove the fixture
surface from default product compilation while preserving both test modes.

M1 now applies this exact gate. Static source accounting therefore excludes 14 fixture Rust modules
and four `include_str!` payloads totaling 8,985 bytes from the default configuration. This is not a
claim about final object-code savings until the managed default/feature artifact matrix is measured.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Test/TextLayoutTestCommon.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Test/TextLayoutTest_LazyGeneration.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Slate.Build.cs`

Unreal keeps shared Slate test data/helpers under `Private/Test` and encloses their definitions with
`#if WITH_DEV_AUTOMATION_TESTS` (`TextLayoutTestCommon.h:11`). The test implementation is guarded by
the same build flag (`TextLayoutTest_LazyGeneration.cpp:6`). Runtime Slate content distinguishes
normal staged assets from `SlateDebug` debug-only content in `Slate.Build.cs`.

The transferable invariant is that test fixtures and debug assets have an explicit build target
boundary; their absence from product execution is not enough if they remain compiled/staged by
default.

## Target architecture

1. Gate `workbench::fixture` with `cfg(test)` or the existing `integration-contracts` feature.
2. Keep external integration-contract tests behind the manifest's required feature. Do not expose
   fixture types from `zircon_editor` crate root or the general `ui` root.
3. If more crates need shared fixtures later, create a dedicated test-support package or feature;
   do not restore an unconditional production module.
4. Keep JSON as test data. Product default layout/descriptors must come from their real runtime
   asset/config owners, never from this preview fixture.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| default-feature compiled fixture modules | 0 |
| default-feature embedded fixture JSON bytes | 0 |
| `cfg(test)` unit fixture availability | preserved |
| `integration-contracts` fixture availability | preserved |
| external non-test fixture consumers | 0 |
| default release library/binary size | no regression; record delta |

Run managed Windows validation with targets on D/E/F: default `zircon_editor` lib check, unit fixture
tests, and the `integration_contracts` target with its required feature. Record compile wall time,
incremental/fresh artifact sizes and final binary/rlib size on one source fingerprint. WPR, power and
RenderDoc are not relevant to this cold test-support boundary.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Record current default/integration feature module and artifact baseline. | source and artifact inventory |
| M1 | Add the explicit module feature gate and static boundary contract. | default off, test modes on |
| M2 | Run managed default/unit/integration Cargo matrix. | all targeted tests pass |
| M3 | Record artifact size and compile-time delta; remove any stale public docs. | quantified current-source evidence |

## Validation state

- Full folder source review: passed, 14/14 files.
- Product call-site classification and Unreal reference guard: passed.
- M1 source implementation: complete. The RED-to-GREEN feature-boundary contract is 2/2; targeted
  `rustfmt` and source accounting pass.
- Managed Cargo and artifact-size measurement: pending while shared Cargo lanes are active.

The folder remains in `pending.md` until M0-M3 pass. Static exclusion alone is not a complete package
or compile-time performance acceptance.
