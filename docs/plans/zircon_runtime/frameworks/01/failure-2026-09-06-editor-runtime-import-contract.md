---
handoff_kind: failure
status: open
created_at: 2026-09-06
summary_slug: editor-runtime-import-contract
plan_link_mode: child_record_only
origin_plan: docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
fixing_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
origin_child_dir: docs/plans/optimize/zircon_app/08
fixing_child_dir: docs/plans/zircon_runtime/frameworks/01
related_code:
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba16float_3d.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests/frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_particle_previous_state.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs
  - zircon_runtime/src/graphics/shader/builtin_global_shader_contracts.rs
  - zircon_runtime/src/graphics/shader/template/tests/material_template_assembly.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process_full_chain/visual_export.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
  - zircon_runtime/src/graphics/tests/render_perf_baseline.rs
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/native.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/table/selection.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/runtime_states.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_device_epoch_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render/render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline/frame_submission_owner.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mod.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/crates/zr_resource/src/readiness_generation.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/manager.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/runtime/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/environment_capture_scene_batch.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer
  - zircon_runtime/src/graphics/shader/global_pipeline_layout.rs
  - zircon_runtime/src/graphics/shader/invocation/mod.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors/schema_allocation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/hzb_history.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/scene_frame_history_textures.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder/parallel_admission.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder/parallel_preparation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder/profiling_contract_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout/rich_artifact_routes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_batches.rs
  - zircon_runtime/src/graphics/backend/mod.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/resources/mod.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/semantic_executor/owner.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/prepare_compiled_scene_mesh_submission.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/resource_resolver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/upscale_params.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/ssao_input_qualification.rs
  - zircon_runtime/src/text/mod.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/render_graph/graph/resource_state_plan.rs
  - zircon_runtime/src/render_graph/builder/compile.rs
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui/input_publication.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui/input_routing.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/scene/render_scene/component_projector/projector.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_compiled_scene_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/generic_compute_executor/buffer_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/external_access_bindings.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/persistent_texture_access_bindings.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_blur/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_depth_of_field/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_motion_blur/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_scene_composite/mod.rs
  - zircon_runtime/src/ui/layout/pass/responsive_mui/candidates.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_layout.rs
  - zircon_runtime/src/ui/v2/component_instancer.rs
  - zircon_runtime/src/ui/surface/virtual_list_prototype_pool.rs
  - zircon_runtime/src/ui/surface/surface/compiled_binding_event_index.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset/shape_projection.rs
  - zircon_runtime/src/graphics/runtime_prepare_collector.rs
  - zircon_runtime/src/graphics/scene/resources/ui_texture/prepare_receipt.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/compute_pipeline_cache/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages_tests.rs
  - zircon_runtime/src/graphics/backend/render_backend/product_diagnostic_delivery_router.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_morph_weights.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_target/required_offscreen_target.rs
tests:
  - managed Editor target-editor-host check with dev-dynamic linking
  - python -m unittest tools.tests.test_frameworks_01_shader_invocation_owner_boundary -v
  - python -m unittest tools.tests.runtime_text_infrastructure_compile_contract.context_geometry -v
---

