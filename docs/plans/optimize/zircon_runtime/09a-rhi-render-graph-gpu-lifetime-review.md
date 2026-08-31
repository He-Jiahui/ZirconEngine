---
related_code:
  - zircon_runtime/crates/zr_rhi/src
  - zircon_runtime/crates/zr_rhi_wgpu/src
  - zircon_runtime/src/render_graph
  - zircon_runtime/src/graphics/backend/render_backend
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/pass_authoring.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/runtime/render_framework
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_submission_completion_journal.rs
  - zircon_runtime/src/core/framework/render/scene_submission_completion.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene
  - zircon_runtime/src/dynamic_api/session
  - zircon_app/src/entry
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-15-rhi-wgpu-submission-gpu-lifetime-current-architecture-review.md
  - docs/plans/performance/02/2026-08-15-render-graph-current-architecture-review.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/RHICommandList.h
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Private/RHIResources.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderCommandFence.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderCommandFence.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphAllocator.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphResourcePool.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/D3D12RHI/Private/D3D12Submission.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/D3D12RHI/Private/D3D12Allocation.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResources.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceRegistry.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Compiler/NativePassCompiler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Compiler/ResourcesData.cs
  - dev/godot/servers/rendering/rendering_device_graph.h
  - dev/godot/servers/rendering/rendering_device_graph.cpp
  - dev/godot/servers/rendering/rendering_device_driver.h
  - dev/godot/servers/rendering/rendering_device.h
  - dev/bevy/crates/bevy_render/src/renderer/mod.rs
  - dev/bevy/crates/bevy_render/src/renderer/render_context.rs
  - dev/bevy/crates/bevy_render/src/error_handler.rs
  - dev/bevy/crates/bevy_render/src/gpu_readback.rs
  - dev/Fyrox/fyrox-graphics/src/server.rs
  - dev/Fyrox/fyrox-graphics/src/read_buffer.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: partial_neutral_rhi_batch_upload_registry_lifetime_batched_completion_observation_poll_receipt_bounded_asset_retirement_asset_device_epoch_recovery_bounded_scene_submission_terminal_journal_source_only
source_recheck_required: true
---

# 09A · RHI / Render Graph / GPU Lifetime 工程化差距

## 1. 结论

Zircon 的图形底层不是空壳。当前已经有中立的 buffer/texture/sampler/bind group/shader/pipeline descriptor、typed handle、command list validation、capability DTO、Render Graph builder、依赖拓扑、pass culling、first/last-use lifetime、逻辑 transient allocation plan、WGPU transient pool、并行 encoder bucket、三槽异步 readback queue、GPU timestamp/pipeline statistics、RenderDoc marker、surface presenter和一批产品离屏测试。这些工作提供了可迁移的词汇、局部算法和测试，不应在重构时退回直接散写WGPU命令的更简易实现。

但当前产品没有真正使用公开RHI。`zr_rhi::RenderDevice`唯一实现是 `zr_rhi_wgpu` 中只在 `#[cfg(test)]` 编译的 `DeterministicRhiContractDevice`；它用单一mutex内的HashMap和CPU字节数组模拟资源，submit同步执行并立即完成fence。真实 `RenderBackend`直接拥有并向渲染器、资源、UI、surface和readback暴露 `wgpu::Device/Queue`。本轮在graphics/UI/WGPU三个范围内检出728处 `wgpu::Device`、250处 `wgpu::Queue`、40个文件含 `queue.submit`，其中虽包含测试和类型签名，仍足以证明产品权威没有通过一个可审计的RHI admission、submission和lifetime边界。更严重的是，当前 `RenderDevice::create_command_list(..., label: impl Into<String>)` 使trait本身不具备对象安全性，产品即使想注入 `dyn RenderDevice`也不能直接做到。

中立合同也不足以表达已声明的能力。command list只有buffer-buffer、buffer-texture、texture-buffer copy、render pass、直接draw和dispatch；没有texture-texture copy、显式subresource/barrier状态、indirect/multi-draw、compute pass、queue wait/signal、surface acquire/present、device loss、异步completion、heap/budget或ray tracing命令。与此同时capability DTO声明multi-draw、sparse texture、acceleration structure和三个queue class。能力表与可执行命令合同不闭合，产品backend还在确实能创建viewport surface时以 `supports_surface=false` 生成caps。这不是“后续补几个方法”能解决的小缺口，而是RHI contract、backend实现和产品消费三者没有同一事实源。

Render Graph同样完成了局部数据结构，却未达到RDG正确性。`RenderPassId`、texture/buffer/external handle都只是builder-local `usize`；没有builder/execution generation，外来builder的同索引handle可以通过范围校验。资源也没有SSA/version。compiler要求连续writer之间存在人工可达关系，只从latest writer推RAW，不能从资源访问自动推完整RAW/WAR/WAW和subresource transition。反向culling维护一个不消费版本的 `needed_resources` 集合，因此一次需要某资源后，所有更早写入都可能被保留；产品authoring又给每个pass强行添加 `previous -> pass`，把本应由资源依赖形成的DAG退化为总链，掩盖了错误并消灭async/parallel宽度。

提交和寿命也没有工程闭环。默认Editor走同步submission；可选pipelined模式只是私有OS线程加容量1 channel，producer每帧等待N-1完成和N开始，worker持有framework operation lock执行整次提交。compiled scene先submit graph，再由 `ViewportSurface::present_texture`每帧新建bind group和encoder做第二次submit；output writeback、UI external copy、retained cache和多个资源路径还有独立submit。transient资源在 `queue.submit(command_buffers)` 后立刻归还exact-descriptor CPU frame pool，没有submission ticket或device generation；在当前单WGPU queue顺序下通常可复用，但它无法证明多queue、device recreate、异步readback和真正deferred destruction下的安全。

产品readback仍有多条同步停机路径。buffer/texture/IBL helper逐次分配staging buffer和encoder、独立submit、`device.poll(wait_indefinitely())`、阻塞mpsc并复制到Vec。新的三槽queue是正确方向，但没有每帧request/byte/age上限、submission ticket、device identity、device-loss终态或shutdown drain；它自身、GPU timer和pipeline statistics又都可以poll设备。全产品没有 `set_device_lost_callback`、`on_uncaptured_error`、device generation或统一retirement owner。surface只局部处理Lost/Outdated，无法让在途ticket、资源handle、cache和UI进入可证明终态。

本轮登记6项P0、12项P1、5项P2。P0首先硬切为唯一产品RHI和GPU generation owner、versioned Frame Graph、单一submission packet/ticket、completion驱动资源退役及全异步readback；P1再收敛adapter/surface/capability、compile/cache、transient memory、UI和diagnostics；P2才扩展多backend、多GPU、显式多queue、稀疏驻留与光追。完成动态验收前，现有“RHI/RDG/transient pool/pipelined submission complete”只能表示局部代码或静态测试存在，不能作为工程级渲染底层完成证明，更不能支持性能优于Unreal的结论。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 范围 | 文件 | 物理行 | `#[test]` | 证据等级 |
|---|---:|---:|---:|---|
| `zr_rhi` | 15 Rust | 4,797 | 35 | E3：capability、descriptor、handle、command、surface/UI contract与tests |
| `zr_rhi_wgpu` | 61 Rust | 18,934 | 214 | E3：deterministic contract、validation、timer/statistics/readback、retained UI与tests |
| `render_graph` | 16 Rust | 5,243 | 49 | E3：builder、compile、culling、lifetime、alias plan、dump/lint与tests |
| `graphics/backend` | 24 Rust | 3,131 | 29 | E3：真实WGPU device、adapter、surface、readback和IBL路径 |
| 产品接线 | targeted E3 | - | - | pass authoring、compiled cache、framework submission、graph materialization、transient pool、scene submit/present与dynamic profile |

RHI统计与2026-08-15 current-architecture报告的76/76、23,731行、249个测试一致；Render Graph统计为16/16、5,243行、49个测试。Graph测试中有7处 `include_str!` 源码形态断言，覆盖builder范围检查、compile数据结构和alias实现文本，不能代替跨builder错误、随机DAG、device loss、GPU completion和规模性能行为。

产品符号普查覆盖 `zircon_runtime/src/graphics`、`zircon_runtime/src/ui`、`zr_rhi_wgpu/src`：728处 `wgpu::Device`、250处 `wgpu::Queue`、66处 `queue.submit`命中分布在40个文件、12处 `device.poll`命中分布在8个文件。数字包含测试和同文件的`#[cfg(test)]`段，只用来界定绕过面，不直接等价于每帧实际调用次数。静态产品路径可证明普通presented scene至少一次graph submit加一次surface blit submit；额外writeback/readback/UI/upload按启用功能增加。

