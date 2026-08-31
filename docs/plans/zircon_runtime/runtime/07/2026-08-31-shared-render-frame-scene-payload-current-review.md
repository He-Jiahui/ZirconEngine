# Runtime07 shared render-frame scene payload current-source review

Date: 2026-08-31

Status: `architecture_review_complete / historical_baseline_recorded / M1_shared_storage_source_complete / M2_renderer_submission_immutable_source_complete / core_mutable_compatibility_gate_open / managed_acceptance_pending / M2_M3_remaining`

Owner session: `root-runtime07-shared-extract-r1-20260831`

Parent finding: `engine-code-review-findings-2026-06.md` F3 and `PERF-MVP-431`

## Decision

The current dynamic extract cache cannot be repaired by storing
`Arc<RenderFrameExtract>` and calling `Arc::make_mut` in the renderer. The cache keeps one strong
owner, so the first renderer mutation would clone the complete frame. The accepted direction is a
hard submission-contract split:

1. An immutable, generation-owned `Arc<RenderFrameScenePayload>` (or exact equivalent shared
   domains) owns world, geometry, animation, authored lighting/environment, sprites, particles,
   visibility, and other cacheable scene data.
2. One small owned submission overlay owns timing and selected/editor view state. It may own compact
   per-camera policy inputs, but it must not duplicate scene vectors.
3. Renderer-derived material census, hydrated environment choice, particle history, effective post
   process, budget degradation, temporal jitter, and view-family state belong to the renderer's
   submission context. They must not be written back into the immutable scene payload.
4. Capture and present consume the same hard-cut submission DTO and preserve one world generation.
   The old owned-extract overload must not survive as a compatibility lane.

The historical direct-current-source baseline below established the linear clone cost before
production mutation. The dependency-ordered M1 shared-storage slice is now implemented in source.
The public `DerefMut`/domain-COW compatibility surface still leaves the immutable submission type
gate open for M2; managed current-source acceptance remains mandatory before even the M1 slice can
be attributed, committed, or reported as green.

## Current-source evidence

Frozen source hashes at review time:

| Path | SHA-256 |
| --- | --- |
| `dynamic_api/session/extract_cache.rs` | `d2acbfc73885a5d8d507100d0569ee146a75328e6f88d305940ea6c384eed81e` |
| `dynamic_api/session/extract.rs` | `89cf1b3a03fb53464dc367e6fb5651ee62dc83f5ac9736385de120178ed3d226` |
| `core/framework/render/frame_extract/frame.rs` | `748bcc03a0e93938fec529c4b781478cafe83bfce0bcd5e03b2015b71b0700b7` |
| `core/framework/render/framework.rs` | `a5b651b7dfd62ba0bdf324a7556ab5bbef0d8b554a6143e78861f7c8a4df78b0` |
| `dynamic_api/runtime_loop.rs` | `3ae21d0f4a214eb6324ea6de36fb464c4353d7481f1f8ce8b6c4d161f5c8911d` |
| `graphics/types/viewport_render_frame.rs` | `077b0fd34865bacca69fec609f1f6db5f1b0c0e5d996b73135d0a11b05f3a7fa` |
| `render_framework_state/viewport_pick_frame_registry.rs` | `fa44100497e51af8af7da7fece49fb5c47c047d842ee27d6869366fd7ef19a9c` |

M1 source hashes after the hard cut:

| Path | SHA-256 |
| --- | --- |
| `core/framework/render/frame_extract.rs` | `4647d0a679827dce14d931327679eb1f585c3715e4daf6056edcadfc91c7dd65` |
| `core/framework/render/frame_extract/frame.rs` | `c03f73d67e45006acf4c643ae9460efaa79a7da8aedd31ffa8810d3ec1a311bf` |
| `core/framework/render/frame_extract/scene_payload.rs` | `bfc4c0f60f07cfff7cd91e33dcd56d19f9027e7ebe913551c33c5600fcc307c9` |
| `core/framework/render/frame_extract/shared_scene_domain.rs` | `b65dc1860024df49bac57d87797dfa18923c9ec20c8d2b69ecb16ff78c39946e` |
| `core/framework/render/frame_extract/tests.rs` | `3011b468a2348536889c1fc24ceb9c9f9ae5d02ba55bdb7ba75ef09bc8b8beda` |
| `dynamic_api/session/extract_cache.rs` | `45a20b2291b34088be691c650e3297d546d0a5d2185a2ba3623358ac12954acb` |
| `dynamic_api/session/extract_stats.rs` | `b7f1db899898c53f4dfd42ae56379a0c11ecad6ec796ec719de336ece6b66db8` |
| `dynamic_api/session/tests/frame_diagnostics.rs` | `8fd1bd3b7cf554cdec60516fde17a52e836e82fdd0b978cdd851c5e33c4ccc77` |
| `scene/world/render.rs` | `6eb9c0d274e8f37e38e4ddb4c523b6a14c8d0840eb8e7759c262823e6b30a892` |
| `scene/level_system_render_extract.rs` | `fb9d156e75958d9ca9af55aa0953d0d481ebae7439cc953cbb10cd7e35db1968` |
| `tests/runtime_frame_extract_shared_payload_performance.rs` | `e5688b28189192981acec480def8fcf4aca0732a9d32fd00ead8b724c0c316da` |

