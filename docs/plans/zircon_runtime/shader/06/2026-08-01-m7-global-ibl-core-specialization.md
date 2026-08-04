# Shader06 M7 Global IBL Core Specialization

Plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
Milestone: M7
Status: in_progress
Forward-repair files: ["zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_write_scene_uniform/write_scene_uniform.rs", "zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/resources.rs", "zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/tests/resources.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs"]
Async-repair files: ["zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline/tests.rs"]
Depends on: M6
Evidence-support files: ["tools/zircon_validate_shader_pbr_viewer_evidence.py", "tools/tests/test_zircon_validate_shader_pbr_viewer_evidence.py"]
Files: ["docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md", "docs/plans/zircon_runtime/shader/06/2026-08-01-m7-global-ibl-core-specialization.md", "zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_brdf_lut.rs", "zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs", "zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_environment_only_pbr.wgsl", "zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl", "zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl", "zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler/tests.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs", "zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/construct/create_sky_pipeline.rs", "zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs", "zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs", "zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs", "zircon_runtime/src/graphics/shader/includes/zr_pbr_extras.wgsl", "zircon_runtime/src/graphics/shader/includes/zr_pbr_extras_core.wgsl", "zircon_runtime/src/graphics/shader/template/assemble.rs", "zircon_runtime/src/graphics/shader/template/material_surface.rs", "zircon_runtime/src/graphics/shader/template/module_registry.rs", "zircon_runtime/src/graphics/shader/template/pass_specialization.rs", "zircon_runtime/src/graphics/shader/template/tests.rs", "zircon_runtime/src/graphics/shader/template/tests/environment.rs", "zircon_runtime/src/graphics/shader/template/tests/standard_material_surface_template.rs", "zircon_runtime/src/graphics/shader/template/tests/standard_pbr_specialization.rs", "zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_environment_core.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_environment_generic_api.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_environment_only_pbr.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_scene_runtime.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr_basic.wgsl", "zircon_runtime/src/graphics/tests/pipeline_compile/dynamic_resolution.rs", "zircon_runtime/tests/runtime_environment_wgpu_cubemap_sampling_contract.rs", "zircon_runtime/tests/runtime_volumetric_shading_contract.rs"]

## Reason

M6's immutable validation input has already been materialized. Its next managed attempt must prove that exact manifest after the Runtime04 lockfile recovery, so a further source-layout optimization cannot be added to M6 without recreating the attribution-staleness failure that M6 was created to avoid.

The environment-only viewer still assembles the full local-reflection provider implementation: probe selection, box projection, probe cubemap sampling, planar-reflection sampling, their group-1 bindings, and the associated dynamic branches. Those paths are unreachable for the exact opaque Standard-PBR Base environment-only profile. The remaining source and DX12 front-end cost is meaningful enough to warrant a forward M7 slice before new current-source timing is requested.

## Design

- `zr_environment_core.wgsl` owns normalized global source/realtime cubemap sampling, PMREM LOD, SH9/IEM diffuse IBL, procedural sky, BRDF LUT, sky reflection, and the shared split-sum PBR composition helper.
- `zr_environment_generic_api.wgsl` owns only the compatibility wrappers required by generic consumers: defensive normalization, source-cubemap lookup, public PMREM LOD, legacy BRDF approximation, SH9/IEM entry points, and the generic sky selector. It is deliberately absent from the environment-only composite.
- `zr_environment.wgsl` is the generic local-provider layer. Together with the core and generic API it preserves all current public `zr_environment_*` functions, group-1 probe/planar ABI, and deferred/fallback behavior.
- `zr_environment_only_pbr.wgsl` supplies the same public PBR entry points using only the global environment provider. It deliberately declares no group-1 probe or planar bindings, local-provider branches, or generic compatibility wrappers.
- Both composites retain the canonical `zr_environment.wgsl` include token. They intentionally differ in source body and therefore content hash, preserving cache separation without creating a second public include name.
- Direct static WGSL consumers concatenate the core, generic API, and generic provider source at compile time. No runtime string construction, descriptor allocation, bind-group layout change, or Rust public API is added.

## Acceptance

