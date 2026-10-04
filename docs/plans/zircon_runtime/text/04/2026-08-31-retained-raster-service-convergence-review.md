---
status: runtime_context_owned_font_policy_exact_cache_vertical_capability_and_geometry_guards_source_implemented_managed_validation_pending
plan: docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
failure: docs/plans/zircon_runtime/text/04/failure-2026-08-31-retained-swash-native-scale-bypasses-physical-raster.md
reviewed_at: 2026-09-01
---

# Retained raster service convergence review

## Scope and current decision

This review covers the retained Editor glyph raster path after correcting its physical ppem policy.
The current-source correction is necessary for product correctness, but it is not the terminal text
architecture. The terminal direction is one Runtime-owned font collection and glyph raster service,
matching Unreal's separation between Slate layout consumers and the shared font cache/renderer.

Do not optimize the Editor `HashMap` or expose Runtime's crate-private Swash types as public API.
First publish a backend-neutral Runtime receipt, cut the retained consumer to it, measure the shared
service, and only then change cache concurrency or residency policy.

## Current-source ownership audit

| Concern | Runtime owner | Retained Editor owner | Structural result |
|---|---|---|---|
| Font collection and revision | `text/font/shared.rs`, `FontCollectionService` | Runtime request identity only | One Runtime font authority |
| Exact shaped face | resolved glyph artifact and `UiTextGlyphArtifactRasterFace` | O(1) artifact face-index consumer | Exact face/instance/generation retained |
| Raster backend | one `GlyphRasterService` and private `SwashRasterizer` per collection session | `TextGlyphRasterReceipt` consumer only | Editor backend deleted |
| Bitmap residency | service measurements precede cache policy | no Editor glyph bitmap cache | One future Runtime residency owner |
| Font metrics and layout | Runtime font metadata, layout, and glyph artifact | artifact placement adapter only | Fontdue fallback deleted |
| Raster scheduling | Runtime service boundary | synchronous MVP request consumer | Future policy remains Runtime-owned |

The retained artifact lane carries source identity, font generation, collection/face handles,
collection index, variations, and shared bytes. The hard cut now consumes that provenance directly.
No Editor system-font, Fontdue layout, private Swash backend, or alternate raster success lane remains.
The Swash scaler identity is the exact 128-bit content/face identity already derived from the complete
font bytes and TTC face index. Collection generation remains the outer stale-publication fence; it is
not XOR-folded into the content identity, which would create avoidable aliases between distinct
`(source, generation)` pairs and discard reusable scaler state for unchanged bytes.
The retained layout cache likewise compares the exact collection generation and
`HostTextFontRequest { face, family, weight }`; it no longer stores only a 64-bit hash of those
values. Its borrowed text/request lookup still avoids allocating a second cache key on hits.
The same audit found two performance-sensitive properties that remain deliberately unchanged until
the managed profile is available: the 2,048-entry cache removes index zero with
`swap_remove_index(0)` and does not refresh recency on hit, and its identity includes absolute
rectangle x/y because layout and placement are still coupled. Moving unchanged text can therefore
miss the retained layout cache, while the current eviction order is FIFO-like rather than LRU. The
follow-up measurement must separate relative layout identity from screen placement and report hit,
miss, eviction, and moved-text behavior before any cache algorithm is changed.

## Reference engine findings

- Unreal `FSlateFontKey` includes font info, outline settings, and scale. `FShapedGlyphEntryKey`
  records the computed render size, and `ComputeFontPixelSize` converts the request to the physical
  pixel size consumed by glyph loading. `FSlateFontCache` owns shared glyph/font cache state; widgets
  do not create a separate FreeType/Slate raster cache.
- Bevy stores physical font size in `FontAtlasKey` and passes the same size to Swash. Atlas identity
  and raster input cannot disagree.
- Slint passes the window scale into layout and rasterizes the resulting physical `run.font_size()`;
  scale changes invalidate the applicable cache state.
- Fyrox exposes supersampling as a separate named policy. It does not make fallback-rasterizer
  identity select a hidden density multiplier.

These references support one physical ppem key and one shared cache owner. They do not support a
fixed 8x multiplier, a renderer-local font database, or public exposure of a third-party backend.

## Target Runtime contract

The public boundary must use Zircon types only. A minimal service request needs:

- exact font collection revision and instanced face identity;
- glyph ID and integer physical ppem bucket;
- normalized horizontal/vertical phase;
- hinting, smoothing, synthetic style, and variation identity;
- requested bitmap format.

The immutable receipt needs:

- request identity and font revision;
- bitmap format and shared `Arc<[u8]>` payload;
- width, height, bearing, and physical ppem;
- typed ready, missing-face, missing-glyph, invalid-request, budget-deferred, and stale-generation
  outcomes.

The service implementation remains under Runtime `text/raster` and may use Swash internally. The
interface must not expose `swash::Source`, `swash::Content`, `fontdb::ID`, `fontdue::Font`, borrowed
font bytes, or a renderer/WGPU atlas type. Native atlas and retained CPU painting consume the same
receipt through separate adapters.

## Hard-cut order

1. Keep the current retained path correct: one physical ppem bucket, Swash primary, straight RGBA,
   and explicit smoothing/phase identity. This is the current failure repair, not final convergence.
