---
related_code:
  - zircon_editor/src/core/export/mod.rs
  - zircon_editor/src/core/export/pipeline.rs
  - zircon_editor/src/core/export/preset.rs
  - zircon_editor/src/core/export/stages/compile_host.rs
  - zircon_editor/src/core/export/stages/executor.rs
  - zircon_editor/src/core/export/stages/platform_bundle.rs
  - zircon_editor/src/core/export/tests.rs
  - zircon_runtime_interface/src/export/mod.rs
  - zircon_runtime_interface/src/export/artifact.rs
  - zircon_runtime_interface/src/export/report.rs
  - zircon_runtime_interface/src/export/stage.rs
  - zircon_runtime_interface/src/export/preset.rs
implementation_files:
  - zircon_editor/src/core/export/mod.rs
  - zircon_editor/src/core/export/pipeline.rs
  - zircon_editor/src/core/export/preset.rs
  - zircon_editor/src/core/export/stages/compile_host.rs
  - zircon_editor/src/core/export/stages/executor.rs
  - zircon_editor/src/core/export/stages/platform_bundle.rs
  - zircon_runtime_interface/src/export/mod.rs
  - zircon_runtime_interface/src/export/artifact.rs
  - zircon_runtime_interface/src/export/report.rs
  - zircon_runtime_interface/src/export/stage.rs
  - zircon_runtime_interface/src/export/preset.rs
plan_sources:
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_editor/src/core/export/tests.rs
  - zircon_runtime_interface/src/export/tests.rs
doc_type: module-detail
---

# Editor Core Export Pipeline

## Purpose

`zircon_editor::core::export` is the headless orchestration and preset-storage owner for project export. It separates
pipeline execution from the export wizard UI and consumes neutral contracts from
`zircon_runtime_interface::export`, so editor, runtime tools, and plugin packages use the same stage
identity and report shape.

The module implements Editor Plan 15 M1.1 and M1.2. It owns execution rules, versioned `.zpreset`
storage, the `zircon_build.py` CompileHost adapter, and staged bundle validation.

## Shared Contract

`zircon_runtime_interface::export` owns the only public eight-stage enum:
`Validate`, `SourceTemplate`, `NativeDynamic`, `CompileHost`, `CookAssets`, `Pack`,
`PlatformBundle`, and `Report`. `ExportStage::ALL` is the canonical order. `cli_id`,
`report_name`, and `FromStr` keep process banners and serialized reports aligned without editor or
runtime forwarding helpers.

The shared DTO family is deliberately neutral:

- `ExportDigest` is a typed 256-bit value. Hash algorithm selection stays in the executor layer.
- `ExportArtifactRef` identifies an artifact by semantic key, locator, and optional content digest.
- `ExportStageIo` records explicit stage inputs, outputs, and the computed stage fingerprint.
- `ExportStageRecord` and `ExportPipelineReport` record passed, skipped, and failed outcomes.

The former runtime-owned `ExportPipelineStage` and editor stage-name wrappers were deleted. No
alias, compatibility re-export, or fallback stage table remains.

## Pipeline Construction

`ExportPipelinePlan::new` accepts stage nodes and validates the graph before execution. It rejects
duplicate stage declarations, references to undeclared dependencies, and dependency cycles. The
stored node list is topologically ordered, which allows a preset or commandlet to select a valid
subset without duplicating the executor.

`ExportStageExecutor` is the boundary for concrete stage implementations. `prepare` projects
current inputs, expected outputs, and the stage-parameter digest. `execute` performs the stage and
returns actual artifact references plus diagnostics. Errors remain typed through
`ExportPipelineRunError<E>` and are exposed as the Rust error source.

## Fingerprint and Resume Semantics

The editor pipeline computes a BLAKE3 fingerprint over the stage id, stage-parameter digest, every
ordered input artifact, and every expected-output key/locator/digest marker. Output identity is
therefore bound to the destination root; a report from another output directory cannot be reused.
Length prefixes are included to prevent concatenation ambiguity.

When a resume report contains a Passed or Skipped record with the same stage fingerprint, the
executor's `can_reuse` hook must also validate the previous outputs. The concrete staged-build
executor recomputes deterministic file/tree digests and validates bundle layout before it permits a
skip. The new report still receives an explicit Skipped record and retains the previous outputs,
allowing downstream preparation to consume the exact artifacts that were reused.
If an upstream output digest or a stage parameter changes, that stage is executed and downstream
fingerprints naturally change when they consume the new artifact.

