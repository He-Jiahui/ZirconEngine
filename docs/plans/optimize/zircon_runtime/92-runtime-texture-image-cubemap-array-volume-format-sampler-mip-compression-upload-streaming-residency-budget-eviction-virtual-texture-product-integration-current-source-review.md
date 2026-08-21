---
title: Runtime Texture、Image、Cubemap、Array、Volume、Format、Sampler、Mip、Compression、Upload、Streaming、Residency、Budget、Eviction、Virtual Texture 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime92
review_date: 2026-08-21
baseline_head: be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1
baseline_epoch: 336
related_code:
  - zircon_runtime/src/asset/assets/texture
  - zircon_runtime/src/asset/artifact
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading
  - zircon_runtime/src/core/framework/render/image
  - zircon_runtime/src/graphics/runtime/render_framework/budget
  - zircon_runtime/src/graphics/scene/resources/gpu_texture
  - zircon_runtime/src/graphics/scene/resources/post_process_lut_texture
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
  - zircon_runtime/src/graphics/scene/scene_renderer
  - zircon_plugins/texture_importer/runtime/src
tests:
  - zircon_runtime/src/asset/assets/texture
  - zircon_runtime/src/asset/artifact
  - zircon_runtime/src/core/framework/render/image
  - zircon_runtime/src/graphics/scene/resources/gpu_texture
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_mip_streaming.rs
  - zircon_runtime/src/tests/runtime_absorption
  - zircon_plugins/texture_importer/runtime/src/tests
  - zircon_plugins/texture_importer/runtime/src/container/tests
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/35-texture-image-cubemap-render-target-sampler-compression-streaming-preview-authoring-review.md
  - docs/plans/optimize/zircon_plugins/18-first-party-texture-source-importer-runtime-editor-dist-catalog-image-cubemap-array-volume-compression-streaming-product-integration-review.md
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/RenderAssetUpdate.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/RenderAssetUpdate.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/Texture2DStreamIn.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/Texture2DStreamIn_IO.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/Texture2DStreamOut_AsyncCreate.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/StreamingManagerTexture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/VolumeTextureStreaming.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/VT/VirtualTextureUploadCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/VT/VirtualTextureUploadCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/VT/VirtualTextureChunkManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/VT/VirtualTextureChunkManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/SparseVolumeTexture/SparseVolumeTextureStreamingManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/SparseVolumeTexture/SparseVolumeTextureStreamingManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Rendering/StreamableTextureResource.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Rendering/StreamableTextureResource.cpp
  - dev/bevy/crates/bevy_render/src/render_asset.rs
  - dev/bevy/crates/bevy_render/src/texture/gpu_image.rs
  - dev/bevy/crates/bevy_render/src/texture/texture_cache.rs
  - dev/godot/servers/rendering/renderer_rd/storage_rd/texture_storage.h
  - dev/godot/servers/rendering/renderer_rd/storage_rd/texture_storage.cpp
  - dev/Fyrox/fyrox-impl/src/renderer/cache/texture.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Textures/Texture2DAtlas.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/ShaderLibrary/VirtualTexturing.hlsl
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/VirtualTexturingSettingsSRP.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/VirtualTexturing/Shaders/DownsampleVTFeedback.compute
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Texture、Image、Cubemap、Array、Volume、Format、Sampler、Mip、Compression、Upload、Streaming、Residency、Budget、Eviction、Virtual Texture 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon的Texture系统不是空壳。资产侧已有D1/D2/D3/Cube、array/cube manifest、颜色空间、用途、mip policy/filter、compression target、sampler与普通mip streaming metadata；upload readiness能够解析和校验DDS、KTX1、KTX2、ASTC、lightmap RGBA16F及明确的mip/layer subresource布局。Importer已有RGBA8 Box/Kaiser与normal-aware mip生成、DX/GL normal convention、BC5编码；KTX2 Zstd/Zlib标准supercompression已经在导入期展开，不能再沿用旧结论说所有KTX2 supercompression都未处理。Runtime planner也真实具备wanted/resident tail range、每帧16次transition、32 MiB upload预算、1 mip hysteresis、稳定排序、预算驱逐与stale transition拒绝。这些机制应保留为未来统一Texture编译、上传与驻留系统的局部内核。

但产品主路径仍不是工程级Texture系统。`TexturePayload`只有`Rgba8`与opaque `Container(Vec<u8>)`，通用artifact又把完整bincode对象先压缩再按64 KiB物理切块；mip、layer、face、slice和page没有可寻址的语义块。render submission内的`ensure_scene_resources`同步调用`load_texture_asset`，cache miss时同步读文件、校验、Zstd解压、bincode反序列化并clone完整资产，再创建WGPU资源。首次纹理无条件完整驻留，随后同一帧可能再建更小的物理tail纹理；promotion/eviction同样重新clone完整CPU payload、创建replacement、复制公共mip并直接写缺失mip。

