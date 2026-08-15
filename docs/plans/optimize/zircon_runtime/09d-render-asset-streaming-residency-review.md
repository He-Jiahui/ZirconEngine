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
implementation_status: pending
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

本篇review完成、implementation pending。没有修改production source，没有运行Cargo、Editor、cook或真实GPU。focused source fingerprint、文件/行数和函数行为都是2026-08-15读取时快照；相关graphics/asset/importer目录存在其他Session改动，M0必须重查后才能实施。

direct lighting/clustered light grid/shadow已由09E审查，environment/sky/IBL/reflection probe已由09F1审查；下一审查单元进入09F2 baked lighting/lightmap/irradiance volume。09D把texture/mesh/material的artifact、residency、upload、budget与retirement边界交给后续光照系统消费，不把shadow atlas、probe更新和lighting算法混进通用streamer owner。
