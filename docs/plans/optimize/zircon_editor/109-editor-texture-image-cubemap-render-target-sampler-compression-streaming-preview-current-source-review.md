---
title: Editor Texture、Image、Cubemap、RenderTarget、Sampler、Compression、Streaming 与 Preview 当前源码复核
category: zircon_editor
report_id: Editor109
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor35
refreshes:
  - docs/plans/optimize/zircon_editor/35-texture-image-cubemap-render-target-sampler-compression-streaming-preview-authoring-review.md
related_code:
  - zircon_runtime/src/asset/assets/texture
  - zircon_runtime/src/asset/importer/image_decode.rs
  - zircon_runtime/src/asset/importer/ingest/import_texture.rs
  - zircon_runtime/src/asset/importer/environment_ibl.rs
  - zircon_runtime/src/asset/importer/environment_ibl
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/src/core/framework/render/image
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture
  - zircon_runtime/src/graphics/scene/resources/output_target_texture
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_mip_streaming.rs
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh
  - zircon_plugins/texture
  - zircon_plugins/texture_importer
  - zircon_plugins/asset_importers/texture
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/Cargo.toml
tests:
  - zircon_runtime/src/asset/assets/texture/descriptor/tests.rs
  - zircon_runtime/src/asset/assets/texture/upload_support/tests.rs
  - zircon_runtime/src/asset/importer/environment_ibl/source_staging
  - zircon_runtime/src/core/framework/render/image/metadata_validation.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset/tests.rs
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_writeback_converter.rs
  - zircon_plugins/texture_importer/runtime/src/tests
  - zircon_plugins/texture_importer/runtime/src/container/tests
  - zircon_plugins/texture/editor/src/tests.rs
  - zircon_plugins/first_party_editor_catalog/src/tests.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/34-sprite-atlas-tileset-tilemap-canvas-2d-animation-collision-preview-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/Texture.h
  - dev/UnrealEngine/Engine/Source/Editor/TextureEditor/Public/Interfaces/ITextureEditorToolkit.h
  - dev/UnrealEngine/Engine/Source/Editor/TextureEditor/Private/TextureEditorToolkit.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/TextureDerivedData.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/Texture2DStreamIn_IO.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/TextureRenderTarget2D.h
  - dev/godot/editor/import/resource_importer_texture.cpp
  - dev/godot/editor/import/resource_importer_layered_texture.cpp
  - dev/godot/editor/scene/texture/texture_editor_plugin.cpp
  - dev/godot/editor/scene/texture/texture_layered_editor_plugin.cpp
  - dev/godot/editor/scene/texture/texture_3d_editor_plugin.cpp
  - dev/bevy/crates/bevy_image/src/image.rs
  - dev/bevy/crates/bevy_image/src/image_loader.rs
  - dev/bevy/crates/bevy_image/src/hdr_texture_loader.rs
  - dev/bevy/crates/bevy_image/src/exr_texture_loader.rs
  - dev/bevy/crates/bevy_render/src/texture/texture_cache.rs
  - dev/Fyrox/fyrox-texture/src/lib.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceTexture.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Textures/Texture2DAtlas.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/ShaderLibrary/DebugMipmapStreaming.hlsl
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 109 · Editor Texture / Image / Cubemap / RenderTarget / Sampler / Compression / Streaming / Preview 工程化差距

## 1. 结论

Zircon 的 Texture 底层并非空白：Runtime 已有 typed `TextureAssetDescriptor`、颜色空间/用途/mip/压缩/SVT metadata，DDS/KTX/KTX2/ASTC container 识别，subresource upload plan，BC/ETC2/ASTC capability gate，2D array/cube 构造，sampler cache，物理 mip residency 状态，以及一条相对完整的 Environment IBL source -> PMREM -> SH/Irradiance 派生链。这些边界校验和 IBL 原子发布逻辑应保留并成为通用 Texture compiler 的样板。

当前不可称为工程级完成的核心不是缺几个格式枚举，而是 source、recipe、canonical decode、platform artifact、bulk/page、runtime install 和 Editor authoring 没有共同的不可变合同。普通图片仍可能因 HDR 量化、provider 装配或目标 compression 标签而得到不同或无法上传的结果；mip streaming、SVT、RenderTarget 和 sampler 的 API 名称也远大于可证明语义。