- Generic Forward, deferred lighting, fallback mesh, source-cubemap contract, and pipeline-compile assemblies retain the complete local provider implementation and parse as WGSL.
- Environment-only Forward contains global PMREM/SH9/IEM/split-sum PBR but excludes probe and planar source, `@group(1)` bindings 16, 17, 18, 29, and 30, all local-provider functions, and generic-only compatibility wrappers.
- The environment-only composite keeps `zr_environment.wgsl` in its include manifest but has a distinct environment include content hash from the generic composite.
- Shared global PBR composition appears in the core exactly once; provider wrappers only establish availability and reflection radiance.
- The specialized assembled Forward source is at least 50% smaller than the comparable generic Forward lower bound, measured as source volume only.
- Fresh managed Rust/WGSL, WGPU prewarm/cache, quantitative image, DX12 timing/screenshot, and RenderDoc replay remain required before this milestone can be accepted. No historical measurement or screenshot is reused as M7 evidence.

## Current Status

Implementation is complete. The generic composite concatenates the 8,488-byte global core, 2,676-byte generic API layer, and 13,479-byte local provider; the environment-only composite uses only the core plus the 2,088-byte global-only provider. Its unused reflection wrappers and all generic-only compatibility wrappers are absent after confirming the exact specialized shading body invokes only `zr_environment_pbr_indirect`; generic `zr_pbr_extras` is not part of this profile. That reduces the environment module from 24,645 bytes to 10,577 bytes (57.1%). Applied to M6's earlier 39,435-byte specialized assembly inventory, the conservative projected specialized assembly is 25,367 bytes, or 65.0% below the 72,564-byte comparable generic lower bound. This is source-volume projection only, not DX12 timing.

The environment-only assembly contract now requires global PMREM/SH9/IEM/BRDF availability, removes every local probe/planar group-1 binding and function plus generic-only compatibility wrappers, keeps the canonical `zr_environment.wgsl` token, and requires its content hash to differ from the generic composite. All direct static consumers now concatenate core plus generic API plus generic provider, so deferred, fallback, source-cubemap, WGPU sampling, volumetric, and pipeline-compile contracts retain the complete generic ABI. Source-level RED was observed before the switch; the post-change partition contract, scoped `rustfmt --check`, and scoped `git diff --check` pass.

The initial independent review reported `Critical 0 / Important 0 / Minor 0 / Ready` for the core/provider split. After wrapper pruning, the required follow-up independent review again reported `Critical 0 / Important 0 / Minor 0 / Ready`: the specialized shading body calls only `zr_environment_pbr_indirect`, excludes `zr_pbr_extras`, and has no dangling reflection-wrapper dependency. Neither review substitutes for managed Cargo/WGSL, WGPU, DX12, screenshot, or RenderDoc evidence.

After the generic API extraction, the required second independent review reported `Critical 0 / Important 0 / Minor 0 / Ready`: generic assembly preserves the `core + API + local provider` ABI, specialized assembly closes with only the core and PBR entry points, direct static consumers retain explicit newline boundaries, and feature-aware environment content hashes remain distinct. The review did not run Cargo and does not substitute for managed Cargo/WGSL, WGPU, DX12, screenshot, or RenderDoc evidence.

The runtime WGPU cubemap contract was also aligned with the new owner split: the full-metal diffuse guard now inspects `zr_environment_pbr_components_from_reflection`, while its separate public-wrapper test still verifies normalized reflection-provider input. This prevents the eventual managed test from treating source ownership movement as a PBR regression; scoped formatting and the shared-owner static guard pass.

The post-split reference check preserves the intended cmft/cmftStudio semantics: Zircon's `reflect(-view_dir, normal)` is equivalent to cmftStudio's `-reflect(view, normal)`, the nonlinear `1.2` roughness-to-LOD function remains paired with the CPU PMREM bake inverse, and WGPU deliberately retains native cubemap edge filtering instead of cmft's legacy OpenGL direction warp. The split changes source ownership only; it does not change environment reflection direction, PMREM sampling, or the established ambient-occlusion contract.

Cargo/WGSL execution is intentionally not relaunched against the already-known clean-main lockfile failure. M6 remains unchanged and in progress pending the external Runtime04 lockfile forward fix; once that forward baseline is coordinator-integrated, M7 must run fresh managed Rust/WGSL and WGPU prewarm/cache validation before DX12 timing, screenshot, and RenderDoc replay. M7 does not alter that Failure ownership or use an unlocked Cargo path.

