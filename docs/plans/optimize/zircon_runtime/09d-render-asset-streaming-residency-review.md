---
related_code:
  - zircon_runtime/src/graphics/scene/resources
  - zircon_runtime/src/graphics/runtime/render_framework/budget/memory_budget.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/asset/artifact
  - zircon_runtime/src/asset/assets/mesh
  - zircon_runtime/src/asset/assets/model
  - zircon_runtime/src/asset/assets/texture
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading
  - zircon_plugins/asset_importers/model
  - zircon_plugins/asset_importers/texture
  - zircon_plugins/gltf_importer
  - zircon_plugins/obj_importer
  - zircon_plugins/texture_importer
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/performance/01/2026-08-15-runtime-asset-project-registry-pipeline-current-architecture-review.md
  - docs/plans/performance/01/2026-08-15-renderer-material-shader-streaming-current-architecture-review.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/RenderAssetUpdate.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/Texture2DStreamIn.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/StreamingManagerTexture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/StaticMeshUpdate.cpp
  - dev/bevy/crates/bevy_render/src/render_asset.rs
  - dev/bevy/crates/bevy_render/src/mesh/allocator.rs
  - dev/godot/servers/rendering/renderer_rd/storage_rd/texture_storage.cpp
  - dev/godot/servers/rendering/renderer_rd/storage_rd/mesh_storage.cpp
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Fyrox/fyrox-impl/src/renderer/cache/texture.rs
  - dev/Fyrox/fyrox-impl/src/renderer/cache/geometry.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/ShaderLibrary/DebugMipmapStreaming.hlsl
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/VirtualTexturingSettingsSRP.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: m1_semantic_manifest_store_texture_mesh_cook_m2_shared_priority_manifest_block_frontier_atomic_batches_semantic_route_cpu_lease_m3_neutral_rhi_batch_upload_single_residency_authority_bounded_completion_retirement_poll_receipt_bounded_retirement_admission_device_epoch_recovery_static_validation_only
source_recheck_required: true
---

# 09D · Render Asset Streaming / Residency / Upload / Eviction 工程化差距

## 1. 结论

Zircon当前不是完全没有streaming基础。`ArtifactStore`已经把压缩对象拆成64 KiB content-addressed chunk，校验manifest/hash/size并维护64 MiB chunk residency cache；texture mip planner具备wanted/resident range、优先级、每帧16次transition、32 MiB upload预算、1 mip hysteresis、mip bias、tail保留、确定性排序、预算紧急驱逐和stale transition拒绝。GPU texture路径支持RGBA8、cube、lightmap、DDS/KTX/KTX2/ASTC等容器布局校验，Mesh/Model/Material/Texture也有revision-aware prepared map。这些局部机制应该保留并迁入统一系统，不能退回每次draw直接读文件或全量重建。

但名为`ResourceStreamer`的产品路径并不是工程级render-asset streamer。每帧`ensure_scene_resources`在renderer提交路径内同步查询revision、同步从artifact读取/解压/bincode反序列化完整资产、clone完整CPU payload、转换Mesh、创建WGPU buffer/texture/bind group，然后才运行mip planner。首次texture总是先完整驻留；同帧的“驱逐”又可能重新创建较小的物理texture。promotion/eviction会再次加载完整`TextureAsset`，创建另一张texture、GPU copy公共mip并写入缺失mip。预算只计算最终逻辑resident texture bytes和promotion源字节，不计算完整I/O、decode clone、staging、old+new过渡峰值、in-flight submission和fence retirement，因此“未超预算”不能证明真实内存受控。

streaming粒度也与artifact形态冲突。64 KiB chunk只切分压缩后的完整bincode对象，不携带texture mip/layer/tile或mesh LOD/cluster/page语义索引；读取一个mip仍需要顺序读取、zstd解压并反序列化整个资产。`TextureAsset`把完整RGBA或container bytes存在单一`Vec<u8>`，`ModelAsset`和`MeshAsset`在glTF/OBJ/model importer中重复嵌入或派生几何；运行时又保留CPU asset、转换后的临时vertex `Vec`、独立GPU buffer和wire segment。已有`ResourceLease`能够表达CPU residency，但graphics使用返回clone的`load_*_asset`，prepared maps又没有产品级remove/retire，因此资产管理器和renderer各自长期持有一份或多份数据。

当前mip demand只来自主视图可见mesh material texture，用transform translation和`scale.length() * 0.5`估算屏幕覆盖。它没有使用真实bounds、UV density/tiling、occlusion与camera history，也不覆盖camera stack/secondary view、sprite、UI、lightmap、cookie、LUT、environment、probe和output target。压缩texture只能完整上传，不能做物理mip streaming；Mesh没有LOD/cluster/page residency，GPU buffer是一primitive一allocation，没有arena/suballocation/partial update；Material dependency也在准备时同步触发texture读取。结果是局部算法看起来像streaming，产品行为仍是“需要时同步完整加载并永久缓存”。

本轮登记6项P0、15项P1、6项P2。P0先建立semantic artifact、唯一residency authority、异步ticket状态机、全资源预算与fence retirement、初始tail residency、CPU/GPU ownership闭环；P1再补完整demand、压缩纹理、mesh arena/LOD、import/cook唯一owner、dirty generation、device recovery、diagnostics和shipping gate；P2才进入virtual texture、GPU decompression/direct storage、多GPU/UMA策略和大规模远程cache。完成cold/warm/traverse/teleport/reload/device-loss/OOM/fault/soak矩阵前，不能声称render asset streaming达到Unreal级，更不能声称hitch、RSS、VRAM或I/O吞吐优于Unreal。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 子域 | Rust文件 / 物理行 | 本轮判定 |
|---|---:|---|
| graphics scene resources | 72 / 11,222 | E3：prepared map、texture/mesh/model/material ensure、mip demand、GPU upload与accessor |
| texture importer | 39 / 9,301 | E3：image/PSD/container、mipgen、BC5 transcode、settings与plugin registration |
| artifact store/cache payload | 17 / 5,030 | E3：manifest/chunk/residency、zstd+bincode read/write与typed payload |
| texture assets/upload support | 23 / 5,733 | E3：descriptor/payload/container layout/readiness/cubemap/lightmap |
| mesh/model assets | 23 / 2,361 | E3：CPU layout、conversion、bounds、SDF、usage和serialization |
| glTF importer | 6 / 3,028 | E3：source import、geometry/image/material/model/mesh subasset与placeholder |
| OBJ/model importers | 12 / 1,832 | E3：parse、root model、mesh subasset、VG/SDF cook与registration |
| legacy texture importer surface | 3 / 290 | E3 descriptor；E0 production registration/implementation |
| ProjectAssetManager loading | 7 / 639 | E3：ensure resident、clone load与lease acquire |
| render budget/submission/lease spot checks | 3 / 725 | E3：state lock、hardcoded budget、lease drop/unload contract |
| 合计 | 205 / 40,161 | 341个inline test属性；focused source fingerprint `e5aab8e2e05c7de487b7a563b962d01a1099c72643d9d24fe7c25438b1781291` |

“E3”表示读到实现、调用链和失败/生命周期语义，不表示完成动态验证。统计包含与实现同文件的局部tests；大规模独立test目录只用于发现产品契约，未把test行数计入上述focused physical scope。

### 2.2 与既有审查的归属

- 04负责通用AssetId、registry、artifact identity、lease和serialization；09D只定义render subresource residency与GPU准备边界。
- 09A负责RHI resource handle、queue、submission fence、device generation和deferred destruction；09D消费这些能力，不私建第二套GPU lifetime。
- 09B负责RenderScene、view visibility、Virtual Geometry cluster/page demand；09D消费统一demand，不在texture streamer重算另一套可见性。
- 09C负责Material/Shader/PSO generation；09D只拥有prepared material dependency residency和binding资源，不编译Shader。
- `docs/plans/performance/01`已登记同步load/deep clone与prepared generation目标；本篇补足逐资源证据和可实施hard-cutover顺序，不复制通用Asset pipeline计划。

### 2.3 参考引擎边界

- Unreal `FRenderAssetUpdate`明确建模current/pending LOD、thread task、cancel/suspend/abandon、同步点和资源更新终态；`Texture2DStreamIn`按new mip分配/锁定/复制shared mip，并等待RHI completion。Zircon的单一pending transition ID只能防一次stale finish，不能替代完整I/O/RHI状态机。
- Bevy `RenderAsset`提供added/modified/unused提取、prepare-next-frame重试、每帧byte budget、remove与device-recovery重提取；`take_gpu_data`允许把大像素/vertex数据移出Main World。它仍以whole asset为主，是Zircon应越过的下限，不是partial residency目标。
- Godot RD texture/mesh storage提供显式allocate/free/update/replace/proxy和buffer region update，说明即使不做Unreal式streaming，也必须有可观察的resource lifecycle和partial update owner。
- Fyrox ResourceManager有异步任务池和unused-resource超时销毁；Texture/Geometry cache按modification counter刷新并通过temporary cache回收。它也是最低生命周期基线，不足以作为最终规模目标。
- 当前仓库的Unity Graphics只包含SRP侧mip streaming debug、Shader Graph feedback与Virtual Texturing设置/consumer，不包含Unity native core texture streamer实现。报告仅引用其debug/feedback/config surface，不根据缺失源码臆造内部调度。

### 2.4 明确未做

本轮没有改production code，没有运行Cargo、Editor、cook、真实GPU、PIX/RenderDoc、WPR、设备丢失、OOM、I/O fault、跨平台、开放世界穿越或soak。静态审查不能证明当前路径在任何设备达到预算，也不能给出与Unreal的性能胜负结论。

