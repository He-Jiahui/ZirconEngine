---
title: Runtime Terrain / Landscape / Heightfield / World Partition 当前工作树复审
category: zircon_runtime
report_id: Runtime172
review_date: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/99zq-runtime-terrain-landscape-heightfield-clipmap-quadtree-lod-material-layer-virtual-texture-foliage-world-partition-physics-navigation-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/29-terrain-landscape-heightfield-quadtree-lod-material-layer-foliage-world-partition-physics-navigation-scalability-product-integration-review.md
related_code:
  - zircon_plugins/terrain/plugin.toml
  - zircon_plugins/terrain/runtime/src/plugin.rs
  - zircon_plugins/terrain/runtime/src/capability.rs
  - zircon_plugins/terrain/dist/src/lib.rs
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene/entity.rs
  - zircon_runtime/src/asset/assets/scene/extensions.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_authoring_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/advanced_slots.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/geometry.rs
  - zircon_plugins/physics/runtime/src/backend/jolt/conversion.rs
  - zircon_plugins/physics/runtime/src/backend/jolt/mesh_shape.rs
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_terrain_editor_workspace.zui
  - examples/vampire
tests:
  - zircon_plugins/terrain/runtime/src/tests.rs
  - zircon_runtime/src/asset/tests/assets/authoring.rs
  - zircon_runtime/src/asset/tests/project/example_vampire/manifest_scene_imports.rs
  - zircon_runtime/src/scene/tests/asset_scene
  - zircon_plugins/navigation/runtime/src/manager/bake
  - zircon_plugins/physics/runtime/src/backend/tests/jolt_contract.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/99zq-runtime-terrain-landscape-heightfield-clipmap-quadtree-lod-material-layer-virtual-texture-foliage-world-partition-physics-navigation-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape
  - dev/UnrealEngine/Engine/Source/Runtime/Foliage
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/WorldPartition
  - dev/UnrealEngine/Engine/Source/Editor/LandscapeEditor
  - dev/Fyrox/fyrox-impl/src/scene/terrain
  - dev/Fyrox/editor/src/interaction/terrain.rs
  - dev/Fyrox/editor/src/scene/commands/terrain.rs
  - dev/godot/scene/resources/3d/height_map_shape_3d.cpp
  - dev/godot/servers/rendering/renderer_rd/storage_rd/particles_storage.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/TerrainLit
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/UnifiedRayTracing/Common/TerrainToMesh.cs
  - dev/bevy/crates/bevy_render/src/view/visibility
  - dev/bevy/crates/bevy_render/src/batching/gpu_preprocessing.rs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime Terrain / Landscape / Heightfield / World Partition 当前源码工程化差距

## 1. 结论

当前 Zircon 有 Terrain 的数据载体，但没有可执行的 Terrain/Landscape runtime。`TerrainAsset`、`TerrainLayerStackAsset`、TOML importer、artifact store/load、`SceneTerrainAsset`、Physics `HeightField` DTO、`BuiltinRenderFeature::Terrain` 和 terrain package manifest 都是真实的局部基础；它们不能证明 World 中存在 Terrain instance、patch geometry、terrain material、collision/nav surface、foliage 或大世界 streaming。

最重要的事实仍未改变：Terrain plugin runtime 在 `register` 中使用 `DiagnosticOnlyAssetImporter`；`World::from_scene_asset` 没有消费 `entity.terrain`，`World::to_scene_asset` 在 `scene_asset.rs:612` 固定写 `terrain: None`；Terrain 仅位于 `DESCRIPTOR_ONLY_ADVANCED_SLOTS`，没有 extract、pass 或 renderer consumer。导航 bake 对 `TriangleMesh | HeightField` 直接注释为由 owning asset path 处理，但当前工作树没有这个 owner；Jolt heightfield 只走独立 mesh conversion，不绑定 `TerrainAsset` generation。

