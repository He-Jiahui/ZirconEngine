# Rich cache owner-qualified reset telemetry

Date: 2026-08-30  
Scope: `RRT-P1-022` parser/provider-qualified interval telemetry foundation  
Status: `RRT-P1-022_parser_provider_qualified_reset_snapshot_static_complete / project_surface_correlation_and_managed_profile_pending`

## Current-source finding

The compiled rich cache is not process-global in current source. Ownership is already a single chain:

`SharedTextLayoutSession -> RichTextParser -> CompiledRichTextCacheOwner -> Mutex<CompiledRichTextCache>`.

The reporting algorithm was stale. `CompiledRichTextCacheReport` held lifetime saturating counters and
`UiTextMeasureCache` retained `CompiledRichTextCacheFrameSampler` to subtract two copies. A saturated
counter never advanced again, so the external delta could not distinguish an idle interval from telemetry
exhaustion. The report also omitted the parser identity and decorator/emoji generations that produced it.

## Reference and decision

Local Unreal `FShapedTextCache` is explicitly created as an instance, owns its key-to-artifact map, exposes
an explicit `Clear`, and retains its font-cache context. `FSlateFontCache` similarly records work at the
cache operation that owns the work. Relevant local sources are:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ShapedTextCache.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/ShapedTextCache.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCache.cpp`

Zircon keeps that instance-lifecycle principle but does not copy Unreal's global stats macros. Its Surface
capture needs bounded interval values, so the cache mutation owner takes and resets event counters under
the existing mutex. The outer Surface/profile session remains the project/surface correlation owner.

## Implementation

- `CompiledRichTextCache::take_report()` copies the report and resets hit, miss, parse, eviction,
  admission-bypass, and candidate-probe counters under the cache mutex.
- Residency entries/bytes and configured maximums are gauges and remain intact across a take.
- All six events use checked addition. Overflow clamps only that interval and emits
  `telemetry_saturated`; the next take starts a clean interval.
- `RichTextParser` stamps parser identity, decorator generation, and emoji generation onto both diagnostic
  reads and destructive interval snapshots.
- `SharedTextLayoutSession` and `UiTextMeasureCache` forward the owner snapshot. The external frame sampler
  type, state, and cumulative subtraction are removed.
- Surface profile emission now contains 16 fixed names after RRT-P1-014 added four contention measurements.
  No markup, pointer, resource id, project string, parser string, or provider string is emitted.

The snapshot operation is `O(1)` time and memory. Cache lookup/insert asymptotics are unchanged. This fixes
telemetry semantics; it is not evidence that parse/layout/render performance improved.

## Tests and evidence boundary

- The failing-first static contract required parser identity, provider generations, saturation receipt,
  owner `take_report`, and removal of the UI sampler.
- A Rust unit test verifies that take/reset clears interval events while preserving residency gauges and
  that checked overflow is observable.
- The profile regression projects all 16 fixed values, including identity/generations, contention, and saturation.
- Current infrastructure static contract suite passes 36/36 in the final 0.206 s rerun.
- Focused Rust files pass `rustfmt --edition 2024 --check`; scoped `git diff --check` passes.
- Old sampler type/field symbols scan to zero. The principal owners are currently 541 production lines for
  `rich_cache.rs`, 340 folder-backed test lines, 720 for `measure_cache.rs`, and 739 for `profile.rs`, below
  the 800-line production review warning.

Managed Cargo/rustc remains unavailable through the managed acquisition path. Project/surface explicit
correlation, matched workload profile, RSS, power, WGPU/framebuffer/PNG, commit, and WeCom remain open.
No screenshot is produced because this is a nonvisual telemetry contract; a source or strategy screenshot
would violate the Text07 evidence policy.