图形目录当前有大量其他Session修改，包括RHI lib/UI、WGPU timer/UI、Render Graph主要文件、backend构造以及新增compute/UI文件。本报告按current source承认这些改动带来的三槽readback、共享UI image、parallel encoder和graph优化，但实现前必须重取文件指纹、复核overlap diff和既有failure/output records，故标记 `source_recheck_required`。

### 2.2 参考引擎边界

- Unreal RHI/RDG是本篇的工程上限主参考。`FRHICommandList`覆盖transition、copy、indirect/multi-draw、GPU fence与RHI thread dispatch；`RHIResources.cpp`把引用归零后的资源放入pending delete队列并在RHI生命周期点统一删除；D3D12 submission把finalized command payload、per-queue fence、cross-queue sync point、query resolve、completion和hang timeout放在同一submission pipe。RDG从输出/never-cull根做引用可达性，编译prologue/epilogue barrier、async compute、render-pass merge和transient资源操作。Zircon不应复制UE类层次，但必须吸收“immutable command payload + queue ticket + completion retirement + compiled barrier/lifetime”的边界。
- Unity Graphics RenderGraph是version和native pass编译主参考。`ResourceHandle`同时编码index、version、resource type和execution validity，明确防止pool中旧pass data跨graph execution误用；每次write产生新version，每version只有一个writer并记录readers。compiler按version裁剪unused producer、计算first/last use和cross-queue fence，尝试合并兼容raster pass/subpass，并把资源创建/释放绑定到编译后的使用区间。Zircon无需照抄16-bit打包，但不能继续用无generation、无version的裸usize。
- Godot RenderingDevice不是RDG替代品，但其driver/command graph证明中型引擎底层也需要真实resource usage和barrier合同。它区分copy、uniform、indirect、storage、sample、attachment、acceleration-structure usage，生成memory/texture/buffer/AS barrier；driver公开graphics/compute/transfer queue family、command pool/buffer、secondary buffer、fence/semaphore、swapchain color space/HDR、indirect draw/dispatch和ray tracing。Zircon当前capability布尔值远多于command contract，不能以WGPU后端为由把这层长期留空。
- Bevy当前主仓已由旧node graph转为ECS `RenderGraph` schedule，因此不作为versioned RDG正确性依据；但它把各render system的encoder汇总到 `PendingCommandBuffers`，在task pool并行finish并恢复原拓扑顺序后统一submit。它还注册device-lost/uncaptured-error callback、提供adapter name/fallback/power/features/limits配置，并让GPU readback异步map后回主World发布。说明WGPU并不强迫Zircon使用多个直接submit/poll owner或同步readback。
- Fyrox graphics server是较小的OpenGL抽象，可借鉴的是object-safe `Rc<dyn GraphicsServer>`、资源wrapper和可poll的 `GpuAsyncReadBufferTrait`。它没有RDG、显式多queue或现代deferred destruction，不能用作降低Unreal/Unity/Godot基线的理由；反而说明即使较小引擎也没有把“异步readback”实现成每次全局finish。

### 2.3 明确未做

- 没有修改production code，没有运行Cargo、Editor、App、真实GPU、RenderDoc、PIX、WPR/xperf、GPU timestamp capture、device-loss注入、长时间soak或规模benchmark。本篇是current-source静态审查和重构计划，不是实现或性能验收。
- 没有要求Zircon立即支持Unreal全部RHI backend、multi-GPU、Nanite/Lumen或所有平台。P0/P1只要求正确的owner、contract、lifetime、submission、device failure和可规模化复杂度；高级backend/feature在核心完成后进入P2。
- 没有否定现有descriptor、validation、graph dump、transient interval coloring、三槽readback、GPU timer、parallel encoder和共享UI image registry。它们是迁移输入，但必须接入同一device generation、dense compiled slot、submission ticket和memory owner，不能继续作为平行生命周期域。

## 3. 当前闭环与必须保留的能力

### 3.1 中立descriptor与deterministic validation适合作为contract test oracle

buffer/texture/sampler/bind group/shader/pipeline descriptor和command validation已经覆盖大量非法资源、render pass状态与copy范围。目标RHI应让这些测试继续对真实product-used contract运行；deterministic实现可以保留为明确的test double，但不能再以crate名 `zr_rhi_wgpu` 暗示它是生产WGPU backend，也不能由它独占所有RHI行为测试。

### 3.2 Graph已有声明/编译/执行分层雏形

builder只声明pass、resource和access，compiled graph持有顺序、依赖、culling、lifetime和stats，执行期再物化WGPU资源。这个分层方向正确。重构应升级handle identity、resource version和compiled packet，不应让executor重新掌握任意资源创建和依赖拼接权。

### 3.3 三槽readback和parallel encoder证明局部机制可复用

readback queue已有256-byte对齐、power-of-two扩容、3个frame slot和连续240低利用帧后缩容；parallel encoder按topology layer分桶、在TaskPool完成record并保持输出顺序；compiled scene会把IBL writeback command buffer并入主submit。这些机制应迁入统一submission batch，而不是删除后再写一套更弱实现。

### 3.4 typed retryable surface result优于静默错误

retained UI presenter已经把Lost/Outdated/Timeout/Occluded映射为 `RetryableNoSubmit`，并在acquire成功后用一个encoder/submit完成present。scene `ViewportSurface`还没有同等级结果，但目标应统一为typed acquire/present outcome，不退回日志或 `Ok(())`混合“成功提交”和“本帧未提交”。

## 4. P0 差距清单

### P0-1：公开RHI没有生产实现，合同还是非对象安全的测试系统

`zr_rhi::RenderDevice`唯一 `impl` 位于 `zr_rhi_wgpu/src/device.rs`，而该module和所有validation module现在都只在 `#[cfg(test)]` 编译。实现以一个 `Arc<Mutex<State>>` 持有descriptor/data HashMap，submit在CPU执行并立即把 `completed_fence`推进到新值。真实RenderBackend完全绕过它。`create_command_list`的泛型label参数又使trait不能形成 `dyn RenderDevice`，全仓没有production `Arc/Box<dyn RenderDevice>` consumer。

目标必须做一次二选一硬切：要么实现产品实际使用的 `WgpuRhiDevice`，并让renderer/resource/surface/UI/readback都只消费该RHI；要么删除当前伪公共合同，以新的product-used RHI contract原位替换。不能保留“测试RHI + 产品WGPU”双真相。新接口必须对象安全或明确采用sealed backend enum/generic composition，不能同时声称可插拔又无法动态持有。

### P0-2：没有device generation、错误监督和GPU completion驱动的资源退役

所有RHI resource handle只是公开可构造/可取raw的 `u64`；fence只是 `FenceValue(pub u64)`，不含device、queue或generation。产品WGPU资源主要依赖Rust Drop，transient资源在submit后立刻进入CPU pool；UI image/cache又各有资源表。全仓没有device-lost callback、uncaptured-error owner、admission stop、in-flight ticket failure、deferred destruction queue或rebuild/terminal policy。

目标建立唯一 `RhiDeviceGeneration` 和 `GpuResourceRegistry`。handle至少含device generation、slot与slot generation；每次create/import产生registry record，每次compiled packet记录last-use submission ticket。destroy只是停止admission并进入retire queue，只有相关queue completion到达后才释放native resource。device loss必须原子停止新packet，给所有未完成ticket/readback/pipeline request发布exactly-one terminal result，废弃旧generation cache和surface，再按profile决定重建或退出。stale handle必须返回typed `StaleDeviceGeneration/ResourceRetired`，不能交给WGPU后才报错。

### P0-3：Render Graph资源无version/execution identity，依赖、覆盖写和culling不具备RDG正确性

Graph handle只有kind-local usize，builder校验只检查index是否小于本builder计数；另一个builder的同index handle可被接受。write不返回新version，compiler先要求writer之间有人工作为manual dependency，再按manual order维护一个latest writer推RAW。没有WAR自动边、同pass读写phase、mip/layer/aspect范围、alias barrier或queue ownership transfer。culling的 `needed_resources` 一旦插入不会在找到对应producer后消费版本，因此被后续overwrite的旧writer可能错误保留；manual dependency又会继续保留无数据必要性的producer。

目标采用versioned handle，例如 `RgTextureVersion { graph_generation, resource_slot, version }`，每次write产生唯一新version；read引用精确version或由builder返回的当前version。compiler由access记录自动生成RAW/WAR/WAW、subresource state和queue transition，manual edge只表达资源外语义。culling从external/persistent/readback/side-effect roots沿version producer和显式边反向遍历，只保留真正可达的pass/version。跨builder、跨frame、旧version、同pass非法phase和未定义read必须在compile前给typed error。属性测试覆盖随机DAG、覆盖写、分支、merge、discard/load和subresource组合。

### P0-4：产品pass authoring强制总序，async queue只是标签，提交模型无法形成真正pipeline