因此普通 mesh 地面、可解析的 TOML、asset reference round-trip 测试、descriptor registration 或 Workbench 的 cell 数字都不能算 Terrain runtime 通过。Vampire 的可见地面仍来自普通 baked mesh；同一实体上的 terrain reference 不是 source-to-pixel、source-to-collision 或 source-to-nav 证据。

本轮是对 Runtime142/99zq 的 current-source refresh，不新增 P0；既有 terrain package/catalog、Editor Workbench 伪产品和 World save 丢失等 P0 继续由先前报告计数。本轮整理 **24 项 P1、8 项 P2、18 项资格门**，当前裁决为 **16 Fail / 2 Partial / 0 Pass**。在这些门关闭前，任何将 Terrain 标记为 Complete、required、默认启用或声称超过 Unreal Landscape 的 profile 都必须 fail-close。

## 2. 审查边界与冻结统计

本轮按 `source/import -> artifact/reference -> scene load/save -> component/world -> render feature -> physics -> navigation -> foliage/partition -> catalog/product` 顺序逐层阅读，并以 Unreal Landscape/Foliage/WorldPartition、Fyrox Terrain、Godot HeightMap、Unity HDRP Terrain/GPUDriven 与 Bevy visibility/batching 为职责对照。没有运行 Cargo、native plugin、asset cook、GPU capture、Jolt/Recast 场景、streaming、fault、soak 或 benchmark。

| 范围 | files | lines | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---|
| `zircon_plugins/terrain` 全量（源码/manifest/README） | 16 | 968 | 38,580 | 10 | 1 | `e59fb4861fd39b49058175fc4d4d9cdd162febd67095fa0aa2cc0262fb219639` |
| Runtime asset/Scene/Render/Nav/Physics 选定证据 | 8 | 2,126 | 87,174 | 2 | 0 | `d671c3fb0baedd1394ffdf4728ba1ee04ec7ad3075e22f4f9580eefc2645e691` |

三份 plugin 资源 `zircon_plugins/terrain/editor/authoring.zui`、`terrain_component.zui`、`templates/default_heightfield.toml` 当前均不存在。指纹按路径排序后计算逐文件 SHA-256 manifest；实施前必须对重叠文件重新冻结，其他会话修改不回退。

## 3. 可保留底座与旧结论校正

| 底座 | 当前事实 | 重构边界 |
|---|---|---|
| Source carrier | `TerrainAsset` 有 width/height/sample_spacing/height_scale/height_samples/layers；layer 有 material/weightmap/strength | 保留 schema，迁移为 source document 与 chunked cooked artifact；不能继续把整张 height_samples 当 runtime resource |
| Asset chain | builtin `.terrain.toml`/`.terrain_layers.toml` importer、typed load、artifact cache/store/reference 真实存在 | 加 compiler fingerprint、content hash、mip/LOD/chunk dependency、atomic publish 和 residency lease |
| Scene carrier | `SceneTerrainAsset` 可被 document/management 统计和 direct reference 枚举 | 补 World component、load/save、Prefab/PIE、generation 和 migration；当前不是 execution |
| Validation | sample count、有限值与 source extension 有局部测试 | 扩展尺寸、format/endian, hole, normal, material, coordinate/large-world、byte decode 与 platform limit |
| Optional policy | package maturity beta、capability status partial、render slot descriptor-only | 保持诚实；只有真实 provider/artifact/install receipt 才能升级 |

## 4. 参考引擎裁决

Unreal Landscape 把 componentized height/weight data、edit layer、streaming proxy、collision heightfield、material/grass/Nanite/HLOD 与 World Partition cell 状态连接到同一 generation；Foliage 另有 prototype、HISM/cluster、cull、wind 和 streaming owner。Fyrox 即使是较小实现，也有 Scene Terrain node、chunk heightmap、quadtree LOD、raycast、brush stroke 和可撤销 chunk swap。Godot 的 height map shape 与 renderer storage 仍区分 authored resource、physics representation、visibility and draw data。Unity HDRP TerrainLit、GPUDriven 和 TerrainToMesh 证明 material/splat/holes、GPU culling 与 ray/path-tracing representation 必须共享 bounds/height contract。Bevy 的 render-world extraction/batching 只提供 ownership 参考，不是 Terrain 功能替代。

