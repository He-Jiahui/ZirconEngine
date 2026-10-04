---
handoff_kind: failure
status: open
created_at: 2026-09-01
summary_slug: default-hdr-output-bypasses-tonemap-and-dither
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_runtime/render/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/render/frame_extract/post_process.rs
  - zircon_runtime/src/core/framework/render/post_process/color_space.rs
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings/color_transform_settings.rs
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings/style_settings.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/constants/texture_formats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_output_transfer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/output_transfer.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
tests:
  - Invoke-Pester -Script tools/tests/zircon_profile_shader_pbr_viewer.Tests.ps1 -PassThru
  - managed cargo test -p zircon_runtime --lib render_product_post_process --locked --jobs 1
  - managed zircon_shader_pbr_viewer current-source SDR PNG and RenderDoc acceptance lane
---

# Render07: default HDR output bypasses tone mapping and quantization dither

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行切片：Shader06 PBR/IBL current-source display correctness and pre-optimization review
- 修复责任计划：`docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md`
- 交接原因：the lowest shared cause is the engine-wide linear-HDR to display-output contract,
  not environment filtering, material shading, or the Shader PBR viewer fixture.

## 失败现象与复现证据

The current source has a complete linear HDR scene-color path, but its default SDR display path does
not guarantee a display mapping before quantization:

- `RenderTonemapSettings::default()` selects `RenderTonemapOperator::None`, and
  `PostProcessExtract::default()` propagates that effect stack to every camera without a volume.
- `post_process.wgsl::apply_tonemap_and_lut` applies exposure, but operator `0` leaves positive HDR
  values unbounded. The pass then writes them to `POST_PROCESS_TONEMAPPED_FORMAT`.
- `TONEMAPPED_SDR_FORMAT` is `Rgba8Unorm`. This quantizes display-linear values to eight bits before
  the named output-transfer pass and clips every channel above one.
- `output_transfer.wgsl` only copies the sampled texel. It receives no `RenderOutputTransfer`, target
  luminance, bit depth, or dither state; `SrgbNonlinear`, `LinearExtended`, and `Hdr10Pq` are therefore
  declarations without an executing transfer contract in this pass.
- `apply_grain_and_dither` executes before FXAA, tone mapping, LUT, and color grading. Its hash uses a
  per-pixel `sin`, and its noise is transformed by later operators instead of being applied once at
  the quantization boundary.

Historical Shader06 artifacts reproduce the visible symptom. A read-only 32-bit pixel scan found:

| Artifact | Pixels | Any channel exactly 255 | All RGB exactly 255 |
|---|---:|---:|---:|
| `runtime_shader_pbr_real_hdri_lakes_ambientcg_metal009_texture_maps_20260823_r3.png` | 1,228,800 | 519,155 (42.249%) | 44,404 (3.614%) |
| `runtime_shader_pbr_interactive_viewer_current_source_20260715.png` | 1,294,704 | 70,910 (5.477%) | 41,092 (3.174%) |
| `zircon_shader_pbr_viewer_m5_environment_only_pbr_20260729_dx12.png` | 1,228,800 | 35,559 (2.894%) | 22,203 (1.807%) |

These are historical symptom measurements, not current-source visual acceptance. The newest listed
artifact also has 513,741 pixels with blue exactly 255, consistent with the visibly clipped cyan sky.
Current-source PNG, HDR readback, RenderDoc, timing, and power evidence remain required.

Changing only the default operator is not an acceptable fix. `RenderPostProcessEffectStackSettings`
uses one aggregate `is_enabled()` bit. Enabling only the default tonemap makes the uber shader enter
the shared effect branch: `apply_effect_blur_family` performs one depth read, one center CoC read, and
five more CoC neighborhood reads before discovering a zero radius; `apply_scene_composite` performs
another resolved-SSR read. That is a static lower bound of eight unrelated texture reads per shaded
pixel: 16,588,800 reads at 1920x1080 and 66,355,200 reads at 3840x2160. These are source-structure
counts, not measured GPU transactions or elapsed time.

Unreal keeps pre-exposed scene color, exposure resolution, tone/output-device mapping, and
back-buffer quantization dither in the terminal mapping owner. Its tonemap shader applies
`OneOverPreExposure * GlobalExposure`, resolves the output device, and adds back-buffer dither after
the final display mapping. cmftStudio likewise resolves luminance/adaptation and a selected tone
curve before display grading; its image filter identity remains separate from display mapping.

## 最低共享层根因

Render07 currently treats tone mapping as an optional stylistic effect while an SDR output is a
mandatory finite-range conversion. It also assigns precision and dither to the pre-transfer uber
pass, then names a later copy as the output transfer. The data model, graph node, texture format, and
shader execution owners therefore disagree about where scene-referred HDR becomes output-referred
SDR and where the only eight-bit quantization occurs.

## 架构修复验收

- Publish one typed display-mapping contract derived from camera/output target. Every SDR target must
  select a bounded tone curve; `None` must not silently mean unbounded HDR written to UNORM.
- Preserve scene-referred linear HDR, including `capture_scene_color_hdr`, before display mapping.
  Do not lower environment intensity or destructively normalize PMREM/source cubemaps.
- Keep the post-tonemap/display-linear chain in a float format through terminal AA and upscale, or
  prove an equivalent single-quantization layout. Do not quantize into linear `Rgba8Unorm` and then
  perform a second output-transfer pass.
- Make `RenderOutputTransfer` executable: bind the transfer/output-device parameters and implement
  the selected sRGB, linear-extended, or HDR10-PQ transfer in the terminal owner.
- Apply target-bit-depth-aware quantization dither exactly once after tone mapping, LUT/grading, AA,
  and upscale. Use a bounded blue-noise/interleaved-gradient owner rather than per-pixel transcendental
  hash work, with deterministic capture and temporal production modes.
- Split effect execution flags or graph entry points so mandatory tone mapping does not execute DoF,
  SSR composite, motion blur, fog, vignette, grain, or their texture reads when those effects are off.
- RED/GREEN must cover HDR values above one, SDR shoulder retention, dark gradient banding, exact
  transfer selection, deterministic capture dither, HDR readback invariance, and zero unrelated
  texture reads for the default tone-only path.
- Before implementation tuning, capture current-source 1080p and 4K RenderDoc/resource evidence plus
  GPU timestamp p50/p95/p99, target allocation/format bytes, texture-read and pass counts, bind-group
  creation counts, RSS/VRAM, and WPR/WPA CPU-package/GPU energy. After evidence must show one display
  mapping/quantization owner and no default-path regression toward the static eight-read bound.
- Rerun the Shader06 current-source HDRI matrix and require unclipped highlight structure, no visible
  gradient bands, material/reflection agreement, and source-bound PNG/RDC/profile provenance.

## 禁止临时方案

- Do not lower the viewer HDRI intensity, clamp source cubemap/PMREM values, or post-process PNG files.
- Do not enable ACES only in `zircon_shader_pbr_viewer`; project cameras without volumes must remain
  correct under the same engine display contract.
- Do not change the default tonemap without first removing the aggregate-effect execution penalty.
- Do not retain the early linear eight-bit intermediate as a compatibility path.
- Do not add a second dither or gamma pass, duplicate output-transfer truth, weaken image or HDR
  assertions, or claim historical screenshots as current-source acceptance.

## 修复结果与回传

Open state: `pending Render07 architectural repair`; no current-source visual, performance, or power
pass is claimed. Shader06 continues only dependency-independent work.