The post-split provider-boundary audit found that local reflection resources can
upgrade after the environment-only Base prewarm, while the specialized WGSL
correctly omits their group-1 ABI. Leaving the feature bit enabled would then
silently omit valid baked-probe or planar reflection radiance. The resource
owner now exposes that upgrade and the mesh variant registry permanently falls
back to the generic Base key before draw construction. The original lightweight
path remains active until a provider is requested; the generic key is retained
afterward to avoid visibility-driven variant thrashing. Regression contracts
cover baked-probe and planar upgrades plus feature-bit separation. Scoped Rust
parsing and source-order/diff checks pass; M7 remains `in_progress` pending the
same managed Cargo/WGSL, WGPU, DX12 screenshot/timing, and RenderDoc evidence.

Independent review then found that an async Base compiler could return a
SkipDraw placeholder for the newly selected generic key, while BaseScenePass
requires a concrete pipeline. The forward repair records the upgrade only for
an environment-preview placeholder (ordinary FullScene capacity does not
trigger it) and forces subsequent Base variants to complete synchronously after
that event. This leaves the normal async policy intact, drains an already
pending target if necessary, and prevents a provider transition from reaching
the BaseScenePass `None`/panic path. Regression coverage now proves both the
provider-only trigger and a generic fallback resolve with async compilation
enabled. Static parsing and diff integrity pass. The source-order contract locks
the core bridge as a Rust regression: `prepare -> provider fallback -> draw
construction`. The post-repair independent review reports `Critical 0 /
Important 0 / Minor 0`: it confirms that only an actual environment-preview
provider upgrade sets the one-way synchronous Base guard, ordinary FullScene
retains asynchronous compilation, and a new or pending generic Base resolves
to a concrete pipeline before `BaseScenePass`. M7 remains `in_progress` until
the managed product evidence is accepted.

The screenshot handoff now has a standalone evidence gate:
`python tools/zircon_validate_shader_pbr_viewer_evidence.py <ready.png> --expected-backend Dx12 --require-direct-present`.
It validates the bounded actual RGBA PNG structure, checksum, dimensions,
visible-pixel non-blankness, and matching v4 ready-frame sidecar, including the
environment-only PBR profile, complete active-cubemap face-size/mip layout,
phase-duration hierarchy, and the explicit process-local `MeshPipelineCache`
scope for the Base-prewarm cache hit plus capture-time Base-pipeline readiness.
The library retains v2/v3 read compatibility for historical inspection, while
the CLI acceptance command requires v4 and the current decimal IBL bake version
`202608020005` by default; `--allow-legacy-schema` is an explicit
historical-baseline inspection opt-in for legacy schemas or bake versions. A
static contract keeps that decimal value aligned with Rust's canonical
`IBL_BAKE_ALGORITHM_VERSION`. It is standard-library-only and never starts the
engine. Its fixture suite accepts a valid evidence unit and
rejects viewport/cache-scope drift, incomplete cubemap mip layouts,
blank/fully transparent images, an oversized encoded input, or a phase total
shorter than its recorded component intervals. Historical screenshots without the current v4
sidecar remain baselines only;
this gate is preparation for, not a substitute for, the required fresh managed
DX12 screenshot and RenderDoc evidence.

The latest forward PBR hardening rejects a zero normal or view direction before
the global-only or generic provider can perform cubemap, PMREM, SH9/IEM, or BRDF
texture work. The post-process SSR resolve alpha remains the already intensity-
modulated visibility used by temporal history, and scene composite now consumes
that visibility directly instead of applying the same intensity a second time.
Both paths have source-level regression contracts and do not change the M7
acceptance boundary: fresh coordinator-managed Rust/WGSL, WGPU, DX12 screenshot/
timing, and RenderDoc replay evidence are still required while this record stays
`in_progress`.