## 5. P1 差距与重构要求

| ID | 当前证据 | 需要重构 |
|---|---|---|
| RT-TER-01 | plugin runtime 注册 component descriptor 与 DiagnosticOnly importer；没有 heightfield byte decoder/cook backend | 建 `TerrainSourceDocument -> TerrainBuildArtifact` compiler，真实解码 raw/r16/png、格式/endianness、quantization、normal/hole/mip 生成，并以 artifact receipt 安装 |
| RT-TER-02 | builtin importer 与 plugin importer 并存，后者固定报告 backend 未安装 | 选择单一 importer/compiler authority；catalog/profile 在 provider 未安装时阻止可见入口，不允许两个 importer 产生不同 artifact |
| RT-TER-03 | `TerrainAsset.height_samples: Vec<Real>` 将整张样本内联；没有 tile/chunk/compression/streaming | 用 content-addressed tiled height artifact、mip/LOD、compression、page table、IO budget、residency/eviction 与 cancellation |
| RT-TER-04 | `TerrainLayerAsset` 只有 material/weightmap/strength | 增加 splat channel/format/color space、holes、normal/roughness/physical material、layer priority、virtual texture and dependency generation |
| RT-TER-05 | Scene 有 `SceneTerrainAsset` reference，但 `from_scene_asset` 不创建 component；`to_scene_asset` 固定 `terrain: None` | 增加 typed `TerrainComponent` 与 World attach/detach/load/save/clone/Prefab/PIE path，确保 reference、transform、layer stack、generation 无损 round-trip |
| RT-TER-06 | `SceneEntityAsset` 的 terrain 只参与 management/direct references，未进入 render/physics/nav runtime | 建 per-World `TerrainWorldRuntime`，从 component 建 instance、query snapshot、generation fence 和 shutdown/retirement receipt |
| RT-TER-07 | Terrain 在 `advanced_slots.rs` 是 descriptor-only，无 extract section consumer/pass | 实现 Terrain extract packet、patch topology、material bindings、depth/shadow/velocity/reactive pass；feature 未安装时 fail-close |
| RT-TER-08 | 全仓 graphics 生产代码没有 Terrain patch/clipmap/quadtree/height sampling/renderer consumer | 建 patch/clipmap owner，支持 frustum/horizon cull、LOD morph/stitch/crack fix、indirect draw、camera-relative/large-world precision |
| RT-TER-09 | 没有 authored conservative bounds、per-cell bounds 或 terrain visibility query | 在 artifact 中生成 bounds/height range/normal cone，接 visibility、HZB、occlusion、distance/significance 和 multi-view cache |
| RT-TER-10 | 没有 Terrain material domain；layer material 仅是 asset reference | 建 terrain material compiler/PSO key，splat/VT/hole/normal/physical-material/lighting/shadow contract；不能把普通 mesh material 结果算作 TerrainLit |
| RT-TER-11 | 没有 virtual texture、texture page residency 或 layer streaming | 接统一 resource residency owner，支持 page request/priority/fence/retry/device loss；source layer 与 GPU page 必须有 generation |
| RT-TER-12 | Physics HeightField DTO 与 TerrainAsset 无引用；Jolt conversion 将 height field 走 mesh/triangle representation | 建 Terrain->PhysicsCook artifact adapter，保留 sample spacing/height scale/holes/material IDs，优先 native heightfield representation，支持 incremental dirty region |
| RT-TER-13 | builtin physics 对 HeightField fail-close，Jolt runtime 不提供 Terrain update/query owner | world sync 注册 terrain collider generation，提供 raycast/shape cast/contact/physical material，cook/replace 不能阻塞主线程 |
| RT-TER-14 | Navigation geometry 对 `TriangleMesh | HeightField` 直接跳过，声称 owning asset path 收集；未找到实现 | 实现 TerrainNavGeometryAdapter，把同一 cooked tile generation 转为 Recast tiles，dirty bounds、partial rebake、off-mesh/area IDs 与 query snapshot 同步 |
| RT-TER-15 | nav bake render mesh fallback 只以固定 unit quad 代表 `NodeKind::Mesh/Cube`，不是 height surface | 删除固定 quad 作为 Terrain 证据；使用真实 patch/collider geometry，并记录 source generation、triangle count、fallback reason |
| RT-TER-16 | 无 foliage/grass prototype、scatter rule、HISM/cluster 或 terrain layer driven spawn | 建 FoliageSource/ScatterIR 与 Terrain layer/biome dependency，deterministic seed、cluster/instance artifact、wind/LOD/cull/streaming 与 physics/nav inclusion policy |
| RT-TER-17 | 无 World Partition manifest、cell coordinate/key、runtime hash、streaming source、data layer 或 HLOD owner | 建 versioned partition manifest、cell artifact、source priority、load/unload state machine、generation/lease、data-layer and HLOD build graph |
| RT-TER-18 | `BuiltinRenderFeature::Terrain` 枚举和 descriptor name 容易被 profile 当成 executable feature | capability registry 必须区分 descriptor/available/installed/executable，并给出 provider/artifact/device reason；未满足时不创建 pass |
| RT-TER-19 | terrain package plugin.toml 只标 beta/partial；first-party catalog/App 没有可运行 Terrain provider composition | 建 catalog -> App target -> runtime/editor/dist 一致的 selected provider receipt，native dynamic 只在真实 backend ready 时可选 |
| RT-TER-20 | dist runtime 明确 stateless，command/event/bridge/save/restore 全 None | dist 仅能作为 ABI shell；若需要 streaming/cook/host callbacks，定义 versioned commands/events/state ownership，不能以 stateless 声明覆盖生命周期 |
| RT-TER-21 | 没有 world clock/fixed update、edit/runtime lock、background build/cancel/deadline | TerrainWorldRuntime 和 TerrainBuildService 使用 typed job、budget、cancellation、progress、atomic swap、last-good generation 与 shutdown fencing |
| RT-TER-22 | large-world/precision/coordinate contract 只由通用 Transform 承担，terrain sample spacing/height scale 未进入 world origin/rebase | 定义 terrain local/chunk/world coordinate、origin rebasing、double/relative precision、seam quantization 与 physics/nav/render一致性 |
| RT-TER-23 | 示例 Vampire 的 `Baked Jungle Terrain` 可见性来自普通 mesh，terrain reference 不产生像素 | 把产品迁移到 typed Terrain instance，保留 mesh A/B oracle；验收 source-to-pixel、material、shadow、collision、nav 与 streaming 同 generation |
| RT-TER-24 | 测试集中在 TOML/sample count/reference/descriptor；没有 pixel/collision/nav/streaming/scale/device-loss | 增加 golden height decode、LOD seam、material/VT page、physics ray/contact、nav tile、foliage determinism、partition stream、device loss、stress/benchmark/product acceptance |

