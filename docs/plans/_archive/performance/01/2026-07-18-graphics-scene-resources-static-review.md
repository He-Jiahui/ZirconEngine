---
related_code:
  - zircon_runtime/src/graphics/scene/resources
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/render_asset.rs
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/godot/core/io/resource_loader.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/TextureStreamingHelpers.cpp
tests:
  - complete graphics scene resources directory sixty-three of sixty-three current Rust files reviewed, 8034 lines
  - standard material uniform fixed-array source contract RED then GREEN
  - mesh temporary position-vector source contract RED then GREEN
  - rgba8 mip upload lazy-iterator source contract RED then GREEN
  - per-frame unique resource ensure source contract RED then GREEN
  - material dependency stable-cache regression contract RED then GREEN
  - shader stable-cache source contract RED then GREEN
  - material ABI allocation source contract passed
  - scoped rustfmt and diff check passed
  - current-source Cargo, F2 pixels, scale counters, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene resources逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`graphics/scene/resources/**`当前63/63个Rust文件、8,034行：root 2/169、fallback 3/138、gpu material uniform 2/445、gpu mesh 8/343、gpu model 3/172、gpu texture 4/1,073、output target 3/452、pipeline 3/243、post-process LUT 2/129、prepared 8/117、resource streamer 22/4,557、runtime 3/196。压缩纹理的多mip/array上传当前由资产readiness显式拒绝，未把单mip上传实现误报为已支持的正确性问题。本记录只声明逐文件静态覆盖、源码合同与局部修复，不声明current-source动态验收。

## 本轮直接止损

标准材质uniform原先把固定48个`f32`再次复制为192B heap `Vec<u8>`；现返回`[u8; 192]`栈数组并直接写GPU。mesh准备原先把转换后的`GpuMeshVertex`位置再复制为`Vec<Vec3>`，仅供bounds和wireframe计算；现两个消费者直接读取GPU vertex slice，删除每顶点12B临时副本。RGBA8 mip upload计划原先先收集`mip_count * layer_count`个descriptor `Vec`再写入；现改为无堆分配迭代器，顺序测试显式collect只留在测试端。

`ensure_scene_resources`原按实例调用mesh/model/material/sprite texture ensure，重复实例会重复revision/registry/cache探测；cookie和irradiance texture也逐条重复。现按帧维护唯一resource id集合或readiness map，每个唯一mesh/model/material/cookie/irradiance/sprite texture至多ensure一次，同时逐实例material/sprite统计语义不变。

材质缓存命中原对每个纹理依赖重新`load_texture_asset`并重算upload readiness。现`PreparedMaterial`记录设备`TextureUploadSupport`，稳定命中只比较material revision、support和locator对应的texture id/revision；能力或依赖revision变化才重新加载验证。同一材质多个slot指向同一texture时也只ensure一次。shader准备同样把有效shader的id/revision命中前移到`load_shader_asset`之前，使稳定shader及递归imports不再加载完整资产。材质ABI正常路径另删除bind-group和每个binding的临时`Vec`，以迭代器首项+重复计数保持原诊断行为。

## PERF-MVP-404：PreparedSceneResourceSet与有界异步驻留

剩余主瓶颈是资源准备仍在render submission线程同步完成。frame先从实例列表找资源，随后每个ensure独立读取registry revision；cache miss会同步load/clone/decode/hash/转换并创建texture/buffer/sampler/bind group。model cache返回深clone的`ModelAsset`，mesh首次准备总是计算完整vertex hash、bounds与wire segments，即使wireframe未启用；material重建又分别解析parent chain、shader contract、texture readiness、ABI和诊断集合，并为两个uniform创建GPU buffer。应由Runtime04发布asset-event/revision generation的批量只读snapshot和依赖DAG，由Render02/08/13消费唯一`PreparedSceneResourceSet`，禁止实例循环自行进入资产管理器。

CPU I/O、解码、mesh hash/bounds/可选wire生成、material/shader依赖解析与upload command构建必须进入有界single-flight asset jobs；render线程只按每帧byte/object/time预算应用ready uploads，未完成时复用last-good或device共享fallback。artifact键至少覆盖resource id/revision、device capability/layout generation和dependency revisions；稳定帧asset load/decode/validation/GPU create/upload均为0，1%变更只触及dirty依赖。Bevy以`AssetEvent`只extract added/modified并用bytes-per-frame limiter延迟GPU prepare；Godot用共享thread-load task和worker pool处理依赖等待；UE把streaming更新分帧、显式限制temp/pool预算并暴露pending/cached/overbudget统计，三者共同说明当前逐帧同步ensure不是目标形态。

纹理owner还应缓存device级sampler和texture/view/sampler generation binding，optional fallback/output converter按compiled feature懒创建；compressed mip/layer容器先在Render13形成显式level layout artifact再开放上传。output-target writeback当前每帧创建conversion bind group和独立command encoder，并单独`queue.submit`；应并入Render01主graph/encoder，缓存conversion binding并只在目标generation变化时重建。UI rich text texture id在prepare与draw重复解析，locator miss还扫描全registry并为每record重算derived id；Render14/Runtime04应发布locator/derived-id O(1)索引和共享解析artifact。诊断用排序、capture seed和大readiness clone只在显式诊断/capture gate开启时生成。

## 验收预算

按instances 0/1/1k/100k、unique mesh/model/material/texture/shader 1/10/1k/10k、重复率0/50/99%、asset size 4KiB/4MiB/256MiB、stable/1% revision change/hot reload/device loss、upload budget 1/16/64MiB、UI locators 1/1k/100k、output targets 0/1/8记录registry lock/probe、asset load/decode/clone/hash/validation、job queue age/drop、CPU allocation bytes、GPU object create、upload call/bytes、command encoder/submit、fallback/last-good与CPU/GPU p95。当前相同frame resource ensure<=1/unique id，稳定material texture asset load=0，稳定valid shader asset load=0，标准uniform/mip-plan/ABI临时heap=0；最终stable准备重活/create/upload=0、registry批量snapshot<=1/frame generation、changed工作近dirty resources、render线程asset I/O/decode=0、upload队列有界、output writeback额外submit=0。focused Cargo、F2像素、规模counter、GPU timestamp和DX12 RenderDoc完成前保留在`pending.md`，不进入`review.md`。

本轮尝试`validate-matrix.ps1 -Package zircon_runtime -SkipTest`，验证器在Cargo启动前于脚本第187行解析协调器输出时触发`ConvertFrom-Json`错误（首字符不是JSON），因此没有产生check结果；该工具入口故障只作为动态验收阻塞记录，不计代码失败或通过。