2. Add the backend-neutral Runtime request/receipt and a session-owned raster service. Initially feed
   it the exact collection/face identity already present in the resolved glyph artifact.
3. Cut retained artifact drawing to the Runtime receipt. Delete retained `ScaleContext`, raster
   `HashMap`, Fontdue fallback rasterization, and duplicated color-alpha normalization in the same
   migration. No facade or dual success route remains.
4. Make retained layout require the canonical Runtime glyph artifact for renderable text. Resolve UI,
   strong, and mono preferences through the injected `FontCollectionService`; then delete the Editor
   system `fontdb`, `fontdue`, embedded-font copy, host font-set cache, and runtime-artifact font cache.
5. Reuse the service from native bitmap atlas preparation so both consumers share generation
   invalidation and residency. Renderer atlas slots and WGPU uploads remain graphics owners; the
   raster service owns no WGPU resource.

Steps 3 and 4 are source-implemented in one integration window: callers and tests now consume the
Runtime artifact/receipt, direct `fontdb`/`fontdue`/`swash` dependencies are removed from Editor, and
the old font snapshot, layout fallback, raster cache, placement-bin, and backend files are deleted.
There is no bytes-only adapter, facade, or dual success route.

## Core context foundation

Runtime201 Phase A requires one Core/session-owned text capability rather than a font-only manager
plus unrelated process globals. The first bounded M0 slice is now source-implemented:

- `TextModule.Manager.FontServices` keeps its existing registry spelling so the Graphics manager's
  declared service dependency does not break, but the registered object is now
  `TextRuntimeContext`, not the former `TextRuntimeServices { font_collection }` wrapper.
- Every context receives an exact nonzero `TextRuntimeContextId`, pins one
  `FontCollectionService` and the compiled `UnicodeDataSnapshot`, and creates layout sessions only
  from that same font authority.
- The context exposes active/draining/closed/faulted lifecycle vocabulary. Session construction
  checks admission before and after creation, so module cleanup prevents a retained external
  context handle from admitting new sessions after shutdown begins.
- Source tests cover stable identity within one Core, isolation across Cores, font/Unicode authority
  coherence, closed-context rejection, and a fresh context after module reactivation.

This is not full Runtime201 M0 completion. Surface-owned parser/cache lifecycle telemetry and
explicit request/session/artifact/frame correlation, scheduler, residency profile, and removal of
every `shared_*` compatibility path remain open. This slice establishes the lifecycle and injection
root they must migrate into without introducing another global singleton.

### Runtime UI product context binding

The first real product consumer now uses that lifecycle root instead of immediately projecting it
back to a naked font collection. Dynamic session construction resolves one `TextRuntimeContext`
after module activation and passes that same context through project UI loading. Each retained
surface and the fallback menu/HUD extract cache creates its `SharedTextLayoutSession` through the
context admission gate and retains the exact `TextRuntimeContextId` in its measure cache. Runtime
font-asset admission still projects the font collection from the same context, so loading, shaping,
layout invalidation, artifact faces, and asset claims share one collection lineage.

`UiSurface` deliberately does not store `Arc<TextRuntimeContext>`: it remains cloneable and
serializable without turning a process-local capability into document state. Owner-aware builders
inject a context-bound `UiTextMeasureCache`; public standalone and Editor-host constructors retain
their process-owner compatibility path until those owners migrate. Source tests cover exact context
identity retention and typed rejection of measure-cache construction after context closure. This is
the bounded Runtime201 M0 product bridge, not completion of parser extension catalogs, scheduler,
residency, request/artifact/frame health, or worker draining ownership.

The context now also publishes an immutable `TextRuntimeContextHealthSnapshot` containing its exact
context identity, lifecycle, font collection/generation, Unicode snapshot identity, cumulative
successful layout-session admissions, and active logical layout-session families. Admission and
active-family counters use checked atomic updates and return typed exhaustion errors instead of
saturating or wrapping into ambiguous telemetry; rejected post-drain construction changes neither.
One context admission creates one lease family and one context-qualified
`TextSessionId { context, sequence }`. Sequence allocation is per context, checked, nonzero, and
fails closed without reuse; cloned sessions/caches explicitly share the family ID and lease while
their parser/cache owners remain distinct. Lifecycle state and active-family count are packed into
one atomic word, so admission, close, and final release have one linearization point and health cannot
observe `Closed` with a newly admitted family. `close()` remains Draining while a family is live and
the final family release atomically publishes Closed; release underflow publishes Faulted without
wrapping or panicking. This is deliberately context/session-family health only. Surface identity,
request/artifact/frame correlation, workers, cache residency, cursors, and terminal receipts remain
open.

The builtin Graphics module host now resolves `TextRuntimeContext` as its capability boundary and
only then projects the collection needed by the renderer. The retired
`font_collection_service_for_core` accessor and re-export are deleted after a repository-wide scan
confirmed they had no remaining product callers. Dynamic UI and builtin Graphics therefore enter
text through the same context authority; the stable manager registry spelling remains unchanged for
declared module dependency compatibility.