## 3. 当前必须保留的基础

### 3.1 artifact chunk integrity和cache可迁入semantic store

manifest schema、BLAKE3 content address、chunk hash/size校验、atomic staging publication、bounded chunk cache和diagnostics都是正确方向。重构应在manifest上增加semantic range/index与async reader，而不是删除content-addressed chunk层。

### 3.2 mip planner已有可测试的policy kernel

wanted/resident range、tail、hysteresis、priority、transition/upload/resident预算、稳定排序、紧急驱逐和stale completion拒绝可以保留为纯policy。需要替换的是它的输入真值、异步执行与预算会计，不是把算法重新散落到每个texture consumer。

### 3.3 upload readiness和container layout校验值得保留

DDS/KTX/KTX2/ASTC布局、block尺寸、feature capability、cube/layer/mip范围和lightmap格式验证已经阻止大量非法upload。它们应成为cook manifest与runtime admission的共享validator，不能只在完整payload到达render thread后才发现不支持。

### 3.4 ResourceLease和prepared revision提供迁移挂点

资产层已有lease acquire/unload，prepared对象已有revision；这允许通过generation-aware render lease和event-driven invalidation逐步替换clone/poll，而无需新增另一个root resource manager。

## 4. P0 差距清单

### P0-1：render submission路径同步执行完整I/O、解压、反序列化、转换与GPU创建

`ensure_scene_resources`由renderer frame调用，逐个visible mesh/material/texture/model执行`resource_revision`和`load_*_asset`。cache miss会进入`ensure_resident`，同步读取chunk、zstd解压、bincode反序列化；随后Mesh转换、bounds/SDF seed、buffer/texture/bind group创建也在同一路径发生。`submit_runtime_frame_locked`持有`RenderFrameworkState` guard穿过renderer调用，这会把asset I/O和driver工作扩大为framework锁域hitch。

必须把请求变成non-blocking typed ticket：render thread只读取上一代`PreparedRenderAssetGeneration`并提交demand；I/O/decode/transcode走Runtime11 task lanes，RHI-affine upload走09A queue owner。稳定帧render-thread filesystem/decompress/asset clone/GPU create必须全部为0。

### P0-2：artifact chunk没有subresource语义，读取一个mip/LOD仍反序列化整个对象

`ArtifactStore::read`用`ChunkReader -> zstd decoder -> bincode`顺序恢复单个`ArtifactCacheAsset`。chunk描述符只有hash和大小，没有texture mip/layer/tile、mesh LOD/cluster/page或material dependency block的offset/codec/priority。`TexturePayload::Container`和RGBA都保存在完整`Vec<u8>`；generic 64 KiB切块只改善去重和cache，不构成semantic streaming。

必须新增versioned `RenderArtifactManifest`：header/metadata独立常驻，bulk blocks按subresource建立content ID、compressed/uncompressed bytes、alignment、codec、platform format、dependency和range。reader必须按block随机异步读取和校验；禁止以反序列化完整资产作为取得单个mip或LOD的前置条件。

### P0-3：初始完整驻留与物理texture重建绕过真实预算

`ensure_texture`无条件`GpuTextureResource::from_asset`并写入`PreparedTexture::fully_resident`，之后才清理mip state。若预算要求tail，同帧再创建较小texture并复制公共mip。promotion/eviction也重新加载完整asset、创建replacement texture、copy、queue write和替换Arc。当前32 MiB upload预算只看新mip source bytes，persistent budget只看最终resident mip bytes；old texture、replacement、CPU full payload、decode scratch、driver allocation、command/fence backlog没有入账。

初次请求必须直接从tail/bootstrap blocks建立最小物理驻留；promotion只读取所需blocks。所有过渡先reserve `compressed IO + decoded CPU + staging + destination + retained source + command/fence`峰值，完成fence后才释放旧代。无法reservation时延迟或降级，不能先超配再发warning。

### P0-4：没有唯一跨texture/mesh/material的residency authority和驱逐闭环

`ResourceStreamer`持有models、meshes、materials、textures、output targets、LUT、shaders和sampler maps，但产品源码只对mip visibility/state做clear/remove；没有按scene/resource unused、age、lease、budget或device generation回收prepared map。只有texture mip有预算，mesh buffers、CPU assets、material uniforms/bind groups、samplers、wire data、SDF seed和output resources均不参与统一会计。

必须由一个`RenderAssetResidencyManager`拥有request state、CPU block leases、RHI handles、last use、pin reason、priority、bytes、generation和eviction。各资源类型提供policy与prepare adapter，不能各建永久HashMap。驱逐顺序必须区分降mip/LOD、释放derived cache、释放CPU source、释放GPU对象，并用fence retirement闭环。

### P0-5：CPU/GPU ownership断裂，clone load绕过已有lease并制造多份完整数据

graphics调用`load_texture_asset/load_mesh_asset/load_model_asset`，最终`load_typed`执行`asset.as_ref().clone()`；它不持有`ResourceLease`。prepared Mesh/Model又保留`Arc<MeshAsset/ModelAsset>`与GPU对象，Model resolve还clone primitives和external meshes。`asset_usage`在descriptor/cache/test中可见，但产品端未用`MainWorld/RenderWorld`决定移动、保留或丢弃bulk CPU数据。

必须建立render lease：metadata与bulk block分离，GPU upload完成后按usage/pin policy释放decode buffer和CPU bulk；Editor可显式pin source，shipping默认只保留需要的metadata/physics/collision部分。禁止GPU准备依赖`Clone`完整资产，也禁止“无lease但manager resident + prepared Arc”双重永久持有。

### P0-6：异步生命周期没有generation、cancel、failure、device和fence终态

`MipStreamingState`只记录一个`transition_id`和wanted range；实际工作仍同步，因此尚未面对I/O完成后asset revision变化、camera demand撤回、device loss、queue failure、OOM、shutdown、hot reload和old resource仍被GPU引用的问题。prepared key只有resource revision，没有device generation；替换Arc也不等于GPU完成后安全销毁。

统一状态机至少需要`Unrequested -> QueuedIo -> Reading -> Decoding -> ReadyCpu -> QueuedUpload -> Uploading -> Resident -> Evicting/Failed/Cancelled`，并携带AssetGeneration、DemandGeneration、DeviceGeneration、ticket和deadline。每个异步完成必须验证全代身份；hot reload保持last-good，new generation原子发布，old generation交给09A fence retirement。

## 5. P1 差距清单

### P1-1：mip demand只覆盖主视图visible mesh material texture

当前收集点没有统一view family或resource demand bus。camera stack/secondary/reflection/shadow view、sprite、UI、lightmap、cookie、LUT、environment、probe、particle和output target不共享同一优先级/预算语义。需要由09B输出每view typed demand并合并pin/boost/deadline；不同资源类别定义bootstrap和fallback，不得默认“不在mesh material中即无需求”。

### P1-2：屏幕覆盖估算没有真实bounds、UV density、tiling与预测

用translation加`scale.length() * 0.5`不能代表skinned/deformed/model bounds，screen coverage也没有texture resolution、UV chart density、material tiling、anisotropy或texel ratio。没有camera velocity、cut/teleport、prestream volume、cinematic boost、occlusion history和priority aging，容易产生过清晰浪费、模糊滞后与来回thrash。需求应基于09B统一bounds、projected size和material UV metrics，并提供预测窗口和显式preload ticket。

### P1-3：压缩texture不支持partial physical residency

当前physical mip rebuild只支持uncompressed RGBA8 2D/cube；DDS/KTX/KTX2/ASTC虽然可完整upload，却不能promotion/eviction。大量生产纹理因此要么保持完整压缩resident，要么走RGBA8并增加VRAM/带宽。应按block-compressed mip/layer建立semantic blocks和partial upload，平台cook选择BC/ASTC/ETC等格式；KTX2 supercompression/transcode必须在cook或预算化worker完成，不能在render submission临时解码。

### P1-4：Mesh是一primitive一buffer，没有arena、LOD/cluster/page residency与partial update

`GpuMeshResource::from_asset`建立临时interleaved vertex `Vec`，用`create_buffer_init`创建独立vertex/index buffer，并遍历全部属性/索引生成signature和CPU wire segment。没有buffer allocator/suballocation、compaction、free list、update region、LOD stream或budget。应消费Bevy allocator/Godot region update作为最低参照，并与09B Virtual Geometry owner汇合：传统Mesh走LOD+arena，VG走cluster/page block，不能建立第二套VG streamer。

### P1-5：Model/Mesh importer和runtime conversion重复几何authority

glTF root Model保留primitives，又生成per-mesh Model和per-primitive Mesh subasset；OBJ/model importer同样先构造embedded primitive，再从它派生Mesh并写回reference。运行时Model resolution再clone primitive/external Mesh，Mesh又`to_model_primitive`扩展数据。需要cook阶段选择唯一geometry artifact owner，Model只引用stable Mesh/LOD/cluster IDs；迁移后禁止root model、mesh model和MeshAsset同时携带相同顶点索引。

### P1-6：glTF内嵌texture绕过canonical texture importer/cook settings

`add_gltf_texture_subassets`直接从decoded image调用`TextureAsset::new_rgba8`，没有经过texture importer的color-space、mip filter/policy、compression、normal-map BC5、asset usage和platform cook。它产生的subasset与独立图片导入不等价。所有embedded/external texture都必须进入同一texture cook service，并把source locator、sampler、color semantic和material slot传入，不允许格式路径改变runtime artifact质量。