`pass_authoring.rs`用单一 `previous` 给全部stage/pass添加相邻dependency；还用嵌套scan预排unique producer，并以硬编码executor id修正Bloom顺序。这样compiled graph即便拥有topology layer和parallel encoder，也通常只有宽度1。`QueueLane::AsyncCompute/AsyncCopy`只进入metadata/stats和同一WGPU queue的command buffer；没有queue family、wait/signal或ownership transfer。默认Editor同步执行；可选worker只有容量1，producer等待前一帧完成和当前worker start，worker持framework operation lock，既不是TaskGraph affinity executor也不是多帧RHI pipe。

目标让pass顺序由versioned resource edge、明确side-effect edge和queue capability编译得到，删除全局 `previous -> pass` 与executor-name特判。record可按ready layer并行，submit阶段接收immutable `RhiSubmissionBatch`，其中包含ordered command packets、queue lane、wait/signal ticket、present和completion actions。WGPU backend可以把逻辑lane合法降级到单queue，但必须报告fallback并保持dependency；未来native D3D12/Vulkan backend再映射真实queue。render/RHI执行权接入Performance02 M3和共享affinity executor，删除容量1私有线程模型。

### P0-5：提交权已部分收口，但typed boundary与统一frame batch仍未闭环

2026-08-30 current-source复审已取代“present固定第二次submit”和“scene clear独立submit”的旧判断：compiled scene的present copy、history initialization、graph buffers、diagnostic tail和IBL writeback现共享terminal scene packet；frame resource upload先进入WGPU submission queue，当前`WgpuSubmissionService::flush`会把连续upload后的command buffers归入同一次原生submit。纹理resident mip迁移仍存在必须先copy旧mip、再写新mip的`command -> upload`依赖，该路径由`TexturePreUpload`独立ticket表达并形成真实物理边界；环境捕获、hit proxy和启动期system texture属于显式任务/启动边界，文本product framebuffer调用只存在于测试。上述是源码拓扑，不是运行次数结论。

normal frame已有`RenderFrameSubmissionTransaction`枚举pre-scene producer和terminal scene ticket；11.9又把当前唯一已证明必须切开的纹理mip保留路径提升为typed boundary reason，11.10保证hit-proxy只有在diagnostic admission完成后才接纳upload并在terminal submit成功后commit GPU-scene状态，但仍未达到完整`RhiSubmissionCoordinator`：wait/readback/retire packet与其它独立任务尚未统一表达，retained UI与显式capture仍有独立frame/task owner。底层此前虽累计native submit与ticket计数，frame receipt没有同一区间投影，导致无法直接验证普通帧是否一次physical submit；11.8先补齐该观测合同，动态WGPU/RenderDoc基线仍是P0-5下一验收门。

目标只有 `RhiSubmissionCoordinator` 可调用backend submit。所有upload、graph、surface blit、UI composition、readback copy和writeback先形成packet并进入同一frame batch；无法合并必须给typed reason和独立ticket。surface target优先直接成为graph external output，避免无必要offscreen-to-surface blit；确需blit时也并入同batch。每帧报告logical packet、physical backend submit、queue、wait、bytes和reason，验收普通单viewport无readback路径在backend允许时为一次physical submit。

### P0-6：普通产品readback已统一，显式阻塞与独立设备completion边界仍未完全收口

2026-08-30 current-source复审取代“多个产品helper逐调用无限等待”的旧判断：`read_buffer_*`、`read_texture_*`和旧同步IBL batch在module级均由`#[cfg(test)]`隔离；产品viewport/HDR capture与hit-proxy、IBL/query diagnostics进入generation-qualified bounded diagnostic service。正常scene frame由唯一`poll_frame_submission_completions`推进backend receipt并同步路由scene journal、IBL、typed query和timer/statistics；viewport-pick poll为non-blocking try-lock路径且复用同一fan-out。显式同步capture只在已提交diagnostic未drain时进入30秒deadline循环，每次poll receipt仍经过同一consumer router。当前产品`wait_indefinitely`只剩明确的RenderDoc/Xcode capture stop控制边界。shared-device retained UI不创建第二个timer/readback timeline并经共享`WgpuRenderDevice`提交；standalone native UI保留独立tool device profile，但已硬切到该profile内部唯一`WgpuRenderDevice` submission/completion owner，不再raw submit或由readback queue直接poll。

剩余目标不是再次迁移已test-only的helper，而是把显式capture的deadline/cancel/consumer executor写入统一typed policy，并用真实device loss/shutdown、surface、diagnostic delivery和task callback验收exactly-one terminal receipt。Standalone UI的ticket-qualified image pin retirement和fault/surface源码终结已完成，仍需真实窗口故障注入验证。普通frame继续禁止blocking wait；唯一completion owner、每帧/单请求/in-flight/result-ring字节与数量预算、submission-qualified map和bounded delivery已存在，后续只按真实fault/scale/profile证据补缺，不能用旧三槽queue描述指导新优化。

## 5. P1 差距清单

### P1-1：RHI命令面与capability surface不闭合

capability声称graphics/compute/copy queue、multi-draw、sparse texture和acceleration structure，但command list无法发出其中多项操作；反过来surface/present不属于RenderDevice。目标为每个capability建立可执行command/descriptor/limit/fallback和contract test，删除无consumer或不可表达的布尔值。命令至少覆盖texture copy/resolve、indirect/multi-draw、compute pass、barrier/subresource、query、queue sync和present；ray tracing/sparse可保持disabled，但不能只在caps中“预留完成”。

### P1-2：adapter/device协商是硬编码策略，不能形成可部署的fallback tier

产品默认只请求HighPerformance adapter，没有adapter name、backend、fallback/software、battery或headless policy；device构造无条件请求 `RG11B10UFLOAT_RENDERABLE`，即使adapter不支持，测试还锁定这一行为。其它feature按支持情况选择，但没有feature tier、拒绝原因或降级配置。Bevy参考已提供adapter name/fallback/power/features/limits策略；Zircon需要versioned `RenderDeviceProfile`，按required/preferred/disabled feature和limit tier协商，并发布最终选择与fallback cause。缺可选postprocess格式时应切换可验证格式/效果tier，只有真正required baseline不满足才拒绝启动。

### P1-3：capability truth和surface合同存在直接错误

真实RenderBackend能够创建ViewportSurface，却给 `wgpu_backend_caps`传 `supports_surface=false`。`RenderNativeSurfaceTarget`只有Win32，non-Windows retained UI明确失败；这与跨平台引擎目标不符。目标caps从同一adapter/device/surface profile生成，区分“device支持surface”“当前target有compatible surface”“当前swapchain配置”。native target应覆盖winit支持的平台或使用安全lifetime-owned window target；raw handle路径必须有可验证owner lease。

### P1-4：surface/present缺HDR、color space、frame pacing和统一恢复状态机

ViewportSurface固定偏好sRGB/AutoVsync或Fifo、每帧新建present bind group，Lost/Outdated重configure后返回 `Ok(())`，Timeout/Occluded同样返回 `Ok(())`，调用方无法区分已提交和无提交。没有HDR monitor/output transfer、color space、max frame latency、VRR/tearing、present policy或resize/device recreate generation。Godot driver已暴露swapchain color space/HDR/max FPS；Zircon目标surface owner要发布typed acquire/present outcome、frame token、configuration generation和monitor caps，并与App cadence/frame pacing统一。

### P1-5：compile在framework全局锁内执行，cache把动态尺寸写进16-entry拓扑缓存

`compile_submission_pipeline_with_options`先 `framework.lock_state()`，随后在 `get_or_compile_with_status` miss closure内完成整个pipeline compile和capability validation。cache容量固定16，key包含view/render width/height和texture target尺寸，动态分辨率、多viewport、resize和camera组合会抖动并在锁内反复compile。目标分离稳定 `FrameGraphSchemaKey`（pipeline revision、feature set、format/sample topology、cap tier）与每帧 `FrameGraphInstance`（extent、imports、dynamic constants）；miss compile在锁外完成，最后以generation compare-and-publish。cache按bytes/compile cost/last use治理并支持single-flight、cancel和async prewarm。

### P1-6：graph authoring和compiled DTO仍以String为主键并在热路径深clone/查找

builder、lifetime、access、compute binding、workload和metadata都持有owned String；compile重建name HashMap并clone；stage执行对每个entry以pass name扫描compiled pass，profile/record又clone pass name和resources。资源解析还保留by-name接口。目标compile期intern所有pass/resource/executor/pipeline label，发布dense `PassSlot/ResourceSlot/ExecutorSlot`；String只用于debug table和离线dump。执行packet以连续ranges索引access/barrier/diagnostic，稳定帧不分配label或重建name map。

### P1-7：逻辑alias plan与物理pool是两个authority，hash可碰撞且overflow被伪装成最大值