The context boundary is intentionally narrower than a process-wide text cache. `RichTextParser`
owns parser-local decorator/emoji registries, compiled markup entries, single-flight state, and
resettable telemetry; `SharedTextLayoutSession` owns that parser so `UiTextMeasureCache::clear()`
clears only its surface and `sample_compiled_rich_text_cache()` reports the same owner. Moving the
parser into `TextRuntimeContext` would let one surface invalidate another surface's cache and would
merge otherwise independent diagnostics. This matches the Unreal ownership split where an
`FTextLayout`/marshaller is session or widget owned while the font cache remains the shared authority.
Future parser work must preserve this owner-local boundary; only immutable font/Unicode capability
and lifecycle leases may be shared through the Core context.

`UiSurface` cloning exposed one remaining violation of that rule. Surface identity already allocates
a fresh owner on clone, while the derived `SharedTextLayoutSession` clone previously retained the
same `Arc<RichTextParser>`. Clearing either clone could therefore invalidate the other surface's
compiled markup and consume its interval telemetry. `SharedTextLayoutSession` now implements clone
explicitly: font authority, shaped/hard-line cache snapshots, budgets, reports, and the context lease
family are retained, but a fresh parser/cache owner is created. The context still counts the clone
family once, while parser clear and telemetry are surface-local. A focused regression compiles the
same markup in both owners, clears one, and proves the other remains resident.

The Runtime artifact owner already has a canonical visual projection for generated ellipses and
synthetic all-LTR visual runs, including virtual-glyph source anchors and font-generation rebuilds.
The surface adapter still carried an older blanket rejection for the same line classes, so retained
single-line ellipsis could never consume the artifact that Runtime had successfully published. That
stale gate is now source-removed. The adapter admits generated visual lines only when the artifact
contains the exact final layout line and still passes generation, collection, and font-handle
validation. Unsupported mixed-direction virtual lines remain `None` at the artifact owner instead of
being guessed by the adapter. Steps 3 and 4 still remain one hard cut; do not bridge only the
non-ellipsized branch while leaving Fontdue/Swash as a second success route.

## Measurement plan before cache optimization

The shared raster service now exposes fixed low-cardinality request-route, actual-format, bitmap-byte,
lock-wait, lock-hold, backend-time, success, and failure counters. The existing native atlas source
cache separately reports hit/miss, pending de-duplication, worker backpressure, completion budgets,
evictions, and current resident entries/font/bitmap bytes. The Editor-private Fontdue/Swash cache
observations described by the earlier baseline no longer exist after the hard cut. Before changing
the surviving Runtime cache or scheduling policy, collect the remaining matched measurements needed
to distinguish:

- lookup mutex wait and hold time;
- duplicate concurrent raster work before publication for one key across synchronous and worker
  consumers;
- raster CPU time by alpha/subpixel/color route;
- resident entry/byte current and peak values;
- generation invalidation, eviction, and deferred work counts;
- surface artifact-line hit and fixed miss reasons: missing layout/artifact/projected line, stale font
  generation, exact layout mismatch, and font-handle mismatch.

Run Windows managed release and profiling lanes for 1/16/128/512 distinct glyphs, shared and distinct
faces, 13px at 100/125/150/200%, cold/warm caches, alpha/subpixel/COLR/bitmap-color glyphs, and one
generation replacement. Record 31 raw samples with p50/p95/p99, allocations, working-set delta,
CPU sampled stacks, GPU timestamps where applicable, and package power. The UI12 upward workload adds
1000 click, 1000 pointer move, and 200 resize/scale transitions.

Only measured bottlenecks authorize one of these changes:

- single-flight for duplicate misses;
- sharded lookup for lock contention;
- byte-bounded LRU for resident growth;
- asynchronous scheduling for miss latency.

No latency, power, Unreal parity, or optimality claim is valid until matched data exists.

### Repeated stable-glyph baseline

The source review found a concrete post-cutover complexity gap in the retained Editor consumer.
`PaintTextLayout` is bounded and reused, but drawing every cached layout still calls
`UiTextGlyphArtifactRasterFace::rasterize_glyph` once per visible glyph. The collection-owned
service currently serializes those calls through one Swash mutex and publishes a new bitmap
allocation each time. Stable-frame backend work is therefore `O(visible glyphs)` rather than
`O(distinct raster misses)`, while the native atlas path already owns bounded CPU source residency
and pending-request de-duplication.

Before changing this algorithm, the ignored managed-release harness
`text_runtime_raster_authority_sync_service_repeated_request_profile` measures 31 alternating
samples of one warmed exact request and one shared receipt clone, each over 1,024 iterations. It
reports cold latency, raster and clone p50/p95/p99, their p95 ratio, and bitmap bytes under marker
`TEXT04_SYNC_RASTER_REPEATED_REQUEST_PROFILE_V1`. The benchmark has no performance pass threshold;
its purpose is to choose between retained receipt residency, single-flight, sharding, or leaving the
service uncached. No cache policy is authorized until this managed result and sampled-stack evidence
exist.

The first managed release submission (`452fd197875f43499206952d5dc46516`) was rejected before Cargo
because the new performance source was absent from the validation overlay's owned file set. After an
exact claim and current-hash attribution, the replacement invocation exceeded its local command
window before returning a receipt. Per coordinator rules it was not retried or polled. There is
therefore still no accepted release profile, sampled stack, allocation, working-set, or power result.