The final material-source specialization also removes the normal-UV transform
and its argument-bearing base-normal call from the exact no-normal-map/
no-clearcoat baseline. Normal-mapped and clearcoat variants retain that UV only
where sampling requires it, while anisotropy-only retains its tangent frame
without normal-map or clearcoat declarations. The fixed group-2 layout and
draw-time bindings remain slots 0..12; this is generated WGSL/front-end work
reduction rather than an ABI change. Normal-only, clearcoat, and
anisotropy-only assembled outputs now have focused source contracts and Naga
validation ownership. The independent follow-up review reports `Critical 0 /
Important 0 / Minor 0`; scoped static RED/GREEN guards, `rustfmt --check`, and
diff integrity pass. No managed Rust/WGSL, WGPU, DX12 screenshot/timing, or
RenderDoc validation has run for this current source, so M7 remains
`in_progress`.

The post-repair independent reviews report `Critical 0 / Important 0 / Minor 0`.
They confirmed that both PBR providers return before reflection or BRDF sampling
for zero normal/view input, that SSR applies its configured intensity once, and
that replay errors preserve their primary timeout or command-failure cause even
when temporary-snapshot cleanup also fails. The standalone evidence-tool suite
currently passes 13 tests, including CLI error reporting, bounded process output,
snapshot integrity, cleanup, and current v4 viewer-sidecar contracts. This is static and
tooling evidence only; it does not promote the milestone beyond `in_progress`.

The deferred PBR GBuffer boundary now also decodes its normal through the
existing `normalize_or_zero` helper in generic, SSS, and environment-only
fragment entry points. A source-assembly regression contract requires all three
decodes to use the helper and rejects the former direct `normalize` form, so a
cleared or otherwise degenerate normal cannot create non-finite PBR input before
the provider-level zero-direction guard. Scoped `rustfmt --check`, diff
integrity, helper-body inspection, and the three-entry source contract pass.
This is a binding-free safety repair: it adds no texture read, descriptor, or
runtime feature. It remains static evidence only and does not change the
managed-product acceptance boundary.

The Shader06 review also preserves a strict ownership boundary for the planned
SSR -> probe -> sky alpha-under composition. The current SSR history is a
separate HDR texture, but its scene-composite consumer receives only already
resolved `scene-color`; standard deferred and forward PBR have already merged
direct light, diffuse IBL, and environment specular there. Consequently the
current `mix(scene_color, SSR)` is not evidence of alpha-under composition, and
changing it locally to an additive blend would double-count fallback reflection.
The complete contract requires a Render07-owned typed indirect-specular carrier
from all applicable lighting producers through graph resource declaration to
the composite consumer, with the final material response applied once. M7 only
retains the existing SSR-intensity-once correction and does not claim that
cross-pass architecture as complete.

An independent review first identified one Minor regression-coverage gap: the
generic SSS fragment entry was repaired in WGSL but was not named explicitly by
the source contract. The forward test now requires both `fs_main_sss` and its
one-component zero-safe decode. The follow-up review reports `Critical 0 /
Important 0 / Minor 0`; it confirms all generic, SSS, and environment-only
GBuffer normal entry points use their already-visible helper, preserve nonzero
results, and introduce no resource, texture, MRT, or ABI change. No Cargo was
run for this static repair.

The current clearcoat hardening uses the same zero-direction contract across
environment and direct-light paths. Its advanced environment helper rejects a
zero clearcoat normal or view before planar/cubemap/BRDF work; its base-energy
helper then preserves the base layer for that invalid input. Direct lighting
uses the clearcoat normal's own `NoL`, skips GGX for a zero/back-facing coat,
and clamps the material weight; base diffuse ambient now receives the same
single clearcoat energy scale as direct light and IBL. The Forward hot path
prepares its clearcoat normal and view once and passes those values to
normalized direct-light, base-energy, and environment helpers; defensive
wrappers remain for non-hot callers. Standard material assembly also keeps the
base normal and skips the clearcoat-normal texture sample whenever the
compile-time feature is absent, without changing the material binding ABI.
It likewise writes stable tangent defaults and constructs an orthonormal
tangent frame only for the compile-time anisotropy variant, whose GGX path is
its only consumer. Source-assembly contracts cover these ordering, energy,
no-extra-sample, and no-extra-frame rules. The material-source generator also
omits clearcoat-normal WGSL bindings and the sampling helper completely when
that feature is absent, while the renderer intentionally retains and binds its
fixed group-2 layout through slots 0..12; this is source/frontend work removal,
not a material-ABI change. The same feature-aware source path omits normal-map
bindings, tangent helpers, and per-pixel frame construction from the exact
no-normal/no-clearcoat/no-anisotropy Base profile; each required helper remains
in normal-mapped, clearcoat, or anisotropic variants. Static RED/GREEN evidence,
scoped `rustfmt --check`, and diff integrity pass; managed Cargo/WGSL, WGPU,
DX12 screenshot/timing, and RenderDoc evidence remain required, so M7 stays
`in_progress`.