Editor 只把 Texture 作为一个 `ResourceKind::Texture`，preview 走原始 `image::open()` 和固定 192x192 thumbnail。`texture` 插件声明了不存在的 `plugins://texture/editor/authoring.zui`，没有 Texture document/toolkit/factory/reimport transaction，也没有进入 first-party Editor product assembly；`texture_importer` 与 `asset_importers/texture` 继续重叠声明 importer owner。因而不能用更多 UI checkbox、静态 ZUI 或 render-thread 包装来掩盖缺失的产品链。

目标边界必须收敛为：

`TextureSourceAsset + versioned ImportRecipe -> canonical decoded/intermediate image -> capability/platform-qualified BuildGraph -> immutable TextureArtifact + BulkMip/VirtualPage artifacts -> generation-qualified RuntimeInstallReceipt`。

Texture2D、Layered/Cube/Volume、RenderTarget、VirtualTexture 和 Sampler 要么有独立 typed identity，要么明确作为同一 artifact 的受限 variant；Editor、headless cook 和 Runtime 必须消费同一 recipe/compiler/artifact，而不是各自解释 source。

## 2. 审查范围与证据

### 2.1 当前工作树物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests | 指纹 |
|---|---:|---|
| Zircon Runtime/Editor/Plugin selected | **157 / 34,911 / 31,886 / 1,224,840 / 416** | `55178c2121a271cf553194e0c0003d78478d33a4d14f78ad0eb5e81dc6f13c5a` |
| Unreal/Godot/Fyrox/Bevy/Unity reference | **20 / 22,454 / 19,547 / 871,783 / 11** | `8675dce2ad42cccedf052fc515e5834d2908a968ab1ea41d19b05f0fca61167a` |
| Zircon selected union | **177 / 57,365 / 51,433 / 2,096,623 / 427** | `88d58698834550e7d20528f37d6a7e671d08a542dce8ecddcf599e342f2b2d3e` |

统计对每个 selected root 去重后按相对路径排序，以 UTF-8 内容计算 SHA-256；测试数是 `#[test]`/`#[async_test]` 属性计数，不等价于通过的产品测试。当前 `baseline_epoch=524`，且工作树包含与本轮无关的在途修改，实施前必须重新导出 manifest、确认 owner 与 fingerprint。没有运行 Cargo、GPU、import corpus/fuzz、cook/package、visual golden 或跨平台动态验证。

### 2.2 已完成的局部底座

1. `TextureAssetDescriptor` 能表达颜色空间、usage、dimension、mip policy、compression target、SVT metadata；RGBA8 upload readiness 会检查 mip/layer shape、字节长度与 format。
2. DDS/KTX/KTX2/ASTC parser 能识别 block layout、mip/layer/subresource，设备能力不足时会拒绝 container；读取预压缩 container 不能被误写成平台 cook。
3. Environment IBL 专用路径保持 HDR/EXR 为 RGBA32F，生成 source cube、GGX PMREM、SH9、optional irradiance，request 带 source/layout/required contents，artifact/cache 带 version、BLAKE3 key 与 atomic restore/rebuild。
4. `texture_importer` 有 mipgen、normal convention 和 BC5 transcode 的局部实现；GPU texture upload、sampler cache、residency state、preview job admission 与 catalog generation 也有可复用的窄底座。

## 3. P0：必须先关闭的断路（5 项，全部 Open）

### P0-1：普通 HDR/EXR 入口会量化或在晚期失败

`decode_texture_source_image()` 无条件调用 `DynamicImage::to_rgba8()`，builtin 与 `texture_importer.image` 都使用它；只有 Environment IBL 使用 `to_rgba32f()`。`TexturePayload` 只有 `Rgba8` 与 opaque `Container`。于是普通 HDR/EXR 要么先丢失 radiance，要么声明 HDR/float format 后在 metadata/upload readiness 报“RGBA8 payload requires conversion”。必须先建立 float/half canonical intermediate、显式颜色与 alpha policy，以及 source -> GPU readback fidelity gate。

### P0-2：requested compression、actual payload 与 provider 结果不一致