## Validation and visual evidence

Lower gates must cover physical buckets 13/16/20/26, equivalent-bucket reuse, phase and smoothing
identity, generation replacement, Fontdue-route deletion after hard cut, COLR unpremultiply, embedded
bitmap straight RGBA, and zero-alpha RGB normalization.

The existing Runtime WGPU DPI gate proves a 1x-to-2x cache transition but writes no image. Product
acceptance still requires current-source real WGPU frames. The multilingual Runtime proof remains:

`docs/tests/runtime/text/runtime_text_mvp_foundation_product_framebuffer_20260831.png`

UI12 must additionally capture real retained Editor frames at 100/125/150/200% under
`docs/tests/runtime/text`. These must show the actual Workbench text with rounded rectangles and SVG
content in the same frame; strategy diagrams, HTML previews, offline bitmap enlargement, old images,
and anything under a Cargo target directory are not evidence.

## Current state

Architecture review and the pre-optimization measurement design are complete. Runtime now publishes
the backend-neutral request, immutable receipt, typed failure, bitmap-format, hinting, smoothing,
mode, synthetic-style, and shared 3x4 phase contracts. `None`/grayscale/subpixel smoothing are all
representable without collapsing cache identity. Each `FontCollectionService` owns one private
`GlyphRasterService`, and exact artifact faces can request receipts without exposing Swash or borrowed
font bytes. Native `GlyphRasterKey` now projects into the same public request, and the synchronous
service plus asynchronous native worker share one typed `TextGlyphRasterRequest` to
`SwashRasterRequest` validator/adapter. Unsupported SDF/MSDF and synthetic-bold bitmap bridge inputs
fail closed instead of silently dropping identity. Existing worker batching, pending de-duplication,
backpressure, byte budgets, face epochs, source LRU, and GPU atlas residency remain unchanged. Fixed
low-cardinality counters cover request route, actual bitmap route/bytes, lock wait, lock hold, backend
time, success/failure, and artifact lookup outcomes.

Editor hard-cut steps 3 and 4 are source-implemented. Retained layout now accepts only a complete
Runtime glyph artifact and fails closed when the exact artifact/face set is absent. It uses the
Runtime line baseline, glyph origins, face/instance handles, physical ppem bucket, and shared phase
identity. Drawing requests `TextGlyphRasterReceipt` from the exact artifact face and consumes receipt
size, bearing, format, and shared bitmap. Synthetic oblique is requested from Runtime rather than
applied a second time by the CPU row mixer. Editor's system font database, Fontdue layout/metrics,
private Swash `ScaleContext`, unbounded glyph `HashMap`, duplicated color normalization, 8-bin phase
model, snapshot caches, source files, tests, and direct Cargo dependencies are removed. Including the
Runtime UI and Graphics context binding, the complete Session-owned production/test/Cargo source set
was 72 files with 3,821 insertions and 6,649 deletions at the parser-owner clone snapshot, including
18 new files and excluding plan records. The subsequent six-path identity/lifecycle slice is 986
insertions and 2 deletions relative to the current HEAD, including its three new owners; a full
manifest recount is deferred to the next sealed validation snapshot. The 72-file snapshot supersedes
the earlier 67-file shortstat, which omitted
`core/framework/text/mod.rs` and the two Runtime surface-artifact owners.

The touched Dynamic Runtime UI root reached 990 lines after its existing behavior tests first moved
to `runtime_ui/tests.rs`. Input dispatch, focus/navigation ownership, pointer capture, publication
refresh, and dispatch-output recording now live in the coherent
`runtime_ui/input_routing.rs` child. The orchestration root is 428 lines, input routing is 574
lines, and its 11 behavior tests remain in the 439-line test owner. The move changes no routing
algorithm or public contract and closes the touched-owner structure warning before further context
work. Other touched production owners remain below 800 lines.

The Core context foundation is also source-implemented. `TextModule` owns one
`TextRuntimeContext` per Core; `text_runtime_context_for_core` exposes the capability root and the
former font-only Core accessor has been deleted after its last product caller migrated.
Module cleanup moves it through draining to closed, retained handles reject new layout sessions, and
reactivation creates a distinct context and font collection. Rustfmt and static retired-symbol scans
pass for this slice. Dynamic project surfaces and the fallback Runtime UI extract cache now derive
their layout sessions from that same context and retain its exact identity; no Dynamic session
constructor uses `font_collection_service_for_core` as its text capability boundary. The focused
59-test static contract set passes, but these source tests have not yet obtained a managed Cargo
result.