# Frameworks01: Editor Runtime import contract

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md`
- 来源执行切片：App08 Editor development-DLL acceptance
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 交接原因：the Editor check reaches Runtime graphics files owned by the archived Frameworks01 source boundary.

## 失败现象与复现证据

The managed Windows command reached Cargo check and failed before code generation with 206 errors. Representative errors are unresolved imports of `ComputeDispatchPlan`, `FullscreenPassPlan`, `ShaderNamedResourceBinding`, and migrated graphics symbols from `zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/construct.rs`. The sealed source hash matched the working file hash; this is current source state, not snapshot drift.

Reproduce through the managed entry:

```powershell
& .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -Features zircon_runtime/target-editor-host -NoDefaultFeatures -LibTests -CheckOnly -LinkMode dev-dynamic
& .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -Features target-client -NoDefaultFeatures -RuntimeProductDll -SkipTest -LinkMode static
```

Editor evidence: `.codex/tmp/app08-editor-dynamic-20260906-r2.log`. The product command independently failed with the same import/visibility error families and exit code 101; evidence: `.codex/tmp/app08-product-client-20260906-r3.log`, job `fbf7a97d89b74590a53a364b58f2a546`, sealed input digest `2967c99b9413f3cf9d87945bbc9dfef5172e3377983a7880a2b9e30f6d9ee935`. Neither failure is performance or ABI success evidence.

## 最低共享层根因

The Runtime graphics export and import surface is not converged after the shader invocation hard cut. The Editor host consumes the normal Runtime path and cannot repair this at its boundary.

## 架构修复验收

- Restore the owning Runtime graphics exports/imports through the canonical module boundary.
- Run the focused Runtime graphics contract checks first.
- Rerun the exact managed Editor target-editor-host check and then its focused test.

## 禁止临时方案

- Do not add Editor-only aliases, compatibility shims, or test-only exports.
- Do not weaken the Editor check or exclude the failing Runtime modules.

## 修复结果与回传

Open state: `待修复`; no Editor pass or DLL-reuse claim is made. The shader
invocation imports have been moved to their canonical owner in the working tree.
Current target-client test-profile job `9e45fc19097d4bfb8267d1647e40cc2a` no longer
reports those original import failures, but still reports 503 source errors,
including 308 coded graphics errors from other import, visibility, and test
contract mismatches. Log: `.codex/tmp/app08-taffy-client-20260906-r2.log`.
The check stops before code generation, linking, and tests; the Editor/product
commands cannot yet supply acceptance or timing evidence.

The production-library check also remains red after the Taffy/import repairs.
Managed static job `20cb9378ea4f4d8eae2ff4b35464f95a` checked `target-client`
without test targets and reported 197 source errors: 156 in graphics, 26 in UI,
8 in dynamic_api, 4 in render_graph, 2 in script, and one reported at a std bound.
This rules out a test-only blocker. The sealed digest was
`349d73696525d60d95a58f36373016c13113039aca2253e4b05fa9ce2e54ea4e`;
queue 11.53 s, source synchronization 403.07 s, check 290.11 s, with zero
compile/link and test time. Log: `.codex/tmp/app08-runtime-client-lib-20260906.log`.
The App server dynamic check now passes after its local host feature/import and
configuration fixes, but Runtime DLL linkage has a separate PE export-count
failure owned by App08; it does not change this source-boundary handoff.

## Additional Consumer Evidence

Designment02 dual ZUI acceptance reproduced the same 206 Runtime compile errors on 2026-09-06 through managed job `e3513aa0f91b4d4dafc39bf68f58bbff`: `-Package zircon_runtime -Features ui -TestTarget zui_native_visual_acceptance -IgnoredTests -TestFilter export_all_zui_native_visual_acceptance -SkipBuild -StorageMode reuse -Linker system`. Cargo check exited 101 before the screenshot test. Log: `docs/_data/layout/evidence/windows-runtime-validation.log`; sealed input digest: `fc66884ec96fc38d691fbec84fb70c9a8605613e384a4b66f9e2559eaae48692`. All 320 dual acceptance entries remain unaccepted. This adds a consumer to the existing failure; ownership and return destination are unchanged.

## 2026-09-07 Current Graphics Boundary Repair

Owner Session `failure-roll-01a07160-frameworks01` compared the production-library
diagnostics with current source before changing seven exact files. The earlier
`render_feature_pass_descriptor/construct.rs` import repair was already present.

- `global_pipeline_layout` and `ibl_bake_wgpu_command_plan` now import invocation
  plans, parameters, cache keys, and binding constants from `graphics::shader::invocation`.
  Neutral resource descriptors remain under `core::framework::render`.
- History construction imports its private texture helpers from their immediate
  owner; the history state imports domain types from the containing history
  module. HZB construction uses the existing `graphics::visibility::HzbBuilder`.
- The schema-allocation helpers expose only the enclosing `render_pipeline_asset`
  scope required by the existing catalog and authoring consumers.
- Removed invalid crate-wide re-exports of compiler-private validators. All actual
  validator consumers already use the compiler owner within the invocation tree.

Ownership transfer `262df422538d4ad1bdaaf58c4f955c88` and attribution
`1e747c3ea80e49c6b4f8c878b781e90b` retain prior source edits. Snapshot `2877`,
request `4d0fae8ea836428eb0715c425fd9b320`, freezes the seven source files.
The existing Python shader invocation boundary suite passed 6/6 in 8.133 seconds;
scoped Rust 2021 rustfmt and `git diff --check` passed.

These are partial source repairs and structural checks, not a successful Runtime
or Editor compilation. The remaining error families in the original log still
require current-source diagnosis. The shared immutable Cargo admission prerequisite
is still blocked by external `E:\Git\zr_vm`, which the user excludes from this
cleanup; no duplicate admission request was sent. Managed Runtime and Editor
checks, focused Rust tests, review, canonical return, and closeout remain pending.
This lifecycle stays open and has no repair commit or WeCom closeout receipt.

## 2026-09-07 Split Module Visibility Repair

Eight further source/test files repair visibility lost when implementation files
became nested modules. Actual consumers were checked before adjusting each scope.

- Environment exports retain graphics-wide capture reservations while limiting
  probe buffers, GPU bindings, layout entries, and the planar size constant to
  their existing scene-renderer owner.
- SSAO descriptor helpers expose the enclosing builtin-descriptor module needed
  by dispatch. Mesh parallel-build helpers expose the enclosing command-list
  module needed by its builder and existing cache regressions. Profiling contract
  extraction now locates the function signature independently of its visibility.
- Rich glyph routes and their payload fields expose the UI render module needed
  by rich-text planning. Text batches and their fallback fidelity query expose
  the UI owner needed by SDF/native consumers; render-only route context stays
  private to render and is no longer exported to UI.

Ownership transfer `0a8a360e557c492d918cb75f18cd2ebe` and pre-edit snapshot `2882`
preserve the inherited source changes. Attribution
`d2ac97eefe3447feb79853d60ff10fe0` and source snapshot `2883`, request
`2aa785407fee47859dac12275f317dee`, bind all 15 repaired source/test files to the
same Frameworks01 Session. Scoped Rust 2021 rustfmt and diff checks passed for
the eight changed files. No Rust compilation or test execution is claimed.

The original production-library diagnostics still require further current-source
triage. The recorded external Cargo admission blocker remains pending without
repeated submissions; no failure return or closeout has been attempted.

## 2026-09-07 Production Consumer Import Repair

Eleven further files repair current production imports and missing exports in the
same Runtime boundary failure:

- Backend exports the existing source-cubemap readback functions and result types
  to graphics. Scene exports the existing device epoch and resource factory;
  scene-renderer exports its existing capture persistence states. Scene resources
  exposes the existing world-release error only to the scene owner.
- Asset generation identities now come from the manager owner. Frame-graph output
  names and half-resolution executor IDs use their existing pipeline/transparency
  paths. Mesh hashing refers to the canonical asset vertex type.
- SSAO qualification and graph-resource resolution import their missing resource
  enums. Upscale parameters and their constructor expose only post-process,
  matching the existing buffer-allocation and execution consumers.

Transfer request `5ba839456843414386638b70a985bf77` covers ten inherited changed
paths; `graphics/scene/mod.rs` matched the shared baseline and was claimed before
its new edit. Pre-edit snapshot `2886` preserves all eleven inputs. Attribution
`012e3071b15e4b8b84e320d6941011d1` and source snapshot `2887`, request
`5ed5b2a3dc50463dac9d05e1e9001e02`, bind the combined 26-file source repair.
Scoped Rust 2021 rustfmt and diff checks passed for this batch.

Independent read-only review of the earlier 15-file snapshot `2883` was requested
once from task `01a07160-5337-7570-a507-ed6decf2d32b`. These fifteen source hashes
remain unchanged in `2887`. The eleven new files still require independent review;
all managed Runtime/Editor compilation and dynamic acceptance remain pending.

## 2026-09-07 Shared Geometry and Dynamic Session Repair

Independent source review accepted snapshot `2883` with Critical 0, Important 0,
Moderate 0. Review of the eleven additions in `2887` returned Critical 0,
Important 0, Moderate 1: existing upscale execution tests also read the parameter
field. `UpscaleParams::input_output_size` now shares the enclosing post-process
scope of its type and constructor, preserving those existing layout assertions.

Seven additional current-source repairs address the original production log:

- The shared text geometry module exposes its existing crate-scoped numeric
  helpers to UI consumers. Frame hit testing imports the existing bubble-route
  lookup from its tree owner.
- Render-graph state plans import access intent/range from their actual access
  owner, and compilation imports the existing resource-name helper.
- Dynamic operation admission reads the ABI byte slice's existing `len` field.
  Input dispatch methods expose only their enclosing session, as required by
  session event consumers. Input query construction moves its non-Copy query
  through exactly one branch, retaining physical and optional virtual coordinates.

Transfer `df14763bf5374ed8aac9b9640b2c3962` and pre-edit snapshot `2889` preserve
the seven input files. Source snapshot `2892`, request
`62de980df1d04430815401eb82dfedc7`, freezes all 33 files. Attribution
`9e0f0e4a549943d4b3f8889437d2ec1b` was accepted after renewing expired leases and
rechecking all eight changed hashes against `2892`. Source leases were released.

The existing context/geometry Python suite passed 18/18 in 0.024 seconds; scoped
Rust 2021 rustfmt and diff checks passed. Existing Rust input-publication, session
event, graph, and upscale regressions still require managed execution. This batch
and the review correction await independent review. No Rust pass, completed
failure, canonical return, commit, or notification is claimed.

## 2026-09-07 Typed Consumer Repair

Thirteen further files repair independent type mismatches verified against the
original production diagnostics and current definitions:

- Texture allocation retains an optional scene-linear fallback with `or_else`.
  Generic compute binding reads the access key's own range. Texture-view caches
  declare their value type before cloning a retrieved view, and imported-target
  construction preserves its `GraphicsError` result type across `transpose`.
- Four post-process consumers pass a full-range `wgpu::BufferBinding` for their
  existing standalone light buffer, matching the current bind-group constructor;
  exposure bindings retain their existing exact ranges.
- Scene resource release and responsive-layout admission methods expose their
  existing enclosing owners. Component instancing declares its node-map type.
  Rich inline advances return the existing typed `Ready` outcome.

Transfer `2ffa2ab3db7145efab3edb9b99b6ffe2`, pre-edit snapshot `2895`, and
attribution `84f66d0a3fec46b79c411c3aea93eabd` preserve inherited changes and the
same owner. Source snapshot `2896`, request `55c234c2cda444d68ce1bc4dfe62f03f`,
freezes all 46 repaired source/test files. Scoped Rust 2021 rustfmt and diff checks
passed; all thirteen source leases were released. Independent review is pending.

The related paragraph source-range mismatch remains in an active Text03-owned
file: `ui/text/layout_engine/paragraph_layout.rs`. Transfer preview
`d0318972539548a19ebf7e182a0f81b8` returned `source_owner_executable` for Session
`text03-current-proof-r1-bee4c707-20260822`; the current hash was
`43688955fff00fe5f6a6fd525f80ec666c4b8027ceecc2a76b6383b469c5db19`.
This path was excluded from the claimed batch, so its original diagnostics remain
pending with its owner while independent Frameworks01 repair continues.

## 2026-09-07 Access API Repair and Review Results

Independent read-only review accepted snapshot `2892` with Critical 0, Important
0, Moderate 0, resolving the prior upscale-field finding. A subsequent review
accepted all thirteen additions in `2896` with the same zero counts; its combined
46-file conclusion retains the unchanged earlier evidence. All 46 source hashes
matched at both review boundaries. Neither review is compilation evidence.

Three further current-source repairs are prepared:

- Virtual-list prototype capture reads `UiTree::layout_slots()` instead of the
  now-private slots field; root matching and internal-edge filtering are unchanged.
- Compiled binding-event entries and their handle expose only the containing
  surface, matching the existing editable-text consumer and accessor return type.
  Other entry fields and index construction retain their narrower scope.
- Texture-shape error context accepts `Display + ?Sized`, allowing both current
  structured resource locators and the existing string regression fixture without
  allocating a URI string on successful validation.

Transfer `9a6fac4e413d4713a7317126b400741e` and attribution
`1a7f583d444b42488700d91de0c8e6df` bind the three files to the same Session. Source
snapshot `2901`, request `08c96752a52747309f3fb37e0c68e706`, freezes all 49 files.
The three inputs were hash-checked under live leases before editing; scoped Rust
2021 rustfmt and diff checks passed. Independent review remains pending.

The active Text03 paragraph-range dependency is now tracked by
[its canonical handoff](../../text/03/failure-2026-09-07-paragraph-list-prefix-range-contract.md),
record snapshot `2900`. Its stable lifecycle is registered in the coordinator.
The repository handoff validator reports no errors for that new record; 74
pre-existing errors elsewhere remain outside this repair's acceptance claims.

## 2026-09-07 Capture Work Item Export Repair

The two capture consumers reached `EnvironmentCaptureWorkItem` through the private
`graphics::runtime::render_framework` module. The runtime owner now exports the
existing work item within `crate::graphics`, and both consumers use that export.
The type, scheduler, and capture behavior are unchanged.

Frameworks01 source snapshot `2904`, request `8d57f63d63344612a3eaa60e49f63986`,
binds `graphics/runtime/mod.rs` and `environment_capture_scene_batch.rs`.
Attribution request `1deb7f30ee6342df8a422da9f315b9f8` records their exact hashes.
The other consumer, `core/scene_renderer_environment_capture.rs`, remains owned
by the Render11 Session in its separate source snapshot `2905`, request
`129e556d27d24ccb979535e5b1681bd2`, and attribution
`77340db76cc843edb1a5f9653f29c880`.

Scoped Rust 2021 rustfmt and diff checks passed for all three paths. Independent
review and managed compilation remain pending. The recorded external `zr_vm`
admission blocker is excluded by the user; no repeated Cargo admission, dynamic
pass, canonical fixed return, commit, or notification is claimed.

## 2026-09-07 Current Consumer Boundary and GPU Fallback Repair

Four remaining diagnostics were confirmed against current definitions and repaired
at their owning Runtime boundaries:

- `RuntimePrepareGpuReadbackRequest` is re-exported from the collector with
  `pub(crate)`, matching the existing `graphics` facade consumers.
- Render-pipeline schema descriptor helpers are re-exported to the enclosing
  `render_pipeline_asset` module, matching both catalog and authoring consumers.
- UI texture scan accounting declares its `usize` counter before calling
  `saturating_add`, preserving the receipt's existing return contract.
- Graph-pass GPU result collection uses an explicit empty tuple for a missing GPU;
  this avoids Rust's twelve-element tuple `Default` limit while preserving the
  existing no-GPU values for every recorded field.

Independent review accepted capture snapshots `2904` and `2905` with Critical 0,
Important 0, Moderate 0. Review of the earlier access-API snapshot `2901` returned
Critical 0, Important 0, Moderate 1: virtual-list fixture helpers still directly
read the private `UiTree::slots` field. The same owner now uses `layout_slots()`
for capture and fixture inspection and `push_layout_slot()` for fixture setup,
preserving UI-tree layout authority.

Source snapshot `2914`, request `515c5013e88c4134ada9f1f03c8dbefd`, freezes the
five current Runtime/UI files. The `2901` aggregate had a moving
`resource_descriptors.rs` hash during review, so it is retained only as individual
source evidence and is not used as a stable aggregate acceptance claim. The source
changes retain unrelated edits from other Sessions. `git diff --check` and targeted
static source-contract inspection remain the available local evidence.

The managed Runtime/Editor compile and focused Rust test remain pending because the
shared admission prerequisite is still blocked by external `E:\Git\zr_vm`, which the
user explicitly excluded. No dynamic pass, canonical fixed return, commit, or WeCom
closeout is claimed.

## 2026-09-07 Readiness Consumer Migration Diagnosis

The original production log reports eight E0599 diagnostics at lines 4490-4565:
render-asset residency, shader preparation, and material preparation still call
`sequence()` or `dependency_revision()` on `ResourceReadinessGeneration`.
These APIs were deliberately removed when the shared owner adopted retained
`ResourceReadinessGenerationIdentity` and `ResourceReadinessRowIdentity`.
Diagnostic publication counts and dependency revisions are not public cache
identities. Asset-facade `row()` calls already use the existing
`ResourceReadinessGenerationAssemblyExt`; they do not require wider visibility.

The attempted accessor restoration in snapshots `2917` through `2920` was
withdrawn after the existing
`test_projection_diagnostic_counters_are_not_public_identities` regression
executed and failed at line 197. The narrow snapshot `2919` review returned
Critical 0 / Important 0 / Moderate 0 but did not cover this governing contract;
it must not authorize integration of those accessors. Both resource source files
were restored to their pre-edit contents. Earlier Frameworks01 graphics/UI
repairs remain intact.

After withdrawal, the unchanged `tools.tests.test_frameworks_01_resource_crate_boundary`
suite executed 15 tests in 3.723 seconds and passed. The generation and projection
test hashes match pre-edit snapshot `2916` exactly (`8441fc6933442026...` and
`982eb7e48f32433b...`); `git diff --exit-code` confirms neither resource file has
a remaining source diff.

The remaining repair must migrate the graphics consumers and their cache/ticket
contracts to retained publication identities, including removal/re-add and
cross-manager identity regressions. Managed Runtime/Editor validation remains
pending while the user-requested external `zr_vm` exclusion is in effect. No
dynamic pass, canonical return, commit, or WeCom closeout is claimed.

## 2026-09-07 Borrow Lifetimes and Submission Receipt Repair

Independent review of source snapshot `2914` completed with Critical 0,
Important 0, Moderate 0; all five hashes matched at review start and end.
The reviewer ran three related Python structure suites: 16 of 18 tests passed.
Two failures are outside that snapshot: the inline-resource receipt guard still
expects `UiTextureDependencies` instead of the current
`Arc<UiTextureDependencies>`, and `ui/image.rs` is 829 lines against its 800-line
limit. These results do not establish an all-green Runtime/UI structure gate.

The remaining E0499/E0502/E0382 diagnostics were confirmed in current sources.
Pre-edit snapshot `2923`, request `d4647f34c0f940098300eb8bcd20532e`, preserves
all existing edits from other Sessions. Source snapshot `2924`, request
`116796f3b8cf44a08241eedbc6264415`, binds five targeted changes:

- Device-epoch cache lookup reads a Boolean match before any mutation and returns
  the retained value only after the replacement branch completes. Old entries
  are still released before creation, and creation failure still leaves no entry.
  The existing reuse/replacement test now also covers a changed device ID with
  the same generation and key.
- Both viewport runtime maps use a borrowed `contains_key` check before insertion
  and the final mutable lookup. Hits still avoid cloning the camera key or
  invoking the provider. The existing performance helper uses the same valid
  borrow order; its 25-percent performance gate has not been rerun.
- GPUScene reads the cached live-instance count before borrowing its mutable
  entry, shadow, and update fields. Its existing dirty-key structural regression
  now names the current `stats()` accessor, preserving the no-full-scan check.
- Direct and pipeline render entry points both read logical packet count before
  consuming the submission receipt in `with_submission_metrics`.

The five paths passed read-only Rust 2021 rustfmt parsing and `git diff --check`.
Independent review of `2924` was requested from the existing coordinator-review
task. The retained source tests for cache reuse/replacement, viewport runtime
providers, dirty transform rolling, and both submission entry points still need
managed Rust execution. Snapshot `2924` has no matching managed dynamic pass;
the external `zr_vm` admission prerequisite remains excluded by the user.
No canonical return, commit, or WeCom closeout is claimed.

Independent review of `2923` -> `2924` subsequently completed with Critical 0,
Important 0, Moderate 0. Review-start preview
`d8f583859ecd41249bc637b1afd7537f` and review-end preview
`60ec711cb1c0441082ffe7f6e38267ae` both matched all five source hashes. The
review explicitly retains the original runtime-map 25-percent performance gate
and does not claim Rust execution or Runtime/Editor integration.

## 2026-09-07 Texture Consumer Type Repair

Two further original diagnostics remained in the current consumers. The
`SourceCubemapEnvironment` override is an owned optional value, so its getter
now uses `as_ref()` to return the existing borrowed-value contract. Its fallback
to the extracted environment remains unchanged. The private final-output schema
helper is now a runtime function: its `TextureUsage` union uses non-const
`BitOr`, and all three callers construct runtime feature descriptors. Texture
format, usage bits, extent, and caller behavior are unchanged.

Pre-edit snapshot `2926`, request `d8b7ebd26cdb4f7eb641318718208c2b`, and source
snapshot `2927`, request `d758f1c2fda4466e8da72ebfda7a8823`, bind the two one-line
changes. Both paths passed read-only Rust 2021 rustfmt parsing and scoped diff
checks. Independent review was requested. Managed Runtime/Editor compilation
remains pending; no dynamic pass or closeout is claimed.

Independent review of `2926` -> `2927` completed with Critical 0, Important 0,
Moderate 0. Start preview `faaaa0a293eb4682b85a84876b724381` and end preview
`cebd4a3d46bc421d9189c3a6a104b6c7` matched both hashes. This is static evidence.

## 2026-09-07 RenderGraph View-Format Equality Repair

The original E0277 at `render_graph/builder/compile.rs:318` came from collecting
`TextureFormat` values into a `HashSet` although their shared contract supplies
`Eq` without `Hash`. The existing WGPU descriptor validator already compares
earlier view formats through a slice. Graph validation now uses the same
`enumerate`/prefix-`contains` pattern, retaining the prior rejection category and
message. The preceding supported-alternate-format check is unchanged; admitted
formats are restricted to the existing sRGB reinterpretation pairs.

Pre-edit snapshot `2930`, request `4b8fa89bcd474475aae95fa2e1fc7910`, and source
snapshot `2931`, request `bd31bac6235c44a9987e17dc70b1255a`, freeze the one-file
change. Existing `builder_rejects_invalid_texture_view_format_declarations`
covers parent-format repetition, duplicate alternates, and unsupported formats.
Its managed execution remains pending. The 969-line compilation module adds no
new helper or responsibility; this patch keeps its size unchanged and does not
mix a module extraction with descriptor-contract repair.

Read-only Rust 2021 rustfmt parsing and scoped diff checks passed. Independent
review was requested; no dynamic or integration pass is claimed.

Independent review of `2930` -> `2931` completed with Critical 0, Important 0,
Moderate 0. Start preview `8f6063a617e841cb9ee68f983ff8ab7f` and end preview
`30fc79ba3a9742bea8b30fb608935fb0` both matched the source snapshot. The reviewer
confirmed that the preceding one-alternate-format contract bounds duplicate
detection and that other graph checks still require the existing `HashSet` import.

## 2026-09-07 Native Factory and Table Hit Repair

The native GPU factory had two applicable `create_buffer_init` trait methods.
It now explicitly calls `DeviceExt::create_buffer_init(self.device, descriptor)`
after the existing single metrics increment. Table row selection now consumes
the `UiNodeId` value returned by `UiPointerRoute::hit_candidates()` directly,
removing the invalid dereference. No selection or resource-creation policy changes.

Pre-edit snapshot `2933`, request `f1eba78db3e94c78b1c5ada5d31839c0`, and
attribution `68584964604248b79cdcffbf09e42923` bind the two-file starting point.
Both edited paths passed read-only Rust 2021 rustfmt parsing and scoped diff
checks. The post-edit snapshot submission returned `offline / descriptor_absent`
without a request or snapshot identity. It was not retried. Exact current hashes:

- `graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/native.rs`:
  `2381b08329430d32dbd3ac85560d55d242a291f811fa5993d48c0e54d72a459b`.
- `ui/surface/surface/default_interactions/table/selection.rs`:
  `1667d16940744c7163cfa0eb1a7e2c41766e25cf69ae15fcfe5cd373d23ff56a`.

These paths are relative to `zircon_runtime/src`. The existing review task was
asked to inspect only this pre-snapshot/current-hash diff, without polling the
coordinator or claiming a frozen post-snapshot. Source and record leases were
acquired by request `a5198316dc374f68a45d87ec44a4ccd5`; their release and the
post-edit snapshot remain pending coordinator availability. No dynamic pass,
canonical fixed return, commit, or WeCom result exists for this batch.

## Remaining Independent Owner Boundaries

The two Python structure failures reported with the `2914` review are already
recorded in the existing
[Text04 retained-raster failure](../../text/04/failure-2026-08-31-retained-swash-native-scale-bypasses-physical-raster.md).
They remain separate from the reviewed five-file diff.

Readiness migration requires the existing
[09D residency owner](../../../optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md)
and [09C shader/material owner](../../../optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md):

- Residency's seed and ticket still contain scalar generation/dependency fields;
  issuance, pending/active transitions, device recovery, and ticket tests must
  retain and compare actual publication identities throughout their lifetime.
- Prepared shader, material dependency, rejected candidate, and `PipelineKey`
  still carry scalar dependency revisions. Their producer tuples and comparison
  helpers must migrate together, with unchanged-row reuse, changed transitive
  dependencies, removal/re-add, and separate-resource-manager regressions.
- `PipelineKey` is process-local, while `shader_variant_key()` and disk identities
  are content-based. Retained resource identity must not leak into persistent
  shader variant serialization. The existing separation regression must remain.

This is diagnosed source work, not implemented migration. Separate owner-scoped
handoff registration and managed validation remain pending; the shared resource
accessor restoration stays withdrawn.

## 2026-09-07 Source Evidence Reception

Independent review of pre-snapshot `2933` and the two post-edit hashes above
completed with Critical 0, Important 0, Moderate 0. At result reception the
coordinator accepted source snapshot `2934`, request
`394e5a31b12f403d8fe6c4d00f1fd721`; its hashes match the independently reviewed
contents. This supersedes the pending post-snapshot state above without replacing
the recorded transient offline result or claiming dynamic validation.

The original failure's two non-Cargo suites then executed together:

`python -B -m unittest tools.tests.test_frameworks_01_shader_invocation_owner_boundary tools.tests.runtime_text_infrastructure_compile_contract.context_geometry -v`

Result: 24 tests passed in 17.396 seconds. Repository handoff validation examined
787 artifacts and reported 74 existing schema/placement/link errors. This failure
record was not among them. The global artifact gate remains failing; these local
results do not replace the managed Editor/Runtime compilation and focused Rust
tests required to return this lifecycle.

## 2026-09-08 graphics production check and test compiler repair

The Runtime09D record retains the terminal target-client production check
`14ab8efb2c8d40b08a1f7f4c882e2a1e`: sealed input
`151f3a6ca148c08305ea907a7b9e828abf77f4753d2e061fe24bd648e53696a2`,
9 compiler errors, Cargo exit 101 and no tests. Four UI/Text diagnostics have
existing current-source repairs requiring their complete support closure; three
VM/host diagnostics remain outside the user's scope. Geometry replay and the
Render11 environment-frame implementation were missing from that input.

The derived input
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime-graphics-owner-support-20260908`
seals 10,955 files, digest
`b88423534de93715bc6f36a6db86d2426850966c6c6d8f8c2056c4c026f9142f`.
Its 18-path overlay adds the complete reviewed Render11 snapshot 2879, its
current capture import from 2905, the two 09C test corrections in 3017, and
read-only geometry replay snapshot 3018. `support-provenance.json` records each
owner and exact hash; geometry replay authorship remains pending. No foreign
support path is attributed to Frameworks01.