compiled graph按完整descriptor bucket做interval coloring，但slot从每bucket重新编号，reservation只以 `(kind, slot, 64-bit FNV hash)`聚合；理论hash碰撞可把不兼容descriptor预算合并。texture storage size overflow被改成 `u64::MAX`而非compile error。执行层又以另一套exact-descriptor BTreeMap池重新决定create/reuse，逻辑plan并不直接绑定物理allocation。目标compiled plan保留完整interned compatibility key或collision-checked key id，overflow立即失败；physical pool消费同一slot plan和backend alignment/heap class，不再第二次独立决策。

### P1-8：transient pool只有CPU帧龄/固定预算，没有GPU completion、heap压力或跨viewport全局治理

pool按descriptor精确复用，texture 256 MiB、buffer 64 MiB，CPU 8帧后淘汰；submit后立即release，预算只统计pool内闲置资源，不含active/persistent/UI/upload/readback/pipeline内存。多个renderer/viewport可能各有pool。目标由单一GPU memory manager按device generation统计requested/committed/active/pending-retire/pool/resident bytes，预算依据adapter memory和profile而非硬编码；退役由ticket，淘汰综合completion、last use、priority和memory pressure。exact descriptor可作为backend不支持heap alias时的保守fallback，但必须可观测。

### P1-9：store lint和attachment ledger是每帧重扫，不是compile artifact

`store_lint_report()`对每个attachment access再次扫描prior/future passes并clone pass/resource String，复杂度可达O(P²*A)；`update_stats`每个submitted frame调用它却只取row count。attachment bandwidth ledger按base extent估算，忽略mip链、压缩、tile/load-store和backend实际行为。目标在compile期生成lint bit/count和静态ledger模板，frame只增量填充实际extent/sample/feature值；详细rows仅observer请求时materialize。真实带宽用GPU counter/平台工具校准，估算字段明确标注model version和误差。

### P1-10：GPU timer/statistics/readback重复poll和字符串聚合，启用观测会改变调度

timer和pipeline statistics各保留64 scope、每pass `to_string()`；两者 `try_collect`分别调用readback queue poll，pipeline statistics再按name线性find聚合同名scope。instrumentation会禁用部分parallel recording。目标由compiled dense diagnostic slot一次预分配query ranges，submission ticket关联frame generation；一个completion pass poll、decode并写sealed arrays，多个observer共享。观测对排程的影响必须按feature精确声明，不以全局禁用并行作为默认。

### P1-11：retained UI仍可自建第二套device并拥有独立submit/lifetime域

产品with-context路径能共享RenderBackend device，这是应保留修复；但 `WgpuUiSurfacePresenter::new` 的native fallback仍创建新Instance/Adapter/Device/Queue，external image copy每次新建texture/encoder并立即submit，retained cache还有多个独立submit。目标Editor/App native surface必须从canonical RHI device generation取得surface和image lease；standalone fallback只能是明确独立tool profile并禁止跨device external image。UI copy/composition进入frame packet，共享resource registry和retirement。

### P1-12：测试数量较多，但错误、寿命、规模和产品证据不足

deterministic RHI的249个测试无法证明真实WGPU fence/Drop/surface/device loss；Graph 49个测试中7个只断言源码文本。没有foreign-builder handle、random DAG/property、hash collision、device recreate、queue reorder、readback over-budget、surface HDR、多viewport、10k frame RSS、2k pass compile或GPU completion race证据。目标测试矩阵见第9节，source-shape test只能守架构禁令，不能充当行为验收。

## 6. P2 差距清单

### P2-1：多backend和native low-level RHI尚无产品路线

先让WGPU backend完整消费统一RHI，再在相同contract下评估D3D12/Vulkan/Metal native backend。不能在P0未完成时复制三套直接device owner。P2要求backend conformance、shader/capability tier、pipeline cache和cross-platform capture一致。

### P2-2：真实多queue、multi-GPU和显式frame overlap尚未建立

WGPU逻辑上只有一个queue，AsyncCompute/Copy只能降级。P2 native backend可映射graphics/compute/copy queue family、timeline semaphore和queue ownership；multi-GPU需要node mask、resource replication/transfer、present affinity和device-group failure policy。前提是P0 packet/ticket已经与backend独立。

### P2-3：稀疏资源、virtual texture/geometry的heap residency没有统一底座

当前caps和descriptor可以标记sparse reserved，但RHI无page commit/decommit、residency set、budget/eviction fence。P2在统一memory manager上增加tile/page heap、feedback、priority和copy queue packet，不能让每个Virtual Texture/Geometry功能自建native allocation owner。

### P2-4：ray tracing、mesh shader和高级indirect pipeline只有capability词汇

P2补齐BLAS/TLAS build/update/compaction、SBT、trace rays、mesh/task dispatch、indirect count和shader feature tier，并纳入graph version/barrier/lifetime；不在P0阶段用空command或false capability占位。

### P2-5：跨版本pipeline cache、GPU crash dump和live diagnostics还未产品化

P2需要backend/driver/device/feature/schema hash绑定的磁盘PSO cache、atomic rollout/rollback、DRED/Aftermath或平台等价GPU crash breadcrumbs、远程capture安全和长期基线。RenderDoc marker是起点，不是crash/telemetry完成证明。

## 7. 目标架构

```text
App / Editor / Runtime frame producer
        |
        v
FrameGraphSchema (stable topology, versioned resources, dense slots)
        + FrameGraphInstance (extent, imports, dynamic constants)
        |
        v
CompiledFramePacket
  pass packets + barriers + queue edges + create/retire ops
  diagnostic slots + readback/upload ops + present token
        |
        v
RhiSubmissionCoordinator (single admission / ordering / completion owner)
        |
        +--> WgpuRhiDevice generation N
        |      queue fallback is explicit
        +--> future native backend generation N
        |
        v
SubmissionTicket / CompletionStream
        |
        +--> GpuResourceRegistry / TransientHeap / DeferredRetirement
        +--> Readback completion / diagnostics / surface result
```

### 7.1 核心身份与状态

- `RhiDeviceId`与 `RhiDeviceGeneration`：adapter/device重建即换generation；所有resource、pipeline、surface、ticket和diagnostic frame携带它。
- `RhiResourceHandle { generation, slot, slot_generation, kind }`：不可公开从raw随意构造；import需要validated external lease。
- `RhiSubmissionTicket { generation, queue, value }`：完成、失败、device lost和shutdown都可查询/订阅；bare u64不得跨queue解释。
- `FrameGraphGeneration`、`ResourceSlot`、`ResourceVersion`、`SubresourceRange`：builder/execution identity和SSA版本同时校验。
- `SurfaceGeneration/FrameToken`：acquire、submit和present只对同一配置generation有效；retry/no-submit是typed终态。

### 7.2 编译产物

- `FrameGraphSchema`只含稳定topology、feature/capability tier、format/sample class和debug intern table。
- `FrameGraphInstance`含per-frame extent、camera/view imports、history generation、external leases和dynamic dispatch constants。
- `CompiledFramePacket`含dense pass ranges、resource-version ranges、barrier batches、queue wait/signal、native render-pass merge hint、physical allocation slot、readback/upload/present actions。
- compiler生成所有correctness和lint artifact；执行期不再按String扫描依赖、resource或executor，不重新推alias/lifetime。

### 7.3 Product RHI合同

- product renderer只能通过object-safe或sealed typed `RhiDevice`创建资源和packet；`wgpu::Device/Queue`限制在backend implementation allowlist。
- capability是negotiated immutable snapshot，命令、descriptor、limit、fallback和测试一一对应。
- backend负责把logical queue/barrier/allocation plan映射到WGPU或native API；不支持的能力在compile/admission时typed降级或拒绝。
- `RhiSubmissionCoordinator`是唯一submit/poll/completion owner；surface、UI、upload、readback、diagnostic都提交packet，不直接调用queue。

### 7.4 GPU内存与寿命

- registry记录descriptor、native owner、resident bytes、last-use ticket、debug id、priority和state。
- transient allocator消费compiled physical slot/compatibility class；exact-descriptor、placed resource/heap alias和sparse page都是backend策略，不产生第二套逻辑plan。
- destroy/unload/device loss先stop admission，再等待或失败ticket，最后retire native object；frame count只能作为cache eviction提示，不能作为GPU completion证明。
- memory report覆盖active、pooled、pending-retire、readback、upload、UI、history、pipeline和external import；预算按device/profile配置。

## 8. 硬切迁移与里程碑

### 8.1 必须删除的旧路径