The cache key is structurally correct and must be preserved:
`ChangeTick + lifecycle_visibility_revision + active_camera + viewport_size`.

The reviewed cache value and return path were not correct:

- A hit executes `entry.extract.clone()`.
- A miss builds one extract and executes `extract.clone()` to retain the cache copy.
- `current_extract` then mutates the returned copy for editor camera and timing.
- `RenderFrameExtract::clone` is linear in all owned `Vec` and nested owned payloads. Its scale is
`O(meshes + poses + lights + sprites + particles + visibility + post/debug/environment payload)`.

M1 changes that ownership without adding a compatibility DTO:

- `RenderFrameExtract` now owns only `Arc<RenderFrameScenePayload>`, timing, and view overlay.
- Each large scene domain has an independent `RenderSharedSceneDomain<T>` Arc handle, so deriving
  one renderer domain cannot clone unchanged geometry/light/sprite/particle/visibility domains.
- Cache population and stable reuse clone only the compact overlay and Arc handles; diagnostics
  continue to retain one sealed summary per cache generation and now report zero full clones.
- World and level producers use the canonical payload constructor. Every current
  `RenderFrameExtract { ... }` consumer was hard-cut; a source scan reports zero old literals.
- Cache admission uses the preserved four-part key. A focused unit regression changes each
  component independently and requires `Rebuilt`; an identical key alone returns `Reused`.
- A real-cache regression covers first miss, stable hit, shared scene identity, and returned
  view/timing mutation without cache-entry contamination.

M1 deliberately does not close the final immutability boundary. `RenderFrameExtract::DerefMut` can
still clone the payload shell, and a mutable `RenderSharedSceneDomain<T>` access can still perform a
linear domain COW. These are renderer migration surfaces, not the accepted final submission API.
M2 must remove public scene mutation from the submission DTO after all renderer-derived writes move
into an explicitly owned submission context. Derived `PartialEq` can also deep-compare distinct Arc
owners and is excluded from performance-sensitive admission until that semantic surface is removed
or replaced.

The renderer already converts the owned extract to `Arc<RenderFrameExtract>`, but then mutates it
through `Arc::make_mut`. Current mutation sites include:

- budget render scale and viewport sizing;
- subsurface-profile and advanced-material census;
- environment IBL hydration;
- renderer-owned particle previous-state injection;
- effective post-process settings and graph;
- view-family pipeline, anti-alias state, and temporal jitter;
- selected-camera switching and camera-loop source restoration.

`FrameSubmissionContext` and the viewport pick registry retain the resulting full extract. This
means a cache-owned `Arc` would force a full copy at the first mutation and would only move the
cost from the dynamic cache into the renderer.

## M2 renderer-derived advanced-lighting compile-input slice

The first M2 slice removes the two unconditional authored-lighting writes that ran on every frame:

- material census now returns `AdvancedPbrMaterialFrameUsage` instead of assigning
  `extract.lighting.advanced_lighting.material_features`;
- subsurface resolution now returns the sorted profile vector and fixed-mask-derived active indices
  instead of assigning both vectors into the shared lighting domain;
- `AdvancedLightingCompileInputs` owns those renderer-derived values. Its variable-length arrays are
  `Arc<[T]>`, so compile-option and cache-key clones remain constant-size;
- `RenderPipelineCompileOptions` carries the one resolved value through both provisional and final
  pipeline compilation. The cache key normalizes authored fallback and renderer override to the
  same complete representation before hashing;
- graph fingerprinting, exact SSS profile-table compilation, late-forward admission, and
  transmission pass insertion all consume that same normalized input. There is no compatibility
  path that still reads renderer census output from the shared extract;
- exact SSS profile float bits participate in equality and hashing. Two active profile sets with the
  same IDs but different scattering data cannot alias one compiled-graph entry.

This removes the unconditional `LightingExtract` COW caused by material/SSS resolution while
preserving the canonical material parent resolver and the Runtime99a 32-bit profile-use mask. The
remaining `Arc::make_mut` sites are separately owned view, environment, particle, post-process, and
camera-loop migrations; therefore this slice does not claim that M2 or the immutable-submission gate
is complete.