Scoped Rust 2024 formatting and `git diff --check` pass. The 116-test Runtime text static run passes
114; the two failures remain foreign shared-worktree checks: an in-flight UI texture dependency owner
does not match the test's old source string, and unowned
`graphics/scene/scene_renderer/ui/image.rs` is 829 lines against its 800-line gate. Managed Runtime
Cargo validation copy `d4da4a0a64424fbfa97845713680b0cf` failed during overlay ownership
materialization before Cargo started. After refreshing the exact path claim, replacement copy
`1e51cc39937545b2a729d7ff8bcbd37a` with request `8c9c4e7e59774579a4db9544fb067aee`
was accepted under `D:/cargo-targets/verify` for the current public-contract/service/atlas-key/worker/
artifact source set, but it has no accepted Cargo result yet.
The expanded context/product snapshot was accepted as managed validation copy
`f83b46a6e8ac4be480abad954a32784a` with request
`84dde68097f9408eb4df88201e369c53`. Its 32-path source manifest includes the shared raster
authority, Core context, Dynamic project surfaces, and fallback UI extract cache, and targets
`D:/cargo-targets/verify`; the receipt is only `materializing`, so it is not a Cargo pass ticket.
After adding the context health source and tests, the changed 33-path snapshot was separately
accepted as copy `92ea424e58844b4a8e5714b46c0a0545` / request
`719cec96ac764c8f99e2d2443362c4ab` with the same focused filter and D-drive policy. This newer
receipt is also only `materializing`; neither receipt is polled or represented as terminal evidence.
The former 34-path snapshot, including the Graphics accessor hard cut, was accepted as copy
`11fa5758d05b44a1a02d7da542fcf1b2` / request
`d39a51d99f3d462ab8cd234e35c10c26`, but the subsequent test/input-routing split supersedes its
source manifest. The current 36-path Runtime snapshot was claim/attributed as
`021962ded48442d9b9240d476fddf209` / `64fb653e8b9547978f7df80342a8b842`
and accepted as D-drive managed copy `57b1960cb4c24b1f83e1ede711503bc0` / request
`86a648d825f14f9bad4d270a418df493`. That receipt is only `materializing` and was superseded by the
context lease implementation. The current 37-path Runtime snapshot was claim/attributed as
`ea81d3e888724051b7cfbb73cec4f918` / `5ce0b8902afa4019a93772faa17e1da8`
and accepted as D-drive managed copy `71435eb9a80b4cb2a7b14c3e7466f08e` / request
`f546baad8fc3473eaf0a8843e5f5df6b`. That receipt is also only `materializing`; no status query
or Cargo success claim follows. The focused 59-test non-Cargo contract set passes on this exact
source.
The repeated-request release profile has likewise produced no accepted ticket: its first overlay was
rejected before Cargo and the attributed replacement returned no receipt before the command timeout.
Measured cache-policy selection,
performance/power sampling, current-source Editor WGPU frames at 100/125/150/200%, PNG inspection,
milestone commit, and WeCom notification remain pending. No performance, power, visual, Cargo, or
terminal milestone claim is made.

The explicit parser-owner clone repair attribution is superseded by the current six-path
identity/lifecycle attribution under `runtime-text04-20260831-r1`: lease claim
`ebc502589c9142a69e1958d9c8f2b07e` and baseline attribution
`335996f7aef741f1b2ea900570eb85fe`. The prior 37-path managed snapshot is therefore superseded; the
next managed validation must materialize the full current manifest before any Cargo or milestone
claim.

The Phase A product-constructor boundary now has a source contract in
`tools/tests/test_runtime_text_infrastructure_compile_contract.py`. It pins Dynamic session
construction, the fallback Runtime UI extract cache, and the builtin Graphics module host to the
Core-resolved `TextRuntimeContext`; those product sections may not call
`shared_font_collection_service()`, `UiTextMeasureCache::default()`, or
`SharedTextLayoutSession::new()`. Test-only and explicitly documented standalone compatibility
constructors remain outside this narrow gate. Existing WGPU/SceneRenderer globals are now explicitly
marked `Runtime201 standalone compatibility`, and the source contract permits exactly four adjacent
global calls across their three owner files; a fifth or unmarked call fails the guard. This advances
Runtime201 `RT-TXT-P2-010`, but does not close `TXT-G02`: Editor preview, every standalone tool,
feature gating, and removal or migration of the compatibility constructors remain open. The focused
non-Cargo contract set passes 60/60 in 1.616 seconds. Final source SHA-256 values are guard
`6F9645DF3DD7C8EA1D9992DB9BAE814409256E133A656AB9040A80286D0489B4`, WGPU constructor
`8BAB2656938E7029B118954AB68429D8E21B7C39C03B94EB053F9D517581FD6D`, SceneRenderer constructor
`8444731760941EFDD0BAD9AF1C507507662C187EBCAB17725DDB2B9B39D14258`, and icon/startup constructor
`51DD683BF5D3935CA464674EBECD97423CBD232B131AF4CA7417BDF78C72A6CA`. Current attributions are
`d878544a9d4f44d1b5ed32059e7ed52f`, `cad18c060a3f448889465b9a4cd6cd69`,
`35bc9b52016348979a9f404ec086cfd7`, and `b62957cfc4a14316a7df14d7d7475179` respectively. This is
source-contract evidence only; managed Cargo and product-frame evidence remain pending. Whole-file
Rustfmt check exposes pre-existing import/test formatting in the three Graphics owners, so no
unrelated formatter rewrite was applied; exact diff-check passes.

The module-lifetime review does not justify a new renderer-wide lease. Builtin Graphics declares
both a module dependency on `TextModule` and a manager dependency on
`TextModule.Manager.FontServices`; Core rejects Text deactivation while Graphics is running and
shuts dependencies down after their consumers. A caller retaining a manager `Arc` after module
unload is a general manager API lifetime concern, not evidence that the renderer should create a
second text lifecycle authority.