| 当前路径 | 硬切目标 | 删除判据 |
|---|---|---|
| test-only deterministic RHI作为唯一impl | product-used `WgpuRhiDevice` + test double | 产品frame通过RHI，deterministic仅tests |
| 非对象安全 `RenderDevice` | object-safe/sealed product contract | 可持有统一device owner，无泛型virtual方法 |
| public raw u64/usize handles | generation + slot generation handles | foreign/stale handle均typed拒绝 |
| unversioned graph write | write produces resource version | RAW/WAR/WAW/culling不依赖人工writer edge |
| 全局 `previous -> pass` | resource/side-effect-derived DAG | 产品graph ready width由真实依赖决定 |
| capacity-1 private submit thread | shared render/RHI affinity executor + packet queue | 私有线程/channel删除 |
| direct `queue.submit/device.poll` consumers | single coordinator | backend allowlist外命中为0 |
| `wait_indefinitely` product helpers | bounded async readback ticket | production命中为0 |
| logical alias + independent exact pool | one compiled allocation authority | physical materialization消费compiled slots |
| per-frame store lint scan | compile artifact | stable frame lint build/scan/alloc为0 |

### M0：current-source冻结、owner判词与动态baseline

重取RHI/Graph/产品接线fingerprint，复核其它Session改动和Render01/Performance02/Render17/Runtime11状态；恢复可运行Editor/offscreen baseline。记录normal frame physical submit、poll、device/queue owner、compile miss/hit、readback、RSS和GPU timeline。没有baseline时只允许继续contract/test工作，不宣称性能改进。

### M1：产品RHI合同与WGPU实现硬切

先写contract tests，定义对象安全device、resource handle、command packet、capability/limit和typed errors；实现真实WGPU backend。逐步迁移resource creation、command encoder和pipeline，最后删除产品对test-only RHI的假依赖。M1结束时一个真实offscreen frame必须完全经过RHI，direct WGPU只留backend内部。

### M2：device generation、错误监督与资源registry

注册device lost/uncaptured callback，建立generation状态机、registry、last-use ticket和deferred retirement。添加synthetic backend和可触发WGPU validation/device failure fixture，验证所有in-flight operation终态、stale handle、surface/cache rebuild和shutdown。

### M3：versioned Frame Graph正确性

替换裸handle和unversioned access；write返回新version，自动生成RAW/WAR/WAW/subresource edge，删除manual writer chain要求。重写culling、lifetime和barrier compile，加入foreign handle和property tests。完成后删除产品 `previous -> pass`，hardcoded Bloom/executor ordering只能以明确语义edge替代。

### M4：schema/instance分离与无锁compile/cache

把稳定topology与dynamic extent/import拆开，发布dense slot和intern table；compile在framework锁外single-flight，cache按generation/bytes/cost治理。稳定resize/dynamic-resolution不重新编译拓扑，feature/revision改变才生成新schema。

### M5：immutable submission packet与共享affinity执行

Frame Graph、upload、UI、readback、writeback和present全部生成packet；coordinator统一batch/submit/ticket/completion。接入Performance02 M3和共享render/RHI affinity executor，删除容量1 `zircon-render-submit`线程与同步双模型。WGPU单queue降级可保留，但逻辑queue统计和依赖必须真实。

### M6：统一transient heap、memory budget与retirement

compiled allocation plan直接驱动物理slot；registry统计全域GPU内存，pool回收绑定completion ticket。验证跨viewport共享/隔离政策、device recreate、budget pressure、alias safety和无hash碰撞。固定256/64 MiB变成profile默认值而非隐藏常量。

### M7：readback/upload全异步与bounded completion

迁移全部buffer/texture/IBL/capture/HZB/UI readback和upload staging；删除production `wait_indefinitely`与独立submit。实现entries/bytes/age/deadline/cancel/device-loss/shutdown终态和consumer executor。三槽ring可作为backend实现基础，但ticket必须绑定device和submission。

### M8：surface、present、frame pacing与UI收敛

统一scene/UI surface owner、typed outcome、configuration generation、HDR/color-space/present policy和App cadence；surface成为graph external output或同batch blit。Editor/UI必须共享canonical device，独立fallback只用于显式tool profile。

### M9：compiled diagnostics与GPU工具闭环

将timer/statistics/lint/marker绑定dense pass slot和submission ticket，一帧只poll/collect一次。RenderDoc/PIX/平台marker与graph dump一一对应；diagnostics off不分配String/row，on时不全局禁用并行。GPU crash breadcrumb和磁盘PSO cache留P2，但接口预留generation和artifact identity。

### M10：产品、故障、规模和比较验收

运行第9节矩阵，保存机器、driver、adapter、commit、profile、场景、分辨率和raw artifacts。只在相同内容/质量/分辨率/平台/driver和统计方法下与Unreal比较CPU、GPU、内存、提交和frame pacing；某一场景胜出不等于全引擎“优于Unreal”。

## 9. 验收门

### 9.1 静态架构门

- `wgpu::Device/Queue/Surface`只允许出现在 `zr_rhi_wgpu` backend实现和经批准的platform glue；renderer、UI、resource、readback和graph execution命中为0。
- production存在且使用真实 `WgpuRhiDevice`；deterministic device只在tests，RHI合同可被统一owner持有。
- production `queue.submit`、`device.poll`、`wait_indefinitely`绕过命中为0；唯一coordinator/backend allowlist有结构守卫。
- graph handle含execution identity和version；产品authoring不再建立全局相邻pass链或按executor string修顺序。
- compile、user callback、device wait、surface acquire/present和GPU allocation不持framework global state lock。

### 9.2 正确性/故障门

- foreign builder、stale graph generation、stale device generation、slot reuse和double retire全部typed失败，无panic/UB/WGPU晚报。
- property tests随机生成至少10k个DAG，覆盖overwrite、branch、merge、side effect、readback、load/store/discard和subresource；compiled result与慢速reference evaluator一致。
- synthetic device loss发生在record、submit、map、present、pipeline compile和shutdown各阶段时，所有ticketexactly once终态，旧generation资源不能进入新device。
- readback over-entry/byte/age、cancel、consumer drop、callback panic、map error和shutdown均有界；0/overflow texture extent在admission失败。
- surface Lost/Outdated/Timeout/Occluded/resize/HDR切换有明确状态迁移，不把no-submit报告成成功present。

### 9.3 复杂度/内存门

- 128/512/2,048 pass与1k/10k resource synthetic graphs记录compile CPU、allocation、peak RSS和edge数；稳定instance更新近O(changed imports/dynamic constants)，不随全graph重编。
- stable 10k frames：schema compile=0、name-map rebuild=0、lint row materialization=0、label String allocation=0，GPU memory在预算附近有界且pending-retire最终归零。
- dynamic resolution/resize 10k次不抖动16-entry topology cache；同topology不同extent共享schema，physical resources按completion安全复用。
- readback burst按配置保持固定in-flight bytes和slot数；慢consumer只提高age/drop/reject telemetry，不无限增长RSS。

### 9.4 产品/性能门

- Windows Editor、runtime、headless/offscreen至少各跑100k frame soak；随后补Linux/Windows双平台和不同GPU vendor。
- RenderDoc/PIX验证普通单viewport无readback帧的logical packet与physical submit；backend允许时目标1 submit，任何额外submit都有typed reason。
- WPR/xperf/ETW记录main/render/RHI queue age、CPU wait/hold、submission thread利用率；main/editor线程不等待GPU readback、pipeline compile或foreign callback。
- GPU timestamp、pipeline stats和capture分别off/on组合，画面一致且调度/CPU开销可量化；观测开关不应把ready width从>1无条件降为1。
- 比较Unreal时锁定scene、shader/material质量、分辨率、AA、阴影、GI、驱动、warmup和capture窗口，报告median/p95/p99、GPU pass、CPU render/RHI、VRAM/RSS和stutter，不只报平均FPS。

## 10. 与既有计划的关系

- 本篇是current-source差距登记与重构验收合同，不是新的implementation owner。RHI/RDG/GPUScene硬切由 `performance/02` M3统筹；versioned graph、compile packet和transient由Render01承接；diagnostics/性能证据由Render17承接；render/RHI affinity和私有线程收编由Runtime11与M3边界共同承接。
- `2026-08-15-rhi-wgpu-submission-gpu-lifetime-current-architecture-review.md` 和 `2026-08-15-render-graph-current-architecture-review.md` 提供76文件/16文件的source freeze及更细静态证据。本篇把两份证据上升为统一的产品RHI/Frame Graph/lifetime硬切顺序，并补入当前test-only RHI、非对象安全contract、Unity version handle、Godot barrier/queue和Bevy error/submission对照。
- Render01现有RG-M1..M4曾完成部分静态结构，但当前source证明“handle/lifetime、transient pool、culling/cache、diagnostics存在”不等于RDG完成。实现时应重开version identity、correct culling、single physical allocation owner、compile-lock和dense diagnostics项，不以历史milestone名称掩盖差距。
- 不新增根crate。`zr_rhi`和`zr_rhi_wgpu`继续作为支持crate/后端叶子，生命周期与产品submission owner仍归 `zircon_runtime`；UI、Editor和App只能持lease/facade，不能成为第四个device owner。