M2 slice source hashes:

| Path | SHA-256 |
| --- | --- |
| `graphics/pipeline/declarations/advanced_lighting_compile_inputs.rs` | `2410a483369092c12c75678505c42f53474fc245cd6a2ded59b119dc413ac93c` |
| `graphics/pipeline/declarations/mod.rs` | `57ea5f1334d441832fce4d575411fe69c5cf69037e25ec21cbb3b5263a9c50a3` |
| `graphics/pipeline/mod.rs` | `5461d1cf186288e696d59383e2548f3c9b5f54f36b77fc480243ab43cb80241e` |
| `graphics/pipeline/declarations/render_pipeline_compile_options.rs` | `c39b306679ebbf628b197ac2d6babb1edffdb50ebce6558dbf911b12bc14408a` |
| `graphics/pipeline/compile_options/default.rs` | `d1d9d98c2ec4412b71b8916cfdc81ce306abf2c6d371d94ce69b8bdcad316503` |
| `graphics/pipeline/compile_options/methods.rs` | `80e551462f94bd7c92a8560f744cdecb3f82f2aaa488e6bd405f1c41136cf056` |
| `build_frame_submission_context/build.rs` | `3b7e32b235e8d0e1d5d506a137e8d8489b2b618191b65eb0d02cfddb15d21f05` |
| `build_frame_submission_context/material_feature_extract.rs` | `033ce5d066c7868a485173aa35e1f6e1c8f3408976c82eb007eaf3d1e2c9c14d` |
| `build_frame_submission_context/subsurface_profile_extract.rs` | `babb3b2ae4e2834afa59afd406ed35352201b5ac75db62be942ab948d4d8749f` |
| `graphics/pipeline/compiled_graph_cache.rs` | `026cb1c82109e494e63aee9711406d54b3e04df670de28b5ec11e4c38960933f` |
| `graphics/pipeline/compiled_graph_cache/tests.rs` | `f136de9c477d28f482cca1c17547b93d30c87806d257e0fbef5e5f6c2ab34835` |
| `graphics/pipeline/render_pipeline_asset/compile.rs` | `5a878c27588fdcba6495098a74de95b6f6666dbe382e4073d7f3cee8536147e1` |
| `tests/runtime_renderer_derived_lighting_compile_input_performance.rs` | `85e077305ddf74aebf7132b86cde10df8a9875daa05ec7356d849fdd73832e60` |

## M2 renderer-derived environment and particle-history slice

The second M2 slice removes two more renderer writes from the shared scene payload and makes the
measured optimized operations match the production ownership path:

- environment IBL hydration returns a renderer-owned `SourceCubemapEnvironment` override instead
  of assigning the hydrated cache result into the authored environment domain;
- environment graph admission, runtime-bake reservation, and hydrated-source selection are
  returned together as one resolution object, so a failed build still releases its reservation;
- the environment value is cloned only as a compact overlay. PMREM/source texels remain shared by
  their internal `Arc` storage, and no extra `Arc::new` allocation is introduced at the submission
  boundary;
- scene-uniform, SH9, cubemap-upload, particle-velocity, and UI-content consumers read the exact
  override-or-authored accessors on `ViewportRenderFrame`;
- viewport-owned particle history moves as its existing `Vec` from viewport state to submission
  context and then to the render frame. It is not copied into the shared particle domain and is not
  converted into a second heap owner;
- after a successful render, the previous-history vector is cleared, rebuilt with current particle
  state, and returned to the viewport record. Its allocation is therefore reused across frames;
- authored particle previous-state data remains authoritative. The renderer-owned fallback is only
  selected when the authored list is empty.

The resulting costs are `O(1)` for environment overlay selection and particle-history ownership
transfer. Particle-history production remains `O(current particles)` because the next-frame
snapshots must be generated, but it no longer adds `O(previous particles)` copying or clones the
entire shared particle domain. Focused tests cover immutable authored environment input, override
visibility at GPU consumers, authored-particle precedence, and exact previous-history allocation
reuse after a successful submit.

The ignored isolation benchmark now measures the real operations at 1, 1,000, and 10,000 probes or
particles. The legacy particle sample includes both the particle-domain COW and previous-history
vector copy inside the allocation/timing interval; the optimized sample measures the owned `Vec`
move. Environment measures the old shared-domain assignment against the compact shallow clone.
Measured Windows release values remain pending managed acceptance and are not yet claimed here.

Current source hashes for this slice:

| Path | SHA-256 |
| --- | --- |
| `core/framework/render/frame_extract/particle_extract_policy.rs` | `8e91d443d28a20c040fc4b8303d26062b9b86a8b5857be35921d88b4dbf55d97` |
| `build_frame_submission_context/build.rs` | `356c0065f28d1af52692b7e0fab09fb73391477212ef3442953af092eeeed302` |
| `build_frame_submission_context/build/effective_view_state.rs` | `c8617f66eea9fda3cb1f270eb6c40ccdffba855982181b9caa8c5c35e4434b10` |
| `build_frame_submission_context/environment_ibl_compile_options.rs` | `339c6823fbc550344d84c1419feffff8d699a23e4d50ca6cebc01d02bee32bf2` |
| `submit_frame_extract/frame_submission_context.rs` | `6e2eecc03e015bf4bbee8293fae404cec21a80f6a742dbefb2831f420d46d2fb` |
| `submit/build_runtime_frame.rs` | `5e36935242a5c2fe3e7b6ff6582937f5395a04b2e089c8afdea6ff4a9bbe2e83` |
| `submit/submit.rs` | `1727a6af86b606ccc0033eb2b6cdf612e861cba46a7a7c27737e699069f4a440` |
| `submit/present_frame_extract.rs` | `f3bf314af30a789ff608d2756e4a6e67955725fc397057c7903f1d6cd2c715ea` |
| `submit/submit_runtime_frame.rs` | `038b7ccfda30e0e207c59c47d578837a736105d1802033520b43b3727381f715` |
| `submit/record_camera_history.rs` | `25347d55935f80ca46eb91c85eb20e7eff294331a06cc96317cbba8e756f51c1` |
| `submit/update_particle_previous_state.rs` | `1b52c63fc11df17945104c8630b8462fa7752f5202d3d76c67b5662ccba43065` |
| `graphics/types/viewport_render_frame.rs` | `b1b7656549aab3da6c83d7073dccf847771b3ec9b423abee23da0999bc6add89` |
| `scene_renderer_core_write_scene_uniform/write_scene_uniform.rs` | `2b5fe2c872fd12c0b51ba7d1e31c588d01fe1bad97530dcb1d39375d101f425d` |
| `scene_renderer/primitives/scene_environment_sh9.rs` | `66ffd27cceb7868fbcc9a1e9c25ae04d3931f997625cdfa125cec5a3d5609881` |
| `scene_renderer/primitives/scene_uniform/from_frame.rs` | `b3bebfc7705a8101c25c9277ee06a3989e4840e5492bce6dfdadb6aaac2acfae` |
| `scene_renderer/particle/build_particle_velocity_vertices.rs` | `11807673abff60423dd5dcd12f7df2d6fe8bdfaa7d32e6d642992c7c0fd2b290` |
| `scene_renderer/ui/render.rs` | `041c58e9020c3f61275e55ee284de8f4d624078a517a4177a4434f5d0160c51a` |
| `tests/runtime_renderer_derived_lighting_compile_input_performance.rs` | `ce123ff67617d3c24ba64f386f74370c9b45f74b91b335dd2a87904b9bd8ffd8` |

The ignored M2 isolation benchmark covers 1, 1,000, and 10,000 directional lights. It compares the
old shared-domain material-usage assignment with construction of the compact renderer-owned storage
and reports allocation count, requested bytes, logical copied lighting bytes, peak live bytes, P50,
and P95. It is intentionally not an end-to-end frame or power benchmark. Measured release values
remain pending managed admission.

## M2 renderer-owned post-process and volumetric slice

The third M2 slice removes the remaining effective post-process writes and the camera-loop source
copy/restore algorithm:

- volume evaluation produces one `RendererPostProcessSnapshot` owned by the submission context;
  `FrameSubmissionContext` and `ViewportRenderFrame` share that exact snapshot through one `Arc`;
- the snapshot contains the resolved AO, bloom, exposure, color grading, effect stack, validated
  stack/graph, and volumetric-fog settings. Authored volumes remain immutable and are not copied
  into the renderer snapshot;
- the resolved exposure is now the GPU exposure authority. The old path built the graph with the
  volume-resolved exposure but left GPU exposure consumers on the authored base value;
- `AoSourceSettingsKey` preserves exact float bits in `RenderPipelineCompileOptions`, so graph-cache
  hashing and AO profile compilation consume the same volume-resolved settings without mutating the
  extract;
- resource streaming, graph execution, LUT baking, post-process parameter generation, temporal
  effects, and froxel media/scatter/integrate passes consume the frame snapshot accessor;
- the four froxel consumers no longer independently evaluate the same volume stack;
- `CameraLoopExtractSourceState` and `CameraLoopFrameSourceState` no longer clone or restore
  `volumes`, `stack`, or `graph` for every camera submission.