The remaining system-font mutation has instead moved to its correct owner. Unreal's
`FSlateRenderingPolicy` consumes injected `FSlateFontServices`, while its D3D/OpenGL standalone
renderers construct those services before renderer creation. Zircon now follows that boundary:
`ScreenSpaceUiTextSystem` no longer discovers fonts or mutates the collection. Public
`TextSystemFontPolicy` is selected by `TextModule::for_target`; builtin client and editor targets use
`DiscoverPlatform`, while server/default standalone contexts are `PackagedOnly`. The policy is
applied to the initial `FontDatabase` before `FontCollectionService` publication, so the initial
revision remains generation 1. Context health reports both the selected policy and the discovered
face count.

This source-implements the owner and target-policy portion of Runtime201 `RT-TXT-P1-006`; it does
not complete `RT-TXT-P1-007`. Versioned project policy, allowlists, locale packs, shipping
determinism, fingerprints, and admission receipts remain open. The expanded non-Cargo contract set
passes 61/61 in 2.276 seconds; scoped Rust 2024 formatting and exact diff checks pass. The exact
eight-path claim is `d53e6b049f844a9ca0b2c9d038805ee0`, with current attribution
`7f29530b2ad941d5bac8686ae0e90af6`. No managed Cargo, WGPU, PNG, performance, power, milestone,
or WeCom claim follows from this source slice.

The exact direction-alias cache comparison has also been corrected. Its bucket fingerprint already
contained font collection and Unicode snapshot identity, but the post-collision candidate compare
omitted both fields. The compare now checks both authorities and a regression rejects foreign
collection and Unicode-generation lookups. Cache capacity, indexing, and eviction are unchanged.
The focused non-Cargo suite is 62/62 in 2.324 seconds; the shaped-cache production/test owners pass
Rust 2024 formatting and diff checks. The current three-path attribution request is
`a6b15577c79840c0bbc02376f5eeb910`; SHA-256 values are
`5DA7EAF1F3652D4A41D2B110F780C60CD29645B0DD8BA70D1CA8D1E7B5E61763`,
`42A9E9D5BBD21E6D9FA750842F38DDEED892AF221C2F6CF91C5CBB4741ACE4E4`, and
`46D619421ABDD6DE73DC2661D7CC7E5DAC2E46B8D30E464B80A5E2A53D4ADE1B`. Managed Cargo and all
visual/performance evidence remain pending.

The vertical provider capability boundary is now fail-closed as the next Runtime201 `RT-TXT-P2-003`
source correction. The trait default returns `UnsupportedWritingMode` instead of delegating to the
horizontal path. All three Runtime production providers already implement vertical shaping
explicitly; the rich-vertical counting test provider now does so as well. A direct regression and a
source contract prevent a horizontal-only provider from fabricating vertical success. The focused
non-Cargo suite passes 63/63 in 2.144 seconds, and scoped Rust 2024 formatting plus four-path diff
checks pass. Claim `fcaba3b5c1b7437ca4fdf71dfa410b6b` and attribution request
`3aaa973cabca42df8d08ad6dd38223dd` cover SHA-256 values
`BE51D717B05F90A18161AC2E2581165902DDDE6F73916491AA9BDF35689E439B`,
`38A5F873F06AADBBBF68D4640880F3B6764E3417E3B75C1F5D6E5933E91432F8`,
`5805033143010061D5B79A8DB4C1A64E853B2AEFED38EAFA0EBAE5962C611E74`, and
`0E5016EDBA405354F0EB9673386675F5E58E3935B277154ABDBC1D32B21D6015` (the shared source guard was
extended by the subsequent geometry contract). This is a capability and
error-contract fix only; managed Cargo, real renderer frames, performance, and power evidence remain
pending.

The next MVP correctness slice hardens `VerticalRl` geometry against finite-input overflow. The column
layout helper preserves its normal `f32` arithmetic, but falls back to the untruncated `f64` frame
geometry and clamps only when a candidate result is non-finite. This preserves normal right-to-left placement and the
single frame-building pass while preventing `f32::MAX` inputs from publishing infinite geometry. A
boundary regression covers extreme frame, advance, and height values, and a source contract keeps the
finite-intermediate rule explicit. The constrained runtime-text infrastructure contract module passes
60/60 in 1.889 seconds; scoped Rust 2024 formatting and all owned-path diff checks pass. Current geometry
owner SHA-256 values are `9453DCD7D1AFFF9CF96E1A4B26041D2EF000C6ED9C3EFE82ACCD0CA8757135BD`,
`A07D750CA76AA9181C6AF96263BE3DC497AB552F3C66A8DC40920B6DAADD500B`, and
`1FEBD79D0B3D17D7E54DF39D24DCBD1E82CC90FFA9599225D2CBE6EA3FC59827`. This is a geometry
correctness guard only; it does not claim optimization, power, Cargo/WGPU, or real-renderer screenshot
acceptance, all of which remain managed-validation pending.