### P1-7：texture importer存在重复且含糊的plugin ownership

`zircon_plugins/texture_importer`拥有实际FunctionAssetImporter和完整实现；`zircon_plugins/asset_importers/texture`主要发布image/container/PSD descriptor，没有对应production register实现。catalog/manifest/历史计划因此可能显示两个“texture importer”能力，但只有一条产品链。应决定canonical owner、迁移manifest和测试后硬删除旧descriptor surface，避免导入优先级与设置schema漂移。

### P1-8：稳定资源仍轮询revision和Model dependency状态

每帧ensure都查询registry revision；Model还逐dependency查询/load以构造state。稳定帧成本随visible resource/dependency增长，而不是随changed frontier增长。应由AssetCatalogGeneration发布typed added/modified/removed/failed events和reverse dependency closure，residency manager只消费dirty generations；poll接口保留调试用途，不能作为产品主循环。

### P1-9：Material readiness同步读取texture且没有共享dependency ticket

material prepare会逐slot读取texture以判断upload readiness，随后texture ensure可能再次读取/clone；同一texture被多个material请求时缺少single-flight block/decode/upload ticket。应让MaterialGeneration只引用texture residency handles与required bootstrap state，所有waiter合并到一个request；material可以last-good/fallback发布，不能把完整texture load嵌进ABI验证。

### P1-10：budget是硬编码参考值和warning，不是设备/质量/平台控制器

`RenderMemoryBudget`固定512 MiB transient texture、256 MiB transient buffer、64 MiB staging、1 GiB persistent texture，只累计四个warning。它没有adapter heap/UMA/discrete分类、OS pressure、quality tier、viewport count、mesh/material/PSO、transition peak或外部占用，也不会根据压力执行策略。预算应来自设备能力、项目/平台profile和运行时pressure，分pool reservation/commit/debt，并提供确定性的降级与恢复。

### P1-11：upload没有共享staging ring、batch与submission completion会计

RGBA/compressed upload按mip/layer直接`queue.write_texture`，Mesh用`create_buffer_init`，runtime mipgen还可单独submit encoder。没有统一upload packet、aligned staging page、copy batching、in-flight bytes、queue ticket、bandwidth/CPU time或backpressure。09A应提供UploadQueue；09D只提交immutable upload plans，按frame/bytes/time/deadline调度并在fence回调后commit residency。

### P1-12：hot reload不是dependency closure的原子last-good切换

revision mismatch会同步重建并直接替换单个prepared对象；Model、Mesh、Material、Texture和Shader dependency没有同一发布generation。中途失败可能让一部分资源新、一部分旧，或反复每帧重试。应先在后台准备完整dependency closure，验证layout/capability/budget后一次发布；失败保留last-good并指数退避/显式retry，不得稳定帧无限触发相同I/O。

### P1-13：resource removal、scene unload和device rebuild没有产品合同

prepared maps缺少unused event和remove API，sampler cache也没有device generation。即使整个renderer重建可能间接清空对象，垂直资源链没有“旧device handles全部失效、仍需资源重新extract、未用资源释放”的显式证据。应定义asset removed、scene/world unload、renderer shutdown、device loss/recreate各自的cancel、retire、re-extract和deadline。

### P1-14：cook/export没有保证shipping artifact可直接按平台stream

当前容器、RGBA和runtime mipgen都可能在运行时决定处理，KTX2 supercompression可到runtime才报告unsupported。发布流程必须为每个target输出semantic block manifest、GPU-ready format、bootstrap tail/LOD、alignment和hash，并验证required dependencies完整。shipping默认禁止source image/model parse、runtime mip generation和不可预算transcode；缺artifact应在cook失败，而非玩家首帧失败。

### P1-15：diagnostics不能解释hitch、模糊、超预算和thrash

现有统计偏向最后一帧resource count、fallback count、persistent texture bytes和chunk cache命中。缺request state/age/reason、wanted-vs-resident、per-pool reserved/committed/in-flight/retired、I/O/decode/upload latency、cancel/stale、eviction reason、re-request distance、budget debt和top offenders。需要统一stable ID到Editor/trace/debug view；Unity Graphics的mip debug surface只能作为可视化参考，不能替代CPU状态证据。

## 6. P2 差距清单

### P2-1：没有virtual texture feedback/page table/physical tile cache

普通mip streaming完成后再评估SVT/VT：GPU feedback去重、page request压缩、physical tile pool、page table更新、border/filter、fallback mip和feedback latency。不能把现有whole-texture rebuild改名为virtual texture。

### P2-2：没有GPU decompression和DirectStorage/GDeflate类I/O后端

semantic block/ticket/budget稳定后，可增加backend-neutral bulk I/O与GPU decode extension；fallback必须是async CPU decode。不得让Windows专用路径污染AssetId或成为唯一可用实现。

### P2-3：没有UMA/discrete/memory-mapped零拷贝策略

同一CPU/GPU retention policy不适合UMA、discrete和移动tile GPU。未来按memory architecture选择upload/retention/readback策略，但前提是先有统一ownership和bytes会计。

### P2-4：没有跨进程/远程DDC的semantic block复用与QoS

content-addressed chunks已有基础，但缺平台cook namespace、remote fetch、range priority、bandwidth quota、offline fallback和poison quarantine。必须复用Runtime04 artifact identity，不能另建graphics私有缓存协议。

### P2-5：没有多GPU/device-group residency和资源迁移模型

DeviceGeneration应先支持单device loss/recreate；之后才能扩展per-device residency、peer copy、mirrored/pinned资源与预算。当前不应提前把device count bool塞进prepared key。

### P2-6：没有长期自适应质量、功耗和热状态控制

可在确定性budget controller之上加入working-set预测、带宽/功耗/thermal输入和quality governor。训练/启发式不得绕过硬预算、deadline和可复现capture。

## 7. 目标架构与所有权

### 7.1 唯一generation链

```text
AssetCatalogGeneration
  -> RenderArtifactManifestGeneration
  -> RenderAssetDemandGeneration
  -> CpuSubresourceResidencyGeneration
  -> UploadBatchGeneration
  -> RhiResourceGeneration
  -> PreparedRenderAssetGeneration
  -> FenceRetirementGeneration
  -> RenderScene / Material / Pass consumers
```

每一代都是immutable、可查询并携带content ID、asset revision、device generation、bytes和diagnostics。Asset layer拥有portable semantic blocks；residency manager拥有需求、CPU block lease与状态；RHI拥有GPU allocation/upload/fence；renderer只消费prepared handles。

### 7.2 核心类型边界

| 类型 | 唯一职责 | 禁止内容 |
|---|---|---|
| `RenderArtifactManifest` | metadata、semantic block目录、codec/format/alignment/hash/dependency | WGPU handle、scene visibility、同步filesystem read |
| `RenderSubresourceId` | texture mip/layer/tile、mesh LOD/cluster/page等versioned identity | runtime pointer、裸offset推测 |
| `RenderAssetDemand` | view/resource/priority/deadline/wanted quality/pin reason | I/O、decode或GPU创建 |
| `ResidencyTicket` | single-flight状态、generation、cancel、failure、waiter与进度poll | render-thread block/wait |
| `CpuBlockLease` | verified compressed/decoded block ownership与bytes | 永久clone完整asset |
| `UploadPlan` | target、copy ranges、staging reservation、dependency与expected generation | 直接queue submit |
| `PreparedRenderAssetHandle` | last-good/current generation和fallback状态 | asset load、revision poll、driver call |
| `RetirementTicket` | fence后释放old GPU/CPU reservation | CPU frame-index猜测完成 |

### 7.3 residency状态机

```text
Unrequested
  -> QueuedIo -> Reading -> Decoding -> ReadyCpu
  -> QueuedUpload -> Uploading -> Resident
  -> Evicting -> Unrequested

any pending state -> Cancelled
any operation -> Failed(last-good remains published)
device/revision/demand generation mismatch -> stale completion dropped
```

`Resident`不是单个bool：texture记录resident mip/tile set，Mesh记录LOD/cluster/page/buffer slices，Material记录dependency bootstrap readiness。request取消不立刻销毁in-flight GPU对象；它转为retirement并计入预算直到fence完成。

### 7.4 线程与锁规则

- render/UI/game线程只submit demand、poll handle、消费last-good，不做filesystem、zstd/bincode、image/model decode、transcode、buffer packing、driver allocation或wait。
- I/O、decode/transcode/cook-preview使用Runtime11共享lane并有task/byte/time/deadline budget；同ArtifactId+block+generation single-flight。
- RHI affinity lane批量处理upload/create/copy；submission completion由09A fence owner回报，residency manager不能直接`device.poll(Wait)`。
- `RenderFrameworkState`锁不得包围asset read/decode/driver wait；publication使用短临界区或immutable generation swap。
- shutdown先拒绝新demand，再cancel optional、deadline drain required upload、retire GPU objects；Drop不做I/O、submit或无期限join。

### 7.5 budget与fallback规则

- pool至少区分compressed source cache、decoded CPU、upload staging reserved/in-flight、persistent texture、persistent mesh buffer、material/binding、derived cache、retired GPU和task count。
- reservation按过渡峰值而非最终大小；`reserved + committed + in_flight + retired <= hard limit`是提交门槛。
- cold required使用bootstrap tail/lowest LOD/fallback material；超deadline发布显式fallback和reason，不阻塞frame。
- hot reload一直使用last-good，直到new dependency closure和RHI generation全部ready；失败不释放last-good。
- camera cut/teleport可以获得受预算约束的boost和临时quality降级，不允许无限超配。