The structural cost is one required volume evaluation and one compact snapshot build per selected
camera, followed by `O(1)` Arc sharing across prepare/render/capture consumers. The removed path was
`O(camera submissions * (volumes + graph nodes + stack resources))` in additional cloning. The
ignored release benchmark measures four camera submissions at 1, 1,000, and 10,000 volumes and
reports allocation count, requested/copied bytes, peak live bytes, P50, and P95 under marker
`RUNTIME07_RENDERER_DERIVED_POST_PROCESS_V1`. Measured Windows release values remain pending the
managed receipt.

Current source hashes for this slice:

| Path | SHA-256 |
| --- | --- |
| `core/framework/render/post_process/ambient_occlusion_settings.rs` | `bd48760d0d502b6af2ef614a908a43d3f86793bb289bd1dd742cffb6a49bba12` |
| `graphics/pipeline/declarations/render_pipeline_compile_options.rs` | `f34bd385164c619c9e90dfde3a8af78e85e1ebf54d3594be6d09ab5cc7dee446` |
| `graphics/pipeline/compile_options/methods.rs` | `e8c029dfccd4f2d42df39a64b9fdc22fc45ac3f0af423296b7364b3b2c50ab42` |
| `graphics/pipeline/compiled_graph_cache.rs` | `9dde8b01879743f43cebd0857005d0982adfc6a5838c4231a65fae4089a1d9b2` |
| `graphics/pipeline/render_pipeline_asset/compile.rs` | `1cecccd445e0b5f2fc945b0104193183224739021b5487f293d83b2de1f1b9dd` |
| `build_frame_submission_context/build.rs` | `9d35750b4343907f8edf75f3dc6d787dc9fea0b67672b10cff27f3243a13cfce` |
| `build_frame_submission_context/build/effective_view_state.rs` | `442f1f4f638e148f19762672c41e3e1a2a24c516430f87607171b4510315fc45` |
| `submit_frame_extract/frame_submission_context.rs` | `81991c8738c54725761cb54731fd8813c5157f66ecd8962642f7d52baca1c18c` |
| `submit/build_runtime_frame.rs` | `c5a45429dc6142e9868386605c3461cef8d85e3c2fe4e2fb90321763a9f499d9` |
| `submit/camera_loop.rs` | `538be866974922f50a4ff026ae22c94bc2ed9830f035c69fdef6e94748e14c48` |
| `graphics/types/viewport_render_frame.rs` | `751149799ebe77c6dc05ac1af8264942372255139941aaac09f9c1165a397d6f` |
| `graph_execution/render_pass_execution_context/gpu.rs` | `2adb1d1f571999feb8977f50edca9840ab257fad25913e72de5a3b10fc78c6e1` |
| `execute/build_post_process_params/build.rs` | `8152bd7d8126a6d0685bfefa0d0f236387b993d5e6bcb3994118a845e5938bca` |
| `tests/runtime_renderer_derived_lighting_compile_input_performance.rs` | `7d5d0a989312f0e264c531398e48d733053f4f67b165c497d66403b9f1300710` |

## M2 immutable renderer submission and camera projection slice

The renderer submission path no longer takes fields out of the scene payload or receives a mutable
extract owner:

- `build_frame_submission_context_from_runtime_frame_extract` accepts `&Arc<RenderFrameExtract>`
  and derives one compact `submission_extract`; its scene Arc and every large scene domain remain
  shared with the generation source;
- `FrameSubmissionContext::submission_extract` names that renderer-resolved view/timing overlay.
  It no longer implies ownership of, or mutation authority over, the cache source;
- `FrameSubmissionSourcePayloads` is deleted. Camera submission does not `take()` virtual geometry
  or hybrid-GI payloads from the geometry/lighting domains and does not restore them through a
  second side channel;
- `ViewportRenderFrame::extract_mut` is deleted. Camera loop and environment capture can only
  replace a camera projection that shares the immutable scene Arc;
- submit, present, and direct runtime-frame entry points use the same read-only context-builder
  contract. The camera-frame restore state retains an Arc to the original extract and restores that
  handle plus small viewport fields between child submissions;
- renderer-authored budget, size, view-family, anti-alias, and jitter changes apply only to the
  newly derived submission overlay.

The initial immutable-source rewrite exposed a second structural cost. Cloning the source extract
before selecting one camera copied the complete `RenderViewExtract::cameras` vector and then
discarded all but the selected descriptor. Across `C` camera submissions this produced
`Theta(C^2)` descriptor copies and requested bytes. `RenderFrameExtract::for_camera_submission`
now constructs a single-camera view directly, shares the scene Arc, and copies only fixed view
fields plus one selected descriptor. Planning remains `Theta(C)` and all projections together are
`Theta(C)`; scene cardinality does not enter this cost.