## 11. 完成定义

### 11.1 2026-08-27 中立 render-asset batch upload 增量

09D M3 定向复审发现，把 semantic asset executor 接到旧 graphics `RenderBackend` 会绕过本计划“renderer/resource 不得拥有 WGPU 对象”的边界，并形成第二套 GPU lifetime owner。该草案已在 source 阶段撤销；当前增量把一票据 batch upload 提升到 `zr_rhi::RenderDevice`，由生产 `WgpuRhiDevice` 在 registry 内解析 generation handle、验证 usage/range、记录全部资源的 submission last-use，再提交给既有 submission service。deterministic backend实现同一 batch、budget 与单 ticket 合同；旧单次 buffer/texture write 退化为单元素 batch adapter，没有新增第二套 queue。

09D 新增 semantic residency artifact 现在只保存 `TextureHandle` / `TextureViewHandle` / `BufferHandle`，该新增路径不导入 `wgpu::`、`zr_rhi_wgpu` 或旧 `RenderBackend`。`RenderAssetResidencyManager` 决定 active/pending/retiring artifact，实际 destroy 仍进入 RHI registry，并按已记录的 last-use ticket 延迟 native release。该增量完成的是 M1/M5/M6/M7 所需的一段公共基础设施，不代表 frame graph version、统一产品 packet、全域 memory profile、completion driver、device-loss exactly-once 或产品 hard cut 已完成。

当前证据只有 scoped rustfmt、源码边界/行预算检查和新增 contract/确定性测试源码；managed Cargo、真实 WGPU、RenderDoc、GPU/RSS/VRAM/功耗及 soak 均未完成。因此 09A 只能标记 partial source-only，不能标记完成，也不能据此给出性能改善或 Unreal 等价结论。

### 11.2 2026-08-27 bounded asset completion consumer 增量

09D 产品接线复审证明当前 `SceneRenderer` 仍是旧 raw WGPU owner，不能在其内部附加 neutral asset manager并同时保留 `ResourceStreamer`。为后续单 owner硬切，本轮给 `RenderDevice` 增加 caller-scratch `append_submission_statuses`：默认实现保留 object-safe兼容，production WGPU 和 deterministic backend均覆盖为一次 submission-state lock内完成一批 ticket查询。该 API 不执行 device poll，也不建立 destructive global completion queue，因此 readback、surface、diagnostics 与 asset consumer不会争抢同一 completion事件。

asset manager使用按完整 device/generation/ticket identity排序的轮转前沿，每 tick只查询显式预算 `K`，并把 tracked总量限制在 RHI terminal-history容量内。稳定帧复用 ticket/status scratch，不创建 per-frame observer对象；复杂度为 `O(K log N + K)`，production submission mutex获取次数为 1。查询失败按 Failed处理而不是推断 Completed，保证 history overflow/device mismatch只能造成新 artifact被丢弃和报告 failure，不能破坏 last-good正确性。

ready artifact retirement 现在也有独立 artifact-count hard limit，默认与 terminal-history容量同源并可由未来产品 profile显式覆盖。新 upload绑定、active reference release batch和terminal publish都在状态突变前检查容量；detached terminal upload容量不足时继续留在 submission-owned retiring map并保持 frontier tracking，不会丢失 GPU handle。维护顺序固定为 receipt验证、按帧预算处理旧 retirement、批量观察 ticket、入队本帧新 retirement，因此旧 backlog先释放槽位，失败 artifact按帧初始快照最多重试一次。队列声明字节进入 report telemetry，但 physical buffer/texture byte admission继续只由 `GpuMemoryBudget` 负责，避免 residency 与 RHI出现两套漂移预算。

`RenderDevice::poll_submissions` 现在返回 `(device id, device generation, poll sequence)` 组成的 `SubmissionPollReceipt`。deterministic 与 production backend 都只在一次完成泵成功后单调发放 receipt；production device fault 路径先取得同一序列的 receipt，再把 unresolved submission、diagnostic 和 surface frame 统一 terminalize。asset manager 在任何 status scratch、批量查询或 retirement 前要求 receipt 与当前 device/generation 匹配且严格前进，foreign/replayed receipt 只产生 typed failure，不修改 manager 状态，也不访问 RHI status 或 destroy。该证据使未来唯一 frame owner 的 `poll -> fan-out completion consumers` 顺序可执行，但公开构造的诊断 DTO 不是安全令牌，也不能证明旧产品路径已只 poll 一次。

同轮结构审计按 `engine-code-structure-convention.md` 的 800 行 review门继续下沉三个完整职责：deterministic device构造进入90行 `device/construction.rs`，根文件由845降至767行；production native-to-neutral capability receipt进入134行 `production/device/capabilities.rs`，根文件为791行；neutral device 的375行公共错误合同硬切到 `device/error.rs`，根文件由887降至515行。production capability/device-ownership测试也迁到98/102行folder-backed owner，`production/tests.rs`为781行。结构测试锁定这些具名owner、禁止错误/capability映射回流并保持相关文件低于800行；没有提高阈值或保留旧函数转发层。

该增量仍没有把产品 scene submission迁到 `WgpuRenderDevice`，也没有替代未来更完整的多 consumer completion journal；它只提供当前单 queue MVP所需且不会阻碍后续 hard cut的 bounded consumer。retirement limit尚未接产品 memory/profile配置，shutdown/device recreate的产品 owner切换、真实 contention/profile数据与 product frame tick仍 pending，因此本节不改变 09A 的 partial source-only性质。

### 11.3 2026-08-27 render-asset device generation recovery 增量

针对 11.2 留下的 receipt stream reset 缺口，本轮先重审 09A device-loss 顺序、09D manager 状态、generation handle、production fault terminalization，并对照 UE `FRenderResource::ReleaseRHIForAllResources` / `InitRHI` 的全局释放后重建责任。结论是不允许新 device 通过放宽 receipt 校验继承旧状态，也不允许每个 upload/draw 调用点自行吞掉 stale handle；必须由 residency owner 在 product device owner 已终止旧 submission 后执行一次显式代际替换事务。

`RenderAssetResidencyManager::recover_device_epoch` 先验证 replacement 不同于 failed、同一 logical device 的 generation 严格递增、全部 live pending/active 与 GPU bound stream 属于 failed，再按稳定 ResourceId 解析当前 catalog/readiness seed并一次性预留 ticket id。任一错误发生在突变前，ticket id、entry、frontier、retirement和receipt均不变。提交后保留 reference count，旧 pending/active各产生原有 typed release，旧 generation 的 active artifact、pending/detached upload、ready retirement、submission frontier和last poll receipt统一失效，每个 live resource得到 replacement `QueuedIo` ticket。GPU state保存 O(1) bound epoch；未显式恢复时 replacement submission/receipt fail-closed，恢复后新 receipt stream从空 cursor重新建立严格单调关系。

恢复为允许 `O(N log N)` / `O(N)` 的冷路径，稳定 tick不新增entry扫描或临时分配。`manager.rs`把 recovery 与批量 ticket issuance分别下沉到246/63行具名 child owner，根文件为747行；Runtime15结构门禁止职责回流。该事务只丢弃上层旧 generation handle引用并量化 abandoned counts/bytes，真实 native object回收仍要求产品 owner drop failed `WgpuRenderDevice` registry；当前没有 product device swap、synthetic/真实 device-loss、managed Cargo、GPU capture或截图证据，不能声称 device-loss闭环完成。

### 11.4 2026-08-30 bounded scene submission terminal journal 增量

针对 Runtime27 AO command-record receipt 不能证明 GPU 已完成的问题，本轮先复审 `SceneRenderer` 两条 frame owner、surface 预轮询路径、`RenderFrameSubmissionTransaction`、RHI terminal history与批量查询合同，再核对 UE `FD3D12FinalizedCommands`、D3D12 submission queue/fence value owner和`FRenderCommandFence` bundling。结论是不允许 AO 或其它 feature 新建第二个 device poller，也不能把当前帧 graph record原地改写为 Completed；当前记录与滞后一帧或多帧的终态必须是两个可按 frame generation/ticket关联的合同。

新增 `SceneSubmissionCompletionJournal` 由 `SceneRenderer` 持有，成功 frame receipt 后只登记scene ticket；正常渲染的 frame-begin 入口通过 `poll_frame_submission_completions` 在同一 `SubmissionPollReceipt` 上先推进该日志，再分发 IBL、typed query和timer结果。surface路径先由该 owner完成一次分发，传给recording路径的 receipt只用于 frame ledger，不会二次消费。显式阻塞capture/readback是有30秒上限的同步边界，允许为已提交诊断工作额外推进timeline，但每次backend poll都必须同步回调同一 `SceneRenderer` fan-out，不能丢弃receipt或建立第二套consumer状态。日志容量直接取 `RenderDeviceProfile::submission_limits().max_unresolved_submissions()`，复用ticket/status scratch；有pending时通过`append_submission_statuses`一次submission-state lock观察整批，无pending时零状态查询。单次推进复杂度为`O(P)`、额外锁次数最多1，内存为`O(max_unresolved_submissions)`且构造后稳定帧不创建observer对象。