当前所谓`MipStreamingTask`只有同步循环中的逻辑transition ID，不是带I/O、decode、upload、completion、cancel、deadline、asset generation、device generation和fence retirement的异步任务。demand只来自主视图可见mesh的material texture，屏幕覆盖用`translation + scale.length() * 0.5`近似，没有真实bounds、UV density/tiling、采样导数、遮挡/运动预测、secondary view、shadow/reflection、sprite、UI、cookie、LUT或volume。compressed texture只能完整上传；普通RGBA8 D1/D3 upload直接拒绝，3D LUT又走另一条私有资源路径。`SvtSettings`只有metadata和validation，没有page cook、page table、feedback、physical tile cache或runtime consumer。

横向扫描进一步证明缺少唯一Texture/RHI服务：production-like Rust中有86处`create_texture`、83处`write_texture`和24处texture/buffer copy，覆盖56、47和17个文件命中集合。普通texture、LUT、IBL、advanced lighting、post process、shadow/history、render graph transient和插件各自创建、上传、计数与销毁；persistent texture budget却只合计`ResourceStreamer::textures`，不包含多数长期纹理、replacement old+new峰值和in-flight allocation。固定1 GiB reference值已经接入streamer，而不是完全未接线；真正问题是预算输入、覆盖范围、设备适配和执行闭环仍不成立。

Runtime09D登记的6项P0在当前源码中仍未闭合，但其局部描述需要上述纠偏。本报告不重复新增P0，登记 **48项P1、12项P2与48个资格门**。目标是硬切到`TextureSchemaAuthority + TextureBuildService + TextureArtifactStore + TextureGenerationService + TextureUploadService + TextureResidencyService + TextureBudgetController + SamplerCatalog + VirtualTextureService`，并消费Runtime85/86/89/90/91的唯一owner。在真实cold/warm I/O、compressed partial residency、camera traverse/teleport、reload、device loss、OOM、100k texture和同画质Unreal对照证据闭合前，不能声称Texture功能达到Unreal级，更不能声称性能或表现优于Unreal。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test markers | 证据等级 | fingerprint |
|---|---:|---|---|
| Runtime production owner roots | **99 / 23,316 / 21,577 / 847,675 / 109** | E3逐文件读取asset texture、artifact/load、image contract、GPU resource、streamer与budget | `38e01a21e18e2577f7ead0e9e5ad286bf1c0dda7d12edefc70937e30da8953ff` |
| Runtime focused tests | **4 / 1,375 / 1,218 / 44,974 / 57** | E3读取upload、mip planner/state、budget和结构guard | `d86b7958e06d95acfb1c67425d392a79b7597a44e11df31b01afde2c6bab761c` |
| Texture importer production | **22 / 5,123 / 4,821 / 179,512 / 12** | E3读取image/PSD/container、array/cube、mipgen、normal与BC5 | `ed34f2c3112f3df341fd49ef980ffb04ac6c96712f81a174ae56434595b01efd` |
| Texture importer tests | **18 / 4,560 / 4,014 / 150,050 / 162** | E3读取DDS/KTX/ASTC、settings、mipgen、BC5与diagnostic测试 | `e6ef2375ef9fda433c3fcde34998e44a7109d89397ef5d1c1c050cbe74c7b009` |
| direct texture callsite corpus | **86 / 27,520 / 25,700 / 1,031,654 / 164** | E3冻结Runtime与Plugins production-like直接texture create/write/copy全集 | `0ab50bc45d5bc393907ea778694926b76459aa7fe6df28357a669b6f3e048de5` |
| 五引擎参考切片 | **26 / 19,780 / 17,066 / 758,851 / 15** | E2/E3读取Unreal streamer/VT/SVT、Bevy lifecycle、Godot storage、Fyrox async cache和Unity VT/atlas | `379de2c10956ef7aa17fde616049d664a8a3274259b81bbc0ecc5fb99623defb` |