metadata 默认可选择 BC1/BC4/BC5/BC6H/BC7，但现有 encoder 只有 BC5；builtin importer 不做 offline mip/normal/BC，而可选 provider 会做。`format: String`、compression target 与 payload layout 无 typed invariant。同一 source 会因插件是否安装而生成不同结果，甚至把目标标签当成实际 artifact。必须先统一 compiler/provider ownership，并以 actual format、mip count、block bytes 和 encoder receipt 发布。

### P0-3：artifact/cache 不能证明 source、recipe、platform 与 bulk 身份

普通 Texture cache 把整个 `TextureAsset` bincode 成单块 payload，缺 source digest、recipe revision、tool/encoder version、platform/feature tier、quality/RDO、bulk mip/page 与 dependency provenance。IBL 专用 key 虽有算法版本，却未接入通用 platform variant/cook/package。必须先让 cache key 和 immutable artifact manifest 覆盖所有影响输出的输入，并支持 atomic generation/GC/rollback。

### P0-4：Mip streaming、SVT、RenderTarget 是命名大于实现的原型

首次 ensure 先完整上传，streaming prepare 同步读取/重建 texture/view/bind group；需求仅来自主视图 mesh/material，屏幕估算只看 transform scale，compressed mip 被排除，预算默认 `u64::MAX`，没有 async bulk I/O、fence、cancel、generation receipt。SVT 只有 page size/border/tail metadata，没有 page compiler、feedback、page table、tile cache。RenderTarget 复用普通 Texture handle，只接受单层单 mip RGBA8、sample 1。必须在声明 capability 前建立 tail-first、compressed bulk、完整 consumer demand、typed target、MSAA/resolve、fault telemetry 和 bounded budget。

### P0-5：Editor Texture 产品与 runtime/compiler 没有闭环

builtin registry 只有一个 Texture kind 和 `SourceImage` thumbnail；container/cube/array/volume/recipe/artifact 不可可靠预览，缺 channel/mip/layer/face/exposure/normal/compression inspection。`texture` Editor plugin 引用不存在的 `authoring.zui`，没有 document/operation factory/toolkit/reimport/save/undo；first-party catalog/App 也未形成可打开可执行的 Texture feature。必须先以真实 source/recipe/artifact/install receipt 驱动 toolkit 与 preview，descriptor-only plugin 不得标 stable。

## 4. P1：Runtime 与资产链（60 项，全部 Open）

