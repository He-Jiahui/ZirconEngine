---
title: Plugins04 Sanitized Reflection Capture Template
category: zircon_plugins
report_id: Plugins04-sanitized-reflection-capture-template-2026-08-26
date: 2026-08-26
session_id: root-plugins04-sanitized-reflection-capture-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Plugins04 sanitized reflection capture template

## Scope

- Parent scope: Plugins04 Rendering reflection-probe capture and its memory/performance qualification work.
- Baseline: `d4ca9a802ecd19976c653caa58614af0c2fb15f7`, epoch `449`.
- Owned paths: `capture/execute.rs`, its focused source contract, the standalone clone model, and this record.
- This is a bounded CPU/allocation fix for the existing manual six-face capture helper. It does not claim to connect capture to an Editor operation, scheduler, cancellable job, atomic publication workflow, or visual qualification gate.

## Change

Six-face capture now constructs a sanitized scene template by cloning only render-scene geometry, environment, and preview fields. Editor overlays and virtual-geometry debug state are excluded before any per-face clone. The first five faces clone the sanitized template and the sixth consumes it, so render-relevant scene cloning remains exactly six copies while full overlay cloning falls from six copies to zero.

Camera selection, HDR rendering, CMFT face transforms, output order, source hash, cubemap build, bake request, and staged persistence remain unchanged. A direct Rust test verifies the template preserves scene/environment/preview content while replacing a non-default overlay with `RenderOverlayExtract::default()` and removing virtual-geometry debug state.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_plugins04_sanitized_reflection_capture_performance_contract -v` produced 4/4 expected failures against the per-face full-scene clone.
- GREEN: the focused source contract now passes 4/4.
- `rustfmt +1.94.1 --edition 2021 --check --config skip_children=true` passes for `execute.rs`.
- Scoped `git diff --check` passes.
- The standalone model compiles with `rustc 1.94.1 -O`; it does not use Cargo or a shared build target.

The deterministic model measures 31 alternating legacy/sanitized sample pairs for a six-face capture with 32,768 geometry items, 2,048 light/environment items, and 1,024 nested overlay nodes containing 32 elements per child array. Both algorithms preserve six render-relevant scene copies and produced checksum `2147483647` in all four runs.

| Metric | Full scene cloned per face | Sanitized template | Change |
|---|---:|---:|---:|
| P50 | 18.8335 ms | 1.2500 ms | -93.3629% |
| P95 | 26.4797 ms | 2.4718 ms | -90.6653% |
| allocations / capture | 30,756 | 18 | -99.941475% |
| full overlay clones | 6 | 0 | -100% |

The other three runs produced P50 reductions of 93.0190%, 93.0912%, and 92.8018%, and P95 reductions of 93.1868%, 76.8267%, and 92.7109%, with identical allocation and overlay-clone counts. These values cover CPU scene preparation before rendering only; they do not claim GPU render-time or end-to-end bake-time improvement.

## Async validation

The coordinator must run the four focused source contracts, the exact Rust test `capture::execute::tests::capture_scene_template_excludes_editor_only_overlays_and_debug`, Rust formatting, scoped diff checks, checksum parity, and the standalone model. Acceptance requires 4/4 source contracts, exactly one filtered Rust test, checksum `2147483647`, P50 reduction of at least 85%, P95 reduction of at least 65%, allocation reduction of at least 99.9%, and zero optimized full-overlay clones.

The ticket joins the outstanding optimization validation batch and this business Session does not wait for it before subsequent work. Cargo remains coordinator-owned. Foreign unmanaged build directories can stop managed copies at `artifact_governance`, and the foreign tracked deletion of `zircon_runtime/src/core/framework/render/environment/skybox.rs` can stop Cargo input-closure planning. Neither condition belongs to this candidate.

Integration and automatic WeCom publication remain coordinator-owned after managed validation and independent review succeed. The WeCom message must include the managed P50/P95, allocation, and overlay-clone reductions and label them as CPU reflection-probe capture scene-preparation evidence for the six-face/1,024-overlay-node workload.