## 8. 依赖顺序与重构里程碑

| 顺序 | 里程碑 | 交付与删除门槛 | 依赖 |
|---:|---|---|---|
| M0 | current-source freeze | 重取205文件/fingerprint、product callsites、dirty overlap与内存路径；锁定术语和owner | 本报告 |
| M1 | semantic artifact manifest | texture mip/layer与mesh LOD/cluster block目录、platform format/hash/alignment；random block reader | Runtime04 |
| M2 | async residency tickets | single-flight I/O/decode、priority/cancel/deadline/generation；render path同步load/decode=0 | Runtime11、M1 |
| M3 | RHI upload/retirement | staging reservation、batch、submission ticket、device generation和fence deferred free | 09A、M2 |
| M4 | texture tail-first streaming | 首次只建bootstrap tail，compressed partial upload、prediction和全view demand；删除whole-asset rebuild主路径 | M1-M3、09B |
| M5 | unified ownership/budget | graphics改用render lease，asset usage生效，全pool peak accounting、unused/scene unload eviction | M2-M4 |
| M6 | mesh residency/allocator | canonical Mesh artifact、buffer arena/suballocation/partial update、LOD；VG page需求复用09B owner | M1-M5、Render02/07 |
| M7 | importer/cook hard cut | glTF/OBJ/model无重复geometry，embedded texture走canonical cook，合并texture importer owner | M1、M4、M6 |
| M8 | material/prepared generation | dependency handle/single-flight、atomic last-good、dirty event替代per-frame poll | 09C、M2-M7 |
| M9 | shipping/device lifecycle | target artifact completeness、runtime source decode=0、loss/recreate/re-extract与shutdown闭环 | M1-M8 |
| M10 | diagnostics/editor | residency inspector、mip/LOD debug、budget/top offender/latency/thrash/failure视图 | M2-M9 |
| M11 | dynamic acceptance | traverse/teleport/reload/device-loss/OOM/fault/corrupt/cross-platform/soak与规模性能门槛 | 全部 |

M1不能只在现有bincode外再包一个mip offset表：如果取得block仍需顺序zstd解压完整对象，就没有完成semantic artifact。M4不能继续先full resident再tail eviction。M5不能只给HashMap增加LRU而保留clone-returning full asset和无fence释放。M7需要资产迁移后硬删除重复owner，不允许两套importer长期并存。

## 9. 量化验收矩阵

| 维度 | 场景 | 硬验收 |
|---|---|---|
| stable frame | 1/1k/100k visible resources，300/10k frames | filesystem/decompress/full clone/GPU create=0；revision full poll=0；锁内I/O/wait=0 |
| cold load | 1/100/10k texture+mesh；HDD/SATA/NVMe；cold/warm cache | 首可见只读bootstrap blocks；duplicate I/O/decode/upload=0；p50/p95/p99与bytes有界 |
| texture | 1K/4K/16K，RGBA/BC/ASTC，2D/cube/array，1/100/10k textures | 首次full-chain upload=0；wanted/resident正确；compressed partial有效；无thrash |
| mesh | 1/10/100M triangles，traditional/VG，static/skinned/morph | lowest LOD deadline可用；arena碎片/compaction有界；重复geometry bytes=0 |
| budget | 256 MiB/1/4/12 GiB VRAM，UMA/discrete，1/4 viewports | 任何时刻all-pool peak不越hard limit；debt可解释；驱逐/恢复确定 |
| camera | steady/pan/fast travel/cut/teleport/camera stack/reflection | 无render stall；boost受预算；fallback/mip lag可见；re-request thrash有门槛 |
| reload | texture/mesh/material/import settings变化，1/100/10k dependents | last-good无闪烁；atomic swap不混代；stale completion=0 publication；old按fence释放 |
| failure | missing/corrupt block、timeout、worker panic、OOM、queue/device loss | frame不阻塞；typed failure+retry/terminal；预算不泄漏；恢复有deadline |
| lifecycle | scene unload/world switch/PIE restart/shutdown/device recreate | unused prepared/CPU/GPU bytes回到基线；orphan ticket=0；Drop I/O/submit/join=0 |
| shipping | Windows DX12、Linux Vulkan、macOS Metal目标包 | required platform blocks完整；source image/model parse=0；runtime mipgen/transcode=0 |

动态证据必须把同一run中的AssetGeneration、DemandGeneration、DeviceGeneration、ticket ID和submission fence串起来：

- WPR/xperf：render/framework锁、filesystem、zstd/bincode、worker ready/wait、alloc/RSS/working set、queue depth/age；
- PIX/RenderDoc/GPU timestamps：texture/buffer create、copy/upload bytes、submission、old/new重叠、resident mip/LOD和fence retirement；
- I/O trace：requested semantic blocks、physical reads、cache hit/miss、coalescing、cancel后浪费和read amplification；
- budget capture：每pool reserved/committed/in-flight/retired、peak、debt、驱逐原因和top offenders；
- 至少三次cold/warm/traverse/teleport/reload runs，记录p50/p95/p99、peak RSS/VRAM、首可见deadline、quality lag、I/O amplification和能耗。

在这些数据前，只能说Zircon已有局部mip policy和chunk cache，不能说“ResourceStreamer”已经构成工程级streaming系统，也不能从静态代码推断其性能优于Unreal。

## 10. 删除清单

替代里程碑不完成下列同切片删除，就不算硬切换：

- render submission内调用`load_model_asset/load_mesh_asset/load_material_asset/load_texture_asset`的同步路径；
- `load_typed`完整asset clone作为graphics residency入口；
- 首次`PreparedTexture::fully_resident`再同帧tail eviction的路径；
- promotion/eviction重新读取完整TextureAsset并重建整张物理texture的产品主路径；
- generic bincode whole-object作为texture mip或mesh LOD/page随机读取格式；
- models/meshes/materials/textures永久HashMap且无unused/remove/retire的authority；
- per-primitive独立buffer、临时完整vertex转换和无allocator release的主路径；
- glTF root/per-mesh Model/per-primitive Mesh重复geometry payload；
- glTF内嵌图片直接`TextureAsset::new_rgba8`绕过canonical texture cook；
- `asset_importers/texture`与`texture_importer`重复产品descriptor/owner；
- 每帧registry/dependency polling替代typed dirty generation的路径；
- 只算final texture resident bytes的1 GiB warning budget；
- source-shape测试中要求保留上述旧路径的断言。

## 11. 本轮状态

### 11.1 2026-08-24 Shader06 定向复审增量（不替代 M0）

本次只重审并修改了两条 glTF texture subasset 路径，不构成 §8 M0 要求的 205 文件全量重取。Shader06 的 PBR correctness 修复已将 base-color/emissive 投影为 sRGB、normal/metallic-roughness/occlusion 投影为 linear；同一合法 glTF texture 被两种 transfer 角色引用时显式产生两个子资源，避免单一 GPU view 静默污染数据贴图。stable `gltf_importer.gltf` 还从借用 `images` 列表改为按 source 引用计数消费：最后一个 RGBA8 texture index 移动像素缓冲，之前的 index 才复制。对已输出 K 份共享 decoded RGBA8 纹理 payload 的单 source，这一局部改动从 `(K + 1) * S` 降为 `K * S` 常驻 decoded 像素，减少一份 `S` 和一次深复制；两个 transfer 输出本身仍是当前 asset/view 模型下正确性所需。

这不是 P1-6 的完成：embedded texture 仍直建单 mip `TextureAsset`，没有进入 canonical cook、semantic manifest 或 partial-residency pipeline。`texture_importer` 的离线 mip、linear/sRGB kernel、normal re-normalization 与 BC5 transcode 也不能被复制到两个 glTF importer。P1-6 只有在 runtime-owned `TextureBuildService` 接收 decoded bytes、source locator、sampler、transfer/normal semantics 与 material-slot usage，并硬删除 direct `TextureAsset::new_rgba8` embedded path 后才能关闭。该定向结论不提供 CPU/GPU 时间、RSS/VRAM、上传带宽或功耗数据；M0 source recheck、动态 profiling 及所有 09D implementation milestones 保持 pending。

同次 source audit 发现 `KHR_texture_basisu` 目前没有 runtime 或 stable plugin consumer：内建 ingest 的 required-extension 白名单不包含它；stable plugin 在 texture subasset 之前直接调用 `gltf::import`；两个路径最终都只接收已解码的 `gltf::image::Data`。这不足以保留 KTX2/Basis 的 encoded bytes、MIME、container metadata 或 platform target，且现有 runtime upload 也明确拒绝非零 KTX2 supercompression，要求 transcoding backend。故 P1-6 的 `TextureBuildService` 输入必须区分 decoded pixels 和 encoded container payload，并由唯一 source resolver 归一化 core、`EXT_texture_webp`、`KHR_texture_basisu` 的 image index；BasisLZ/其他超压缩内容只能在预算化的 cook worker 转为目标平台 GPU format、写入 semantic manifest 后进入 runtime。服务落地前，两条 importer 对 required BasisU 必须一致返回 typed unsupported，不得把 extension 名加入白名单或将容器伪装为 RGBA8。这个结论是接口和算法边界记录，不表示 BasisU 已支持，也不提供 CPU/GPU 时间、内存、带宽或功耗数据。

2026-08-24 current-source implementation 补齐了上段的最小安全边界：stable plugin 在 `gltf::import` 前、内建 ingest 在 image decode 前都拒绝 `KHR_texture_basisu` texture；可选 texture 返回 `KTX2/BasisU transcoder is required`，内建 required-extension policy 仍返回 typed unsupported。两条回归断言可选路径在外部 KTX2 读取前失败。仍无 encoded-container handoff、平台转码、semantic manifest 或 residency 改动，P1-6 保持未完成。

