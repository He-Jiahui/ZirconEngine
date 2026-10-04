---
doc_type: feature-completion
status: source_candidate_validation_pending
validation_status: scratch_rustfmt_passed_managed_release_pending
performance_status: product_gate_open
record_target: docs/plans/astra/features/runtime/1066-runtime214-compile-timing-scope-completion-list.md
plan_sources:
  - docs/plans/optimize/zircon_runtime/214-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/214/2026-09-29-compile-timing-scope.md
implementation_files:
  - zircon_runtime/src/render_graph/tests/scaling.rs
tests:
  - zircon_runtime/src/render_graph/tests/scaling.rs::render_graph_compile_scale_reports_p50_p95_and_p99
candidate_id: runtime214-compile-timing
apply_status: applied
---

# Runtime1066 / Runtime214 compile timing scope completion list

| Plan item | Candidate result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| Measure compile separately from fixture setup | The ignored Release scale test constructs its graph builder before the timer and times only `compile()`. The chain, shared-source fanout, three-texture multi-writer, and plugin-labeled chain fixtures preserve the 16/64/256/1024 passes, rotating 4/8/16 widths, three warmups, 31 samples, nearest-rank p50/p95/p99, and compile-stat assertions. | The patch is applied to shared source; scratch rustfmt and patch applicability checks passed. Managed Release has not run. | source_candidate_validation_pending |
| Managed Release scale lane | Record the four-topology 16/64/256/1024 compiler p50/p95/p99 and compiled work counters. | Managed request args: `-Package zircon_runtime -CargoProfile release -LibTests -TestFilter render_graph_compile_scale_reports_p50_p95_and_p99 -IgnoredTests -NoCapture`. Execution has not occurred; the scale lane has no latency threshold and remains supplemental synthetic compiler evidence. The separate 5 s Runtime89 projection assertion is not this G14 gate. | pending |
| Runtime214 G14 product performance | Same-source, same-scene product render/graph-compile p50/p95/p99, allocator/RSS, selected device and quality profile, and a numeric ceiling. | Product scene ID/deterministic camera workload, adapter/device/backend/driver, quality/render-scale/AA settings, numeric ceiling, and product p99 reporting definition remain unspecified; no product comparison has run. Render01 requires three steady captures per scene with same-frame PNG/RDC/graph-profile sidecars and p50/p95 system metrics. Render17 specifies fixed 1080p cold/warm samples with 60 warm-up and at least 300 measured frames. Those references specify p50/median and p95, not p99. See Runtime214 master G14 row at `docs/plans/optimize/zircon_runtime/214-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-working-tree-review.md:319`, Render01 at `docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md:597-600`, and Render17 at `docs/plans/zircon_runtime/render/17/2026-08-11-render17-profiling-readiness-and-optimization-research.md:131-136`. | open |

This completion list does not accept Runtime214 G14 or claim product performance.
