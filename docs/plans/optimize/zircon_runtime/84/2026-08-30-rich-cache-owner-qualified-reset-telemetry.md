# RRT-P1-022 rich cache owner-qualified reset telemetry

Status: `RRT-P1-022_parser_provider_qualified_reset_snapshot_static_complete / project_surface_correlation_and_managed_profile_pending`

Current source already scopes the compiled rich cache to the `SharedTextLayoutSession` parser owner. The
defect was the external cumulative-delta sampler: lifetime saturating counters could freeze at `u64::MAX`,
and samples carried no parser/provider generation.

The cache now takes and resets six interval events while holding its own mutex, preserves residency gauges,
and reports checked-overflow saturation. The parser stamps parser identity plus decorator/emoji generations
onto the same snapshot. UI profiling emits 12 fixed low-cardinality names and no source content or dynamic
tenant label. Snapshot cost is `O(1)` and cache lookup/insert asymptotics do not change.

This is a telemetry correctness foundation, not measured optimization evidence. The current infrastructure
static suite passes 35/35 in the final 0.315 s rerun; rustfmt and scoped diff-check pass, and old sampler
symbols scan to zero. Managed Cargo, explicit project/surface correlation, matched profile, RSS, power,
WGPU/PNG, commit, and WeCom remain open. See
[`../../../zircon_runtime/text/07/2026-08-30-rich-cache-owner-qualified-reset-telemetry.md`](../../../zircon_runtime/text/07/2026-08-30-rich-cache-owner-qualified-reset-telemetry.md).