The ignored Windows release benchmark
`camera_submission_projection_avoids_full_view_camera_clones` compares the removed full-view clone
against the direct projection at 1, 1,000, and 10,000 cameras. Marker
`RUNTIME07_CAMERA_SUBMISSION_PROJECTION_V1` reports allocation count, requested bytes, logical
camera-descriptor bytes, peak live bytes, P50, and P95. The deterministic source contract requires
the optimized logical descriptor work to remain one descriptor per projection. Measured values
remain pending managed acceptance.

Current source hashes for this slice:

| Path | SHA-256 |
| --- | --- |
| `core/framework/render/frame_extract/frame.rs` | `1b8d90699a213a14844d4ecca1d86d78752566b4787653d98cc311a252b22261` |
| `core/framework/render/frame_extract/view.rs` | `8ccd9860fb1b97a0889f1f505624c7a723868505ff5fe300ac0981a8d74e7858` |
| `core/framework/render/frame_extract/tests.rs` | `1359f4e6ed1d70ca117c04498cf30759faa6419e2d29ebdd688a9acc1d747ea2` |
| `build_frame_submission_context/build.rs` | `1ae2ffb5a03d68fb6489e5d686445d49f3377f8e4975a640cd64dd02abec1173` |
| `build_frame_submission_context/mod.rs` | `145284f23914ccc444f590b606d08a98b9ac9433e234586da62b77265e0837f8` |
| `submit_frame_extract/frame_submission_context.rs` | `ad1a4a7abcc373fe53413029c1c3a32eb134ff9fa1e8c128e1a198920e71a2b8` |
| `submit/build_runtime_frame.rs` | `01a0e31d3c623f271fd06950d7fe6ade33357366c6f54c8654a6caefba188179` |
| `submit/camera_loop.rs` | `65339d35292e7090037603adcd7f1f1e6dfe4db47df44b4e07b27faed10b165d` |
| `submit/camera_loop/tests.rs` | `528317fd09dc47cedf56f2ded382af3b736e3094dc4e386b420d55cdd189e7a7` |
| `submit/camera_loop/tests/frame.rs` | `ae30209c4d1ca7817f97523a021b325abb7a2f96c1ba616bc94e211b8726541d` |
| `submit/submit.rs` | `89087c15cb4a97d26f8b6981357fffe1155ac96354dff2753044c0059921397a` |
| `submit/present_frame_extract.rs` | `7fa89f26c6882126fb226cf07d73cf0a44943fb70d723ea41ec89a6eceec59f3` |
| `submit/submit_runtime_frame.rs` | `ecff7b08e0df13665b469c51286149d2665d9812ad1229aa098acd1a6a81d3f7` |
| `graphics/types/viewport_render_frame.rs` | `037abe563b283bd3c51c44941868c96c87550002870b0f03b479443b148b8faf` |
| `environment/environment_capture_scene_batch.rs` | `06f064bad7ff13ea302a2b1da2affdee197319a3c98a151e6bef850bd1f9b6bd` |
| `graphics/tests/render_framework_bridge/stats.rs` | `eb238d472affe5bc8639535bc62b8d5ac77132d072924b307ab2c15d6ad629b4` |
| `camera_loop_sharing.rs` | `778183956bc5141b03ddadba0be47389e31cfed9d68c4c49d13347da7e0d83d2` |
| `source_extract_payloads.rs` | `ec16607db9a863fabb3af5183f1efdb1ab38cad122a3f932919a59a8b5bff6c9` |
| `split_layout/route.rs` | `a5c5d06fc207c1d789b0bda9528060c85a410ef0708af7d01c063f20da54fb68` |
| `tests/runtime_renderer_derived_lighting_compile_input_performance.rs` | `ece6229ba9016a35aca60ddb75b244448648426497bf3f9888d484ef3feb31bb` |

This closes renderer production mutation through the runtime-frame and camera-loop APIs. It does
not yet remove `RenderFrameExtract::DerefMut` or mutable `RenderSharedSceneDomain<T>` from the core
construction API, so the final public type gate remains open and no full immutable-contract claim
is made.

## Reference-engine alignment

The local Unreal source is used for ownership and lifetime structure, not API copying:

- `dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/SceneView.h:2782` defines
  `FSceneViewFamilyContext` as the lifetime owner for per-family views.
- `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneRendering.h:2097` separates
  transient `FViewFamilyInfo` render resources from the scene.
- `SceneRendering.h:2239` states that `FSceneRenderer` is initialized on the game thread, passed to
  the render thread, rendered, and deleted when the render scope finishes.
- `SceneRendering.h:2161` keeps `FScene*` as a referenced scene owner while a renderer-scoped bulk
  allocator owns transient render data.

The corresponding Zircon rule is: keep the stable scene snapshot shared, make the view/submission
state lifetime explicit, and allocate derived render artifacts in the submission owner. Do not
mutate the shared scene snapshot to hold transient render decisions.