Managed Windows job `3e61161de7f8480e906067c0a37784dd` passed the production
library check using `-Package zircon_runtime -Features graphics
-NoDefaultFeatures -SkipTest -CheckOnly -LinkMode static`, with that exact
source path/digest and Cargo `--locked`. Operational Session:
`validate-matrix:failure-roll-01a07160-frameworks01`. Entry and receipt exit 0;
queue 6.910 seconds, sync 21.913 seconds, check 102.478 seconds. Log and embedded
receipt: `results/graphics-production-check.log`. Post-run manifest verification
passed. This feature set does not enable script/VM; it is lower graphics
compilation evidence, not the original target-client or Editor acceptance.

On the same input, Render07's managed job `1131b404d33449d899409926a2a7831a`
requested the original SSAO Naga library test. Test compilation reported 180
errors, Cargo exit 101; entry/receipt exit 1, no linked test and zero executed
tests. Log: `results/ssao-naga-graphics-library.log`; the adjacent structured
diagnostics record groups the first source location and existing owner for 179
diagnostics. Queue 19.876 seconds, sync 20.749 seconds, check 220.260 seconds;
the source manifest remained unchanged. The other diagnostics retain their
owners and are not hidden by dropping tests or restoring retired interfaces.

Ten diagnostics belong to three unchanged Frameworks01-owned source paths.
Snapshot 3022, request `5149f0d835a24ebaa6ac95ec0a5fdcf2`, repairs only their
test code: resolver tests import `RenderGraphResource`, `BufferDesc` and
`BufferUsage`; the recorded-pass fixture initializes the current
`native_resource_creates` field; buffer-range error assertions still require
the same error messages but no longer impose `Debug` on the successful private
binding type. The pre-edit hashes matched coordinator attribution and the
failed input. Lease `43c27dda2d08466d8df549bc1ab5d1cd` and attribution
`1651c849f3074e818d6f91dcf824a004` retain this Session's ownership. Rustfmt
passed; actual test execution and incremental independent review remain pending.

