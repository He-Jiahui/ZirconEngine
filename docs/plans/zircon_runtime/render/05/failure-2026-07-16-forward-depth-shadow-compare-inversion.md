---
handoff_kind: failure
status: open
created_at: 2026-07-16
summary_slug: forward-depth-shadow-compare-inversion
origin_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
fixing_plan: docs/plans/zircon_runtime/render/05-lighting-shadows.md
origin_child_dir: docs/plans/zircon_runtime/render/18
fixing_child_dir: docs/plans/zircon_runtime/render/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs
tests:
  - managed cargo test -p zircon_runtime render_shadow_atlas_compare_function_matches_forward_depth_contract --locked
  - managed cargo test -p zircon_runtime advanced_lighting --locked
---

# Render05：Forward-depth shadow comparison inversion

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 来源执行切片：Render18 AF-M3 D3D12 replay / volumetric light-scatter ShadowAtlas consumer.
- 修复责任计划：`docs/plans/zircon_runtime/render/05-lighting-shadows.md`
- 交接原因：ShadowAtlas depth-clear and comparison-sampler ownership is Plan 05; Render18 only consumes the atlas.

## 失败现象与复现证据

D3D12 replay recorded a nonempty 4096 D32 atlas with min depth `0.538` and max depth `1.0`.
Shadow rendering uses forward depth (`LessEqual`) with a clear value of `1.0`, while the atlas comparison sampler is bound as `GreaterEqual`. A receiver evaluates `light_ndc.z - bias`; therefore a clear aperture sampled as `1.0` is incorrectly treated as occluded/dark.

The focused regression now requires `SHADOW_ATLAS_COMPARE_FUNCTION` to be `LessEqual`; against the pre-fix sampler value it must fail before implementation is changed.

## 最低共享层根因

`shadow/atlas/resources.rs` declared the comparison sampler as `GreaterEqual`, which inverts the forward-depth `LessEqual` / clear-`1.0` atlas convention shared by every receiver, including Render18 light scatter.

## 架构修复验收

- The focused Plan 05 shadow sampler contract test passes with `LessEqual` for the forward-depth/clear-`1.0` atlas.
- The managed original Render18 AF-M3 `advanced_lighting` gate is rerun after the focused test.
- No Render18 froxel, plugin, or Shader06 source path is changed; no threshold is weakened.

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- Do not change Render18 froxel/plugin or Shader06 source paths to compensate for the shared atlas contract.

## 修复结果与回传

Open state: `source repair and static fixture review complete; managed validation pending`.

- The shared forward-depth contract is `SHADOW_ATLAS_COMPARE_FUNCTION = LessEqual` with a clear value of `1.0`; the volumetric consumer imports that single constant instead of restating sampler polarity.
- The Render18 froxel fixture now writes an occluder depth and exercises a receiver exactly at that depth, proving the equality case remains visible under `LessEqual` while preserving the unshadowed comparison region.
- The scoped Rustfmt check passes and the focused diff check reports only repository CRLF warnings. The required managed Plan05 and Render18 gates, plus real product evidence, remain outstanding, so this handoff remains `open`.

### 2026-08-23 managed current-source validation

- Coordinator job `69a871b625ac4bc6b3f113fef45452c7` ran the declared focused `zircon_runtime` lib-test filter and released normally with exit code 1 after Cargo returned 101.
- Compilation stopped before the focused test executed. The first current-source errors are unrelated UI test debt: `ui/tests/asset_surface_index/binding_ownership_performance.rs:129` cannot resolve `TARGET_BINDING_COUNT`, followed by `ui/tests/text_pipeline/measure_cache.rs:653-654` failing to resolve `measure_line_width`.
- No diagnostic was attributed to `shadow/atlas/resources.rs`, but a compile-blocked test is not a pass. The Plan05 focused gate and Render18 upward gate therefore remain pending; this record stays `open` and the unrelated UI owners are not absorbed here.