公共 `RenderSceneSubmissionCompletionReport` 把`Completed/Failed/Cancelled/DeviceLost`与`ObservationFailed/TrackingFailed`分开，保留frame generation、完整submission ticket及poll receipt，并量化pending/capacity与最近poll observed/terminal数。foreign device/generation、replayed poll和批量结果宽度不一致产生typed error并在任何status查询/queue突变前停止；status history miss按`ObservationFailed`丢弃该observer项，绝不推断Completed。`RenderStats`和11个固定`render.submission.completion.*`诊断路径暴露最近终态与backlog，因此 Runtime27 当前 AO command report可通过generation与GPU终态关联，但AO本身仍不拥有fence。

审查修复进一步覆盖三个曾绕过fan-out的路径：非阻塞readback poll改走frame owner，阻塞RGBA8/RGBA16F capture与diagnostic drain在每次poll后立即路由receipt；submission失败返回前也发布最新completion stats，框架边界保留typed completion error，不再降级为字符串。新增职责一度使pipeline owner达到860行，现已把完整capture/readback职责硬切到148行folder-backed owner，主文件回落到724行。该切片当前有7个journal行为测试及frame/readback source-order、错误发布和11个固定诊断路径测试源码；相关文件精确rustfmt、scoped whitespace/diff与locked metadata通过。受管Cargo仍受既知target复用门阻断，未取得Rust compile、真实WGPU terminal、device-loss、PNG/RDC、GPU profile、锁竞争或功耗数据；旧产品仍直接持有raw WGPU backend，因此09A继续是partial source-only，不计accepted milestone或性能改善。

### 11.5 2026-08-30 demand-compiled scene history physical allocation 增量

针对 Render07 `PERF-MVP-395`，本轮先复审 `SceneFrameHistoryTextures` 的构造、绑定、帧尾复制和 resize 路径，再对照 UE 5.5.4 per-view state 中按语义持有 GTAO/history 的方式。旧实现只要任一历史功能启用，就同时创建 TAA 双缓冲、GI lighting/metadata、SSR、HZB、曝光双缓冲和可选 froxel history；这使 feature-off 的物理资源不为零，也把 viewport-independent 的曝光/froxel 历史错误耦合到 viewport resize。

新增 `SceneFrameHistoryRequirements`，由 compiled pipeline 的实际 writer、temporal资格与 froxel quality 生成；构造只创建 TAA、hybrid GI、SSR、HZB、exposure、volumetric 中被要求的具名 owner。空需求立即释放当前 handle 的物理历史；仅 exposure、仅 volumetric 或仅 HZB 不再录制无附件的 history-clear submission。TAA/GI/SSR只匹配 history extent，HZB只匹配 render extent，固定 froxel quality 与 32-byte exposure ping-pong 不再因 viewport resize 重建。graph binder、history epilogue copy 和 HZB identity consumer均改为显式 optional lease，缺少物理 owner 时 fail closed；无合格 temporal consumer 的 AO history 仍保持删除状态。

静态容量模型按当前格式计算：TAA 双 `Rgba16Float` 为16 bytes/history-pixel，1080p约31.64 MiB、4K约126.56 MiB；GI lighting+metadata同为16 bytes/history-pixel；SSR `Rgba16Float`为8 bytes/history-pixel，1080p约15.82 MiB、4K约63.28 MiB；volumetric `160x90x{48,64,96}` `Rgba16Float`分别约5.27/7.03/10.55 MiB。以上只是关闭对应功能时可避免的容量模型，不是实测VRAM、带宽、帧时或功耗改善。

该切片仍在 requirements 集变化时替换整个 aggregate，并因此重置全部 domain state、扩大 TAA bind-group cache invalidation；尚未实现每个 domain 独立 reconcile/resize/retire。精确 rustfmt、调用点 optional 扫描、owner低于800行、scoped diff与locked metadata通过；受管 Cargo 仍受既知 `cargo_reuse_target_mismatch` 阻断，真实 WGPU、RenderDoc、PNG、GPU timestamp、VRAM、功耗与跨引擎比较均未验证。状态为 `runtime09a_scene_history_demand_compiled_allocation_source_implemented_static_checks_passed_dynamic_validation_pending`，不计 accepted milestone，也不改变09A的partial source-only性质。

### 11.6 2026-08-30 per-domain scene history reconcile 增量

11.5留下的整aggregate替换已继续收敛为固定6域的 `SceneHistoryAllocationChanges`。变化判定明确区分history extent依赖的TAA/GI/SSR、render extent依赖的HZB、quality依赖的volumetric和extent无关的exposure；稳定需求只更新最新extent元数据，不创建WGPU对象或初始化submission。occupied handle现在先在局部构造本次变化域，只有新TAA/GI/SSR clear submission成功后才逐域发布replacement；关闭域直接释放对应owner，未变化域的physical identity和`SceneHistoryDomainStates`保持不变。frame transaction只给变化域写`AllocationChanged`，camera-cut等spatial reason仍作用于其它相关域，feature-disabled reason最终覆盖已关闭域。

TAA bind-group cache invalidation从旧`history_recreated`整包信号收窄到`taa_history_allocation_changed`：SSR/HZB/exposure/volumetric单域启停或resize不再主动清空TAA cache。变化判定为固定`O(6)`、零动态集合；新增源码合同覆盖单域toggle、history/render extent分离、稳定空变化、replacement发布顺序，并加入可用WGPU环境下“启用SSR保持TAA/HZB identity”和“关闭SSR保持exposure且无clear submission”的行为测试源码。

该切片当时仍沿用旧 `RenderBackend::submit_graphics_command_buffers` 执行history clear，尚未迁入统一immutable frame packet/completion-retirement owner；该遗留已由11.7的scene-packet fusion源码切片取代。精确rustfmt、source-contract、owner低于800行、scoped diff与locked metadata通过；受管Cargo、真实WGPU执行、RDC/PNG、VRAM、GPU timestamp和功耗仍pending。状态为 `runtime09a_scene_history_per_domain_reconcile_source_implemented_static_checks_passed_dynamic_validation_pending`，不计accepted milestone或性能改善。

### 11.7 2026-08-30 scene-ticket history initialization fusion 增量

本轮先复审 `SceneFrameHistoryTextures`、`RenderFrameSubmissionTransaction`、`WgpuSubmissionService::flush` 与完整compiled-scene frame owner。结论是把history clear简单改为pre-scene enqueue仍不成立：后续frame resource upload会形成新的upload/command边界，而且任何scene submit前失败都可能留下“物理owner已发布但clear从未执行”的history。UE 5.5.4标准renderer以RDG clear pass进入同一graph/command stream；`LumenInUE5.5.4WithComputeShader`的逐pass `SubmitCommandList()`只作为算法复刻参考，不作为Zircon frame submission ownership样板。

history构造/reconcile现在只返回可选`wgpu::CommandBuffer`，不直接submit或enqueue。compiled scene在frame resource upload进入既有pre-scene ledger后，才把该buffer放到terminal scene packet的command-buffer索引0；scene draw、诊断尾与surface copy继续共享原scene ticket。稳定帧没有初始化buffer，也不执行`Vec::insert`冷路径。旧`RenderFrameSubmissionProducer::HistoryInitialization`已硬切，receipt只保留真实独立ticket生产者。若scene ticket接受前任一步骤失败，frame owner删除本帧当前history handle，下一帧重新创建/clear；若错误是`FrameFailedAfterSceneSubmission`，clear和scene已被同一ticket接受，history保留。

同步处理F16结构债：`render.rs`按foundation、mesh submission preparation、graph-frame preparation、success commit拆为具名owner，主编排器481行；resource binding把SSAO child拆出后153行，scene submit把HZB readback与folder-backed tests拆出后627行，均通过既有严格`<500/<160/<650`守卫，没有放宽预算。精确rustfmt、history scene-packet source contract、旧producer负向扫描、行数预算、scoped diff、尾随空白与locked metadata通过。受管Cargo仍受既知`cargo_reuse_target_mismatch`阻断；真实WGPU native submit批次、PNG/RDC、GPU timestamp、VRAM和功耗均未验证。状态为 `runtime09a_history_initialization_scene_packet_fusion_source_implemented_static_checks_passed_dynamic_validation_pending`，不计accepted milestone或性能改善。

### 11.8 2026-08-30 frame submission interval metrics 增量