The read-only increment review, bound to source `3022` and record `3023`, was
queued once to `优化协调器验证效率` (`01a07063-6f03-7803-a12d-13ea015ca645`) in
message `01a07ec5-9d07-7010-8feb-f2c2ce5742f7`. Its requested conclusion is
separate from the other source owners in that dispatch. No review result or
dynamic pass for source `3022` is claimed.

## 2026-09-08 Additional Test Contract Migrations

The same managed graphics library diagnostics identify five further test callers
that still use earlier shader, GPU-factory, execution-batch or diagnostic-admission
contracts. The fixes stay in existing test bodies and keep the original rejection,
last-good and ownership assertions:

- Source `3031`, request `6acfb5274b85434da3c66d2de6ab979e`, migrates the IBL
  shader tests to `graphics::shader::invocation` and renames a local SH9 request
  that shadowed the PMREM request factory. Exact paths:
  `zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs`
  (`0fa44ada75a45183efa711f40dfcac6195836ef264228106a3606821920eaf4b`) and
  `zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan/tests.rs`
  (`951c97b803acb83fa2061166dc351636a3f8bc51bfeeeafa67ba38c78c6f7c28`).
- Source `3033`, request `4b7909fa39034364883185d831c702fc`, supplies the existing
  WGPU device factory to compute-cache tests, derives the expected batch from the
  compiled pipeline for pass-coverage tests, and wraps tracker-issued request IDs
  in `DiagnosticReadbackAdmission::Admitted` when exercising the router boundary.
  The full-router test still proves rejection preserves the first callback owner.
  Exact paths and hashes:
  `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/compute_pipeline_cache/tests.rs`
  (`5257a41fcc36dd60c2362f4b24cbf38a1079f4218edb8fcc4b859840fe707440`),
  `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages_tests.rs`
  (`0e37541b0b5d55607024a39c190fae859d73b0c611ed1459cbab6ea6845d4e2c`), and
  `zircon_runtime/src/graphics/backend/render_backend/product_diagnostic_delivery_router.rs`
  (`3962c09d7f2e0783b33eefd176eb84668d528dca7d66cfe67cda8416e3bfcfbf`).