## Baseline profile contract

The ignored integration benchmark
`zircon_runtime/tests/runtime_frame_extract_shared_payload_performance.rs` uses a process-local
counting allocator and `Instant` around the operation under test. It creates exactly:

- 10,000 mesh snapshots;
- 10,000 directional-light snapshots;
- 10,000 sprite snapshots.

It reports 17 measured samples after 3 warmups at 1, 1,000, and 10,000 items per large domain:

`allocation_count_p50`, `requested_bytes_p50`, `peak_live_bytes_p50`, `elapsed_p50_ns`, and
`elapsed_p95_ns`.

The direct F-drive baseline recorded in the canonical Runtime07 failure measured the old clone:

| operation | P50 | P95 | allocations/op | allocated bytes/op | copied bytes/op |
| --- | ---: | ---: | ---: | ---: | ---: |
| old stable hit | 836.2 us | 1,350.7 us | 3 | 1,920,000 | 1,920,000 |
| old miss | 2,477.5 us | 6,679.4 us | 6 | 3,840,000 | 1,920,000 |
| shared-base ownership model | 4 ns | 17 ns | 0 | 0 | 0 |

The ignored current-source benchmark measures the exact clone operation used by cache retain and
cache return at 1/1k/10k mesh-light-sprite cardinalities and additionally reports
`copied_scene_bytes_p50`. It requires zero copied scene bytes, cardinality-invariant allocation
count/requested bytes/peak live bytes, and less than 64 KiB of bounded overlay/handle allocations.
It does not claim M4 zero allocation because the owned view overlay can allocate independently of
scene size. The managed Windows release result, exact stdout hash, and measured values are pending.

## Managed validation state

- Baseline ticket `5979086274ae4e01b969a6155934c39f` failed before the benchmark because the
  validation source union called `CoreHandle::active_module_shutdown_order` without copying the
  current defining blob. Stderr SHA-256:
  `65a25f28cdcec2ed270492e189d09a3467161cb6fdebbad73b1e87ee7dc1fd8f`.
  The method already exists in the current workspace, so no duplicate runtime fix was admitted.
- M1 release ticket `693854d59b4140968c949926560c2d5f` was submitted with source-manifest
  SHA-256 `2deff7c6df9bd7b27f883fa59e15026d25a3f7c0f337588bbb62489098c5d409`,
  then superseded before acceptance by the real-cache regression, diagnostics correction, and V3
  three-cardinality benchmark. It cannot accept the current source and is not polled.
- Current behavior ticket `b27f2c10f9114236973377181a51b9cb` runs the real-cache unit filter.
  Current release performance ticket `e479bfc0b2e64aaea6689f589b339a92` runs the ignored V3
  1/1k/10k benchmark. Both use source-manifest SHA-256
  `067bda0cb4e18f1adf2ed54864c0a3548762149572918b25e8a502df35b23cd3`,
  returned initial status `queued`, explicitly supersede the stale ticket, and are not polled.
- The first M2 focused request attempt was rejected before ticket creation with
  `request_overloaded`. After the source implementation and final exact-path attribution completed,
  the idempotent request `runtime07-m2-derived-lighting-lib-20260831-r1` was admitted as ticket
  `62eba79a779e425bb60747abf01143fb` with source-manifest SHA-256
  `996d5b4a6f0acccfb3a006b16e207bb16c50f8759695f5d7bfa17d9a7a9b6571`.
- Release isolation request `runtime07-m2-derived-lighting-benchmark-20260831-r1` was admitted as
  ticket `4d9ba802cf5f40c0b3df5792196b81e9` with source-manifest SHA-256
  `5af55e802e4994efe08c941ca45450f4401026b2575dfb9c08b0e5787d9799d8`. Both M2 tickets returned
  initial status `queued` and are not polled. No Cargo-green, measured-value, or power claim is
  attached before their terminal managed receipts.
- Environment/particle focused lib ticket `62792a9e6b6946c7a4d87b62f845b541` was admitted with
  source-manifest SHA-256 `02e398f37fb791bb00c3b22d88d8ca9049fb3c04c91faf504162324aa3568b40`.
  Its release benchmark request was accepted under recovery request
  `9b183165033644e1a149be0e2315a530`; ticket reconciliation remains pending and is not polled.
- Post-process/volumetric current-source snapshot `2443` was admitted for focused lib validation as
  ticket `15dcd9757350476486330786bf81c8c9` with source-manifest SHA-256
  `5f3745703936b584d8c028acda7117824d1b52524cf80d3fa7a053bbccd5a5b2`.
- Release request `runtime07-m2-post-process-benchmark-20260831-r1` was accepted but returned
  `command_post_timeout`; durable recovery request `4b069a5f00e74975a624fd5b52039b3e`
  remains pending and is not polled. No measured post-process value is claimed.
