---
title: Runtime214 compile timing scope
category: zircon_runtime
report_id: Runtime214-compile-timing-scope-2026-09-29
date: 2026-09-29
implementation_status: source_candidate_pending_managed_validation
validation_status: scratch_rustfmt_passed_managed_release_pending
performance_status: product_gate_open
record_target: docs/plans/optimize/zircon_runtime/214/2026-09-29-compile-timing-scope.md
plan_source: docs/plans/optimize/zircon_runtime/214-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-working-tree-review.md
candidate_id: runtime214-compile-timing
---

# Runtime214: isolate compile timing from graph fixture construction

## Change

The applied source candidate in `zircon_runtime/src/render_graph/tests/scaling.rs` constructs each `RenderGraphBuilder` before starting the measured interval. The measured call is now only `builder.compile()`. Warmups use the same graph construction and compile path, with fixture construction outside the timing interval.

The harness retains its four graph shapes (chain, fanout, multi-writer, plugin-labeled chain), 16/64/256/1024 passes, 4/8/16 texture widths, three warmups, 31 measured samples, nearest-rank p50/p95/p99, compiled-stat checks, and the at-least-two-distinct-texture-bucket assertion. The chain writes one transient texture per pass and reads the prior texture; plugin labels cycle over eight names. Fanout has one seed texture read by each consumer. Multi-writer round-robins its writes over three textures and reads all three in the final pass. Widths rotate 4/8/16. With 31 samples, p99 is the maximum. This corrects the compiler timing scope for the synthetic scale lane; it does not add a latency threshold.

## Evidence and pending validation

- Source SHA-256 before candidate: `d480121d836131529f3f0026e258590e7b6f52dc2c46118de1fca014512062f2`.
- Offline candidate SHA-256: `e00f6d984644fcb6b59cf1fc842b468590bfffe652ed750326c0f1656b79acf2`.
- Scratch `rustfmt --edition 2021 --check` passed. The root reports `git apply --check` passed. Managed Release execution is pending; no Cargo run is claimed.
- Applied source target: `zircon_runtime/src/render_graph/tests/scaling.rs`; SHA-256 `2675c54beefe2e7026d85e4f3120d90d53e3cf4ede35b3199a1a671be355e723`. The LF-normalized text matches the scratch candidate; managed execution remains pending.
- Managed Release request arguments: `-Package zircon_runtime -CargoProfile release -LibTests -TestFilter render_graph_compile_scale_reports_p50_p95_and_p99 -IgnoredTests -NoCapture`. This run records synthetic compiler work only and has no latency threshold.
- The separate five-second `MAX_COMPILE_PROJECTION_LATENCY` assertion belongs to the ignored Runtime89 10,000-pass manual-dependency projection test, not this four-topology scale lane or Runtime214 G14.

## Runtime214 G14 boundary

G14 remains open. It needs same-source, same-scene product p50/p95/p99 render/graph-compile timing and allocator/RSS comparison, plus the selected product scene ID and deterministic camera workload, device/adapter/backend/driver profile, quality/render-scale/AA settings, and an explicit numeric ceiling. Render01 calls for three steady captures per scene, each aligned to same-frame PNG, RenderDoc RDC, and graph/profile sidecars, and records RenderGraph compile/cache, GPU frame, CPU render/RHI, RSS/VRAM p50/p95. Render17 specifies a fixed 1080p scene, separate cold/warm conditions, 60 warm-up frames, and at least 300 measured frames per sample. These references do not identify the G14 product scene/device or set its numeric ceiling. Their profile passages name p50/median and p95 but do not define the requested product p99 calculation/reporting. The synthetic compile scale lane cannot substitute for that product comparison.

Profile/acceptance references: the Runtime214 master review's G14 row is at `docs/plans/optimize/zircon_runtime/214-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-working-tree-review.md:319`; its global Render01 capture contract is at `docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md:597-600`; the Render17 frame sampling method is at `docs/plans/zircon_runtime/render/17/2026-08-11-render17-profiling-readiness-and-optimization-research.md:131-136`.