Coordinator transfer receipts `2aaf5a99c4f34be39040cc4ba9ed7037` and
`0756a06ddbc342a08e2cb2bf3d6ed244` extend this same fixing Session's exact scope.
Four pre-edit files matched HEAD byte-for-byte. For the pass-coverage file,
formatting HEAD with Rust 2021 rustfmt reproduced the entire pre-edit content
(`785837e6c208e517e109135a8a966027a22e1d102ffe5604d0e1b9a1dbfe0938`), proving the
existing difference was formatting only. The new changes preserve that layout.

All five edited files pass scoped whitespace checks and Rust 2021 parsing/format
checks. The compute-cache and router files retain their existing 2024 formatting
style with `style_edition=2024,skip_children=true`; no child test module is edited
by the formatter. No new production API, compatibility export or test exemption
was added. Incremental review and managed dynamic acceptance remain pending for
sources `3031` and `3033`.

### Managed Library Recheck

Managed Windows job `b0612b75cf6b4a4dada9e3d89b53dbc6` compiled the exact 13-file
overlay from snapshots `3022`, `3026`, `3031`, `3032` and `3033` on the prior
sealed graphics input. The resulting 10,955-file input is
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime-graphics-test-support-3033-20260908`,
manifest `1adec2abe9c4763f4b8e02163a57b8d752481555cf35fc893e43383772015761`.
Each overlay retains its original fixing owner in `support-provenance.json`.

The command was `tools/dev/dev-fast-build.ps1 -Action test -Package zircon_runtime
-FeatureOverride graphics -LibTests
-TestFilter ssao_evaluate_spatial_and_upsample_shaders_are_valid_wgsl
-LinkMode static -Linker auto`, with the exact snapshot path and digest. Cargo
retained `--locked`; no VM, scripting or UI feature was enabled. The prerequisite
check returned Cargo 101 / wrapper 1: 126 compiler errors, zero tests. The same
configuration previously reported 180 errors. All 54 diagnosed errors from these
13 edited files disappeared, with no remaining diagnostic in any edited file.
This is measured compiler progress, not a library or test pass.

Queue 6.156 seconds, input sync 22.364 seconds, check 210.044 seconds, no link
or test execution. All 354 dependency packages were verified and none repaired;
the complete manifest was verified again after execution. Durable receipts and
the 125 source-located diagnostics are in `results/graphics-library-recheck.json`
and `results/graphics-library-compiler-diagnostics.json`. The remaining global
compiler summary is the 126th error. Continue independent owner fixes before
repeating this batch; original Editor/target-client acceptance remains pending.

Review supplement `01a07eed-55b6-77c2-8c6c-d2a9852a4a0e` was queued once to the
same user-selected task for sources `3031`/`3033` and the new managed evidence.
It does not replace the pending earlier `3022` review.

Source `3036` (request `c59f57accc7a48eb856b7b2e164ff516`) addresses five further
diagnostics: the existing morph history regressions import their production
rollover helper, and the two offscreen-target regressions retain their typed
error assertions without a `Debug` bound on the success resource. Pre-edit files
matched HEAD; transfer `30549930b100487788fe5b932d750f23` and attribution
`cc27481575f8432baeec5a8c58566762` preserve this fixing Session. Current hashes:
`prev_morph_weights.rs` = `d41c62ce1ff2fd60085eef2d69b5e5ccb26365c9ed6b4a7c130da958f2ccc63f`;
`required_offscreen_target.rs` = `b042be8b1c99fd56011334e40e44d084e53a33f370fcdd609656ead3cccd25d2`.
Scoped rustfmt and whitespace checks pass. The `3036` increment is not covered
by the preceding managed job or review dispatch; its compilation and review
remain pending. Unattributed production/test changes in component-projector,
mesh-bounds and GPUScene sync files were inspected but not modified or absorbed.

### Source 3036 Compilation Result

Managed Windows job `b18ea987bdfc49cd8c5c0799dbc9a46d` compiled the exact two
`3036` files and the separately owned Render11 `3037` fixture. The 10,955-file
input `E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime-graphics-test-support-3037-20260908`
has manifest `c0d346a32524a0f477d3e98178f52834f25db67babf834d5466c83db27231813`.
It differs from the preceding input only at those three paths, with original
owner/snapshot provenance retained outside the compiler inputs.

The unchanged graphics-only static locked library-test command reports 115
compiler errors, down from 126. All 11 diagnosed errors in the three new paths
disappeared; no diagnostic remains in them. Cargo returned 101 / wrapper 1 and
zero tests ran. Queue/sync/check times were 6.636/28.213/283.327 seconds; no link
or test phase ran. All 354 dependencies and the complete post-run manifest were
verified. `results/graphics-library-recheck.json` retains the managed receipt;
`results/graphics-library-compiler-diagnostics.json` records 114 source-located
error blocks and the rustc summary. This is compiler progress only. Original
Editor/target-client acceptance and independent source review remain pending.

## Source 3045 And 3048 Test Consumer Corrections

Source `3045`, request `0037186fc3a84d6195f19ae19d3c9bab`, freezes seven
HEAD-proven test-consumer corrections. They supply the missing performance
const generic, retain the shared scene before moving its frame into a closure,
construct particle history through its current API, import the canonical
lighting profile and shader invocation, format the typed geometry ID in a
failure message, and compare all thirteen visual-export counters as arrays.
The existing performance scale, allocation/identity checks and visual counters
are retained. Transfer `52f9c0a105b3411499b0090bd9517e7f` preserves this
Session's exact seven-path scope.

Managed Windows job `c65a3313efeb4c13b6220485b58c9561` compiled that overlay
on source `3037`. Input `runtime-graphics-test-support-3045-20260908` under
`E:/cargo-targets/zircon-engine/cache/build-benchmarks` contains 10,955 files,
manifest `d37438f85f2834dd6fd3e899b9f1abd3a3865299e6e6e7319919f2ff60f9d3b0`.
Rustc reported 104 errors, previously 115; no diagnostic remains in those seven
files. Ten source-located diagnostic blocks disappeared. Rustc's reported
total and grouped diagnostic-block counts are distinct measures. Cargo 101 /
wrapper 1, zero tests; queue/sync/check 13.526/25.438/209.560 seconds. The
complete post-run input and 354 dependency packages were verified.

Source `3048`, request `017c8f85dfb8494bb999bfdad23038f4`, adds three
HEAD-proven consumer fixes: `compile_tests.rs` uses `access.access`, the
performance fixture obtains the encoder through `require_gpu().native_context()`,
and the anti-alias fixture expects `Some(PostProcess)`. These address five
diagnostics without a new production API or changed assertions. Transfer
`3a56f4084d4844c6830043518cf401e8` preserves this Session and extends its scope.

## Independent Review And Source 3050

The existing user-selected review task returned Critical 0, Important 0,
Moderate 1 for Frameworks01 sources `3022`, `3031`, `3033`, `3036`. Exact
hashes matched before and after review and no reviewer-owned conflict existed.
Report: `.codex/tmp/graphics-support-review-resume-20260908-result.txt`.
The duplicate-pass test expected the coverage-array error, but the lower
execution cursor correctly rejects the out-of-order pass first.

Source `3050`, request `9d59b2c5dc4342c5ab883f41c10a7250`, updates that
expectation to the actual pass-1/pass-0 error and then admits the next legal
pass, proving rejected admissions did not advance the cursor. Missing and
out-of-range checks remain. Its stage-test hash is
`3611748ced95d543a54dda00d56e86816da1c20664f6aaec38a60ba00b74a024`.
The edit used the preceding live lease; after expiry, lease
`cf238aafb064433580a0616934c3aeba` and attribution
`292374cb72d848dd85791b9f856660c5` refreshed the unchanged frozen bytes.

Managed job `3a8034c59d4546a39836f6bde23c5ce6` compiled sources `3048`,
`3050`, Render07 `3049` and Render11 `3051` on the prior sealed input.
Input `runtime-graphics-test-support-3051-20260908`, 10,955 files, digest
`bc81d18f821d3f151fc4919286259e9b1e02e953d5a220aca06d81cd4451ab9b`,
retains each owner in `support-provenance.json`. The Windows static locked
graphics library-test command used filter `graphics::` and linker `auto`.
It reported 95 compiler errors, down from 104; all nine newly covered files
are diagnostic-free. Cargo 101 / wrapper 1, zero tests. Queue/sync/check were
6.912/22.386/214.729 seconds. The complete input was verified after execution.
`results/graphics-library-managed-recheck.{json,log}` retains the receipt.
The earlier `graphics-library-recheck.log` contains only a rejected `msvc`
wrapper argument, with no job or test execution; it is not compiler evidence.

## Source 3052 Indirect Upload Test Migration

The two remaining HZB/indirect-execution tests called a now-private phase
workspace with the retired queue-writing signature. Both pre-edit files
matched HEAD. Audited transfer `b4f251dfec974a4c8a9f821d642f612c` and lease
`7f20868215fd445f9e295715e31bccff` bind this increment to Frameworks01.
Source `3052`, request `2e833bb04d0149a38f2a639a9d425e8b`, freezes:

- `hzb/hzb_occlusion_culler/tests.rs`:
  `661eff407b79fc8ea3685dc11f4a62851d73e5626a977f1748555a59919febf1`.
- `mesh/mesh_pass/indirect_draw_execution.rs`:
  `f42fe05693358bfc76b215980c60795819829b6ab2a90b6fda20baeb0eb55391`.

Both fixtures use current command-list partitioning and per-pass indirect
workspace preparation. They append the prepared uploads, enqueue through the
backend, submit the graphics batch and only then commit the CPU shadows. The
HZB fixture also includes and commits its parameter uploads. Its original
hidden/visible counts and GPU readback assertions remain intact. Scoped
rustfmt and whitespace checks pass; dynamic execution and fresh review remain
pending. The derived immutable input `runtime-graphics-test-support-3052-20260908`
has digest `6710407b6b37fa9e7fdf06aebcddab341860e0b68357b7559a914fc942f7fdf7`;
only these two paths differ from the `3051` input. No accepted closeout is claimed.

## Review Result And Source 3060

The user-selected task `优化协调器验证效率` completed the next independent
review with Critical 0, Important 0, Moderate 0 for sources `3045`, `3048`,
`3050` and `3052`. Report:
`.codex/tmp/graphics-3052-review-resume-20260908-result.txt`.
It verified source/ObjectStore hashes before and after review and found no
reviewer ownership conflict. The duplicate-pass regression preserves cursor
state, and the HZB/indirect fixtures preserve production upload/submission/
CPU-shadow order and the original readback assertions. This supersedes the
fresh-review-pending statement for these four source increments only.

Managed job `3db36b30be88485bb06b0a5ff2c6d3e4` compiled the exact `3052` input
above with 90 errors and zero tests. Source `3060`, request
`41a05f2955cf40c7a505fdd7bbf5fcdb`, subsequently repairs two more test consumers:

- `mesh/mesh_pass/replay.rs` imports the canonical GPUScene bind-group slot;
  hash `65abeaf960f6b84de3a695a97cd4cf331467ef5588033d1392afdc1dfa4d3fb1`.
- `mesh/mesh_pass/mesh_draw_command_list/tests/cache.rs` retains the changed
  command buffers and stores their statistics separately, preserving the
  command-buffer identity assertion and all six statistics checks;
  hash `c50a6ea0e3641c2d86e27cebb128eb512d2208b24cfe2cdee8cf2a336f5e99a7`.

Pre-edit snapshot `3059` retains the exact archived-owner attribution from
`01a019a3-02af-7012-84cd-f8d605f12c4d-r1`, baseline 587. These two files were
not HEAD-identical: the existing profile scope and resolver-configuration
regression remain intact. Transfer `f024f4a03be24b95a5335a192d4effa2` records
the explicit provenance. Scoped formatting and whitespace checks pass.

Managed Windows job `94560adfdb1a45daa7e2d5785ae6677c` compiled these files
and separately owned Text increments in immutable input
`runtime-graphics-text-test-support-3067-20260908`, digest
`c93b37d1c23413b5f1ff16f4705abfbf56ec2511dfb45f89a1b1c85500a839da`.
The 10,955-file manifest and 354 dependencies were verified after execution.
Rustc reported 67 errors, with no diagnostic in the two `3060` paths; Cargo
101 / wrapper 1, zero tests. The durable receipt is
`results/graphics-library-recheck.json`. Source `3060` still needs independent
review. Actual WGPU/readback, the original Editor-target gate and formal
fixing-Session evidence binding remain pending; this failure stays open.

### Source 3060 Review Result

The existing task `优化协调器验证效率` completed source `3060` review with
Critical 0, Important 0, Moderate 0. Report:
`.codex/tmp/text-framework-3080-review-20260908-result.txt`.
Both source hashes matched their snapshot, ObjectStore and attribution at
review start and finish. The archived profile scope and 53-line resolver
regression remain intact, including command identity, variant invalidation,
stable-frame cache hits and statistic reset. The reviewer acquired no source
ownership. This resolves source review only; actual WGPU/readback, original
Editor acceptance and formal evidence binding remain pending.
