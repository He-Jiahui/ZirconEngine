---
title: Editor prepared render-list Runtime authority convergence
date: 2026-08-28
module: zircon_editor retained-host presentation and zircon_runtime UI surface render cache
status: m3b_source_owned_prepared_segment_design_ready_dynamic_pending
---

# Goal

Remove repeated Editor host command reconstruction without creating an Editor-owned command payload
cache beside Runtime UI. Runtime `UiSurface::render_extract` and `UiSurfaceRenderCache` remain the
single command/range authority. The retained-host bridge may retain generation cursors, range
references and backend products, but must not retain a second independently invalidated command list.

## Current-source findings

1. Runtime already owns per-node command identity. `UiSurfaceRenderCache` indexes
   `command_ranges: BTreeMap<UiNodeId, (usize, usize)>`, supports changed-node extraction and exposes
   `commands_for_node`. This is the lower authority to extend, not a cache to reproduce in Editor.
2. Editor `present_redraw` now reads one coherent `HostPresentationGeneration` and passes both
   `generation.structure()` and its six-domain cursor
   `{structure, interaction, viewport, hit_test, theme, diagnostics}` through the GPU and softbuffer
   presenter boundaries. Thread-local paint overrides still exist for legacy painters, but presenter
   generation identity is no longer inferred from them.
3. An ordinary GPU damage present still performs:

   `record Vec -> chrome extraction Vec -> icon atlas scans -> image compaction -> Runtime command Vec`

   Runtime WGPU retains final pixels and can retain compiled batch/vertex products only when the
   producer supplies a stable generation. Ordinary damage streams are intentionally unversioned
   because each contains only the damaged subset, not a complete projection.
4. SVG/image source identity is already versioned by `(resource_key, resource_generation)`. The icon
   atlas retains immutable pages and the Runtime presenter queries GPU residency before staging source
   bytes. Repeated command classification is real CPU work, but it is not evidence that SVG bytes are
   decoded or uploaded again. Product counters must distinguish source decode/raster, atlas lookup,
   CPU staging, GPU upload and resident resolve.
5. The first image-resource compaction is required. Before this change, conversion of the already
   compacted owned stream called the same compaction entry again. Its `any` probe scanned every damage
   command only to discover that no uncompacted payload remained.
6. Runtime projection caches retained `UiNodeId` row signatures and command ranges, but
   `ViewTemplateNodeData` dropped that identity and the legacy template bridge explicitly emitted
   `surface_node_id: None`. Published `UiRenderFrameCommands` also retained immutable command segments
   without an addressable node-range index. The resulting host paint stream was anonymous.
7. `PaneData.body_surface_frame` is not the source render frame. It is reconstructed from projected
   template rows for Host hit testing. The authoritative source `UiSurface` remains inside the view
   projection cache, so using `body_surface_frame` as a render-cache qualifier would bind commands to
   the wrong tree and generation.
8. The projection cache is also the correct publication boundary for that source frame. It runs after
   dirty property/layout work and already owns the original `UiSurface`; publishing there keeps lazy
   `surface_frame()` work out of Host paint and input paths. An `Arc` can then qualify command
   references without copying hit grids, command segments or tree metadata per row.

## Reference-engine invariant