冻结集合代表2026-08-21共享working tree，不是只读HEAD或实现验收receipt。Git基线为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator baseline epoch为336。Bevy、Godot、Fyrox与Unity Graphics参考revision分别为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`与`a7e4c051d256a781ab362c64316b125a1e104694`。Unreal镜像`Build.version`为6.0.0/UE5/changelist 0且无独立`.git`，由reference aggregate fingerprint冻结。

冻结时另一个共享Session正在修改`zircon_plugins/texture_importer/runtime/src/mipgen/kernel.rs`。本文读取并计入该working-tree版本，但不拥有也未修改它；因此`source_recheck_required: true`，实施前必须重新生成fingerprint并核对行为差异。

### 2.2 子域物理范围

| 子域 | 文件 / 行 / test markers | 当前判定 |
|---|---:|---|
| asset texture | **23 / 5,733 / 42** | descriptor、payload、array/cube、metadata、container upload plan |
| core render image | **10 / 1,101 / 19** | image dimension/usage/fallback、metadata、sampler与validation |
| GPU texture | **7 / 2,249 / 24** | uncompressed/compressed/lightmap、sampler cache、physical tail rebuild |
| resource streamer + prepared | **27 / 6,361 / 44** | synchronous ensure、demand、planner、transition、prepared maps与accessors |
| special texture resources | **6 / 722 / 5** | LUT/irradiance/output等分叉资源路径 |
| artifact/loading | **25 / 6,874 / 18** | generic 64 KiB chunks、whole-object decode、typed clone load |
| budget/product plumbing | **5 / 1,651 / 14** | fixed reference budget、profile写回与scene renderer转发 |

### 2.3 横向创建与复制统计

统计覆盖`zircon_runtime`与`zircon_plugins`的production-like Rust路径，排除常规tests目录；它证明authority分散，不等同于单帧调用次数。

| 模式 | occurrences / files | 判定 |
|---|---:|---|
| `device.create_texture` / `.create_texture(` | **86 / 56** | sampled、persistent、transient、history、atlas和feature各自直建 |
| `queue.write_texture` | **83 / 47** | 缺共享staging ring、upload ticket与in-flight bytes authority |
| `copy_texture_to_texture` | **12 / 6** | physical rebuild、resolve/copy路径未统一进入预算与completion |
| `copy_buffer_to_texture` | **3 / 3** | staging copy是少数局部实现，不是默认upload service |
| `copy_texture_to_buffer` | **9 / 8** | readback/capture缺共享生命周期与backpressure模型 |

### 2.4 证据限制与owner边界

- 本轮只做current-source review，没有修改Rust、Cargo、asset或tooling，也没有运行Cargo、Editor/App、真实GPU、RenderDoc、cook/export、device-loss、OOM、soak或benchmark。
- 用户已明确tooling未来迁移到Rust；本文不审查Python/Node工具链，也不把新增外部脚本当成架构修复。
- Runtime64拥有通用Resource authority；Runtime85拥有source/import/cook/build graph；Runtime86拥有schema/version；Runtime89拥有Render Graph resource lifetime；Runtime90拥有RHI device/upload submission/completion；Runtime91拥有Material/Shader/PSO。Runtime92只拥有Texture领域artifact、generation、upload request、residency policy与预算接入，不复制这些服务。
- Editor35拥有Texture authoring/document/preview，Plugins18拥有首方Texture package/importer纵向装配；本篇只定义它们必须产出并消费的Runtime合同。
- `dev/Graphics`只包含Unity Graphics/SRP package源码，不含Unity native texture streamer；本文只引用atlas与VT settings/feedback shader，不臆造其native调度。

## 3. 当前产品链与断裂点

```text
source image / PSD / DDS / KTX / KTX2 / ASTC / array / cube manifest
  -> texture_importer decode / parse / mipgen / optional BC5
  -> TextureAsset { rgba Vec | opaque container Vec, string format, duplicated extent fields }
  -> ArtifactCacheAsset -> bincode -> zstd -> generic 64 KiB chunks
  -> ensure_scene_resources (render submission)
  -> ensure_resident -> whole artifact read/decode -> load_typed clone
  -> GpuTextureResource::from_asset -> direct create/write/optional private submit
  -> PreparedTexture::fully_resident
  -> synchronous mip planner/rebuild -> replacement texture + copy/write + Arc swap
```

断裂集中在四个位置：importer产物不具备平台化语义块；artifact random access不知道mip/layer/page；render submission承担同步I/O和GPU创建；resident generation没有独立于CPU asset clone与WGPU对象的生命周期。`SvtSettings`在该链上没有下游节点，special textures又绕过普通Texture generation和预算。

## 4. 当前应保留的真实基础

1. `TextureUploadPlan`及DDS/KTX/KTX2/ASTC的边界、format、mip/layer布局校验应保留，迁入typed platform artifact validator。
2. KTX2 Zstd/Zlib展开、BC5 normal转码、Box/Kaiser/normal-aware mip kernel是真实算法，不应被临时外部命令替换。
3. `MipStreamingPlan`的纯policy部分，包括tail保留、wanted range、priority、upload/resident budget、hysteresis和稳定排序，可作为新ResidencyService的测试内核。
4. transition ID与revision recheck提供最低stale防护，可升级为generation-qualified ticket，不能误当完整异步状态机。
5. artifact chunk的content hash、size验证、atomic manifest publish和LRU cache可迁入semantic block store。
6. sampler descriptor与cache虽然不完整，但已形成不可变key和Arc生命周期挂点，可并入设备generation化的SamplerCatalog。
7. RenderMemoryBudget已经把persistent texture参考值传入streamer；应扩展同一预算链，而不是另建Texture私有全局变量。

## 5. 既有P0 current-source复核

| 既有P0 | 当前状态 | current-source证据与纠偏 |
|---|---|---|
| 09D P0-1 submission同步I/O/decode/GPU创建 | **开放** | `ensure_scene_resources:17-120`直接ensure全部consumer；`ensure_texture:27-41`同步load与create；`ensure_resident:53-100`同步artifact read |
| 09D P0-2 artifact无semantic subresource | **开放** | `store.rs:125-170`恢复完整bincode对象；`publish_chunks:441-473`只按64 KiB压缩字节切块，无mip/layer/page目录 |
| 09D P0-3 full-first与replacement峰值绕预算 | **开放** | `ensure_texture:31-42`先full resident；`from_asset:433-550`另建tail texture；预算只记录最终logical bytes |
| 09D P0-4 无跨资源唯一residency authority | **开放** | 普通textures有mip预算，LUT/IBL/history/output/render graph/plugin texture分散；横向仍有86处create |
| 09D P0-5 clone load绕过lease | **开放** | `load_typed:22-25`在ensure后`asset.as_ref().clone()`；promotion再次`load_texture_asset`完整clone |
| 09D P0-6 无异步generation/cancel/fence终态 | **开放** | transition ID和revision检查已存在，纠正“完全无stale防护”；但`apply_texture_mip_streaming:425-455`同一循环同步完成，无I/O/GPU completion ticket |

这6项P0仍由09D与跨报告P0总表计数。本篇不重复累计；后续实现必须更新09D状态，而不是让新Texture facade掩盖旧阻断。

## 6. P1：Schema、Identity与Asset Contract

| ID | 当前差距 | 必须重构的工程合同 |
|---|---|---|
| TEX-P1-01 | `TexturePayload`只有`Rgba8`或opaque container `Vec<u8>` | typed canonical pixel/block/planar payload与明确subresource table；bulk bytes与元数据分离 |
| TEX-P1-02 | `format: String`晚解析 | 唯一`TextureFormatId`/typed format，携带numeric class、channel、transfer、block extent与capability requirement |
| TEX-P1-03 | `depth_or_array_layers`与`array_layer_count`双authority | `TextureExtent3D + TextureViewKind`单一shape真值，禁止normalize猜测冲突字段 |
| TEX-P1-04 | source、canonical、cooked、runtime和physical descriptor混装 | 五阶段不可变descriptor及显式转换receipt，任何relabel都必须有转换产物 |
| TEX-P1-05 | D1在schema可声明但RGBA8 upload直接拒绝 | 要么完成D1 artifact/upload/view/product test，要么在authoring/cook前硬拒绝，不得运行时晚失败 |
| TEX-P1-06 | ordinary D3/volume upload拒绝，3D LUT使用私有路径 | canonical volume texture generation覆盖slice/mip布局、format、upload和residency；LUT只是usage policy |
| TEX-P1-07 | array/cube assembly把来源展平为一段RGBA | 保留layer/face/source locator/orientation/crop/dependency/provenance与每subresource digest |
| TEX-P1-08 | `SvtSettings`注释宣称runtime page table consumer但实际无消费 | capability只有完整provider graph存在才可发布；缺consumer时schema/Editor必须标记unavailable |

## 7. P1：Format、Container、Mip、Compression与Cook

| ID | 当前差距 | 必须重构的工程合同 |
|---|---|---|
| TEX-P1-09 | compression target只有Auto/Uncompressed/BC族 | platform/device profile选择BC/ASTC/ETC及fallback variants，build key含codec/version/quality/RDO |
| TEX-P1-10 | KTX2 Zstd/Zlib已展开，但BasisLZ/ETC1S/UASTC没有transcoder | 在worker/cook期transcode并产生多平台artifact；render submission禁止临时解码 |
| TEX-P1-11 | KTX1多mip/array、ASTC 3D与DDS/KTX volume readiness矩阵不统一 | 一张可测试的container-format-dimension-subresource能力矩阵，unsupported在import/cook期终止 |
| TEX-P1-12 | compressed texture完整创建/上传且`mip_streaming_supported=false` | block-aligned mip/layer semantic blobs、partial upload与tail-first physical residency |
| TEX-P1-13 | offline mip链仍是level-major单一RGBA `Vec` | mip/layer/face独立索引、hash、offset、codec、alignment、priority与bootstrap tail |
| TEX-P1-14 | runtime mipgen只支持RGBA8 D2/Cube并在ensure中私有submit | 统一compute job由Render Graph/RHI queue owner调度，带budget、completion和format/color/normal policy |
| TEX-P1-15 | GPU capability到upload才决定是否可用 | cook按target capability选择artifact；runtime只在已声明fallback集合中选择，不现场猜格式 |
| TEX-P1-16 | lightmap RGBA16F、external cubemap、IBL等私有container绕过统一format authority | derived texture provider仍产同一typed artifact/generation，不允许私有raw format字符串成为长期ABI |

## 8. P1：GPU Resource、Sampler与Upload

| ID | 当前差距 | 必须重构的工程合同 |
|---|---|---|
| TEX-P1-17 | streamed tail仍保存source descriptor，physical base extent/mip count靠隐式offset解释 | 分离`TextureSourceDesc`、`TextureGenerationDesc`与`PhysicalTextureAllocationDesc`，view显式映射source mip |
| TEX-P1-18 | first load full allocation后再tail rebuild，old+new峰值不计 | bootstrap tail直接创建；reservation在create前覆盖CPU、staging、new、old-retired和driver overhead |
| TEX-P1-19 | RGBA/compressed/lightmap按mip/layer直接`queue.write_texture` | Runtime90唯一UploadService提供aligned staging pages、batch、priority、deadline和backpressure |
| TEX-P1-20 | write/submit无completion receipt，resident在提交后立即逻辑commit | upload ticket必须到queue completion/fence后才能publish resident generation，失败保持last-good |
| TEX-P1-21 | sampler只有UVW address和nearest/linear三滤波 | typed compare、LOD min/max/bias、border、reduction、anisotropy、unnormalized、static/immutable sampler合同 |
| TEX-P1-22 | sampler cache是无界`Mutex<HashMap<key, Arc<Sampler>>>` | device-generation scoped catalog、lock外创建/single-flight、count/bytes/TTL/retirement/diagnostics |
| TEX-P1-23 | LUT、irradiance、IBL、output和feature texture复制普通资源逻辑 | 统一TextureGeneration/View/Sampler，usage和lifetime policy不同但不复制上传与设备生命周期 |
| TEX-P1-24 | 86 create、83 write、24 copy分散在产品代码 | Runtime89/90下建立ImageService/UploadService入口；允许的低层调用点由结构guard白名单固定 |

## 9. P1：Streaming、Demand、Residency、Budget与Eviction

| ID | 当前差距 | 必须重构的工程合同 |
|---|---|---|
| TEX-P1-25 | `MipStreamingTask`在同一函数同步schedule/rebuild/finish | 独立request queue与I/O/decode/upload阶段，跨帧poll，render thread只消费ready install packet |
| TEX-P1-26 | task只有局部transition ID | ticket固定asset revision、artifact generation、device generation、request epoch、target range与cancel token |
| TEX-P1-27 | 每次promotion重新`load_texture_asset`并clone全量payload | 按semantic block lease读取缺失mip；single-flight合并waiter，CPU bulk按引用和预算回收 |
| TEX-P1-28 | demand只取main view visible mesh material texture | 汇总camera stack、shadow/reflection/probe、sprite/UI、lightmap、cookie、LUT、particle与预加载请求 |
| TEX-P1-29 | coverage用transform scale近似sphere | 消费统一world bounds、projected footprint、UV density/tiling、sampler derivative、anisotropy和texture resolution |
| TEX-P1-30 | promotion排序先于eviction，超预算promotion被跳过后至少延迟一帧 | budget controller先生成释放/保留/提升的原子计划，预留释放收益并避免promotion starvation |
| TEX-P1-31 | 固定16 transitions、32 MiB/frame、1 mip hysteresis | device/IO/quality/profile/viewport动态policy，按bytes/time/task slots与latency target自适应 |
| TEX-P1-32 | persistent texture bytes只合计普通`textures` | 全Texture pool统一记录ordinary、LUT、IBL、history、shadow、output、VT tile、staging、retired和rebuild峰值 |

## 10. P1：Lifecycle、Consumers、Diagnostics与Qualification

| ID | 当前差距 | 必须重构的工程合同 |
|---|---|---|
| TEX-P1-33 | prepared texture maps没有unused/TTL/scene unload/remove闭环 | resource event驱动release，lease/age/budget共同决定evict，GPU对象按fence retirement释放 |
| TEX-P1-34 | revision mismatch逐资源同步替换 | dependency closure后台构建并atomic publish generation；失败保留last-good并有retry/backoff |
| TEX-P1-35 | cookie/UI/LUT等多处吞掉`ensure_*`错误只记fallback count | stable request/error ID、原因链、retryability、fallback provenance与Editor/trace可见诊断 |
| TEX-P1-36 | sprite/UI/cookie/LUT/irradiance等虽被ensure却不进入普通demand/预算图 | 所有consumer声明bootstrap、quality、priority、pin和view-specific demand |
| TEX-P1-37 | material capture为一个texel同步加载并clone完整资产 | derived preview/capture从canonical CPU thumbnail或budgeted readback/cache采样，不穿透完整runtime load |
| TEX-P1-38 | device loss/recreate没有Texture generation重建协议 | device generation失效、bootstrap恢复、优先级重传、old handle拒绝与fallback连续性 |
| TEX-P1-39 | telemetry主要是最终bytes/fallback/warning | per-request stage/age、wanted/resident、I/O/decode/upload latency、cache、thrash、debt与top offender |
| TEX-P1-40 | 1 GiB reference预算与设备/项目/质量无关 | adapter heap/UMA、OS pressure、platform profile、quality tier和viewport组合产生reservation与降级梯 |

## 11. P1：Virtual Texture、Testing与跨Owner产品闭环

| ID | 当前差距 | 必须重构的工程合同 |
|---|---|---|
| TEX-P1-41 | SVT没有page cook、mip tail、page table、feedback与tile cache | 实现完整Sparse/Virtual Texture artifact和runtime，或移除stable capability声明 |
| TEX-P1-42 | 没有Runtime Virtual Texture producer/composition | scene producer、dirty region、dependency、page render、feedback/install与fallback mip统一调度 |
| TEX-P1-43 | 没有Sparse Volume Texture streaming | volume brick/page artifact、3D feedback、physical pool、indirection和时间序列/quality policy |
| TEX-P1-44 | 没有统一runtime atlas/array packing authority | 参考Unity atlas的allocate/update/release/mip validity，提供generation化slot和fragmentation/relocation合同 |
| TEX-P1-45 | 测试集中在parser、CPU planner与source guard | 增加真实GPU cold/warm、partial upload、completion、cancel、fault、device loss与image correctness测试 |
| TEX-P1-46 | cook/export不证明目标平台artifact可直接stream/upload | BuildSet gate验证格式支持、fallback、semantic block、alignment、package range与零运行时transcode |
| TEX-P1-47 | 无100k资产、快速相机、teleport、budget pressure和长时间soak | 固定场景、seed、设备、画质与p50/p95/p99/peak VRAM/RSS/I/O/blur/thrash证据 |
| TEX-P1-48 | 无可复现“优于Unreal”对照 | 同source、画质、分辨率、预算、设备、warmup、camera path、失败策略比较hitch、VRAM、I/O与视觉误差 |

## 12. P2 长期能力

| ID | 长期能力 | 前置条件 |
|---|---|---|
| TEX-P2-01 | DirectStorage/GDeflate或等价direct I/O + GPU decompression | semantic artifact、security、fallback、completion与预算闭合 |
| TEX-P2-02 | bindless/sparse descriptor residency与大规模descriptor compaction | Runtime90 descriptor/device generation和Texture view lifetime稳定 |
| TEX-P2-03 | GPU feedback去重、预测与自适应prefetch | P1 demand/telemetry基线可解释且可复现 |
| TEX-P2-04 | 多GPU/device-group texture replication与迁移 | 单GPU generation、budget、fence retirement完全正确 |
| TEX-P2-05 | UMA/discrete/memory-mapped零拷贝策略 | typed storage、ownership、cache coherence与平台证据 |
| TEX-P2-06 | distributed/remote DDC semantic block QoS | immutable build key、可信artifact、租户/权限/eviction |
| TEX-P2-07 | UDIM/tiled texture set与material tile dependency | stable tile identity、material binding、partial rebuild和VT |
| TEX-P2-08 | neural texture compression | 传统codec基线、model/version provenance、设备fallback和帧预算 |
| TEX-P2-09 | texture transcoding hardware/offline quality autotuning | 可复现视觉metric、platform profile与cook determinism |
| TEX-P2-10 | thermal/power-aware long-session quality control | 完整telemetry、deterministic floor和用户policy |
| TEX-P2-11 | cross-process shared physical tile/upload cache | security、device isolation、generation和crash recovery |
| TEX-P2-12 | 超越Unreal的长期Texture benchmark与自动回归判定 | P1-48公平基线先成立，结果跨多设备/驱动/场景稳定 |

## 13. 五引擎差异证据

### 13.1 Unreal Engine

`FRenderAssetUpdate`明确区分Game、Render与Async thread task，持有同步、abort/cancel、scheduled task和完成状态；Texture stream-in按mip分配、锁定、I/O、复制shared mip、创建intermediate resource并等待RHI阶段。`StreamingManagerTexture`维护view/instance数据、prioritized assets、memory pool/margin、max temp memory、async mip calculation与为请求释放内存的stream-out。Unreal还提供volume texture streaming、Virtual Texture upload/chunk/page系统和Sparse Volume Texture manager。Zircon的同步whole-asset rebuild与单个transition ID未达到这条最低工程线。

### 13.2 Bevy

Bevy用extract/prepare分离MainWorld与RenderWorld asset lifecycle，处理added/modified/removed/unused事件，`PrepareAssetError::RetryNextUpdate`表达依赖未就绪，render asset bytes-per-frame limiter提供上传节流；`TextureCache`按frame保留、复用和淘汰compatible texture。它不是Unreal式高级streamer，但已证明Zircon至少要先有异步准备、remove/unused与统一cache lifetime，而不是永久prepared map。

### 13.3 Godot

Godot `TextureStorage`集中allocate、initialize、free、update、replace、proxy与invalidation，区分2D/array/3D/cube，提供partial update、shared slice、path/name和memory信息，并统一render target相关texture ownership。Zircon的ordinary texture、LUT、IBL、output和feature texture分叉还没有达到这一较低但完整的storage基线。

### 13.4 Fyrox

Fyrox ResourceManager使用异步request/loading共享状态，TextureCache按resource event、modification和TTL更新/移除。它同样不是性能上限，但证明`load_typed -> clone full asset -> permanent GPU map`不能作为工程级资源生命周期终点。

### 13.5 Unity Graphics

当前本地Graphics包中，`Texture2DAtlas`具备allocate/release/update/needs-update和mip validity；VT settings表达CPU cache、按format的GPU cache与mip preload，shader/compute文件形成feedback下采样和virtual texturing采样合同。由于缺Unity native renderer，本文只把这些作为atlas/VT产品surface证据，不推断其内部streamer性能。

## 14. 目标架构与唯一所有权

### 14.1 纵向产品链

```text
TextureSourceSnapshot + TextureImportRecipe + TargetDeviceProfile
  -> Runtime85 TextureBuildService graph
  -> TextureArtifactManifest
       { typed desc, build/provenance key, variants, mip/layer/face/slice/page blocks, bootstrap tail }
  -> Runtime64 semantic block lease + request coalescing
  -> TextureGenerationService
       { asset/artifact/device generation, views, sampler, dependencies, last-good }
  -> TextureResidencyService
       { multi-view demand, priority, budget reservation, async stage machine, eviction }
  -> Runtime90 TextureUploadService + completion/fence receipt
  -> Runtime89/91 Scene/RenderGraph/Material typed TextureViewHandle
```

### 14.2 必须固定的核心类型

| 类型 | 唯一责任 | 禁止承载 |
|---|---|---|
| `TextureFormatId` | typed channel/numeric/transfer/block/capability identity | 任意用户字符串、容器名猜GPU format |
| `TextureSubresourceId` | mip/layer/face/slice/page的versioned identity | 从裸offset或Vec顺序反推 |
| `TextureArtifactManifest` | platform variant、semantic blocks、digest、alignment、bootstrap tail | WGPU对象、运行时指针 |
| `TextureGenerationId` | asset + artifact + device generation | 单一revision或局部transition ID替代 |
| `TextureResidencyTicket` | demand、stage、cancel、deadline、reservation与completion | render线程同步执行work |
| `TexturePhysicalAllocation` | source-to-physical mip mapping、bytes、views与retirement fence | 复用source descriptor隐式表达tail |
| `SamplerStateId` | resolved immutable sampling policy与device generation | compression或pixel payload |

### 14.3 状态机与预算规则

```text
Absent
  -> Requested
  -> BlockQueued
  -> Reading
  -> Decoding/Transcoding (cook-only or explicitly budgeted fallback)
  -> UploadReserved
  -> UploadSubmitted
  -> CompletionPending
  -> ResidentBootstrap / ResidentTarget
  -> EvictRequested
  -> Retiring
  -> Absent

任何阶段 -> Cancelled / Superseded / Failed(last-good retained)
device generation变化 -> RecreateQueued，不得把旧handle继续发布为ready
```

预算至少区分compressed block cache、decoded CPU、upload reserved/in-flight、ordinary persistent、special persistent、VT tile、sampler/descriptor、replacement overlap和retired GPU。所有reservation在I/O或GPU create前完成；逻辑eviction不能在fence释放前提前返还物理bytes。

## 15. 依赖顺序与重构里程碑

| 里程碑 | 内容 | 依赖/owner |
|---|---|---|
| M0 | current behavior characterization：固定schema/upload/container/planner调用链和删除清单 | Runtime92，review-only转实现前重扫 |
| M1 | typed Texture schema与五阶段descriptor，硬切string format/双extent authority | Runtime86 + Plugins18 + Editor35 |
| M2 | platform Texture build graph与semantic artifact blocks，tail与variant可随机读取 | Runtime85 + Runtime64 |
| M3 | Runtime90统一TextureUploadService、staging、completion、device generation与retirement | Runtime90；Runtime92只提交request |
| M4 | TextureGeneration/SamplerCatalog统一ordinary与special texture；删除分叉upload | M1-M3 + Runtime89/91 |
| M5 | 异步TextureResidencyService、multi-view demand、全pool budget与eviction | M2-M4 + visibility/bounds owner |
| M6 | compressed tail-first、array/cube/volume与device-loss/hot-reload闭环 | M5 |
| M7 | SVT/RVT/Sparse Volume Texture完整artifact、feedback、page/tile runtime | M2-M6，不能先建metadata facade |
| M8 | Editor/Plugin/product/cook/export与Unreal对照资格 | Editor35 + Plugins18 + default product profiles |

M0-M6是普通工程级Texture系统必需项，不应因VT属于长期能力而延后。M7必须在普通资源lifecycle、budget与upload owner稳定后开始；不得把whole-texture rebuild改名为virtual texture。

## 16. 资格门

| Gate | 必须形成的证据 |
|---|---|
| G01 | source/canonical/cooked/runtime/physical descriptor类型互不混装，转换均有receipt |
| G02 | 全生产路径无任意string到GPU format的晚解析 |
| G03 | D1/D2/D3/array/cube/cube-array能力矩阵在import/cook期确定 |
| G04 | duplicated extent/layer authority为0，invalid shape不能normalize成另一语义 |
| G05 | HDR/float/integer/block-compressed数据round-trip保持格式与误差边界 |
| G06 | array/cube/volume每subresource保留source/provenance/digest |
| G07 | mip/filter/color/normal/alpha处理有golden image与边缘/seam证据 |
| G08 | BC/ASTC/ETC/Basis平台variant与fallback由TargetDeviceProfile确定 |
| G09 | KTX2 Zstd/Zlib/Basis支持矩阵明确；unsupported在cook期失败 |
| G10 | artifact manifest可按mip/layer/page随机读取且不反序列化完整TextureAsset |
| G11 | bootstrap tail独立可读、可校验、可package range request |
| G12 | cold首次可见不读取或上传full chain，除显式fully-resident policy |
| G13 | render submission调用栈无文件I/O、Zstd、bincode、image decode/transcode |
| G14 | render submission无Texture device create/write的同步等待路径 |
| G15 | upload staging按alignment和row pitch验证，batch与backpressure可观察 |
| G16 | upload只在completion receipt后publish resident generation |
| G17 | cancel/supersede/failure/device-generation stale ticket不能安装 |
| G18 | old/replacement/in-flight/retired bytes完整计入预算直到fence |
| G19 | compressed 2D/array/cube partial mip residency正确且不读取整资产 |
| G20 | ordinary volume texture支持真实slice/mip upload与view，不依赖LUT私有路径 |
| G21 | sampler compare/LOD/border/reduction/anisotropy capability完整或显式拒绝 |
| G22 | sampler cache有single-flight、device generation、TTL/retirement与诊断 |
| G23 | ordinary/LUT/IBL/lightmap/history/output等都使用同一generation/upload基础设施 |
| G24 | 直接texture create/write/copy调用点仅存在于审核白名单owner |
| G25 | multi-view demand覆盖main/secondary/shadow/reflection/probe/UI/sprite/particle |
| G26 | demand使用真实bounds、UV density/tiling、resolution和camera prediction |
| G27 | camera cut/teleport可预取或受控降级，无无界hitch与长时间模糊 |
| G28 | promotion/eviction在单一原子预算计划中，无可证明的starvation |
| G29 | device/IO/quality/profile决定动态bytes/time/task预算 |
| G30 | 全Texture pool reserved/committed/in-flight/retired总账与实际GPU证据相符 |
| G31 | scene unload/resource removal/unused event最终回收CPU、GPU、sampler和descriptor |
| G32 | hot reload dependency closure atomic swap，失败保留last-good且退避 |
| G33 | device loss后按priority恢复bootstrap，旧handle被generation拒绝 |
| G34 | OOM/pressure触发确定性降级、debt、recovery，不只增加warning |
| G35 | diagnostic可从Texture ID追踪build block、ticket stage、GPU allocation与consumer |
| G36 | top offender、I/O/decode/upload latency、thrash与fallback reason进入trace/Editor |
| G37 | material capture/preview不为单texel同步clone完整runtime asset |
| G38 | cook/export验证目标平台所有Texture artifact可直接stream/upload |
| G39 | package range、hash failure、truncated block、corrupt codec有fault测试 |
| G40 |真实GPU覆盖RGBA/float/BC/ASTC、2D/array/cube/volume与partial update |
| G41 | 1/1k/100k texture规模下request、descriptor、cache与CPU frame time有界 |
| G42 | 4K/8K/16K cold/warm traversal记录p50/p95/p99 I/O、upload与hitch |
| G43 | budget pressure soak无永久debt、无重复I/O风暴、无mip thrash |
| G44 | hot reload、project switch、device loss、OOM后无stale publication/leak |
| G45 | SVT capability只有page cook/page table/feedback/tile pool/fallback全链存在时发布 |
| G46 | RVT和Sparse Volume Texture若对外可见，具备独立产品与failure evidence |
| G47 | 默认Client与Editor Host从preset到真实textured frame、reload、unload均通过 |
| G48 | 同画质、设备、预算、camera path的Unreal对照可复现，且结果达到声明阈值 |

## 17. 禁止的临时实现

- 禁止给`TexturePayload`继续增加更多opaque `Vec<u8>`标签而不建立semantic subresource manifest。
- 禁止保留string format并在每个consumer中新增`match str`或默认映射。
- 禁止把D3 Texture继续只实现为post-process LUT特例。
- 禁止在render submission、material prepare或capture路径同步读、解压、转码或clone完整纹理。
- 禁止先full resident再同帧rebuild tail，并把最终bytes当作峰值预算。
- 禁止把同步`for task in tasks`改名为async而没有跨帧ticket、cancel与completion。
- 禁止为Texture新建第二套task pool、DDC、RHI queue、device recovery或Render Graph。
- 禁止特殊feature各自持有永久Texture cache且不进入统一预算、remove和retirement。
- 禁止只增加`SvtSettings`字段或Editor开关就宣称支持Virtual Texture。
- 禁止用parser/unit test数量替代真实GPU、产品、fault、soak与公平benchmark证据。
- 禁止保留旧路径作为compat facade、re-export或双写镜像；硬切后删除旧owner。
- 禁止在没有公平对照数据时写“优于Unreal”的完成声明。

## 18. 本轮输出边界

本篇完成Runtime Texture/Image/Cubemap/Array/Volume/Format/Sampler/Mip/Compression/Upload/Streaming/Residency/Budget/Eviction/Virtual Texture的当前源码E3静态审查，未实施production重构。旧09D的6项P0仍开放；KTX2 Zstd/Zlib、BC5、mip planner、artifact integrity和预算接线等已存在基础已被纠偏记录。Runtime92新增的48项P1、12项P2与G01-G48专门约束Texture领域纵向闭环，实施必须复用Runtime64/85/86/89/90/91、Editor35与Plugins18的唯一owner。

当前判定是：Zircon已经拥有若干可迁移的Texture算法和校验内核，但asset形态、semantic artifact、异步upload、generation、全consumer demand、全pool budget、eviction、device recovery、VT/SVT与产品资格仍未达到工程级。后续应先完成M0-M6的普通Texture主路径，再进入M7高级虚拟化；在G01-G48全部形成BuildSet-bound与真实产品证据前，不得把本报告标记为implemented，也不得宣称达到或超过Unreal。