The generated `standard_material_surface` Forward Standard-PBR path now has a
second source-specialization layer for the exact
no-clearcoat/no-anisotropy/no-transmission feature set. It retains the canonical `zr_pbr_extras.wgsl` and
`zr_shading_standard_pbr.wgsl` tokens, but selects a 1,538-byte shared
isotropic-GGX core and a 9,177-byte basic shading body instead of the
advanced 21,647-byte combined PBR closure. This is a 50.5% reduction in the
two source-module inputs only, measured statically; it is not a DX12 loading
time claim. The advanced feature set restores the complete source, including
transmission bindings 31/32, and therefore produces distinct content hashes
under the same canonical tokens. Custom surface sources and custom shading-model
descriptors continue to receive the complete generic Forward PBR closure,
including when `ENVIRONMENT_ONLY_PBR` is set, so the specialization cannot
truncate their declared helper closure. The assembly tests require both source
closures, canonical-token/hash separation, the retained base IBL/light/shadow
closure, absence of advanced code in the basic variant, every individual
advanced feature restore, custom-surface generic closure preservation, and WGSL
validation.
Scoped static RED/GREEN, `rustfmt --check`, and `git diff --check` pass; the
record remains `in_progress` pending the already-required managed Rust/WGSL,
WGPU prewarm/cache, DX12 timing/screenshot, and RenderDoc evidence.

The required post-repair independent review reports `Critical 0 / Important 0 / Minor 0`:
the eligibility predicate requires both the generated Standard Material surface entry and descriptor absence, and that single predicate gates
both pass support and shading-body selection. Descriptor-free custom surfaces,
including their environment-only requests, retain the generic Forward closure.
Each advanced feature bit independently restores the complete canonical-token
source and distinct content hashes. The pre-existing no-shadow stub branch was
reviewed as out of scope and remains unchanged. No Cargo ran; this static source
completion does not change M7 from `in_progress`.

The environment-rotation hot path now moves trigonometry from fragment shader
execution to `SceneUniform::from_frame`: the appended
`environment_rotation_sin_cos` tail vec4 carries finite rotation sine, cosine,
and a nonzero flag. `zr_environment_rotated_direction` retains its zero-rotation
early return, then consumes those two CPU-precomputed values instead of calling
`sin` and `cos` for every global PMREM, SH9/IEM, or source-cubemap lookup. The
existing rotation radians, environment availability, source layout, bindings,
and reflection-direction semantics remain unchanged; the only ABI change is the
matched Rust/WGSL 16-byte scene-uniform tail extension. CPU and assembled-WGSL
RED/GREEN contracts, scoped `rustfmt --check`, and scoped `git diff --check`
pass. This is a runtime ALU optimization, not a DX12 timing claim, and M7
remains `in_progress` pending the same managed product evidence.

The first independent review found that the initial field placement between
`environment_params` and `environment_sample_params` would shift renderer-local
WGSL prefix mirrors and that the independent skybox shader still evaluated
fragment trigonometry. The forward repair moves the new vec4 after every
existing `SceneUniform` field, so local shaders that do not consume rotation
retain their byte offsets; the skybox mirror appends the same tail and consumes
the CPU values with the identical zero-rotation fast path. The two direct Rust
`SceneUniform` literals publish the identity tail. Mirror/tail source scans,
main and skybox hot-path contracts, scoped formatting, and diff integrity pass.
The required post-repair independent review reports `Critical 0 / Important 0 /
Minor 0`. These static results do not replace managed Rust/WGSL, WGPU, DX12
timing/screenshot, or RenderDoc replay evidence, so M7 remains `in_progress`.