Both preparation and execution failures append a Failed record before returning. Execution
failures retain expected outputs for diagnostics; preparation failures use empty I/O and the zero
digest because no trustworthy fingerprint exists. The returned error also contains the partial
report, so the next run can reuse all earlier passed stages and restart at the failed stage.

## Constraints

- Pipeline records do not prove that a locator still exists. Filesystem executors must implement
  `can_reuse` and compare current content with the persisted output digest.
- Artifact order participates in the fingerprint and must be deterministic.
- The core pipeline does not spawn processes, read presets, mutate UI state, or infer dependencies.
- Tests are authored with the slice and are executed at Plan 15 M1's declared testing stage.

## Preset and Staged-Build Integration

`ExportPreset` is a neutral versioned DTO with schema id `zircon.export-preset`. It references an
existing profile and owns target mode, debug selection, include/exclude filters, entry/keep assets,
plugin subset, cook options, and per-path file modes. `ExportPresetStore` only accepts portable
preset names and persists `<project>/export/<name>.zpreset` through same-directory staging and
fsync followed by an atomic replacement (`ReplaceFileW` on Windows, overwrite rename elsewhere).
The preset-specific decoder requires the `$zircon` envelope even for schema version zero; generic
unwrapped version-zero compatibility is intentionally unavailable to `.zpreset`.

The retained export wizard loads the independently selected preset from the active project before
it resolves the referenced profile. It rejects target-mode disagreement between the two contracts.
No production panel fallback fabricates a preset. Python validates the complete Rust wire shape,
rejects unknown/wrongly typed fields, and projects the normalized preset fields on the stage
namespace. Non-CompileHost Python stages carry `--preset`; CompileHost is hard-cut to the core
executor.

`CompileHostStage` converts preset target/debug intent into one non-interactive
`tools/zircon_build.py` command. Client exports request `hub,editor,runtime`; server exports request
`runtime`. `PlatformBundleLayout::validate` then enforces the staged `ZirconEngine/` contract:
merged `assets/`, runtime library beside the binaries, Hub as client launcher, and Editor present as
its child executable. Missing artifacts are typed errors; no fallback layout is synthesized.

The wizard's visible command rows are a projection of `ExportPipelinePlan`; the job loop consumes
that plan's topological order. Production CompileHost invokes `ZirconBuildStageExecutor`, while
PlatformBundle runs the core layout validator before its packaging process. This removes the former
second stage-order owner and the test-only executor island.

## Test Coverage

`zircon_runtime_interface/src/export/tests.rs` covers stage-name round trips and typed report DTO
retention. `zircon_editor/src/core/export/tests.rs` covers missing dependencies, cycles, identical
fingerprint skipping, upstream invalidation, failed-stage reporting, and resume from the failure.
Tests additionally cover output-root changes, staged-output deletion/tampering, strict preset
envelopes, and malformed Python payload fields. Per milestone policy, final Cargo evidence is
recorded only when the whole M1 testing stage runs.

## M1.2 Production Hard Cutover (2026-07-12)

- `<project>/export/*.zpreset` is the wizard export identity. A preset resolves `profile_ref` only
  against `ProjectManifest.export_profiles`; built-in profile templates are not a second runtime
  authority and are not injected during projection.
- ClientRuntime fingerprints editor, Hub, installed Tauri CLI, and Node identity because it builds
  `hub,editor,runtime`. ServerRuntime fingerprints runtime/app/shared inputs and Rust/Python/Cargo
  tooling only; a server build machine does not need Hub files or Node.
- `wizard/execution.rs` owns process streaming and orchestration;
  `wizard/execution/core_pipeline.rs` owns core CompileHost/PlatformBundle adapters, resume-report
  loading, and atomic report replacement.
- CompileHost final reports use staged-build fields and validate `zircon_build.py` options
  `--targets`, `--out`, `--mode`, `--runtime-features`, and `--cargo`. Cargo-direct report options
  (`-p`, `--bin`, `--target-dir`, `--features`, `--release`) are legacy and rejected.
- `ExportPreset.debug` is the sole staged-build mode authority. The final aggregator validates that
  `--mode` is `debug` or `release`, but does not let the older profile Cargo plan override the
  preset-selected mode.
