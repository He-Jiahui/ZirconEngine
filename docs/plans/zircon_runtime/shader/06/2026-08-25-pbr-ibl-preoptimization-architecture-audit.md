# Shader06 PBR/IBL Pre-optimization Architecture Audit

Plan: `docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`

Status: `implementation_in_progress_validation_deferred`

Date: 2026-08-25

Last updated: 2026-08-31

Current gate: `C2 capture_replay_reflection_feedback_isolation_six_face_direct_light_grid_and_atomic_terminal_publication_implemented_static_validation_passed_managed_gpu_validation_pending; C3 typed_target_identity_and_ticket_owned_array_copy_implemented_validation_pending; C4 runtime_cache_writeback_implemented_editor_staging_validation_pending`

The explicit capture replay now builds one opaque command set, projects one
capture-specific Forward receiver when the light grid is disabled, or six
face-specific receivers when direct lights are present. Its local-probe header
and planar-reflection parameters are immutable zero metadata, so the unchanged
generic group-one ABI cannot recursively sample local or planar reflection
providers while the scene-owned global environment remains available. If an
asynchronous PSO is still deferred or has failed, the replay count no longer
permits the partial cubemap to enter filtering or publication. Scoped source
contracts, Rust formatting, and diff checks pass. The E-drive isolated Naga
29.0.3 probe validates the six finite-normalizer delegates (6/6), environment-
only Deferred assembly (1/1), skybox variants (2/2), standalone fallback
assembly including its scalar capture rule (1/1), and the production Forward
capture-surface-policy helper (1/1).
Cargo, WGPU, RenderDoc, image, timing, and power evidence remain pending.

The direct-light review rejects common viewport preparation because all six
faces are recorded before one submission and each face needs immutable
view-specific z-bin/tile-mask data. `EnvironmentCaptureLightGridPlan` now packs
the captured light list once, derives six grids from the six capture cameras,
and shares one CPU byte payload across 18 upload ranges. The GPU Scene light
owner is written once before the existing staged mesh/GPU Scene transaction;
that transaction derives `visible_instance_remap_params.values.y` from the
same `light_shadow.len()`, so the shader-visible count and light-buffer payload
cannot diverge.
An empty light list builds zero face grids, retains a zero-byte CPU payload,
allocates zero capture-grid bytes, emits zero grid uploads, and reuses one
disabled receiver; a lit capture emits 18 uploads and owns six
receiver groups. Exact-size grid allocation is bounded by the existing layout
at `128 + 4096*4 + 8192*4 = 49,280 B` per face and `295,680 B` for six faces.
These are source/resource bounds, not measured CPU/GPU timing results. The 18
per-capture buffer allocations are a profiling candidate, but pooling or a
dynamic-offset ABI is not authorized until WGPU/RenderDoc evidence identifies
them as a material bottleneck. The capture light-grid plan has a dedicated CPU
profile scope, and the submission reports direct-light count, upload count, CPU
payload bytes, and GPU buffer bytes for that decision.
This capture slice deliberately reuses the viewport's current packed-light ABI
and shader evaluation. It therefore also inherits 09E's unversioned generic
`intensity`, range-only `(1-d/r)^2` punctual falloff, degraded point-like rect
light approximation, and missing object lighting-channel consumer. Restoring
the same direct-light inputs inside capture does not close the photometric,
area-light, or layer contract and does not establish Unreal lighting parity.

Capture lighting is now an explicit product policy rather than inherited
viewport preview state. Until the request contract gains an emissive-only mode,
authored ambient and direct lighting stay enabled; a non-empty direct-light list
also performs the renderer-lifetime one-way switch from `ENVIRONMENT_ONLY_PBR`
to the generic PBR variant before capture pipeline construction. The legacy
snapshot does not carry a capture shadow plan, so this MVP records unshadowed
direct lights through the existing disabled shadow fallback. Unreal retains
lighting while disabling reflection environment and screen-space reflections,
and its offline path forces surfaces diffuse with a view-owned full-roughness
override. The cross-product review found that `sky_sun_params.w` is explicitly
reserved by the current 496-byte CPU layout, is present in every matching WGSL layout,
and has no production reader. It is therefore the selected capture-policy lane:
capture uniforms set it without changing ABI size, binding, shader feature,
permutation, PSO identity, or bake recipe. Both Forward templates apply the
policy after alpha clip to base and clearcoat roughness, so built-in and custom
materials returning `ZrSurfaceOutput` share one boundary; the standalone
fallback applies the same scalar rule. The branch is uniform for the complete
draw. Full capture shadows remain follow-up work because no matching shadow
view/atlas transaction exists in the snapshot contract.

Although the PMREM/SH filter recipe is unchanged, the captured source radiance
has changed. Runtime cache identity therefore now hashes an independent
capture-raster algorithm version, `2026_08_31_0001`, before all request fields.
This invalidates old scene-capture artifacts without advancing the canonical v8
CPU/GPU filtering recipe or invalidating unrelated imported HDRI artifacts.
Changing material, lighting, visibility, sky, or exposure behavior inside the
capture raster path must advance this source version even when PMREM/SH math is
unchanged.

Capture-layer ownership is now explicit. `RenderEnvironmentCaptureRequest`
carries a separate `RenderLayerSet`, hashes its canonical layer sequence into
the renderer-owned bake key, installs it on the selected camera before the one
`O(M)` mesh-draw census, and retains it on all six capture camera descriptors.
The neutral default covers the
32 scene-schema-v1 layers; an empty set deliberately produces a sky-only
capture. Reflection-probe request JSON hard-cuts from schema v1 to v2 and adds
its own `capture_layer_mask`; the placement `layer_mask` remains receiver-only,
so capture visibility and finished-probe influence cannot silently alias.
The same set is retained in the terminal output identity. Its bake-key encoding
uses a fixed `u64` layer count and `u32` layer ids, so host pointer width cannot
change durable cache identity.

This only closes packet-local geometry narrowing. The current framework API
receives a `SceneViewportRenderPacket` after `World::build_viewport_render_packet`
has already filtered meshes and lights by the viewport camera layers and chosen
LOD from the viewport position. A later capture mask cannot recover omitted
objects, so a capture-owned extract boundary remains required before geometry
layer selection is complete. Direct-light layer filtering is also a
shared-renderer limitation:
lights carry a layer mask, but the current GPU primitive/instance ABI does not
expose an object receiver mask to lighting. Self exclusion, explicit show/hide
sets, LOD, sky/transparency/emissive policy and exposure also remain separate
view-policy work. A capture-local shader test cannot repair those cross-module
contracts.

A second cross-owner structural performance candidate was identified in the
mesh-instance path. A current prepared-model cache hit deep-clones the complete
`ModelAsset` payload before the caller immediately wraps that clone in a new
`Arc`; repeated instances can therefore repeat `O(model payload)` copying even
though the prepared owner already stores `Arc<ModelAsset>`. The required hard
cut is an `Option<Arc<ModelAsset>>` load boundary with `Arc::clone` on cache hit,
one wrap on fallback, and unchanged revision/stale semantics. No source is
changed here because the dependency set spans archived Render ownership and
active Shader attribution. Implementation requires one atomic coordinator
scope/ownership reconciliation, followed by pointer-identity/revision tests and
an ignored current-source allocation or byte-count benchmark. This is a
profile candidate, not measured elapsed-time or power evidence.

The BRDF LUT lifecycle is no longer renderer-local. `SceneRendererCore` only
projects a `SystemTextureGenerationLease`; the device-generation owner publishes
one immutable native texture, reuses it across renderers, batches system-texture
uploads, and reports payload wait/build plus upload-submission durations. The
encoded RG16F payload is cached once per process with `OnceLock`, but the first
process generation still performs the 128x32x128-sample CPU integration. A
versioned prebuilt payload remains a cold-start candidate only after the startup
report demonstrates material build cost on the target machine; moving or
embedding it without that measurement is not an accepted optimization.

The source/filter/resident-output slice and the ticket-owned probe-array copy
boundary are implementation-complete at the static-contract level. C3, M5-M8,
and the realtime-SH9 Failure remain open until the coordinator can run managed
current-source validation and validate the completion-ticket-owned copy/commit
on the target GPU.

The capture slot state machine now distinguishes existing Ready content from a
first capture or revision replacement. A pending capture without matching old
content is unavailable to viewport sampling; cancellation restores Ready only
for a same-revision reservation and otherwise leaves `Pending(0)` for a forced
reload. Duplicate reservations for an active cubemap fail closed. This closes
the stale-revision publication hole at source-contract level, but has no managed
runtime or GPU evidence yet. The reservation may overwrite the physical array
slot only through the ordered GPU copy; metadata never exposes that reservation
as Ready until completion, and a failed revision replacement remains invalid
until a fresh upload restores it.

The scheduler progress-publication failure path now also takes the retained
source submission and cancels its probe reservation before terminal failure;
this prevents an accepted graphics submission from leaving a slot permanently
`CapturePending` when control-plane progress cannot be advanced.

The deferred environment-only provider is also guarded at the capture boundary.
Its placeholder local cube array is intentionally `1x1x1`; a typed capture
target must expand that provider before reserving a slot or recording the eight
128-face PMREM copies. The placeholder invariant forbids an accepted local-
provider upload in that state, so expansion cannot discard an in-flight upload
or produce an out-of-bounds copy. The expansion makes the generic
environment-PBR ABI sticky, while this review remains static until managed WGPU
validation confirms the target dimensions on the selected backend.

The C4 persistence review preserves a second owner boundary. The existing
`IblBakeRuntimeGraphWritebackQueue` is coupled to render-graph transient
PMREM/SH9/IEM resources, an `IblBakeArtifactRequest`/descriptor, and an
`IblBakeArtifactCacheStore`; `EnvironmentCaptureGpuOutput` is a separately
retained filtered GPU object with capture identity but no artifact persistence
context. The request/descriptor and queue-owned readback reservation now exist
for the explicit capture path: PMREM/SH9 readback borrows the filtered target,
is encoded into the capture command buffer's diagnostic tail, and is committed
to the bounded completion queue only after submission. The queue writes the
runtime cache after the backend completion poll. Editor asset-derived staging
and output-URI file publication remain pending, so no broader persistence or
visual-performance claim is made.

The first identity prerequisites are now present without coupling the owners:
the neutral capture request and output identity carry an optional validated
persistence output URI and an explicit `IblBakeArtifactRequest`, while the
reflection-probe plugin forwards its existing `output_uri`. The artifact
request must come from the asset/editor owner; it is not inferred from the
runtime snapshot key. The URI is a destination locator rather than bake content
and is excluded from `ibl_bake_key`, so moving the destination cannot invalidate
a content cache entry. Admission rejects source face/mip mismatches,
non-canonical PMREM layout, empty content masks, and unsupported IEM content
before a queue can reserve GPU readback. The explicit capture path now owns the
descriptor, readback reservation, completion ticket, and runtime-cache write;
the URI remains a destination identity for the not-yet-wired editor/cook asset
staging path.

## Scope and decision

This record is a prerequisite for performance work, not a performance result.
No successful current-source Cargo build, WGPU run, RenderDoc capture,
screenshot, WPR trace, power trace, or benchmark exists. The managed Cargo
attempt described below stopped at the shared lockfile gate before compilation,
so no PMREM resolution, sample count, cache-size, dispatch shape, or material
ABI optimization is authorized by this audit.

The MVP keeps one canonical CPU asset-derived IBL recipe, one separate runtime
GPU bake cache, and one shared WGSL PBR environment core. The next work is to
obtain the declared managed five-cold/five-warm matrix, then change only the
phase proven to dominate under a fixed visual/error contract.

## Source and reference review

| Area | Current owner and finding | Reference conclusion |
| --- | --- | --- |
| PMREM roughness LOD | `IblBakeRecipe` and `zr_environment_core.wgsl` use a last-index `maxMip` and preserve Unreal's `maxMip - 1 - LevelFrom1x1`, where `LevelFrom1x1 = 1 - 1.2 * log2(roughness)`. | Unreal `SceneRendering.cpp` sets `ReflectionCubemapMaxMip = FloorLog2(capture_size)`: 128 has eight mips but passes max index 7. |
| PMREM filtered-importance caller | CPU and GPU retain the same `E.y *= 0.995`, `D/4` light-direction PDF, texel solid-angle `* 2`, and roughness-above-0.99 cosine branch. Their centered `E.x = (i + 0.5) / N` sequence is Zircon's CPU/GPU parity choice, not an exact Unreal sequence claim. | Unreal `ReflectionEnvironmentShaders.usf::FilterCS` applies these four rules around `ImportanceSampleGGX`; inspecting only the generic helper incorrectly suggests the `0.995` factor is absent. No recipe change is justified. |
| PMREM low-roughness PDF | CPU/GPU `distribution_ggx` floored the complete `PI*d²` denominator, flattening the first filtered mip's valid D peak and selecting unnecessarily coarse source LOD. Both now use a positive, cancellation-resistant equivalent denominator and recipe v8. | Unreal `D_GGX` divides by `PI*d*d` without this floor. The repair must invalidate persisted PMREM while retaining the verified FilterCS sampling rules above. |
| Split-sum and energy | `zr_environment_pbr_components_from_reflection` obtains material F0 and applies BRDF LUT plus specular occlusion to reflections. The 2026-08-28 whole-path review supersedes the earlier source-independent experiment: direct GGX returns its already-computed Fresnel for the Khronos diffuse complement, while ambient/environment/lightmap use the shared view-Fresnel approximation and transmission consumes the same base-layer budget. | Khronos Appendix B and its material extensions are normative for glTF lobe composition. Unreal remains the engine-ownership/performance reference; its optional directional-albedo model is a future measured quality tier, not a reason to mix incompatible energy owners. |
| Standard-PBR base-color domain | Texture, tint, and vertex color remain raw material inputs, but Forward/basic, environment-only, fallback, deferred, lightmap, environment-core, and transmission consumers now route Standard-PBR diffuse reflectance through `zr_pbr_base_color`. Unlit, Blinn-Phong, and custom-model base color remain unchanged. | Unreal `MaterialTemplate.ush::GetMaterialBaseColor` applies `saturate` before physical diffuse/specular decomposition. The clamp belongs at the PBR consumption boundary, not at shared sampling or GBuffer storage. |
| Direct GGX visibility | Forward/basic, fallback, and deferred consume one self-contained isotropic direct-light owner using the same joint-Smith approximation as the BRDF LUT. Advanced Forward separately consumes the Khronos axis mapping plus Burley NDF and directional Heitz/Unreal anisotropic visibility; environment-only deferred excludes direct-light owners. | Unreal default isotropic `SpecularGGX` uses `Vis_SmithJointApprox`, while its anisotropic overload uses `D_GGXaniso` and `Vis_SmithJointAniso`. The two models have distinct owners and must not be collapsed through geometric-mean roughness. |
| Direct-light attenuation boundary | `GpuLightData` receives unitless authoring intensity, while forward/basic, fallback, deferred, and froxel sources retain copies of bounded `(1 - distance / range)^2` attenuation. Rect-light width/height are packed but do not enter surface area integration. | Runtime95 already owns photometric units, shared attenuation/shape evaluation, inverse-square cutoff, and rect-area integration through `09E-P0-5/6`, `RDL-P1-009..012`, and `RDL-G07..G10`. Shader06 must not perform a local formula or ABI migration. |
| Path convergence | Standard, basic, fallback, and environment-only Forward preserve `surface.dielectric_f0`; deferred environment-only explicitly supplies `0.04` at its GBuffer-limited boundary. All paths consume the common PBR/environment functions. | No duplicate BRDF/LOD rule or hidden profile-wide F0 constant should be added to a fallback or generated source. |
| IOR routing | Current source still uses non-default dielectric F0 as draw-list routing state and suppresses `ENVIRONMENT_ONLY_PBR`, so the material takes the correct generic Forward path without adding PSO identity. The specialized provider can now consume authored F0, making that exclusion a startup/source-volume debt rather than a correctness requirement. | Preserve the generic path until exact current-source profiling and owner-safe migration; do not continue describing fixed F0 as a reason to exclude the specialized path. |
| Variant dimension identity | `GeometrySourceId` exposes a full `u8` plugin range. The old in-memory packed key allocated only four geometry bits and collided with shading IDs above geometry 15; it now assigns complete 8-bit geometry and shading segments and keeps persistence on the canonical string. | Unreal keeps ShaderMap identity in explicit deterministic fields; Bevy's `u64` mesh keys reserve masks/shifts and assert non-overlap. A compact accelerator must not truncate the public ID domain. |
| Shader compile-path attribution | Mesh source construction, recursive module-include resolution, three material template domains, WGSL/content hashing, Naga validation, disk lookup/write, and total material requirement admission previously had no independent CPU scopes; a cold/warm profile could not prove which phase dominated. They now publish eight static profile stages plus assembled source bytes, segment/include counts, include source bytes, and disk meta/compressed/decoded/write bytes. The standalone PBR viewer now starts and exports the existing environment-controlled Zircon profiler, while each measured runner process owns an E-drive output root, immutable session id, bounded 10k-scale capacity, and native timeline/hotspot/counter fingerprints; any overwritten span/counter makes the run invalid. Normal builds evaluate none of the added counter expressions because the profiling macros are feature-gated. | Unreal separates preprocess/compiler job, shader-map/DDC, backend compile, and PSO cache timing rather than treating first draw as one opaque duration. Zircon needs the same attribution before changing template, cache, worker, or PSO policy; these scopes are measurement infrastructure, not proof that any stage is slow. |
| Mesh PSO submission lifetime | Eight Ready-side pipeline lookup sites now record nine exact Mesh pass targets, including the two TAA and two shadow kinds, or the independent OIT target immediately before binding. Direct and compiled scene submissions attach the deduplicated use set to their returned device-qualified ticket; terminal maintenance scans only unresolved scene tickets and never walks the full variant registry. The compiled command cache incrementally counts cross-frame variant pins, and every PSO installation publishes a reverse shader-module edge with shared-key reference counts. No general PSO retirement transaction, registry tombstone, or ID reuse exists yet. | Unreal's `FPipelineStateCache::DiscardAndSwap` first consolidates cache ownership, verifies no use, and waits incomplete compile tasks before deleting retired entries; material ShaderMaps remain explicitly reference-counted across game/render/compile owners, while `FRHIResource::MarkForDelete` separately queues RHI resource destruction. Zircon must likewise separate logical cache eligibility, CPU ownership, submitted GPU use, shader-module ownership, and native destruction rather than using an N-frame guess. |
| Skeletal tangent frame | The CPU fallback, the standard/morphed WGSL paths, and the fallback mesh shader all apply the normalized LBS blend to position, normal, and tangent, then normalize the directions. | Unreal `GpuSkinVertexFactory.ush::SkinTangents` likewise applies its `BlendMatrix` to tangent and normal before normalization. This is the conventional LBS approximation; it is not a reason to introduce an independently blended per-joint inverse-transpose palette. |
| Normal-map TBN and convention | Authored/generated glTF tangents are right-handed after the shared Mikk `w` correction, and glTF raw normal texels are currently sampled without a green flip. However, Standard/fallback Gram-Schmidt and normalize each interpolated TBN axis, while standalone normal-convention metadata calls the no-flip path DX and flips explicit GL input. | `bevy_mikktspace` requires the shader to use the matching unnormalized interpolated T/B/N and normalize only the final mapped normal; Bevy uses the same Mikk `w` correction and treats no-flip as right-handed while DX input flips Y. Runtime canonical must be right-handed/GL, with DX converted once at cook time; repair both render paths and version importer outputs before visual acceptance. |
| Compiled-graph material admission | Executor feature collection was gated only by active staged candidates. Publication clears that set before the next viewport performs context admission, so the next viewport built an empty feature set and incorrectly treated zero PSO requirements as Ready. The gate now remains active for either staged candidates or published generations requiring context admission. | Publication state and consumer capability discovery are separate axes. Clearing the candidate queue must not erase the render-graph pass set needed to validate the published generation in another context. |
| Published context-admission lifetime | `context_admission_material_ids` is inserted on publication and is not cleared after a successful context check. This conservatively protects future viewport, geometry, quality, fog, and graph-executor combinations, but it can keep the current-generation requirement census active and retain `previous_published` after the original transition. Removing the set is not authorized without an exact context-generation replacement. The profile now reports tracked material count, scanned pending-draw count, actually referenced candidate count, requirement count, ready/deferred/failed counts, and previous/error selection under the existing CPU scope. | Unreal keys cached mesh draw commands and material shader maps by the complete pass/vertex-factory/material context and invalidates them by generation. Zircon needs equivalent exact context identity or a proven frame/viewport lifecycle before it can retire the previous generation without risking a new-context ABI mismatch. |
| Material parent invalidation | Project material import omitted the parent locator, and a prepared bundle compared only the requested child revision, shader closure, and texture revisions. A parent-only scalar or feature change could therefore leave child uniforms and pipeline routing permanently stale. Import now publishes the parent edge, while each bundle snapshots the actual loaded material root plus the resource graph's recursive dependency revision. | Unreal explicitly recaches every material instance whose parent chain reaches a changed material. Zircon should use its maintained reverse dependency closure for the same invalidation semantics, not walk all ancestors or descendants on every stable-frame cache lookup. |
| Effective material projection and capture | Parent-chain evaluation had separate implementations in material preparation and advanced-PBR frame extraction, while Hybrid GI capture read a staged runtime or an unresolved raw child asset. `ProjectAssetManager` now owns one bounded effective-material loader; preparation, frame feature extraction, and cold capture share it, while live capture reads only the published draw proxy and fails closed for staged/rejected-only state. | Unreal Lumen card capture consumes `FMaterialRenderProxy`, follows its fallback chain, and uses the default material when the required material shader/PSO is unavailable. Capture must not bypass atomic publication to inspect authoring state. |
| Hybrid GI texture generation | Material capture previously paired the published scalar/binding generation with the latest `TextureAsset` and cached samples only by `ResourceId`; a hot reload could therefore expose new texels before material publication and overwrite the old generation's cache row. Each prepared texture now captures one fixed center sample from the same asset revision as its GPU upload, material bundles retain that sample and revision, and Hybrid GI keys samples by `(ResourceId, revision)`. | Unreal's material render proxy boundary implies that texture bindings are part of the selected material generation. Zircon's temporary center-sample capture must preserve that generation even though it is not yet a full UV-space card capture. |
| Asset/revision publication snapshot | Texture, shader, and root-material rebuilds previously read the registry revision separately from the payload, so a hot reload between those reads could label one payload with another generation. `ProjectAssetManager` now exposes one typed `ResourceSnapshot` owner backed by the existing single-lock `ResourceManager::snapshot`; GPU preparation and cold generation-bound capture consume it, and material publication rechecks the complete O(1) bundle identity after PSO admission. | Unreal material/shader resource publication selects immutable render resources by generation and does not reconstruct an identity from independently sampled mutable state. Zircon must preserve the same invariant at its resource-manager boundary before optimizing queues or shader work. |
| Failed-material retry admission | A deterministic post-resolution material validation failure retained last-good output but recorded only the requested root revision. The unchanged candidate therefore repeated parent resolution, readiness, shader preparation, texture dependency snapshots, and CPU payload projection every frame. The rejection now retains the same lightweight root/parent, shader-closure, texture-revision, and texture-capability identity used by prepared bundles; typed pipeline failure keeps its existing exact staged-bundle identity. Exact validation/pipeline failures are cache hits and dependency or capability changes retry automatically. Residency, I/O, queue, RHI, receipt, device-validation, and channel failures remain uncached and retryable because they can recover without an asset revision change. | Unreal associates material compile failures and fallback selection with an immutable material/shader-map identity while retaining a renderable proxy. Zircon must likewise distinguish an unchanged terminal input from transient execution failure; a blanket time backoff would delay valid hot-reload recovery and a blanket permanent cache would freeze streaming. |
| Offline filter | cmft provides an independent radiance/irradiance filter with configurable Phong/Blinn power mapping and optional legacy edge warp. | cmft is useful for offline content and job decomposition, but its Phong/Blinn radiance model is not a replacement for Zircon's GGX split-sum runtime contract. |
| Cubemap seams | WGPU cube sampling has seamless face filtering; `zr_environment_fix_cube_lookup_for_face_size` is intentionally a no-op. | cmft documents warp only for APIs without seamless cube-map filtering. Applying it here would distort valid WGPU samples. |
| Background work | cmftStudio runs cmft filtering on a background thread, while its UI observes mutable `ThreadStatus` and shared output state. Zircon has CPU importer ownership and a bounded runtime GPU tier. | Use the reference only for separating UI work from long filters. It supplies neither immutable publication nor cancellation semantics for Zircon's runtime GPU bake, and is not a reason to move canonical asset cooking into the render thread. |

### PMREM mip-coordinate disposition

The initial review of the scene uniform found an apparent off-by-one because
it stores a mip count while WGSL passes `mip_count - 1` to the helper. The
Unreal C++ assignment resolves the ambiguity: it independently passes
`FloorLog2(capture_size)`, a last index, to the same HLSL formula. Therefore
the relevant coordinate system is:

```text
max_mip = mip_count - 1
lod = max_mip - 1 - LevelFrom1x1
```

At eight mips, `max_mip = 7`, `roughness = 1`, and `LevelFrom1x1 = 1`, so
the correct selected PMREM mip is 5. The CPU recipe inverse and GPU bake
parameters deliberately saturate mips 5, 6, and 7 to full roughness. The
existing v7 roughness-LOD repair is therefore retained. The eight-mip anchors
are `r=1 -> mip 5`, `mip 4 < 1`, `mip 5 = 1`; v8 below is caused by an
independent PMREM PDF correction, not a second LOD-map change.

The filtered-importance follow-up also reviewed the complete Unreal call site,
not only `MonteCarlo.ush::ImportanceSampleGGX`. The generic helper consumes
`E.y` directly, but `ReflectionEnvironmentShaders.usf::FilterCS` first applies
`E.y *= 0.995`; the same caller uses `SolidAngleTexel * 2`, the `D/4` reduction,
and cosine sampling above roughness 0.99. Zircon's CPU and GPU paths preserve
those four rules. The earlier candidate to remove the scale was rejected before
acceptance and fully rolled back. No recipe change is justified by that
candidate. Zircon centers
the Hammersley azimuth stratum with `(i + 0.5) / N`, unlike Unreal's `i / N`;
because both Zircon bake paths share it and an isotropic GGX distribution is
azimuth-rotation invariant in expectation, this audit records the finite-sample
sequence difference without changing it absent current-source image/error data.

The same caller review exposed the real low-roughness defect in
`distribution_ggx`. Its complete `PI*d²` denominator was floored at `1e-6`.
For the canonical 128x8 layout, mip1 roughness is `0.0992126`; at `NoH=1`,
the valid Unreal D value is `3285.3633`, while the floor returned `96.8873`.
Twenty-seven of the mip's 32 samples are affected. With a 256 source face, the
old PDF selected source LOD an average `1.3430` mips too coarse, with a `2.1676`
mip center-sample maximum. CPU and GPU now share the positive stable form
`d=(1-NoH²)+NoH²*a²` and divide directly by `PI*d²`. Persisted output changes,
so canonical recipe and viewer current evidence advance to `2026_08_26_0008`.
These are formula/PDF-LOD measurements, not framebuffer, timing, or power data.

### Direct-light visibility disposition

The base direct GGX lobe formerly used exact joint-Smith visibility while the
corrected split-sum LUT and Unreal default isotropic `SpecularGGX` use
`Vis_SmithJointApprox`. A 256-cubed `(roughness, NoV, NoL)` formula scan found
7.322394% mean and 29.151215% maximum relative visibility difference. At
`roughness = 0.5` and `NoV = NoL = 0.5`, the exact and approximate values are
0.917663 and 0.800000; at `NoV = NoL = 0.1`, they are 9.325048 and 7.692308.
These are formula-level visibility values, not framebuffer error or timing.

Forward/basic, fallback, and deferred now consume the single self-contained
`zr_pbr_extras_core.wgsl` owner. The fallback and deferred raw shaders no longer
declare GGX term structs, visibility functions, or isotropic integrators; the
environment-only deferred profile does not register the direct-light module.
The initial base repair deliberately left the advanced exact scalar function
unchanged until the anisotropic model could be reviewed as a whole. The later
2026-08-29 review below found that geometric-mean visibility and both axis
roughness values were structurally wrong, then replaced them atomically with
Khronos axis mapping, the Burley NDF, and tangent/view/light directional
visibility equivalent to Unreal's `Vis_SmithJointAniso`.

The same review found a separate low-roughness defect. Zircon's positive
`alpha >= 0.001` contract already guarantees a nonzero GGX distribution
denominator for bounded normalized dot products, but the old implementation
also floored the complete `PI * d * d` denominator at `1e-6`. That floor starts
altering the distribution below roughness 0.154119. At the viewer IOR fixture's
`roughness = 0.08` and `NoH = 1`, Unreal's `D_GGX` gives 7771.237456 while the
floor produced 40.96, retaining only 0.5271% of the peak. The shared owner now
clamps `NoV`, `NoL`, `NoH`, and `VoH` to their normalized domains and evaluates
the positive denominator directly, matching Unreal's formula structure.

The approximation statically removes two square roots from the base lobe and
the owner convergence deletes two shader-source copies, but no current-source
GPU capture or timestamp has run. These facts are not a performance or power
claim. The managed direct-light image matrix and GPU profile remain the gate.

### Standard-PBR base-color domain disposition

The shared F0 repair alone did not close the material input domain. Standard
material sampling deliberately multiplies texture, draw tint, and vertex color
without a global clamp because Unlit, Blinn-Phong, and custom shading models
share that surface/GBuffer data. Before this follow-up, metallic F0 was bounded
by `zr_pbr_material_f0`, but Standard-PBR direct diffuse, ambient diffuse,
environment diffuse, baked lightmaps, and diffuse transmission still consumed
the raw value. One material could therefore use a physical specular color and
an impossible negative or greater-than-one diffuse reflectance at the same time.

`zr_pbr_common.wgsl` now owns `zr_pbr_base_color`, matching Unreal
`GetMaterialBaseColor` saturation. Forward/basic and fallback derive the value
once before the light-grid loop; generic deferred selects it only for the
Standard-PBR model; the Standard-PBR-only Forward/deferred profiles use it
directly; lightmap templates select it only for Standard PBR; and the shared
environment composition defensively normalizes its PBR API inputs. The direct
transmission lobe receives the already-normalized per-pixel `diffuse_color`, so
it does not add a clamp per light. Sampling, GBuffer storage, Unlit output,
Blinn-Phong output, and custom shading-model data remain raw by design.

At normal incidence with `metallic=0`, `F0=0.04`, and authored RGB
`[2.0, -0.5, 0.5]`, the pre-irradiance diffuse-energy product was
`[1.92, -0.48, 0.48]`; the physical-domain result is
`[0.96, 0.0, 0.48]`. This is a formula anchor, not framebuffer evidence. The
repair adds no binding, material/GBuffer ABI field, feature bit, permutation,
PSO identity, or bake-recipe change. Static red/green source contracts cover
the shared owner and all built-in render paths. No GPU timing or power sample
exists, so neither compiler clamp elimination nor a performance result is
claimed; the current image and RenderDoc gates remain mandatory.

### Diffuse energy decomposition disposition (current, reaffirmed 2026-08-31)

This is the current Shader06 MVP decision. The previous heading and introduction
incorrectly described the source-independent Unreal Default Lit decomposition as
superseded while the implementation record below selected and required it. The
2026-08-31 full Forward Basic/Advanced, Deferred, fallback, environment,
lightmap, transmission, clearcoat, Unreal, and Bevy review resolves that record
conflict in favor of this section. Khronos transmission/clearcoat Fresnel
complements remain in their layered lobes; they do not authorize a per-light
`1-F(VoH)` multiplier on the opaque base diffuse term.

The complete direct/ambient/environment/lightmap callsite review found a
structural error in the earlier shared remaining-energy helper. Direct light
used `(1-F(VoH))*(1-metallic)`, environment used
`(1-F(NoV))*(1-metallic)`, and baked lightmaps used
`(1-F0)*(1-metallic)`. The same material therefore acquired a different
diffuse albedo according to light source; at grazing `VoH` the direct diffuse
could collapse to zero. This is neither Unreal's default
`DiffuseColor = BaseColor - BaseColor * Metallic` decomposition nor Unreal's
optional directional-albedo LUT/analytic energy-preservation model. Bevy also
forms a source-independent metallic diffuse color before direct, irradiance,
and lightmap evaluation.

The Shader06 MVP therefore hard-moves the unique
`zr_surface_metallic_diffuse_energy_scale` owner from generic surface types to
`zr_pbr_common.wgsl`, with no legacy alias, for the same
`1-clamp(metallic, 0, 1)` diffuse energy scale in direct, ambient, environment,
and baked lightmaps. The duplicate `zr_pbr_diffuse_energy_scale` introduced by
the rejected design is removed. F0 remains
the common specular input; non-default IOR routing is unchanged. Unreal-style
directional-albedo/multiple-scattering compensation is deferred until a
current-source GPU profile can justify its texture sample or analytic cost.
For `F0=0.04`, the old scale was `0.96` for baked light, `0.93` at
`NoV=0.5`, approximately `0.39313` at `NoV=0.1`, and zero at `VoH=0`;
the MVP scale is `1.0` for all sources. These are formula anchors only, not
framebuffer, timing, or power evidence. The implementation adds no binding,
material/GBuffer ABI field, feature bit, shader permutation, PSO identity, or
bake-recipe change.

The source implementation now routes all 15 production consumption points
through that scalar owner: forward/basic ambient and prepared direct BRDF,
environment-only ambient, environment-core IBL, both lightmap templates, and
the standalone fallback, deferred-lighting, and deferred-environment products.
The rejected duplicate helper is absent. Direct lighting prepares
`direct_diffuse_brdf` once at the light-grid boundary, so N visited lights no
longer express N metallic clamps/subtractions, RGB scales, and `/PI`
operations; the source count is one, a reduction of `N-1`. The GGX functions
again return only specular, deleting the now-unconsumed Fresnel result struct
and its two wrapper layers. These are source-structure counts; compiler
instruction selection, GPU time, and power remain unmeasured.

The assembled Forward contract also locks the scalar owner itself: exactly one
declaration, the exact `1-clamp(metallic, 0, 1)` expression, and no Fresnel
input. This `3/3` static regression prevents the rejected source-dependent
model from returning through a helper-body change; it is not WGSL compilation
or runtime evidence.

### Cross-plan direct-light attenuation boundary

The same source review confirmed a larger direct-lighting contract problem but
did not create a second owner for it. `GpuLightData` copies the public unitless
intensity into the GPU packet, the light types use materially different default
scales, and forward/basic, fallback, deferred, and froxel shading retain bounded
distance-fade copies. Rect-light width and height reach the packet but surface
lighting still evaluates the source as a one-sided point approximation.

This is already open in the Runtime95 optimization plan as `09E-P0-5/6` and
`RDL-P1-009..012`, with acceptance gates `RDL-G07..G10`. Its required migration
is structural: versioned source photometric units, one effective radiometric GPU
contract, a Runtime91-owned shared attenuation/shape module, inverse-square plus
smooth range cutoff and near-field source-radius handling, rect-area reference
fixtures, and forward/deferred/froxel parity. It also requires a current-source
same-scene GPU timing and power comparison before any optimization claim.

Shader06 therefore makes no local `1 / distance^2`, LTC, default-intensity, or
`GpuLightData` ABI change. Applying only one of those changes would shift project
brightness while leaving the other render paths inconsistent. This audit records
the dependency and returns to the owned material/environment MVP closure; the
managed Shader06 evidence queue is not used as a reason to start that migration.

### Non-default dielectric F0 deferred disposition

The fixed deferred GBuffer stores metallic, roughness, occlusion, and packed
shading-model/shadow flags. It deliberately has no dielectric-F0 field, so its
deferred lighting fallback uses the standard `0.04` value. This is a fixed ABI
constraint, not evidence that authored IOR is dropped: material preparation
sets `PipelineKey::pbr_ior_override` when the derived dielectric F0 differs
from the standard value, and that routing-only bit makes opaque draws use the
late Forward `advanced_pbr_opaque` list instead of `Opaque3d`/GBuffer.

The Forward standard-material template reads the derived F0 from
`data12.yzw`, and the fallback Forward shader supplies it to both direct
Standard-PBR lighting and `zr_environment_pbr_indirect_with_dielectric_f0_normalized`.
The routing bit is removed from PSO identity and feature bits, so this repair
does not multiply shader variants or pipeline states. A source regression now
asserts that an opaque static material with only `pbr_ior_override` bypasses
the opaque cache/GBuffer list, appears once in `advanced_pbr_opaque`, and is
recorded dynamically. It has not been executed under the managed-validation
policy.

The existing viewer Ready-frame gate is valid only for its default metal
mirror: it waits for the specialized `ENVIRONMENT_ONLY_PBR` Base prewarm.
Current IOR routing still excludes that feature, so the fixture must continue
waiting for its exact generic Forward `PipelineKey`; it must never reuse
`environment_only_base_pipeline_ready` as proof that the IOR material was
drawn. This is the current evidence contract, not the target architecture.

The specialized provider now accepts authored dielectric F0, so the original
reason for `MeshPipelineVariantKey::new` to suppress `ENVIRONMENT_ONLY_PBR`
when `pbr_ior_override` is set no longer exists. The warmup owner likewise
enables the specialized profile only for default IOR. Removing those two
conditions could eliminate the IOR fixture's otherwise unreachable generic
direct-light, receiver, lightmap, shadow, and volumetric source closure, but no
fresh assembled-byte count, Naga/module/PSO time, first-present GPU time, RSS,
or power result exists. The affected registry/warmup sources also contain
other unowned changes. The migration is therefore frozen as a measured
startup-routing follow-up: compare exact generic and specialized source bytes,
Naga/module/PSO CPU, first present, GPU timestamp, RSS/WPR, and energy under the
same IOR image oracle, then update the fixture Ready schema and both route
conditions atomically. Until then the generic route remains correct but
redundant.

## Current cost model

The source-selected diffuse IEM path is bounded by
`6 * 32^2 * 6 * 32^2 = 37,748,736` source/output direction candidates per
bake before the positive-hemisphere early rejection. This establishes a
candidate-count bound only. It does not establish wall time, CPU utilization,
memory traffic, energy, or a bottleneck.

The viewer's checked-in 2K HDRI resolves to source face size 512 and, without
an explicit override, requests a `512x10` PMREM. The `128x8` recipe is a
separate fixed-layout baseline and must not be made the viewer default unless
the managed profiles show a material reduction while the visual matrix remains
within its declared error tolerance.

The 2026-08-25 skin-weight correctness repair adds one reciprocal and four
scalar weight multiplications only on already-skinned Standard-PBR vertices.
It changes no binding, feature bit, pipeline identity, or instance ABI. This is
not sufficient to infer vertex cost or power consumption; collect GPU timestamps
and a skinned scene alongside the PBR viewer before considering a skinning
algorithm change.

Non-default dielectric F0 has a different cost topology. It shares the normal
Forward PSO identity and does not add a shader feature bit, but its opaque draw
cannot use the static opaque command-cache path and is recorded in the late
Forward list. That trade preserves material correctness without changing the
fixed GBuffer ABI. Its cost is unknown until a managed scene reports the count
of `advanced_pbr_opaque` commands, command-cache hits/rebuilds, CPU queue time,
and the late Forward GPU timestamp with a defined population of overridden-IOR
materials. Do not add an F0 GBuffer field, introduce a separate IOR PSO, or
fold these draws back into deferred lighting on the basis of source topology.

For non-uniform joint scale, a per-joint weighted inverse-transpose is not an
exact repair for LBS. The deformation is `D(p) = sum_i w_i(p) * M_i * p`; its
surface differential contains both the blended matrix and the weight-gradient
terms. Zircon has no weight-gradient or deformed-neighbourhood representation,
so neither `inverse_transpose(sum_i w_i * M_i)` nor
`sum_i w_i * inverse_transpose(M_i)` can be presented as the exact normal of
that deformation. The existing CPU/GPU behavior is the conventional
blend-matrix approximation also used by the Unreal reference. A future change
therefore needs an explicit skeletal-deformation/tangent-frame contract, not a
local PBR fix that adds inverse operations, palette bandwidth, and a second
normal policy without a measured visual error.

The shader-template boundary is a separate structural candidate. Nine regular
templates invoke `fetch_position`, `fetch_normal`, and `fetch_tangent` once
each; the two velocity templates additionally invoke `fetch_prev_position`.
For four influences, the source-level upper bound is therefore three current
blend-matrix constructions (twelve joint-matrix fetches) per regular vertex,
and three current plus one previous construction (sixteen current/previous
joint-matrix fetches) in a velocity vertex. The fallback shader has the same
three explicit current-palette loops. Backend inlining/common-subexpression
elimination may reduce that work, so these are source bounds, not measured GPU
costs. Only a captured current-source shader plus pipeline statistics and
timestamps can decide whether a shared `SkinnedVertexFrame` fetch contract is
worth the cross-template migration.

Hybrid GI material capture previously performed up to five
`ProjectAssetManager::load_texture_asset` calls for each unique captured
material while building a frame cache, then stored those results under five
bare resource IDs. The published path now performs zero asset reads: one RGBA
center sample is derived once when a texture revision is prepared, copied into
the immutable material bundle, and projected into the frame cache with the
pair `(ResourceId, revision)`. A same-revision mip rebuild copies the existing
sample and performs no additional sampling. Thus the source-level stable-frame
asset-load count changes from at most `5 * M` to `0` for `M` unique published
materials, while preparation adds one fixed center-texel lookup per texture
revision. This is an algorithmic call-count result, not wall-time, RSS, GPU, or
power evidence; the cold compatibility path may still load assets before a
streamer material state exists.

The asset/revision snapshot repair does not add payload cloning to stable cache
hits: texture, output-target, post-process LUT, shader, and material fast paths
still compare one requested registry revision before returning. Rebuilds retain
the previous one asset clone but obtain its revision from the same
`ResourceSnapshot`. Mip rebuilding removes the separate registry-revision read,
changing the revision/payload identity reads from two to one snapshot and the
total authority reads including the unchanged residency probe from three to two
per rebuild attempt. Material publication adds one O(1) complete-bundle
identity check after PSO admission; stale attempts increment
`material_candidate_publication_stale` and remain non-visible for the next
preparation pass. These are lock/call-count bounds only. No CPU duration, GPU
duration, RSS, energy, or power improvement is claimed without the managed
profile matrix.

An unchanged deterministic post-resolution validation failure now performs the
same complete O(1)-per-record identity probe as a prepared bundle and returns
the published last-good generation. Typed pipeline failures already retain the
exact staged bundle and use the same probe. For `F` stable frames, a parent
chain of `P` materials, a shader closure of `S` resources, and `T` texture
dependencies, the validation-failure path changes from repeated
`O(F * (P + S + T))` preparation plus possible upload attempts to one failed
preparation followed by `O(F * (1 + T))` indexed identity checks. Both
maintained material-parent and shader-closure generations are O(1), while the
exact texture rows remain O(T). One rejection-time clone of the compact shader and
texture identity rows is retained. `material_candidate_terminal_cache_hit`,
`material_prepare_cache_hit`, and `material_prepare_rebuild` expose the result.
Residency/I/O/queue/RHI/receipt/device-validation/channel failures deliberately
retain the old retry behavior and therefore are not included in this bound.
`GraphicsError::Asset` is not treated as inherently terminal: `ensure_resident`
uses that envelope for Pending/Reloading and project-generation supersession,
which can recover while a leaf texture revision remains unchanged. These are
call-count and asymptotic statements, not CPU-time, GPU-time, memory, power, or
visual measurements.

Published context admission remains a measurement candidate rather than an
optimization. Once at least one material is tracked, the current source can
scan all `D` pending draws to build an exact requirement census even when only
`M <= D` materials match the tracked set. The added
`material_context_admission_tracked_material_count`,
`material_context_admission_scanned_draw_count`, and existing candidate and
requirement counters quantify that topology beside the
`render/material/context_admission` CPU duration. A replacement must prove
that it still detects every new viewport/geometry/quality/fog/executor
combination before current bindings become visible; no set clearing, context
hash, previous-generation retirement, or complexity improvement is claimed in
this slice.

Shader/PSO profile attribution is now structurally available without changing
the compiler or cache algorithm. The nested CPU stages are
`material_requirement_admission`, `mesh_source_build`,
`module_include_resolution`, `template_assembly`, `source_hash`,
`naga_validation`, `disk_cache_lookup`, and `disk_cache_write`. Source,
segment, include, and disk byte counters provide the denominator for the
declared 1/100/10k variant matrix. WGPU module creation, PSO creation, queue
wait, and the existing synchronous error-scope pop remain separately required;
the current instrumentation must not be used to infer them by subtraction
across asynchronous workers. No queue, cache, template, validation, or pipeline
policy is changed in this slice.

## Measurement gate

The coordinator-managed run must use only E: output paths and publish immutable
completion evidence. For each cold and warm process run, record:

1. Ready-sidecar phase durations for HDR decode, equirectangular projection,
   source-mip construction, PMREM, and optional IEM, with their derived total.
2. CPU attribution from WPR or an equivalent managed collector, including worker
   utilization and queue wait, rather than treating dispatch counts as time.
3. GPU timestamps for environment passes where the adapter supports them,
   plus cache byte counts and runtime bake readiness/stale age.
4. The PBR image matrix and RenderDoc replay from the current managed build,
   including a non-uniform instance and non-normalized skin-weight fixture.
   The current environment-only viewer's default metal mirror is not an IOR
   fixture: a separate generic-Forward, non-default dielectric-F0 scene must
   prove the routed material's direct and environment reflection before IOR
   coverage can be claimed.
5. Energy only when the collector names its counter, unit, interval, and scope.
   CPU sample duration and GPU timestamps are not power measurements.

### IOR routing observation contract

The renderer now publishes the already-computed
`PreparedMeshQueueStats::{opaque_command_count, advanced_pbr_opaque_command_count}`
as matching `RenderStats` fields and diagnostic series
`render.mesh.queue.{opaque_command_count, advanced_pbr_opaque_command_count}`.
These are direct command counts, rather than shader-variant estimates, and are
emitted alongside the existing dynamic-command and cache-hit/rebuild counters.
A non-default-IOR managed fixture must record all five values for the same frame
population. For a single static overridden-IOR sphere, the expected routing
proof is one advanced-PBR opaque command, zero ordinary opaque commands for
that sphere, and a dynamic/rebuilt command on the first frame; cache behavior
after that remains an observation, not an assumed optimization target.

This observation has intentionally no per-draw hot-path work: the command list
already computes the count, frame extraction copies the scalar once, and the
diagnostic store publishes it once. It proves neither CPU duration, GPU duration,
power, nor image correctness. Those remain the WPR/GPU-timestamp/RenderDoc and
current-source screenshot gates above.

The viewer GPU-timing sidecar now has schema
`zircon_shader_pbr_viewer_gpu_timing_evidence_v3`. The renderer-owned timestamp
report starts without mesh data; `WgpuRenderFramework` attaches a clone of
`RenderFrameProfile::mesh_submission` only when its resolved profile has the
same `frame_generation`. The viewer rejects a missing snapshot, and requires
the five values (`opaque_command_count`, `advanced_pbr_opaque_command_count`,
`cached_command_hit_count`, `command_rebuild_count`, and
`dynamic_command_count`) to remain exactly equal across all 31 retained samples.
The standalone sidecar validator and startup-profile summarizer enforce the
same exact per-sample field set. This prevents asynchronous timestamp resolution
from accidentally reporting counters for a newer frame; it establishes neither
the measured values nor a timing, power, or visual result.

`ViewerMaterialFixture::DielectricIor` supplies deterministic material source
data (`ior = 2.0`, metallic `0`, roughness `0.08`) and a distinct project-asset
identity prefix, while the default `MetalMirror` preserves the existing
`viewer-project-v4` identity. The runtime now owns
`pbr_ior_forward_base_pipeline_ready`: its queued key is the viewer's static
fallback material with `receive_shadows = false` and `pbr_ior_override = true`.
The registry removes the routing bit from PSO identity but still uses it to
suppress `ENVIRONMENT_ONLY_PBR`, so the retained variant uses the generic
Forward receiver layout. That suppression is retained current-source behavior,
not a post-F0 target requirement. The viewer exposes this fixture through the closed
`--material-fixture <metal-mirror|dielectric-ior>` CLI mode and calls the exact
gate from its protected scene owner. The dielectric fixture has an isolated
project identity and waits only for its generic-Forward IOR Base PSO; it never
uses the mirror prewarm as a readiness proxy. This source closure still does
not establish an IOR image, RenderDoc capture, or timing result.

The profile publication lease keeps partial runs outside the immutable artifact
closure, holds an exclusive owner lock for the complete capture, and permits a
subsequent scavenger to recover a valid completion receipt instead of
quarantining it. The runner now scavenges stale released leases before it
allocates a staging root, acquires the matching lease before it writes the
manifest, heartbeats around every cache-mode and RenderDoc stage, commits only
after summary validation and immutable receipt publication, records a failed
terminal state when no receipt exists, and always releases its lock. This is
source/static evidence only; managed Pester/profile execution remains required.

Profile provenance PBR-P1-04 is implemented at source level. The profile
contract now replaces its former six-file viewer subset with the recursive
eighteen-module production closure rooted at `main.rs`, while retaining 74
explicit cross-module owners, for a 92-file critical set. The original nine
additional owners bind the non-default-IOR F0 derivation, routing,
prepared-queue bridge, and public observation path; five more bind the
frame-profile snapshot, its stats projection, renderer report, framework
generation match, and timestamp producer; the current addition binds the
realtime IBL CPU timing owner beside its execution-resource cache. The closure honors
inline or standalone Rust attributes including `#[path]`, excludes `cfg(test)`
modules, accepts declaration-tail comments, rejects unresolved or escaping
paths, and emits stable `relative_path`, SHA-256, and byte-length records. The
runner now passes its repository root to this owner, so the profile manifest,
build provenance, coordinator ticket, and artifact receipt bind the same
expanded set. The integration test asserts the viewer subset equals the
recursive closure and that no critical path is duplicated. It remains
source/static evidence only: the coordinator must issue a fresh ticket for the
expanded manifest before a managed profile can complete.

Ready-frame PBR-P1-03 is also implemented at source level. Before a profile
launch, the runner writes a v1 identity manifest that binds the profile run
identifier, the managed source-manifest hash, and exact viewer binary, HDRI,
and build-provenance fingerprints. The viewer verifies those three live inputs
before it creates the RenderDoc bridge or event loop, then writes the v16
Ready-frame sidecar with the screenshot fingerprint and the complete identity
binding. The Python validator re-hashes all artifacts; the profile summarizer
then binds every measured role/ordinal to the enclosing profile identity. The
identity file deliberately projects only `path`, `sha256`, and `byte_length`:
timestamps are not content identity. Its Windows transport paths omit the
Rust `\\?\` verbatim prefix and the validators normalize that legacy form
before comparing paths, so PowerShell and Rust publish the same representation.
Every viewer output path now resolves its existing ancestors, rejects reparse
points, validates the final volume is not C:, and retains the resolved path for
the later write. This remediates the review findings before any managed run;
the new Rust/Python/Pester regression sources have not been executed under the
coordinator policy.

The capture toolchain now uses schema 2. When `renderdoc` is enabled it must
pin both the injected `renderdoc.dll` and the replay `renderdoccmd.exe` with an
absolute path, byte length, and SHA-256. The runner must pass the latter to the
replay validator explicitly; a validator-local default command is not accepted
as current-source evidence because it cannot prove that capture and replay used
the declared toolchain. The resolver rejects a generic `.dll` or `.exe` even
when its fingerprint is valid: the two files must be named `renderdoc.dll` and
`renderdoccmd.exe`. It also requires the evidence backend to equal the WGPU
backend through the canonical renderer-name mapping: selector `dx12` must
produce sidecar evidence `wgpu(dx12)`, rather than the stale bare `Dx12`
label. This prevents both a cross-API declaration and the earlier
selector-versus-renderer-name mismatch. The runner forwards the serialized
toolchain evidence field to the ready validator and the profile summarizer
derives its expected backend from that same field. When a RenderDoc capture is
requested, the runner passes the pinned replay command explicitly and records
the capture/replay fingerprints; the summarizer verifies that replay against
the same command fingerprint before accepting the run. The checked-in DX12
manifest records the RenderDoc 1.44 DLL (`27,145,600` bytes) and command
(`578,944` bytes), but it is a toolchain fixture, not a new capture or
performance measurement.

The parent Shader06 and M8 child plans retain the verified v7 roughness-coordinate
contract and the independent v8 low-roughness PDF repair. The Python
ready-evidence validator uses the current decimal recipe identity
`202608260008`; a future bake-content or mapping change must advance the Rust
and Python constants together, with an explicit fixed-version test anchor.

The broader Runtime09F1 source review identifies the real structural candidates
for a later engine-wide optimization: synchronous runtime-cache hydration on
the submission path, frame snapshots that carry resource payloads instead of
ready handles, and reflection-probe global-list scans. Those belong to the
09D resource/residency and 09B visibility/assignment owners. Shader06 must not
hide those costs behind a local PMREM or BRDF micro-optimization; only measured
profiles with owner-approved contracts may schedule that work.

## Independent review reconciliation

Two independent static reviews on 2026-08-25 found no P0/P1 defect in the
Standard-PBR IOR routing or in the capture-gated CPU timing path. Non-default
IOR is a draw-list routing bit rather than a shader permutation, and the generic
Forward material path preserves the uniform-derived dielectric F0. CPU clocks
are gated by the profiling capture state, while GPU timestamp fields do not
claim CPU recording windows.

The formula-level follow-up traces that F0 from `data12.yzw` through the
Standard-PBR direct GGX lobe and the shared environment indirect call. Both
paths use the shared `zr_pbr_material_f0` owner. This review originally also
applied the environment Fresnel residual to diffuse energy; the disposition
above supersedes that source-dependent design, while the preintegrated-GF
lookup and Unreal's `saturate(50 * F0.g)` F90 rule remain specular inputs. A
subsequent input-domain review found
that the shared owner only lower-bounded metallic `base_color`; material tint
or vertex color could therefore create F0 above one in both direct and
environment reflection. Unreal's `MaterialTemplate.ush::GetMaterialBaseColor`
saturates this value before diffuse/specular decomposition. Zircon now matches
that physical reflectance boundary in the shared owner with
`mix(clamp(dielectric_f0, 0, 1), clamp(base_color, 0, 1), clamp(metallic, 0, 1))`.
The change adds an upper-bound vector clamp at each shared F0 evaluation; it is
a correctness repair, not a measured performance improvement, and it does not
change a binding, material ABI, feature bit, shader permutation, or PSO key.
The PMREM bake inverse and WGSL lookup likewise share the canonical
`roughness = 1 -> mip_count - 3` anchor. cmft remains an offline
radiance-filter reference, not an alternative runtime GGX LOD convention. This
closes the apparent formula/mip discrepancy without authorizing a PMREM recipe,
sample-count, or cache-capacity change; only a managed current-source profile
may nominate such a change.

The same source review found that the CPU BRDF LUT used a separable
Schlick-IBL visibility approximation even though its size, sample count, lookup,
and documentation claimed Unreal `PreIntegratedGF` parity. A matched 128-sample
grid comparison against `SystemTextures.cpp` joint Smith measured channel MAE
`0.022225` and maximum error `0.300572`; at `NoV = 0.1, roughness = 1`, the old
A/B result was `0.540614/0.013935` versus Unreal's
`0.754189/0.025194`. The integrator now uses the same
`NoL * Vis_SmithJointApprox * (4 * VoH / NoH)` factor. With the canonical 128
samples retained, its 16x16 comparison against the same integrator at 4096
samples measures channel MAE `0.00302355` and maximum error `0.02227185`.
This corrects a split-sum pairing error without changing LUT extent, sample
budget, upload layout, cache ownership, or runtime texture work. It is not a
startup, frame-time, or power result; those claims remain behind the managed
measurement gate.

The IOR contract has one deliberately unclaimed boundary. The Khronos
[`KHR_materials_ior`](https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_ior/README.md)
ordinary values are valid from `1.0` upward, while its `0.0`
specular-glossiness compatibility case requires a separate effective-infinity
interpretation. Zircon's glTF projection rejects that zero value, and the
runtime normalization defensively maps other finite values below `1.0` to
`1.0`; the zero compatibility mode is therefore not implemented by this MVP
contract. The selected Unreal-compatible split-sum approximation reaches
`F90 = 1` at `F0 = 0.02`, which corresponds to `IOR ~= 1.3294`; the `IOR = 2.0`
fixture is above that boundary and cannot establish low-IOR visual coverage.
Preserve the current F90 rule and fixture for this milestone. Any claim of full
`KHR_materials_ior` conformance, support for its zero compatibility mode, or
visual parity across `1.0 <= IOR < 1.3294` requires a separately owned material
contract, fixture, and managed image/RDC comparison.

The persistent PMREM/SH9 parameter-buffer and bind-group cache changes are not
part of this audit's measurement-only work. They are the already-declared
Shader06 M4 P1-2 binding-lifetime implementation: cache identity is limited to
stable resource slots and command slices, while a sky revision still rebuilds
source content and the full PMREM/SH9 result. Do not remove or extend that
implementation based on static review alone. Its output parity, warm-hit
counts, CPU attribution, GPU duration, image matrix, and RenderDoc replay all
remain current-source managed-validation requirements.

One compatibility follow-up remains P2: CPU cache observations belong to the
public `RealtimeIblCpuTimingReport` shape, while
`RealtimeIblGpuTimingReport` remains timestamp-only. There are no in-tree
external struct literals, but a future public API stabilization must choose an
explicit versioning or extension policy before treating either report as
semver-stable.

## Optimization candidates after measurement

| Candidate | Required proof before change | Explicitly not authorized now |
| --- | --- | --- |
| CPU IEM task granularity | IEM dominates warm/cold CPU samples and workers are underutilized. | Change integrator, quality, source resolution, or persistent recipe identity. |
| PMREM face size/sample budget | Matched image matrix and a material CPU/GPU reduction. | Promote `128x8` as the viewer default based on candidate counts. |
| Realtime IBL recorder/cache | CPU/GPU timing identifies bind-group creation or recorder work as a significant phase. | Replace resources independently of the recorder or add a second cache owner. |
| Probe assignment | A local-probe workload measures fragment cost proportional to the global list. | Patch a shader-local heuristic around the fixed global scan; Runtime09B/09D own spatial assignment and residency. |
| Skinned vertex fetch fusion | Current-source RenderDoc capture identifies the generated vertex shader, D3D12 pipeline statistics normalize vertex invocations, and matched skinned-frame GPU timestamps show repeated LBS work is material. The A/B corpus must include four influences, non-normalized valid weights, non-uniform instance scale, non-uniform joint scale, and CPU fallback parity. | Assume source calls equal executed instructions, or add per-joint inverse-transpose storage/inversions without an approved tangent-frame contract. |
| Non-uniform joint-scale tangent frame | An owner-approved deformation model defines the asset/import restriction or a differentiated deformation basis, with CPU/GPU image/error evidence and palette/bandwidth cost. | Describe either weighted inverse-transpose formula as an exact LBS normal repair or hide an algorithm-policy change inside PBR/IBL work. |
| Non-default IOR routing | A scene with a known overridden-IOR draw population reports ordinary and `advanced_pbr_opaque` command counts, dynamic/cache hit/rebuild counts, CPU attribution, GPU timestamp, and image parity against the forward material reference. | Extend the fixed GBuffer for F0, add an IOR-specific PSO, or reroute these draws to deferred solely to reduce a source-level queue transition. |
| Material uniform growth | A complete ABI/PSO/bindless/fallback review and measured corpus prove the need. | Add `KHR_materials_specular` texture fields to the 256 B standard uniform. |
| Direct-light photometry and rect shape | Runtime95's unit conversion, reference fixtures, shared-module parity, same-scene GPU profile, and power gates pass before migration. | Add inverse-square attenuation, LTC, new light units, or a `GpuLightData` field locally in Shader06. |

## Exit criteria

This audit becomes actionable only after current-source managed evidence
identifies a named phase as the bottleneck and reproduces the visual result.
Any accepted optimization must update the master Shader06/M8 plan with measured
p50/p95/p99 timing, CPU/GPU/RAM/VRAM scope, visual evidence, and energy data
when a valid meter exists. Until then, the only permitted changes are MVP
correctness repairs and observation/publication infrastructure.

## IOR fixture implementation update

The source implementation now exposes the closed viewer argument
`--material-fixture <metal-mirror|dielectric-ior>`. `metal-mirror` remains the
default and preserves its original project root and `viewer-project-v4`
identity. `dielectric-ior` creates and opens an isolated `dielectric-ior`
project subtree, so a ready default-material cache can never be mistaken for
the material that carries `ior = 2.0`.

The IOR startup retains the lightweight environment renderer profile but calls
`without_environment_only_pbr_base_prewarm()` before enabling asynchronous
compilation. It queues and polls only the static material's
`generic-forward-pbr-ior` Base PSO. That key has the same fallback source,
no-shadow receiver setting, and no-texture assumptions as the generated
fixture, has `pbr_ior_override = true`, and intentionally does not enable the
environment-only feature. The mirror baseline continues to use its specialized
PSO and requires its startup prewarm report.

Ready evidence is now schema v16. It binds the selected fixture, the required
Base pipeline kind, and that exact pipeline's capture-time readiness. When the
IOR fixture is selected, the old specialized prewarm is explicitly marked
`not_requested` with false/zero status and timing fields; the validator rejects
any IOR evidence that reports the mirror PSO as its proof. Identity manifests
and profile summary readers use the matching v16 validation policy. Source
contracts cover the isolation, startup routing, CLI closure, and both accepted
and rejected v16 IOR sidecars.

Only scoped static checks have run for this update: Rust formatting, diff
whitespace integrity, Python AST parsing, and source-contract inspection. No
Cargo, WGPU, DX12, RenderDoc, screenshot, WPR, power, or benchmark command was
run. Therefore the record is
`implementation_complete_pending_managed_validation`; no image, timing, power,
routing-count, or performance claim is made here.

## Raw shader module identity implementation update

The P0-5 follow-up separates generic validated shader source from explicit
material and executable product domains. Raw WGSL/GLSL/SPIR-V import paths now
produce `ShaderAssetKind::Module`; `.zshader v2` remains the only declaration
path for Surface, Include, Compute, and Fullscreen products, and the built-in
PBR shader remains an explicit `standard_pbr` Surface. Readiness publishes a
structured kind diagnostic and checks executable stages from the already-built
canonical entry rows, so the added work is O(E), O(1) for Module/Surface, and
allocates a diagnostic string only on failure. Material validation separately
requires Surface and cannot consume a ready generic Module.

Cache hydration exposed and closed a related ownership leak: material artifact
regeneration previously ran for every shader kind, so an empty raw module could
gain a 16-byte material uniform layout and generated material WGSL after a cache
read. Regeneration is now restricted by the material-variant kind predicate and
clears stale derived material data for every non-Surface kind. New cache data
preserves Module, while old missing-kind records still default to Surface and
fail readiness when they lack a shading model. Static source contracts and
scoped formatting/diff checks pass; the artifact-cache regression is staged
but no Cargo, WGPU, RenderDoc, image, timing, RSS, or power command was run.

The offline prewarm scan was a second identity owner and still promoted raw
WGSL to Surface with the full material-pass set. That bypass is now removed:
standalone and single-unit-metadata WGSL remain Module with no material passes,
the source enumerator rejects every non-material kind before expansion, and a
material that resolves to such a source returns a typed kind-mismatch error.
Explicit Surface packages remain the only custom material prewarm source.
This changes the invalid raw-module request scale from `6 x Q x G` per source
to zero while retaining one bounded inventory/read/hash pass; it is an
algorithmic work-removal statement, not measured timing or power evidence.
The `8/8` offline source-contract gate and scoped formatting/diff checks pass;
Cargo, Windows CLI, WGPU, timing/RSS, image, RenderDoc, and power validation
remain coordinator-managed and pending.

## Generated WGSL source-cache identity update

The current-source 09C P0-2 review confirms that the compressed disk entry is
only a generated-WGSL source cache. Runtime lookup happens after full template
assembly, source hashing, and validation admission, and a hit still proceeds to
renderer-device shader-module and PSO creation. It therefore cannot be used as
evidence that the historical startup bottleneck has disappeared; compressed
I/O, decompression, and payload verification may instead be net overhead.

The correctness boundary is now content-addressed. A prewarm source ID contains
the WGSL payload hash, ordered include hashes, template revision, Naga version,
and WGPU version; the source label is provenance only. Manifest schema v3 and
disk schema v2 reject or naturally miss the old identity. Disk key, metadata,
write, and read all bind or verify the same source contract, and dynamic
builtin manifests deduplicate equal source IDs. This permits removal of the
post-hit full-source equality comparison without weakening payload integrity.

The next managed run must use E-drive roots and separately measure assembly,
hashing, metadata/read bytes, decompression, payload rehash, Naga, WGPU module,
and PSO phases for 1/100/10k variants across cold/warm/reload runs. It must also
record hit/miss/error counts, source/variant deduplication, peak RSS, PNG/RDC,
GPU timestamps, and power when a valid meter exists. No compression, cache
retention, PSO identity, or shared-compiler redesign is authorized until that
profile identifies the dominant phase. Scoped `rustfmt --check`, diff integrity,
and a `17/17` source-contract gate pass; Cargo, WGPU, RenderDoc, image, timing,
RSS, and power validation remain coordinator-managed and pending.

## Material texture binding identity update

The 09C P1-5 review found a second false-identity path adjacent to the PBR
material ABI. `MeshPipelineVariantRegistry` enumerated 16 combinations of
base-color, metallic-roughness, occlusion, and emissive presence to count
equivalent variants, but still inserted every full `PipelineKey`; the scan
therefore added work without reusing a PSO. Those four slots have a fixed bind
layout and neutral fallback textures, so presence belongs to material binding,
capture, and readiness state rather than shader or pipeline identity. Normal
texture presence remains in `HAS_NORMAL_TEXTURE` because it changes tangent
requirements and generated source. Unreal's material uniform-expression path
uses the same ownership principle: texture parameters bind through the material
resource table, and an invalid 2D resource resolves to `GWhiteTexture` instead
of becoming an unrelated global-shader permutation.

Current source removes the four fields from `PipelineKey` and
`ShaderPipelinePrewarmState`, deletes the 16-way equivalence scan, and keeps
the runtime texture IDs/fallback diagnostics unchanged. The old new-variant
path performed one canonical lookup plus up to 16 extra HashMap probes and
could retain 16 PSO keys for source/descriptor-equivalent bindings; the new
path performs one lookup and admits one key. This is a correctness and
algorithmic-scale result, not a measured speedup. Static hard-cut and formatting
checks are required in this Session; Cargo/WGPU, 1/1k/100k variant timing,
pipeline creation counts, peak RSS, PNG/RDC, GPU timestamps, and power remain
coordinator-managed and pending. The plan status and Shader06 milestones do not
advance from this source-only slice.

## Surface source contract publication

The runtime material-path review found that `ShaderAssetKind::Surface` still
covered both template material functions and historical complete
`vs_main`/`fs_main` programs. `ResourceStreamer::shader_uses_material_surface_source`
therefore scanned every requested mesh pipeline source with
`str::contains("fn zr_material_surface")`. That accepted commented anchors,
could not diagnose duplicate or mixed material/executable declarations, and
left readiness able to publish a Surface with no usable material function.
The builtin PBR asset also omitted its existing vertex/fragment entry rows, so
an immediate all-Surface hard cut would have misclassified the fallback marker.

Current source introduces the typed `ShaderSurfaceSourceContract`. A
comment-aware, allocation-free single pass accepts exactly one authored
`zr_material_surface` with one `ZrVertexOutput` parameter (or its public
`ZrSurfaceInput` alias), a `ZrSurfaceOutput` return, and no executable entry rows as `MaterialFunction`, or the exact
transitional `vs_main` vertex plus `fs_main` fragment pair as
`LegacyFullPass`. Missing, duplicate, mixed, and non-canonical shapes return a
typed error; signature mismatch is also a typed failure. Readiness and streaming use the same classifier; streaming stores
the accepted enum in `ShaderRuntime`, after which pipeline source resolution is
an O(1) enum comparison. Initial classification remains O(source bytes), while
the previous per-request O(source bytes) scan is removed. This is an algorithmic
bound, not measured CPU, GPU, or power evidence.

`LegacyFullPass` is explicit migration debt, not a newly endorsed architecture.
The follow-up must either move it to an executable shader kind or remove it
across `.zshader`, material, prewarm, and fallback identity owners; no builtin
URI exception is acceptable. Reflection-derived stage IO, bind-group/material
ABI, and layout hash also remain open P0-5 work. The structural red gate was
5/5 before implementation; unit and structural regressions are staged, but
Cargo/WGPU/PNG/RDC/profile were not run and milestone status is unchanged.

## Compiled shader reflection artifact decision

The follow-up P0-5 review confirms that Zircon currently has three disconnected
layout views. `ShaderAsset.pipeline_layout` is an author-authored descriptor,
`renderer_material_layout_diagnostics` treats an empty descriptor as an opt-out,
and Mesh pipeline construction consumes fixed WGPU bind-group layouts. Naga
already returns `ModuleInfo` during import and template validation, but current
code discards it after collecting entry-point names. Background template
validation also completes independently from WGPU pipeline creation, so adding
more checks only to asset readiness would not establish compiled ABI authority.

The Unreal reference keeps parameter-structure metadata, the compiled shader
parameter map, and RHI binding identity connected: binding records the
`StructureLayoutHash`, rejects compiled parameters not represented by the
declared structure, and hashes resource/member layout into both runtime and
persistent signatures. The applicable Zircon direction is therefore a compiled
reflection artifact produced from the same validated Naga `Module + ModuleInfo`,
not another WGSL text scan or hand-maintained layout hash.

The next non-accepting infrastructure slice will enrich the existing Naga
validation result with immutable entry-point stage IO, entry-point-reachable
resource bindings, merged stage visibility, and deterministic interface/resource
layout hashes. Reflection work is bounded by O(entry points x globals + reflected
type graph) and reuses the existing parse/validation pass; it must not add Naga,
filesystem, WGPU, or wait work to the frame thread. The artifact is preparation
for authoring-schema comparison and exact pipeline admission. It does not by
itself make the current asynchronous diagnostic worker a fail-closed admission
gate, remove the manual layout DTO, or complete P0-5. CPU timing, queue latency,
WGPU creation, GPU timing, RSS, RenderDoc, image, and power evidence remain
coordinator-managed and pending.

Current source now publishes that artifact from the existing template
validation pass. Every reflected entry stores canonical stage IO plus its own
entry-reachable resource identities; the module resource table independently
merges stage visibility. Resource identity includes group, binding, address
space/access, memory decorations, and a recursive type-layout hash. This avoids
the incorrect assumption that two fragment entries necessarily use the same
resources, and it preserves distinct ABI shapes that reuse one binding location
instead of silently taking the first declaration. Entry, IO, and resource rows
are sorted before hashing. Resource-variable names and diagnostic labels are
excluded, while buffer structure member names remain in the recursive type hash
because material/host property mapping consumes them. Declaration order and
resource-variable renames therefore do not perturb the ABI hash.

The reflection path performs no second parse or validation pass. Its additional
work remains O(entry points x globals + reflected type graph), with type hashes
memoized once per Naga type and no frame-thread, filesystem, WGPU, or wait work.
Eight unit contracts are staged for unused-resource exclusion, cross-stage
visibility union, flattened stage IO, same-stage entry isolation, declaration
order/name stability, independent interface/resource ABI invalidation,
buffer-member-name identity, workgroup-size overrides, and override-sized
resource layouts. Scoped formatting and the 14/14 static source gate pass
locally; Cargo/Naga
execution, current-source Windows tests, WGPU admission, timing/RSS, PNG/RDC,
GPU timing, and power remain coordinator-managed. P0-5 and all Shader06
milestones therefore remain open.

Naga 29 keeps resolved default workgroup dimensions separate from
`workgroup_size_overrides`; pipeline constant values are applied later. Current
source now publishes the module override count, each entry's overridden
dimensions, and specialization-dependency flags for stage-IO and resource type
layouts. Those flags participate in interface/resource hashing, while raw Naga
handle indices do not. A future exact `PipelineDescriptorId` must include the
selected pipeline constant value set before treating a specialized layout as
exact. The current repository has zero tracked WGSL `override` declarations and
all 139 WGPU `compilation_options` rows are default, so this adds no current
permutation or PSO dimension. The specialization static contract passes 10/10;
the two new Naga tests remain coordinator-managed and unexecuted locally.

## Pipeline admission and reflection publication disposition

The next ownership review rejects a local "reflection diagnostic" patch. Mesh
runtime currently owns two independent bounded workers: up to 64 source
validation jobs and up to 64 Base PSO jobs. `mesh_pipeline_shader_source_with_cache`
queues Naga validation, then immediately returns WGSL to either synchronous
WGPU creation or the unrelated PSO worker. The async jobs can race; the
synchronous path always proceeds before validation completion. Depth, GBuffer,
OIT, shadow, velocity, and TAA paths also create modules and pipelines
synchronously. Consequently a reflection result stored only in the diagnostic
completion queue cannot become an admission authority.

Offline prewarm already validates one unique source once and may additionally
run WGPU module/pipeline validation, but `ShaderVariantCacheDisk` persists only
compressed WGSL plus source/version metadata. A runtime hit still queues Naga
and creates a WGPU module/PSO. This confirms that the current disk layer is not
a validated source artifact or compiled pipeline cache, and that a second
reflection sidecar per variant would duplicate source-owned data.

The selected structural direction is one content-addressed
`ValidatedShaderSourceArtifact` owner keyed by the complete source contract.
It will contain authenticated WGSL/source identity, Naga version, canonical
reflection rows/hashes, structured diagnostics, and specialization-dependency
flags. Offline prewarm or a background compiler produces it once per unique
source. A pipeline compile job must consume that artifact, select exact entry
points and specialized resource identity, then publish module plus PSO only
after the WGPU validation scope succeeds. Frame code observes pending/ready/
failed state and last-good/error policy; it neither runs Naga nor waits except
at an explicit bootstrap synchronization point. The same service must cover
all mesh passes rather than only Base.

Before changing worker counts, default async policy, disk persistence, or PSO
publication, the managed profile must record per phase (assembly, source hash,
Naga parse/validate/reflection, source-cache IO/decompress/rehash, WGPU module,
PSO, queue wait) p50/p95/p99 for 1/100/10k requests, unique-source and
unique-pipeline counts, duplicate validations, queue-full/unavailable outcomes,
peak in-flight WGSL/reflection bytes, peak RSS, ready latency, and WGPU errors.
GPU timestamps, PNG/RDC, and power remain separate product evidence. The target
algorithm is one validation/reflection per unique source contract and one WGPU
creation per exact pipeline identity, with no frame-thread Naga/I/O/wait. No
queue or cache optimization is authorized from source inspection alone.

## Surface full-pass compatibility removal

The source-shape review found no repository-owned Surface package that needs an
embedded executable pass: all six tracked Surface packages author the canonical
`zr_material_surface` function. The only remaining full-pass Surface source was
the built-in `builtin://shader/pbr.wgsl`. Runtime mesh source selection already
identified that asset through the fallback pipeline key and assembled the
standard material templates instead of consuming its raw executable source.
Keeping `LegacyFullPass` therefore preserved a second ownership model without a
current runtime consumer.

Current source deletes that transitional classification. Surface readiness and
streaming accept only the canonical material-function ABI; authored full-pass
WGSL must use an executable shader kind. The built-in PBR fallback is now the
same minimal `zr_material_surface` input as project Surface packages and exposes
no executable entry rows. Before the cut, its concatenated WGSL payload was
24,220 bytes (19,149 bytes of includes plus 5,068 inline bytes). The replacement
is 225 bytes, removing 23,995 bytes or 99.07%; because the built-in asset retains
both `source` and `wgsl_source` strings, the immediate resident-string reduction
is about 47,990 bytes before prepared/cache clones are considered. This is a
source-size and ownership measurement, not a CPU/GPU/power performance claim.

The hard cut removes one per-request behavioral branch and prevents Surface
metadata from bypassing the template ABI, but it does not close P0-5. Managed
Cargo/Naga/WGPU validation must still prove package import, fallback template
assembly, and every pass pipeline. The 1/100/10k admission profile, PNG,
RenderDoc replay, GPU timing, RSS, and power gates remain pending; all Shader06
milestone states remain unchanged. The local 15/15 static hard-cut contract and
scoped formatting gate pass.

## Authored pipeline-layout DTO disposition

The follow-up ownership audit rejects comparing compiler reflection to
`ShaderAsset.pipeline_layout` as the next admission step. Schema-v2 `.zshader`
explicitly forbids authored `pipeline_layout`, every current Surface package is
imported with the default empty descriptor, and material ABI diagnostics treat
that empty value as an opt-out. The descriptor is cloned through asset cache and
reported by readiness, but no production consumer uses
`pipeline_layout_descriptor()` and no WGPU bind-group or pipeline layout is
constructed from it. Its push-constant ranges are strings with no runtime
consumer. It is management/legacy metadata, not the renderer ABI authority.

The correct comparison boundary is the specialized reflection artifact against
the actual pass pipeline-layout owner immediately before WGPU module/PSO
publication. Reflected required bindings must be compatible with the selected
layout; extra bindings permitted by a shared layout must not be mistaken for
shader requirements. That check belongs in the planned unified compile service,
not material readiness and not a second hand-maintained DTO. Removing the DTO is
a separate serialized-schema/cache hard cut that needs an explicit migration
inventory before code changes. No deletion or admission optimization is made in
this slice, and the existing managed measurement/acceptance gates remain in
force.

## Base pipeline admission implementation update

The upstream fallback review confirms that a fallback pipeline cannot be
selected independently from material bindings. Unreal
`MaterialRenderProxy.h:147-164` and `MaterialRenderProxy.cpp:865-887` walk a
proxy chain until both a complete shader map and its corresponding proxy are
available. Base, depth, and shadow mesh processors repeat that material/proxy
selection in `BasePassRendering.cpp:1927-1945`,
`DepthRendering.cpp:1089-1105`, and `ShadowDepthRendering.cpp:2551-2567`;
shadow may select the domain default material only when opacity and vertex
modification rules make that substitution valid. Bevy
`pipeline_cache.rs:45-80,282-353,691-743` independently confirms that WGPU
pipeline state must distinguish queued/creating/ready/error and that transient
missing shader dependencies may be requeued while processing and module errors
remain terminal.

Current source therefore deletes the label-only
`PipelinePlaceholderPolicy::{SkipDraw, DepthOnly}` rather than pretending it is
a fallback implementation. Base admission now returns
`PipelineAdmission::{Ready, Deferred, Failed}` with stable reasons. Queued,
pending, and saturated work is recoverable; compilation disabled, worker loss,
job panic, unknown/wrong-pass variants, geometry/source failure, and WGPU
validation failure are terminal. Per-variant state age resets when the reason
changes. Opaque and transparent Base consumers publish canonical shader/material
identity, pipeline variant id, entity, consumer, state, action, reason, age, and
occurrence through `ShaderVariantMissReport`. The report coalesces repeats in
place and caps fallback rows at eight. The ready path adds no diagnostic
allocation or scan; once the buffer is full, a new context only increments the
aggregate counter and returns before canonical-key, consumer, or reason string
allocation, while an existing context still updates its count and maximum age.

The same slice removes an impossible cache-source failure state.
`mesh_pipeline_shader_source_with_cache` always selects a disk hit or the
assembled WGSL fallback, so its return type is now `String` rather than
`Option<String>`. Eight mesh-pass call sites no longer use `?` to collapse that
nonexistent failure into `None`. Real geometry/source assembly failures and the
legitimate absence of an OIT fragment-store entry remain explicit.

The follow-up pass audit found that GBuffer, DepthPrepass, ShadowDepth,
Velocity, TAA reactive masks, and OIT synchronously created a PSO behind
`Option<&RenderPipeline>`. Unknown/wrong-pass variants, geometry/source
failure, and WGPU validation were collapsed into `None`; several consumers
then called `expect`, making this a frame-path panic contract rather than an
explicit fallback policy. These passes cannot all share a color fallback:
depth, shadow, velocity, and reactive-mask outputs have different attachment
and correctness semantics.

Failure identity also cannot be keyed by `MeshPipelineVariantId` alone. OIT
intentionally reuses a transparent Base variant id while publishing a distinct
OIT PSO and layout. The shared terminal-failure and state-age storage is now
keyed by `(PipelineCreationTarget, MeshPipelineVariantId)`, and toggling Base
async compilation clears only the Base target. A target-identity regression
test fixes Base and OIT as distinct keys. Every mesh consumer now uses that
target-scoped typed admission store. GBuffer command projection, DepthPrepass,
Velocity, both Shadow kinds, both TAA reactive kinds, and OIT return
`PipelineAdmission`; their terminal failures publish `deferred_gbuffer`,
`depth_prepass`, `shadow_atlas`, `velocity_object`, `taa_reactive_mask`, or
`oit_fragment_store` with `RejectDraw`, then invalidate replay state. Ready
cache hits fetch the already-published pipeline without cloning the complete
variant key. Shadow and TAA accept the command's exact kind so sibling targets
sharing a cache container remain isolated. OIT retains its distinct target,
classifies a missing `fs_oit` entry as `oit_fragment_store_unavailable`, and
still returns a graph execution error after recording the diagnostic. Its
failure is not silently converted into a successful frame. The two TAA PSO
paths now share one source/module/error-scope state machine and differ only at
exact-kind pipeline creation and publication. The legacy five synchronous
`Option` APIs and their production frame-path `expect` calls are absent.

The same call-graph review found that all seven synchronous Mesh PSO creation
paths resolved their WGPU error scopes before storing a receipt, while the
bounded receipt queue still treated 64 successful results as outstanding work.
Because the product frame drained that queue only before creating the current
frame's pipelines, a 65th valid cold PSO could be rejected as a terminal
validation failure. The source now consumes a full resolved batch on capacity
rollover and retains the current receipt, preserving the 64-entry bound and the
prewarm `finish` contract without turning successful receipts into admission
failure. This is a correctness repair only. The synchronous error-scope
`block_on` remains an unoptimized structural candidate. The existing profiler
now receives one `render/shader_pipeline/wgpu_pipeline_error_scope_pop` scope
per synchronous validation plus queue-depth and rollover counters, so cold
1/64/65/1,000 variant CPU p50/p95/p99, submission wait, queue peak, and
false-failure counts can be collected without adding an acceptance-only probe.
Those coordinator-managed measurements remain required before any performance
claim.

The material publication review then traced the complete Zircon draw path and
the Unreal fallback proxy chain. Zircon previously replaced an existing
`PreparedMaterial` with a blocking candidate before returning the readiness
error, after which `ensure_scene_resources` aborted the frame. That destroyed
the only coherent old revision even though each immutable draw payload already
retains its pipeline identity and both custom and standard material bindings.
The current leaf now retains that renderable revision, runtime, pipeline key,
and both uniforms for blocking readiness, shader dependency preparation, and
texture residency failures. The rejected revision and readiness report remain
attached to the entry for management and frame statistics. A root
`PreparedShader` publishes only after its import and registry dependency
closure succeeds, and texture residency now precedes candidate uniform-buffer
creation. Cold failures retain fail-closed behavior and return the original
error.

The draw-facing material fields are now owned by one `PreparedMaterialBundle`:
material revision, shader and texture dependency identities, runtime state, six
texture resource snapshots, and both custom and standard GPU uniform bindings
move together. Hot updates with a last-good bundle are staged. Direct and
compiled scene paths derive the exact current-scene material/geometry/quality
requirements for the active render-graph features, advance all
Base/GBuffer/Depth/Shadow/Velocity/TAA/OIT admissions, and publish only when the
complete set is ready. Current-frame draws keep the old bundle; a successful
candidate becomes visible as one bundle on the next frame. Cold success also
remains staged and uses the engine error proxy until its exact requirements are
ready; there is no unadmitted immediate-publish exception.

Transient visibility and current velocity-history availability do not trim the
candidate set: doing so would publish an incomplete generation that first needs
a PSO when an object becomes visible or gains history. Material pass disables,
phase/routing, shadow policy, graph executors, and actual geometry sources do
trim it. The stable path uses an active-candidate ID index; with no candidate it
does not scan graph passes, draws, or all materials. A terminal PSO rejection
retains the complete staged identity outside the active index, so the same
identity creates zero additional uniform buffers on later frames while any
shader, texture, material, or upload-support identity change can retry.

The existing backend submission timeline is the retirement authority. Its
generation-qualified `SubmissionTicket` and compressed terminal history already
support exact completion checks, and the compiled render path uses that ticket
for transient-resource retirement. Shader modules and PSOs still lack per-frame
variant-use attachment to the submitted ticket; adding a separate fixed-frame
delay would duplicate the RHI owner and is not an acceptable retirement design.
Draw and command payloads retain owning handles for old material uniforms,
textures, and geometry through the current submission, but that reference
safety is not bounded PSO retirement.

The same review found a cache-identity defect independent of material revision:
`PreparedMaterial` did not track its shader resource identity, so shader-only
edits returned from the cache-hit branch without preparing any candidate. The
published material now snapshots the direct shader locator, resource id, root
revision, and the process-local readiness revision of its complete dependency
closure. The same dependency revision participates in the runtime
`PipelineKey`, so include/import leaf changes invalidate every mesh pass that
derives its variant from that key without polluting persistent
`ShaderVariantKey` identity with a process-local epoch. The immutable readiness
generation and maintained reverse-dependency projection keep stable-frame
checks O(1); actual dependency traversal remains on rebuild paths. Atomic
cross-pass material-bundle publication, exact Mesh PSO readiness, and
same-identity rejected-candidate suppression are now implemented. Cold
candidates now keep `published=None`; draw construction pairs the default
pipeline identity with fallback custom/standard uniforms, fallback textures,
and a magenta base tint until exact requirements are Ready. This closes the
mesh-scope ABI-paired error-proxy source boundary without inventing a shader-only
fallback. A follow-up camera-loop audit found frame generation advances per
camera, so publication now resets stale state at an explicit viewport start,
aggregates observed/deferred admission across the whole camera sequence, and
commits only at the viewport-terminal camera. An unreferenced candidate remains
staged instead of bypassing admission with an empty requirement set, but is
parked outside the active census until a later material-preparation cache hit
reactivates it. After publication, draw construction admits the newest bundle
against the exact current graph/quality/fog/geometry context before phase
ordering, virtual geometry, GPUScene, or static command-cache projection. A
non-ready context selects the complete previous bundle, or the complete error
proxy when no usable previous bundle exists; it never substitutes only a shader
or only uniforms.

The texture portion of that proxy is revision-aware. Each bundle retains all six
resource handles. A replacement `GpuTextureResource` is consumed from the
streamer only when its asset revision equals the selected bundle's captured
revision, allowing same-revision mip-residency promotion/eviction without
rebuilding a material. A texture hot reload with a different revision keeps the
selected previous bundle on its captured texture resource. This removes the
former old-uniform/new-texture mixed generation while preserving mip streaming.
Old variant/module/PSO retirement and submission-ticket usage attachment remain
open.

The Hybrid GI follow-up found that draw texture publication was atomic but its
CPU material-capture projection was not. `RuntimePrepareMaterialCaptureCache`
read the latest asset for every referenced channel and indexed its result only
by `ResourceId`. During a texture hot reload, an old published material bundle
could therefore combine with a newer texture sample; two live revisions of the
same id also collided in the frame cache. This bypassed the exact bundle
identity established above.

`PreparedTexture` now stores an optional fixed RGBA center sample computed from
the same loaded `TextureAsset` used to create its GPU upload work. The six-slot
material bundle copies that sample with the texture revision, and the published
draw proxy projects the five Standard-PBR capture channels from that exact
bundle. A mip-residency replacement with the same revision carries the original
sample. Output-target and fallback bindings publish no CPU sample. The runtime
collector DTO carries both revision and sample for each channel, while Hybrid
GI stores samples by `HybridGiMaterialCaptureTextureKey { id, revision }`; the
card and voxel sampling interfaces no longer accept a bare texture id. Old and
new revisions can coexist until their material generations retire.

The no-streamer-state cold compatibility path still derives a revision-bound
sample from the canonical effective material, but any staged, rejected, or
published material state is governed by the proxy path. A test-first hot-reload
contract fixes the sequence `published white -> prepared black -> staged black
-> published black`: capture retains the original revision/sample through both
intermediate states and changes only on material publication. Static contracts
also require the generation-keyed Hybrid GI map, all five revision/sample DTO
pairs, the complete re-export chain, and zero latest-asset reads in the stable
frame cache. Scoped Rust formatting and diff integrity pass. Cargo/WGPU, image,
RenderDoc, timing, RSS, and power evidence remain blocked at the shared
lockfile gate, so this repair does not close M6.

A material-parent cache review on 2026-08-27 found a second cross-layer
generation split. `material_with_parent_chain` correctly resolves inherited
values during rebuild, and advanced-PBR feature extraction can independently
resolve the current parent data, but the render bundle cache had no parent
generation identity. Because the project material importer also omitted the
parent locator from `AssetImportOutcome.dependencies`, the resource reverse
closure could not invalidate a prepared child after a parent-only scalar,
texture, or routing-feature change. The renderer could therefore pair newly
extracted features with permanently stale uniforms and pipeline identity.

The importer now consumes the asset layer's single direct-reference collector,
which publishes the parent locator in the same preallocated dependency vector
as shader and deduplicated textures; registry extraction and import metadata no
longer carry separate enumeration rules. `PreparedMaterialBundle` snapshots
the actual loaded material id, its direct revision, and
`ResourceReadinessGeneration::dependency_revision`; the actual id is required
because a missing requested material may be served by the engine fallback.
Cache admission compares this three-field identity before accepting either a
published or staged bundle. The stable path performs one registry lookup and
one sharded readiness-generation lookup per inspected cache slot, so the
material-preparation cache complexity remains `O(1)` and parent depth does not
enter that cache lookup. Advanced-PBR frame feature extraction is a distinct
per-frame consumer and still resolves each unique visible material; its cost is
recorded below rather than incorrectly described as rebuild-only. This matches
Unreal's parent-chain uniform-cache invalidation intent while using Zircon's
existing reverse closure instead of an editor-style global material-instance
scan. Test-first source contracts cover parent dependency publication and
direct/recursive generation mismatch; Rust formatting and scoped diff integrity
pass. Managed Cargo and product validation remain pending at the shared
lockfile gate.

A follow-up effective-material call-graph review found three semantic owners.
Material preparation emitted typed missing/cycle/depth/shader-mismatch parent
diagnostics, frame feature extraction silently maintained a second walk, and
Hybrid GI material capture preferred `latest_prepared_material_bundle`, which
selects a staged candidate before the published bundle. Its cold branch then
read `MaterialAsset::standard_material_descriptor` directly, omitting parent
inheritance and shader-aware descriptor projection. The accessor also called a
test-only `material` method from production source, a compile-time defect that
the shared lockfile gate had prevented current-source Cargo validation from
exposing.

`ProjectAssetManager::load_effective_material_asset` and its loaded-root variant
now own the complete parent policy. The lineage has capacity five (root plus the
declared maximum four parents), performs at most four parent asset loads, and
uses the already-bounded lineage for cycle detection instead of allocating a
second tree/set. At the declared bound this is at most ten `ResourceId`
comparisons; changing the depth or adding an effective-material cache requires
profile evidence and a generation identity, not an unbounded walk. Missing or
invalid parents still preserve the renderable child and return the same typed
validation diagnostic consumed by material readiness. Material preparation and
advanced-PBR feature extraction now call this owner; the former supplies its
already-loaded root to avoid a duplicate root load.

Hybrid GI capture now mirrors the Unreal
`LumenSceneCardCapture.cpp` boundary: an existing material state can contribute
only its published runtime proxy. A cold staged or rejected candidate returns
no capture seed and therefore uses the plugin's default-material fallback; it
cannot leak new scalar, texture, shading-model, or parent data before PSO
admission. Only an id with no streamer material state may use the compatibility
cold asset path, and that path now resolves the canonical parent chain plus the
shader-aware standard descriptor. Static RED/GREEN contracts cover the single
parent-policy owner, published-before-cold ordering, staged-state guard, and
parent inheritance in cold capture. Scoped Rust formatting and diff checks
pass; Cargo, WGPU, Hybrid GI image capture, RenderDoc, timing, RSS, and power
remain unrun.

The structural convergence does not claim to remove the remaining per-frame
feature-census cost. `render/material/advanced_feature_census` now records the
scope, `advanced_feature_material_resolutions` records unique visible material
resolutions, and `advanced_feature_parent_diagnostics` records invalid parent
results. Capture separately records `material_capture_cold_asset_resolution`
and `material_capture_unpublished_fallback`; the published branch additionally
records `material_capture_published_proxy` and the number of channels carrying
both revision and fixed sample as
`material_capture_generation_bound_texture_samples`. A managed
1/100/10,000-material profile must determine whether effective-material
cloning/parent loads are material before a dependency-generation keyed
cross-frame cache is authorized.

A compiled-graph state-transition review on 2026-08-27 found that executor
feature discovery was guarded only by `active_staged_material_ids`. Successful
publication removes that id and adds it to `context_admission_material_ids`.
Consequently, the first later viewport could enter published-generation context
admission with `MaterialPipelineFeatureSet::default()`: the census created an
empty requirement set and `ensure_material_pipeline_requirements` returned
`Ready { requirement_count: 0 }`, even though that graph's PSOs had never been
admitted. `ResourceStreamer::has_material_pipeline_admission_work` now keeps
compiled executor discovery enabled while either state set is nonempty. A
test-first source contract failed against the staged-only gate and passes after
the single guard replacement; Rust formatting and scoped diff integrity pass.
This is static state-machine evidence only. A managed multi-viewport WGPU run
must still demonstrate deferred/current-to-previous selection and subsequent
promotion before acceptance.

Profile evidence must use the new `render/material/prepare`,
`render/material/advanced_feature_census`,
`render/material/staged_requirement_census`,
`render/material/current_requirement_census`,
`render/material/previous_requirement_census`, and
`render/material/context_admission` scopes and
`material_prepare_cache_hit`, `material_prepare_rebuild`,
`material_last_good_rejection`, `material_uniform_buffer_creations`,
`material_candidate_terminal_cache_hit`, `material_candidate_reactivated`,
`material_candidate_publication_stale`,
`material_cold_error_proxy`, `advanced_feature_material_resolutions`,
`advanced_feature_parent_diagnostics`,
`material_capture_published_proxy`,
`material_capture_generation_bound_texture_samples`,
`material_capture_cold_asset_resolution`,
`material_capture_unpublished_fallback`, the material pipeline candidate and requirement
counters including `material_pipeline_candidate_unobserved`, the
`material_context_admission_*` selection/readiness counters, and
`shader_artifact_cache_hit`, `shader_artifact_rebuild`,
`shader_dependency_generation_invalidation`, and `shader_artifact_publish`
counters. Warm 1/100/10,000-material frames must show zero rebuilds and uniform
allocations. An invalid transitive shader-leaf reload must retain draws with
zero root-shader publish and zero material uniform allocations; recovery must
publish exactly one replacement generation. CPU p50/p95/p99, allocation, RSS,
submission wait, old-generation residency, and the number of retries over 300
invalid frames remain required managed measurements rather than inferred
results.

This remains a prerequisite rather than the complete renderer-wide publication
policy. Deferred hot reload keeps the old published bundle; terminal candidate
failure keeps that same old bundle and publishes a structured rejection report.
Cold load uses the engine-owned default material proxy/bindings and default PSO
identity as one fallback contract while the real candidate is staged. A later
viewport with an unseen graph/quality/fog/geometry context now receives
context-qualified current/previous/error selection. Eagerly compiling the full
`4 qualities x 2 fog states x pass x geometry` domain remains rejected because
it would create a compile storm without usage evidence.

Two structural limits remain explicit. First, only the immediately preceding
published bundle is retained. Rapid `G1 -> G2 -> G3` publication can retire `G1`
while a rare context still depends on it; the result stays ABI-correct and
fail-closed through the error proxy, but last-good continuity is not yet
generation-complete. The eventual owner must retain generations by observed
context usage and submission lifetime, not add another fixed history depth.
Second, once any material enters context admission, current source performs an
additional `O(D + R)` census for a build, where `D` is pending draws and `R` is
the deduplicated exact requirement set. A later optimization may cache an
observed-context signature or fold census collection into the existing draw
walk, but only after counters and a managed profile show this scan is material;
no viewport-only key or full Cartesian precompile is authorized.

No queue size, worker count, cache policy, or shader algorithm is changed from
static inspection. The managed profile must still collect assembly,
Naga/reflection, disk, module, PSO, context-census and queue-wait p50/p95/p99 for
1/100/10k requests, duplicate validation/create counts, queue outcomes,
in-flight bytes, peak RSS, ready latency, GPU timing, image error, RenderDoc,
and power before performance tuning or milestone acceptance. Explicit old
PSO/module retirement still requires submission-ticket usage attachment.

The retirement audit also confirms that mesh variant IDs, all per-pass PSO
maps, and the shader-module map are append-only during normal operation. The
existing miss report already exposes registered variant, cached pipeline,
cached module, creation-count, and creation-CPU gauges, so the next 300-frame
reload run can prove growth without new instrumentation. Safe reclamation still
requires the exact variant-usage set from final command buffers to be attached
to the backend submission ticket in both direct and compiled paths. Deleting or
reusing IDs before that would invalidate command-cache and async-job identity;
touching every resident variant each frame would be O(N) and would prevent
retirement. No reclamation code is authorized by this source-only audit.

The transitive-generation review is now grounded in the same runtime boundary.
Unreal's `FMaterialShaderMapId` includes referenced functions, shader/pipeline/
vertex-factory dependencies, expression includes, and external code references;
Bevy tracks resolved imports, reverse dependents, and every affected pipeline.
Zircon already computes a sharded resource reverse closure and exposes a
process-local `dependency_revision`, while offline prewarm separately computes
a stable SCC-compressed include topology hash. These identities must not be
collapsed: the readiness revision is the O(1) runtime dirty epoch, and final
assembled WGSL/include/toolchain hashes remain the persistent disk identity.

The MVP source slice now stores root plus transitive revisions in
`PreparedShader`, snapshots the transitive revision in `PreparedMaterial`, and
adds it only to the runtime `PipelineKey`. Stable hits stop recursively walking
the shader graph; a changed leaf rebuilds and publishes dependencies before the
root, then changes every Mesh pass/OIT variant derived from the shared pipeline
key. `ShaderVariantKey`, prewarm JSON, and disk canonical strings do not receive
the process-local epoch. Old-generation PSO retirement remains open until a
frame/fence owner can bound lifetime across passes; managed reload profiling
must quantify that resident growth rather than hiding it.

Source-level coverage now includes the O(1) root/transitive identity predicate,
material cache rejection of a transitive-only epoch change, runtime PipelineKey
separation with identical persistent ShaderVariantKey, and a product leaf
include sequence `valid -> invalid -> recovered` with unchanged material/root
revision. The invalid stage must retain the old shader, pipeline key, and both
uniform buffers; recovery advances only `shader_dependency_revision` and
publishes a replacement material bundle. Rust formatting and scoped diff
integrity pass, but Cargo/WGPU/PNG/RDC/profile remain unrun, so M6 stays in
progress and no timing, RSS, GPU, or power gain is claimed.

The subsequent draw-command audit found that publishing the new bundle was not
enough to make the generation authoritative at submission. Static command-cache
entries share an immutable payload containing the pipeline key, material texture
and sampler bind group, custom and standard uniform bind groups, and geometry.
Its previous material invalidation input was derived from the asset revision
only. A transitive shader reload could therefore produce a new runtime pipeline
key while the cache replayed the old PSO payload; mip streaming could replace a
`GpuTextureResource` while replay retained the old texture bind group. Both are
correctness failures, not optional cache tuning.

Each successfully constructed `PreparedMaterialBundle` now owns a process-local
nonzero draw generation. After frame policy finalizes volumetric-fog state and
texture anisotropy, mesh preparation combines that generation with the complete
runtime `PipelineKey`, all six material texture binding identities, and both
uniform resource identities. The resulting material submission revision is the
static command-cache invalidation authority. A source revision of zero remains
zero, so non-authoritative draws cannot become cacheable through pointer identity.
The identity is deliberately resident-only and is excluded from serialized
assets, `ShaderVariantKey`, prewarm, and disk cache contracts. Prepared-queue
batch identity now also includes the previously omitted clearcoat-normal binding.

Source coverage fixes final-pipeline and texture-resource changes as distinct
submission revisions and preserves zero authority. Rust formatting and scoped
diff checks cover this slice; Cargo, WGPU replay, mip-streaming product capture,
RenderDoc, timing, RSS, and power remain pending. The cross-pass all-ready
publication coordinator and cold error proxy are now implemented at mesh scope;
`SubmissionTicket`-based old-generation retirement and managed product/profile
evidence remain open, so this repair does not close M6.

Two managed Windows validation attempts on 2026-08-26 stopped before Cargo was
launched. The first accepted `cargo.acquire` but timed out during coordinator
reconciliation; the second was rejected as `unmanaged_artifacts_detected` for
pre-existing D/E/F artifacts owned outside this session. A new package-scoped
Windows attempt on 2026-08-27 acquired the managed D-drive pool and launched
`cargo build -p zircon_runtime --locked`, but Cargo stopped before compilation
because the shared `Cargo.toml`/`Cargo.lock` state would update the lock file.
`-NoLocked` was not used because lockfile work is outside this shader slice.
Those shared files and artifacts were not changed, deleted, or adopted, and no
raw Cargo bypass was used. Rust formatting, material-generation source-contract
checks, and scoped diff integrity are the only current evidence; Cargo type
checking, WGPU, PNG, RenderDoc, timing, RSS, and power remain pending, so M6
stays in progress.

## Atomic asset/revision snapshot implementation update

The 2026-08-27 publication-boundary review found a lower-level race beneath the
generation-bound Hybrid GI repair. `ensure_texture`, mip rebuilds, shader source
preparation, root material preparation, output-target preparation, LUT
preparation, and cold texture capture could read `ResourceRecord::revision` and
the typed payload through separate authority-lock acquisitions. A committed hot
reload between those reads could therefore publish revision `N+1` with payload
`N`, or revision `N` with payload `N+1`. A `(ResourceId, revision)` cache key
cannot repair an identity that was already assembled from incoherent reads.

The core resource manager already owned the required primitive:
`ResourceManager::snapshot` clones the record and typed payload under one
authority read lock. `ProjectAssetManager::load_typed_snapshot` now makes that
primitive the single typed loading owner for material, shader, and texture
snapshots. GPU preparation derives the prepared revision from the returned
snapshot, and the old revision-only lookup remains only as a cache-hit probe.
The material root and shader contract retain their exact loaded revisions;
texture upload-readiness diagnostics likewise store the revision of the payload
they inspected. Mip rebuild accepts the snapshot only when its revision equals
the currently prepared generation.

PSO admission can span frames, so atomic loading alone is insufficient. Before
moving a staged material into the draw-visible slot, the publication boundary
now reruns the existing O(1) root/transitive shader/material/texture identity
predicate. A stale candidate remains staged and increments
`material_candidate_publication_stale`; the next preparation pass replaces it,
while the current published bundle or cold error proxy remains authoritative.
This prevents an otherwise valid PSO result from publishing a material whose
asset closure changed during admission.

Test-first source contracts were observed RED for texture upload, shader
rebuild, cold capture, and pre-publication revalidation, then GREEN after the
repair. A resource-layer unit test also retains a white revision snapshot across
a later black-texture publication and verifies the later snapshot advances its
revision. Direct `rustfmt` parsing/formatting and scoped static closure checks
pass. Managed Cargo remains stopped at the shared lockfile gate, so the unit
test is not execution evidence; WGPU, PNG, RenderDoc, timing, RSS, energy, and
power remain pending and M6 stays in progress.

## Runtime shader-stage profile evidence closure

The standalone Shader PBR viewer previously captured WPR CPU sampling, GPU
timestamps, Ready evidence, and optional RenderDoc data, but it never started
the engine's in-process profiler. The named shader spans added during this audit
therefore could not reach the matrix evidence. The profiling viewer now starts
the existing feature-gated recorder before viewer execution and exports it after
the viewer terminates. Each measured run uses a unique evidence run ID as its
profile session, writes beneath that run's E-drive `runtime-profile` directory,
and binds exactly one native timeline, hotspot report, counter-hotspot report,
and Markdown summary by path, SHA-256, and byte length.

The capture gate rejects a disabled or still-active recorder, a session or
output-root mismatch, missing retention evidence, and any overwritten frame,
span, or counter sample. The matrix consumer replays those checks from the
fingerprinted `timeline.zrtrace.json`, validates the complete retention counters
and sequence bounds, and recomputes all stage counts instead of trusting the
`run_report.json` pre-aggregation. The measured stage closure is:

- `material_requirement_admission`
- `mesh_source_build`
- `module_include_resolution`
- `template_assembly`
- `source_hash`
- `naga_validation`
- `disk_cache_lookup`
- `disk_cache_write`
- `wgpu_pipeline_error_scope_pop`

For each stage, the consumer first sums all same-name inclusive spans inside one
measured run, then applies the runtime hotspot analyzer's upper-nearest index
`ceil((n - 1) * percentile / 100)` across repetitions for p50/p95/p99/max. It
also reports run presence, total span count, and per-run span-count
p50/p95/p99/max. Different stages may be nested, so their inclusive durations
must not be added into a fabricated total compile duration. Each stage p50 is an
independent startup bottleneck candidate alongside renderer initialization,
IBL restore, shader-module creation, PSO creation, and async admission wait.
This shape is sufficient for the planned 1/100/10,000-material matrix to
distinguish per-operation latency from compile-count growth.

The integration replay exposed a separate content-identity defect before any
real profile was accepted. PowerShell canonicalized the 117-file source
manifest with `OrdinalIgnoreCase`; lower-case letters were folded to upper case,
so `project_assets.rs` sorted before
`project_asset_fixture_validation.rs`. The coordinator canonicalizes JSON keys
by original ordinal code point. Both payloads were 17,187 bytes but produced
different SHA-256 values: PowerShell `605077094e3fa3b07a008c9981249bb789de644f1659f529b44ae7af0551aa8c`
versus coordinator replay
`1443fe7f7b5e6850e06111411f83d7537df5a3f0c9d350bc6efbff9faf73528f`.
The profile contract now uses ordinal sorting, matching coordinator compact
sorted JSON. A two-key underscore/letter regression fixture pins the expected
coordinator hash `8077d3e44a4cad290a39ff3e24679c9d6f49d8d34b3cbe36effd896d41bd630a`.

Binary provenance and analysis provenance are now separate closures. The
managed build ticket continues to bind the 117 Rust/WGSL production sources
that can change the viewer binary or runtime behavior. The profile manifest
additionally binds the exact 16-file PowerShell/Python capture, identity,
validation, visual-oracle, RenderDoc replay, and summarization implementation.
The Python consumer requires that exact path set, rejects missing, additional,
duplicate, unsafe, or repository-escaping entries, and replays every byte length
and SHA-256 against the repository before accepting analysis. This prevents a
dirty-worktree tool edit after capture from silently changing the reported
percentiles or qualification result. The new Python validator is itself in the
closure. Runtime-profile artifact collection was also extracted from the
1,306-line runner into a responsibility-owned helper; the runner is now 1,201
lines and remains the orchestration owner rather than accumulating the trace
parser.

Source-level validation after the repair is: PowerShell profile contracts
`30/30`, Python summary and evidence replay `40/40`, production PowerShell
manifest writer to Python consumer acceptance/tamper cases `2/2`, PowerShell
AST parse errors `0`, critical production sources `117` with `0` missing, and
profile tools `16` with `0` missing. The synthetic
profile fixture proves only identity rejection and aggregation arithmetic; it
does not provide a Zircon WGPU timing, RSS, GPU, RenderDoc, image-quality,
energy, or power result. Managed Cargo still has not compiled the current
source snapshot, and no 1/100/10,000-material product capture has run. M6-M8
therefore remain in progress and no optimization or bottleneck-removal claim is
authorized by this infrastructure slice.

## Mesh pipeline submission-lifetime infrastructure update

The Shader09C P0-4 review followed the current source from final draw command
replay through the direct and compiled scene submit owners and the frame-level
completion pump. The decisive boundary is `PipelineAdmission::Ready`: a built
command or resolved variant is not GPU use when admission is Deferred or
Failed. The implementation therefore records immediately before the eight
Ready-side pipeline lookup sites covering Base, GBuffer, DepthPrepass, ShadowDepth,
ShadowDepthAlphaMask, Velocity, both TAA reactive targets, and OIT. Base and OIT
remain separate `PipelineCreationTarget` values even when they share one
`MeshPipelineVariantId`.

`MeshPipelineSubmissionUsage` owns three bounded structures. The current
recording is a retained-capacity `HashSet` deduplicated by `(target, variant)`.
Each variant's last-use frontier keeps only the newest ticket for one
`(device_id, device_generation, queue_class)` timeline while preserving
independent timelines. The in-flight list retains the exact variants attached
to each unresolved scene ticket. After the existing frame owner pumps backend
completion, direct and compiled rendering query only that in-flight list;
`Accepted`, `Submitted`, and unknown/error status remain fail-closed, while
`Completed`, `Failed`, `Cancelled`, and `DeviceLost` remove only the exact
ticket from each frontier. A successful scene submit binds its ticket before
any fallible post-submit finalization, so an error that retains the scene
ticket also retains its pipeline use.

The hot-path scale is `O(U * T + S + R)`: `U` is the number of distinct
actually-bound target/variant pairs in the submitted scene, `T` is the number
of independent device-generation/queue timelines for one variant (normally
one), `S` is the unresolved scene-ticket count queried after a completion pump,
and `R` is the number of associations released by terminal tickets. There is no
per-frame `O(V)` scan over all registered variants and no wall-clock or
N-frame retirement guess. One variant vector is retained per unresolved scene
ticket; the eventual profile must report its count/capacity before any storage
micro-optimization is proposed.

GPU terminal status is necessary but not sufficient because CPU owners can
still replay an old ID. A complete `MeshPipelineVariantId` use-site inventory
found 35 Rust files, including tests. The only production owner that retains a
draw payload across frames is `CachedMeshDrawCommands`, and only the compiled
path populates it. Direct rendering builds uncached frame-local commands. The
compiled path already calls `retain_generation` after command construction.
`PipelineVariantPinCounts` now updates on the cache's existing store, replace,
retain, and clear paths, so one retirement candidate can query its cross-frame
command ownership in `O(1)` without adding a second `O(C)` command-cache scan.
The existing retain pass publishes `mesh_pipeline_cpu_pinned_variant_count` for
later profile correlation. Frame-local command lists are protected by the
maintenance boundary: retirement may run only before command construction and
submission recording start, never between construction and submit.

Async Base ownership is already represented by the bounded
`PipelineAsyncCompiler<MeshPipelineVariantId, ...>::pending` set; retirement
must query that set rather than duplicate it. Material and streamer structures
carry `PipelineKey`/revision inputs but do not retain a resolved
`MeshPipelineVariantId`. Pending pipeline diagnostics retain an exact target,
variant, and module key and therefore remain an explicit retirement gate.

`PipelineShaderModuleReferences` now interns module keys as `Arc<str>`, maps
each exact `(PipelineCreationTarget, MeshPipelineVariantId)` PSO to its key, and
counts shared PSO references. All six non-Base creation owners plus synchronous
and asynchronous Base installation bind this reverse edge. A validation failure
first removes the exact PSO; it removes the WGPU module cache entry only when
that edge was the module's final reference. This matters for the two shadow and
two TAA targets, whose distinct PSOs can share one shader-module identity, and
for Base/OIT, whose equal numeric variant IDs remain distinct targets.

This infrastructure still intentionally exposes no general eviction operation.
The remaining structural closure is:

1. Define a bounded retirement-candidate policy, measured against the variant
   and PSO population. Candidate discovery must not add an unconditional
   per-frame full-registry scan or a blind N-frame destruction rule.
2. At the pre-build maintenance boundary, require command-cache pin count zero,
   async pending false, no pending diagnostic for the exact target, and an empty
   GPU last-use frontier. Unknown backend status remains fail-closed.
3. One owner may then remove every pass-specific PSO/failure/unavailable/
   background entry for the retired registry identity, release module reverse
   edges, and tombstone the registry row. IDs remain monotonic and are never
   reused; a stale payload must fail identity validation rather than alias a new
   pipeline.

The standalone Rust harness observed the missing state machine as RED, then
passed `7/7` tests for binding deduplication, same-timeline replacement,
cross-queue/device-generation frontiers, abnormal terminal release,
fail-closed status, all eight Ready lookup sites and nine target-kind contracts, and direct/compiled submit
ordering. Scoped formatting and diff checks are separate static gates. No Cargo,
WGPU, DX12, PNG, RenderDoc, timing, RSS, energy, or power run has validated this
source snapshot. This is P0-4 lifetime infrastructure, not retirement
completion or a performance result; M6-M8 remain in progress.

The follow-on CPU/module ownership harnesses separately passed `5/5` command
pin tests and `6/6` module-reference tests. They cover repeated cache entries,
same/different-variant replacement, unbalanced release rejection, no-second-scan
source wiring, shared-module final release, Base/OIT target isolation, immutable
module identity, unknown release, and all synchronous/asynchronous PSO install
sites. These are standalone pure-Rust and source-contract results, not a managed
workspace compile; the acceptance limits above remain unchanged.

## Mesh shader validation admission and reflection lifetime update

The Mesh validation worker previously parsed and validated assembled WGSL in the
background but discarded the successful reflection. Disk-cache publication,
WGPU shader-module creation, and PSO creation were allowed to proceed without
that result, so validation was diagnostic side work rather than a publication
gate. A Naga failure could therefore arrive after an invalid candidate had
already entered the driver-facing path. This was a structural admission defect,
not a reason to tune parser details.

The reference boundary is explicit. Unreal's
`Engine/Source/Runtime/Engine/Private/Materials/MaterialShared.cpp` implements
`FMaterial::HasValidGameThreadShaderMap` by requiring both a ShaderMap and
`IsCompilationFinalized()` before reporting it valid. Its render-thread ShaderMap
is a separately published retained owner. cmftStudio's `backgroundjobs.cpp` and
`backgroundjobs.h` support moving a long filter off the UI thread, but their
mutable `ThreadStatus` observation is not an immutable shader publication
contract and was not copied.

Mesh source admission now uses one exact
`(ShaderVariantKey, validation_source_identity)` state machine:

- `Missing` queues the bounded Naga job and returns typed Deferred admission;
  `Pending` remains Deferred, while Naga rejection, worker loss, and job panic
  publish typed terminal failures for that exact source identity.
- `Ready` carries `Arc<ShaderTemplateReflection>`. Only then may the disk cache
  be read or written and a WGPU module/PSO be created. Disk hits rehash their
  decoded WGSL against the same source hash, so the reflected and consumed
  source identities cannot silently diverge.
- A cached module is now a single owner of the WGPU handle plus its successful
  reflection. Required executable entries are checked by exact name and stage
  before PSO creation and again on cached-module admission. One pure program
  table covers Base, GBuffer, opaque/alpha DepthPrepass, opaque/alpha Shadow,
  Velocity, both TAA masks, and OIT; vertex-only passes do not invent fragment
  requirements and unrelated extra entries remain legal.
- A Ready artifact is transferred out of the validation map only when its module
  is actually installed. Async Base PSO queue saturation therefore retains the
  reflection and does not repeat Naga. The seven physical creation owners have
  one source gate and one cached-module gate each; six synchronous owners consume
  one Ready artifact each, while Base has one synchronous and one asynchronous
  installation path.
- Normal frame admission only drains completed validation work. The only
  blocking finish is the explicit startup prewarm/test path, so this repair does
  not turn ordinary draws into a parser wait.

The state map provides O(1) lookup and maintains O(1) pending/ready/failed/total
gauges. Its worker bound is 64 exact identities. Stable installed modules bypass
the state map; a PSO queue-full retry retains Ready reflection but can repeat the
verified disk lookup, which must be measured before adding another resident
source layer. Failed hot-reload identities also remain resident until a future
measured retirement owner exists. Neither behavior authorizes an unbounded
cache policy or an unconditional per-frame sweep.

This slice validates executable entries, not the complete bind-group ABI.
`MeshPipelineCache::construct` currently receives external scene/material/GPU
scene layouts as opaque WGPU handles, so there is no truthful owner DTO against
which to compare every reflected binding. WGPU error scopes remain the final
driver ABI gate. The follow-up must first plumb exact layout descriptors from
their owners; it must not infer a layout from duplicated magic constants.

The standalone TDD harness first failed for the absent pure program mapping and
then passed `11/11` tests covering state transitions, terminal identity,
hot-reload independence, exact-once Ready transfer, O(1) gauges, exact
stage/name checks, extra entries, specialized TAA names, and all ten program
modes. Limited `rustfmt --check`, seven-owner source inventory, production file
budgets, and scoped `git diff --check` passed. No current-source Cargo build,
WGPU run, RenderDoc capture, PNG, shader-stage profile, RSS, GPU timing, energy,
or power evidence exists, so M6-M8 remain in progress and no bottleneck-removal
or performance-improvement claim is made.

## Shader cache toolchain identity update

The next cache-identity review found that the workspace dependency declarations
allowed compatible patch upgrades from `naga = 29.0.1` and `wgpu = 29.0.1`,
while `Cargo.lock` currently resolves both packages to `29.0.3`. Runtime mesh
lookup, dynamic builtin prewarm, and the standalone asset-scan prewarmer each
encoded the literal strings `naga-29.0.1` and `wgpu-29.0.1`. The cache contract
therefore claimed an older compiler/backend identity than the code that parsed,
validated, and created modules. A patch upgrade that changes accepted WGSL or
backend translation could alias old and new artifacts under one identity.

The workspace now pins both direct dependencies exactly to the already locked
`29.0.3` resolution. One render-framework owner exports
`SHADER_VARIANT_CACHE_NAGA_VERSION` and
`SHADER_VARIANT_CACHE_WGPU_VERSION`; all three production consumers use those
constants. Its unit contract parses `Cargo.toml` and `Cargo.lock` with the
existing TOML parser and requires exact declaration, resolved package version,
and cache token to agree. This deliberately fails on a future dependency bump
until the cache namespace is reviewed and advanced in the same snapshot.

The red source contract reported five defects: two floating declaration versus
lock mismatches and three independently owned production literals. The green
contract reports exact workspace/lock identity and all three shared consumers;
scoped Rust formatting and diff integrity pass. This adds no per-frame,
per-draw, Naga, WGPU, allocation, or I/O work. It causes the intended one-time
cache namespace change from `29.0.1` to `29.0.3`; cold/warm miss counts and
compile timing still require managed product evidence. No Cargo, WGPU, PNG,
RenderDoc, GPU timing, RSS, energy, or power run was performed, so milestone and
performance status remain unchanged.

## Mesh resource-class ABI admission update

The executable-entry gate still left one structural hole: a validated entry
could declare a resource class that did not match the pipeline layout selected
for that target. `MeshPipelineCache::construct` received scene, material, and
GPU-scene layouts as opaque WGPU handles, while asset readiness exposed a much
coarser DTO. Reconstructing the driver ABI from asset constants would have
created a second, divergent owner instead of a publication boundary.

The MVP boundary now starts at the actual renderer layout owners. Scene,
forward-receiver, material, GPU-scene, and OIT owners expose the exact
`wgpu::BindGroupLayoutEntry` arrays used to create their real layouts. Mesh
construction converts those entries once into three immutable semantic
contracts: full groups `0/1/2/3`, environment-only groups `0/2/3`, and OIT
groups `0/1/2/3/4`. There is no reflection against opaque WGPU handles and no
duplicated binding-number table.

Naga reflection now classifies every reachable global as uniform buffer,
read-only/read-write storage buffer, sampled texture with dimension/sample
class/multisample state, comparison/non-comparison sampler, or unsupported.
Only resources reachable from the exact required vertex/fragment entries are
checked; unrelated helper entries and extra layout bindings remain legal.
Storage textures, binding arrays, acceleration structures, external textures,
and unsupported image shapes fail closed for the Mesh MVP. The Ready-source
gate runs before disk-cache access or WGPU module creation, and cached modules
run the same target-specific contract before reuse.

This began as a resource-class contract and is still not a false claim of
complete WGPU ABI equivalence. Buffer minimum size now has a truthful
shader-declared source and is covered by the later update below. Buffer member
byte layout, dynamic offsets, float texture filterability versus
non-filterability, and sampler operation compatibility remain enforced by the
real owner descriptors and WGPU validation/error scopes until both sides have
an exact semantic source.

Construction is `O(L)` for each of three small layout maps and occurs once per
Mesh cache. Admission is expected `O(R)` through a `(group, binding)` HashMap,
where `R` is the reachable resource count of the required entries. Stable
installed pipelines bypass reflection and contract lookup; there is no
per-frame scan of all layouts or registered variants. A managed profile must
still measure 1/100/10,000 exact variants: contract construction time and map
capacity, reachable-resource count, admission p50/p95/p99, disk/module/PSO
creation counts, peak RSS, and interface-rejection count. Acceptance requires
stable cached frames to record no new resource admission, rejected candidates
to reach neither disk publication nor WGPU module creation, and admission cost
to remain linear in reachable resources rather than total variant population.

The standalone TDD path first failed because the resource-contract module did
not exist. The current pure contract harness passes `9/9`, the existing-WGPU
type converter harness passes `5/5`, Naga reflection passes `9/9`, and the
source ownership/gate contract passes `16/16`; scoped Rust formatting and diff
integrity pass. The override-sized storage-buffer fixture is intentionally
checked as an unspecialized type-layout dependency that Naga 29.0.3 rejects for
publication, rather than bypassing validator invariants. No Cargo, product
WGPU, DX12, PNG, `D:\Tools\renderdoc`, GPU timestamp, RSS, energy, or power run
has validated this snapshot. M6-M8 and all performance claims remain open.

## Authoring layout visibility admission update

The resource-class work prompted a review of the older serializable
`RenderShaderBindingResourceType` and the new runtime reflection type. They are
not interchangeable DTOs. The authoring descriptor intentionally exposes a
coarse, stable, backend-neutral asset shape and permits an empty visibility
list as an opt-out. Runtime reflection carries access mode, texture dimension,
sample class, multisampling, and comparison state for exact WGPU admission.
Unreal keeps the same separation: `FShaderParametersMetadata::FMember` owns
base type, offset, dimensions, and structure metadata plus a layout signature;
`FShaderParameterParser::ValidateShaderParameterTypes` compares compiler
reflection against that metadata, while the RHI layout remains a later owner.
Collapsing Zircon's two types would couple asset compatibility to one backend
and was therefore rejected.

The review did expose an authoring admission bug. Material and GPU-scene
descriptors name an allowed stage set, but
`binding_has_required_visibility` accepted any declaration with one
overlapping stage. A material texture declared as `[fragment, compute]` was
therefore accepted because `fragment` matched, even though the runtime
material layout does not expose that binding to compute. The check now treats
non-empty declared visibility as a subset of the allowed set; the existing
empty-list opt-out remains legal. Field and diagnostic names now say
`allowed_visibility` and `subset`, matching the actual contract.

The algorithm remains `O(B * S)` for `B` authoring bindings and at most three
serializable stages, with no frame, Naga, WGPU, allocation, disk, or GPU work.
The RED source contract found the overlap predicate and missing mixed-stage
regression. The GREEN source contract passes `4/4`; a standalone truth-table
test passes `1/1` for empty, vertex, vertex+fragment, fragment+compute, and
compute-only declarations. Scoped formatting and diff integrity pass, and the
temporary E-drive test directory was removed. No Cargo or product validation
was run, so milestone status remains unchanged.

### Canonical material layout ownership update

The same review found a second structural risk behind the historical group-2
ABI drift: the WGPU material-layout creator and the authoring validator each
owned a separate 13-row binding table. They happened to agree in the current
snapshot, but a future clearcoat or texture-slot change could again update only
one side. The renderer now owns one static, backend-neutral material contract
with group number, binding number, resource class, allowed stage set, and
diagnostic label. Both the real WGPU layout-entry factory and the authoring
validator project that contract. Binding numbers are exposed only within the
scene module and the contract has a contiguous/unique invariant test.

This is not a merge of authoring and runtime reflection DTOs. The static table
contains only semantics that are genuinely shared by both boundaries. WGPU
still owns texture dimension, filterability, multisampling, sampler class,
buffer minimum size, and dynamic-offset policy; Naga reflection still owns
shader-declared access and resource detail. Likewise, the GPU-scene authoring
subset continues to be assembled locally, but it now consumes the existing
five draw-facing constants from the real GPUScene owner instead of duplicating
their numeric values. The full 12-binding GPUScene WGPU table remains a wider
backend contract and is deliberately not projected into Surface assets.

The material WGPU factory builds one fixed stack array of 13 entries during
renderer construction and performs no heap allocation. Authoring admission
reads a static slice; its existing diagnostic work remains `O(B * S)` and
allocates messages only on rejected layouts. This removes a drift mechanism,
not a measured runtime bottleneck, and no CPU/GPU or power improvement is
claimed. TDD first failed because the canonical owner did not exist. The pure
contract tests now pass `3/3`, the existing-`wgpu 29.0.3` projection tests pass
`3/3`, obsolete local symbol inventory is empty, and scoped formatting/diff
integrity pass. No Cargo, product WGPU/DX12, PNG, RenderDoc, timing, RSS,
energy, or power evidence was collected; M6-M8 remain unchanged.

The remaining five-row GPUScene authoring table has also moved into this
static owner. It consumes the real GPUScene binding constants while retaining
the intentional draw-facing `0..4` subset and vertex/fragment visibility
policy; the validator no longer constructs a local expected table. The
contract constructor is now owner-private, so scene consumers can read the
published rows but cannot create another competing table. This removes the
last renderer-layout table construction from material admission without
claiming a measurable CPU benefit.

### Authored pipeline-layout hard-cut inventory

A repository-wide tracked-data inventory found zero authored
`pipeline_layout` rows in `.zshader`, `.zmeta`, TOML, or JSON assets, and
schema-v2 already rejects the field with an explicit migration diagnostic.
That supports eventual removal, but the Rust field is not isolated: it is
serialized into the artifact-cache shader payload, projected into shader
readiness/management reports, copied by importer/builtin constructors, and
used by programmatic tests plus the material diagnostic opt-out. The artifact
store currently guards a versioned manifest, so deleting the payload member
without an explicit cache/schema invalidation decision would create an
unreviewed compatibility change.

The hard cut is therefore a separate cross-module migration, not part of this
scene-local owner change. Its required order is: freeze the external
readiness/management API disposition; decide artifact cache invalidation and
old-payload behavior; remove the field, framework DTO exports, material
validator, importer defaults, and programmatic fixtures in one snapshot; then
run exact cache round-trip, schema rejection, shader readiness, material
publication, and managed WGPU gates. The relevant asset/importer/framework
owners are currently modified by other Sessions, so this Shader06 Session does
not overwrite or absorb them. This is an ownership/risk conclusion, not an
acceptance blocker and not a performance result.

## Buffer minimum-size ABI admission update

The resource-class gate previously discarded one shader/layout field that is
both truthful and available before WGPU publication. The locked
`wgpu-core 29.0.3` validator computes a buffer global's minimum binding size
with Naga `TypeInner::size(module.to_ctx())`; Naga 29.0.3 defines a trailing
runtime-sized array as requiring one element. WGPU rejects a provided layout
`Some(min_binding_size)` when it is smaller than that shader minimum. A layout
`None` is intentionally different: the effective buffer range is checked at
draw/dispatch time and pipeline creation remains legal.

Reflection now publishes `min_binding_size` only for uniform/storage globals,
using that exact Naga rule during the existing validated-module pass. The Mesh
semantic layout projection preserves WGPU's optional minimum, and Ready plus
cached-module admission rejects only `Some(layout) < Some(shader)`. `None`
remains late-bound. No new Layouter, parse, validation, filesystem, WGPU, wait,
or frame-thread work is introduced. The existing type-layout hash already
contains every type input that determines the size, so the scalar is not
double-hashed and this change does not gratuitously advance the reflection hash
domain.

This is still not complete buffer ABI equivalence. Member offsets/type shape
are reflected in the shader hash but do not yet have a host-layout hash to
compare against; dynamic-offset policy has no shader-side declaration; actual
late-bound buffer ranges remain a command-time WGPU responsibility. Texture
float filterability and sampler-operation compatibility are now covered by
the sampling-pair update below. The source RED gate found five missing links.
GREEN evidence is
pure semantic contract `5/5`, existing-WGPU conversion `6/6`, Naga reflection
`9/9`, and source wiring `5/5`; the Naga cases include 16-byte uniform and
4-byte one-element runtime storage minima. Scoped formatting/diff integrity
also pass. No Cargo, product WGPU/DX12, PNG, RenderDoc, CPU/GPU timing, RSS,
energy, or power evidence was collected; P0-5 and M6-M8 remain open.

## Pipeline retirement precondition correction

The P0-4 retirement path now has real submission-lifetime evidence, but a
second architecture pass found that this is not yet sufficient for safe
capacity eviction. Mesh variant usage is attached to actual direct and
compiled submission tickets, and terminal status is available from the shared
submission coordinator. The registry still does not own an immutable pin for
the currently published material generation and its explicit last-good or
context fallback set. Nor is there a profile-derived resident capacity budget.
An LRU or age-only policy could therefore evict a valid current-generation PSO
between sparse uses and turn memory control into repeated module/PSO creation.

The required order is now explicit: publish the active material generation and
required variant set; pin current, last-good/context fallback, queued work, and
non-terminal submission users; derive capacity and pressure thresholds from
the 1/100/10,000-variant profile; then retire registry tombstones, module rows,
pass-specific PSOs, and failure/admission state as one transaction after every
use ticket is terminal. This follows Unreal's generation/ref-usage separation
and Bevy's stable pipeline identities without copying either container model.
cmftStudio's `ThreadStatus` remains only a job-lifecycle reference and is not a
GPU retirement authority. No eviction code is added in this snapshot, P0-4
remains open, and no resident-memory or power improvement is claimed.

## Texture sampling-pair ABI admission update

Resource declarations alone cannot determine whether a WGPU layout is legal.
`texture_2d<f32>` describes a float sampled texture but does not say whether
its concrete view is filterable; legality depends on the exact sampler used by
the entry point. Naga 29.0.3 already computes this operation graph as
`FunctionInfo::sampling_set`, including pairs propagated through helper
function parameters. wgpu-core 29.0.3 consumes the same pair relation when it
jointly validates shader entry resources against provided layout entries.

Validated reflection now projects each entry's exact texture/sampler binding
pair, sorts it for deterministic identity, and includes it in a v2 entry and
module resource-layout hash. The real WGPU layout projection preserves float
`filterable` and the sampler operation class (`Filtering`, `NonFiltering`, or
`Comparison`). Ready-source and cached-module admission then reject filtering
samplers paired with non-filterable float or integer textures before disk
publication or WGPU module creation. Non-filtering pairs remain legal, while
comparison compatibility continues through the exact reflected resource
classes. An unused filtering sampler does not reject an unrelated
non-filterable texture because only Naga-proven operation pairs are checked.

The implementation reuses Naga's existing validated sampling set and does not
walk WGSL IR a second time. Projection costs `O(P log P)` once per new entry to
canonicalize `P` pairs; admission performs expected `O(P)` hash-map lookups.
Stable installed pipelines perform neither operation, so there is no
per-frame, per-draw, registry-wide, filesystem, wait, or WGPU object work. The
v2 hash namespace intentionally invalidates older reflection identities whose
resource set was equal but sampling operation ABI differed.

TDD RED recorded five missing production links. GREEN source wiring is `5/5`;
standalone E-drive compilation and tests pass reflection `11/11` plus semantic
contract/WGPU projection `8/8` (`19/19` total). Coverage includes helper
argument propagation, identical resource sets with different sampling
operations producing different hashes, and filtering versus non-filtering
layout behavior. Scoped `rustfmt --check` passes and the temporary E-drive
test directory was removed. No Cargo, managed product compile, WGPU/DX12
execution, PNG, `D:\Tools\renderdoc`, CPU/GPU profile, RSS, energy, or power
evidence was collected; P0-5 and M6-M8 remain open.

## Cross-stage vertex/fragment interface admission update

The executable-entry and resource gates still left one pipeline ABI decision
to late WGPU creation: Naga validates each entry point independently, so a
required fragment input could be absent or incompatible with the selected
vertex output while both entries remained individually valid. Treating the
existing type-layout hashes as equality keys would not be correct. The locked
`wgpu-core 29.0.3` implementation intentionally accepts a downstream scalar or
narrower vector when its scalar kind matches and its width does not exceed the
producer, while matrices require equal dimensions. It also requires exact
interpolation, sampling, and `per_primitive` metadata; unused vertex outputs
remain legal.

Validated reflection now retains only the minimal numeric descriptor needed
for that rule: scalar kind/width plus scalar, vector, or matrix dimension. The
Ready-source and cached-module contract gate link the exact selected vertex
and fragment entries before resource checks, disk publication, or WGPU module
creation. User locations are already canonically sorted, so validation uses a
two-cursor merge over vertex outputs and fragment inputs. Its cost is
`O(V + F)`, allocates nothing, ignores built-ins as wgpu-core does for this
link, and is absent from stable installed-pipeline frames.

TDD RED failed only because the stage-link function was absent. The standalone
E-drive reflection harness now passes `16/16`, including legal `vec4<f32>` to
`vec3<f32>` consumption plus an unused vertex output, and rejection of missing
location, scalar-kind mismatch, interpolation mismatch, and sampling mismatch. Scoped formatting
and source-order gates pass. No Cargo, managed product compile, WGPU/DX12,
PNG, `D:\Tools\renderdoc`, CPU/GPU profile, RSS, energy, or power evidence was
collected; P0-5 and M6-M8 remain open.

## Error-proxy state correction and remaining P0-7 boundary

A current-source review supersedes the older statement that a shared error
material and binding set were still absent. `PreparedMaterial.published=None`
already selects one coherent engine-owned proxy before virtual geometry,
GPUScene, and command-cache construction: magenta custom/standard uniforms,
all six fallback textures, no material runtime payload, and the default
standard-PBR pipeline key. Previous-or-error selection is also covered at the
material-context admission boundary. Reimplementing a second error-material
path would split ownership rather than close an MVP gap.

The remaining structural risk is narrower and more important: fallback PSOs
do not yet have an explicit startup readiness deadline, and publication still
lacks exact viewport/context identity for choosing current versus last-good
generation across heterogeneous contexts. A deferred fallback PSO is reported
as `DeferDraw`, so failure is not silently converted into a partial material,
but startup readiness is not guaranteed. The context-admission material ID set
also has no demonstrated retirement boundary. No scan removal, LRU, or
capacity policy is added without the required context identity and profile.
P0-7 therefore remains open for fallback-PSO bootstrap guarantees and
cross-context generation selection, not for creation of another proxy bundle.

## Error-proxy actual-context PSO closure update

The current/previous generation admission chain had a third-layer gap. When
both real generations were unavailable it selected the coherent error proxy,
but did not derive or admit that proxy's own pass/geometry/quality PSOs. The
later mesh pass could therefore return `DeferDraw` for the fallback itself.
Cold materials made the gap less obvious: `published=None` projects the error
proxy while the sparse selection enum still says `Published`, so checking only
explicit `ErrorProxy` overrides would miss first-generation startup.

Context admission now performs an explicit third and terminal layer. It
detects the final selected proxy by the absence of a runtime bundle, covering
both explicit previous-generation failure and implicit cold publication. It
projects the default opaque Standard-PBR inputs through the same requirement
builder used by real materials, preserving actual graph features, geometry
source, shader quality, volumetric-fog policy, depth/shadow/velocity eligibility,
and cross-material deduplication. The opaque proxy cannot request OIT or either
reactive-mask target. The scan is gated by an O(1) error-selection count plus
the existing `has_active_staged_material_candidates()` index. The former covers
context-admission failures because they explicitly select `ErrorProxy`; the
latter covers implicit cold `published=None` projection. Stable frames with no
active cold candidate or explicit error do not acquire a new draw scan. Review
first caught and removed a speculative call to the nonexistent
`material_pipeline_publication_required()` API, then rejected the broader
`has_material_pipeline_admission_work()` gate because its currently unretired
context rows would keep the extra O(D) census active indefinitely.

The resulting exact requirement set is admitted synchronously. Base bypasses
background defer and can finish an already queued variant; source validation
has three bounded attempts to cover queue-full, queued, then Ready transitions;
each Ready variant completes its retained WGPU error-scope diagnostic. Any
terminal or still-deferred result becomes a `build_mesh_draws` error instead of
falling through to a fourth implicit fallback. This is a lazy actual-context
deadline, not startup cartesian precompilation. It reuses the existing
environment-only synchronous warmup mechanics and follows Unreal's high-priority
default-material PSO principle while avoiding its all-vertex-factory expansion
for Zircon's four built-in plus plugin geometry domains.

The stable-path increment is O(1). Only an error/publication context pays
`O(D + R)`, where `D` is pending draws and `R` is the deduplicated exact error
requirement count; PSO/Naga/WGPU work occurs only for missing requirements.
TDD RED was one missing-method compile failure plus four missing production
links; the post-format review added RED contracts for the speculative API and
for the compile-valid but persistently broad context gate before switching to
the existing active-staged index. GREEN is the standalone E-drive
sparse-selection harness `1/1`, source closure `7/7`, and minimal
production-file type harnesses `2/2` for requirement collection and publication
admission; scoped formatting and file-budget checks pass. No Cargo, managed
product compile, WGPU/DX12 frame, PNG, `D:\Tools\renderdoc`, CPU/GPU timing,
RSS, energy, or power evidence was collected. The concrete fallback defer gap
is source-closed, while P0-7 remains open for durable cross-context generation
identity/retirement and M6-M8 remain open.

## Generation-qualified material requirement admission update

The follow-up P0-7 review rejected viewport identity as the material readiness
owner. `ViewportRenderFrame` does not own a viewport handle, but more
importantly a viewport ID cannot prove readiness when the same viewport later
observes another geometry source, graph pass set, quality tier, or fog variant.
Unreal keeps the analogous identity on the immutable material shader map:
`FMaterialRenderProxy::GetMaterialWithFallback` accepts a material only when its
render-thread shader map is complete, while `FMaterialShaderMap::IsComplete`
checks shader/pipeline and vertex-factory layout membership. Its PSO precache is
a separate operation. Zircon remains deliberately lazy at PSO granularity, so
the corresponding owner is the Mesh pipeline cache rather than ResourceStreamer
or a viewport registry.

`MeshPipelineCache` now owns a generation-qualified admission ledger keyed by
material resource ID, immutable material `draw_generation`, and the exact
`MaterialPipelineRequirement`. The requirement identity retains creation target,
the full pipeline key, geometry source, and shader quality; volumetric fog is
already projected into the pipeline key. It does not replace exact identity with
a hash fingerprint. ResourceStreamer exposes only its three live bundle slots
`[published, previous_published, staged_candidate]`; every ledger access prunes
rows outside that tuple. This is a structural three-generation bound, not LRU or
wall-clock retirement.

Staged publication, current-context admission, and previous-context admission
all pass their actual draw generation. A requirement set is cached only after
the complete admission returns `Ready`; partial Ready rows from a Deferred or
Failed set are not published as generation-complete. Repeated camera-stack or
context observations can therefore bypass variant resolution and PSO admission
for an exact already-ready set without mixing another material generation.
The generation-less error proxy intentionally stays outside this ledger and
continues to use its synchronous terminal admission path.

For one touched material the prune bound is three generations, lookup and record
are `O(R)`, and retained storage is `O(M * 3 * R)` for `M` material IDs and exact
ready requirements `R`. Counters expose generation cache hit/miss plus retained
material/generation/requirement counts. This snapshot does not remove the old
material-level `context_admission_material_ids` census, so its persistent `O(D)`
draw scan remains and no frame-time gain is claimed. The next safe optimization
must fuse exact requirement discovery into pending-draw construction (or another
already-required traversal), then retire the material-level row; future PSO
eviction must also invalidate matching ledger requirements before P0-4 can close.

TDD RED was the missing ledger type. GREEN is the E-drive ledger harness `1/1`
and the expanded publication type harness; source-order gates require live-row
pruning before lookup and complete `Ready` before record. Managed Cargo/product,
WGPU/DX12, screenshot, RenderDoc, timing, RSS, energy, and power validation remain
pending, so P0-7 and M6-M8 remain open.

## Pending-draw census fusion and permanent-owner retirement update

The generation ledger made the old ResourceStreamer owner both redundant and
structurally harmful. `context_admission_material_ids` was inserted whenever a
staged candidate became published and had no successful-context retirement
event. Once any material entered the set, every later frame performed another
whole `pending_draws` census even when every exact generation requirement was
already Ready. Removing the set before generation-qualified lookup existed
would have been incorrect because a later geometry, graph, quality, or fog
context can introduce a requirement that was not present at publication.

Current-generation requirement discovery is now part of pending-draw
collection. The collector observes only the range appended for each mesh
instance and accumulates its exact published requirements while that range is
hot. `PendingMaterialDraw` retains the unhashed `draw_generation`; cold error
proxies have `None`, so the current-generation census does not perform a
ResourceStreamer material lookup for every draw and cannot accidentally admit
the generation-less proxy. Requirement inputs come from the already projected
pending material. In particular, shadow admission now consumes the effective
renderer/material `CastShadowsMode` rather than the material asset flag alone,
avoiding PSO work for a renderer that has disabled shadow casting.

Context admission consumes that explicit census. It first prunes the
generation ledger to `[published, previous, staged]`, removes exact Ready rows,
and performs variant resolution and PSO admission only for remaining material
generation misses. A Ready hit keeps the default published selection without
another admission. Deferred or failed misses still select the complete previous
generation and then the synchronous error proxy using the existing atomic
fallback order. The ResourceStreamer field, construction, publication write,
accessors, replacement cleanup, and submission-failure cleanup are deleted as
one hard cut; production references to the permanent owner are zero.

This changes the stable structure but does not justify a measured speedup
claim. Pending collection is already `O(D)` and the fused observer still emits
and deduplicates at most `P` requirements per draw, so its added work remains
`O(D * P)`. Generation pruning and exact-set lookup are `O(M * (3 + R))` for
the `M` observed materials and their `R` requirements. The removed costs are a
later whole-vector traversal, per-draw ResourceStreamer proxy/hash lookup, the
unbounded material-ID owner, and stable-frame variant resolution/PSO admission.
The next profile must measure the observer/dedup constant and may justify a
generation-qualified compact draw-context index; it must not replace exact
requirement equality with a lossy fingerprint. Future PSO eviction still needs
to invalidate matching ledger rows transactionally.

TDD first failed `3/3` structural contracts, then the no-per-draw-proxy slice
failed `2/4`; the final E-drive source harness passes `4/4`. Scoped formatting,
owner inventory, and diff-integrity checks pass. Windows managed validation was
submitted once, but coordinator request
`407a3c26779543bda7bad9163125c9e1` returned `command_post_timeout` after the
`cargo.acquire` submission was accepted; it is not compile evidence and was not
retried or polled. Product WGPU/DX12, PNG, `D:\Tools\renderdoc`, CPU/GPU timing,
RSS, energy, and power evidence remain uncollected. P0-7 is source-closed for
generation identity and permanent-census retirement but remains unaccepted;
P0-4 and M6-M8 remain open.

The post-change lifetime review did not authorize an eager ledger reverse
index. All seven synchronous Mesh PSO creation paths resolve the WGPU error
scope with `block_on(pop)` before returning Ready and immediately drain and
invalidate a failed target. The asynchronous Base worker also resolves the
scope before returning its installable product. Current source exposes no
capacity-eviction operation; diagnostic invalidation therefore cannot leave a
previously recorded generation Ready row pointing at a later-removed PSO.
Adding a reverse material-generation index now would have no production
consumer. It remains a hard prerequisite for the future P0-4 retirement
transaction, where the PSO removal operation must invalidate matching ledger
requirements before publishing completion.

The fused path now also publishes frame-level observed, generation-cache-hit,
and generation-cache-miss requirement counts in the same census `retain`
pass. Material hit/miss counts remain separate. This gives the next profiling
run the denominator needed to distinguish draw-context projection from actual
variant/PSO admission without a second vector scan or a per-draw profiling
scope. Timing must compare the combined collector across identical builds and
scenes; instrumenting every mesh/draw callback would perturb the hot path it is
intended to measure.

## Typed draw-context census optimization

The fused observer still performs heavyweight work before its exact requirement
set can deduplicate identical instances: each draw clones the full pipeline key,
enumerates the active pass domain, clones the key again per pass, and hashes the
same requirements. The generation ledger skips later PSO admission but cannot
remove this collector-side `O(D * P)` repetition.

The complete draw-to-PSO dependency review reduces the per-draw identity to a
typed material/generation owner, final shader geometry source, dynamic-mobility
velocity eligibility, and effective cast-shadow eligibility. Graph features,
quality, fog, disabled passes, phase, and TAA strength are candidate/census
inputs rather than instance dimensions. Indirect submission, mesh LOD, entity,
visibility, and mesh identity do not enter the created PSO key. Renderer
`receive_shadows` is written to GPUScene uniform data and does not rewrite the
actual material `PipelineKey`, so it is not an omitted context dimension.

The implemented collector deduplicates a collision-free
`MaterialPipelineDrawContext` before requirement construction, qualifies each
row by exact `ResourceId + draw_generation`, caches staged/previous candidate
inputs once per material, and shares error-proxy contexts across materials.
Candidate, contexts, and requirements share one census owner lookup; admission
consumes the stored generation instead of resolving the proxy again. A sparse
selection gate also prevents the previous-generation collector from scanning
draws when no previous proxy exists.

After the 2026-08-27 shadow raster identity repair, the complete current Mesh
domain is four shader geometry families by velocity eligibility by three exact
shadow policies: exactly 24 states. A typed enum to `u32` bitset bijection
represents this fixed set without allocation or hash collision. Construction is
`O(D + U * P + M)` with temporary `O(M + R)` state; current `U <= 24M` and the
shared error proxy `U <= 24`. No persistent viewport
index or lossy fingerprint is introduced, and scope counters derive from
container lengths rather than per-draw profile increments.

The pre-shadow-extension E-drive release-rustc equivalence microprofile first asserts identical
requirement sets. At 10,000 identical draws it reduces full-key clones from
90,000 to 9 and requirement hashes from 70,000 to 7; median model time is
16.563 ms versus 0.436 ms. At 100 materials/10,000 draws it is 17.841 ms versus
0.757 ms. The all-unique 10,000-material case is 50.272 ms versus 47.770 ms,
so the fixed-domain representation does not introduce the earlier worst-case
regression. The 24-state source has not rerun this model; these remain isolated
algorithm baseline numbers, not current product frame evidence.

TDD now passes 7 cross-file structure tests and 10 production requirement
type/behavior tests; the production file is 762 lines after test extraction.
The new managed Windows compile produced no output and timed out after 124
seconds, so no compile ticket exists. Product CPU timing, RSS, energy, power,
PNG, and RenderDoc evidence remain pending; no product bottleneck-removal claim
is made.

## Shadow raster identity review

The next source review found a structural mismatch before any further PSO
optimization. `CastShadowsMode::TwoSided` survives through `MeshDraw`, but is
collapsed to a cast/no-cast boolean in `MeshBatchRef`, the pending command cache,
and the material requirement census. The shadow pipeline then uses
`wgpu::PrimitiveState::default()`; in the locked wgpu-types 29.0.3 contract the
derived default has `cull_mode = None`. All current shadow casters therefore use
two-sided rasterization, while the renderer-level two-sided policy has no exact
pipeline identity.

Unreal keeps these inputs separate until final shadow raster-state selection.
Its `SetupShadowCullMode` uses `Material.IsTwoSided() ||
PrimitiveSceneProxy->CastsShadowAsTwoSided()` and feeds the resulting cull mode
to both live drawing and PSO precache. Zircon's MVP rule is correspondingly
frozen as material double-sided OR renderer shadow-two-sided, with no culling
only for that final state and back-face culling otherwise. `Off` remains the
command eligibility gate and `ShadowsOnly` remains a main-view routing mode.

The typed census must therefore represent three exact shadow states: disabled,
enabled one-sided, and enabled forced-two-sided. Its bounded domain becomes four
geometry families by two velocity states by three shadow states, or 24 contexts
per material generation. A fixed collision-free bitset still keeps construction
at `O(D + U * P + M)` with `U <= 24M`; the error proxy remains bounded by 24.

A separate, broader optimization remains blocked on product evidence. The mesh
variant registry retains almost the complete material `PipelineKey` for every
pass even though opaque shadow/depth/velocity templates do not require a
material surface and alpha variants require only alpha-relevant surface code.
This can multiply variants for receive-shadow, normal/PBR, and fog differences
that a shadow pass does not consume. It is not yet safe to erase those fields:
custom options may affect alpha and a future vertex-deformation contract may
affect opaque shadow position. Before pass-local identity projection, managed
profiles must correlate full keys with final source hashes, module/PSO creation,
resident bytes, and per-pass GPU work for 1/100/10,000-material scenes. RenderDoc
must verify one-sided back-face culling and material/renderer two-sided no-cull.
No current-source product timing, power, screenshot, or capture evidence exists,
so only the correctness repair is authorized at this stage.

An E-drive release-rustc identity microprofile provides an algorithm-only
baseline. With 100 material keys differing only in shadow-irrelevant feature
bits, the current full identity produces 100 rows versus an opaque raster lower
bound of 2 and takes a median 23.686 us versus 3.797 us. Ten thousand inputs
covering 256 feature combinations produce 256 versus 2 rows and 3.156 ms versus
0.311 ms. Ten thousand unique layout/option keys produce 10,000 versus 2 rows
and 4.688 ms versus 0.339 ms. The renderer `On/TwoSided` correctness case moves
in the opposite direction: current identity incorrectly collapses both to one,
while the exact final raster policy needs two. These numbers isolate registry
identity entropy; they do not establish source equivalence, WGPU creation cost,
GPU duration, RSS, or power improvement.

The authorized source repair is now implemented. `PendingMaterialDraw` preserves
the renderer-authored `CastShadowsMode` separately from material merge;
requirement census maps renderer/material inputs to Disabled, OneSided, or
ForcedTwoSided and widens only the Shadow requirement key. Static command-cache
material revision hashes the final merged mode so an On/TwoSided transition
cannot reuse stale command payload. Live and rebuilt batches project the same
forced flag, and one `effective_shadow_pipeline_key` drives variant resolution,
command payload, ensure, runtime prewarm, prewarm validation, and WGPU pipeline
creation. One-sided keys now request back-face culling; material or renderer
two-sided keys request no culling. Base, GBuffer, Depth, and Velocity keys remain
unmodified by the renderer-only override.

The source-level shadow raster contract moved from 16 expected defects to zero;
scoped formatting and whitespace checks pass. This does not close the milestone:
no managed Rust/WGPU compile ticket, DX12 timing, current RenderDoc replay, PNG,
RSS, WPR, energy, or power evidence exists. The broader pass-local variant
projection remains profile-gated, and M6-M8 stay `in_progress`.

The next non-visual infrastructure slice is pass-local PSO attribution, not key
normalization. Existing diagnostics merge all mesh/OIT registered variants,
source hashes, module creations, pipeline creations, and creation CPU time into
global totals, so they cannot identify whether Shadow inflation is necessary or
caused by pass-irrelevant `PipelineKey` fields. The measurement contract uses ten
fixed targets and updates counters only on registry insertion, exact source
observation, and actual WGPU object creation. Report reads remain fixed `O(10)`
and do not scan registry/cache/module tables or touch draw/replay hot paths.
WGPU exposes no trustworthy resident-byte value, so object counts and unique
source hashes must be paired with WPR/RSS and RenderDoc/PIX evidence rather than
an invented byte estimate. Pass-local identity projection remains forbidden
until the current-source 1/100/10,000-material matrix proves source and final PSO
state equivalence without losing the one-sided/two-sided shadow cull contract.

The attribution infrastructure is now source-complete. A fixed public target
enum owns ten Base/GBuffer/Depth/HitProxy/Shadow/Velocity/TAA/OIT rows. Registry
insertion updates a ten-element target count without rescanning variant keys;
exact source observation is target-partitioned and allocates only for a new hash;
actual synchronous, asynchronous, and prewarm WGPU module/PSO creation sites
record count and CPU microseconds against the creating target. A shared prewarm
module is charged only to its primary target, while a companion target records
its source and its own PSO; transformed OIT source remains separately attributed.

Report construction copies ten POD rows, and the diagnostic store publishes
sixty compile-time static target series without formatting paths or scanning
registry/cache/module owners. Installed-pipeline draw, cache-hit, and replay paths
do not acquire the metrics lock. Legacy serde, target order, coherent cumulative
snapshot merging, source deduplication, registry reset, and diagnostic path tests
were added; the E-drive structural contract now passes with zero missing target
observation sites, and scoped formatting/whitespace checks pass. Managed
Rust/WGPU compile, product profile, DX12 timing, RSS/WPR, resident allocation,
RenderDoc, PNG, energy, and power evidence remain absent, so this closes only the
measurement source prerequisite and does not authorize pass-local key reduction
or close M6-M8.

## Mesh Vertex Factory input ABI review

The next P0-5 source audit found one remaining late pipeline boundary. All eight
Mesh render-pipeline creators bind the same production
`GpuMeshVertex::layout()`; Velocity alone adds
`GpuMeshVertex::previous_position_layout()` at location 8. The geometry-source
descriptor's vertex-attribute and required-binding rows have no production PSO
declaration consumer, and geometry includes only replace fetch helpers. The MVP
therefore has one fixed Mesh Vertex Factory ABI, not arbitrary plugin vertex
declarations.

This matches Unreal's ownership rule: `FLocalVertexFactory` supplies both the
runtime declaration through `GetVertexElements`/`InitDeclaration` and the PSO
precache elements through `GetPSOPrecacheVertexFetchElements`. The locked
`wgpu-core 29.0.3` implementation requires every shader vertex-input location
to exist and compares scalar kind only; vector dimension and scalar width are
not part of vertex-attribute compatibility, while extra layout attributes are
legal. An exact type-layout-hash comparison would therefore be stricter than
the backend and was rejected.

The implemented contract is projected once from those production WGPU layouts:
an eight-row standard contract and a nine-row Velocity contract. Both async
reflection Ready admission and cached-module admission now reject missing
locations, unsupported or mismatched scalar kinds, and per-primitive vertex
inputs before disk publication or WGPU PSO creation. The two contracts retain
17 small rows total; admission is `O(I log A)` for `I<=9`, `A=8/9`, and installed
draw/cache-hit/replay paths do no new work. A release E-drive `rustc` harness
passes for projection/lifetime/lookup, the structural contract passes `7/7`,
and scoped formatting passes. The managed focused validation produced no output
and timed out at 124 seconds, so it yielded no compile or test ticket. DX12,
PNG, RenderDoc, timing, RSS/WPR, energy, and power evidence remain pending;
P0-5 and M6-M8 stay open.

## Async Naga source-validation measurement review

The next non-visual P0-5 slice measures a structural duplication hypothesis
without changing compiler identity. `ShaderSourceValidationKey` currently owns
the complete `ShaderVariantKey` plus a source identity containing exact WGSL
hash and diagnostic-segment provenance. Naga parse, validation, and reflection
consume only WGSL and segments; target entry/resource/vertex/link admission is a
later reflection consumer. A Ready reflection is removed after module install,
so different variants producing the same source/provenance can theoretically
repeat the worker job instead of reusing the compiler result.

The source now records cumulative queue outcomes, actual worker jobs, unique
source contracts, duplicate jobs, success/failure, queue wait, and Naga
validation CPU microseconds. The real worker body is also wrapped in the
`render/shader_pipeline/source_validation_worker` scope. Eleven static
`render.shader_variant.source_validation.*` series expose the snapshot without
scanning queues or validation-state tables. Duplicate detection runs only for
jobs that actually start, uses a borrowed lookup, and allocates an identity only
for a newly observed source contract. Its retained state is `O(S)` in unique
source/provenance identities; installed draw, cache-hit, and replay paths do no
new work.

An E-drive release-rustc lower-bound model reports variant-qualified versus
source-contract identity costs. For 10,000 requests sharing one source it keeps
10,000 versus 1 row and measures median 3.948 ms versus 1.720 ms. For 10,000
requests over 100 sources it keeps 10,000 versus 100 rows and measures 4.258 ms
versus 1.805 ms. With 10,000 unique sources both keep 10,000 rows and measure
3.793 ms versus 3.380 ms. These values include only collection identity,
hashing, and allocation; they exclude Naga, source assembly, scheduling, WGPU,
RSS, and power.

No key reduction is authorized yet. The product matrix must correlate
`duplicate_job_count` with Naga scope p50/p95/p99, queue wait/outcomes, module
and PSO creation, frame CPU/GPU, RSS/WPR, and power for 1/100/10,000-material
cold/warm runs. Exact diagnostic provenance, hot reload, target-contract
admission, and cache invalidation must remain correct before compiler-result
identity can move to the source/provenance owner. The structural contract passes
`7/7` and scoped formatting passes. The only managed focused request,
`34fd4b6732114735a6cfcc7045b0a231`, was accepted but `cargo.acquire` returned
`command_post_timeout` during post-response reconciliation, so it produced no
compile/test ticket and was not retried or polled. Managed Cargo/WGPU, DX12
product profile, PNG, RenderDoc, energy, and power evidence remain absent. P0-4,
P0-5, and M6-M8 stay open.

## Mesh fragment-output and attachment ABI review

The remaining Mesh publication audit found that entry, vertex input,
vertex/fragment link, resource, buffer-size, and sampling-pair admission still
did not prove that a fragment output was numerically compatible with the real
pass color attachment. An otherwise valid shader could therefore reach WGPU
pipeline creation before a uint/float or component-width mismatch was rejected.

The locked `wgpu-core 29.0.3` rule is intentionally asymmetric. It iterates
shader outputs and checks only locations backed by a present color target. The
target numeric type must be a subtype of the shader output: identical scalar
kind, no wider target scalar, and no more target components than the output.
Unpaired shader outputs and targets are legal. This preserves the existing
Velocity `vec4<f32>` output to `Rg16Float`; an exact vector-size check or a
requirement to write every MRT would be a false rejection.

Unreal keeps the same ownership separation at a larger scale:
`ShaderMaterialDerivedHelpers.cpp` derives `PIXELSHADEROUTPUT_MRT0..6` from the
pass/material domain, while `GBufferInfo.cpp` owns semantic-to-target/channel
packing. Zircon now keeps compiler-reflected outputs separate from pass-owned
attachment formats and compares them only at pipeline publication admission.

Mesh construction projects six immutable contracts from the dynamic Base
format, four existing GBuffer format owners, three HitProxy format owners, and
single Velocity/TAA owners; Depth, both Shadow targets, and OIT share an empty
color contract. Velocity and TAA creation APIs no longer accept a caller format,
so runtime, prewarm, validation, and admission cannot drift. Ready-source and
cached-module paths reuse the same gate before disk publication or WGPU object
creation and report mismatches through the existing `ShaderInterfaceMismatch`
owner.

Only ten target rows are retained. Construction costs `O(T log T)` and a new
fragment entry costs `O(O log T)`, with current `O<=4` and `T<=4`; installed
draw, cache-hit, and replay paths do no new work. The structural test moved from
RED to `9/9`, and an E-drive release-rustc harness directly compiling both
production contract files passes HitProxy, wide Velocity output, and empty OIT
cases. Scoped formatting passes. Managed Cargo/Naga/WGPU, DX12 product frames,
PNG, RenderDoc, CPU/GPU timing, RSS/WPR, energy, and power remain absent. The
only managed focused request, `8f3403a7fa914865bdd54af59707754b`, was accepted
but `cargo.acquire` returned `command_post_timeout` during post-response
reconciliation, so it produced no compile/test ticket and was not retried or
polled. P0-5 and M6-M8 stay open.

## GPU Scene dynamic palette minimum owner review

The resource-contract follow-up found a false-negative introduced while
projecting the opaque GPU Scene bind-group layout. Mesh construction supplied
`Some(1)` only because the shared layout factory requires a non-zero skinned
palette size, then preserved that placeholder as if it were the live layout's
explicit minimum. Bindings 3 and 4 are production
`array<mat4x4<f32>>` storage buffers, so their reflected minimum is at least one
64-byte stride and every reachable skinning entry could be rejected before
WGPU saw the real layout.

WGPU distinguishes an explicit pipeline-time minimum from a `None` minimum
whose actual bound range is checked later. The real GPU Scene layout still owns
its palette capacity; the Mesh reflection contract cannot recover that value
from the opaque layout object. Its projection now clears only the two palette
placeholder minimums, using their exported owner binding constants. All other
explicit layout minimums remain strict. Unreal's
`FShaderParametersMetadata` / `FRHIUniformBufferLayout` and
`RHICreateUniformBuffer` / RHI validation likewise separate structural shader
metadata from the actual resource instance and its validation lifetime.

This adds at most 24 binding comparisons during cache construction and no
source-admission, worker, installed-pipeline, or draw-path work. The source gate
moved from RED to `4/4`; a source unit regression locks both bindings to
late-bound `None`. An E-drive release-rustc harness directly using the
production `MeshShaderPipelineLayoutContract` proves that fabricated `Some(1)`
rejects a 64-byte shader requirement while `None` preserves the intended WGPU
boundary. Scoped formatting passes. The change is newer than managed request
`8f3403a7fa914865bdd54af59707754b`, which already produced no compile/test
ticket, so it does not validate this snapshot. No request was added or polled;
Cargo/WGPU/DX12, PNG, RenderDoc, timing, RSS/WPR, energy, and power remain
absent. P0-5 and M6-M8 stay open.

## P0-7 context correction and fallback queue-debt review

The latest end-to-end admission review supersedes the older statement that a
persistent viewport/context identity is still required. Current publication is
already qualified by material resource ID, immutable draw generation, and the
exact requirement containing target, full pipeline key, geometry source, and
quality; fog is part of that pipeline key. Typed pending-draw contexts discover
new requirements for a later graph, geometry, quality, fog, velocity, or shadow
combination, and the complete-Ready ledger retains only the published,
previous-published, and staged generations. A miss selects the complete
previous bundle or complete engine error proxy. Runtime, textures, both uniform
resources, and generation are projected from one immutable bundle, so a new PSO
cannot be paired with an old binding bundle. A separate context ID would be a
second, weaker readiness owner rather than a correctness repair.

The remaining startup problem is scheduler ownership. The error proxy's three
source-admission attempts are the minimum worst-case transitions for a saturated
bounded FIFO: Full then drain, Queued then finish, Ready. Correctness converges,
but the current finish call waits for all pending validations, so the first
fallback draw may pay for up to 64 unrelated Naga jobs. The generic compiler's
target-through wait cannot solve the initial Full state because the required key
has not entered the queue. Unreal keeps the analogous concerns separate:
`MaterialRenderProxy.cpp:865` selects a complete fallback shader map and submits
the missing material's jobs, while `PSOPrecacheMaterial.cpp` tracks request IDs,
completion prerequisites, and raises only draw-required requests to Highest
priority rather than draining all compile work.

No queue algorithm is changed without product evidence. This slice makes the
existing debt measurable: a non-empty error-proxy admission publishes total
attempts, sync count, actually completed jobs, sync wait microseconds, and
queued/pending/saturated reason counts. The finish API returns its existing
worker completion count; no queue scan, new worker, lock, collection, Naga job,
or WGPU object is added. Empty requirement sets return before the seven scalar
counters, and ordinary installed draw/cache-hit/replay paths do no new work.
Failure exits publish the same sample so results are not success-biased.

The decision matrix is 0/32/64 pre-existing validation jobs against actual
static and skinned fallback contexts, at least 30 cold processes per case. The
new counters must be correlated with the global validation queue/job/duplicate/
wait/Naga-CPU metrics, first-present CPU/GPU, RSS/WPR, and energy/power. A shared
compile-service priority/reservation change is authorized only if unrelated
completed jobs and fallback sync wait dominate p95; target Naga or WGPU PSO
cost instead routes to required-domain prewarm/cook. Any later scheduler change
must prove that unrelated later work remains pending and queue-full admission no
longer requires a full drain.

The source gate moved from missing five core debt series to `7/7`; scoped
formatting and whitespace checks pass. No managed validation request was added
because the existing accepted request has no compile/test ticket and predates
this snapshot. DX12/WGPU product execution, PNG, `D:\Tools\renderdoc`, timing,
RSS/WPR, energy, and power remain pending. Cross-context generation selection is
source-closed; P0-7 product acceptance, P0-4 retirement, and M6-M8 remain open.

## Runtime WGSL disk-cache measurement boundary

The P0-2 current-source recheck confirms that the corrected source-cache
identity reaches the real Mesh runtime consumer. Lookup follows template
assembly, final WGSL hashing, and asynchronous Naga reflection. Its expected ID
contains the complete shader variant key plus final WGSL hash, ordered include
content hashes, template revision, and the exact Naga/WGPU versions locked by
the workspace; metadata must match every field and the decompressed payload is
hashed again before a hit is accepted. This source boundary can no longer
silently accept stale WGSL, but it does not avoid assembly, Naga, device-module,
or PSO creation.

The remaining question is architectural rather than a compression tweak: does
runtime compressed-WGSL I/O have any net value after those costs were already
paid? The former coarse lookup/write scopes could not distinguish key building,
metadata I/O/decode, payload I/O/decompression/rehash, or write-side hashing and
commit. Eleven fixed static scopes now wrap exactly those existing operations.
They add no I/O, hash, allocation, collection, worker, Naga job, WGPU object, or
draw/cache-hit/replay work, and retain the existing coarse scopes and byte
counters.

The managed decision matrix remains 1/100/10,000 variants across cold, warm,
and reload runs, with at least 30 cold processes per case and an E-drive work
and cache root. These stages must be correlated with Mesh source build/hash,
Naga worker wait/CPU, per-target module/PSO creation, first-present CPU/GPU,
RSS/WPR, and energy/power. Runtime lookup may be optimized or retained only when
warm p95 is lower than work it demonstrably avoids and Ready/first-present/RSS
show repeatable net benefit. If it still avoids no compiler or device work and
its disk stages dominate, the P0-1/P0-3 shared artifact service should remove or
demote runtime lookup instead of tuning zstd locally.

The TDD source gate moved from `0/11` to `11/11`. Scoped `rustfmt --check` and
`git diff --check` pass; Git emitted only LF/CRLF notices and no whitespace
error. No coordinator validation request was added, and no Cargo/WGPU, DX12
product frame, PNG, RenderDoc, WPR, energy, or power evidence was collected.
Generated-WGSL source-cache identity is
`source_closed_pending_managed_validation`; full artifact/target/device/PSO
identity and the runtime cache performance decision remain open, as do M6-M8.

## RHI programmable-stage contract and PMREM mapping disposition

The current-source P0-1 review supersedes the old claim that Zircon has no
production RHI implementation. `zr_rhi_wgpu::production::WgpuRenderDevice`
now owns real native shader/pipeline registries, device generation, and
submission-last-use retirement. Product `SceneRenderer` has not cut over to
that authority: a focused production inventory still finds 48 render-pipeline
creation calls in 45 files, 15 compute-pipeline creation calls in 15 files,
and 67 shader-module creation calls in 61 files.

The neutral descriptor also cannot represent the Mesh pipeline domain without
duplicating artifacts. `ShaderModuleDesc` owns source, stage, and entry point,
while `PipelineDesc` stores only shader handles; the production WGPU creator
therefore reads each selected entry point back from the module descriptor. Mesh
uses one assembled WGSL module with multiple entries selected by Base, GBuffer,
Depth, HitProxy, Shadow, Velocity, TAA, and OIT PSOs. Migrating that path as-is
would create entry-qualified duplicate module handles/native modules for one
source and leaves no pipeline-owned specialization-constant identity.

Unreal's `FGraphicsPipelineStateInitializer.BoundShaderState` and
`PipelineStateCache` keep shader artifacts/RHI shader objects separate from the
complete PSO initializer. The Zircon target is consequently a source-only
`ShaderModuleDesc`, plus pipeline-owned programmable-stage records containing
module handle, entry point, and specialization constants. Exact `PipelineId`
must additionally cover layout, vertex declaration, attachments/depth/sample,
raster/blend, device capability profile, and generation. Source/module and PSO
deduplication are distinct identities. The RHI crates currently contain broad
foreign tracked/untracked work, so this Session records the hard-cut contract
without editing those owners or introducing another SceneRenderer-local
pipeline service.

The accompanying PBR audit rejects a roughness-to-mip change. Unreal uploads
`FloorLog2(capture size)` as the maximum mip index and intentionally maps
roughness 1 to the third mip from the end with the 1.0/1.2 heuristic. Its
capture filter also switches to cosine importance sampling above roughness
0.99 because GGX is constant there. Zircon's canonical CPU recipe, CPU and GPU
PMREM producers, and runtime WGSL share those exact semantics. cmft's complete
mip-chain glossiness/specular-power convention is a different asset contract
and must not be mixed into this recipe. This is a retain decision backed by
source review, not managed WGPU, image, RenderDoc, timing, memory, energy, or
power acceptance; P0-1/P0-3/P0-4/P0-5 and M6-M8 remain open.

## Authored pipeline-layout DTO migration contract recheck

The tracked authoring-data inventory remains empty: no `.zshader`, `.zmeta`,
TOML, or JSON asset declares `pipeline_layout`, and `.zshader v2` already
rejects that authored field. The only production read chain for the retained
Rust DTO is `resource_streamer_ensure_material` into
`renderer_material_layout_diagnostics`; an empty descriptor explicitly opts
out. `ShaderAsset::pipeline_layout_descriptor()` has no production caller,
no WGPU layout factory consumes the DTO, and its string push-constant rows
have no execution consumer. It is therefore a legacy diagnostic projection,
not a valid reflection or pipeline-layout authority.

The exact hard-cut boundary is wider than the asset struct. Shader artifact
payloads are sequential bincode records, and their owner explicitly forbids
assuming `serde(default)` or skipped fields preserve compatibility. The
current manifest is `ZRARTM06`, schema v6, and is checked before payload
deserialization. Removal must advance that manifest identity so old payloads
fail closed, then remove the asset member/accessor, cache member, readiness and
management layout fields/counters, framework DTO/export, importer/builtin
defaults, material validator, and programmatic fixtures in one snapshot. Its
gates are old-v6 rejection, new cache round-trip, authored-field rejection,
Surface readiness/material publication, and specialized-reflection versus
actual pass-layout WGPU admission.

This Session does not perform that migration because the asset/readiness and
material-validator owners already contain other Sessions' unowned edits.
Overwriting those changes would be less correct than deferring the explicit
schema hard cut. This closes the migration inventory only; it does not close
P0-5 and provides no compile, product, visual, timing, RSS, energy, or power
evidence.

## Live-generation resolved-pipeline pin prerequisite

The P0-4 lifetime review found one missing edge after submission-ticket and
compiled-command ownership were added. The generation admission ledger kept
authored `MaterialPipelineRequirement` rows but discarded the exact
`(PipelineCreationTarget, MeshPipelineVariantId)` selected by normalization.
A future retirement candidate could not prove in O(1) that a current,
last-good previous, or staged material generation still referenced its PSO.
Re-resolving during retirement or scanning every material generation would
duplicate identity ownership. Unreal likewise keeps material shader-map
completeness separate from PSO precache/residency; this change records the
reference edge without turning the material ledger into another registry.

Each live generation now owns both its authored requirement set and a resolved
pipeline set. The existing cache-miss admission traversal collects resolved
rows once; only a completely Ready set is published. Duplicate observations
within one generation are set-deduplicated, while sharing across generations
or materials increments one reverse count. Pruning to the ResourceStreamer's
three live slots decrements every retired row and removes the exact pin only
after its last generation leaves. `PipelineCreationTarget` stays in the key,
so Base and OIT remain isolated even when their numeric variant IDs match. The
new `material_generation_admission_pinned_pipeline_count` exposes the number
of unique exact PSOs with material-generation owners.

Stable generation hits retain the existing requirement-membership path and do
not allocate the resolved vector, resolve a variant again, parse Naga, create
WGPU work, or scan the registry. A miss preallocates one `R`-row vector and
performs at most `R` set/map updates. Pruning visits at most three generation
rows for the touched material plus the resolved rows that actually leave.
Storage is `O(M * 3 * (R + V) + U)`. A material removed from ResourceStreamer
and never observed again can still leave a conservative stale pin; this blocks
reclamation rather than permitting unsafe eviction. Material-removal cleanup
must therefore enter the same unpin owner before any capacity policy is added.

The source contract moved from `0/4` missing production links to `8/8` closed.
An E-drive release `rustc` harness directly included the production ledger and
passed same-generation deduplication, shared-generation `2 -> 1 -> 0`
reference counts, Base/OIT isolation, and final row removal. Scoped formatting
and whitespace/conflict-marker checks pass. No Cargo, WGPU/DX12 product, PNG,
RenderDoc, timing, RSS, energy, or power evidence exists, and no LRU,
capacity threshold, tombstone, or eviction operation was added. P0-4, M6-M8,
and the realtime-SH9 Failure remain open.

## Environment-only dielectric F0 consumption closure

The F2 consumption-chain review found one basic material-ABI divergence. Full
and basic Standard-PBR Forward already pass `surface.dielectric_f0` into the
split-sum environment BRDF, while the environment-only provider injected a
constant `vec3(0.04)` inside its component function. Selecting the lean
environment-only profile therefore removed authored dielectric reflectance in
addition to intentionally removing direct-lighting modules. Unreal's
environment BRDF consumes the material/GBuffer `SpecularColor`; 0.04 is a
fallback for an unavailable channel, not a replacement for an available
material value. cmft owns convolution-side PMREM data and supplies no contrary
material-F0 contract.

The specialized component and indirect entry now take dielectric F0 explicitly.
Environment-only Forward supplies `surface.dielectric_f0`, and the shared
`zr_pbr_material_f0` plus BRDF LUT remain the sole evaluation owner. The current
deferred GBuffer has no separate dielectric-F0 channel, so its environment-only
preview supplies an explicit `vec3(0.04)` at that lossy boundary. A future
GBuffer extension must migrate encode, decode, layout, and this call together;
the shared BRDF must not regain a hidden profile-wide constant.

This preserves per-pixel `O(1)` work. The production delta is one `vec3<f32>`
argument: zero added texture samples, branches, loops, normalizations, probe
iterations, shader permutations, Naga jobs, WGPU objects, or disk-cache
identities. The focused RED source contract moved from `0/4` to `4/4`, and the
Rust source guards pin the Forward authored value and Deferred explicit
fallback. Scoped `git diff --check` passes. There is still no managed shader
assembly/Naga/WGPU ticket, DX12 product frame, PNG, `D:\Tools\renderdoc`
capture, register/occupancy data, CPU/GPU timing, RSS/WPR, energy, or power
evidence. This F2 defect is `source_closed_pending_managed_validation`; M6-M8
and the realtime-SH9 Failure remain open.

The follow-up routing audit does not yet remove the old non-default-IOR
exclusion. `MeshPipelineVariantKey::new` still suppresses the specialized bit
for `pbr_ior_override`, and the warmup owner enables the environment-only
profile only for default IOR. Rendering remains correct through generic
Forward, but the old fixed-F0 rationale is gone and the IOR viewer may compile
and retain a larger source/receiver PSO than necessary. Because those owners
contain overlapping work and no current generic-versus-specialized product
profile exists, this is recorded as a startup-routing hypothesis rather than
implemented as an optimization. The required comparison binds identical IOR
pixels to exact assembled bytes, Naga/module/PSO CPU, first-present CPU/GPU,
RSS/WPR, and energy before the route and Ready schema can change together.

## glTF flat-normal and MikkTSpace TBN infrastructure re-review

The full tangent-frame review rejects a pixel-only green-channel conversion.
glTF defines `bitangent = cross(normal, tangent.xyz) * tangent.w`, and both
Zircon material paths use that same frame. Flipping glTF normal-map Y without
also migrating the tangent-basis handedness changes the physical normal. Raw
glTF pixels, authored `tangent.w`, and the current shader cross formula are
mathematically consistent; the fact that normal-convention metadata does not
yet participate in runtime shader identity is a separate contract debt.

The proven defects are earlier in mesh preparation. Missing glTF normals must
be flat, while the importer currently accumulates shared indexed vertices into
smooth normals. Missing tangents for a normal-mapped primitive should use
MikkTSpace and the normal texture's effective UV set, while the importer emits
`[1, 0, 0, 1]`. The shared `MeshAsset` tangent generator is also a simple
triangle sum rather than MikkTSpace. Unreal routes mesh NTB construction
through `ComputeTangentsAndNormals` plus `UseMikkTSpace`; Bevy duplicates
indexed vertices for flat normals and then invokes one shared Mikk owner.
Zircon must converge on the same ownership instead of adding a glTF-only
tangent implementation.

The implementation order is shared infrastructure first: migrate
`MeshAsset::try_generate_missing_tangents` to `bevy_mikktspace` v1 and expose
UV0/UV1 selection; then make glTF resolve normalTexture `texCoord` including
the `KHR_texture_transform.texCoord` override, expand every vertex and morph
stream when flat normals are required, ignore invalid authored tangents in
that case, and generate missing Mikk tangents from the selected UV set.
Authored normals and tangents remain untouched. UV channels beyond the current
two-channel GPU ABI fail closed rather than silently sampling UV0.

All work is import/cook-only: zero additional runtime samples, branches,
permutations, Naga jobs, WGPU modules/PSOs, or frame scans. Flat expansion is
`O(I)` and only applies to primitives without normals. Mikk work scales with
faces/corners/vertices and publishes one 16-byte tangent row per vertex; its
internal working set and import CPU still require managed 1/100/large-mesh
cold/warm profiling. Correctness gates cover indexed hard edges, morph/skin/
UV/color remapping, UV1 normal maps, mirrored-UV handedness, authored tangent
preservation, and unsupported-UV rejection. No product image, RenderDoc,
timing, RSS, energy, or power acceptance is claimed yet.

### 2026-08-28 source implementation status

The shared implementation now uses pinned `bevy_mikktspace` 1.0.0 through a
single `MeshAsset` owner. The existing UV0 entry remains compatible, while a
new explicit-UV entry accepts UV0 or UV1. The adapter follows Bevy's current
face-corner geometry contract and flips the generated sign once so Zircon's
right-handed `cross(N, T) * tangent.w` reconstruction remains consistent.
The former per-triangle tangent/bitangent accumulator and its fallback-vector
math were removed rather than retained as a second algorithm.

The glTF importer now selects flat missing-normal policy while OBJ retains its
smooth policy. Flat glTF primitives allocate exact `I`-row vertex/index
outputs, copy color/UV0/UV1/joint/weight data by the original corner index,
ignore authored tangents that have no authored normals, and feed the expanded
mesh into VG and SDF cook. Morph POSITION/NORMAL/TANGENT streams are remapped
by the same original index list before MeshAsset validation. Missing tangents
are generated only when a normal texture exists; its effective texcoord uses
the shared `KHR_texture_transform` projection. Missing UV data and UV channels
above 1 fail closed.

Focused source tests cover selected UV1 generation, mirrored-UV handedness,
hard-edge flat expansion, morph remapping, and UV2 rejection. `rustfmt`,
scoped `git diff --check`, and read-only `cargo metadata --locked --no-deps`
pass; metadata resolves the exact 1.0.0 dependency and keeps workspace/target
paths on E. Managed validation job
`149f8578c42a4166b6392aba1a4cf3b0` was durably accepted under
`D:\cargo-targets\verify` for `cargo test --locked -p zircon_runtime --lib
tangent`, but its source and tests changed after submission. Replacement job
`bc7e5000f4ed43b5a5575ecb1d9c6c82` was also durably accepted and last
observed only as `materializing`; the importer-owner convergence below later
changed that exact snapshot too. Both tickets are stale for the current tree,
are not polled, and cannot count as compile/test evidence.

This source snapshot is not an accepted milestone. `Cargo.lock` and
`zircon_runtime/Cargo.toml` still have executable foreign ownership, so their
correct additive dependency rows cannot enter a scoped candidate until that
ownership is reconciled. No product frame, PNG, RenderDoc capture, measured
import CPU, peak RSS, WPR energy, or power result exists. P0-1/P0-3/P0-4/P0-5,
M6-M8, and the realtime-SH9 Failure remain open.

## glTF geometry authority and remaining tangent-frame boundaries

The product-route audit invalidates the earlier assumption that repairing the
builtin importer closes the glTF path. `gltf_importer.gltf` registers at
priority 120 while `zircon.builtin.model.gltf` registers at priority 10. The
stable plugin therefore owns normal product imports, but it maintained a
second primitive builder: missing normals were smoothed, authored tangents and
colors were dropped, default tangents remained, Virtual Geometry was cooked
unconditionally, and root Model plus Mesh subasset both retained geometry.
Runtime93 already records this split as `MESH93-P1-11`; Shader06 must consume
one runtime-owned projection rather than patch both copies.

The current source moves indexed-triangle validation, smooth/flat normal
policy, face-corner expansion, attribute preservation, VG/SDF request gating,
and primitive cooking into one `asset::importer` projection. Builtin OBJ/glTF
and the stable glTF plugin consume that owner. The stable path now reads
authored tangent/color, applies the same flat-normal policy, resolves the
normal texture's effective UV set, invokes the shared MeshAsset Mikk owner,
and publishes geometry only in the Mesh subasset; root/mesh Model primitives
retain references rather than a second payload. Default import settings now
perform zero optional VG cook instead of eagerly building it. This removes one
algorithm copy and one product geometry duplication source; it does not close
the full builtin/plugin decode, animation, texture, or material authority.

The original two design boundaries have since been separated into one product
oracle and one source-closed material path. The Mikk/morph source slices still
require managed compile/test validation before any milestone claim:

1. `KHR_texture_transform.texCoord` correctly selects UV0/UV1 for Mikk, while
   the affine transform itself remains sampling-only. Khronos defines the
   transform as `translation * rotation * scale`; its official Sample Renderer
   keeps authored vertex TBN and applies only the transformed sampling UV.
   Zircon therefore must not bake the affine transform into Mikk as a second
   basis transform. Rotation and negative-scale product pixels still require
   the Khronos TextureTransformMultiTest normal-row oracle before acceptance.
2. `clearcoatNormalTexture` source projection is now complete through its own
   texture-slot texCoord/transform/scale metadata, asset/runtime/GPU packing,
   and Standard-PBR sampling. Missing tangent admission is shared by both
   importers. Different base/coat UV sets remain a product-image gate rather
   than an unresolved source ABI; required factor/roughness texture support is
   still intentionally rejected as a separate advanced-material capability.

The shared MeshAsset owner now retains Mikk's face-corner output, performs the
minimal `(source vertex, tangent bits)` render-vertex split transactionally,
and remaps every base/morph stream through the same projection. It also rebuilds
missing morph target flat normals and target Mikk tangents as deltas relative to
the completed base frame, rejecting handedness changes or target corner groups
that the base split cannot represent. VG ordinals/pages and Mesh SDF source
hashes are cooked only after this final geometry projection. These are source
contracts; their managed compile/test and product evidence remain pending.

The convergence is import/cook-only. Runtime samples, shader branches,
permutations, Naga jobs, WGPU modules/PSOs, and frame scans remain `+0`.
Projection is `O(V + I)`, flat expansion is exact `I` rows, and eliminating
the root geometry copy removes one full `O(V + I)` product payload for the
stable path. These are algorithmic bounds, not measured CPU/RSS/power data.
The new product-route fixtures and shared projection still require a fresh
exact managed compile/test snapshot, followed by cold/warm import profiling
and the normal-map image/RenderDoc gate. No main-plan status changes are
authorized by this source record.

### 2026-08-28 Mikk face-corner projection review and source implementation

The adapter defect is confirmed against both APIs rather than inferred from an
image. `bevy_mikktspace::Geometry::set_tangent` returns a value for a particular
`face, vert` pair. Unreal's `MikkSetTSpaceBasic` stores it at
`FaceIdx * 3 + VertIdx`; `StaticMeshBuilder::BuildVertexBuffer` later compares
the complete pending vertex, including tangent basis and UVs, before reusing an
existing render vertex. The old Zircon adapter mapped the callback through the
source index immediately and therefore discarded a representable wedge split.

The reviewed shared-owner correction is import/cook only:

1. Generate and retain exactly `I` corner tangent rows for an indexed triangle
   list (`V` rows for an unindexed list).
2. Project each `(source vertex, tangent bits)` group to one render vertex.
   Reuse the original vertex for its first group and append only additional
   groups; remap every base and morph attribute by the same source ordinal.
3. Promote U16 indices to U32 only if an appended vertex exceeds U16 range.
4. Make the projection transactional, and reject a morph target if its generated
   handedness changes relative to the completed base `w`, because glTF morph
   tangent deltas contain xyz only.

The implemented projection publishes `V + S` rows, where `S` is the number of
additional tangent groups and `0 <= S <= I - V_referenced`; a normal indexed
mesh with one group per vertex keeps `S = 0`. Core grouping is expected
`O(V + I)`. Only the `S` appended rows are copied across base/morph attribute
streams, so additional attribute work and storage are `O((A+M) * S)` instead of
copying all `V + S` rows whenever one split exists. This replaces silent
last-corner-wins behavior without unconditionally expanding every normal-mapped
mesh to `I` vertices.
The exact tangent-bit key does not invent a second approximate grouping rule:
bevy_mikktspace first creates deterministic orientation/subgroups internally and
writes the same completed tangent value to their member corners. Canonicalizing
only signed zero reconstructs those already-decided groups. Adding a second
epsilon merge here could incorrectly join distinct Mikk groups.
The base/morph projection is transactional. Optional Virtual Geometry and Mesh
SDF derived data now cook after the split, so VG vertex ordinals/pages and the
SDF source hash describe the final render geometry rather than a stale pre-split
snapshot.

Cold/warm import CPU, peak RSS, split ratio `S/V`, output hash, product pixels,
RenderDoc, WPR energy, and power remain measurement gates. This record authorizes
the current source snapshot but is not validation or milestone acceptance. The
managed ticket `6032100d054f4e05bc97b2ad75ec231e` predates the final derived-data
cook-order changes and is therefore stale; it cannot provide compile/test
evidence for this snapshot.

### 2026-08-28 normal convention production-route correction

The second source review invalidates the earlier canonical-DX conclusion. Bevy
applies the same Mikk `w` correction as Zircon and reconstructs
`B = cross(N, T) * w`; with that right-handed basis, glTF's raw +Y texels require
no green flip. The current glTF product path is therefore correct by behavior,
but its DX descriptor label is false. The standalone owner is reversed as well:
it leaves declared DX input unchanged and flips declared GL input before
publishing DX, so it changes the physical normal in the wrong direction.

The runtime canonical representation must be right-handed/GL. Declared DX input
is converted once at cook time, glTF publishes GL explicitly without changing
pixels, and both shader paths consume canonical GL without a per-pixel convention
branch or permutation. This is a coordinated source migration: shared include,
standalone/glTF descriptors, importer versions, cache invalidation, and host tests
must change atomically. The current source does not yet implement that migration
and cannot count as normal-map acceptance.

### 2026-08-28 clearcoat normal texture-slot source convergence

The missing clearcoat behavior was traced through the complete material path
before changing the ABI. Group-2 bindings 11/12 and the conditional clearcoat
normal sample already existed. The loss occurred earlier: both glTF importers
discarded `clearcoatNormalTexture.texCoord`, `KHR_texture_transform`, and
`scale`, while the shader reused the base-normal UV. A shared extension
projection now owns factor/roughness defaults and the independent normal
texture metadata; builtin and priority-120 stable importers feed the same
texture-slot descriptor, which survives asset, runtime, uniform, and shader
projection. Base and coat samples use their own UV selection/affine transform
but the same glTF vertex TBN. The transform is not baked into Mikk.

The required-extension gate remains intentionally strict. Clearcoat factor and
roughness textures still have no channel/slot owner, so optional use reports
diagnostics for those fields and required `KHR_materials_clearcoat` remains
rejected. This avoids publishing partial support as full conformance.

The 256-byte standard uniform and 288-byte bindless row do not grow. Five UV
selector floats are replaced by one exact six-bit mask, and dielectric F0 is
stored once instead of as three equal channels. Those eight reclaimed scalars
carry clearcoat scale/offset, precomputed `(cos, sin)`, UV selection, and normal
scale. A direct two-vec4 extension would cost 32 bytes: +12.5% for the uniform
and +11.11% for the bindless row. The current source adds 0 bytes, 0 bindings,
0 samples, and 0 permutations. Material packing adds one `sin_cos` per rebuild;
the clearcoat pixel path adds one UV bit test and one affine transform around
the existing sample. No measured GPU instruction, timing, RSS, WPR, energy, or
power claim is made until managed Naga/WGPU and product captures complete.

The geometry admission was reviewed separately because glTF exposes only one
vertex tangent basis. Khronos requires a clearcoat-only normal map to have
authored `NORMAL` and `TANGENT` unless the base material also has a normal map;
when both maps exist they should use the same texture coordinates because they
operate in that one tangent space. The shared builtin/stable importer owner now
validates both referenced UV attributes. If tangents are missing, the base
normal map's effective UV remains the sole Mikk input. A clearcoat-only normal
map without authored tangent space fails during import instead of retaining a
default tangent. Different base/coat UVs remain representable with an authored
or base-derived basis, but require a product image oracle because the extension
only recommends, rather than mandates, matching coordinates. This admission is
import-time `O(1)` and adds no runtime work.

The full clearcoat shading path was then checked against the normative Khronos
layering model and the more advanced Unreal/Bevy implementations. Applying one
`1 - clearcoat * Fresnel(NoV)` weight to the complete base material is the
intentional KHR simple-layering contract; Unreal/Bevy apply additional
transmission to selected base specular terms, but that is a different material
model and must not silently replace glTF semantics as a presumed optimization.
The complete binding trace disproved the preliminary suspicion that an absent
coat map inherits the base mapped normal. A clearcoat permutation always
overwrites that initialization by sampling group-2 binding 11, and the resource
streamer binds its `[128,128,255]` normal fallback when the descriptor has no
coat texture. The result approximates the geometric normal rather than reusing
the base map, but it still pays one texture sample, UV transform, and TBN rebuild
for every clearcoat material and introduces the small UNORM flat-normal XY
quantization instead of expressing Khronos' exact no-normal-mapping branch.
Bevy specializes `STANDARD_MATERIAL_CLEARCOAT_NORMAL_MAP` separately, while
Godot passes the vertex normal directly when clearcoat ignores the base normal
map. Zircon should profile clearcoat-without-map and clearcoat-with-map corpora
before adding the corresponding presence identity; if material, the static
geometric-normal route is the structural target, not a runtime branch.

One correctness defect was confirmed independently: the final shader added
`surface.emissive` after all clearcoat weighting, although Khronos explicitly
defines `coated_emission = emission * (1 - clearcoat * clearcoat_fresnel)`.
The checked Bevy snapshot instead multiplies emission by the Fresnel term
itself, so it is not copied over the normative glTF contract. The source repair
now reuses the already prepared `clearcoat_base_energy` for emission in the
advanced Standard-PBR return. Basic and environment-only profiles remain
unchanged because their specialized sources do not admit clearcoat, and the
unlit early return remains uncoated. Contract regressions lock the exact WGSL
composition and the numeric anchors `(coat=1, NoV=1) -> 0.96`,
`(coat=1, NoV=0) -> 0`, and `(coat=0.5, NoV=1) -> 0.98`. The repair adds one
emission scale but no branch, sample, binding, permutation, helper, or repeated
Fresnel evaluation. This is source-complete pending managed Naga/WGPU and
product capture; it neither authorizes an Unreal-style multilayer migration nor
claims a measured performance result.

### 2026-08-28 runtime-owned normal convention canonicalization re-review

The upload-origin audit remains valid: decoded rows reach GPU upload without a
hidden image transform, and both material paths reconstruct the same handed
cross-product basis. The conclusion drawn from that evidence was not valid.
Calling the fixed no-flip path DX made a self-consistent glTF result look like a
DX canonical contract, while the actual basis and texel mapping are GL/right-
handed. The runtime-owned converter consequently moved the duplicate loop but
retained the wrong conversion direction.

There is a second Mikk contract defect in the consumers. Standard and fallback
currently project and normalize the interpolated tangent frame before applying
the map. `bevy_mikktspace` explicitly requires the shader inverse to use the
matching unnormalized interpolated T/B/N and normalize only the final mapped
normal; Bevy's PBR path preserves that rule. Per-axis Gram-Schmidt changes the
baker inverse and can distort authored or generated normal maps after
interpolation and non-uniform/skinned transforms.

The transferred source slice now implements that atomic boundary. The runtime
converter flips green only for decoded tangent-space DX payloads and publishes
GL metadata; explicit GL and glTF normal images retain their pixels. Compressed
DX payloads fail closed, and BC5 transcode/passthrough accepts only canonical GL
input. The shared WGSL normal decoder has a one-argument canonical ABI and no
convention constant or branch. Standard base/clearcoat mapping and fallback use
the unnormalized interpolated Mikk N/T and `cross(N,T)*w`, then normalize the
final mapped normal; anisotropy retains its separate orthogonalized frame.

The static hot-path delta is one convention conditional per normal sample to
zero. The Standard Mikk frame removes three pre-map normalize/Gram-Schmidt
operations; fallback removes geometric/tangent/bitangent and tangent-normal
pre-normalization while retaining BC5 reconstruction and final world-normal
normalization. These are source-operation counts, not measured GPU timing or
power results. Asymmetric DX/GL, compressed-container, generated-WGSL and
fallback source guards are present. Scoped rustfmt, diff integrity, production
shader-contract (`runtime_convention_branches=0`) and locked workspace metadata
pass. Fresh managed Cargo/Naga/WGPU, PNG, RenderDoc, timing, RSS and power gates
remain pending, so M5-M8 and the realtime-SH9 Failure remain unchanged.

### 2026-08-28 glTF texture-subasset owner convergence

The next importer review found a separate three-owner split below material
semantics. Builtin decode, builtin labeled-subasset assembly, and the priority-120
stable plugin independently implemented BasisU admission, core/WebP source
selection, decoded image validation and RGBA8 expansion, color-space/usage
variant publication, and glTF sampler projection. The two publication paths
were complete production copies, so a sampler, image-format, or extension fix
could change the default product without changing builtin behavior, or vice
versa.

Runtime asset importer infrastructure now owns that complete projection once.
Builtin decode uses its support gate, builtin labeled import calls its subasset
builder directly, and the stable plugin re-exports the same functions while
retaining only its material/mesh/scene assembly. Production definitions for
the texture-subasset algorithm are therefore reduced from three partial/full
authorities to one complete authority; no importer retains a private source,
RGBA expansion, sampler, or BasisU implementation.

This is structural convergence, not a payload-sharing optimization. For `T`
textures and total published output pixels `Q`, source counting and projection
remain `O(T + Q)`, with image/variant clones retained where independent
`TextureAsset` payloads require them. Runtime sampling, bindings, shader
permutations, Naga/WGPU work, and per-frame scans remain `+0`. A shared decoded
payload or copy-on-write representation is not authorized until cold/warm
import profiles show pixel copies are a material CPU/RSS bottleneck. Scoped
formatting, diff integrity, and workspace metadata pass; fresh managed compile
and product-route tests remain required before this source slice can enter a
milestone candidate.

### 2026-08-28 decoded-RGBA8 texture-build architecture review

The glTF texture path still ended after decode and published a one-mip RGBA8
payload. That bypassed the standalone texture importer's normal-aware, transfer-
correct offline mip construction, so minification quality and streaming
eligibility depended on which importer won priority. This is an infrastructure
defect, not a shader tuning issue: the renderer cannot recover a missing cooked
chain without moving avoidable work into frame or upload time.

The service boundary was reviewed before implementation. `AssetImportContext`
contains source bytes, settings, transaction snapshots, and reference repair;
it has no service injection contract. Both builtin and plugin importers are
registered as pure function handlers. Adding a texture-build service locator to
that context would introduce global lifetime and activation ordering into a
deterministic cook. Unreal's `FTextureBuildFunction` provides the relevant
reference instead: the build function is explicitly stateless, consumes named
immutable inputs plus build settings, changes its build version whenever output
semantics change, and separates encoded streaming mips from the packed mip tail.
Zircon therefore uses a runtime-owned stateless build kernel rather than a
runtime manager/service object. Platform encoding and streaming-tail splitting
remain later stages, not hidden work in glTF projection.

The decoded-RGBA8 v2 kernel now owns validation, canonical GL conversion and full
offline mip construction. Admission is restricted to single-layer `D2`; 2D
arrays, cube maps and cube arrays fail closed until a dedicated array-mip or
seam-aware cube owner and image oracle exist. It also rejects truncated base
payloads, runtime-mip requests, and compression targets that need a platform
encoder. glTF color/data/normal variants call this owner, and glTF normal
descriptors explicitly declare their specification-defined GL convention before
the build. Explicit non-mip
`minFilter=NEAREST/LINEAR` declares `None` and keeps only the base level; mipmap
filters and the engine-selected unspecified default declare `GenerateOffline`.
Both builtin and priority-120 stable importer versions include
the texture-build version, so the artifact cache cannot reuse old one-mip
outputs after this semantic change. The standalone builtin image importer and
all stable texture-plugin descriptors also advance their importer version for
the convention/cook semantic change.

The implementation retains the existing Box/Kaiser and normal-aware semantics
but changes storage ownership: it reserves/resizes one final level-major buffer,
keeps the base level in that buffer, and writes each later level directly into a
disjoint tail slice. It creates one sRGB decode LUT per texture instead of per
level, caches bounded Kaiser weights per axis/level, and renormalizes averaged
tangent-space normals. There is no per-level pixel `Vec`, base-payload clone, or
runtime texture/shader branch. For `P` base texels the complete chain is `O(P)`
texels (below `2P` even for a one-texel-wide chain and approximately `4P/3` for
square power-of-two textures). Box and normal filters do bounded four-sample
work per output texel; Kaiser is bounded by 25 source samples, so all three are
`O(P)`. Persistent memory is the `O(P)` output; filter scratch is `O(W + H)`.

The existing plugin mip files carry uncommitted `Plugins07` performance work
and have no live lease, but their last attribution belongs to a cancelled plugin
aggregate. Shader06 did not move, overwrite, or claim those mixed blobs. The
runtime kernel is the target authority; a later Plugins07 ownership transfer
must delete its duplicate algorithm and make standalone texture import call the
runtime owner. BC5/KTX2/BasisU/platform compression, alpha-coverage preservation,
and streaming-mip/mip-tail artifact separation remain advanced texture-build
work and are not implied by decoded-RGBA8 v2.

No timing, RSS, energy, or power improvement is claimed. Acceptance requires a
managed compile/test snapshot, byte-equivalence/product image checks for sRGB,
linear/data, normal, odd extents and sampler variants, then cold/warm glTF import
profiles recording source pixels, output bytes, mip count, allocations, wall/CPU
time and peak RSS. Power evidence must use the declared Windows profiler with
adapter/clock conditions and raw artifacts outside C:. RenderDoc and screenshots
under `docs/tests/runtime/shader` remain product gates; this section advances
source infrastructure only.

### 2026-08-28 per-draw material binding amplification review

The material binding path was re-read end to end before extending the clearcoat
ABI. This is the Shader06 measurement and implementation detail for canonical
optimization finding `09C P1-10`; ownership remains with the material/shader
pipeline plan rather than being duplicated here.

The current fixed material layout has 13 bindings: one uniform and six
texture/sampler pairs. `create_mesh_draw` receives a newly projected
`MaterialTextureSet`, prepares all six sampler variants, and creates both the
selected-material and standard-fallback bind groups. Each group projects all 13
entries. The direct-scene and hit-proxy routes pass no mesh-command cache. The
compiled-scene route can remove complete static cache hits before this point,
but residual dynamic, transparent, skinned, morphed, reactive, visibility-miss,
material-override, and cache-rebuild draws still execute the same constructor.
An override additionally constructs a `GpuMaterialUniformResource` in the draw
builder before creating the two groups.

For `R` residual draw constructions and `O <= R` draws with a material-property
override, the source therefore establishes the following static upper-bound
work, before measuring driver cost:

- `6R` sampler-variant preparation calls. A texture-backed slot performs a
  resource sampler-cache query; an output-target slot does not. The projected
  texture set is new per pending draw, so its variant result does not survive
  into another draw.
- `2R` WGPU bind-group creations and `26R` bind-group entry projections.
- `O` WGPU uniform-buffer creations carrying a logical 256-byte material row,
  in addition to the two bind groups for those draws.

These are exact source counts, not timings, allocation sizes inside the backend,
GPU cost, or power data. Multiple cameras and the separate hit-proxy submission
multiply `R`. Static compiled-command hits can reduce `R`, but they do not fix
the ownership of the remaining path. In particular, a stable direct scene with
10,000 draws sharing one unchanged material still asks WGPU to create 20,000
bind groups per build under the current algorithm.

The reference review supports moving GPU binding ownership out of draw
construction. Bevy prepares a `MaterialBindingId` while preparing the material,
stores it in `PreparedMaterial`, releases it on asset unload, and only resolves
and binds the prepared group in `SetMaterialBindGroup`. Unreal stores uniform
expression buffers on `FMaterialRenderProxy`, invalidates them explicitly,
rebuilds deferred caches with dedicated CPU profile scopes, and compares the
active shader-map identity before reevaluation; its mesh-draw system then
separates cached commands, dynamic commands, and dynamic instancing. These are
contract references, not a claim that either engine's backend object model can
be copied directly into WGPU.

Zircon already has the correct lifetime anchor. `PreparedMaterialBundle` owns a
process-local `draw_generation`, the complete texture set, and both uniform
resources, while `PreparedMaterial` retains only published, previous-published,
and staged bundles. The target is a generation-qualified prepared GPU binding
bundle owned with that state. It resolves sampler variants and the ordinary and
standard bind groups once per device generation, effective anisotropy cap, and
complete resolved texture/uniform identity. Same-revision streaming replacement
must invalidate only affected bindings; `draw_generation` alone is insufficient
because current texture residency may replace the resource without publishing a
new material generation. A global pointer-only cache is rejected because it
cannot express device loss, resource replacement, submission retirement, or
bounded old-generation retention.

The no-override steady-state complexity target is `O(U)` binding preparation
for `U` unique resolved material binding identities, instead of `O(R)` for draw
count. An override must reuse the standard group and route its custom row through
a bounded per-frame uniform arena or a generation/revision-qualified override
binding cache. That path is measured separately as `O(V)` for `V` unique active
override payload identities; permanent entity-keyed growth is not acceptable.
Published/previous/staged eviction and device-generation replacement must retire
GPU handles using the existing submission-lifetime contract rather than CPU
frame age.

Instrumentation is the first authorized production change. It must add existing
profiling-channel scopes and counters around residual draw construction, sampler
variant resolution, override uniform allocation, and bind-group creation. Each
viewport/camera sample must report direct versus compiled versus hit-proxy mode,
pending/residual draw count, complete/partial command-cache hits, unique material
generation count, unique resolved binding identity count, six-slot sampler
queries and cache misses, ordinary/standard bind-group creates, override uniform
buffer count/bytes, and frame submission wait. Ready-path labels and identities
must remain bounded and allocation-free.

The managed Windows baseline matrix is:

1. `1`, `100`, and `10,000` visible draws sharing one material, then the same
   counts with distinct materials;
2. no overrides, one shared override, and distinct overrides;
3. direct scene, compiled scene cold fill, compiled scene warm full-hit, forced
   residual rebuild, and hit-proxy submission;
4. one and multiple cameras, anisotropy unchanged/changed, one texture-residency
   replacement, one material hot reload, and device-generation recreation.

Each corpus records cold and at least 300 warm stable frames over repeated runs:
CPU scope p50/p95/p99, object-creation counts, allocator/RSS peak, submission
wait, GPU frame timestamps, and correctness pixels. Tracy/source counters locate
CPU ownership; WPR/WPA on a fixed DX12 adapter, driver, resolution, clock/power
mode, and AC state records scheduling plus CPU/GPU energy evidence; RenderDoc at
`D:\Tools\renderdoc` verifies final resource bindings and draw correctness, not
CPU timing. Raw ETL/RDC/profile artifacts stay on D: or F:, never C:. Accepted
PNG evidence remains under `docs/tests/runtime/shader` when workspace capacity
allows it.

Implementation proceeds only after that baseline. A measured warm path passes
when unchanged no-override corpora report zero sampler queries, zero uniform
buffer creations, and zero bind-group creations after preparation, and when the
10,000-shared-material result is independent of draw count for these metrics.
Reload, streaming, anisotropy, and device changes may rebuild exactly the
affected unique identities once. CPU p50/p95/p99, RSS, frame time, and energy
must then be compared against the unchanged corpus; no improvement or
engine-comparable power claim is allowed from object counts alone.

This review also changes the order of clearcoat work. Factor and roughness
textures are independent Khronos texture infos and may have distinct samplers,
UV sets, and transforms, so they cannot generally alias the metallic-roughness
or coat-normal binding. Adding both to the current fixed layout would require
four more bindings, taking each group from 13 to 17 entries. Under the current
per-residual-draw duplication that changes entry projection from `26R` to
`34R`, a 30.77% increase, before any texture-sample cost. Their ABI extension is
therefore deferred until the prepared-binding baseline and owner correction are
accepted. This does not defer the MVP base PBR, emissive clearcoat attenuation,
normal convention, or decoded-D2 correctness repairs.

Low E: workspace capacity and the absence of an exact passed current-source
validation ticket still prevent the managed before/after profile. The audited
scope remains immutable, so the normal-convention/shared-ABI repair requires an
explicit ownership transfer rather than another Session registration. No cache
implementation, current-source performance result, power result, PNG, or
RenderDoc acceptance is claimed, and M5-M8 plus the realtime-SH9 Failure remain
unchanged.

### 2026-08-28 historical RenderDoc diagnostic baseline and profiler slice

Two retained 2026-08-23 Vulkan captures were replayed through RenderDoc v1.44
before any material-binding optimization. The shared Performance01 audit script
still called the removed `ReplayController.GetResourceName` API. A diagnostic
wrapper under `D:/zircon-profiles` adapted only that query by constructing a
`ResourceId -> name` map from `GetResources()`; it did not modify repository
source or either capture. Raw JSON and wrapper artifacts remain on D: and are
diagnostic, not candidate or acceptance files.

The real-HDRI textured-material export capture has immutable SHA-256
`449923FEB035FFD3F824C9E47D320FA36935E321C3B41478E2DD7E0336BDC5D5`.
Its replay reported Vulkan, 963 actions, 7 draw actions, 25 dispatches, 208
copies, 54 clears, 886 resources, 48 textures, 211 buffers, zero API debug
messages, and 294 GPU-duration samples totaling 5.630272 ms. The top-25 copy
samples alone total 3.945536 ms, at least 70.08% of the captured GPU time. The
capture contains 48 `zircon-readback-ibl-pmrem` destinations and 74 WGPU
internal texture-clear passes; it is therefore an export/acceptance frame, not
a steady game-frame oracle. Its audit JSON is
`D:/zircon-profiles/shader06-material-ibl-20260823-r3-renderdoc-audit-20260828-r5.json`
with SHA-256
`D8D1296DAC63F6517BCF4303837E7B60DFCEBDBE1C608D0B567B4A4C8A74C9B6`.

The retained final-SH9 capture reported Vulkan, 586 actions, 133 draw actions,
17 dispatches, 43 copies, 7 clears, 656 resources, 32 textures, 150 buffers,
zero API debug messages, and 200 GPU-duration samples totaling 2.322656 ms.
One final `vkCmdCopyImageToBuffer` readback consumed 1.17 ms. Within the top-25
samples, copies total 1.254304 ms, compute dispatches 0.296960 ms, and draws
0.404320 ms. This capture includes 64 indirect material-sample submissions and
one final output readback. It demonstrates that BRDF/IBL shader execution is not
the first justified optimization target in this historical corpus, but it does
not measure the current CPU material-binding constructor or prove current-source
frame time. Its audit JSON is
`D:/zircon-profiles/shader06-realtime-ibl-final-sh9-20260823-renderdoc-audit-20260828-r1.json`.

The first production slice is therefore observation only. Residual draw
construction now emits one aggregate sample per build for:

- `material.binding.residual_draw_count`;
- `material.binding.sampler_variant_query_count` (`6R`);
- `material.binding.bind_group_creation_count` (`2R`);
- `material.binding.entry_projection_count` (`26R`); and
- `material.binding.override_uniform_buffer_creation_count` (`O`).

One `material/binding.build_residual_draws` CPU scope covers the residual draw
constructor. The profile arithmetic has focused 1-draw and 10,000-draw unit
contracts, including the exact 60,000 sampler-query, 20,000 bind-group and
260,000 entry-projection scale at 10,000 draws. Counter emission is aggregated
instead of producing four extra events per draw. Shader equations, binding
layout, resource lifetime, command-cache behavior and render output are
unchanged.

The current `build/build.rs` orchestration file is 1,007 lines and therefore at
the repository modularization warning boundary. This slice keeps the profile
arithmetic and its tests in the 274-line `create_mesh_draw.rs` owner and adds
only the residual-count/scope call site to the orchestrator. The next coherent
split is the residual `PendingMeshDraw` to `MeshDraw` materialization phase;
extracting it before the current exact snapshot is validated would mix a large
ownership refactor into a profiling-only evidence slice, so that split remains
an explicit pre-cache-implementation task. The remainder iterator's exact lower
and upper length bounds and the actual 13-entry bind-group array are now debug
asserted so the aggregate counter cannot silently drift after either boundary
changes.

This instrumentation is not the prepared-binding optimization. The next gate
is a managed current-source profiling build and the declared shared/distinct
material matrix. Only measured residual CPU p50/p95/p99 plus these object counts
may authorize the bounded generation-qualified cache. The historical RDC data
cannot be relabeled as current-source, steady-state, DX12, power, or post-change
evidence; M5-M8 and the realtime-SH9 Failure remain unchanged.

### 2026-08-28 core Fresnel and transmission layer-composition review

The complete Standard-PBR lighting path was re-read before changing the
transmission multiplier. This review covers the basic and advanced Forward
sources, deferred lighting, the fallback mesh shader, the shared GGX/environment
helpers, material feature projection, the scene-color copy, and glTF extension
projection. Khronos glTF 2.0 Appendix B is the normative base contract;
`KHR_materials_transmission`, `KHR_materials_diffuse_transmission`, and
`KHR_materials_clearcoat` define extension layering. Unreal remains the primary
engine-architecture reference, with Bevy used as a second implementation check.
`cmft` and `cmftStudio` own cubemap radiance/irradiance filtering and preview
resource flow only; neither is an authority for runtime BSDF layering.

The review rejects the current aggregate-lighting multiplier as a local bug
fix. It found two dependent structural defects:

1. Basic Forward, advanced Forward, deferred, and fallback direct lighting form
   `diffuse * (1 - metallic) + GGX(F)`. The diffuse term is not multiplied by
   the complementary Fresnel weight. Khronos instead defines
   `f_diffuse = (1 - F) * baseColor * (1 - metallic) / pi`. The existing form
   can allocate the same incident energy to both diffuse and specular lobes and
   is therefore a core PBR defect, not a transmission-only defect. Environment
   diffuse and ambient use only the metallic complement as well, so repairing
   one Forward shader would leave render-path parity broken.
2. Advanced Forward then aggregates ambient diffuse, reflected diffuse,
   reflected specular, clearcoat, and environment lighting into
   `opaque_lighting`, and multiplies all of it by
   `1 - specular_transmission`. This incorrectly removes base specular and the
   clearcoat top layer, lets transmitted scene color bypass clearcoat, does not
   tint specular transmission with base color, adds custom diffuse transmission
   without subtracting reflected diffuse, and adds both transmission modes when
   they coexist. Khronos changes only the dielectric base: reflected specular is
   unchanged, specular transmission replaces the diffuse BSDF, and specular
   transmission overrides diffuse transmission.

The importer boundary is intentionally narrower than the shader ABI.
`KHR_materials_transmission.transmissionFactor` maps to
`specular_transmission`; `transmissionTexture` is diagnosed as unsupported.
`KHR_materials_diffuse_transmission` is not in the supported-required-extension
set and its color/factor textures are not represented. The existing scalar
`diffuse_transmission` therefore remains an engine approximation and must not be
reported as Khronos extension support.

The implementation order is changed to preserve a single mathematical owner:

1. The shared direct-GGX owner will expose both the specular BRDF and its already
   computed Fresnel term. Isotropic and anisotropic callers then apply the
   complementary Fresnel to diffuse without recomputing the half vector or
   Schlick power. Basic Forward, advanced Forward, deferred, and fallback must
   cut over together. Ambient and environment diffuse must use one documented
   view-dependent complement so all four paths retain the same base-energy
   convention.
2. Advanced Forward will keep reflected diffuse, reflected specular, diffuse
   transmission, specular transmission, clearcoat, and emission separate until
   the final layer composition. For dielectric weight `q = 1 - metallic`,
   specular-transmission factor `t`, diffuse-transmission factor `d`, and
   clearcoat base weight `c`, the intended ownership is conceptually
   `c * (Rs + (1-t) * mix(Rd, Td, d) + q*t*Ts + E) + Cc`. `Rs` and `Cc` are
   never multiplied by `1-t`; `Ts` is base-color tinted; `t` suppresses the
   custom diffuse-transmission branch. The exact Fresnel complement remains in
   the shared base owner rather than being multiplied a second time here.
3. Environment PBR must retain its existing diffuse/specular component result
   until composition instead of returning an early aggregate. A transmission
   fallback must consume raw environment radiance in the transmitted direction,
   not the material-colored reflected PBR aggregate. The existing scene-color
   resource has exactly one mip, so this MVP slice may retain a sharp screen
   sample but must not claim rough microfacet BTDF fidelity. A scene-color
   pyramid or dedicated rough-transmission convolution is a later measured
   capability, not an implicit full-screen cost added to this correctness repair.

This design adds no material binding, bind-group entry, uniform field, PSO key,
or feature permutation. It replaces repeated per-light transmission clamping and
view-Fresnel work with per-fragment prepared weights, while reusing the Fresnel
already required by GGX for base diffuse coupling. Reflected environment sampling
remains one provider query. Only an unavailable scene-color transmission fallback
may perform an additional directional environment query; its hit/miss rate and
cost must be instrumented before optimizing that fallback.

Acceptance requires numeric endpoint tests for metallic/Fresnel/transmission/
clearcoat combinations, generated WGSL validation for basic and every advanced
feature variant, direct/deferred/fallback source-parity guards, and product images
covering dielectric, metal, clearcoat plus transmission, and the two transmission
factors together. The managed profile must record per-fragment/per-light operation
counts, scene-copy availability, fallback count, GPU p50/p95/p99, RSS, and fixed-
adapter WPR/WPA energy evidence. RenderDoc at `D:\Tools\renderdoc` verifies final
resource/sample and lobe composition. Until those gates pass, this section is an
architecture disposition only and does not advance M5-M8 or the realtime-SH9
Failure.

### 2026-08-28 core Fresnel and transmission source implementation status

The architecture disposition above is now implemented at source level, without
claiming managed or product acceptance. The shared isotropic and anisotropic GGX
owners return one `ZrPbrSpecularComponents` value containing both the BRDF and
the Fresnel term already computed for it. Basic Forward, advanced Forward,
deferred, and fallback direct lighting consume that same Fresnel value for the
diffuse complement. Ambient, lightmap, environment-only, and deferred indirect
paths use the shared view-Fresnel diffuse-energy helper, so the metallic and
Fresnel complements no longer diverge by render path.

Advanced Forward now carries base diffuse and retained reflection as separate
components through the direct-light loop. The environment provider likewise
exposes diffuse and specular components until final composition. Prepared
specular- and diffuse-transmission weights are decided once per fragment;
specular transmission overrides diffuse transmission, reflected specular is not
attenuated by transmission, transmitted scene radiance is base-color tinted and
metal suppressed, and both the transmitted scene and emission remain below the
clearcoat base-energy layer. A missing scene-color sample falls back to raw
environment radiance in the transmitted direction rather than an already
material-colored reflected IBL aggregate.

The per-light diffuse BTDF branch now tests the prepared effective
`diffuse_transmission > 0.0`. Consequently a specular-transmission material no
longer evaluates a zero-contribution diffuse BTDF for every active light. The
slice adds no binding, uniform field, bind-group entry, PSO key, feature bit, or
texture sample on the ordinary opaque path. The sharp single-mip scene-color
sample remains an explicit MVP limitation and is not described as rough
microfacet transmission.

Source-contract scans reject the previous `direct_diffuse_brdf + specular`
composition and the previous advanced aggregate-lighting multiplier; scoped
`git diff --check` is clean. These are static implementation checks only. A new
coordinator-managed Windows Cargo/Naga/WGPU batch, numeric endpoint tests,
current-source product images, DX12 RenderDoc replay, GPU p50/p95/p99, RSS, and
WPR/WPA energy evidence remain required. The historical captures still identify
copy/readback as at least 70.08% of the sampled export-frame GPU time and cannot
be relabeled as current-source or post-change evidence. M5-M8 and the open
realtime-SH9 Failure therefore remain unchanged pending those gates.

### 2026-08-28 rejected M6 transmission-frame and scene-copy review

The sealed 31-file M6 snapshot was independently reviewed before a coordinator
commit. The review rejected it with one Critical and four Important findings.
The current screen-space specular-transmission path retains the reflected GGX
lobe but colors transmission only with `baseColor * (1 - metallic) * t`; it is
missing the complementary Fresnel energy. It also uses
`normal.xy * (ior - 1) * thickness * 0.02` as a screen offset, while the
environment fallback always macro-refracts even when glTF volume thickness is
zero. These are one structural defect: screen projection, environment direction
and Fresnel must be derived from one Snell transmission frame instead of three
unrelated approximations.

Khronos `KHR_materials_transmission` defines transmission as the base-layer
energy that penetrates rather than being specularly reflected, and treats the
default material as infinitely thin. `KHR_materials_volume` makes nonzero
thickness the opt-in boundary for volume displacement and attenuation. Bevy's
reference implementation projects `world_position + refracted * thickness` and
uses a transmitted-side cosine approximation. Unreal's primary thin-translucent
path keeps specular separate and applies complementary Fresnel from entry
`N dot V`; this slice follows that view-energy convention so exterior grazing
incidence converges to zero transmission under Schlick. Unreal's
single-layer-water path validates
refracted scene-color samples with depth and an optional refraction mask rather
than material color alpha.

The scene-copy fallback exposed a second boundary error. The shader currently
queries probe/sky radiance before sampling scene color, so every transmissive
fragment pays both providers even when the scene copy is usable. Static source
cost is therefore one scene-color sample plus one complete probe-selection/sky
query per transmissive fragment. At 1920x1080 full coverage that is 2,073,600
unnecessary fallback queries per frame before probe blending expands the number
of cubemap samples. The previous RenderDoc corpus is historical rather than a
current-source transmission capture, so this count is an algorithmic upper
bound, not a measured GPU time claim. Current-source hit rate and GPU p50/p95/p99
remain mandatory before reporting a performance gain.

The existing `scene_color_sample.a > 0` predicate is not a valid replacement
for depth/coverage. Forward output currently preserves authored base-color alpha,
including for glTF `OPAQUE` materials where that alpha is semantically ignored.
The resource graph already knows whether a scene-copy texture is bound, but did
not project that availability into WGSL. The source repair therefore adds one
16-byte, generation-owned availability uniform at the transmission resource
boundary. When a full-viewport copy is bound it is the background source for all
covered pixels, including valid black or alpha-zero scene values; only the
zero-step/unbound case queries the environment fallback. Depth/coverage remains
a later refraction-occluder rejection capability, not a reason to reinterpret
business alpha as resource validity.

The review also found that formal Forward and deferred lightmap templates pass
`cos_theta = 1.0` to the shared diffuse-energy helper while the fallback shader
uses actual `N dot V`. This contradicts the claimed cross-path energy contract.
The immediate repair routes all three through one view-dependent owner. No new
binding, texture sample or material permutation is needed for that convergence.

Coordinator validation run `6bcd5c361cfc4bd69039e101eac0ba0c` is explicitly
excluded as shader evidence: its accepted job ran 45 session-coordinator Python
tests and did not compile Rust or validate WGSL. The rejected M6 snapshot must
not be committed or used to release the mixed Cargo manifests. A successor
snapshot requires exact Windows Cargo/Naga validation, a new independent review,
product screenshots, and current-source RenderDoc/profile evidence.

### 2026-08-29 transmission-frame successor repair and view-owner convergence

The rejected review findings are repaired in current source, but this remains a
successor worktree rather than an accepted milestone. Screen and environment
specular transmission now consume one `ZrPbrTransmissionFrame`. A zero-thickness
thin surface keeps the straight transmitted direction; positive thickness opts
into the Snell exit displacement and macro-refracted environment direction.
Both sources use the entry-side `N dot V` Fresnel complement and the canonical
material F0/metallic energy owner, while reflected base specular remains outside
the transmission multiplier. This follows the glTF thin/volume boundary and the
Unreal thin-translucent entry-Fresnel convention. Bevy confirms the equivalent
exit projection after accounting for its opposite refracted-vector sign. Probe
selection remains at the entry position, matching the reference implementation;
only the transmitted lookup direction changes.

Scene-copy validity is now an explicit 16-byte generation-owned uniform at group
1 binding 38. The bound full-viewport copy is valid even when a sampled pixel is
black or has authored alpha zero. Only the unbound/zero-step branch evaluates
the environment fallback, so a scene-copy hit no longer eagerly executes both
providers. At 1920x1080 full transmissive coverage this removes the previous
static upper bound of 2,073,600 unnecessary environment fallback queries per
frame; one fallback query may itself reach the sky plus two probes. These are
source operation counts, not measured GPU-time or power improvements. The
current-source hit rate, compiler output, GPU p50/p95/p99 and energy samples are
still required.

The camera view-direction algorithm had six independent owners across
environment-only Forward, basic/advanced Standard-PBR Forward, two deferred
paths and fallback mesh, even though every assembled module already includes
`zr_pbr_common.wgsl`. Those six local definitions are removed. The common owner
retains zero-safe normalization and early returns for both perspective and
orthographic endpoints, and all six consumers now use it directly. This removes
the second source owner from each assembled module and prevents formal lightmap
Fresnel from drifting from surface shading. It does not yet prove fewer runtime
normalizations: Forward shading and the separately composed baked-lightmap term
can still request the same view direction twice. Adding it unconditionally to
`ZrShadingContext` was rejected for this slice because it would charge unlit
fragments or change the custom shading interface before a profile establishes
the tradeoff. The next performance slice must inspect compiler output and
permutation-specific instruction counts before changing that context boundary.

A separate foundational gap remains for double-sided PBR. Feature projection
defines `ZR_FEATURE_DOUBLE_SIDED`, but the core fragment shading paths do not yet
own a `front_facing` normal/tangent-handness policy. Folding that issue into the
transmission-frame repair would change Forward, deferred and normal-map
contracts without a complete parity design, so it remains an explicit next
architecture item rather than an implicit local normal flip.

Exact validation is still blocked and no pass is claimed. Managed job
`ab2519bf94ac43eea728f9e8223fe03f` reached the `zircon_runtime` lib-test build
but failed before target tests with 425 Rust errors while the foreign
`zircon_runtime_interface` refactor was changing; its observed approximately
9.50 GB peak RSS is compiler RSS and must not be reported as runtime memory.
A production-only retry submitted cargo-acquire request
`40ca2a3a16dc409493f6578bf9f81725`; it reached terminal `failed` with
`unmanaged_artifacts_detected` for
`D:\cargo-targets\zircon-engine\python-temp` and launched no Cargo process.
The unknown artifact was not deleted. Current non-build evidence is limited to
one remaining view-direction definition across graphics source, zero legacy
call sites, exact-file `rustfmt --check`, and scoped `git diff --check`.
Fresh Cargo/Naga/WGPU, numeric endpoint tests, the advanced transmission product
scene, `docs/tests/runtime/shader` PNG output, DX12 RenderDoc replay, timing/RSS,
and WPR/WPA power evidence all remain open. M5-M8 and the realtime-SH9 Failure
therefore remain unchanged, and the mixed Cargo manifests remain retained by
Shader06 until an exact coordinator service commit exists.

### 2026-08-29 double-sided surface-frame and mirrored-raster disposition

The next source review found that `ZR_FEATURE_DOUBLE_SIDED` currently changes
pipeline culling but has no fragment-facing contract. Forward and G-buffer
entry points do not consume `@builtin(front_facing)`, and the standalone
fallback shades its sampled normal without a face-orientation step. A visible
back face therefore reaches PBR with a negative `N dot V`: view Fresnel clamps
to grazing, reflected diffuse collapses, clearcoat uses the wrong hemisphere,
and transmission refracts from the wrong side. This is a surface-frame ownership
defect rather than a BRDF tuning issue.

Unreal is the primary reference. `MaterialTemplate.ush` builds `TwoSidedSign`
from raster facing, view culling sign, and primitive determinant sign, then
applies it after tangent-to-world normal evaluation. Bevy independently reaches
the same logical-face requirement: it corrects `front_facing` by the model 3x3
determinant, flips the tangent-space normal for a double-sided back face, and
uses the corrected normal in both forward and prepass paths. Zircon already
classifies negative determinants once per changed GPUScene instance and adjusts
normal inverse-transpose and tangent handedness, so no new transform inversion,
uniform, binding, or material permutation is required.

The deeper raster review supersedes the initial proposal to XOR
`front_facing` with a GPUScene flag in every double-sided fragment. WGPU already
defines `front_facing` from the pipeline's selected `front_face`; selecting
clockwise for a negative-determinant draw both preserves one-sided culling and
normalizes the fragment builtin to the authored face. This matches Unreal's
reverse-culling PSO ownership while avoiding a fragment-stage GPUScene read.

The implemented source contract is therefore:

- classify normal-transform flags once while building each stable pending
  instance, reusing the cached GPUScene shadow value when its transform is
  unchanged and otherwise using the existing constant-time matrix classifier;
- retain those flags on `PendingMeshDraw`, clone the material `PipelineKey`, and
  set the PSO-only `reverse_raster_winding` bit from the negative determinant;
- keep that bit in draw, batch, command-cache, and runtime PSO identity, but
  exclude it from `ShaderVariantKey` and disk WGSL identity;
- use one shared mesh-pipeline helper to select clockwise or counter-clockwise
  front face for base, G-buffer, depth, shadow, velocity, reactive-mask, OIT,
  and hit-proxy passes;
- complete the material surface, including normal maps, before double-sided
  raster orientation; flip base and clearcoat normals, preserve the tangent,
  and flip the bitangent so `B = cross(N, T) * handedness` remains true;
- consume the normalized builtin in Forward, deferred, environment-only,
  debug G-buffer, OIT, hit-proxy-alpha, and the retained standalone fallback.

The steady-state CPU work remains one bounded GPUScene lookup for an unchanged
stable instance; a changed or new transform performs the pre-existing O(1)
normal-transform classification once. There is no new material permutation,
bind-group entry, texture sample, fragment GPUScene load, transform inversion,
or `cull_mode = None` fallback. Runtime PSO cardinality for an otherwise
identical key can grow by at most two when mirrored and non-mirrored draws both
exist, which is the minimum state split needed to retain early back-face
rejection. Double-sided fragments add one scalar facing sign and orientation of
the surface frame; one-sided variants retain the compile-time feature gate.
Current-source compiler output and GPU counters are still required before
claiming that the one-sided path folds completely or that the double-sided cost
is negligible.

The previously missing OIT and raster owners were acquired through audited
coordinator scope transfer rather than left as a Forward/G-buffer half-fix.
Source-level regression guards cover PSO identity without shader permutation,
all eight raster pass owners, the shared material-frame orientation, OIT entry
assembly, hit-proxy alpha, and standalone fallback parity. Exact-file
`rustfmt --check` and scoped whitespace/source-contract checks are the only
evidence available in this shared snapshot. Fresh Cargo/Naga/WGPU acceptance,
DX12 product PNG and RenderDoc capture, draw/PSO counters, timing, RSS, and
WPR/WPA power evidence remain open; no performance win or milestone acceptance
is claimed. M5-M8 and the open realtime-SH9 Failure remain unchanged.
### 2026-08-29 volume-thickness space contract and scale-aware transmission plan

The current glTF projection preserves `KHR_materials_volume.thicknessFactor` as the
material `thickness`, but the fragment path consumes that value directly as a
world-space displacement and Beer-Lambert distance. This is not a valid contract
for scaled instances: Khronos defines thickness in mesh coordinates, requires node
transforms to affect it, and defines `attenuationDistance` in world space. A scale-2
instance therefore needs twice the refracted displacement and optical distance; the
current path leaves both unchanged.

The rejected alternative is to divide thickness by an incidence cosine. Khronos
explicitly describes the raster thickness map as a lossy estimate of the travelled
distance, and both its sample renderer and Bevy propagate the refracted ray by the
authored thickness rather than applying a second angular correction. The Khronos
sample renderer instead extracts the lengths of the three `modelMatrix` basis
columns, multiplies the normalized refracted ray by that rotation-independent scale,
and uses the length of the resulting world-space ray for attenuation. Bevy confirms
the existing thin/volume environment-direction approximation and direct
`thickness / attenuationDistance` Beer-Lambert convention for an unscaled material.

The accepted successor remains inside the current transmission owner:

- pass the existing flat `ZrShadingContext.instance_index` into the frame owner and
  read the already-bound GPUScene `world_from_local` row only after the transmission
  lobe and positive-thickness volume branches are known active;
- construct one scale-aware world-space transmission ray from normalized Snell
  refraction, material thickness, and the three basis-column lengths;
- make exit projection, environment fallback direction, and Beer-Lambert attenuation
  consume that same ray or its length so the three users cannot drift;
- retain the thickness-zero endpoint as zero displacement, zero attenuation distance,
  and straight-through `-V` environment lookup;
- add no binding, varying, material-uniform field, PSO key, or shader permutation.

The attenuation endpoint also needs hard convergence. glTF permits zero channels in
`attenuationColor`; for every positive optical distance, `0^(x/d)` must therefore be
zero. The current epsilon floor creates non-physical residual light. An explicit
zero-distance branch is the correct owner: thin walls return unit attenuation without
evaluating `pow`, while positive-distance volumes evaluate the Khronos/Bevy formula
against the normalized `[0, 1]` color, including exact zero. This avoids undefined
`0^0` behavior and removes three `pow` evaluations from the thin-wall path.

The same attenuation owner must cover diffuse transmission. Khronos assigns all
inside-volume transport to `KHR_materials_volume`, and Bevy applies Beer-Lambert to
the accumulated transmitted-light bucket after its diffuse/specular contributions.
Zircon currently applies attenuation only inside the scene/environment specular
transmission helper, so back-lit diffuse BTDF energy bypasses absorption entirely.
The successor computes diffuse attenuation once before clustered-light traversal and
passes the resulting `vec3` through the direct-light call chain; each light performs
only the final multiply. Specular and diffuse transmission are already mutually
exclusive in this material model, so an active pixel evaluates at most one frame and
one attenuation `pow`, never one per light.

Diffuse transmission also needs an indirect owner. The current composition removes
the authored fraction from front-side ambient/environment diffuse but only restores
punctual back-light BTDF energy, so an environment-lit material loses energy instead
of transmitting irradiance from its opposite hemisphere. Bevy builds a transmitted
environment input with the inverted normal and explicitly avoids diffuse occlusion.
Zircon already exposes `zr_environment_diffuse_color_normalized`, which evaluates only
SH9 or the irradiance cube and does not pay for reflection-probe selection, PMREM
specular, or the environment BRDF LUT. The accepted closure samples it at `-N` only
for active diffuse transmission and combines it with unoccluded scene ambient before
applying canonical base color, entry Fresnel/metallic energy, the authored lobe weight,
and the shared volume attenuation.

Screen-space transmission must distinguish a real scene-color hit from a clamped
sample. The current viewport helper preserves the sign of near-zero/negative `w` and
clamps every projected UV into `[0, 1]`; with the scene copy available, points behind
the camera and refracted exits outside the view therefore become center/edge colors
and never reach the environment fallback. Unreal rough refraction explicitly rejects
samples outside the view rectangle. Zircon's stronger local contract is a projection
record containing unclamped viewport UV plus validity: positive `w`, XY inside NDC,
and Z inside the WGPU visible range. A valid copy hit returns immediately and performs
no environment query; an unavailable copy or invalid projection falls back from the
same transmission frame's exit position and environment direction. This also removes
the stale entry-position probe-selection origin for thick volumes.

The fragment cost for an active transmission pixel gains one existing GPUScene row
read plus three basis-vector lengths; the row is already resident for mesh shading and
the work is compiled out of non-transmission variants. Whether this is hidden by the
existing scene-color/environment texture latency is a profiling question, so no GPU
time or power improvement is claimed before the managed product and RenderDoc gates.
The thin-wall branch performs none of that volume work: it returns the entry position,
straight-through `-V`, zero optical distance, and the already-required Fresnel cosine
before loading the instance matrix or evaluating Snell refraction.

### 2026-08-29 anisotropic environment-reflection direction plan

The Advanced Standard-PBR path currently applies anisotropy only inside the
per-light GGX loop. Global PMREM, reflection probes, and the non-planar fallback all
continue to sample `reflect(-V, N)`, so an HDRI-dominant material loses the elongated
anisotropic response even though its direct highlights use the anisotropic tangent
frame. This is a structural split between two consumers of the same material lobe,
not a PMREM bake or probe-selection defect.

Khronos defines the single-sample IBL approximation by constructing an anisotropic
bitangent from the rotated material tangent, bending the normal in the view plane,
and moving the reflected direction toward that bent normal as roughness increases.
Bevy implements that contract before environment-map lighting. Unreal uses the same
ownership boundary: `ModifyGGXAnisotropicNormalRoughness` changes the environment
normal before reflection construction, then `GetOffSpecularPeakReflectionDir`
redirects the rough lobe before reflection-capture/skylight lookup. The exact Zircon
direction follows the Khronos material semantics while the API boundary follows the
Unreal separation of lobe direction from environment provider selection.

The accepted implementation must keep diffuse and energy terms independent from
the lookup approximation:

- rotate the existing surface tangent by the material anisotropy rotation and derive
  the matching bitangent in the already-normalized surface frame;
- construct the Khronos bent normal only when the compile-time anisotropy feature
  is present and runtime strength is positive;
- pass that bent normal to a dedicated environment-components entry, which derives
  the dominant direction only after a provider is active, while retaining
  the original normal for diffuse irradiance, `N dot V`, Fresnel/BRDF-LUT energy,
  specular occlusion, and clearcoat layering;
- preserve the existing planar-reflection precedence; the derived direction affects
  global PMREM and reflection-probe lookup only because the planar capture is already
  projected from world position;
- fall back to the isotropic direction for a degenerate tangent frame instead of
  returning black.

The anisotropy-specific path adds no texture sample, binding, uniform, varying, PSO
key, or permutation. An active anisotropic pixel adds two cross products, tangent and
bent-normal normalization, and a scalar bend mix before the shared dominant-direction
and single provider lookup. The base isotropic dominant-direction cost is owned and
budgeted separately below. A future multi-sample anisotropic PMREM integration is an
advanced quality/performance tradeoff and is not part of the MVP closure.

This review also exposes, but does not optimize, a separate direct-light hot-path
shape: `zr_aniso_ggx_components` currently rotates and normalizes the tangent frame
inside every clustered-light iteration. Its static scale is `O(L)` trigonometric and
frame-preparation work for `L` affecting lights, while the environment repair prepares
its frame once per pixel. The likely target is a shared once-per-pixel rotated frame
plus `O(L)` GGX evaluation, but changing that owner is a performance optimization and
requires shader disassembly/GPU timing for representative 1/8/32-light scenes first.
It is therefore recorded for the managed profiling matrix rather than folded into a
correctness patch without before/after evidence.

The separate rough-transmission review found that Khronos applies
`roughness * clamp(ior * 2 - 2, 0, 1)` to the transmission framebuffer LOD. Zircon's
scene copy is intentionally a one-mip `Rgba16Float` viewport-local resource, so
applying that correction only to environment fallback would split source semantics.
The correction therefore remains coupled to a future scene-copy mip-chain/filter
owner; it is not smuggled into this anisotropic reflection repair.

### 2026-08-29 base rough-IBL dominant-direction convergence

The anisotropic review exposed a base-path defect shared by formal Forward,
deferred, clearcoat, and the reduced environment-only shader: every isotropic PMREM
lookup uses the perfect-mirror direction `reflect(-V, N)` for every roughness. At
high roughness that points the center of the prefiltered lobe below the tangent plane
at grazing angles and can gather energy from the wrong hemisphere. PMREM convolution
cannot repair a wrong lobe center at lookup time.

Unreal owns this correction in `GetOffSpecularPeakReflectionDir` before reflection
capture/skylight sampling, and applies it after its anisotropic normal modification.
Khronos specifies the normalized single-sample form
`normalize(mix(reflect(-V, N), N, roughness^2))`, explicitly noting that the same
correction is more accurate with and without anisotropy. Zircon should use one
environment-core helper with exact mirror and fully rough endpoints, then route the
formal environment wrapper, reduced environment-only shader, clearcoat, and the new
anisotropic bent normal through it. Provider-empty and planar-hit exits stay before
this ALU.

This adds no sample, binding, uniform, varying, PSO key, or permutation. Roughness 0
returns the mirror direction and roughness 1 returns the normal without normalization;
intermediate active IBL pixels add one vector mix and normalization before the same
single PMREM/probe lookup. Fresh shader disassembly, GPU timing, and power comparison
remain required because this is a broad correctness cost even though it does not
change asymptotic texture work.

The first implementation review caught a provider-boundary regression before dynamic
validation. Zircon's procedural-sky fallback has no PMREM and the main plan explicitly
requires one perfect-reflection lookup rather than a fabricated rough lobe. Applying
the dominant direction before provider selection silently violated that contract and
the existing test only inspected the sky helper after its input had already changed.
The accepted API therefore carries separate sky and probe directions: it constructs
the perfect reflection once, derives the dominant direction only when a cubemap,
realtime-IBL PMREM, or local probe can consume it, and keeps the perfect direction for
an unconvolved procedural sky even when that sky is blended with a probe. Explicit
transmission radiance passes its already-normalized refracted direction to both inputs.
The reduced environment-only path applies the same source-kind branch. This preserves
one reflect in every active path, adds no normalization to procedural-only fallback,
and retains the intended one dominant normalization for intermediate-roughness PMREM
or probe shading.

### 2026-08-29 specular-occlusion reference recheck

The environment and clearcoat paths both call the shared
`zr_environment_specular_occlusion(NoV, roughness, AO)` helper. A cross-engine scan
initially made its exponent look suspicious because Filament-derived renderers use a
different fitted exponent. The exact Unreal owner resolves the ambiguity:
`ReflectionEnvironmentShared.ush::GetSpecularOcclusion` accepts `RoughnessSq` and
evaluates `saturate(pow(NoV + AO, RoughnessSq) - 1 + AO)`, while its reflection and
mobile-lighting callers pass squared perceptual roughness. Zircon clamps the three
public inputs, squares perceptual roughness once, and evaluates the same expression.

The current helper is therefore retained. Replacing its exponent with the
Filament/Bevy fit would be a model change, not an Unreal-parity correction, and would
also alter base-environment and clearcoat occlusion together. This recheck adds no
production instruction, binding, sample, permutation, or ABI field. The existing
formula and boundary tests remain the source contract; current-source image,
RenderDoc, GPU-time, and power gates remain pending.

### 2026-08-29 direct anisotropic GGX correctness plan

The provider review then returned to the direct anisotropic lobe before attempting the
recorded once-per-pixel tangent-frame optimization. Its current axis mapping is not the
KHR material model: it sets `alpha_t = base_alpha * (1 + strength)` and
`alpha_b = base_alpha * (1 - strength)`, clamps strength below one, and feeds
`sqrt(alpha_t * alpha_b)` into an isotropic joint-Smith visibility function. Khronos
instead defines the increased-roughness axis as
`mix(material_alpha, 1, strength^2)`, keeps the perpendicular axis equal to
`material_alpha`, and evaluates direction-dependent anisotropic Smith visibility from
the tangent/bitangent projections of both view and light. Unreal's `D_GGXaniso` and
`Vis_SmithJointAniso` use the same Burley/Heitz distribution and directional visibility
ownership.

This is a correctness defect that must be repaired before moving frame preparation out
of the light loop. At perceptual roughness `0.5`, `H=N`, and strength `1`, the current
`0.99` clamp produces `alpha_t=0.4975`, `alpha_b=0.0025`, and an NDF peak of
approximately `255.93`; the Khronos axes are `1.0` and `0.25`, with a peak of
approximately `1.273`, so the current normal-direction peak is about `201.01x` too
large. Strength zero is an exact isotropic endpoint in both models. At strength `0.5`
and `0.8`, the same anchor is already about `2.33x` and `8.11x` too large.

The accepted correctness slice keeps the current material ABI, feature bit, tangent
rotation owner, clustered-light traversal, F0/Fresnel owner, and one lobe evaluation per
visited light. It replaces only the two alpha values, the low-roughness-safe Burley NDF,
and the scalar geometric-mean visibility with the directional Heitz function. The new
visibility performs the same two vector lengths/two square roots already paid by the
old exact scalar helper, with additional tangent/bitangent dot products and multiplies;
no texture sample, binding, varying, PSO key, permutation, or asymptotic loop change is
introduced. Moving rotation/frame construction from `O(L)` to once per pixel remains a
separate measured optimization requiring 1/8/32-light disassembly, GPU p50/p95/p99,
and power evidence. This slice may claim only formula and source-structure correction
until managed Naga/WGPU and product evidence run.

The anisotropic base-alpha floor also converges from its private `0.002` to the shared
isotropic owner's `0.001`. With the stable NDF form there is no denominator-floor reason
to retain a wider lobe, and matching floors prevents a zero-roughness material from
jumping when strength crosses the runtime epsilon. Strength zero and a degenerate
tangent frame return the common isotropic components directly; an exact zero
distribution vector returns zero instead of reintroducing a complete-denominator floor.

### 2026-08-29 unbounded volume-attenuation sentinel plan

The volume follow-up found that the exact-black repair still mishandles glTF's
default unbounded `attenuationDistance`. CPU material state represents that default
with the finite `STANDARD_PBR_NO_ATTENUATION_DISTANCE = f32::MAX`, while WGSL sends
every positive optical distance through
`pow(attenuationColor, transmissionDistance / attenuationDistance)`. For a one-unit
ray the exponent is approximately `2.94e-39`: nonzero colors round to the intended
unit transmittance, but an exact black channel evaluates `0^positive = 0` instead of
the required no-attenuation result `1`. An authored black attenuation color therefore
absorbs all positive-distance light even when no finite attenuation distance exists.

This is a cross-layer material sentinel, not a shader-helper tuning threshold. The
accepted repair must converge the existing Rust owner, the `ZrSurfaceOutput` WGSL
default, generated Standard-PBR material projection, and the Beer-Lambert helper on
one finite `1.0e30` sentinel. The shader must return unit transmittance before `pow`
when the optical distance is zero or the material distance is at least that sentinel;
the `>=` comparison also accepts already-staged `f32::MAX` payloads. Finite positive
distances preserve the exact-zero-color behavior introduced by the prior repair.

The change keeps the existing uniform slot, material ABI, feature bit, binding set,
texture samples, and loop order. An active volume pixel gains one scalar comparison
on the already-loaded attenuation distance and can skip three `pow` evaluations for
the default unbounded material. This is a correctness and static work-count statement,
not measured GPU or power improvement; current-source Naga/WGPU, product image,
RenderDoc, timing, RSS, and power evidence remain mandatory.

The source implementation now uses `1.0e30` in the existing Rust contract and one
`ZR_PBR_NO_ATTENUATION_DISTANCE` declaration in `zr_surface_types.wgsl`; default
surface construction and generated Standard-PBR projection consume that owner. CPU
normalization clamps older/larger finite sentinels to the canonical value so material
and cache identity do not split, while the volume helper also accepts them through the
`>=` guard. Static red/green checks observed all four missing contracts before the
production edit and all four present afterward. Rust formatting, single-owner scan,
and scoped diff integrity pass; managed compiler and product gates remain open.

### 2026-08-29 direct versus split-sum F90 contract recheck

The shared direct-light Schlick helper was re-read after the low-IOR path exposed an
apparent mismatch with environment reflection. Khronos glTF Appendix B and
`KHR_materials_ior` define ordinary dielectric Fresnel as
`F0 + (1 - F0) * (1 - abs(VdotH))^5`, so direct reflection and its complementary
diffuse/transmission weight have a fixed white `F90 = 1`. Zircon's shared direct GGX,
anisotropic GGX, clearcoat, diffuse-energy, and transmission-frame consumers already
use that one owner and are therefore consistent with the normative glTF lobe contract.

Unreal's default direct GGX instead calls the one-argument `F_Schlick`, whose grazing
term is gated by `saturate(50 * SpecularColor.g)`; its `PreIntegratedGF` applies the
same gate to the split-sum B term. Shader06 deliberately adopted that Unreal
split-sum contract in sections 4.4/4.5, and the earlier IOR disposition explicitly
keeps it for this milestone while excluding full visual parity below `F0 = 0.02`
(`IOR ~= 1.3294`). Bevy is an implementation cross-check rather than a normative
owner and likewise uses a low-F0 grazing gate in its principal specular helper.

Changing the shared direct helper to Unreal would violate the selected Khronos base
and transmission model; changing only the environment LUT consumer to fixed white
would invalidate the declared Unreal preintegration contract and its existing
fixtures. This recheck therefore makes no production change. The known divergence is
restricted to the already excluded low-IOR boundary and must be resolved only by the
separately owned low-IOR material contract, fixture, regenerated LUT interpretation,
managed image/RDC comparison, and GPU/power profile. No extra ALU is added to the
current per-light path, and M5-M8 remain unchanged.

### 2026-08-29 required glTF material-extension admission plan

The CPU-to-shader feature audit found an earlier fail-open boundary in both glTF
importers. Their required-extension allowlists advertise `KHR_materials_transmission`,
`KHR_materials_volume`, and `KHR_materials_ior`, but material projection only diagnoses
and ignores `transmissionTexture` and `thicknessTexture`, and diagnoses the special
`ior = 0` compatibility mode while retaining fallback material values. When any of
those extensions appears in `extensionsRequired`, continuing import violates the glTF
admission contract: the asset explicitly requires semantics that the material ABI and
shader do not implement.

The repair belongs in one runtime-owned material-extension preflight shared by the
builtin importer and the priority-120 stable plugin. After the document is parsed but
before buffer/image decode, it receives the original required-extension set and scans
only material extensions whose required owner is active. A required transmission
texture, thickness texture, or zero-IOR compatibility request returns a typed
`AssetImportError::Parse` with the extension field and material index. Factor-only
transmission/volume and ordinary `ior >= 1` remain admitted; optional unsupported
fields retain their existing diagnostic fallback. Required clearcoat remains rejected
at the allowlist because its factor/roughness texture fields still have no binding
owner.

The preflight is `O(M)` in material count with constant field probes, performs no
texture decode or payload copy, and runs once per import. It changes no material ABI,
binding, sample, shader feature bit, permutation, or render-path cost. Acceptance
requires source-level red/green tests at both importer entry points for all three
unsupported required semantics plus preservation of supported factor-only admission;
managed Cargo remains a later exact-validation gate rather than a prerequisite for
implementing this fail-closed infrastructure slice.

The source implementation now exports one runtime-owned
`validate_required_gltf_material_extension_support` preflight and calls it from both
the builtin decoder and stable-plugin preflight before external buffers or images are
loaded. It retains the original required-extension set after the parser-only removal
needed by `gltf::Document`, probes only the three active material owners, and reports
the exact extension field plus material index. Static red evidence recorded the absent
owner/calls after both entry tests were added; static green evidence records one owner,
two calls, all three field guards, the typed error, successful `rustfmt`, and scoped
diff integrity. No Cargo test, product image, RenderDoc capture, timing, RSS, or power
claim is made, so the managed validation and M5-M8 gates remain open.

### 2026-08-29 glTF anisotropy factor-ingress plan

The remaining Standard-PBR projection review found an asset-to-shader contract break,
not another BRDF defect. `StandardPbrMaterialFeatures`, material normalization, the GPU
uniform projection, the shader feature bit, and the repaired direct/environment
anisotropic lobes already own `anisotropy_strength` and `anisotropy_rotation`, but the
shared glTF material-extension projector never reads `KHR_materials_anisotropy`.
Consequently an authored factor-only glTF asset silently renders isotropically even
though every downstream layer can represent its scalar semantics.

The Khronos extension defines `anisotropyStrength` in `[0, 1]` with default `0` and
`anisotropyRotation` as a finite angle in radians with default `0`; rotation is
counter-clockwise from the tangent direction. Without `anisotropyTexture`, the default
texel contributes the positive tangent direction and full texture strength, so those
two scalar fields are a complete representable subset. Bevy's glTF extension loader
projects strength and rotation independently of its separately feature-gated texture
path. Unreal's GLTF parser and material importer likewise read the two scalar fields
and texture as distinct inputs before material configuration. These references align
with Zircon's existing scalar material ABI and do not justify inventing a texture
binding in the importer.

The accepted MVP slice therefore extends the one runtime-owned projector rather than
duplicating logic in either importer. It projects the two scalar fields with their
normative defaults and validation into the existing property names, and emits the
existing explicit optional-field diagnostic when `anisotropyTexture` is present.
Both required-extension allowlists may advertise `KHR_materials_anisotropy` only after
the shared preflight rejects a required `anisotropyTexture`; required factor-only
assets are then admitted, while required assets whose RG/B texture semantics have no
binding owner fail closed before external image decode. Missing mesh tangents retain
the existing downstream orthonormal fallback; a texture implementation is excluded
because its UV-dependent direction/strength would require a declared color-space,
sampler/texture binding, texture-transform and tangent-owner contract.

Import-time cost stays `O(M)` for `M` materials with constant JSON probes. It adds no
frame-time ALU, texture sample, material-uniform slot, bind-group entry, PSO key,
shader feature bit, or permutation; factor-only assets merely activate capabilities
already compiled behind the existing anisotropy feature. Tests must first fail at both
importer boundaries for absent scalar projection/admission, then lock the default and
authored factor values, optional texture diagnostic, required factor-only acceptance,
and required-texture rejection. Managed Cargo/Naga/WGPU, anisotropic HDRI product PNG,
RenderDoc, GPU timing, RSS, and power gates remain separate and cannot be inferred from
the source-level closure.

The source implementation now adds `KHR_materials_anisotropy` to both required
allowlists only in conjunction with the shared preflight's
`anisotropyTexture` rejection. The one material-extension projector maps the
normative scalar defaults and authored values into the existing strength/rotation
properties and preserves an explicit diagnostic for optional texture use. Static TDD
evidence first observed that both tests required projection/rejection while the owner,
guard, and allowlists were absent; the green scan records one scalar projector, one
required-texture guard, both allowlists, no shader-binding expansion, successful
`rustfmt`, and scoped diff integrity. Empty anisotropy remains strength zero and does
not activate the existing Forward feature. No managed Cargo, Naga/WGPU, product PNG,
RenderDoc, timing, RSS, or power evidence exists yet, so no milestone advances.

### 2026-08-29 required clearcoat field-admission plan

The required-extension review found that clearcoat's blanket rejection is now more
conservative than the field-aware admission used for transmission, volume, IOR, and
anisotropy. Zircon already projects the ratified extension's factor and roughness with
their zero defaults, owns a separate clearcoat-normal texture slot, preserves its
linear normal-map usage, UV channel, `KHR_texture_transform`, and scale, and routes the
result through the Standard-PBR clearcoat layer. Geometry import also resolves the
clearcoat normal UV requirement and rejects unsupported tangent/attribute situations.
The only extension fields without shader/material owners are the scalar multiplier
textures `clearcoatTexture.r` and `clearcoatRoughnessTexture.g`.

Khronos defines factor, roughness, and normal as independent optional inputs, and an
absent factor/roughness texture contributes one. Unreal parses and maps all five inputs
independently before selecting the clearcoat material path. Therefore an asset that
requires the extension but authors only factor/roughness and an already-supported
normal texture does not depend on either missing texture semantic. Keeping it rejected
does not protect correctness; it rejects a representable asset based on fields that
are absent.

The accepted infrastructure slice adds the ratified extension to both required
allowlists only after the shared runtime preflight rejects either unsupported scalar
texture when clearcoat is required. Optional occurrences retain the projector's
explicit diagnostics. This preserves fail-closed behavior for unrepresented R/G
modulation while admitting exactly the owned subset, including existing clearcoat
normal validation downstream. The scan remains `O(M)` with two constant field probes,
runs before external image decode, and adds no frame-time instruction, texture sample,
binding, uniform, material ABI field, PSO key, feature bit, or permutation.

Red/green tests must cover both importer preflights, positive required factor-only
admission, both rejected texture fields, and preservation of the existing optional
diagnostics. This source closure cannot advance a milestone without fresh managed
Cargo/Naga/WGPU, clearcoat product PNG, RenderDoc, timing, RSS, and power evidence.

The implementation now admits required clearcoat in both importers and folds the two
missing clearcoat texture fields into the same fixed guard table as anisotropy,
transmission, and volume. That table produces the exact `extension.field` error before
external image decode, while the existing scalar and clearcoat-normal projection is
unchanged. Static red evidence observed factor-only admission and both texture guards
missing; green evidence records both allowlists, one declarative five-field guard
loop, retained normal projection, successful `rustfmt`, and scoped diff integrity.
There is no new runtime shader work and no managed/product evidence, so M5-M8 remain
open.

### 2026-08-29 diffuse-transmission glTF disposition

The last scalar in `StandardPbrMaterialFeatures` without a glTF mapping is
`diffuse_transmission`, but `KHR_materials_diffuse_transmission` is not a safe
factor-only ingress target. As of this review the Khronos extension is a Release
Candidate, not a ratified contract, and its BSDF requires an independent three-channel
transmission color plus optional alpha factor texture and sRGB color texture. Zircon's
current ABI owns only one scalar and colors its diffuse BTDF through the existing base
color/volume path. The checked-in Unreal and Bevy references do not import this
extension, while the Khronos Sample Renderer does own its separate factor and color
inputs.

Mapping only `diffuseTransmissionFactor` would therefore advertise an extension while
silently discarding authored color and texture semantics and applying a different
color owner. No mapping or required allowlist is added. Required assets continue to
fail at the generic extension allowlist, and a complete future implementation must
first define the color factor/texture ABI, color space, binding and texture-transform
owners, composition with specular transmission/volume, and measured permutation and
sample cost. This is a structural compatibility disposition, not an optimization or
acceptance result.

### 2026-08-29 remaining Standard-PBR parameter and energy recheck

The post-ingress review traced every current `StandardPbrMaterialFeatures` scalar from
material properties through normalization, uniform slots, surface projection, and the
Advanced shader. Clearcoat, anisotropy, specular transmission, thickness, IOR, and
attenuation now have representable glTF ingress; the private diffuse-transmission
scalar has the explicit Release-Candidate disposition above. Ratified glTF specular,
sheen, iridescence, dispersion, and the remaining factor/color textures have no
partial allowlist claim, so no required asset can silently pass by relying on them.

The lighting composition was then re-read at its actual owners. Direct diffuse starts
with base color and one metallic complement, then consumes the Fresnel already
computed by the isotropic or anisotropic GGX lobe exactly once. Ambient and environment
diffuse each multiply base color once and call the same metallic/Fresnel energy owner;
environment specular derives F0 once from dielectric F0/base color/metallic and does
not receive a second base-color multiplier. Specular transmission applies base color,
entry Fresnel complement, metallic complement, factor, and volume attenuation once.
Diffuse transmission is mutually excluded when specular transmission is active.

Advanced composition also retains the intended layer ownership: base diffuse,
transmitted diffuse, base specular, specular transmission, and emission receive the
single view-dependent clearcoat base-energy scale; direct and environment clearcoat
lobes do not receive it again. The environment base specular is attenuated before it
joins retained reflection, while direct base specular is already attenuated in the
per-light result, preventing a second final multiply. This matches the selected
Khronos layering structure and the Unreal separation of base and coat lobes.

No production edit follows this recheck. It adds no ALU, sample, binding, uniform,
feature bit, PSO key, permutation, or loop work. The recorded direct-anisotropy
once-per-light frame preparation remains a profile-gated optimization; it must not be
moved until managed 1/8/32-light disassembly, GPU distribution, and power evidence
identify it as a material bottleneck. Current-source compiler/product gates remain
open.

### 2026-08-29 optional unsupported glTF material diagnostics plan

The importer-policy review found one remaining observability asymmetry. Khronos
explicitly permits non-required material extensions to fall back to core PBR, and
Zircon correctly rejects unowned extensions when they appear in
`extensionsRequired`. For optional use, however, only `KHR_materials_specular`
currently records that the imported result differs from the authored material.
Ratified dispersion, iridescence, and sheen, the Release-Candidate diffuse
transmission extension, the Initial-Draft subsurface extension, and archived
specular-glossiness are silently ignored by both importers because they share the same
projector.

The fallback itself must remain successful, but it needs one runtime-owned diagnostic
per present unsupported extension so product/editor tooling can explain the loss of
semantics. A fixed registry in the shared material projector is sufficient: it probes
the six material extension objects, records the exact extension name and that core PBR
fallback was retained, and leaves the existing more specific specular diagnostic in
place. Required admission remains governed by the separate top-level allowlists, so
this helper cannot accidentally turn an unsupported required extension into success.

Import complexity adds six constant probes per material, `O(M)` overall, and allocates
diagnostic strings only for assets that actually use an unsupported optional
extension. It adds no frame work, material field, texture slot, shader code, binding,
sample, feature bit, PSO key, or permutation. Red/green coverage must exercise both
importer material paths and assert all six names; source guards must also prove none of
them entered either required allowlist. Managed Cargo and product evidence remain
open.

The source implementation now owns those six names in one fixed projector registry.
Both importer tests retain successful core-PBR material creation and require every
extension name in diagnostics; static red evidence observed zero production names,
while green evidence records the complete registry, one diagnostic template, and zero
required-allowlist entries in both importers. `rustfmt` and scoped diff integrity pass.
No managed Cargo, product, RenderDoc, timing, RSS, or power claim is made.

### 2026-08-29 reflection-probe layer and selection boundary review

The `09F1/P0-7` review was rechecked against the current probe upload, GPUScene, mesh
visibility, and WGSL owners before changing the 64-probe fragment loop. The CPU
prepare path first intersects each probe with the selected camera layers, ranks the
remaining candidates by camera distance/priority, and uploads at most 64 probes. The
GPU record nevertheless preserves `probe.layer_mask()` only as the low 32-bit,
scene-schema-v1 value bitcast into `GpuReflectionProbe.misc.w`; no WGSL code reads that
lane. `zr_environment_select_probes` receives only world position and linearly ranks
every uploaded probe for every shaded fragment.

The missing half is not a local shader parameter. Mesh visibility still owns the full
`RenderLayerSet`, represented as a growable `Vec<u64>` and tested for intersection on
the CPU. Neither `GpuPrimitiveData` nor `GpuInstanceData` carries an object reflection
mask, and the fragment shading/environment APIs receive no equivalent. Treating the
two remaining primitive padding words as an implicit low-32-bit mask would silently
drop layers 32 and above, duplicate the explicitly lossy scene-v1 conversion, and
still leave deferred, sprite/particle, planar/capture, and future clustered paths with
different semantics. Reading `misc.w` in WGSL without this receiver input would be a
false fix.

The accepted structural direction remains the `09F1` design: visibility owns one
persistent-generation `ReflectionProbeSpatialAssignment`; it intersects the complete
object/camera/capture/reflection layer sets before producing bounded per-cluster or
per-object candidate ranges. Forward and deferred consume the same packed
offset/count/index ABI and retain the current spatial weight, priority, top-two blend,
box projection, and sky fallback only within that local list. Overflow must be
explicit and deterministic, never a silent mask truncation. Before implementation,
the profile must record current admitted-probe counts and 1/8/32/64-probe GPU timing,
fragment/list visits, occupancy/overflow, upload bytes, RSS/VRAM, and power with the
same scene and resolution. This audit makes no production change and does not claim a
scan optimization, layer repair, timing result, or milestone advance.

### 2026-08-29 `KHR_materials_specular` structural disposition

The ratified Khronos extension was reviewed after the factor-only anisotropy and
clearcoat admission slices. It cannot be projected into the current scalar IOR owner.
The extension independently defines a dielectric `specularFactor` with default one,
a linear-RGB `specularColorFactor` with default white and values allowed above one,
an alpha-channel strength texture, and an sRGB color texture. Strength changes both
dielectric F0 and F90, color changes F0 but not F90, the resulting F0 is clamped, and
the metallic BRDF must remain unaffected. Diffuse energy is recovered with the scalar
maximum of the RGB dielectric Fresnel rather than a component-wise complement.

Zircon currently derives one achromatic dielectric F0 from IOR, mixes it with the
metal base color before the shared GGX Fresnel, assumes white F90, and uses the
resulting vector complement for diffuse. Deferred hard-codes `vec3(0.04)`. Mapping
only either factor would therefore change authored color, grazing response,
metal/dielectric separation, transmission reflection ratio, and energy ownership.
Required `KHR_materials_specular` remains fail closed and optional use retains its
explicit diagnostic.

A complete future slice must add factor and color fields to the material property and
GPU ABI, force non-default use through Forward until the GBuffer contract is expanded,
separate dielectric and metallic Fresnel inputs in the shared direct/environment
helpers, carry explicit dielectric F90, and reuse the same scalar max-Fresnel energy
owner for direct diffuse, IBL diffuse, and transmission. Texture admission follows
only after alpha/sRGB channels, sampler, UV, and texture-transform owners exist.
Acceptance requires the Khronos SpecularTest factor/color rows, IOR and transmission
interaction fixtures, Naga/WGPU validation, RenderDoc inspection, and GPU/permutation
measurements. No production ABI or shader change is made by this disposition.

### 2026-08-29 reflection-probe workload telemetry plan

The current probe prepare path already computes the information needed for a before
profile but discards its `ReflectionProbeUploadReport` at `write_scene_uniform`.
Instrumentation must use that owner rather than add shader atomics or a second report.
One structured workload report will expose extracted probes, camera-layer candidates,
attempted candidates, active/resident probes, candidates deterministically dropped by
the fixed capacity, newly uploaded cubemap count/bytes, and asset rejections through
the existing `RenderStats` product surface.

The report may additionally calculate
`render_width * render_height * active_probe_count` with saturating `u64` arithmetic,
but it must be named a full-resolution fragment-probe visit *upper bound*. It is not a
measured shaded-fragment count, does not include overdraw or reduced shading rate, and
cannot be presented as GPU work eliminated. The CPU cost is constant increments in the
existing prepare loops plus one stats copy; there is no extra allocation, GPU buffer,
binding, readback, shader instruction, texture sample, PSO key, or permutation. The
later 1/8/32/64-probe profile combines this workload context with existing scene-pass
GPU timestamps, RenderDoc, RSS/VRAM, and power evidence before any local-list
optimization is implemented.

### 2026-08-29 reflection-probe workload telemetry implementation

The source slice now implements the planned observation boundary without changing the
probe shader or its resource ABI. `RenderReflectionProbeWorkloadReport` is the public
frame-stat contract. The existing prepare pass records all extracted probes, probes
whose full CPU `RenderLayerSet` intersects the camera layers, eligible candidates that
actually enter asset/slot resolution, resident active probes, capacity-dropped eligible
candidates, newly scheduled cubemap uploads and payload bytes, and asset rejections.
The submitted-frame stats path copies that report and derives the full-resolution
fragment-probe visit upper bound with saturating `u64` multiplication.

The counter semantics preserve failure attribution. Camera-layer candidates may still
lack a baked cubemap or have zero intensity. Attempted candidates have passed those
gates. Capacity-dropped candidates are the eligible count minus attempted count after
the invalid-nearest replacement loop, so an asset rejection that causes a farther
candidate to be evaluated is not mislabeled as capacity pressure. Upload bytes count
the validated PMREM payload newly appended to the frame upload batch; they do not claim
queue submission or GPU completion.

The hot path adds one counter increment for each layer match, one for each attempted
candidate, one saturating byte addition only for a new cubemap upload, one final
subtraction, and one structured stats copy. It adds no allocation, shader atomic,
GPU buffer, readback, bind group, texture sample, feature bit, PSO key, permutation, or
loop. Test-first static red evidence found the report type and four probe accounting
fields absent from production. The source contract then turned green, exact `rustfmt`
completed, and scoped `git diff --check` passed. Resource tests lock exact PMREM upload
bytes and both invalid-nearest replacement and healthy-overflow attribution. The
existing public WGPU product contract now also checks enabled two-probe, feature-off
two-probe, and subsequent sky-only frames through `query_stats`, including clearing
current-frame values rather than retaining prior uploads. Managed Rust/WGPU execution,
screenshots, RenderDoc replay, 1/8/32/64-probe GPU timing, RSS/VRAM, and power evidence
remain open; this is profiling infrastructure, not a performance improvement claim.

### 2026-08-29 reflection-probe 1/8/32/64 before-profile fixture

The existing public WGPU probe product contract now owns a manual 1080p profiling
entry rather than a second renderer harness. It keeps one framework, viewport, camera,
mesh, mirror material, quality profile, and shared PMREM fixed while changing only the
active probe count through 1, 8, 32, and 64. Sharing the PMREM removes residency and
upload growth from the intended shader-scan comparison while the workload report still
proves the active count and visit upper bound for every submitted frame.

Each case warms 16 frames, then accepts 120 asynchronous GPU profiles only when their
`frame_generation` belongs to post-warmup submissions for that case. Generations are
deduplicated. The JSON keeps every `RenderFrameProfile`, including per-pass timing and
pipeline statistics when available, and additionally records GPU frame min/p50/p95/p99
and the workload snapshot. This preserves raw evidence for later re-analysis instead
of reducing the run to one average.

The optional `ZR_RENDERDOC_CAPTURE_REFLECTION_PROBE_64=1` request is issued only after
all timed cases complete, on a separate final 64-probe frame. The fixture writes
`runtime_environment_reflection_probe_linear_scan_before_profile_20260829.json` and
`runtime_environment_reflection_probe_linear_scan_64_20260829.png` beneath
`docs/tests/runtime/shader`; it never targets C. Source formatting and scoped diff
integrity pass, but the ignored test has not run. No GPU percentile, RenderDoc capture,
RSS/VRAM, or power result exists yet, so no local-list implementation is authorized.

### 2026-08-29 reflection-probe cold-residency call-graph review

The P0-6 synchronous-I/O risk is confirmed in current source rather than inferred from
the public asset API. A reflection-probe slot miss enters
`ReflectionProbeResources::prepare`, calls `ProjectAssetManager::load_texture_asset`,
then `load_typed`, which unconditionally calls `ensure_resident`. If the resource is not
already present in `ResourceManager`, `ensure_resident` prepares a project artifact read
and executes `PreparedProjectArtifactRead::read()` on the calling thread before cloning
the texture payload and scheduling the PMREM upload. Resident slot hits bypass this
path; the 1/8/32/64 steady-state fixture deliberately shares and warms one PMREM, so its
GPU scaling data cannot be presented as cold-residency latency evidence.

This is an architectural boundary, not permission for an isolated shader-side cache.
The correct target is a 09D-owned asynchronous residency/prefetch contract that admits
only ready revision snapshots to render prepare, retains retry semantics for I/O and
generation supersession, and lets Shader06 preserve last-good/environment fallback
without blocking the frame. Before that migration, a separate process-cold profile must
measure artifact-read/decode duration, first-frame CPU time, queued upload bytes, and
time-to-first-resident for a unique cubemap; the steady-state probe corpus remains the
GPU local-list baseline. No production path changes in this review, and no latency,
power, or asymptotic improvement is claimed.

### 2026-08-29 focused managed compile result

The Windows managed validator ran locked `zircon_runtime` build plus the focused
`runtime_environment_reflection_probe_product_contract` target in the coordinator test
lane at
`D:/cargo-targets/zircon-engine/pool/f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`.
The library build stopped before test execution with 32 diagnostics. Two belonged to
this telemetry slice: both probe consumers imported the new report from the public
render facade while `render/mod.rs` had not re-exported it. The report is now included
in the existing `backend_types` facade list, and exact `rustfmt --check` plus scoped diff
integrity pass after the repair.

The remaining 30 compile errors are outside this slice and span the in-flight
`zr_contracts::random` wiring, text/font services, post-process visibility, UI binding,
GPUScene previous transforms, material streaming, and platform host service. In
particular, multiple current random sources already import `zr_contracts` while the
mixed manifests do not yet expose that crate, matching the active Frameworks01
manifest-transfer dependency. No second Cargo request was launched against the known
broken shared snapshot. The focused WGPU product contract therefore remains unexecuted,
and this result does not authorize the ignored performance fixture, screenshot,
RenderDoc capture, milestone acceptance, or manifest release.

### 2026-08-29 reflection-probe cold-load telemetry implementation

The workload report now separates probe admission from the confirmed synchronous asset
boundary. `asset_load_call_count` increments only when a cubemap revision misses the
probe slot cache and enters `load_texture_asset`; `asset_load_cpu_time_us` accumulates
the elapsed CPU time around exactly that call, including any `ensure_resident` artifact
read and the returned texture clone. Failed loads are counted and timed, while missing
registry revisions never claim an asset-load call. Resident slot hits, feature-off,
and sky-only frames execute neither `Instant::now` nor asset loading.

This adds two clock reads only per cold slot-miss call and no allocation, GPU query,
shader work, binding, sample, pipeline, or permutation. Resource contracts require one
call on first valid/rejected cold use, zero on resident reuse and disabled frames, and
no overflow asset work after capacity is filled. The public product contract correlates
successful calls with scheduled uploads and requires zero timing on no-call frames; the
manual JSON preserves both fields. Exact formatting, scoped diff integrity, and a
source scan proving one timer owner around one load owner pass. Managed compilation is
still blocked by the recorded foreign current-worktree diagnostics, so no nonzero timing
sample or cold-latency conclusion is claimed.

The same current-source recheck narrows the old P0-6 upload statement. Probe upload no
longer performs 8 mip x 6 face writes. `append_probe_pmrem_texture_uploads` shares one
immutable payload and emits eight `WgpuTextureUpload` records, one per mip with all six
faces represented as array layers. Frame buffer and texture setup share one logical RHI
resource-upload ticket, but production RHI flush still iterates those records and calls
`queue.write_texture` eight times per new cubemap; it does not encode one staging-buffer
copy batch. `scheduled_texture_write_count` now records that exact count beside upload
bytes, with contracts requiring eight for one successful PMREM and zero for resident,
disabled, rejected, or capacity-dropped paths. Moving these writes to a reusable staging
arena/copy encoder remains a 09A/09D change and requires cold-upload CPU/GPU/submission
evidence; Shader06 does not duplicate that backend here.

The probe upload lifecycle is transactionally correct but the first telemetry names were
not. Both direct and compiled render paths enqueue the combined resource-upload batch
before scene submission, while `commit_pending_uploads` runs only after
`submit_graphics_command_buffers_with_frame_diagnostics_and_surface` returns a scene
ticket. An enqueue error, receipt error, or scene-submit error returns without making the
slot `Ready`; the next `prepare` clears the stale pending vector, advances the prepare
epoch, and schedules the cubemap again. An already accepted resource upload can therefore
be repeated after a later scene-submit failure, which wastes cold-path bandwidth but is
required for correctness because the failed frame cannot publish residency.

To preserve that distinction in performance data, the new public/internal fields are
named `scheduled_cubemap_upload_count` and `scheduled_cubemap_upload_bytes`, rather than
`uploaded_*`. `scheduled_texture_write_count` likewise describes RHI records appended by
the probe owner, not successful queue writes or GPU completion. A resource contract drops
an uncommitted first batch and requires the next prepare epoch to schedule the same PMREM
and its eight mip writes again. Existing source-order contracts cover direct and compiled
scene submission before slot commit. No additional render-thread work, backend hook, or
retry loop is added; accepted-but-unpublished upload waste must be measured with submission
failure injection before any 09A/09D transaction redesign.

## 2026-08-29 reflection-probe capture hard-cut and target architecture

The P0-8 problem is more fundamental than a slow implementation. Commit
`7a20f921b` removed `SceneRenderer::render_scene_color_hdr` and
`SceneRenderer::last_transient_resource_pool_report` together with the old
`scene_renderer_hdr_capture.rs` owner. Retained HDR capture now belongs to the
neutral `RenderFramework` viewport contract: `capture_scene_color_hdr` finishes
the current submission, takes the framework operation/state locks, waits for
readback completion, and then copies the retained RGBA16F scene color into a
`CapturedHdrFrame` containing `Vec<[f32; 4]>`. The optional reflection-probe
runtime still calls both deleted `SceneRenderer` methods, while its editor
trigger still accepts a raw `&mut SceneRenderer`. Therefore this plugin boundary
is stale and cannot compile when targeted; it must not be repaired by restoring
the deleted renderer API or by adding a compatibility shim.

The current plugin intent also remains structurally synchronous. It prepares six
camera variants, renders and reads back one face at a time, transforms each face
to cmft layout, retains all six RGBA32F faces, builds source mips/PMREM/SH9 on the
CPU, and persists `.zcube/.zribl` before returning. The public captured-face
entry calls the serial mip/PMREM owner even though `SourceCubemapMipChain` has a
separate parallel-executor API. Moving the stale call mechanically to six
temporary framework viewports would restore the same six submit/wait/readback
barriers and keep job orchestration in the plugin/editor, so that is also
rejected as the target architecture.

For face size `S`, the unavoidable lower bounds of the stale CPU route are:

| Retained data | Bytes | 64 | 128 | 256 | 512 | 1024 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Six RGBA16F readback payloads | `48*S^2` | 0.188 MiB | 0.750 MiB | 3 MiB | 12 MiB | 48 MiB |
| Six decoded RGBA32F capture faces | `96*S^2` | 0.375 MiB | 1.500 MiB | 6 MiB | 24 MiB | 96 MiB |
| Capture faces plus newly allocated full source mip storage | at least `224*S^2` | 0.875 MiB | 3.500 MiB | 14 MiB | 56 MiB | 224 MiB |

The last row is not a peak-memory claim. Readback staging, the per-face returned
vector, PMREM/IEM outputs, artifact encoding, and allocator overhead coexist for
part of the operation, so the real peak is higher. CPU filtering also scales
with produced texels and its sampling quality after the six scene renders; no
timing or power conclusion is inferred from the byte model.

Unreal's useful reference is ownership rather than a claim that its current
branch is already stall-free. `CaptureSceneIntoScratchCubemap` constructs six
face views/renderers beneath one `FSceneRenderBuilder` and executes that builder
once. Every face is copied into a shared GPU cubemap; the subsequent graph builds
mips, diffuse irradiance and the filtered cube, then copies the filtered result
directly into the scene cubemap array for shading. CPU radiance persistence is a
separate optional readback stage. That same source currently inserts a
`FlushRHIThread` plus `SubmitAndBlockUntilGPUIdle` pass after each face render,
so Zircon must measure backend command-memory and scheduling limits instead of
assuming that merely using one graph guarantees one native submission. The
portable invariant is one GPU-resident capture job with no mandatory per-face
CPU readback, not an unmeasured fixed submission count.

cmftStudio supplies the complementary UI/job lesson only. `cmftFilterFunc` runs
filtering on a background thread and exposes Started, Completed, ExitSuccess and
ExitFailure states for UI polling. Its mutable shared output has no scene
revision, cancellation, immutable publication, or GPU-residency contract, so it
is not a runtime scheduler design to copy.

The hard-cut target is one dependency-ordered environment-capture service:

1. Framework exposes neutral request, handle, progress/terminal status and
   output identity DTOs; it does not own plugin behavior or a raw renderer.
2. Graphics runtime owns a bounded scheduler with queued, capturing, filtering,
   persisting, succeeded, failed, cancelled and superseded states. Each job
   snapshots scene/environment revision, capture transform, quality, budget and
   output generation before any GPU work.
3. The renderer records six capture views into one GPU cubemap job. A backend may
   split submissions only through an explicit measured budget; no face requires
   conversion to `Vec<[f32; 4]>` or a synchronous CPU wait.
4. Existing GPU source-mip, PMREM, SH9/IEM graph owners consume that cubemap
   directly. Only a complete current generation may publish to reflection-probe
   residency; cancellation, device loss, scene revision change or filter failure
   retains last-good/sky fallback.
5. Optional `.zcube/.zribl` persistence begins after GPU completion through one
   asynchronous cubemap readback/write operation. Editor manual capture, runtime
   on-demand capture and cook tooling submit the same request and observe the
   same handle; editor code never owns `SceneRenderer`.

Implementation order is foundation-first: C0 removes the stale public boundary
and establishes the neutral job/state contract; C1 adds the bounded runtime
scheduler, generation/cancellation/failure rules and telemetry without a second
capture implementation; C2 records the six-view GPU cubemap capture; C3 connects
the existing GPU IBL graph and atomic residency publication; C4 adds optional
asynchronous persistence plus editor/cook consumers; C5 performs product,
failure, revision, cancellation and performance acceptance. C0-C4 are
non-validation production work and must precede claims based on screenshots.

The before/after protocol fixes scene, source revision, face size, quality,
backend, driver and output hash, then records CPU submission and blocked time,
GPU face/capture/filter time, submission and explicit-wait counts, readback
count/bytes, artifact latency, RSS/VRAM peak, cancellation/supersession latency,
WPR/WPA power interval and RenderDoc graph/resource identity. At minimum the
default 128 and product 256 sizes need five cold/five warm samples; 64/512/1024
are scalability points, not substitutes for the product comparison. The current
plugin cannot produce a valid before run because its hard-cut dependency no
longer exists, and the shared tree also retains the recorded foreign compile
failures. No capture optimization, GPU timing, power improvement, screenshot,
RenderDoc acceptance or milestone advancement is claimed by this review.

## 2026-08-29 C0 source implementation status

The C0 foundation slice is now implemented on the planned ownership boundary. The
neutral render contract defines a validated request with capture id, scene and
environment revisions, output generation, capture transform, face size and source
prefilter quality. It exposes a stable handle and a nonblocking status model with
queued, capturing, filtering, persisting, succeeded, failed, cancelled and
superseded phases. `RenderFramework` supplies default unsupported
request/poll/cancel capabilities so existing backends remain source-compatible
until the bounded runtime scheduler is added.

The reflection-probe plugin now maps its JSON request into that neutral contract
and exposes only request/poll/cancel wrappers. The editor trigger submits the same
request and no longer owns `SceneRenderer`, a cache directory, synchronous face
readback, or asset registration. The old synchronous six-face CPU execution path
and its report were removed; persisted artifact consumption remains a typed
`PersistedReflectionProbeCapture` input for the later asynchronous persistence
slice. Exact Rust formatting, scoped diff integrity, stale-symbol elimination and
ownership-boundary source guards pass. This is source-level infrastructure only:
no Cargo/Naga/WGPU run, GPU timing, PNG, RenderDoc replay, RSS/VRAM or power data
is claimed, and C0 does not advance M5-M8 or the open realtime-SH9 Failure.

## 2026-08-29 C1 bounded scheduler source implementation status

C1 now supplies the graphics-runtime control plane that C0 deliberately left
unsupported. `EnvironmentCaptureScheduler` retains at most one pending scene
snapshot, one active work item, and 64 terminal statuses. A complete duplicate
request reuses its live handle. A newer `output_generation` for the same
`capture_id` supersedes queued work immediately and marks active work with a
terminal intent; a late GPU success therefore cannot publish a stale output.
Cancellation is idempotent, progress is monotonic and bounded by the six-work
item contract, and terminal status retention is explicitly evicted at a fixed
capacity. A bounded capture-id generation ledger also rejects replayed older
requests after a terminal result, while active phase and work-item progress are
both monotonic. Scheduler telemetry counts acceptance, duplicate, capacity,
stale generation, supersession, cancellation, success, failure and eviction
events.

`WgpuRenderFramework` exposes only the request/poll/cancel control-plane calls
under a dedicated scheduler mutex. Those calls do not finish frame submission,
take the renderer state lock, or start a background thread. The scheduler's
`begin_next`/`advance_active`/`finish_active_*` hooks are intentionally not wired
to GPU recording yet; C2 must consume the work item at the existing
`SceneRenderer` realtime-IBL graph owner, preserve its last-good double-buffered
publication, and measure command-memory/submission limits before choosing a
backend submission policy. Exact formatting, diff integrity, capacity/ordering/
cancellation/lock-boundary source contracts pass. No Cargo/Naga/WGPU, GPU
timing, PNG, RenderDoc replay, RSS/VRAM, power or performance result is claimed;
M5-M8 and the realtime-SH9 Failure remain open.

## 2026-08-29 C2 cubemap projection/capture source contract

Before recording scene geometry, the six capture views must share the existing
projection convention. `CubemapFace::projection_axes` exposes only the texture
U, texture V and face-forward axes derived directly from the `FACE_UVN` table
used by `cubemap_texel_direction` and equirectangular conversion. Calling these
three values a camera basis was incorrect: the cmft/D3D cubemap convention has
`U x V = -forward`, whereas Zircon's right-handed `Transform` uses local `-Z`
as forward and `looking_at` derives screen-right as `forward x image_up`.

Consequently, selecting image-up as `-V` makes a normal Zircon camera's
screen-right equal `-U`. Six ordinary `looking_at` cameras would therefore
write horizontally mirrored cube faces. Unreal avoids this by constructing a
D3D cube-face view matrix with `right = up x direction`, which is a reflection
relative to Zircon's camera transform. The C2 scene-capture projection must
express that clip-X reflection explicitly and account for its winding reversal;
the private point-shadow face camera is not a valid drop-in cubemap writer.

`cubemap_capture_view_from_world` now owns that conversion. It maps cubemap U to
view X, cubemap V to negative view Y and face-forward to view `-Z`, while
applying capture-origin translation in the same matrix. `cubemap_capture_camera`
packages the same reflected projection with 90-degree FOV, 1:1 aspect, request
clip planes, HDR, no temporal jitter, and the requested face size. Both surfaces
expose `reverses_winding=true`; material draw construction XORs that view bit
with the existing model negative-determinant flag before selecting
`PipelineKey::reverse_raster_winding`, without expanding `ShaderVariantKey`.
The renderer must consume this bit when selecting the raster pipeline instead of
silently relying on ordinary camera winding.

The contract is deliberately pure CPU metadata: tests assert each forward axis
matches its face center, all axes are unit length and pairwise orthogonal, lock
both the cubemap handedness and right-handed-camera reflection relation, and
verify origin/U/V projection, the request camera's projection equivalence, the
negative matrix determinant, and model/view winding XOR. It does not allocate
textures, encode commands, perform readback, or claim any visual or performance
result. C2 GPU work remains responsible for constructing
one scene-capture job, recording six layers under the existing frame submission
budget, and handing the resulting source cubemap to the current source-mip,
PMREM, and SH9 graph without changing last-good publication semantics.

The first implementation slice now exposes `cubemap_capture_camera(face,
request)` as the request-to-view adapter. It uses one 90-degree perspective
projection, request near/far and face size, HDR, 1:1 aspect, one sample, and no
temporal jitter. Mesh draw construction receives a capture-view winding bit and
combines it with the existing per-instance determinant bit by XOR, so a mirrored
model does not accidentally get culled twice. This keeps the CPU setup at
`O(6 + M)` for six face descriptors plus `M` scene meshes; it does not deep-copy
the scene six times and does not add a shader permutation. The adapter is still
metadata-only until C2 records all six faces into one GPU-resident source cube.

The second CPU-side slice adds `EnvironmentCaptureSceneBatch`. It consumes the
snapshot packet once, moves it into one `RenderFrameExtract`, strips editor
overlays and virtual-geometry debug metadata, and mutates only the selected
camera descriptor for each canonical face. The extract `Arc` identity remains
stable across all six selections, so the GPU recorder can prepare resources and
build the `M` mesh draws once rather than cloning or rebuilding them per face.
The batch deliberately owns no texture or command encoder: C2 is not complete
until one RGBA16F six-layer source cube and per-face scene constants are recorded
under one accepted queue submission, after which scheduler progress may advance
to `Capturing 6/6` and the existing source-mip/PMREM/SH9 graph may take ownership.

The target allocation is intentionally a separate owner from the procedural
realtime IBL double buffer. That buffer is fixed at 128-face and is not a render
attachment, while capture requests admit 64..1024 power-of-two faces. The new
`EnvironmentCaptureGpuTargetPlan` computes the complete RGBA16F cube mip-chain
and one sequentially reused Depth32 face before allocation; its 128-face budget
is `1,114,096 B` and its 1024-face upper bound is `71,303,152 B`. The target
exposes one sampled Cube view, six mip0 color-face views, per-mip D2Array storage
views, and one depth view. This makes memory admission explicit and prevents a
capture from silently aliasing the published last-good IBL slot.

The capture mesh entry point selects the explicit environment-capture pipeline
feature set and disables snapshot-inexpressible virtual geometry, fog, and
command-cache sidebands. It retains one direct-light preparation and one mesh
draw census; the view reflection is XORed with the model determinant flag in the
existing pipeline key. No shader variant dimension or per-face scene copy is
introduced. GPU render-pass recording, accepted-submission progression, and
source-mip/PMREM/SH9 handoff remain the next C2 work item.

The existing scene group cannot be rewritten once per face into the same
uniform buffer before a single submission: all six passes would observe the
final write. `EnvironmentCaptureSceneUniformPlan` therefore derives all six
`SceneUniform` values from the shared extract, packs them into one immutable CPU
payload, and emits six non-overlapping upload ranges. The active-job workspace
owns six exact-size uniform buffers and six bind groups while sharing the
existing environment textures, sampler, BRDF LUT, SH9 buffer, and scene layout.
This keeps face selection out of shader permutations and avoids dynamic-offset
alignment waste; its GPU constant capacity is exactly
`6 * sizeof(SceneUniform)`. The later recorder must append all six writes to one
resource-upload ticket before recording the six color passes and must not
publish capture progress until both that ticket and the graphics submission are
accepted.

The scheduler-to-recorder ownership boundary is also explicit: the queued
`EnvironmentCaptureWorkItem` is re-exported through the graphics runtime and
offers a consuming `into_parts()` transfer. `EnvironmentCaptureSceneBatch` can
therefore take ownership of the scene snapshot and request after the scheduler
mutex is released. Resource preparation and command recording never need to
hold the control-plane lock or clone a queued snapshot; this remains a CPU
ownership contract until an accepted GPU submission advances scheduler
progress.

The concrete framework consumer now mirrors that boundary. `WgpuRenderFramework::begin_environment_capture_work_item`
locks only `environment_captures`, calls `begin_next`, and returns the owned
work item. It does not drain frame submissions or acquire operation/state
locks, so the future renderer recorder can build target, uniforms, and draws
after the mutex is released.

The capture draw profile is now explicit about the view-dependent boundary. It
enables opaque, alpha-mask, and advanced-PBR-opaque consumers, while disabling
base transparency, transmission, shadows, velocity, and OIT. This matches
Unreal's `r.ReflectionCapture.Runtime.Translucency` default of `false` in
`ReflectionEnvironmentCapture.cpp`: a single shared mesh census is valid for
the MVP only when it does not claim transparent ordering. Enabling transparency
later requires per-face depth ordering of `T` transparent draws, with at least
`O(6T)` CPU sort/build work and a separately budgeted GPU pass policy; it must
not be smuggled into the six-face shared draw list.

`EnvironmentCaptureRenderPlan` now provides the recorder's CPU-side input
contract. It binds the six canonical face ordinals to unique mip0 array layers
and uniform slots, carries the reflected-view winding bit and opaque-only
policy, and reuses the target plan's full source-mip memory budget. The plan
contains no WGPU handles and does not enter ordinary viewport submission, so it
cannot accidentally add a per-frame capture cost or claim a native submission
count. Its static tests lock `6 == RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT`,
face/layer/slot identity, and the 1024-face `71,303,152 B` upper bound. The
WGPU recorder now consumes this plan with the moved scene batch and six uniform
bind groups. It builds one opaque command set and replays only opaque,
alpha-mask, and advanced-PBR-opaque streams into the six face attachments; a
per-face sky pass clears color/depth and the mesh pass uses load/store. The
capture-specific builder runs only `OpaqueBasePassProcessor`, reducing source
processor visits from the generic builder's `6M` (depth, shadow, opaque,
transparent, velocity, TAA across `M` batches) to `M`; the necessary GPU replay
remains `6K` for `K` admitted capture commands. It never builds or records
transparent/transmission streams. Renderer-owned upload/submission, accepted
progress, and GPU-resident source-cube handoff to source-mip, PMREM, and SH9 are
still open. No Cargo, executed WGPU, product PNG, RenderDoc, timing, RSS/VRAM,
or power result is inferred.

The framework boundary is now symmetric: `begin_environment_capture_work_item`
transfers ownership out of the scheduler, while progress and failure remain
scheduler-only. Successful settlement is the deliberate exception: it holds
the scheduler mutex, invokes the physical `Publish` or `Discard` callback under
renderer state, then exposes terminal status. It still does not drain pending
frame submissions or acquire the operation lock. This fixed lock order gives
the recorder one explicit ownership path for acquire, GPU acceptance,
cancellation/supersession-aware failure, physical publication, and terminal
visibility.

The moved scene packet has an equally strict semantic boundary. The current
`EnvironmentCaptureSceneBatch` owns the already extracted mesh, light,
environment, and camera data in `SceneViewportRenderPacket`; it intentionally
removes editor overlays and virtual-geometry debug metadata. It does not imply
that animation, particle, sprite, or other sideband producers are represented
in the capture. The recorder must not re-extract those producers once per face
or fabricate a successful capture when the packet lacks them. Full-scene
reflection requires a separately reviewed extension of the shared packet and
extract owner, followed by parity evidence for both ordinary viewport and
capture paths. Until that contract exists, the six-face plan is an opaque
geometry/environment MVP with `O(M + 6)` CPU preparation, not a claim of full
scene visual parity.

## 2026-08-29 C2/C3 GPU filtering and completion review

The source transaction now records capture and filtering under one renderer
owner. Six RGBA16F mip-zero faces are followed in the same graphics command
buffer by the existing realtime-IBL RGBA16F downsample pipeline, the canonical
IBL PMREM command/pipeline cache, and the canonical SH9 kernel. The generic
runtime mip generator was rejected because its storage ABI is RGBA8; adding a
capture-only conversion would duplicate the current HDR owner. Capture PMREM
quality scales only the existing sample budget: Fast is half Normal with a
16-sample floor, Normal is unchanged, and High is twice Normal.

The static work and allocation models are now explicit:

| Face size | Raster passes | Source-mip dispatches | PMREM | SH9 | Peak GPU bytes | Resident PMREM+SH9 bytes |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 128 | 6 | 7 | 8 | 1 | 2,162,800 | 1,048,704 |
| 1024 | 6 | 10 | 8 | 1 | 72,351,856 | 1,048,704 |

The prior `1,114,096 B` / `71,303,152 B` C2 figures counted only source plus
depth and are therefore historical lower bounds, not the current target peak.
The current 64-entry residency owner has an exact filtered-output upper bound
of `67,117,056 B`. It replaces one capture id atomically and evicts at fixed
capacity. Conversion to resident output drops source, source views, storage
views, face attachments, and depth; only the PMREM texture/view and SH9 buffer
survive.

The renderer submits one resource-upload ticket and one graphics ticket. A
later successful viewport frame runs the renderer's existing sole nonblocking
completion pump; the capture service only queries the resulting ticket states.
Both tickets must be `Completed`. Failure terminates the job without replacing
last-good; cancellation or supersession consumes the completed transient output
without publication. No capture path calls `device.poll`, waits for a ticket,
or performs CPU readback. Accepted raster/filter work publishes scheduler phase
`Filtering 6/6`, matching what was actually encoded rather than leaving the job
mislabelled as Capturing.

This does not yet close C3. The neutral request identifies a capture with a
string, while `SceneReflectionProbeResources` publishes through a `ResourceId`
slot and reflection-probe placement carries a separate numeric `u64 probe_id`.
Neither identity is present in the render request. Guessing a slot by parsing
the string or matching position would make generation-safe publication
impossible. The next C3 contract must carry a typed target identity from the
plugin/editor request through the scheduler and copy the completed PMREM into
the selected probe-array slot before terminal success. C4 asynchronous
persistence remains optional and must reuse the existing product-diagnostic
readback/writeback owner after GPU publication. No Cargo/Naga/WGPU, DX12 PNG,
RenderDoc replay, GPU timing, RSS/VRAM, WPR/WPA power, or performance conclusion
is claimed by this source implementation.

## 2026-08-29 typed target review and publication boundary

The request identity gap is now closed at the API boundary: placement
validation derives a stable `ResourceId` from the persisted PMREM URI and
forwards it with the placement's numeric `u64 probe_id`. The capture request
and output identity expose that typed pair, so a future array publication step
can be generation-checked without parsing a capture string or matching probe
position.

The first direct array-copy implementation was rejected during review. It
acquired and committed a probe-array slot in the same call that submitted the
copy, while ordinary frame prepare could observe the slot before the copy
completed. That violates last-good visibility and can overwrite a pending slot
with a CPU asset upload. The code was removed; the accepted path publishes only
the bounded filtered GPU output. C3 remains open until a submission-ticket-owned
copy/commit state machine is implemented and validated. No Cargo/Naga/WGPU,
product PNG, RenderDoc, timing, RSS/VRAM, WPR/WPA, or power result is claimed.

## 2026-08-31 capture evidence and Base-profile state review

The existing `zircon_shader_pbr_viewer` v16 evidence owner validates imported-HDRI
`metal-mirror` and `dielectric-ior` fixtures. The separate ignored
`reflection_probe_linear_scan_before_profile` product test measures sampling an
already resident shared PMREM at 1/8/32/64 active probes. Neither path executes
the scene-capture raster transaction, so neither can prove capture direct-light
inclusion, the full-roughness material override, or capture-raster cache-key
invalidation. Historical PNG/RDC artifacts remain non-current context. A fresh
scene-capture product run and RenderDoc replay are still required; adding fields
to the imported-HDRI sidecar would conflate two different products.

The source review also found a state-consistency defect in the lit capture path.
Direct lights require the generic Forward Base profile, so capture disables the
environment-only registry profile before building its one mesh command set. The
ordinary viewport owner already clears cached mesh commands on that profile
transition, but the capture owner did not. A later viewport could therefore
reuse a command stream built under the old profile after the registry had moved
to generic. Capture now mirrors the established transition rule and clears the
cache only when the profile changes from enabled to disabled. This is one
transition-time invalidation, not per-face or per-frame work; no measured timing,
power, or throughput improvement is claimed. The exact RED/GREEN source guard,
focused `rustfmt --check`, and scoped diff-integrity check pass. Managed Cargo,
WGPU, current-source PNG, RenderDoc, GPU timing, RSS/VRAM, and WPR/WPA remain
open.

The request-identity review found one additional canonicalization defect.
Scheduler duplicate detection compares `RenderEnvironmentCaptureRequest` using
floating-point value equality, while `ibl_bake_key` hashes the position's bit
pattern. IEEE `-0.0` and `+0.0` were therefore duplicate-equal but produced
different runtime-cache keys. Request construction now canonicalizes every
finite zero component to `+0.0`; nonzero finite values retain their exact bits
and non-finite values remain rejected. The regression locks request equality,
bake-key equality, and the canonical stored bit. This is constant construction
work with no frame, draw, dispatch, binding, or allocation delta.

The capture recorder/filter reports were also audited as optimization inputs.
They already computed command replay and filter-construction facts, but those
values terminated inside the retained submission/output object. Successful
capture events now publish 17 profiler counters: face passes, command builds,
commands per face, three command classes, draw calls, state changes, bind skips,
source-mip/PMREM/SH9 dispatches, source-mip parameter-buffer/bind-group creation,
the already measured source-mip binding-creation microseconds, and IBL
parameter-buffer/bind-group creation. This adds no new clock read, query set,
GPU readback, iteration, allocation, or device poll. It is measurement
infrastructure only; a current-source run must correlate these counters with GPU
timestamps, RenderDoc events, RSS/VRAM, and WPR/WPA before changing command,
binding, or filter algorithms.

Typed probe publication now also fails closed at submission admission. The
previous `Option::and_then` path converted both a missing target resource
revision and a refused `CapturePending` slot reservation into `None`, after
which the scheduler could report `Succeeded` even though no probe-array copy
had been recorded. An explicit target now requires both revision resolution and
reservation; either error terminates the capture before resource-upload or
graphics submission, while a request without a target retains the generic
resident-output path. This supersedes the earlier wording that capacity pressure
could silently skip publication. Last-good remains visible because no slot is
committed or replaced on the failed admission path.

The follow-up admission-order review found that revision lookup still happened
after the capture target, six-face command set, and filter graph had been built.
On an environment-only renderer, the same invalid target could first expand the
local reflection provider. The full provider's two texture allocations have a
static format/layout floor of `78,292,648 B` (`67,107,840 B` for the 64-entry
128-face PMREM array plus `11,184,808 B` for the 1024 planar mip chain), excluding
buffers, views, allocator overhead, and driver alignment. The capture target adds
`2,162,800 B` at face size 128 or `72,351,856 B` at 1024. These are source-derived
capacity bounds, not measured RSS/VRAM. The selected cut resolves the target
revision before either provider expansion or capture-target construction and
reuses that exact revision at the later reservation. The mutable reservation is
deliberately not moved: doing so would require an explicit rollback guard across
every subsequent fallible upload, material, pipeline, recorder, filter, and
diagnostic path. Current-source GPU timing, allocation, RSS/VRAM, and power
evidence remain required before claiming a measured performance improvement.

## 2026-08-30 C4 runtime-cache completion boundary

The explicit capture path now reuses the bounded product-diagnostic readback
queue through one shared submission ticket. Capture-originated writeback items
are marked optional through completion: readback assembly errors and runtime
cache write errors remove only that pending item, preserving the submitted
capture and its last-good resident output. Graph-owned artifact writeback keeps
strict error propagation. Editor asset-derived staging and managed WGPU,
RenderDoc, timing, memory, and power validation remain open.

The reflection-probe request now has an explicit optional source-hash field.
When the asset/editor owner supplies that hash, the plugin constructs the
canonical request identity and enables runtime-cache persistence; without it,
the existing capture remains non-persistent. This avoids deriving artifact
identity from a capture id, placement, or destination URI, and it does not
relabel a GPU runtime artifact as an `AssetImporterCpu` staged asset.

## 2026-08-31 terminal publication atomicity review

The current completion path has two independent publication steps. It first
locks the capture scheduler and exposes `Succeeded` (and, for persistence, the
source payload), then releases that lock and commits the reserved probe slot and
resident filtered output under renderer state. A control-plane poll can
therefore observe terminal success while the physical output is still absent.
Reversing those two calls is also invalid because cancellation or supersession
can arrive between them and would permit stale physical publication.

The selected hard cut makes the scheduler the single publication gate. While
holding the scheduler mutex it validates the active handle, phase, progress,
terminal intent, source-payload layout, and mailbox capacity. It then invokes
one narrow publication callback exactly once: `Publish` commits the reserved
probe slot and residency under renderer state; `Discard` cancels the
reservation without replacing last-good. Only after that callback returns may
the scheduler publish `Succeeded` and make a source payload visible. The lock
order is fixed as scheduler then renderer state; control-plane request, poll,
and cancel remain scheduler-only, and no state-to-scheduler path is introduced.

The hard cut is implemented. `EnvironmentCaptureScheduler` now separates
control-plane and completion responsibilities, and the former 1,135-line file
is 892 lines after the mailbox regression, with the extracted production
modules. The old bool-returning
success APIs and post-terminal publication path are removed. Focused Rust
formatting and source-order/old-API contracts pass. Transient source/depth
scratch is converted and released before the scheduler mutex is acquired; the
locked callback retains only constant-time probe commit/cancel and bounded
residency-map publication. An E-drive `rustc --test` harness compiled the real
scheduler/control-plane/completion modules and reports 16/16, including the
runtime-only-success/persistence-mailbox isolation regression. Its source-bound receipt is under
`E:/zircon-profiles/shader06-environment-capture-atomicity-20260831/`. It is not
a crate/WGPU compile ticket. Managed Cargo remains pending.

This is a lifecycle/correctness repair, not a measured performance result. It
adds no frame pass, draw, dispatch, sample, PSO, shader permutation, queue,
readback, allocation, or device poll. The scheduler remains bounded to one
pending and one active capture. Pure state-machine regressions must prove that
the callback sees nonterminal status and no ready source payload, that publish
precedes terminal/source visibility, and that cancellation and supersession
invoke only `Discard`. Managed Cargo, WGPU, RenderDoc, timing, memory, and power
evidence remain pending.

## 2026-08-31 capture view-neutral scene and LOD ownership review

The six-face capture path still accepts `SceneViewportRenderPacket`, whose mesh
entries are already projected for the caller viewport. `World::visit_render_mesh_snapshots_for_camera`
selects a concrete base/LOD model, mesh, material, and primitive list from the
viewport camera position before constructing each `RenderMeshSnapshot`.
`EnvironmentCaptureSceneBatch` then moves that packet once and changes only the
camera descriptor for each cubemap face. Consequently, a probe far from the
viewport camera can rasterize the viewport's LOD and even the wrong primitive
or material resource set. Clearing `mesh_lod` cannot repair this because that
field is only metadata; the selected resource handles have already replaced the
view-neutral source.

The Unreal reference does not reuse a caller viewport's selected mesh packet.
`CaptureSceneIntoScratchCubemap` creates an `FSceneView` at the capture position
for every face, marks it as a reflection capture, and applies the skylight LOD
distance factor before scene visibility/LOD selection. Zircon does not need six
independent full CPU extracts for distance-threshold LOD because all faces share
one origin, but it does need the same authoritative view-neutral primitive
source before building the reusable six-face command set.

The repository already contains the correct lower-layer shape. The neutral
`RenderComponentChangeArtifact` owns base/all-LOD resource bindings in immutable
`Arc` slices, and `RenderSceneComponentProjector` projects those changes into a
persistent `RenderScene` whose primitives retain the complete
`RenderSceneMeshSource`. That owner is not yet scheduled into `SceneRenderer`;
the active environment-capture API still receives the legacy viewport packet.
Adding all LODs to `RenderMeshSnapshot`, cloning `MeshRenderer` into the capture
queue, or reconstructing another capture-only scene cache would create a third
scene representation and violate the Render03 residency/journal design.

The selected dependency-ordered cut is therefore:

1. Render03 wires the existing `scene_changes -> RenderScene` projector into the
   renderer and publishes one generation-qualified read view backed by the
   unified all-LOD residency owner.
2. The capture work item retains that scene generation plus lighting/environment
   inputs, not a viewport-selected mesh vector. At capture admission it performs
   visibility/layer policy and one distance-based LOD selection per persistent
   primitive using the probe origin.
3. Resource preparation and mesh-command construction consume those selections
   once; all six faces replay the same command set while changing only view
   uniforms and raster orientation. A scene-generation or residency mismatch
   fails closed before recording rather than falling back to viewport-selected
   resources.
4. The old `SceneViewportRenderPacket` capture overload is removed in the same
   hard cut. No compatibility path may preserve the camera-bound source.

The intended CPU scale is `O(M log L + K + 6)` for `M` candidate primitives,
maximum per-primitive LOD count `L`, and `K` selected primitive bindings; with
the current sorted thresholds, selection can use `partition_point`. Command
construction, asset resolution, and material binding remain once per capture,
not once per face. Stable scene generations must report zero source projection,
zero all-LOD payload clone, and zero redundant resource admission. The managed
E-drive profile must compare a probe near/far from the viewport with at least
three LODs and different per-LOD primitive/material bindings, record candidate
visits, LOD selections, selected bindings, command builds, CPU p50/p95/p99,
allocations/bytes, GPU capture timestamps, VRAM/RSS, and WPR/WPA energy, then use
RenderDoc to prove all six faces consume the probe-origin selection. No elapsed,
power, or improvement claim is made from this static review. Status is
`architecture_review_complete_blocked_on_render03_persistent_scene_wiring`;
shader/capture work continues on independent MVP gaps.

## 2026-08-31 capture cold-procedural specular fallback repair

The capture surface policy forces base and clearcoat roughness to one, but the
raw procedural fallback has no PMREM and deliberately samples the perfect
reflection direction for ordinary viewports. Before a realtime IBL slot had
ever been published, environment capture reused that fallback and could bake a
sharp procedural sun reflection into a probe despite the full-roughness policy.
Waiting implicitly was not a valid contract: the scheduler admitted the job and
the capture path had no ready-generation prerequisite.

The minimal shader-owned cut keeps the existing viewport fallback and every
ready source/realtime PMREM path unchanged. In the single
`zr_environment_sky_reflection_color` owner, a capture with no PMREM now returns
zero for the unfiltered specular term. Diffuse sky lighting, authored ambient,
direct lights, emissive surfaces, and the directly rendered sky background use
separate paths and remain present. This adds no ABI field, binding, feature bit,
permutation, PSO identity, texture, upload, or CPU work. On that capture-only
cold path it avoids one sharp procedural-radiance evaluation and lets the
existing zero-reflection guard skip the environment BRDF lookup; those are
source facts, not measured GPU savings.

An assembled Forward regression preserves the order `ready PMREM -> capture
fail-closed -> ordinary viewport reflected-direction fallback`. The pre-change
source guard failed because the capture branch was absent; the post-change
contract, focused Rust formatting, and diff integrity pass. Managed Naga/WGPU,
pixel, RenderDoc, timestamp, memory, and power evidence remain pending, so the
status is `implemented_static_contract_passed_pending_managed_gpu_profile`.

## 2026-08-31 capture direct-light shadow ownership review

The lit capture path packs direct lights and builds six face-specific light
grids, but its dedicated command builder excludes shadow commands and its
forward receiver is created without `ShadowAtlasResources`. Every admitted
direct light is therefore evaluated against fallback unshadowed metadata. This
is not parity with the Unreal reference: reflection capture starts from the Game
show-flag set, disables post processing, motion blur, particles and other named
features, but does not disable shadows before constructing the scene renderer.

A caller viewport shadow atlas cannot be reused as the repair. Directional
cascades are camera-volume products and can be invalid at the probe; rebuilding
the generic plan for each face would multiply caster traversal and command
construction by six. Point/spot shadow views are light-owned, while directional
capture coverage needs an explicit probe-volume policy. The selected structural
direction is a generation-qualified capture lighting product: reuse light-owned
point/spot/static products when their scene/light/caster generations match,
define one bounded directional probe-volume shadow policy, then bind that exact
product to all six face receivers. Missing or stale shadow data must be typed and
observable; it may not silently convert a shadowed light to unshadowed.

The target scale is one caster census and one shadow preparation per unique
light product, followed by six color replays; `O(6M)` shadow-plan or caster
rebuild is rejected. Measurement must separately report caster visits, shadow
views, commands, atlas bytes, reuse hits, CPU/GPU p50/p95/p99, VRAM/RSS, and
WPR/WPA energy for 0/1/8 shadowed lights. RenderDoc must prove the six capture
passes bind the capture-qualified shadow generation. This review makes no source
change and no performance claim; status is
`architecture_review_complete_pending_capture_shadow_product_owner`.

## 2026-08-31 source-independent direct diffuse convergence

The current source review covered both Forward Standard-PBR closures, Deferred
lighting, the standalone fallback, environment and lightmap composition, shared
isotropic/anisotropic GGX, transmission, and clearcoat before changing code. The
hybrid defect was exact: environment, ambient, and lightmaps already consumed
the material-owned `base_color * (1 - metallic)` diffuse term, while Forward
Basic/Advanced, Deferred, and fallback multiplied that same term by a per-light
`1 - F(VoH)`. Thus one prepared material BRDF acquired a light/view-dependent
diffuse albedo and the four direct paths disagreed with the rest of the engine.

The primary Unreal evidence remains `BasePassPixelShader.usf`, where Default Lit
publishes `GBuffer.DiffuseColor = BaseColor - BaseColor * Metallic`, and
`ShadingModels.ush` / `ForwardLightingCommon.ush`, where Lambert consumes that
diffuse color while Fresnel stays in the specular closure. Bevy's direct-light
composition follows the same ownership shape. The hard cut therefore makes
`zr_pbr_isotropic_ggx` and `zr_aniso_ggx` return only the specular BRDF, deletes
`ZrPbrSpecularComponents` and both `*_ggx_components` wrappers, and composes the
prepared metallic Lambert term directly in Forward Basic/Advanced, Deferred,
and fallback. Clearcoat and transmission retain their own Fresnel energy gates.

This changes no material/GBuffer ABI, binding, texture sample, feature bit,
shader permutation, PSO identity, loop count, or bake recipe. Direct lighting
remains `O(number_of_visited_lights)` and continues to prepare the metallic
diffuse BRDF once outside the light loop. Removing the tuple-like component
owner and wrapper calls is a source/IR simplification only; no compiler
instruction, GPU-time, power, or allocation improvement is claimed without the
managed current-source profile. Static RED captured all four old consumers;
GREEN reports zero legacy component symbols in production WGSL and all six new
owner/composition contracts present. Managed Cargo/Naga/WGPU, HDRI PNG,
`D:\Tools\renderdoc`, GPU p50/p95/p99, RSS/VRAM, and WPR/WPA energy remain open,
so M5-M8 and coordinator commit status do not advance.

## 2026-08-31 realtime IBL failure-state architecture closure

The pre-change review followed the complete realtime path from frame snapshot,
time-slice scheduling, graph recording, queue submission, publication, and the
public diagnostics facade. Both recording and submission failures collapsed to
one boolean `Retry`, retained the same ticket indefinitely, retried on every
eligible frame, and exposed no reason or terminal receipt. A persistent driver,
shader, resource, or submission error could therefore consume a realtime IBL
job slot forever while leaving callers unable to distinguish an active retry
from a permanently failed generation.

The hard cut keeps the existing ready/work atomic-publication model and adds one
scheduler-owned failure state machine. Consecutive failures are counted per
logical stage, not per whole bake. Attempt one skips one complete frame, attempt
two skips two, and attempt three is terminal. Success resets the stage counter.
The terminal key is suppressed until source identity changes, while the previous
published key/slot remains sampleable. Runtime terminal handling clears both the
frozen active snapshot and latest-wins queued snapshot so an abandoned
generation cannot be revived implicitly. Reports carry bake key, generation,
logical state/substep, operation, typed recording/submission cause, attempt
count, next eligible frame, terminal flag, and last-good availability through
the existing renderer/framework ownership chain. A failure-only public getter
was rejected because it could not distinguish fallback, initial bake, ready,
last-good refresh, or the two terminal availability outcomes. The final public
owner is one allocation-free `RealtimeIblStatusReport`: it adds published,
pending, and queued identities, current/start frame, coalesced source-change
count, and a six-state readiness enum while retaining the optional typed failure.
The framework finishes pending submission before reading this snapshot, and no
parallel failure-only facade remains.

The runtime frame counter already wraps at `u64`, so retry eligibility uses a
half-range sequence comparison and wrapping target addition. This keeps the
bounded delay valid across counter rollover: a first failure at `u64::MAX`
skips frame `0` and becomes eligible at frame `1`.

Unreal's `FRealTimeSlicedReflectionCapture` ready/work/state progression remains
the primary lifecycle reference; Zircon deliberately retains its own explicit
last-good and generation-key semantics rather than copying engine-private retry
policy. The constants are private to the scheduler because they are local MVP
fault policy, have one production owner, and are not yet a user-facing quality
profile. The successful hot path remains constant-state and does not add GPU
work. Retry complexity is bounded to three attempts for one stage, so a
permanently failing key changes from unbounded work to `O(1)` attempts and
storage. This is an algorithmic liveness bound, not elapsed-time, energy, or GPU
throughput evidence.

Static source/facade contracts and focused formatting pass. An E-drive harness
directly compiled the production scheduler with `rustc 1.94.1 -O` and exercised
successful publication, all six readiness states, 1/2-frame backoff, terminal
last-good preservation, failed-key suppression, published-key restoration,
new-key recovery, delayed-completion rejection through per-attempt frame identity,
bounded frame age, and frame-counter rollover. Source and binary SHA-256 are
`C0BF937A1F9A51FDB031834515B46821DABBA9E0F830683281BB3764015A6EA3` and
`1A015F79BC5731A716E82291687415F1001C88F3280A90FC9B592FC691E22718`.
Managed Cargo/WGPU failure injection, public diagnostic product evidence,
current-source PNG/RenderDoc, timing, RSS/VRAM, and WPR/WPA remain required.
Status is
`implementation_complete_static_and_isolated_behavior_green_pending_managed_fault_product_validation`.
The post-repair independent review reports `Critical 0 / Important 0`: the old
attempt completion is stale and cannot clear or advance its same-stage retry.

### Realtime IBL last-good freshness product follow-up

The six-state report still left last-good freshness to consumers. Unreal keeps
capture status and `SecondsSinceLastCapture` in the same sky-light owner; the
equivalent Zircon boundary is the runtime that atomically publishes the ready
slot. `RealtimeIblRuntime` now records the successful publication frame and
adds published frame, last-good age, and inclusive active-generation elapsed
to the existing status report. The scheduler's half-range wrapping sequence
helper is the sole age comparison owner, including the pre-first-slice zero and
`u64` rollover cases. Retry, terminal failure, and restored published identity
retain the timestamp; only another successful publication replaces it.

This is fixed state and integer projection, not a performance optimization. It
adds no wall clock, allocation, lock, GPU query, binding, dispatch, sample, PSO,
or permutation. Production behavior regressions are written but have not run
under managed Cargo. The current E-drive scheduler harness verifies the shared
sequence primitive and reports the source/binary hashes above. Status is
`implementation_complete_static_and_isolated_sequence_green_pending_managed_status_product_validation`.
Independent follow-up review reports `Critical 0 / Important 0 / Minor 0` for
pre-first-slice elapsed, publication age, rollover, terminal/restoration
retention, and successor-publication replacement.

## 2026-08-31 BRDF LUT cold-start profile and hard-cut architecture

The current architecture has one correct GPU owner but still performs the
wrong work in that owner's synchronous publication boundary. The first
`SystemTextureGenerationOwner::acquire` holds the publication mutex and invokes
a process-wide `OnceLock` initializer. That initializer integrates a 128x32
split-sum table with 128 samples per texel and encodes it before the generation
lease can exist. Later device generations reuse the CPU payload, but the first
renderer must pay all 524,288 GGX/Hammersley iterations.

An E-drive harness directly included the current production integrator and used
a counting allocator. Thirty-one independent optimized Windows processes each
performed exactly one build. Min/p50/p95/p99/max wall times were
20.967/22.086/36.618/40.505/40.505 ms (mean 23.888 ms); every process reported
one 32,768-byte allocation, 4,096 texels, 524,288 sample iterations, and checksum
3171.452236790. The production integrator and system-owner SHA-256 values were
`0DBD5D51E783D60D74C1FCE3E50588E02C1B6061A26551BBC3A75CE8D9351C5F`
and `761BF0117B0E775BE012F2A966586CE5C3E3E56A97B5ED191D9F727624C80349`.
The harness, executable, raw data, and summary are source-bound under
`E:/zircon-profiles/shader06-brdf-lut-startup-20260831/`; their hashes are
recorded in optimize plan 09F1 section 12.30. This isolated profile excludes
half encoding, WGPU work, renderer startup, memory residency, and energy.

Unreal remains the primary algorithm and ownership reference: its
`SystemTextures.cpp` builds the same 128x32, 128-sample PreIntegratedGF table,
and `BRDF.ush` fixes the RG scale/bias consumers. Its synchronous generation is
not adopted as a startup-performance requirement. The alternative analytic
`EnvBRDFApproxLazarov` changes the numerical contract and is rejected until the
base, clearcoat, and SSR consumers share an error-qualified recipe. cmft and
cmftStudio contain no split-sum LUT publication owner, so they do not justify a
parallel runtime path.

The selected cut is a checked-in, versioned RG16F builtin artifact certified by
the existing integrator. A neutral recipe identity owns algorithm version,
extent, sample count, format, payload length, and content hash. Certification
regenerates the bytes and compares them exactly; the release path materializes
and uploads the immutable payload and never falls back to runtime integration.
The existing per-device-generation GPU owner, drop order, one upload ticket,
and scene leases remain unchanged. Startup diagnostics must report builtin
materialization and zero payload-build time instead of treating first-process
integration as a cache build.

This changes startup CPU complexity from `O(width * height * samples)` to
`O(payload bytes)` for 16,384 bytes while leaving shader ABI, texture sampling,
bindings, passes, PSOs, permutations, and LUT content unchanged. Source work is
pending exact reconciliation of the untracked system-texture owner and startup
diagnostic dependency set; splitting that behavior across owners is forbidden.
After implementation, the same 31-process profile, byte certification, product
startup report, DX12 screenshot/RenderDoc replay, and matched WPR/WPA interval
must prove that the integration bottleneck disappeared without visual or power
regression. Status is
`profile_confirmed_hard_cut_planned_pending_exact_owner_reconciliation`.

The pre-integration artifact feasibility step is complete on E:. The current
production integrator plus canonical RG16F encoder generated a 16,384-byte
payload with SHA-256
`406956356B136BD079CDCCE8DCB86F9E20D596681F7457AD38D91A7EE472674D`;
an independent second generation produced identical bytes. Thirty-one cold
processes materializing that static payload into the current `Arc<[u8]>` upload
ABI measured min/p50/p95/p99/max 22.7/28.9/55.5/74.6/74.6 microseconds and one
16,400-byte allocator charge per process. The isolated p50 ratio against the
production integrator was 764.22x. This establishes artifact feasibility only:
the payload is not in the repository, the product owner still invokes the
integrator, and no WGPU, DX12, RenderDoc, whole-startup, or energy result exists.
The status therefore does not advance.

## 2026-08-31 realtime procedural stale-finding disposition

The current source no longer matches three early optimization findings. The
default realtime scheduler admits at most two capture faces per frame and
publishes an unretried generation after 21 accepted slices, so first-generation
capture, source mip, PMREM, and SH9 work is not submitted as one frame-sized
batch. This is a state-machine work bound, not measured GPU time, and does not
close first-resource or first-PSO startup timing.

The default policy is already key-driven OnChange. A published key with no
active ticket rejects the same rebake request; changing input while active
keeps only the latest snapshot. The key covers radiance-changing colors,
effective sun state, source revision, and shared capture-source identity while
excluding final-sampling intensity and rotation. A static procedural sky pays
one required generation to obtain PMREM/SH9 and then stops. Adding Static,
threshold, interval, importance, or EveryFrame controls before a cooked
procedural artifact owner exists would expand the MVP API without removing a
repeated workload.

Procedural sky math also has one WGSL definition. Viewport environment paths
call `zr_procedural_sky_radiance` through `zr_environment_core.wgsl`; realtime
capture concatenates the same `zr_procedural_sky.wgsl` with a cube-direction and
storage wrapper. The shared source identity participates in the bake key. P1-3,
P1-4, and P1-5 are therefore source-closed or reclassified for the MVP, while
managed Naga/WGPU, image, RenderDoc, GPU timing, and energy gates remain open.

## 2026-08-31 general IBL command-plan structural profile

The general artifact bake path still has a quadratic CPU construction shape.
Each of its ten WGPU executors reconstructs the request, builds all ten PMREM,
SH9, and IEM commands, then linearly selects one command. One complete bake
therefore builds 100 command descriptors and their parameter/resource/readback
containers. The realtime topology cache is a different, bounded owner and its
slice recorder already builds an exact runtime kernel; it neither fixes nor
should absorb the general artifact path.

The pre-change harness under
`E:/zircon-profiles/shader06-ibl-command-plan-20260831/` directly compiled the
current production invocation builder, shader plan, and command plan. The only
profile-copy edit points the existing `realtime_slice` module at its absolute
source path. For a 512/10-mip source and the canonical 128/8-mip
`PMREM_SH9_IEM` output, one full plan contains ten commands and 55 readback
copies and allocates 396 times / 41,570 bytes. The current ten-pass recording
shape allocates 3,960 times / 415,700 bytes; one build plus ten lookups allocates
396 times / 41,570 bytes.

Across 31 independent processes with 128 scenario iterations each, current
ten-pass p50/p95/p99 was 1,716,197.7/2,848,416.4/3,053,585.2 ns. The isolated
one-build/ten-lookup feasibility p50/p95/p99 was
141,324.2/472,246.9/481,593.8 ns, a 12.14x p50 ratio. These timings link existing
debug dependency artifacts and are not product frame, WGPU, GPU, or energy
evidence. The deterministic allocation counts and 100-versus-10 command count
are the algorithmic evidence.

Unreal's `FComputeShaderUtils::AddPass` captures one shader, parameter block,
and group count in each RDG pass, and reflection capture allocates parameters
per mip/face during graph authoring. Zircon's long-term owner should likewise be
an immutable compiled-pass payload. Its current neutral compute metadata cannot
yet represent IBL uniform words, mip-scoped storage views, and artifact readback
copies, so widening the whole render-graph ABI is deferred rather than hidden in
this MVP repair.

The selected MVP hard cut constructs only the exact current pass command,
including its readback contract. It changes total construction from `O(P^2)` to
`O(P)` without adding a global cache, PSO key, permutation, dispatch, or legacy
fallback. The five-file production dependency set remains foreign modified
source and has not received exact ownership reconciliation, so no production
edit was made. Status is
`profile_confirmed_o_p_squared_hard_cut_planned_pending_exact_owner_reconciliation`.

## 2026-08-31 P1-9 direct-cosine IEM ownership recheck

The previous P1-9 wording treated the CPU direct-cosine integrator as though it
were a renderer production path. A repository-wide call review found one
non-test caller: `asset/importer/environment_ibl.rs`. Default HDR/EXR import
requests `PMREM_SH9`; only the explicit
`environment_ibl_irradiance_cube=true` setting requests `PMREM_SH9_IEM` and
invokes the direct integrator. `staged_bundle_state == Current` returns before
cubemap reconstruction or IEM work, so unchanged imports consume the durable
source/artifact pair rather than reconvolving it.

The renderer has a separate GPU PMREM/SH9/IEM graph for a missing or stale
runtime artifact. Runtime hydration decodes the accepted artifact and builds
the immutable upload payload; it does not call the CPU direct integrator.
Tests use the CPU path as a parity/reference oracle. The resulting ownership is
already the intended MVP shape: GPU runtime, cached opt-in CPU cook, and CPU
oracle/high-quality offline capability.

The local references agree with that separation. cmft's command-line/offline
irradiance filter projects the source to SH and reconstructs each output texel
from the coefficients. Unreal reflection capture allocates convolution and
diffuse-irradiance work as GPU RDG passes. Neither reference supports moving an
optional cached cook algorithm into the frame path or optimizing it without a
representative import profile.

No production code was changed and no performance improvement is claimed. P1-9
is reclassified as
`runtime_path_closed_opt_in_cook_profile_retained`. If real opt-in cook traces
later show `irradiance_cube_build` dominating, the before/after gate must cover
the same HDR corpus and record numerical error, phase p50/p95/p99, allocation
and RSS, plus WPR/WPA energy before selecting SH reconstruction or GPU cook.
Reviewed SHA-256 values are:

- CPU IEM source: `7C859E43E4F60C70306BF0DCE76DE09B4170A732E9A9CE555C0177A35063B937`.
- asset importer: `128157450A2C9D5078552E93F1912E0AC3C5260089C6CCAD7C455C607589DB40`.
- import settings: `491F0AC4724265CDA9112D2005239179E5BB2018856CAB43E5372B5F0DD3ED46`.
- artifact hydration: `5C87FBE12C355F705D61279A101D32B8D316E17F0D0AC38888B4996B232F5648`.
- upload builder: `03BB7CCFBF4194785BD2012E9353DF929B3CC9BF30A6F78EC275424171DB1748`.
- cmft filter: `FB31E35A0ADA6A3502FAAC5C0236D232F1DEFB6F1315D8D60F6C4FE1028F56A8`.
- Unreal capture filter: `19D9A1B52CD08A3AE80A00501EDAE8F283FA63DF7C3F3D985BFEF48F2A195C02`.

## 2026-08-31 P1-18 AmbientLight/lightmap source-policy closure

The asset and scene component already exposed `affects_lightmapped_meshes`,
but render extraction dropped it and every Forward, fallback, and Deferred path
added the same ambient sum after baked indirect. This was a cross-pipeline
contract defect, not a lightmap filter problem. The accepted boundary keeps
09F2 as the baked-lighting energy owner and makes 09F1 own only whether an
ambient source is eligible for a surface with a resident lightmap.

`RenderAmbientLightSnapshot` now carries the flag. `SceneUniform::from_frame`
uses one O(A) fold to produce the existing all-mesh ambient sum and a second
lightmapped-mesh sum; there is no second light scan or per-instance CPU work.
The uniform ABI grows by one aligned `vec4<f32>` (16 B), and every shader that
declares fields after ambient mirrors the insertion. Forward/generated and the
fallback mesh shader select the sum using the existing GPU Scene instance
lightmap bit.

Deferred does not allocate another GBuffer channel. The existing emissive
target is `Rgba16Float`; its alpha was not consumed by lighting. GBuffer encode
stores the same lightmap-presence bit there, and full plus environment-only
Deferred lighting decode it from the existing emissive `textureLoad`. This
adds no MRT, binding, texture sample, shader feature, permutation, PSO key,
allocation, or per-light/per-probe loop. The selection is one comparison and
one `select` per shaded fragment.

Static RED first observed all five owners absent: snapshot field, uniform
field, shared selector, GBuffer bit, and Deferred selector. GREEN finds all
five, finds no complete SceneUniform ABI that omits the inserted field, and
scoped diff-integrity checks pass. Production and assembly tests cover scene
extraction, the two ambient sums, Forward/fallback selection, GBuffer encoding,
full Deferred selection, and environment-only Deferred selection.

The existing isolated E-drive `naga 29.0.3` probe was extended to assemble the
full Deferred shader and reads all production WGSL at execution time. It now
passes full Deferred 1/1, environment-only Deferred 1/1, fallback 1/1, skybox
variants 2/2, and normalizer delegates 6/6. Probe source/binary SHA-256 are
`1738DA460B1CF5535F45297C5E70EFAD0A8FD6AA561ACE34A971E29413DB37AC` and
`D8E7D5E589B67B875A1F36405D00B11090751B2D536B713A97CDB10E5AC64EAC`.
This did not parse workspace manifests and is not WGPU or image evidence.
Managed Rust/WGPU tests, current-source `docs/tests/runtime/shader` PNG,
`D:\Tools\renderdoc`, GPU timing, RSS/VRAM, and power remain required. Status
is `implementation_complete_static_and_isolated_naga_green_pending_managed_wgpu_and_product_validation`.

## 2026-08-31 P1-6 cold-fallback/bootstrap disposition

The procedural fallback still samples the sky directly for diffuse and
specular, so specular does not broaden with roughness. The six-state realtime
IBL report now exposes that result as `Fallback` or `FailedFallback`; it is not
accepted as ready PBR output. The remaining question is how to shorten the
cold interval without moving the complete bake into an unbounded first-frame
spike.

The default scheduler requires 21 accepted one-operation batches: three
two-face captures, seven source-mip generations, three two-face PMREM mip-zero
batches, seven whole-face PMREM mips, and terminal SH9. The batch array is
fixed at one operation, and topology identity, attempt token, stale-completion
rejection, retry/backoff, and timing attribution all use that operation as the
atomic unit. The reviewed source hashes are:

- scheduler: `3E642C150E3B64106C054CDA4EEB537FF3A9AB0C92510DEC189E025A133721F7`.
- graph plan: `E808CB793521AF9A097577A7C551914723A293BBFA17765E3FE52C031F816DF6`.
- runtime: `FB3E2F4D0F29BE789362789C22DAEA47BED9B559AFC72252B92D382B0D3C76CD`.

Collapsing the all-face work into one submission would still record roughly
17 GPU passes: one all-face capture, seven source mips, eight PMREM mips, and
SH9. It reduces frame boundaries, not filtering complexity, and can increase
cold-frame command, GPU, and power peaks. A partial change to only the batch
array would also desynchronize topology caching and retry/completion identity.

No production optimization is authorized from source shape alone. The same
cold-start project must first record per-pass GPU timestamps, CPU graph/command
build and submission cost, publication latency, RSS/VRAM, and matched WPR/WPA
energy. Only then may an implementation atomically change scheduler batches,
topology keys/cache capacity, attempt and stale-completion tokens, retry
semantics, and timing schema. Current status is
`algorithm_defect_confirmed_structural_bootstrap_cut_pending_current_source_gpu_and_power_profile`.

## 2026-08-31 P1-2 dynamic-binding ownership recheck

The current recorder no longer creates PMREM and SH9 bindings on every stable
generation. `RealtimeIblWgpuBindingCache` owns entries by work slot and exact
command key. A default ticket has ten PMREM keys and one SH9 key; replay on an
already populated work slot reports eleven hits and zero parameter-buffer or
bind-group creations. This removes those resources from the steady stable-slot
problem statement.

Capture and source-mip recording remain dynamic. The default 21-batch ticket
contains three capture passes and seven source-mip passes, and each creates one
small uniform buffer plus one bind group. The residual structural count is
therefore ten buffers and ten bind groups per generation. A cold work slot also
creates the eleven cacheable PMREM/SH9 pairs. General artifact bake retains its
per-command parameter buffer and bind group as well.

The source already contains an ignored WGPU profile that reports adapter
identity, cold/warm elapsed time, and separate creation counts/microseconds for
capture, source-mip, and PMREM/SH9 paths. It has not been executed against the
current source in this slice. The involved recorder, capture, binding, timing,
and tests are foreign modified, so no allocator or cache change was made.
Current-source cold/warm p50/p95/p99 plus matched WPR/WPA must establish that
the residual ten pairs dominate before selecting a persistent parameter arena,
dynamic offsets, or immutable per-slot templates. Status is
`realtime_pmrem_sh9_source_closed_dynamic_capture_source_mip_profile_pending`.

## 2026-08-31 P1-8 base-lobe recipe disposition

The current source already has a versioned `IblBakeRecipe`; introducing a
second `EnvironmentPbrRecipe` for the same production parameters would create
two authorities. The existing owner fixes PMREM sample tiers, roughness/mip
mapping in both directions, filtered-importance-sampling texel scale, the
full-roughness cosine threshold, diffuse source-mip selection, RGBA16F output,
and distinct asset-CPU versus runtime-GPU diffuse integrator identities.

The narrower unresolved contract is the BRDF LUT. Its 128x32 extent, 128 sample
count, joint-Smith integrator, and single-scatter interpretation are separate
constants and are not represented in the recipe identity. No production WGSL
or Rust consumer implements base-lobe multiple-scattering energy compensation.
That is an advanced quality decision, not an MVP correctness patch.

The MVP baseline therefore remains the current Unreal-equivalent joint-Smith
single-scatter split sum. Managed image/error and GPU/performance evidence must
accept that baseline before extending the identity with a BRDF-LUT mode or a
multiple-scattering mode. Clearcoat, sheen, anisotropy, and cross-lobe energy
remain owned by 09G. Status is
`base_single_scatter_mvp_recipe_present_brdf_lut_identity_and_multiscatter_contract_pending`.