- The post-process benchmark file passes current `rustfmt --edition 2024 --check` in isolation.
  The complete touched-source rustfmt parse completed but the shared tree retains pre-existing
  format-order differences, so no full-format green claim is made. No Cargo-green, measured value,
  commit, or milestone-close claim is made before a current-source managed receipt.

## Implementation milestones after baseline

### M1 - Shared storage contracts

- Add the shared scene payload and owned submission overlay in named leaf modules below
  `core/framework/render/frame_extract`.
- Keep one canonical submission DTO at the `RenderFramework` boundary.
- Move the dynamic cache to a shared scene-payload owner; hit and miss return submissions pointing
  to the same retained payload without cloning it.
- Keep editor camera and timing overlay changes local to each submission.

### M2 - Renderer derivation ownership

- Replace `Arc::make_mut` of the complete extract with a renderer-owned derived submission context.
- Compute material/subsurface summaries without mutating authored lighting.
- Carry hydrated environment, particle history, effective post-process, budget, and selected-view
  outputs as derived fields.
- Preserve multi-camera source semantics without taking or restoring fields in the shared payload.

### M3 - Product consumers and generation parity

- Capture and present accept the same submission DTO.
- Viewport pick retention keeps a shared scene handle plus the exact submission generation.
- No compatibility overload accepts an owned deep-cloned extract.

### M4 - Performance and behavior acceptance

- Unchanged cache hit: large scene payload pointers are identical and deep copied bytes are zero.
- Cache miss: one payload build and no second full-payload clone.
- Overlay mutation: editor camera/timing is visible in the submission and cannot mutate the cache.
- Each key component independently forces a rebuild.
- Capture and present observe the same world generation.
- The 10k benchmark reports allocation count, requested/copied bytes, peak live bytes, and elapsed
  time for baseline and optimized implementations.
- Acceptance target: unchanged-hit large-payload requested bytes are zero; elapsed time is
  effectively constant with scene cardinality. Product power comparison remains open until a real
  Windows WGPU capture provides CPU package-energy or equivalent telemetry; a microbenchmark must
  not claim engine-level power parity.

## Current completion state

- [x] Reviewed the complete dynamic cache -> session overlay -> framework -> camera loop -> frame
  context -> viewport frame/pick dataflow.
- [x] Proved why a cache-level `Arc<RenderFrameExtract> + Arc::make_mut` is invalid.
- [x] Reviewed renderer mutation ownership and classified it as submission-derived state.
- [x] Grounded the ownership direction in the local Unreal source.
- [x] Added an allocation/timing baseline benchmark contract.
- [x] Recorded the direct-current-source historical clone baseline and its limits.
- [x] Implemented the M1 shared payload/overlay storage split in current source.
- [x] Migrated world/level producers, synthetic consumers, and whole-domain test assignments.
- [x] Added shared-pointer, real-cache overlay isolation, per-domain COW, and four-key admission
  regressions.
- [x] Corrected diagnostics to report zero full-scene clones for cache retain/return.
- [x] Replaced material census and SSS profile writes with one renderer-owned advanced-lighting
  compile input.
- [x] Normalized authored fallback and renderer override before graph-cache hashing and compilation.
- [x] Added exact-profile-bit, Arc-sharing, no-source-mutation, and 1/1k/10k COW-isolation tests.
- [x] Moved hydrated environment selection and particle previous-state fallback into the renderer
  submission context without mutating authored scene domains.
- [x] Routed environment and particle GPU consumers through the canonical runtime-frame overlay.
- [x] Recycled particle-history vector storage after successful render and aligned the 1/1k/10k
  allocation benchmark with the production ownership transfer.
- [x] Moved effective post-process, validated graph, resolved exposure, and volumetric fog into one
  renderer-owned snapshot shared by submission and render consumers.
- [x] Removed per-camera post-process volume/stack/graph clone-and-restore and added the four-camera
  1/1k/10k volume isolation benchmark.
- [x] Removed `FrameSubmissionSourcePayloads`, virtual-geometry/HGI scene-domain take/restore, and
  mutable runtime-frame extract access from renderer production submission.
- [x] Replaced full-view clone-per-camera with a scene-sharing single-camera projection, reducing
  camera descriptor work from `Theta(C^2)` to `Theta(C)` and adding a 1/1k/10k benchmark.
- [ ] Managed M1 current-source result accepted and recorded.
- [ ] M2 core `RenderFrameExtract::DerefMut` / mutable domain compatibility surface removed after
  all construction and test producers migrate to explicit builders.
- [ ] M3 product consumers migrated.
- [ ] M4 focused behavior, performance, WGPU product, and power evidence accepted.