1. 用 typed `TextureFormat`、`TextureDimension`、`TextureUsage`、`AlphaMode`、`ColorEncoding` 替换跨模块自由字符串。
2. 让 payload layout、format、extent、mip、layer、face、row alignment 由同一 invariant 校验。
3. 分离 `TextureSourceAsset`、canonical intermediate、platform `TextureArtifact` 和 runtime handle。
4. 保存未知 recipe 字段并提供 versioned migration，不再 silently drop settings。
5. 普通 image importer 与可选 plugin 共享一个 compiler 入口和 diagnostics。
6. HDR/EXR 支持 `rgba16float`/`rgba32float` intermediate，禁止隐式 RGBA8 fallback。
7. 明确 sRGB/linear/HDR transfer、premultiply、coverage alpha 和 NaN/clamp policy。
8. normal map 明确 tangent convention、renormalization、BC5/BC7 fallback 与 mip filter。
9. 将 channel packing、swizzle、resize、crop、border、atlas padding 纳入 recipe。
10. offline mip 生成必须真正产出每级 bulk，而不是只写 `GenerateOffline` metadata。
11. runtime mip generation 只作为显式 fallback，不能覆盖 cook 产物。
12. 补齐 BC1/4/6H/7、ETC2、ASTC、Basis backend 或将 capability 降级为 unsupported。
13. encoder receipt 记录 tool/version/quality/RDO/seed、actual format 与 quality metric。
14. requested target 与 actual target 分字段持久化，unsupported 不得伪装成功。
15. 将 source bytes、recipe、platform、GPU tier、encoder version 纳入 artifact key。
16. 将 bulk mip/page 划入 chunk manifest、hash、size、compression、dependency graph。
17. 让 DDC 支持分块读、校验、GC、size/age budget 与旧 generation rollback。
18. IBL source/PMREM provider 接入通用 build graph，保留专用算法版本。
19. IBL artifact 增加 platform compression、packaging variant 和 reflection consumer receipt。
20. cubemap 明确 face order、handedness、axis、rotation、flip 与 seam fixup recipe。
21. cubemap cross/latlong/six-file/DDS/KTX 使用统一 orientation golden。
22. cubemap mip edge fixup 与 irradiance/PMREM sampling 必须有数值误差门。
23. array/volume source 采用 stable layer/slice identity，不以相邻文件路径代替资产引用。
24. array 支持 per-layer dimensions、crop/resample、missing-layer policy 与 deterministic ordering。
25. volume 需要 depth extent、slice stride、3D filter、bricking 与 fallback policy。
26. `.zcube`/`.zarray` 保留可编辑 source recipe，不折叠为无来源 TextureAsset。
27. `.zcube`/`.zarray` 依赖必须进入 asset graph、reimport 和 cook invalidation。
28. container ingestion 与 source cook 分开命名、权限和 artifact kind。
29. KTX2 Basis transcoding 需要明确 backend、target format、quality 与 deterministic receipt。
30. DDS/KTX/ASTC malformed input 做 bounded offset/extent/decompression validation。
31. 设备 capability 转换为 compile-time platform matrix，不在 render prepare 才发现不支持。
32. RenderSamplerDescriptor 增加 compare、border、LOD min/max/bias、reduction、unnormalized 等字段。
33. anisotropy、mip bias 与 sampler cache key 必须有一致的 typed policy。
34. 独立 Sampler asset/preset 支持 material/UI/terrain 复用和版本迁移。
35. RenderTarget 使用 typed target descriptor，禁止普通 Texture handle 任意引用。
36. 支持 HDR、depth/stencil、UAV、array/cube/volume、MSAA 与 resolve 合法组合。
37. RenderTarget 支持 relative/dynamic size、resize generation 与 format fallback receipt。
38. 引入 target pool/alias/history 生命周期，防止旧 view 跨 generation 使用。
39. readback、capture、post-process 与 target producer 记录 ownership、latency、fence。
40. camera target、reflection capture、secondary view、UI target 使用独立 consumer contracts。
41. mip streamer 采用 tail-first startup，初次加载不再全量上传。
42. 构建 async bulk I/O、decode、staging ring、copy fence 和 render-frame handoff。
43. 需求模型消费 mesh bounds、UV density、viewport footprint、anisotropy 与 camera priority。
44. 覆盖 sprite、UI、particle、terrain、decal、reflection、capture、preview 等 consumer。
45. 以 bytes、VRAM、CPU、I/O、GPU copy、deadline 建立项目/平台预算。
46. 引入 request priority、dedupe、cancel、backoff、late completion generation fence。
47. compressed BC/ETC2/ASTC bulk 支持独立 mip in/out 和 copy alignment。
48. eviction 保留 mip tail、pinned/critical 资产并报告原因，不静默降级。
49. SVT 建立 page compiler、mip tail、page table、feedback、physical tile cache。
50. SVT 支持 request overflow、dedupe、prefetch、eviction、device loss 和 fault telemetry。
51. SVT 未完成前从 capability/admission 中移除，而不是 metadata 后静默全驻留。
52. 所有 streaming/target 操作生成 stable diagnostics code、asset、generation、budget receipt。
53. cache/writeback 使用 temp+fsync+atomic rename，禁止半写 artifact/preview。
54. import/build/stream 任务进入统一 background job，具备 cancellation 与 shutdown drain。
55. I/O、decode、upload 不能在 render thread 同步等待。
56. 设定 deterministic single/multi-thread、warm/cold cache 和 machine-to-machine golden。
57. 建立 1/1k/100k asset、4K/8K/16K、slow disk、VRAM pressure、upload saturation 基准。
58. 建立 device lost、OOM、disk full、worker panic、cache corruption fault injection。
59. headless cook/package 必须在无 Editor cache 环境生成完整 target/platform manifest。
60. 公开与 Unreal/Godot/Fyrox/Bevy/Unity Graphics 可比较的 build/startup/VRAM/hitch/quality 方法学。

## 5. P1：Editor、Plugin、Preview 与发布（同样计入上述 60 项）

上述 Runtime P1 的资产合同必须在 Editor 侧有对应 owner，当前具体缺口是：