截至 2026-08-15 的 review 完成、implementation pending。该历史审查没有修改 production source，也没有运行 Cargo、Editor、cook 或真实 GPU；focused source fingerprint、文件/行数和函数行为都是当日快照。相关 graphics/asset/importer 目录存在其他 Session 改动，M0 必须重查后才能实施。

### 11.2 2026-08-26 M2 residency contract foundation

本次在开始实现前定向重审了当前 `ResourceStreamer`、核心 `ResourceManagementGeneration` / `ResourceReadinessGeneration`、RenderScene 资源净引用 journal、Runtime11 bounded keyed I/O lane、09A `DeviceGeneration` / `SubmissionTicket`，并复核 Unreal `FRenderAssetUpdate`、`StaticMeshUpdate`、`Texture2DStreamIn` 与 Bevy `render_asset.rs`。结论保持不变：旧 `ensure_scene_resources` 的同步 load/clone/WGPU create 路径不能作为新 authority；核心资源 `record.revision` 是单资产版本，readiness sequence/dependency revision 是依赖闭包版本，RHI device epoch 与 submission ticket 继续由 09A 拥有。

已新增 `graphics::scene::resources::render_asset_residency` 的 CPU-only contract foundation：

- `RenderAssetResidencyManager` 是本路径唯一引用计数、single-flight pending request 和 last-good active residency authority，直接消费 RenderScene 已排序去重的 typed net reference delta；
- ticket 绑定 `ResourceId + ResourceKind`、asset revision、readiness generation、dependency revision、demand generation、`DeviceId + DeviceGeneration` 与 bootstrap/all-LOD scope；Model/Mesh 明确申请 all-LOD，Material/Texture/Shader/Skeleton 申请 bootstrap；ticket 同时携带显式 route：Mesh/Texture 为 `SemanticBlocks`，Model 为 `CanonicalMeshSet`，Material/MaterialGraph/Shader/Skeleton 为 `PreparedDependencies`，后续 consumer 不再根据 `ResourceKind` 自行猜测加载路径；
- delta 先完整 preflight，再原子提交；underflow、overflow、重复 delta、catalog/readiness 缺失、kind 冲突或 ticket id 枯竭均不改变引用表，也不消耗 ticket id；
- entry 同时保留 active 与 pending。reload/I/O/decode/upload 失败保留 active last-good；新 submission `Completed` 后才原子交换，旧 active 作为带 submission ticket 的 `RetireResident` 工作交给 09A fence retirement；
- pending state 覆盖 `QueuedIo -> Reading -> Decoding -> ReadyCpu -> QueuedUpload -> Uploading` 及 Failed/Cancelled，非法跨阶段、stale ticket、错误 device epoch、错误 submission 与非终态 completion 均返回 typed error；
- 最后一条引用释放时，未提交工作输出 `CancelPending`，in-flight upload 输出带 submission 的 `RetireInFlight`，resident 输出 `RetireResident`；本层不直接销毁 GPU 对象；
- 已写 7 个 folder-backed regression，覆盖 generation-bound scope、显式资源 route、引用 single-flight、原子 underflow 回滚、RHI submission 绑定、reload last-good/原子替换和 in-flight fence handoff。

复杂度按 change frontier 收敛：空 journal 不分配；`C` 条资源 delta 的 admission 为期望 `O(C)`、临时内存 `O(C)`，只有 `0 -> referenced` 才读取 catalog/readiness 并签发 ticket；hot reload reconciliation 为期望 `O(H)`，`H` 是 typed dirty frontier，而不是 live scene/resource 总量。一个 authority HashMap 是必要的 residency state table；没有按 primitive 建 shadow map，也没有 frame-visible 全表轮询、WGPU 调用、filesystem、decode、完整 asset clone 或 GPU create。

`CanonicalMeshSet` 目前只是必须保留的架构边界，不是已接通的成功路径。当前 `ModelAsset` 仍可嵌入完整 vertex/index payload 并同时引用独立 Mesh，RenderScene journal 又会同时发出 Model 与 Mesh 资源引用；若现在把 Model 直接映射到 semantic block loader，会重复读、解码和上传同一份几何。M7 必须把 model 收敛为 canonical mesh-set metadata/reference owner，删除 importer/runtime 的重复 geometry payload 后才能接线。`PreparedDependencies` 同样不伪装成 semantic manifest，它只表示这些资源应消费已经准备好的依赖闭包。

本节不完成 M0 全量 205 文件 source freeze，也不越过依赖顺序声称 M1/M2 完成。Runtime11 block loader 与 residency route 的产品编排、09A upload/retirement consumer、transitive material dependency expansion、budget reservation、旧 `ResourceStreamer` hard cut 和产品 frame scheduling 仍 pending。当前仅通过 scoped rustfmt 与源码边界检查；managed Cargo 仍无可用 completion，因此没有 compile/test pass、截图、RDC、GPU timing、RSS/VRAM、I/O、功耗或 Unreal 对比结论，也不提交 milestone commit/企微完成通知。

### 11.3 2026-08-26 M1 semantic manifest contract foundation

M1 定向重审确认当前 `ArtifactManifest` 只记录整个 `ArtifactCacheAsset` 经 bincode+zstd 后的 content hash 与 64 KiB 物理 chunk；`ChunkReader` 必须按顺序恢复完整压缩流并验证整体 hash，`TextureAsset` 也仍把完整 RGBA/container payload 放在单个 `Vec`。因此已有 `read_compressed_chunk` 只能随机取得 whole-object 压缩片段，不能随机取得可独立校验、解码和上传的 mip/LOD semantic block。该结论与 Unreal texture 每 mip bulk data、static mesh per-LOD stream-in，以及本报告 M1 的原始判定一致。

已在 Asset artifact 层新增独立的 `render_manifest` contract，未修改正被其它 Session 编辑的 `store.rs` / `chunk_residency.rs`：

- `RenderArtifactManifest` 绑定 typed resource、asset revision、target platform、portable layout、typed asset dependencies 和 canonical block directory；header/layout 可独立驻留，不含 WGPU/RHI handle；
- Texture layout 必须完整列出每个 `mip x array layer`，并以 `bootstrap_first_mip` 定义 tail；缺任一 semantic owner、重复 owner、越界或错误 residency class 都拒绝；
- manifest schema 已硬切到 v2；texture layout 还必须记录基础 width/height、2D compression block width/height 与 bytes-per-block，并能为每个 mip/layer 确定性导出 extent、tight bytes-per-row、block rows 和 decoded bytes。block decoded size 与该 tight upload layout 不一致、mip chain 超过基础 extent 或 block geometry 为零均 typed reject，不能在释放旧 `TextureAsset` 后再靠 format 字符串猜 row pitch；
- Mesh layout 必须完整列出每个 LOD，并以 `bootstrap_first_lod..lod_count` 定义最低质量 bootstrap tail；可追加 `(lod,page)` cluster block，但 page 必须归属合法 LOD 并继承该 LOD 的 residency class；
- 每个 block 携带独立 256-bit content ID、Raw/Zstd codec、encoded/decoded bytes、非零 power-of-two alignment、platform format、bootstrap/streamable class 和 block dependency；zero content ID、Raw 大小不等、空 block、格式漂移、自依赖、缺依赖和 dependency cycle 都拒绝；
- manifest 构造对 asset dependency、block 和 block dependency 做 deterministic sort/dedup；查询按已排序 subresource 二分，不建立另一张 runtime cache；
- 未臆造平台 alignment 上限：contract 只验证 power-of-two，实际上限必须来自后续 target cook/RHI capability profile。
- `RenderArtifactStore` 将每个 encoded semantic block 作为独立 immutable content-addressed 文件发布，路径由 BLAKE3 content ID 分片；manifest key 绑定 typed resource、asset revision 和 target-platform hash，target 字符串不直接进入路径；
- block 发布前验证 descriptor size 与 encoded-byte hash，已存在对象只有重新有界读取并复验后才能 `Reused`；manifest 发布前验证 contract 并逐块确认已发布内容，使用 `atomic_write_new` 保证同 key 不覆盖，内容不同返回 typed conflict；
- manifest codec 带固定 magic、fixed-int encoding、trailing-byte rejection 和 caller-owned byte cap；block/manifest 读取都先检查 metadata、再以 `limit + 1` 有界读取并复验实际大小，避免无界文件分配。该同步 API 是 cook/Runtime11 worker primitive，不得在 render frame 线程直接调用。

构造/验证成本为 `B` 个 block 的 `O(B log B + E log B)`，临时内存 `O(B + E)`；任意 caller 的 `publish_manifest` 为保证 content-addressed store 完整性仍顺序复验 `B` 个独立 encoded block，总 I/O 为 `O(sum(encoded block bytes))`，这是 cook/publication 边界，不进入稳定 render frame。受类型约束的 cook bundle publication 见 §11.5。单 block random read 的内存与 I/O 为 `O(encoded block bytes)`，不会恢复整个 asset。已写 17 个 `render_` regression：11 个覆盖 manifest contract、texture/mesh upload layout 与拒绝路径，6 个覆盖 block/bundle publish/reuse/random read、hash 拒绝、caller limit、manifest block completeness/round-trip 和 target path isolation。scoped rustfmt、命名、行预算、生产 panic、Asset/Graphics 边界与有界 codec 检查通过；filesystem 只存在于 store worker primitive 及其 workspace-local 测试。

