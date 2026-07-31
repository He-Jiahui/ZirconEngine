---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: woc-gltf-meshopt-webp-import
origin_plan: docs/plans/woc/00-woc-engine-capability-foundation.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/woc/00
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/asset/importer/ingest/gltf_decode.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_meshopt.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets/material.rs
  - dev/world-of-claudecraft/public/models
tests:
  - cargo test -p zircon_runtime woc_meshopt_required_glb_imports --locked
  - cargo test -p zircon_runtime woc_required_gltf_extensions_preserve_render_assets --locked
---

# Runtime 04: WOC required glTF compression and texture extensions do not import

## 来源执行者

- 来源计划：`docs/plans/woc/00-woc-engine-capability-foundation.md`
- 来源执行切片：WOC engine capability assessment / MVP foundation
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：The lowest shared cause is the Runtime-owned asset importer and artifact graph, before graphics consumption or WOC gameplay code.

## 失败现象与复现证据

The original failure snapshot contained 938 GLB files. The 2026-08-01 implementation audit now
walks all 949 checked-in GLBs and found:

- 869 files require `EXT_meshopt_compression`;
- 803 files require `EXT_texture_webp`;
- 868 files require `KHR_mesh_quantization`;
- all 938 contain animation and skin data, totaling 1,581 animations and 1,043 skins;
- 13 use `KHR_materials_unlit`, four use `KHR_materials_emissive_strength`, and individual files use IOR/specular extensions.

Zircon declares `gltf = 1.4.1` with only `KHR_texture_transform` and raw `extensions`. There is no meshopt decoder dependency or `EXT_meshopt_compression` consumer. `import_gltf.rs` calls `gltf::import` directly, while material projection consumes the core metallic/roughness model and texture transforms but not the target material extensions.

This means at least 869 target models cannot be treated as directly importable engine assets. Representative input: `dev/world-of-claudecraft/public/models/biome/beach_anchor.glb`.

## 最低共享层根因

The asset ingest layer assumes accessor buffer views are directly readable and does not decode required meshopt-compressed buffer views. It also does not project `EXT_texture_webp` image selection or the exact material extension set into Zircon texture/material assets. The problem precedes renderer resource preparation.

### 2026-07-22 性能审查补充

当前`import_gltf`先从已读`context.source_bytes`解析一次只为检查external buffers，随后`gltf::import(source_path)`重新打开并解析同一源；root model、labeled model和Mesh subassets又复制vertices/indices/VG/morph/skin，依赖用Vec `contains`逐项去重，animation/skin为每项重复构造parent/target扫描。PERF-MVP-504要求本failure实现meshopt/WebP/material扩展时同时采用Runtime04 content reader、单次document/buffer/image decode、单遍parent/skin/dependency indices与共享immutable cooked payload，不得在增加扩展支持时保留2× parse/I/O或按subasset复制完整buffer。

## 架构修复验收

- Import all 938 WOC GLBs from their original checked-in bytes without a WOC preprocessing step or decompressed duplicate corpus.
- Decode required meshopt buffer views with validated bounds and deterministic artifact output; preserve quantized attributes correctly.
- Resolve `EXT_texture_webp` texture sources through the existing texture asset pipeline.
- Project the target material extension set into render-ready material semantics, with explicit diagnostics for any intentionally unsupported extension.
- Preserve animation clips, skeletons, skins, primitive materials, textures, and dependency edges through artifact-cache round trips.
- Parse/read each source content generation at most once; share decoded/cooked mesh and image payloads across root/labeled subassets, with graph work near O(V+E) and bounded worker/RSS budgets.
- Add target-derived focused fixtures and run the existing glTF importer, artifact, scene, animation, and renderer-data upward suites.

## 禁止临时方案

- Do not commit decompressed/re-encoded WOC assets as a substitute for importer support.
- Do not silently ignore required glTF extensions or replace affected models with primitives.
- Do not special-case WOC filenames in the importer.
- Do not weaken import diagnostics, dependency checks, or visual acceptance.

## 修复结果与回传

Open state: `实现完成，受管验证待回执`; no pass or fixed return is claimed yet.

### 2026-08-01 实现状态

已完成：

- added a Runtime-owned, single-read glTF decode path with explicit required-extension validation;
- added validated `EXT_meshopt_compression` buffer-view decode for ATTRIBUTES, TRIANGLES and
  INDICES modes plus NONE/OCTAHEDRAL/QUATERNION/EXPONENTIAL/COLOR filters;
- routed embedded and external `EXT_texture_webp` sources into the existing RGBA texture asset
  pipeline without a WOC filename special case;
- projected unlit, emissive-strength, IOR, transmission and volume semantics, while retaining an
  explicit diagnostic for unsupported specular fields;
- replaced repeated animation/skin parent scans and dependency `Vec::contains` walks with one
  generation-level hierarchy index and insertion-order-preserving membership sets;
- hard-cut root and `MeshN` model payloads to reference-only primitives, leaving one cooked
  `MeshAsset` owner for vertices, indices, morph targets, skin and virtual geometry;
- added focused WOC extension/material/artifact tests, an all-current-corpus import gate and
  migrated existing glTF channel assertions to the canonical mesh subasset owner;
- completed a second source review and `rustfmt` / `git diff --check` static gate.

待完成：

- generate the managed lockfile update for the new `meshopt` dependency;
- receive source-bound Windows Cargo evidence for the focused and upward glTF importer suites;
- only after terminal green evidence, rename this record to `fixed-*`, update parent status and
  create the exact managed milestone commit.

受管验证 receipt（不等于通过）：

- source snapshot: `1398`;
- source manifest hash: `becf12695938e7f4ed359aab18ad9a97c4bec62b3fe6904b832c9c9a9414f9b4`;
- ticket: `a546d9acf24d468182a9f20d8c7f11d2`, status at submission: `queued`;
- Windows command: `cargo +1.94.1 test -p zircon_runtime --lib asset::tests::assets::gltf_importer --jobs 1 -- --nocapture --test-threads=1`;
- coverage: all glTF importer lib tests, artifact roundtrip and all 949 checked-in WOC GLBs;
- the Session did not poll or wait for queued/running validation.