1. 为 Texture2D、Layered、Cube、Volume、RenderTarget、VirtualTexture 定义 AssetTypeId/subtype 与 creation template。
2. 用 schema-driven import inspector 显示 recipe default、platform override、resolved actual 和 validation。
3. 引入 Texture document、revision、dirty/save/autosave/recovery/conflict 与 transaction receipt。
4. 将 decode/mip/compression/IBL/preview/reimport 接入 bounded/cancellable background job。
5. 真实 toolkit 必须能打开 source、recipe、artifact、generation 与 install diagnostics。
6. 2D preview 提供 fit/1:1/zoom/pan、alpha checker、RGBA/luma、sRGB/linear、exposure/gamma、pixel probe。
7. 支持 mip、layer、face、slice、cross/latlong、3D orientation、normal 与 compression compare。
8. RenderTarget preview 显示 live/pause/frame pin、HDR/depth/stencil、MSAA resolve、history/fence。
9. Thumbnail provider 按 subtype/artifact decoder 选择，不能继续 raw source image 拉伸到 192x192。
10. preview key 纳入 provider/renderer/schema/artifact generation，cache 有 size/count/age GC。
11. preview scheduler 按 visible/selected/background 分级并限制 decoded/upload CPU/GPU bytes。
12. 删除不存在的 `plugins://texture/editor/authoring.zui` 引用，补齐真实 view/template 或拒绝 admission。
13. `texture`、`texture_importer`、`asset_importers/texture` 只能保留一个 importer owner。
14. plugin registration 必须验证 module、factory、resource URI、operation/controller/service。
15. first-party runtime/editor catalog 与 App target 显式声明 Texture feature，覆盖 default/client/server/editor 矩阵。
16. importer/provider priority、builtin fallback 和 plugin install 状态不能改变 source 结果。
17. reimport/build 只有在 artifact receipt 原子提交后替换当前 generation。
18. source-vs-artifact diff 显示 digest、recipe、format/mips/bytes/quality 与依赖影响。
19. 外部 source/recipe 修改触发三方 diff，不覆盖未保存编辑。
20. Editor 产品必须消费真实 IBL PMREM/SH artifact，而不是固定环境 thumbnail 或静态 summary。

## 6. P2：长期能力（12 项，全部 Open）

1. GPU/remote texture encode farm 与 deterministic cost/quality receipt。
2. perceptual/RDO 自动搜索与内容感知质量预算。
3. 多层材质 virtual texturing、anisotropic feedback 和跨场景 physical cache。
4. sparse/reserved resource backend 与 copy fallback。
5. UDIM stable tile、缺失策略、per-tile cook/stream/paint。
6. procedural/video/canvas/compute producer graph 与 dirty-region double buffering。
7. atlas、glyph、bindless table 与 residency heat 协同。
8. neural compression/super-resolution 作为同一 artifact provider。
9. alpha bleed、颜色空间、normal、mip shimmer、compression artifact 自动审计。
10. recipe/channel/platform override 的多人 field-level merge 与 lock。
11. artifact schema/encoder 双读、canary、rollback、patch 和 cache migration。
12. 跨引擎、跨平台、固定 VRAM/磁盘预算的公开质量与性能基准。

## 7. 分层重构顺序

### M0：Truthfulness 与编译基线

冻结 Texture stable capability；统一 importer owner，修复当前 plugin/lock/mipgen 阻断；普通 HDR/EXR 不能以 RGBA8 成功，requested compression 不能冒充 actual，缺 factory/ZUI 的 Editor admission 必须失败。

### M1：Source、Recipe、Typed Shape

建立 versioned `TextureSourceAsset`、`TextureImportRecipe`、typed format/extent/subtype/color/alpha/normal policy；保留旧字段并提供迁移，禁止 source recipe 在 `.zcube`/`.zarray` 导入时丢失。

### M2：Canonical Decode 与 Shared Compiler

统一 builtin/plugin 入口，产生 UNorm/float canonical intermediate、mip/normal/channel/resize 结果和 structured diagnostics；Environment IBL 作为 provider 接入但保留算法版本。

### M3：Platform Artifact、DDC 与 Atomic Publication

按 source/recipe/tool/encoder/platform/feature key 生成 actual descriptor、bulk mip/page chunks、provenance、quality receipt；实现 cache 校验、GC、clean headless cook 与 rollback。

### M4：Runtime Install 与 Physical Streaming

