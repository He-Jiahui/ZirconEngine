# Static Mesh Command Cache Visibility-Lifetime Research

Date: 2026-08-26

Status: source repair implemented; managed validation and product evidence remain pending.

Plan owner: `docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md` (MD-M2).

## Required Contract

Plan 02 defines static mesh draw commands as cross-frame cache entries and states that visibility
changes filter commands rather than trigger rebuilds. The cache lifetime must therefore be owned by
the source instance and its static state, not by a transient main or shadow view decision.

The current Render-02 parallel-preparation research also requires one deterministic owner for cache
transactions. This investigation does not move cache ownership to worker threads, does not add a
second cache, and does not change the direct Rayon migration boundary.

## Reviewed Product Path

The following current-source sequence was reviewed:

1. `build_mesh_draws/build/build.rs` sends every pending draw through
   `extract_pending_static_mesh_command_cache_hits` before residual `MeshDraw` materialization.
2. `pending_command_cache_extract/extract_item.rs` computes cacheable phases by combining static
   eligibility with `main_view_visible`, `shadow_view_visible`, and per-phase relevance.
3. When every candidate phase is visibility-pruned,
   `commands_for_extract_item_with_stats_and_context` returns an empty successful extraction. The
   pending draw is skipped and no cache lookup is performed.
4. `scene_renderer_core_render_compiled_scene/render/render.rs` calls
   `CachedMeshDrawCommands::retain_generation` after command construction. An entry survives only
   when `last_touched_generation` equals the current frame generation.

Consequently, a still-live static instance can lose all valid cached commands after one hidden
frame. On reappearance it must rebuild them, even when transform, geometry, material, pass mask,
and GPUScene registration are unchanged.

## Root Cause and Boundary

`last_touched_generation` currently means both "the source instance remains resident" and "a view
selected this command this frame". Those are different facts. The first is a cache-lifetime fact;
the second belongs to visible command selection.

This change must remain within the cache owner boundary:

- `CachedMeshDrawCommands` owns state comparison and generation touch.
- Pending-cache extraction owns the view filter and identifies phases omitted only because of that
  filter.
- Workers and the WGPU/GPUScene owners do not gain mutable cache access.
- Cached command data remains unchanged. This is not the larger cache-ABI redesign required before
  cached commands can safely outlive a recycled GPUScene span.

The last restriction matters: a pending draw proves the instance entered the current frame's
GPUScene synchronization path, so retaining a matching cache entry for that frame does not extend
it past an absent source instance. An absent instance produces no pending draw and receives no
touch, so frame-end retirement still removes its entries.

## Unreal Reference Check

Unreal 5.5 keeps this separation explicit. `FVisibleMeshDrawCommand` in
`Renderer/Public/MeshPassProcessor.h` stores a pointer to a persistent `FMeshDrawCommand` together
with the per-view sort key, culling payload, and view overrides; its source comment says it carries
only InitViews visibility/sorting data and not draw-submission data. The same header defines
`FCachedMeshDrawCommandInfo` as scene-cached metadata separate from the command itself, while
`PrimitiveSceneInfo.cpp` retires cached command state through
`FPrimitiveSceneInfo::RemoveCachedMeshDrawCommands` when the primitive is actually removed.

Zircon does not copy Unreal's RHI objects or allocator, but it adopts the same ownership rule:
visible-command selection is frame/view-local, while command-cache retirement follows source
lifetime and static-state invalidation.

## Proposed Minimal Repair

Add a cache-owner operation with the following semantics:

```text
touch_if_state_matches(key, state, generation)
    find the existing entry without cloning its MeshDrawCommand
    if its RenderMeshStaticState equals state:
        set last_touched_generation = generation
    otherwise:
        leave it untouched for retain_generation to retire
```

In pending-cache extraction, enumerate only the static cache phases that are valid independent of
visibility (enabled depth prepass, enabled shadow for a shadow caster, and enabled opaque or
alpha-mask base phase). For each such phase that was omitted by the visibility/relevance filter,
call `touch_if_state_matches`. Phases actually selected by a view continue through the existing
`lookup_status` path and are touched there.

This retains these required properties:

- No `MeshDrawCommand` clone, rebuild, variant allocation, WGPU allocation, or cache-hit statistic
  is introduced for a visibility-pruned phase.
- A changed transform, geometry, or material revision does not refresh an old entry.
- A changed disabled-pass mask creates a different key; the old key is not refreshed and retires.
- A deleted or unloaded source instance receives no touch and retires at frame end.
- The existing serial/parallel command-order and cache-stat parity contract is unaffected because
  this path runs before residual command preparation and does not publish commands.

The repair adds at most one expected O(1) hash lookup and static-state comparison per visibility-
pruned cacheable phase. It does not add allocations or command clones beyond the existing
extraction path. Whether this reduces a material frame cost must be measured; it is first a
correctness and cache-lifetime repair.

## Regression Coverage

The implementation test is a three-frame extraction sequence using one static opaque shadow caster
with cached prepass, shadow, and opaque commands:

1. Generation 1 stores all three commands.
2. Generation 2 has no main or shadow visibility. Extraction emits no commands and requests no
   rebuild; after `retain_generation(2)`, all three entries remain.
3. Generation 3 restores visibility. Extraction reports three cache hits and zero rebuilds.

A companion state-change case must prove that a hidden item with a changed static revision is not
refreshed and is removed by `retain_generation`. This keeps invalidation authoritative rather than
turning visibility retention into an unbounded residency rule.

## Measurement and Acceptance Discipline

No CPU-time, GPU-time, power, or algorithmic-optimality claim is made by this review. The existing
Render-02 measurement protocol remains mandatory before an optimization claim: typed
mesh-preparation observations, matched static/material-diverse/skinned controls, 30 warm-up plus
120 settled frames, GPU timing, and matched RenderDoc/PNG artifacts under
`docs/tests/runtime/render/`.

Current evidence is limited to source tracing and static checks. Managed Cargo validation is
temporarily unavailable because the coordinator reports unmanaged shared target artifacts, and
Render-17 owns the visual-artifact directory. No screenshot, RDC capture, performance number, or
power number has been manufactured for this item.

## Implementation Status

Completed:

- Verified the issue against the product extraction and renderer retirement path.
- Reconciled the repair with Plan 02's visibility and cache-lifetime contract.
- Reconciled ownership with the Render-02 deterministic parallel-preparation research and Unreal's
  separation of persistent command storage from per-view visible command lists.
- Defined the minimal owner API, invalidation rules, regression cases, and performance gate.
- Implemented `CachedMeshDrawCommands::touch_if_state_matches` and routed every view-pruned
  cacheable phase through it before generation retention.
- Added focused extraction coverage for fully hidden entries, shadow-only visibility with
  main-view-pruned siblings, and a hidden static-revision change that must still retire old
  entries.

Pending:

- Run the managed current-source validation for the cache/extraction owner scope.
- Produce measured profile, RenderDoc, and PNG evidence only through the artifact-owning validation
  lane.