P0-5调用图复审确认当前底层`WgpuSubmissionMetricsSnapshot`已有generation-local单调计数，但正常frame receipt只保存逻辑ticket，性能工具无法把同一帧的logical packet、flush ticket、physical backend submit和upload workload关联起来。本轮新增backend-neutral `RenderFrameSubmissionMetrics`：分别发布frame owner接纳的logical packet数、flush实际提交的ticket数、native submit数，以及buffer/texture upload batch、write和payload bytes。`RenderFrameSubmissionReceipt`只持可选值；device/generation owner改变或计数回退时，WGPU `delta_since`返回`None`，不能拼接两个timeline或伪造零值。

compiled与legacy terminal frame owner都在frame completion poll之后、任何scene resource preparation之前采集baseline，并在scene ticket已提交且receipt完成后采集终点；采样不flush、不poll、不wait，也不新增queue work。viewport product与present共享scene ticket时不会增加logical packet数。该统计能让后续真实运行直接回答普通帧的physical submit数，但本轮没有执行WGPU workload，因此不把源码分组规则写成一次submit实测。结构上，新增DTO为93行；原515行且继续增长的receipt owner已把203行测试迁入folder-backed child，该切片落地时production root为370行，11.9继续抽出producer record后current source为334行。

精确rustfmt、双frame owner采样顺序合同、backend无flush/poll合同、physical/logical分离测试源码、物理行预算、尾随空白、scoped diff与`cargo metadata --locked --no-deps`通过。受管Cargo仍受既知`cargo_reuse_target_mismatch`阻断；真实WGPU、RenderDoc、PNG/RDC、GPU/RSS/VRAM/功耗和跨引擎比较均未验证。状态为 `runtime09a_frame_submission_interval_metrics_source_implemented_static_checks_passed_dynamic_validation_pending`，不计accepted milestone或性能改善。

### 11.9 2026-08-30 texture mip preservation typed boundary 增量

在11.8可量化physical submit之后，本轮继续复审`GpuTextureUploadWork`的pre/copy/post顺序。当前`pre_upload_commands`只由resident-mip replacement产生：必须先把旧texture仍驻留的mip复制到replacement，再由queue write上传新mip；WGPU flush明确把`command -> upload`视为真实排序边界。该依赖不能通过把direct submit机械改成enqueue消除，且不应只靠`TexturePreUpload`名称推断原因。

新增backend-neutral `RenderFrameSubmissionBoundaryReason::TextureMipPreservationBeforeUpload`。只有非空pre-upload command ticket通过`record_pre_scene_resource_submission_with_boundary`写入success receipt；copy upload和post-upload ticket保持无reason。producer与reason的合法配对由独立record owner集中校验，错误地把preservation reason标到copy/post producer会在transaction状态变化前返回typed `BoundaryReasonProducerMismatch`，receipt重建时再次fail closed。failure settlement把同一typed reason连同resource id、ticket和terminal status保留，避免失败profile丢失边界解释。reason类型不依赖WGPU；具体纹理variant只有resource streamer生产点赋值一次。该切片不改变submit/enqueue顺序，也不实施跨纹理批处理。

精确rustfmt、唯一production赋值点、`pre command -> boundary receipt -> upload`顺序、错误配对拒绝、success/failure传播与`cargo metadata --locked --no-deps`通过。boundary reason/producer record/receipt root/receipt tests/transaction root/transaction tests/failure receipt/backend/resource streamer为12/90/334/203/174/228/312/380/248行；Runtime09A结构守卫root/child为351/122行，均低于各自预算。受管Cargo仍受既知`cargo_reuse_target_mismatch`阻断；真实WGPU/RenderDoc/profile未运行。只有后续数据证明同帧多个mip replacement导致显著submit成本，才评估“集中收集pre copies -> 一次边界 -> 合并uploads”的frame-level批处理；当前不宣称瓶颈或性能改善。状态为`runtime09a_texture_mip_preservation_typed_boundary_source_implemented_static_checks_passed_dynamic_validation_pending`。

### 11.10 2026-08-30 hit-proxy submission transaction ordering 增量

复审独立hit-proxy task发现其在bounded diagnostic admission和frame prepare之前就把GPU-scene upload接纳到共享submission queue，并立即commit CPU侧GPU-scene状态。若diagnostic预算拒绝或prepare失败，调用会返回，但后续无关submission仍可能flush该孤儿upload；CPU状态还会把未形成terminal hit-proxy packet的准备结果误记为已提交。这是失败事务边界错误，不是需要profile后才能判断的性能微调。

当前顺序硬切为`record hit-proxy -> admit/prepare all diagnostic readbacks -> enqueue GPU-scene upload -> terminal command+diagnostic submit -> commit GPU-scene upload`。所有diagnostic fallible边界发生在upload admission前，`gpu_scene_upload.commit`只位于terminal submit成功路径；draw、readback callback、native submit数量和flush策略均未改变。production中upload enqueue与commit各只有一个调用点，源码回归锁定prepare/enqueue/submit/commit严格顺序。`scene_renderer_hit_proxy.rs`当前516行，低于800行owner预算。

精确rustfmt、事务顺序/唯一调用点静态合同与scoped diff检查通过；locked metadata在同一Runtime09A工作批次通过。受管Cargo仍受既知`cargo_reuse_target_mismatch`阻断，真实WGPU hit-proxy、device-loss、PNG/RDC、profile与功耗未验证。该切片没有给environment capture、retained UI或其它task建立统一receipt，因此P0-5仍开放；状态为`runtime09a_hit_proxy_submission_transaction_order_source_implemented_static_checks_passed_dynamic_validation_pending`。

### 11.11 2026-08-30 product readback/completion current-source reaudit

P0-6重新按当前module gating、调用点和completion routing分类：同步buffer/texture/IBL helper是test-only，产品diagnostic service已具备generation/ticket identity、admission与result-ring预算、submission-qualified map、terminal delivery和30秒显式capture deadline；正常scene与viewport-pick都通过同一completion fan-out。该复审没有把测试readback或RenderDoc stop误算为普通产品frame，也没有把静态调用面写成运行时性能数据。

仍开放的真实边界是standalone native UI的独立device/raw present submit、本地completion timeline，显式capture policy尚未统一表达consumer executor/cancel，以及真实device-loss/shutdown/timeout注入和scale/profile证据未完成。本节只纠正计划输入，不改生产代码、不计accepted milestone；状态为`runtime09a_product_readback_current_source_reaudit_documented_remaining_boundaries_open`。

### 11.12 2026-08-30 standalone UI local submission/completion hard-cut design

切入前复审确认standalone UI不能只把`self.queue.submit`包装为`WgpuRenderDevice`：其本地`GpuReadbackQueue::poll_completed`仍会主动`device.poll`，从而在同一native device上形成中央submission owner与旧readback completion owner并存。共享UI已经复用runtime `Arc<WgpuRenderDevice>`、返回真实ticket且不拥有completion timeline；迁移范围只属于自有device的tool/compatibility profile。

依照UE SlateRHI经`FRHICommandListImmediate`记录、RHI owner finalize/submit、viewport transaction present和deferred cleanup的顺序，新增[`90/2026-08-30-standalone-ui-local-submission-completion-hard-cut-design.md`](90/2026-08-30-standalone-ui-local-submission-completion-hard-cut-design.md)。SUI-0至SUI-3源码迁移已落地：offscreen/standalone共用initial profile factory；所有native UI context必有typed `Arc<WgpuRenderDevice>` owner；standalone raw submit删除并返回真实ticket；local frame在surface acquire前由唯一owner poll，旧readback只收集已轮询结果，产品直接`device.poll`退到test-only；image pin随native packet进入submission service，以ticket为键由现有唯一completion callback或fault terminalization释放，不再创建UI私有完成回调。基础owner/profile合同为11/11，SUI-3合同从0/8转为8/8；精确rustfmt、scoped diff、结构行数和locked metadata通过。SUI-4真实窗口/PNG/RDC/profile仍开放。状态为`runtime90_standalone_ui_sui_0_through_sui_3_source_implemented_static_checks_passed_dynamic_validation_pending`。

- 一个真实产品frame从graph schema/instance经过versioned compile、RHI packet、统一submit、surface present、completion和resource retire形成可追踪闭环。
- 所有GPU handle、ticket、surface和cache都受device/execution generation保护，device loss和shutdown对在途工作有exactly-one终态。
- RHI command/capability/limit/fallback一致，产品不直接依赖WGPU对象；deterministic backend只作为同contract的测试实现。
- Graph culling、barrier、lifetime和alias基于资源version/subresource，不依赖总序或人工writer chain；执行热路径使用dense slot，不按String重建关系。
- readback/upload/diagnostics/present/UI进入统一submission和completion owner，生产代码没有无限GPU等待或未经预算的队列。
- 静态、行为、fault、scale、soak、GPU capture和跨平台证据全部落盘；性能比较满足同质量可复现条件。只有到此才能把09A从 `implementation_status: pending`改为完成。