这仍不是 M1 完成：同步 store reader 已有 Runtime11 worker adapter foundation，Texture 与单 LOD traditional Mesh semantic cook 已有纯 CPU foundation，manifest async admission 也已有独立 ticket owner；但多 LOD/importer cook、VG page cook、旧 whole-object migration 与 shipping completeness gate 均 pending。没有执行 Cargo 或真实文件测试，也没有接入产品 render path，不能声称 read amplification、吞吐、内存或 hitch 已改善。

### 11.4 2026-08-26 Runtime11 semantic block loader foundation

实现前复审了 Runtime11 `BoundedKeyedIoLane`、`ExecutionRuntime` / `ExecutionScope`、现有 preference/dynamic-scene consumer 和 asset worker completion registry。`BoundedKeyedIoLane` 当前只有一个 active entry，是为有序持久化和 fence 设计的串行 lane；若直接用于 texture mip / mesh page，会把所有 semantic block read 全局串行化。因此本路径只复用 Runtime11 显式 worker owner 与 scope lifecycle，不复制造线程池，也不误用串行持久化 lane。

已在 Asset artifact 层新增 `render_manifest::loader` foundation：

- content/decode key 由 content ID、codec、encoded bytes 与 decoded bytes 组成；等价请求只有一个 I/O/decode flight。共享对象仅是 decoded `Arc<[u8]>`，每个 ticket 保留自己的 mip/page descriptor，避免跨 subresource 复用时继承第一个请求的上传身份；
- 每个 loader 显式绑定一个 `ExecutionRuntime` scope。独立 block read 提交到 `TaskPoolKind::Io`，只有 Zstd decode 提交到 `TaskPoolKind::AsyncCompute`；Raw block 在 hash/size 验证后直接共享 encoded allocation，不做第二份 payload clone；
- admission 同时限制 live entry、total ticket、per-entry ticket、store encoded bytes、decoded bytes 和 retained bytes。新 entry 按 `encoded + decoded + metadata` 预留峰值，任何溢出或超预算都在任务提交前 typed reject；
- deadline 由 `BTreeSet<(Instant,ticket)>` 保存并在维护点只消费到期前沿；admit/remove 为 `O(log W)`，一次维护为 `O(X log W)`，`W` 是 live tickets、`X` 是本次到期 ticket，不逐帧扫描全部 entry，也不为每个 mip 占用 process timer 的固定 512 项容量；
- ticket drop/cancel/expiry 只移除该 observer；最后 observer 离开才从 single-flight registry 释放预算并 drop `ExecutionTask` 请求 cooperative cancellation。owner close 关闭 scope admission、发布 `OwnerClosed` 并释放 registry；running 文件 syscall 不能被中途抢占，但单 block 大小受 caller cap，返回后不再进入 decode/upload；
- Zstd decoder 以 `decoded_bytes + 1` 有界读取并要求精确 decoded size，防止压缩炸弹越过预留；诊断记录 live entry/ticket/bytes、merge、I/O/decode task 数、ready/fail/cancel/expiry、encoded/decoded bytes 与两个 worker wall；
- 已写 7 个 `render_` regression，覆盖 Raw single-flight、同内容不同 subresource descriptor 隔离、Zstd staged decode、retained-byte 原子拒绝、deadline frontier、store verification failure、close 与 decoded-bomb preflight。加上 manifest/store contract 共 24 个 focused tests。
- 新增独立 `RenderArtifactManifestLoader`，以 `resource + asset revision + target platform` 为 identity single-flight，并把 manifest filesystem read 放到自己的 Runtime11 I/O scope。新 identity 在提交任务前按 caller 的 `max_manifest_bytes + fixed owner metadata` 预留 serialized-manifest budget，限制 live entry、总 ticket 与 per-entry waiter；ticket drop、deadline、owner close 和 typed `NotFound/StoreLimitExceeded/InvalidManifest/StoreUnavailable` failure 不再要求 render caller 同步读文件。identity lookup/merge 为期望 `O(1)`，deadline admit/remove 为 `O(log W)`，维护只消费 `X` 个到期 ticket；新增 5 个回归入口覆盖 shared manifest owner、missing failure、deadline frontier、retention config preflight 与 close。此时 manifest/store/loader focused tests 为 29 个；
- manifest dependency validation 新增 residency 单调性：Bootstrap block 依赖 Streamable block 直接拒绝，避免 texture tail/最低 mesh LOD 首驻留反向拉入高质量数据；
- `RenderArtifactLoadPlan` 对 Bootstrap/All scope 先求 dependency closure，再以 canonical subresource `BTreeSet` 生成依赖优先的确定性并行 batch，并预计算 block count 与 encoded/decoded byte totals。构造为 `O(B log B + E log B)`、内存 `O(B + E)`，只发生在 manifest generation/request frontier，不进入稳定帧全量循环；新增 3 个回归覆盖非法 residency edge、bootstrap 排除高质量 block 和重复 All plan 的批次确定性。此处 focused tests 总数为 32。

后续实现已把 block request 从“admit 后立即 submit”硬切为 loader-owned priority/deadline frontier。排序键为 priority 降序、同优先级 deadline 升序、再按 FIFO；同 content 的新 waiter 会在尚未 dispatch 时提升现有 flight，ticket cancel/deadline 会删除自己的 waiter 并重算有效顺序。`dispatch_io` 在提交前消费 deadline frontier，并同时受 caller task/encoded-byte budget 与 Runtime11 scope 剩余容量约束；因此 priority 会实际改变进入 I/O worker 的顺序，不是 ticket 上的装饰字段，也没有用等待 commit gate 占住 Rayon worker。

manifest request 已硬切到同一套公共 `RenderArtifactIoFrontier<K>` 和 `RenderArtifactIoPriority`。`request_batch` 以 `resource + revision + target platform` 分组，在任何 registry mutation 前一次性预检 total/per-entry ticket、unique entry、manifest retained-byte quote、ticket ID 与 frontier sequence；`dispatch_io` 再按 deadline/priority/FIFO 和 Runtime11 scope 剩余容量提交真实 filesystem read。manifest 与 block 因此共享调度语义，但仍各自拥有 bounded scope、entry state 和 typed failure，避免把两种 payload 生命周期塞进单一巨型 loader。

`request_batch` 先验证全部 descriptor，再一次性预检 total/per-entry ticket、unique entry、retained bytes、ticket ID 与 frontier sequence 区间；所有可失败检查完成后才修改 registry，批内重复 decode key 只创建一个 entry。`RenderArtifactLoadBatch` 直接映射到该入口，上层不需要逐 block admission。期望复杂度为 `O(B log W_e + U log E)`，额外内存 `O(B + U)`；`B` 是批内 tickets、`U` 是新 unique flights、`W_e` 是单 flight waiter 数、`E` 是 queued flights。一次 dispatch 为 `O(D log E)`，不扫描 live entries。

SemanticBlocks residency route 已增加 dependency-batch cursor：只有当前 batch 全部 Ready 才准入下一 batch，中途容量不足保留为可重试 Deferred，block failure/cancel 保持 typed terminal。低层 `RenderAssetCpuBlockLease` 同时持有 decoded block 与所有完成的 block ticket batch；高层 `RenderAssetSemanticLoad` 再串联 manifest ticket/poll、load-plan 构造与 block cursor，最终 `RenderAssetCpuArtifactLease` 同时持有 manifest header/ticket 和 block lease。后续 upload consumer 因此可以直接读取 schema 布局，而且 manifest/block 两侧 retained-byte 会计都持续到 CPU lease drop，不需要同步 manifest read，也不会只保留 `Arc<[u8]>` 后提前释放预算。

新增 4 个 block loader regression 覆盖 batch 全有或全无、批内 content single-flight、真实 priority 顺序和 load-plan adapter；manifest loader 新增 2 个 regression 覆盖批量准入全有或全无与真实 priority 派发；公共 frontier 新增 2 个纯算法 regression，覆盖高优先级 waiter 离开后的重排，以及同优先级 deadline/FIFO 顺序；residency 新增 2 个 regression，分别覆盖 dependency batch 顺序与 block budget lease，以及 manifest-to-block 完整 route 与双 loader budget lease。`render_manifest` focused test 入口为 46 个，连同 render-asset residency 的 9 个入口，当前 scoped test 属性总数为 55。

当前 scoped rustfmt、diff、生产 panic/unwrap/expect、私有线程/channel、文件预算与 Asset→Graphics/WGPU 反向依赖静态检查通过；既有 touched production owner 不超过 687 行。该结果仍是 source-only foundation：产品 frame scheduler 尚未拥有 semantic load 集合，cursor 仍 poll 当前 batch 而没有 ready completion frontier，CanonicalMeshSet/PreparedDependencies route、09A staging/upload/submission/fence retirement、产品 shutdown、旧 `ResourceStreamer` hard cut 均 pending。没有 managed compile/test、真实 I/O wall/RSS/吞吐、GPU 上传、截图、RDC、功耗或 Unreal 等价工作量对比，因此 M2 不得提交 milestone。

### 11.5 2026-08-26 Texture semantic cook foundation

在实现 cook 前重审了当前 `TextureAsset`、`TextureUploadPlan`、RGBA array/cubemap mip 交错规则、DDS/KTX2 subresource table、ASTC header 与 WGPU compressed upload consumer。审查先发现 manifest v1 缺少基础 extent 和 compression block geometry，semantic block 离开旧 asset 后无法唯一恢复 row pitch；因此本轮先完成上述 schema v2 hard cut，再新增 `cook_texture_render_artifact`，没有在不完整 contract 上继续堆 adapter。