完成 tail-first、async bulk I/O/decode/upload、compressed mip、全 consumer demand、priority/budget/cancel/fence/generation receipt；失败写入统一 journal。

### M5：Sampler、RenderTarget 与 RenderGraph Lifetime

完成完整 SamplerState、typed target、HDR/depth/MSAA/resolve、relative size、pool/alias/history/readback 与 device-tier matrix。

### M6：Virtual Texture

实现 page compiler、tail、feedback、page table、physical cache、prefetch/eviction/fault debug；未完成前 capability 不可见。

### M7：Editor Toolkit 与 Preview

装配 subtype-aware registry、schema inspector、transaction/save/reimport/build job、2D/layer/cube/volume/target toolkit、artifact diff 和 atomic preview cache。

### M8：Plugin 收敛与 Release Qualification

删除重复 owner，补 module/factory/catalog/App closure；完成 malformed/fault/determinism/visual/performance/cross-platform/package/rollback 门禁后才允许 stable。

## 8. 验收门禁

1. HDR/EXR source -> artifact -> GPU readback 保持 float 动态范围，普通图片不得经过 RGBA8。
2. 相同 source/recipe/provider 安装状态生成相同 key、artifact、diagnostics；多线程、warm/cold cache、不同机器 deterministic。
3. actual format/mip/layer/face/bytes 与 payload 逐字节一致，platform key 改变必定失效旧 artifact。
4. DDS/KTX/KTX2/ASTC/image/cube/array malformed、overflow、decompression bomb、path traversal 有界拒绝且无 panic/OOM。
5. cubemap orientation/seam、sRGB/linear/HDR、alpha/normal/mip、compression quality 通过 CPU 数值和 GPU framebuffer golden。
6. IBL PMREM/SH/Irradiance serial/parallel、cache reuse、algorithm invalidation 与 reflection consumer 不退化。
7. RenderTarget 合法 HDR/depth/MSAA/array/cube/storage 组合创建、clear、sample、resolve、readback；非法组合早拒绝。
8. tail-first、compressed mip streaming、async/cancel/fence/late-generation、完整 consumer demand 在固定 VRAM/IO/upload budget 下收敛无 hitch/thrash。
9. SVT source -> page artifact -> feedback -> page table/cache -> frame 闭环，缺页、overflow、device loss 可恢复诊断。
10. Editor recipe transaction、reimport、preview、catalog、plugin admission 与 Runtime artifact/install receipt 全链路可执行。
11. clean headless package 只携带实际 target/platform chunks，无 Editor cache 依赖，client/server/editor 组合均有 capability 证据。
12. Stable/Complete 只由 compile、registration、artifact、runtime、Editor、fault、platform 和 scale evidence 派生。

## 9. 禁止的临时修补

1. 禁止继续让普通 HDR/EXR 经过 `to_rgba8()` 后靠 format 字符串伪装 float。
2. 禁止只增加 BC6H/BC7 enum、UI 选项或 manifest capability 而没有 encoder、actual descriptor 和 quality gate。
3. 禁止把读取 DDS/KTX/ASTC 描述为平台 Texture cook。
4. 禁止 builtin 与 plugin 维护不同 mip/normal/compression authority。
5. 禁止把 descriptor、静态 ZUI、固定 summary、raw thumbnail 当作 Texture Editor 产品。
6. 禁止让 Camera 接受任意 Texture handle 充当 RenderTarget。
7. 禁止在 render thread 同步读取 source、decode、重建大 Texture 或等待文件 I/O。
8. 禁止把全驻留后调整 GPU mip range 命名为完整 streaming，也禁止 SVT metadata 静默全驻留。
9. 禁止用 test attribute 数量、ignored screenshot 或手工截图替代 import/cook/GPU/preview/scale 资格。
10. 禁止在重新导出 manifest/fingerprint 前实施本报告假设，或修改用户 lockfile 绕过 `--locked`。

## 10. 本轮产出边界

本轮只新增 Editor109 审查、索引和分层计划，没有修改 Runtime、Editor、Plugin、App 或 tests production code，也没有运行 Cargo 或动态验证；未查询或实时跟踪协调器。实现必须从 M0 开始，并在实施前重新检查当前工作树的 157-file scope、baseline、owner、lock drift 与本报告所有路径。