The vertical shaping owner receives the same finite-input protection. `apply_vertical_layout` now keeps
the original `f32` accumulation for ordinary values, while tracking `f64` cluster/cursor totals and
falling back to finite geometry only when an `f32` candidate overflows; run measured width follows the
same rule. An extreme-glyph-advance regression checks glyph positions and advances, line width, and run
extents, with a source contract preserving the dual-precision boundary. `shaping/vertical.rs`,
`shaping/vertical/direct.rs`, and its tests currently hash to
`B81546E27CF76E54923589106FD7E1B966ECC6C9EE7847864E1314AA557588A4`,
`6CCB6BEE1CCBF7F0851E1A16C684F7F06823D17B0E54D3B2F6291B6BEB6AF295`, and
`79CA9E22273C2B18169955EAF542BC3B263FBC9A56C92AD0D7D3F00D3C5089A8`; the 60/60 contract run above
includes this regression. Direct positioning reuses the shared finite-geometry guard for cluster advance,
cursor, upright offset, and multi-column measured width, keeping the Direct and Cosmic provider paths behaviorally aligned. This remains numeric correctness work, not a shaping-performance, power,
Cargo/WGPU, or PNG acceptance claim.

The final vertical numeric guard covers upright glyph centering as well. The orientation owner preserves
ordinary `f32` offset arithmetic and uses an exact `f64` difference only when that candidate is non-finite,
so an out-of-invariant extreme input cannot publish an infinite `offset_x`; the regression pins the finite result to
`-f32::MAX * 0.5` (the defensive fallback is exercised only for a non-finite candidate). `vertical/orientation.rs` hashes to
`FAFF0CA26B648F66F8B6E8771EA1414AAC64017FC1A6C15EBE04A2F2C06343FE`. This is numeric correctness only;
performance, power, and visual acceptance remain pending.

The measurement owner now applies the same finite-geometry rule to multiline intrinsic size.
`measure_text_size_with_provider` keeps ordinary `f32` height accumulation, tracks the exact `f64`
total, and falls back only when the candidate overflows; a two-line `f32::MAX` regression prevents
`TextSize.height` from publishing infinity. The regression is included in the 60/60 source contract.
`layout/measure.rs`, `layout/measure/measured_line_contract_tests.rs`, and the contract module hash to
`5A600A1170CD6DA9D979888BEDF4A5B4AE1E294F37B4E7B400E633BEBC72190B`,
`9FE53A5D0CB54791BD8FE5130861BB896D74AEBAC193871ADF87FEF0FCB41ACB`, and
`1FEBD79D0B3D17D7E54DF39D24DCBD1E82CC90FFA9599225D2CBE6EA3FC59827`. This is measurement correctness only;
performance, power, Cargo/WGPU, and visual acceptance remain pending.

The horizontal shaping owner now applies the same finite-geometry rule. Direct shaping, Cosmic
hard-line normalization, and partial horizontal composition share `shaping/horizontal/mod.rs::position_glyphs`:
ordinary glyph cursor accumulation remains `f32`, while only a non-finite candidate falls back to
an exact `f64` total; multi-line run `measured_height` uses the matching `finite_sum` guard. A two-
`f32::MAX`-advance regression checks finite glyph x, line width, and height. The constrained source
contract is now `60/60`; `horizontal/mod.rs`, `horizontal/direct.rs`, `horizontal/composition.rs`,
`cosmic.rs`, `cosmic/hard_lines.rs`, and the regression test hash to
`FC639ECD3D25F293A013BC49C649471514862F4C1B76E450D4EBBE78E86E8A69`,
`82FF15212B3AD66035AB5228E6461DFAC176543E8C096AA46F5F14454773797C`,
`56EAE5D246D8A74F363A17E8E6E11899C6D9B5EB35B46FE4D8BBF5DAD19E3DAC`,
`C06FDBEF28A03816961192883183492EA680CDB757C751A3569132E78EFD4BB1`,
`336047454307AA2F8929EEFE4FAC92AAC27467FD1E75B031C4942A07C2D4D992`, and
`71E8A97B6C44FC71746121FE92BBF25E8AE57589A11D3379FD0F2804D83735A5`; the contract module is
`1FEBD79D0B3D17D7E54DF39D24DCBD1E82CC90FFA9599225D2CBE6EA3FC59827`. This is horizontal numeric
correctness protection only; shaping performance, power, Cargo/WGPU, and visual acceptance remain
managed-validation pending.

Finite-geometry publication now has one module owner. `text/layout_geometry.rs` owns
`finite_geometry`, `finite_f32_or_geometry`, and `finite_sum`; measurement, VerticalRl,
horizontal shaping, vertical shaping, orientation, shared cluster geometry, and tab measurement consume that
owner without retaining private copies. A source contract rejects reintroduced duplicate
declarations. The central owner hashes to
`7E55F5AD5E35AB9C966393440A3058394B7CE62D6465A42E945AF4BE2E752180`, and shared cluster
geometry hashes to `4253499C33B1670861E76246F652766536E540AB4C94512101C1F550B2903B7D`.
Tab measurement hashes to `1A7CA9018B82468245D96BBBADD921F418F32DF0AB79B6FACEFBCFAA213036C4`;
its interval, cursor, and no-tab fallback use the same finite-publication owner.
Ordinary finite `f32` candidates remain unchanged; only non-finite candidates use the exact
`f64` recovery path. This is infrastructure correctness convergence, not a performance claim;
Cargo/WGPU, real-renderer PNG, power, and profile acceptance remain managed-validation pending.