- cook 只接受当前 `TextureUploadReadiness::Ready` 且 block depth 为 1 的 D2/array/cube payload；target capability 由显式 `TextureUploadSupport` profile 输入，不读取 live WGPU device；
- RGBA mip-major/layer payload、DDS layer-major table、KTX2 mip-major table和单块 ASTC 都归一化到固定 `mip * layer_count + layer` 槽位。范围建立、重复/缺失/越界检查为 `O(B)`，不需要树表或排序；
- 每块直接引用一个共享 `Arc<Vec<u8>>` payload owner 与 byte range，原 `Vec` 被移动进 owner 而不转换为可能重分配的 `Arc<[u8]>`；内容 hash 只覆盖 GPU-ready semantic bytes，container header/gap 不进入 block，cook 不为每个 mip 分配第二份 `Vec`。总 CPU 为 `O(S + B)`，额外 metadata 为 `O(B)`，`S` 为被 hash 的 semantic bytes；
- 每个高质量 mip 显式依赖同 layer 的下一粗 mip，bootstrap tail 保持 residency 单调；`RenderArtifactCookOutput` 同时返回已验证 manifest 与逐块可发布 slice；
- `publish_render_artifact_cook_output` 先核对 output block directory 与 manifest 完全一致，再逐块执行现有 size/hash/immutable create 校验；全部 block 成功后才调用 store 私有的 verified-manifest writer，普通 caller 无法绕过 completeness check。新 block 不再由 `publish_manifest` 重读一次；复用 block 仍必须从磁盘读取并复验。失败可留下由 content ID 命名的不可变 orphan block，但不会发布不完整 manifest，后续相同 cook 可安全复用；report 分别量化 published/reused block count 与 encoded bytes；
- 新增 4 个 regression，覆盖双 layer/双 mip RGBA 精确切片与依赖、ASTC header 剥离、bootstrap 越界拒绝，以及 bundle 首次发布/复用/manifest round-trip。连同 Runtime11 manifest loader、Mesh schema v3 与 Mesh cook 回归入口，`render_manifest` focused tests 合计 38 个，所有 production/test owner 仍不超过 500 行，scoped rustfmt、生产 panic、filesystem/WGPU 反向依赖检查通过。

该 foundation 尚未接入 importer/project asset manager 的 cook transaction，也没有多 LOD/VG Mesh cook、target-format transcoding、per-block Zstd policy、manifest-to-block load-plan execution、09A upload consumer和旧 whole-object 删除。managed Cargo 仍无 completion，所以本节没有 compile/test pass、真实 cook wall/peak RSS、I/O amplification、GPU timing、截图或 RDC 证据，不能提交里程碑或发送企微完成通知。

### 11.6 2026-08-26 Mesh semantic cook foundation

在写 Mesh cook 前重新检查了 `MeshAsset` 的 SoA attributes/index、`MeshAsset::to_model_primitive`、当前 `GpuMeshResource::from_asset` 的临时 AoS packing、固定 96-byte `GpuMeshVertex`、固定 `Uint32` index binding，以及 Unreal `FRenderAssetUpdate` / `StaticMeshUpdate` 的 per-LOD bulk I/O、intermediate buffer 和 cancel/completion owner。重审发现当时的 manifest v2 Mesh contract 不足以生成可独立上传的 semantic block：

- `RenderArtifactLayout::Mesh` 只有 `platform_format + lod_count + bootstrap_first_lod`，`MeshLod` block 没有 vertex/index byte range、stride/layout、index format、topology、vertex/index count 或 bounds。block 离开旧 `MeshAsset` 后，consumer 无法唯一构造 buffer 与 draw metadata；
- 当前 GPU 路径把 `ModelPrimitiveAsset` 全量转换为 96-byte AoS vertex `Vec`，再分别 `create_buffer_init` vertex/index buffer，并固定按 `Uint32` 绑定。直接在 cook 复制这段代码会把 graphics-private layout 复制进 Asset owner，且仍没有 arena/upload packet；
- `MeshAsset` 同时允许 morph、skin、SDF 和 virtual geometry payload，而 Model 仍可能嵌入同一几何。单个“LOD 0 bytes”不能诚实代表这些 owner，也不能提前关闭 M6/M7 的 canonical geometry hard cut；
- Unreal 参考路径把每个 LOD 的 bulk payload、I/O request、intermediate buffers、RHI create 和 cancel completion 分开持有；Zircon 必须保持同样的阶段边界，但不能复制其线程阻塞与遗留对象生命周期。

本轮已先完成 schema v3 hard cut：`RenderArtifactMeshLayout` 显式持有 portable `StaticMeshV1` 96-byte vertex format、`Uint32` index format、bootstrap LOD 与 canonical per-LOD metadata；`RenderArtifactMeshLodLayout` 记录 topology、vertex/index count、index offset 和以 IEEE bits 稳定序列化的 min/max bounds；manifest 可直接导出 vertex/index byte ranges与 decoded bytes。validator 拒绝空/非 canonical LOD、identity 溢出、零 count、非有限或反向 bounds、index overlap/misalignment、block decoded-size mismatch 与错误 residency，同时保留同 LOD `MeshClusterPage` extension。新增 2 个回归入口覆盖 upload range/bounds 与 decoded-size 拒绝；schema v2 artifact 不保留兼容 facade。

在 schema v3 上已新增 `cook_mesh_render_artifact` 的传统 Mesh 单 LOD foundation：

- 只接受当前 GPU 产品主路径实际支持的 `TriangleList`；morph、skin、mesh SDF 与 virtual geometry 必须先有各自 artifact owner，否则 typed reject，不把它们静默丢进 static LOD blob；
- 从 `MeshAsset` 的 SoA built-in attributes 直接写入最终 little-endian `StaticMeshV1` 96-byte AoS vertex 和 `Uint32` index block。U16 index 在写入时转换，无 index 的 triangle list 直接物化隐式 `0..vertex_count`；不经过 `ModelPrimitiveAsset`、graphics-private `GpuMeshVertex` 或第二个完整 vertex `Vec`；
- packing 同一趟计算 min/max bounds，并拒绝 position/normal/UV/weight/tangent/color 的非有限值；最终 byte count 必须与 schema 预计算完全一致后才能构造 manifest。算法为 `O(A + V + I)`，cook bulk memory 是一个最终 `96 * V + 4 * I` payload 加常量 attribute views/metadata；这是结构分析，不是实测 wall/RSS；
- 新增 3 个回归入口，覆盖精确 vertex/index byte layout、U16 到 Uint32 转换、implicit index 与当前不支持 topology 的拒绝。相同 `RenderArtifactCookOutput` 可复用已实现的 bundle publication。

这仍不是 Mesh streaming 完成：只有一个传统 LOD，尚无 importer LOD source、LOD simplification/quality policy、VG page cook、Mesh SDF split、manifest/load-plan executor、buffer arena/suballocation、09A upload/fence retirement或 M7 canonical Model hard cut。当前只有 static contract/rustfmt/source-boundary 证据，没有 managed compile/test、真实 packing wall/peak RSS、GPU upload、截图或 RDC。

### 11.7 2026-08-27 M3 semantic GPU upload foundation

本轮在实现上传前重新检查了中立 `RenderDevice`、生产 `WgpuRhiDevice` registry/submission service、deterministic backend、旧 graphics `RenderBackend`、texture batch upload、mesh `create_buffer_init`、`RenderAssetResidencyManager` 状态机，以及 Unreal `FRenderAssetUpdate` / `Texture2DStreamIn_IO` 和 Bevy `RenderAssetBytesPerFrame` / prepare-retry 边界。第一次草案让 semantic executor 直接依赖旧 graphics WGPU backend；按 09A 的静态门复审后判定该方向会建立第二套 handle/lifetime owner，已在源码阶段撤销。最终边界是：09D 只生成中立上传计划并持有 CPU lease；`zr_rhi` 定义 batch contract；`zr_rhi_wgpu` 负责 native translation、submission timeline 和 registry lifetime。

- 新增 `RenderAssetGpuUploadPlan::prepare`，逐块复核 manifest descriptor、decoded byte count、重复/未知 subresource，并在任何 RHI resource 创建前一次性预检 subresource、staging bytes 与 destination bytes 三类预算。失败均为 typed error，不允许部分创建或部分准入；
- texture 计划要求所选 mip 区间连续且每个 array layer 完整，将 source tail `[first_mip, end)` 映射到从物理 mip 0 开始的紧凑 texture；当前只映射语义明确的 `rgba8unorm`、`rgba8unorm_srgb` 和 `rgba16float`。DDS/KTX/ASTC 等容器格式在 manifest 尚不能唯一表达 physical format / color space 前返回 typed unsupported，不做猜测；
- mesh 计划只接收 canonical `StaticMeshV1` LOD block，按 LOD 稳定顺序把 vertex ranges 和 index ranges分别放入每资产两个 packed buffer，并保留 draw metadata；这不是全局 arena/suballocation，M6 仍 pending。decoded `Arc<[u8]>` 直接成为 batch source owner + range，不重打包 payload；
- `zr_rhi::RenderDevice` 已提升 `write_buffer_batch` / `write_texture_batch` 为中立合同，旧单次 write API 只是单元素 batch adapter。一个语义 texture 或 mesh asset 只签发一个 copy submission ticket；生产 WGPU 后端在 registry lock 内验证 generation/usage/range、把共享 payload range 转为既有 native batch，并用同一 ticket 标记全部 resource last-use 后交给已有 submission service，不允许 semantic/resource 层直接 `queue.submit`；
- deterministic backend 镜像同一合同，批准入先完整验证全部 destination/range 与 pending upload/staging budget，再签发一个 copy ticket；执行辅助已从 919 行设备文件拆到独立 `device/uploads.rs`，设备 owner 收敛为 763 行；
- `RenderAssetGpuUploadLease` 同时拥有 CPU artifact lease、RHI handle artifact、submission ticket 与预算 quote。Accepted/Submitted 保持 pending；Completed/Failed/Cancelled/DeviceLost 才释放 CPU lease，避免 staging source 在 GPU copy 完成前提前离开 retained-byte 会计；终态转换不再导出到整个 crate，只能由 residency 子系统执行；
- `RenderAssetResidencyManager` 已接管 pending upload、active artifact、hot-reload/release 后的 in-flight upload 与 ready retirement。Completed 才原子替换 last-good artifact；失败保留 last-good，失败时新建 handle进入 retirement；旧 active 替换、entry 释放和在途释放都转交 RHI handle destruction，native 释放仍由 09A registry 的 last-use ticket 延迟；
- plan validation 对每块执行 manifest 二分查找并维护 hash set，期望复杂度为 `O(B log M)`、额外内存 `O(B)`；texture/mesh canonical packing 使用有序 map，为 `O(B log B)`；batch 构造与 backend translation 为 `O(B)`。这些是源码算法界限，不是实测性能数据。