## 6. P2 性能与质量差距

| ID | 当前差距 | 需要重构 |
|---|---|---|
| RT-TER-25 | height samples 与 layer references 以单一 Vec/whole asset 访问 | chunked immutable pages、mip-aware prefetch、zero-copy upload 与 bounded decompression |
| RT-TER-26 | 没有 patch draw/instance batch、GPU culling 或 HLOD | indirect patch/cluster buffer、GPU frustum/HZB culling、HLOD/Nanite-compatible path 与 telemetry |
| RT-TER-27 | nav/physics/render 各自可能重新解释高度，无法共享 cache | canonical geometry artifact + per-consumer views，generation keyed cache |
| RT-TER-28 | layer references 只做 direct reference 枚举，未做 dependency graph/cascade invalidation | compiler dependency graph、dirty region/page invalidation 与 rebuild coalescing |
| RT-TER-29 | 没有 per-world memory/IO/GPU budget、priority、hysteresis | scalability service 按 camera/source/significance 预算 admission、eviction、quality tier |
| RT-TER-30 | 无 seam/crack/precision/normal continuity diagnostics | build-time seam checks、runtime edge validation、debug overlay 与 structured diagnostic |
| RT-TER-31 | 没有 GPU timing、page fault、cell residency、cook/streaming latency metrics | profile counters、trace receipt、capture/replay 与 budget regression gate |
| RT-TER-32 | 既有 tests 不能证明大世界多相机/多层/多线程安全 | deterministic stress/soak、multi-world/multi-viewport、concurrent edit/build/load、fault injection |