The ordinary builtin Standard-PBR Forward path now has a third environment
composite between the generic and global-only forms: it retains the global core
and complete local probe/planar provider, but omits the generic public API
wrappers that its generated shading body does not call. The 22,151-byte source
input is 2,677 bytes (10.8%) smaller than the 24,828-byte generic environment
input. This is source-volume evidence only, not a DX12 timing claim. The
existing builtin eligibility keeps the optimization limited to the Standard
material entry without a custom shading-model descriptor; custom source,
FullScene deferred, fallback, and environment-only material paths retain their
respective complete closures. Every composite keeps the canonical
`zr_environment.wgsl` token with its own content hash. RED/GREEN source
contracts require local reflection ABI and PBR indirect shading to remain while
all generic wrappers are absent only from the builtin Standard-PBR assembly. M7
remains `in_progress` pending the same managed product evidence.

The required independent post-implementation review reports `Critical 0 /
Important 0 / Minor 0`. It confirmed that builtin Standard-PBR Forward used
the core-plus-local-provider composite while custom, FullScene deferred,
fallback, and environment-only assemblies retained their required complete
closures at that point. The review also confirmed preservation of the
probe/planar ABI, the canonical
`zr_environment.wgsl` token with distinct content hashes per composite, and the
advanced-PBR closure. No Cargo command ran under the managed-validation policy;
M7 therefore remains `in_progress` until the coordinator supplies managed
Rust/WGPU/DX12 screenshot and RenderDoc evidence.

The later forward repair closes two deferred-path omissions found by the
required second static review. Both direct deferred WGSL `SceneUniform` mirrors
and the fallback mesh mirror now append `environment_rotation_sin_cos` after
the existing sample parameters, matching the CPU uniform tail consumed by the
shared environment core. `StandardPbrPreview` now uses the same canonical-token
core-plus-local-provider composite as the builtin Standard-PBR Forward path
(22,151 bytes versus the 24,828-byte generic environment source, a 10.8%
source-volume reduction); `FullScene` deferred remains generic.

A subsequent provider-upgrade audit found that `EnvironmentOnlyPbrPreview`
cannot use the global-only deferred closure: a real probe or planar resource can
arrive after startup, and the existing upgrade path switches mesh variants to
generic without rebuilding the deferred profile. The forward repair therefore
keeps the direct-light-free environment-only template but restores its generic
environment source and all five local-provider bind-group entries. The profile
continues to defer the heavyweight provider textures with its existing 1x1
placeholder, so this preserves functional local reflections without reintroducing
the full provider allocation at startup. Source contracts cover the three
deferred profile closures, the provider-upgrade ABI, and all five current WGSL
`SceneUniform` mirrors. The post-repair second static review reports `Critical 0
/ Important 0 / Minor 0`; scoped `rustfmt --check` and `git diff --check` now
pass for every touched Rust path, including the fallback test source after its
earlier mapped-file writeback issue cleared. Managed Rust/WGSL, WGPU, DX12
screenshot/timing, and RenderDoc remain required. M7 remains `in_progress`.

The device-lifetime BRDF LUT upload now reuses one process-local immutable RG16
byte payload. `SceneEnvironmentBrdfLut` still creates and owns a separate
texture/view for every WGPU device, then uploads the same bytes to that device;
only the deterministic `128x128x1024` CPU Hammersley integration and f16
encoding are cached after the first renderer-core construction. A focused local
`OnceLock` regression requires the producer closure to run once while both
callers observe the initial bytes. This removes repeated CPU setup without
claiming a DX12 PSO or GPU timing improvement. M7 remains `in_progress` pending
the existing managed Rust/WGSL, WGPU, DX12 screenshot/timing, and RenderDoc
evidence.

The current view-direction hot-path repair now short-circuits the two exact
camera endpoints in every PBR consumer: environment-only Forward, advanced and
basic Standard-PBR Forward, environment-only and generic deferred, and fallback
mesh. After clamping the camera-direction blend, perspective (`0`) returns its
already zero-safe normalized camera-to-fragment vector, while orthographic
(`1`) returns the zero-safe camera direction without constructing the unused
perspective vector. Only an intermediate blend retains the existing normalized
`mix` calculation, preserving that path's result and its invalid-direction
handling. The source regression asserts branch ordering across all six
consumers; scoped formatting and diff integrity pass. Independent review reports
`Critical 0 / Important 0 / Minor 0`. This is fragment ALU convergence only,
not a DX12 timing or acceptance claim, and M7 remains `in_progress` pending the
existing managed Rust/WGSL, WGPU, DX12 screenshot/timing, and RenderDoc
evidence.