本轮新增 7 个 focused test 属性：4 个 semantic plan/边界测试、1 个 residency owner 静态守卫、1 个中立 batch payload/range contract 测试、1 个 deterministic 双 buffer 单 ticket 行为测试；连同此前 55 个 scoped 入口，相关属性总数为 62。scoped rustfmt、diff、production panic/unwrap/expect、新增 semantic residency 路径的 raw `wgpu::` / `zr_rhi_wgpu` /旧 `RenderBackend`、raw submit/create-buffer-init 与文件预算只做 source-level 检查。

M3 仍不是可验收里程碑：产品 frame scheduler 尚未驱动 semantic load/upload/completion 集合，ready retirement 还必须由产品 RHI tick 持续排空，队列上限/age/telemetry 尚未接入 profile admission，旧 `ResourceStreamer` 也未 hard cut。当前没有 managed Cargo completion、真实 WGPU execution、GPU timing/RSS/VRAM/I/O、RenderDoc capture、功耗数据或 `docs/tests/runtime/render` 实机截图；因此不得提交 milestone、发送企微完成通知或声称与 Unreal 等价。下一步优先接产品有界 completion/retirement 驱动，再逐步切除旧 whole-object texture/mesh 路径。

### 11.8 2026-08-27 M3 bounded completion/retirement maintenance foundation

产品接线前再次复核 `SceneRenderer`、`WgpuRenderFrameworkCore`、neutral MVP renderer 和 `RenderDevice` 后确认：真实场景 renderer 仍持有旧 raw `RenderBackend + ResourceStreamer`，`WgpuRenderDevice` 目前只进入独立 neutral MVP fixture。把新 residency manager 直接塞进旧 `SceneRenderer` 会形成双 device/queue/lifetime owner，不是可接受的产品接线。因此本轮先补齐 neutral product owner 必需的有界 completion consumer，不修改旧 WGPU renderer 来伪装硬切完成。

- `RenderAssetGpuSubmissionFrontier` 用 `(device id, device generation, submission sequence, queue class)` 的 `BTreeMap` 保存 manager 自己拥有的 ticket，并保留轮转 cursor。每 tick 只取 `K = min(configured status budget, tracked submissions)` 个 successor，复杂度为 `O(K log N)`，不会扫描 residency entry HashMap 或全部 RHI submission；
- observation ticket/status scratch 由 manager 跨 tick 复用。`RenderDevice::append_submission_statuses` 新增中立批量观察合同，production `WgpuSubmissionService` 与 deterministic backend 都只持有一次 submission-state lock并按输入顺序返回逐 ticket `Result`，消除 `K` 次 mutex 获取；
- tracked submission 总量默认绑定 `SubmissionLimits::max_terminal_statuses`，也允许显式构造限制。重复 ticket 或容量耗尽在 residency 状态突变前分别返回 `SubmissionAlreadyTracked` / `GpuTrackingBackpressure`，避免产品长时间不维护时观察前沿无界增长；
- `maintain_gpu_after_rhi_poll` 明确不拥有 `device.poll_submissions`，并强制接收 backend 成功完成泵后发放的 generation-qualified `SubmissionPollReceipt`。入口在任何 status 查询和 retirement 前拒绝 foreign、跨 stream 或未严格前进的 receipt；拒绝只写入返回 report，不修改 manager、查询 RHI 或销毁资源。唯一产品 RHI owner 后续必须每帧只 poll 一次并把同一 receipt 分发给 completion consumers；status history 已过期或查询失败时记录 typed failure并按 Failed fail-closed，释放 CPU lease、保留 last-good、把未发布 artifact送入 retirement，绝不把未知工作误发布为 resident；
- Completed upload 原子替换 active artifact，并把旧 metadata release 返还给调用者；Failed/Cancelled/DeviceLost 保留 active last-good。hot-reload/release 后 detached upload仍由同一 ticket frontier推进到终态；
- ready artifact 按独立 `max_artifact_retirements` 预算进入 RHI destroy，report量化 status checks、terminal/published/failed/detached、retirement attempts/success bytes、remaining backlog、metadata releases 和 typed failures。全量 `take_ready_gpu_retirements` 旁路已删除；retirement artifact记录子资源级成功进度，texture严格先释放 view再释放texture，mesh独立推进两个buffer。任一步失败时保留artifact并排到下一帧重试，且本帧尝试次数在排队前取快照，避免单个持续失败项在同一帧重复耗尽预算；native释放继续由 registry last-use ticket治理。
- `RenderAssetGpuResidencyLimits` 另外硬限制 ready-retirement artifact 数，默认与 RHI terminal-history容量同源。upload bind、active release batch和terminal publish在状态突变前 fail-closed；release batch在 ticket id分配前做一次 `O(D)` 精确计数，保证 backpressure不消耗 ticket或部分删除 entry。detached terminal upload只处理当前可用槽位，其余继续留在 bounded submission frontier；队列声明 bytes和本帧 deferred terminal upload进入 maintenance report。维护顺序先按预算释放旧 backlog，再批量观察 completion，避免队列满时平白增加一帧 terminal retry。physical bytes仍由 RHI `GpuMemoryBudget`唯一治理，不复制第二套 residency byte limit；
- 触达的 neutral/deterministic/production RHI device roots同步遵守结构预算：公共错误合同、构造策略与capability receipt分别迁入具名child owner，根文件为515/767/791行；结构测试禁止职责回流并继续执行 `< 800` 门。

新增 5 个 focused functional test 属性：3 个覆盖观察预算轮转、device-generation identity/精确移除和“不重复 poll/不扫描 entries”的静态门；1 个覆盖 tracking duplicate/capacity 原子拒绝；1 个覆盖中立批量状态观察的顺序和逐 ticket failure。随后新增 2 个 receipt 属性，分别覆盖 deterministic backend 的 generation-qualified 单调序列，以及 residency 对 matching/advanced、replay 和 foreign receipt 的验证；再新增 1 个 retirement capacity 属性，覆盖边界与 `usize` overflow fail-closed；device recovery 又新增 3 个属性，覆盖 active/uploading exactly-once 终止与重发、失败预检不消费 ticket，以及未显式恢复时拒绝 completion stream 换代并在 reset 后接受 replacement。连同此前 62 个 scoped 入口，相关功能属性源码入口总数为 73；另累计新增 3 个 Runtime15 结构门，锁定 neutral error、production capability、folder-backed tests、residency recovery/ticket child owner 和 `< 800` 行预算。上述数字都只是测试源码入口计数，不是执行通过数。

residency 层现有显式 `recover_device_epoch`：按稳定 ResourceId 做完整 catalog/readiness/epoch/ticket-id 预检，失败不消费 ticket、不改 entry/GPU state；成功保留引用计数，对旧 pending/active 发布 exactly-once typed release，丢弃旧 generation 的 active/pending/detached/ready-retirement handle 投影，清 submission frontier/last poll receipt，并为每个 live resource 发布 replacement `QueuedIo` ticket。该冷路径是 `O(N log N)` / `O(N)`；稳态只增加 O(1) bound-epoch 守卫。实现按 UE `ReleaseRHIForAllResources -> InitRHI` 的 owner 顺序设计，但不复制全局资源表；旧 native registry 的真实释放仍归 failed `WgpuRenderDevice` generation owner drop。

当前仍是 source-only foundation：旧 `SceneRenderer` 尚未吸收 neutral device，semantic manifest/load/upload 尚未由产品 frame owner编排，retirement limit尚未接产品 memory/profile配置，产品 device owner swap、旧 registry drop、shutdown/device recreate调度和全局 telemetry仍未接线。没有 managed Cargo、synthetic/真实 device-loss 注入、真实 WGPU、RenderDoc、GPU/RSS/VRAM/功耗或截图证据，因此 M3 仍不得提交 milestone或发送企微通知。

direct lighting/clustered light grid/shadow已由09E审查，environment/sky/IBL/reflection probe已由09F1审查；下一审查单元进入09F2 baked lighting/lightmap/irradiance volume。09D把texture/mesh/material的artifact、residency、upload、budget与retirement边界交给后续光照系统消费，不把shadow atlas、probe更新和lighting算法混进通用streamer owner。