## 7. 资格门

| Gate | 必须证明 |
|---|---|
| RT-TER-G01 | source heightfield/weight/layer schema 有版本、migration、byte decode、format/endian/finite/sample-limit 证明 |
| RT-TER-G02 | compiler 产出 content-addressed artifact、dependency graph、fingerprint、last-good 与 atomic publish receipt |
| RT-TER-G03 | TerrainWorldRuntime 是每个 World 的唯一 owner，component attach/detach/load/save/clone/PIE/shutdown 可追踪 |
| RT-TER-G04 | Scene/Prefab/PIE round-trip 保留 terrain reference、transform、layers、generation，不再固定写 None |
| RT-TER-G05 | render feature 从 descriptor 变为真实 extract/patch/material/depth/shadow/velocity pass，graph 拥有资源生命周期 |
| RT-TER-G06 | patch LOD/clipmap/quadtree 的 bounds、stitch/morph、frustum/HZB/occlusion、multi-view 通过 golden seam tests |
| RT-TER-G07 | terrain material/VT/layer/holes/normal/physical material 与 PSO/cache/device profile 有一致 ABI |
| RT-TER-G08 | physics 使用同一 terrain artifact generation，heightfield query/contact/material 与 incremental dirty update 可验证 |
| RT-TER-G09 | navigation 使用同一 generation 生成真实 Recast tiles，dirty partial bake/query snapshot 可验证 |
| RT-TER-G10 | foliage/scatter prototype/rule/seed/cluster/LOD/wind/cull 与 Terrain layer dependency 可重现 |
| RT-TER-G11 | partition manifest、cell key、streaming source、data layer、HLOD、load/unload/retire、failure recovery 可重放 |
| RT-TER-G12 | source/runtime coordinates、origin rebase、sample spacing、height scale 在 render/physics/nav 一致 |
| RT-TER-G13 | catalog/App/editor/runtime/dist 的 capability/availability/installed/executable 状态和 receipt 一致 |
| RT-TER-G14 | optional/partial backend 缺失时不暴露 executable pass、pixel、collision、nav 或 success statistics |
| RT-TER-G15 | build/load/edit/stream/cook 有预算、cancel、deadline、atomic swap、device loss 和 stale generation 处理 |
| RT-TER-G16 | per-world/per-cell memory/IO/GPU/CPU budget、significance、quality tier、hysteresis 有生产 caller 与 telemetry |
| RT-TER-G17 | Vampire 或独立产品场景由 Terrain instance 产生可见 geometry/material/shadow、collision、nav、foliage 和 streaming 结果 |
| RT-TER-G18 | required test matrix 覆盖 byte/golden、seam/LOD、multi-world、stress/soak、fault/device loss、render/physics/nav/product acceptance |

## 8. 推荐实施顺序

1. 先硬切 source/artifact schema 与 TerrainWorldRuntime，接通 Scene load/save/Prefab/PIE；同时保持 plugin partial 并阻止 descriptor-only 入口冒充 executable。
2. 建 tiled height/layer compiler、residency/generation、patch geometry/LOD/bounds 和真正 Terrain render graph provider。
3. 以同一 artifact generation 接 Physics heightfield、Navigation tiles、material/VT、foliage/scatter 与 query snapshots。
4. 建 World Partition manifest/cell/HLOD/streaming authority、budget/cancel/device-loss/telemetry，并替换产品普通 mesh/固定 sample 证据。
5. 最后接 Editor authoring/preview/build receipt；所有 gate 通过后才改变 package maturity 或 profile 状态。