Local Unreal source was read at:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/FastUpdate/WidgetProxy.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElements.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/DrawElements.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/ElementBatcher.cpp`

Slate retains cached element data at the invalidation root, repaints dirty widget proxies by stable
widget index and patches cached geometry for render-transform changes. It does not build an unrelated
window-level payload cache for each consumer. The transferable rule is one source-owned cached element
authority plus generation-qualified backend products.

## Authority decision

- Runtime `UiSurface` owns ordered command payloads and per-node ranges.
- `HostPresentationGenerationCursor` must become an explicit presenter input before any ordinary
  presenter cache is keyed from it.
- Editor-native panes still outside Runtime UI may publish source-owned range descriptors. The bridge
  maps those descriptors into Runtime command/range identity; it does not copy them into a second
  long-lived payload arena.
- Stable text/image payloads remain immutable shared resources. Geometry, clip, opacity, selection and
  hover are compact patch domains.
- A changing damage subset must not reuse a full-projection producer generation. A generation advances
  whenever its command payload/order changes; partial compiled reuse needs range/segment generations,
  not a falsely stable draw-list generation.
- Full rebuild, missing range identity, reorder and backend recovery are typed fallbacks with counters.

## M3b implementation order

| Step | Work | Gate |
| --- | --- | --- |
| M3b.1 | Remove provably redundant stream work without changing authority. | no second compaction command probe |
| M3b.2 | Pass the complete presentation cursor explicitly through GPU/softbuffer presenter inputs. | no cache key inferred from TLS or structure pointer |
| M3b.3 | Publish source-owned native-pane range/role descriptors and map them to Runtime range identity. | dirty interaction rebuilds only addressed roles |
| M3b.4 | Hard-cut transient duplicate payload routes after GPU and softbuffer parity. | one command payload authority |
| M3b.5 | Run managed product CPU/allocation/RSS/p50/p95/p99, SVG decode/raster/upload/residency and pixel parity. | current-source dynamic acceptance |

## M3b.1 static candidate

`ChromeCommandStream` now publishes explicit `image_resources_compacted` state. Constructors start
dirty, successful compaction publishes clean, and every command append reopens the state. A repeated
compaction returns before `self.commands.iter().any`, so the owned GPU conversion no longer performs a
second O(D) probe over the damage stream.

The deterministic pressure model
`tools/editor_chrome_command_stream_allocation_pressure.py` uses 4,096 presents and 32,768 commands
per present. It reports 134,217,728 removed redundant command visits. This is an operation count only;
it excludes first compaction, recording/extraction, atlas scans, Runtime conversion/compaction,
batching, CPU, allocator, RSS, latency, power and GPU timing.

Artifact: `E:\zircon-profiles\editor-chrome-command-stream-compaction-state-20260828.json`, SHA-256
`6F62471ECDE4DA4C41990B64D1FB603EBC995DC7745CF646853C72FE0F965720`.

Static evidence:

- focused Python contract: 5/5 GREEN;
- lower Rust dirty/clean/append state regression: source present, Cargo pending;
- product dynamic evidence: pending managed validation lane;
- no claim that M3b.3-M3b.5 or retained render-list convergence is complete.

## M3b.2 static candidate

The redraw path now derives `presentation_cursor` from the same `HostPresentationGeneration` that
supplies `generation.structure()`. Both ordinary and native-resize present calls carry that cursor
through `HostChromePresenter` and into GPU/softbuffer backend functions. The existing paint scope stays
in place for painters that still read generation-owned overrides through TLS.

This change deliberately does not assign the cursor as an ordinary damage draw-list generation. A
damage stream is only a subset of the projection, and reusing a full-projection generation for changing
subset payloads would make Runtime compiled caches stale. M3b.3 must first publish stable owner/range
identity and typed dirty roles.

Static evidence after M3b.2:

- cursor boundary source contract: expected 4 RED, now 4/4 GREEN;
- M3b.1 plus adjacent fragment/damage contracts: combined 18/18 GREEN;
- ten affected Rust files: `rustfmt --check` GREEN;
- Rust compile/behavior and product dynamic evidence: pending managed validation lane;
- no claim that M3b.3-M3b.5 or retained render-list convergence is complete.

## M3b.3a range identity boundary static candidate

`UiRenderFrameCommands` now owns an immutable `UiNodeId -> Range<usize>` index beside its persistent
64-command segment directory. A full snapshot builds the index in one average O(C) pass and stores at
most O(U) entries for C commands and U owners. Lookup is average O(1). Only contiguous owner ranges are
published; a non-contiguous owner fails closed instead of returning an invalid slice.

Fixed-cardinality publication patches do not rebuild or globally scan this index. Because the
persistent directory replaces complete touched leaves, node identity is checked across those leaves in
O(S * (D + 64)) for S touched segments and directory depth D: each leaf is located once and compares at
most 64 node ids. Payload-only patches share the index `Arc`; any owner change rejects the local patch
and returns control to the existing full-snapshot fallback. The serialized frame schema remains the
pre-existing flat command list because deserialization rebuilds the same derived index.

Editor `ViewTemplateNodeData` now carries `Option<UiNodeId>`. Both Runtime-backed view materialization
and UI-asset node projection set the real source id, and `TemplatePaneNodeData` conversion forwards it.
Manually authored Editor-only nodes retain `None`; no synthetic Runtime ownership is invented.

Static evidence after M3b.3a:

- focused source contract: expected 4/4 RED, now 4/4 GREEN;
- adjacent M3b allocation/cursor/fragment/damage contracts: combined 22/22 GREEN;
- lower Rust regressions are present for contiguous lookup, shared patch identity, non-contiguous
  rejection, owner-changing patch fallback and borrowed/owned Editor conversion parity; Cargo pending;
- typed native-pane role descriptors, direct Runtime-range consumption, Chrome payload hard cut and
  CPU/RSS/latency/GPU acceptance remain pending. M3b.3 is therefore still in progress.

## M3b.3b exact command identity static candidate

`UiRenderFrameCommandRef` now identifies one command relative to a published frame as
`(UiNodeId, node_command_index)`. Resolution first looks up the node's authoritative range, then uses
checked addition and rejects an index outside that range. The reference cannot address another
owner's adjacent command and is deliberately not process-global; a later cache key must pair it with
the source frame's tree id and render-domain generation.

View materialization assigns node-local indices in one average O(C) pass with O(U) transient counters
for C commands and U owners. Skipped structural/component-owned rows still consume their original
index, so a surviving row retains the exact Runtime command position rather than a filtered-row
ordinal. Incremental topology validation now compares this typed reference in addition to node and
kind. A same-kind reorder therefore fails through the existing topology fallback instead of silently
retargeting a cached product.

The UI Asset bootstrap projection intentionally publishes `None`: one projected row aggregates all
commands owned by a node and cannot truthfully claim one command offset. The borrowed and owned
template bridges preserve a valid optional reference unchanged. This is a fail-closed boundary, not a
claim that Editor already consumes Runtime command payloads directly.

Static evidence after M3b.3b:

- focused source contract: expected 4 new RED with 3 prior GREEN, now 7/7 GREEN;
- lower Rust regressions are present for range-bounded reference resolution, independent node-local
  ordinal assignment and non-empty borrowed/owned conversion parity; Cargo pending;
- no per-row tree-id string and no per-command `Arc` source-frame clone was added;
- source-frame publication from the projection cache, a stream-level surface table, compact `u32`
  command owner keys, native role descriptors, Chrome/RHI segment consumption and product dynamic
  acceptance remain pending. M3b.3 is still in progress.

## M3b.3c source-frame publication static candidate

`CachedProjection` now publishes the authoritative `Arc<UiSurfaceFrame>` from the retained Runtime
surface at the `ProjectionCacheUpdate::Ready` boundary. That point follows any required mutation and
`rebuild_dirty` work. `ViewTemplateNodeProjection` carries an optional clone of the same `Arc`; it does
not call `surface_frame()` and does not rebuild or copy the frame.

The first product path is Assets Activity. It captures the source frame before consuming the row
projection in composition, then forwards it through `AssetsActivityPaneViewData` and
`AssetsActivityPaneData`. Projection failure publishes `None`. The Host bridge regression uses
`Arc::ptr_eq` to require object identity, so a later stream-level owner table can pair
`UiRenderFrameCommandRef` with the real source tree and render generation rather than the synthetic
`PaneData.body_surface_frame`.

Asset Browser is intentionally not included in this slice because its independent model cache needs a
separate invalidation audit. No per-row frame pointer was added, and Host paint sources remain free of
`.surface_frame()` calls.

Static evidence after M3b.3c:

- source publication contract: expected 3 RED and 1 existing GREEN, now 4/4 GREEN;
- Rust bridge regression is present for `Arc` identity preservation; Cargo pending;
- publication is one `Arc` clone per projected pane result, not one clone per node or command;
- stream-level surface table, compact owner keys, native role descriptors, direct Runtime segment
  consumption and managed CPU/RSS/latency/GPU acceptance remain pending. M3b.3 is still in progress.

## M3b.3d Chrome source-owner table static candidate

Host recording now owns a transient `HostRenderSourceTable` for each recorded stream. A published
`Arc<UiSurfaceFrame>` is registered by `Arc` address identity and held by the table for the table's
entire lifetime. A `HashMap<usize, HostRenderSourceKey>` makes repeated registration average O(1),
while the dense frame vector makes resolution O(1). Keys are checked `u32`; overflow fails closed.
No tree-id string, frame clone or frame deep equality is stored per command.

Assets Activity opens one pane-level source-frame scope. Each projected template row tags only the
`HostPaintCommand` slice it appended with its existing `UiRenderFrameCommandRef`. Primitive recording
then publishes `HostRenderCommandSource { surface_key, command_ref }`. Damage clips, Editor-native
commands, synthetic rows and panes without a real Runtime frame publish `None`. The recording container
moves its table through extraction into `ChromeCommandStream`, whose resolver returns a borrowed frame
Arc plus the relative command reference without cloning either payload.

This source is provenance and owner grouping, not yet a unique compiled-output key. One Runtime command
may produce more than one Host primitive, so multiple Chrome commands may intentionally share the same
source pair. M3b.4 must either consume the authoritative Runtime segment directly or add a stable typed
fragment/role discriminator before caching reconstructed Host products. The owned RHI conversion still
drops this table after conversion; no backend reuse claim is made here.

Static evidence after M3b.3d:

- source-owner contract: expected 6/6 RED, now 6/6 GREEN;
- registration complexity contract: expected 1 RED on the linear scan, now GREEN with no `.position`
  in `register`;
- lower Rust regressions are present for pointer-identity deduplication, recording-scope pairing and
  Chrome-stream resolution; Cargo pending;
- adjacent allocation/cursor/range/frame/damage contracts: combined 35/35 GREEN;
- direct Runtime segment consumption, stable fragment/role identity, Asset Browser publication and
  managed CPU/RSS/latency/GPU acceptance remain pending. M3b.3 remains in progress.

## M3b.3e Asset Browser source-frame cache consistency static candidate

Asset Browser now caches one atomic projection result: the composed node model and the authoritative
`Arc<UiSurfaceFrame>` captured from the same `ViewTemplateNodeProjection`. A stable cache hit clones
both handles from that entry. It cannot return nodes from one projection generation with a frame from
another, and it does not call `surface_frame()` during Host paint or pointer routing.

The full result is published through `AssetBrowserPaneViewData`, scene projection and
`AssetBrowserPaneData`. The pane painter opens the same render-source recording scope already used by
Assets Activity, so existing row-local `UiRenderFrameCommandRef` values can resolve against the real
Runtime frame. The compatibility `asset_browser_pane_nodes` function remains a thin nodes-only wrapper
for callers that do not record render provenance; it does not create another cache.

Static evidence after M3b.3e:

- focused source-frame contract: expected 2 new RED with 4 prior GREEN, now 6/6 GREEN;
- lower Rust regressions require pointer-identical source-frame reuse on a stable Asset Browser cache
  hit and pointer-identical preservation through the Host conversion; Cargo pending;
- adjacent source-table/range/cursor/allocation/fragment/damage contracts: combined 37/37 GREEN;
- ten directly touched Rust paths pass scoped `rustfmt --check`; the scene projection hunk passes
  source and `git diff --check`, while whole-file rustfmt still reports an unrelated pre-existing
  `dock_patch` re-export wrapping difference;
- direct Runtime segment consumption, stable fragment/role identity and managed
  CPU/RSS/latency/SVG-GPU acceptance remain pending. M3b.3 remains in progress.

## M3b.4a Runtime fragment identity and borrowed resolution static candidate

A Runtime command is not a unique Host primitive. One command may generate multiple Host paint
commands, and a quad command may split again into separately recorded background and border
primitives. Reusing only `(surface, command_ref)` would therefore alias distinct output. Fragment
identity is now assigned at the final `HostRecordedPaintCommand` boundary, not at the earlier template
row or intermediate Host command boundary.

`HostPaintRecording` maintains average O(1) fragment counters keyed by the compact
`(HostRenderSourceKey, UiRenderFrameCommandRef)` pair. Every final primitive receives a checked `u16`
ordinal. The key contains no string or frame clone, output may be non-contiguous, and ordinal overflow
fails closed for that primitive instead of wrapping or colliding. The resulting Chrome provenance key
is `(surface_key, command_ref, fragment_index)`.

`ChromeCommandStream` can now resolve that key through the source table and
`UiRenderFrameCommands::command_by_ref` to a borrowed `&UiRenderCommand` without iterating the frame
command list. Missing source frames, owner ranges and out-of-range command ordinals return `None`.
This establishes direct Runtime addressability; it does not yet remove the Host/Chrome payload or the
owned RHI converter's `drop(render_sources)`.

Static evidence after M3b.4a:

- fragment identity and borrowed-resolution contract: expected four initial RED plus one incremental
  RED, now 5/5 GREEN;
- lower Rust regressions cover unique ordinals across re-entered command scopes and borrowed Runtime
  command resolution with frame pointer identity; Cargo pending;
- adjacent source-table/frame/range/cursor/allocation/fragment/damage contracts: combined 42/42
  GREEN;
- six directly touched Rust paths pass scoped `rustfmt --check` and `git diff --check`;
- RHI payload hard cut, GPU/softbuffer parity and managed CPU/RSS/latency/SVG-GPU acceptance remain
  pending. M3b.4 remains in progress.

## M3b.4b source-owned prepared segment architecture decision

The M3b.4a address is necessary but not sufficient for payload reuse. Product tracing shows that the
Editor does not preserve a one-to-one projection of Runtime commands: specialized pane painters can
skip structural commands, merge Runtime semantics, lay out or ellipsize text, split a quad into
background and border primitives, and add Editor-owned hover, selection and material overlays. A
matching `(surface_key, command_ref, fragment_index)` therefore proves provenance and stable
addressability only. It does not prove that the reconstructed Host primitive is byte-for-byte or
semantically interchangeable with the original Runtime payload.

The post-recording alternatives are rejected for this milestone:

- Do not infer equality by comparing command kinds, frames or payload fields after Host recording.
  That still pays projection, text allocation and recording cost before discovering a cache hit, and
  it risks treating a deliberately transformed Editor primitive as source-original.
- Do not retain a second Editor command arena keyed by source references. It would duplicate Runtime
  invalidation authority and require independent eviction, damage and backend-recovery rules.
- Do not make RHI depend on `zircon_runtime_interface` or teach it Runtime command semantics. RHI owns
  backend-neutral draw products, while Runtime owns UI command and resource meaning.
- Do not assign the full presentation generation to a changing damage subset. Reusing that key would
  make a partial stream look like a complete immutable frame.

Current Runtime already contains the correct lower-layer precedent in
`graphics/scene/scene_renderer/ui/render/plan_cache.rs`. `ScreenSpaceUiPlanCache` retains one cache
entry per immutable `Arc<UiRenderFrameExtract>` segment, compares source identity with `Arc::ptr_eq`,
visits commands only for changed segments, replays ordered background effects and composes cached
`Arc<PlannedScreenSpaceUi>` products without cloning command payloads. This is not yet reusable by the
Editor window presenter because it is private to the scene renderer, but its ownership and
invalidation model should be generalized instead of reimplemented in Editor.

The reference engines reinforce the same boundary:

- Unreal `FSlateCachedElementData` owns per-widget cached element lists at the invalidation root.
  `FSlateCachedElementList::ClearCachedRenderBatches` discards stale vertices, indices and render
  batches while deliberately retaining source draw elements and clip state. Fast-path painting walks
  invalidated widget proxies, not every widget in the window.
- Slint stores item-local `CachedRenderingData`, derives dirty regions from old/new geometry and
  dependency state, propagates clip/opacity effects, and lets the renderer reject items outside the
  dirty intersection. Backend work remains subordinate to the source item identity.
- Fyrox layout invalidation provides supporting evidence for directional propagation, but its broader
  drawing rebuild is not the performance authority selected for this hard cut.

### Target ownership

1. Runtime owns immutable prepared UI segments derived from Runtime command segments. A segment
   retains source identity, ordered fragment roles, resource dependencies and a generation-qualified
   backend-neutral plan; it never copies its payload into a long-lived Editor list.
2. Presentation publishes compact placement descriptors beside a segment: affine transform, clip and
   z domain. Placement changes can rebuild geometry/backend products without reconstructing semantic
   text/image payloads.
3. Editor-native overlays remain explicitly separate segments. A specialized pane may either consume
   a Runtime-original segment or emit an Editor-owned segment; it cannot silently relabel a repainted
   primitive as Runtime-original.
4. GPU and softbuffer consume the same ordered segment resolver. Backend caches may retain compiled
   vertices, glyph/image bindings or raster products qualified by source segment identity, placement,
   resource generations and backend epoch.
5. Damage selects segment/range identities. A missing source, changed ordering, unsupported semantic
   transform, resource epoch change or backend recovery takes a typed fallback and increments a
   diagnostic counter; it does not fall back invisibly.
6. The existing Chrome/RHI owned payload route remains until both presenters have pixel, hit-route and
   damage parity. The final hard cut removes that route rather than leaving a compatibility shim.

### Dependency-ordered implementation

| Step | Work | Acceptance before next step |
| --- | --- | --- |
| M3b.4b.1 | Extract a Runtime-owned prepared-segment cache contract from the existing scene UI planner without changing render output. | unchanged segments perform zero command visits and preserve ordered background effects; a measured stable-frame trace must bind render/plan/record/image/text/segment-cache sources and prove frame/segment reuse conservation |
| M3b.4b.2 | Add an explicit placement/clip/z descriptor and include it in segment plan identity. | placement-only change rebuilds geometry product without cloning semantic payload |
| M3b.4b.3 | Publish typed Runtime-original versus Editor-owned segment roles at pane composition. | no heuristic source equivalence; unsupported specialization fails closed |
| M3b.4b.4 | Route GPU window presentation through the shared segment resolver and retain discardable backend products. | damage, resize, device recovery and resource-generation regressions pass |
| M3b.4b.5 | Route softbuffer through the same resolver and remove the duplicate owned Runtime conversion. | GPU/softbuffer pixel and ordering parity pass before old route deletion |
| M3b.4b.6 | Remove obsolete Host/Chrome payload duplication and fragment-only cache plumbing. | one source payload authority remains; no compatibility path or hidden full scan |

## M3b.4c renderer dependency-product review

The current image preparation cache retains segment geometry, but it does not retain a complete
ready-to-draw segment product. `ScreenSpaceUiImageSystem::prepare` allocates a new prepare epoch and
iterates every render segment on every call. A reused segment skips geometry rebuilding, yet still
calls `refresh_segment_dependencies`, which visits every unique texture dependency, obtains the
current GPU texture, looks up or creates its bind group and marks the binding with the new epoch.
`retain_prepare_epoch` then scans the binding map. An unchanged resource-management generation only
preserves the requested-to-resolved resource-id map; it does not make the stable frame O(1).

The text cache proves the stable-frame boundary: `prepare_frame_product` compares ordered segment
identity, viewport and font generation and returns its retained `Arc<ScreenSpaceUiTextFrameProduct>`
before any segment work. Its delta path is still incomplete because one changed segment rebuilds the
frame-wide native glyph dependency union and run index from all segment products. That work is less
expensive than reshaping every batch, but it is not a dependency-delta product yet.

The required image architecture is:

1. Publish one frame key containing ordered segment identity, viewport, resource-management
   generation, backend epoch and forced-upload state. A stable hit returns before prepare-epoch
   allocation, dependency lookup and binding retention.
2. Let each image segment retain a shared dependency/binding product. The product owns the lifetime
   of its bind groups; a global epoch sweep must not require touching every unchanged segment merely
   to keep its bindings alive.
3. Rebuild geometry and dependency bindings only for changed segments. Persistent frame composition
   replaces only their directory leaves and preserves ordering.
4. Resource-generation change, device/backend recovery, viewport change and explicit full upload are
   typed fallbacks. Each fallback reports its reason and exact segment/dependency work.
5. Apply the same persistent-directory principle to text dependency spans and run spans so one
   changed text segment does not reconstruct frame-wide indexes.

The current-source residual pressure model is
`tools/runtime_ui_render_dependency_product_pressure.py`. Its default scenario partitions 4,096
frames into 4,060 stable frames, 32 one-segment delta frames and four resource-generation fallback
frames over 64 segments. Each segment has four image dependencies, 32 text dependencies and eight
text run spans; the image binding cache contains 512 entries. This is deliberately narrower than the
older flat-frame models: it measures work that remains after segment products already exist.

| Operation | Current source | Dependency-product target |
| --- | ---: | ---: |
| image segment visits | 262,144 | 288, including 256 typed full-fallback visits |
| image texture dependency/binding lookups | 1,048,576 | 1,152, including 1,024 typed full-fallback checks |
| image binding-retention map entry visits | 2,097,152 | 0 |
| text delta dependency entry visits | 65,536 | 1,024 |
| text delta run entry visits | 16,384 | 256 |

The modeled image segment/dependency reduction is 910.22x after retaining the four explicit full
fallbacks. Text delta dependency and run entry work falls 64x and depends on changed segments rather
than unrelated segment count. These are operation counts, not CPU, allocator, RSS or latency
measurements. The artifact is
`E:\zircon-profiles\runtime-ui-render-dependency-product-pressure-20260829-r3.json`, SHA-256
`697A5000E4A8C3D283D2A815337E18B6A2F76F55E274FB9EE2412D7F42990ACD`. It binds HEAD
`29dfa4a73de5dbc1a4eebe793b50db844c3db93e`, `image.rs` SHA-256
`F1E0FD558DC9AC948163B976DBF46787FEC30C2F0319B1B15BF3F04DF2F99659` and text segment-cache
SHA-256 `64FF29CC167B0812A710D3E25A08204303A879FBDDC161795759B6D2C8207DF0`.

### Unreal resource-lifetime refinement

The checked-in Slate source provides a more specific resource contract than a generic "cache draw
elements" analogy:

- `SlateCore/Public/Textures/SlateShaderResource.h:105-163` defines
  `FSlateSharedHandleData`. A brush/resource handle shares that data with the proxy; proxy destruction
  nulls the shared proxy pointer and invalidates every handle at once.
- `SlateCore/Private/Rendering/ShaderResourceManager.cpp:33-63` reuses the brush's existing
  `FSlateResourceHandle` while the proxy identity matches. It does not require a per-frame epoch touch
  to keep that handle alive.
- `SlateCore/Private/Rendering/ElementBatcher.cpp:518-591` calls `AddElementsInternal` only for
  `ListsWithNewData`; retained `CachedBatches` are appended directly to the current ordered batch
  arrays. Stable widgets therefore do not re-resolve their brush resources while batching.
- `SlateRHIRenderer/Private/SlateRHIResourceManager.cpp:330-360,408-411` performs conditional manager
  cleanup and removes expired UObject/material resources. This is resource-owner maintenance, not a
  draw-segment liveness scan.
- `SlateCore/Private/Rendering/DrawElements.cpp:646-665` can discard stale render batches and
  vertex/index data while deliberately preserving clip state referenced by retained draw elements.
  Source semantic lifetime and discardable backend products remain separate.

The transferable rule is a segment-owned shared resource/binding product whose destruction or backend
epoch invalidates consumers explicitly. Zircon should not reproduce Unreal's UObject/GC mechanism,
and it should not use a frame epoch to prove that every unchanged segment is still alive. Slate still
iterates cached render batches to assemble submission order each frame; the Zircon acceptance claim is
therefore zero stable *prepare/dependency* work, not zero draw traversal or zero GPU submission.

### Retained-memory contract

Dependency products must not turn CPU savings into an unbounded retained-frame cache. Runtime command,
image vertex and glyph/text payloads remain segment-owned. Unchanged payloads are shared across frame
generations; a changed segment publishes one new segment payload version while the previous version stays
alive only until the oldest referencing generation retires. A frame product stores compact
segment/resource handles, never another full-frame payload copy. One binding product is owned per unique
GPU texture identity and invalidated by resource/backend generation. Ordered image/text frame indexes use
a persistent directory, so an additional in-flight delta generation copies
`O(D log S + dependency_delta + run_delta)` metadata for `D` changed segments out of `S`, not the
complete dependency or run table.

The number of simultaneously retained frame generations must be explicit and bounded. Retirement must
release generation-local directory leaves and allow unused binding products to expire without a
per-present liveness scan. CPU metadata bytes, live/retired generation counts and binding product counts
need exact counters. `wgpu::BindGroup` and driver allocations are opaque and cannot be justified by a
Rust struct-size estimate; product acceptance must additionally report process private/working set and
GPU resident/resource counts after warmup, pressure and quiescence.

The conservative retained-memory model
`tools/runtime_ui_render_dependency_product_memory_pressure.py` makes that bound explicit. Its default
fixture uses 64 segments, three simultaneously retained generations, one changed segment per delta
generation and one million presents. Present count does not affect retained bytes.

| Retained category | Modeled bytes |
| --- | ---: |
| one base image-vertex + text-glyph payload | 1,769,472 |
| two live changed-segment payload versions | 55,296 |
| binding, directory, dependency and run metadata | 26,352 |
| target total retained bytes | 1,851,120 |
| rejected three-full-generation payload clone | 5,308,416 |
| payload duplication avoided by segment sharing | 3,483,648 |

The metadata budget is 8 MiB; the default fixture leaves 8,362,256 bytes of headroom. A 4,096-segment,
1,024-binding stress fixture retains 1,148,016 metadata bytes and leaves 7,240,592 bytes of headroom.
These are deterministic modeled sizes, not allocator, RSS, driver or GPU measurements. The focused
contracts pass 10/10, including generation-bound rejection, present-count independence, multi-segment
path/payload scaling and a single-segment directory edge case. Artifact
`E:\zircon-profiles\runtime-ui-render-dependency-product-memory-pressure-20260829-r2.json` has SHA-256
`728DDCA38D2AAD4A5FF60011EDB65DF671C67921512AF92EB84CD18AC0B9CD44`; tool SHA-256 is
`E9F9DAA66F0E2C9895FBB378B0D394BFCCBBDCE463AEA35F6974B29CFF5D7C98` and test SHA-256 is
`5CF5397800478BF07C27CB2575D7F9A0D97C217467B971C4C9CE68A58F7961A4`. The artifact binds HEAD
`29dfa4a73de5dbc1a4eebe793b50db844c3db93e`, `image.rs` SHA-256
`F1E0FD558DC9AC948163B976DBF46787FEC30C2F0319B1B15BF3F04DF2F99659` and text segment-cache SHA-256
`64FF29CC167B0812A710D3E25A08204303A879FBDDC161795759B6D2C8207DF0`.

Product memory acceptance is defined by `tools/ui_render_dependency_memory_evidence.py`. It reuses the
existing same-process CPU/working/private/quiescence contract from
`tools/ui-profile-process-evidence.ps1` (64 MiB end/quiescent growth and 96 MiB peak growth), plus the
existing 64 MiB local and 64 MiB shared WGPU UI image-pool limits. It requires exactly one source-bound
warmup, pressure and quiescent snapshot; at most three live generations; at most 8 MiB dependency
metadata; binding-product/identity conservation; at least five completed same-identity delta cycles;
complete retirement to one generation; and zero global binding scans, per-present liveness scans or
full-generation payload clones. RHI-accounted image payload bytes are accepted evidence, but they are not
misreported as driver-wide GPU residency.

The focused gate passes 10/10. Replaying the two current deterministic model artifacts as if they were a
product capture correctly returns exit 2 with 40 blockers: 38 missing product counters, one missing
interaction record and one invalid source-manifest schema. Rejection artifact
`E:\zircon-profiles\ui-render-dependency-memory-evidence-20260829-current-gap-r2.json` has SHA-256
`BFC9C2083825B9E6B6F1995D32D6A7A19CE587C11EB16F62099E2941EC2ED5DB`; tool SHA-256 is
`CE1FCBF8521C397D33D352B6C42BB56831D2F670C9B2C5DACB1E6A6CA1898658` and test SHA-256 is
`8A45F7844B0A9D29B4BF401EA2C94DAFE399889D17C2D8954D46B930F0D61D02`.

The current product wiring is incomplete in a specific, bounded way. `UiSurfacePresentStats` already
contains shared/local image resident bytes and CPU decoded image bytes. Editor GPU stats publishes the two
GPU-side byte accounts, but does not publish `image_cache_cpu_resident_bytes` into `UiPerfCounter`.
`Get-ZirconProfileCriticalSourcePaths` also omits the renderer `image.rs` and text `segment_cache.rs`
owners required by this gate, and `ui-profile-scenarios.ps1` has no
`render_dependency_memory_pressure` action. Both capture files are externally modified, so this slice
keeps them read-only; the gate already rejects either omission instead of accepting a partial manifest.
Renderer live/retired generation, metadata, source-payload and binding-product snapshot counters are also
absent, and the capture pipeline has no `render_dependency_memory_pressure` action. Those production
owners are externally dirty, so this review records the required contract and makes no overlapping edit.

Static acceptance already rejects any dependency work in a measured stable-frame trace through
`tools/ui_render_segment_evidence.py`. The complementary
`tools/ui_render_dependency_delta_evidence.py` requires, for `N` input segments and `D` changed
segments, reuse counters to conserve `N - D`, render/image/text work to equal the published changed
payload, and binding-map/global dependency scans to remain zero. Its focused contracts pass 9/9. A
historical image pressure artifact is deliberately rejected with 21 missing delta counters and one
missing source manifest; rejection artifact
`E:\zircon-profiles\ui-render-dependency-delta-evidence-20260829-historical-regression.json` has
SHA-256 `8933909683F78C166A5404B2F1345D4A4F6CE836997E8E0A2548D5DF409035AD`.

Product timing must bind those counters to the current source and report CPU, allocations,
working/private set and input-to-present latency. `image.rs` and the text/plan segment-cache owners
currently contain external worktree changes, so this review makes no production edit and no
performance claim.

The generic `ui-profile-capture.ps1` pipeline already publishes measured-run metadata, source
fingerprints and counter timelines, but it has no `render_segment_stable` or
`render_segment_delta` scenario. A normal Editor click is not a valid substitute: it may mutate
several Runtime and Editor-owned segments, and the resulting counter total cannot identify one
dependency-product delta. Product acceptance therefore requires a source-bound harness action that:

1. warms one known image/text segment product and publishes its source segment ids;
2. records a stable phase with no semantic/resource mutation;
3. mutates exactly a declared segment and publishes the changed segment id/count plus expected image,
   text, glyph and dependency cardinalities;
4. separately advances a resource generation to exercise the typed full fallback; and
5. restores the original product before visual/pixel parity capture.

The capture scenario may wrap the generic profiler once its current owner is stable, but it must keep
the measured manifest scenario names `render_segment_stable` and `render_segment_delta`. Aliasing an
uncontrolled `click` scenario or rewriting the manifest after capture is explicitly rejected.

Dynamic acceptance remains M3b.5: current-source managed Rust first, then real Editor hierarchy,
viewport, Asset Browser scroll, button/popup interaction and continuous resize profiles. Required
measurements are CPU, allocator traffic, RSS, input-to-present p50/p95/p99, segment command visits,
fallback counts, SVG decode/raster/stage/upload/resident-resolve counts and GPU timings. This design
decision is not evidence that those budgets currently pass.