The wider `test_runtime_text*.py` static discovery ran 129 tests with 127 passing. The two remaining
failures are both foreign to this lifecycle and point to
`zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs` (an existing
`UiTextureDependencies` signature contract and its 800-line owner budget); the path was left untouched.

## 2026-09-01 Runtime Text algorithm/profile gate before optimization

The current optimization direction remains architecture-first. Source review found that the highest
risk is not a local `reserve` or loop micro-cost: system-font discovery inherits backend face
enumeration order, the collection snapshot is a Context-owned identity, and shaping/layout/cache keys
must observe the same collection and Unicode authorities. Sorting discovery candidates, adding a
versioned font fingerprint, or changing cache admission before measuring these boundaries could change
fallback order, startup cost, and cache residency at once.

The next action is a managed Windows release baseline before any algorithmic change. Fixed cohorts are:
packaged-only and platform-discovery Context startup; 1/100/1,000/10,000-face synthetic collections;
cold and warm Latin/CJK/emoji shaping; horizontal LTR, explicit RTL, and VerticalRl layout; and
repeated retained-frame cache hits/misses. Each cohort records p50/p95/p99 wall time, allocation
count/bytes, peak RSS, face count/order, collection and Unicode identity, shaped-cache hit/miss/alias
counts, layout rejection counts, and power or energy-per-frame when supported. Raw samples and
environment/build/profile identity must be retained, with an Unreal Slate/FontServices lifecycle
mapping; a faster isolated loop alone is not acceptance evidence.

Only after that baseline identifies a dominant term may a separate implementation slice choose stable
candidate sorting plus descriptor fingerprinting, discovery snapshot reuse, or cache/index changes.
The candidate must retain fallback ordering semantics, use one fingerprint owner, and repeat the same
cohorts with before/after allocation/RSS/power comparison. Until then, no performance gain, power
improvement, or asymptotic claim is made.

## 2026-09-01 finite-geometry accumulation recovery owner

Status: `current-source implementation complete / managed validation pending`.

The pre-optimization review found a correctness defect in the prior dual-state pattern. After the
first `f32` overflow, the published value was saturated, but the next finite candidate could be
computed from that saturated publication and accepted without consulting the exact history. Thus
`f32::MAX + f32::MAX - f32::MAX` incorrectly collapsed to zero. This was a missing state owner, not
a local performance hotspot.

`text/layout_geometry.rs` now owns `FiniteGeometryAccumulator`. Ordinary additions retain their
existing ordered `f32` result. Once recovery is required, every later publication is derived from
the retained `f64` history, so sign cancellation cannot discard the pre-overflow total. `finite_sum`,
shared cluster geometry, horizontal and vertical glyph cursors, Direct/Cosmic vertical cluster sums,
multiline height, and tab cursors all consume this owner. A source scan finds no remaining manual
`f32`/`f64` accumulation pair under `zircon_runtime/src/text` outside the owner itself.

The regression pins `MAX + MAX - MAX == MAX` and the following subtraction back to zero. The focused
source contract passes 60/60 in 2.491 seconds on the final rerun. Wider `test_runtime_text*.py` discovery runs 129 tests
with 127 passing; its two failures remain foreign `scene_renderer/ui/image.rs` signature and 829-line
owner-budget findings, and that path was not modified. Source-range glyph width aggregation now also
uses `finite_sum`; a two-`f32::MAX`-glyph Rust regression is present and awaits managed Cargo execution.
Rust 2024 formatting for the eight Rust owners and the eleven-path diff check pass.

Terminal SHA-256 values are:

- `layout_geometry.rs`: `DAA0E5E9E8CCD949ECD89F1E01C8B985115F6EDDE7D00DA77EBB7526EFE79D8D`
- `cluster_geometry.rs`: `9F3F51BDF1347A280EBED32D5E1FAFF1F14A66D55C3D91B6A83AF7413D0EF7C1`
- `layout/measure.rs`: `5B63E8FC76B014C91C7839062BC566B3EC500A0BD613717B6CE613005CDC6F79`
- `layout/measure/measured_line_contract_tests.rs`: `B93534525855DACB65A92CC288B8D780F535C463E91BEDCAB3AE51DF9E102ED6`
- `shaping/horizontal/mod.rs`: `A83381B22E2C3C1C239D815FA624CD6D825B4BDE7A7CE736A21A9E4218ED1A11`
- `shaping/vertical.rs`: `531A4E995A359DEF78AFA5774F442CA74822A65AABA046D7F4A10033940E1BD2`
- `shaping/vertical/direct.rs`: `5E862D4BAF43380F184FBB68A4C94ABBE053342B74CA12A98F571F538EA88550`
- `layout/tab.rs`: `EB4F327B4F8A72EE67F168163A84ABD7925DB1EE68F4E28ECB662862A716256F`
- source contract: `1825B500D16EB15AFAE95E1497B3649DD6472DAB359820AD7873B553692425A3`

This is MVP numeric correctness and single-owner convergence, not a performance result. Cargo/WGPU,
the real-renderer PNG, power, release profiling, and quantitative Unreal-experience comparison remain
managed-validation pending. No milestone commit or WeCom completion notice is legal before that gate.
