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
implementation_status: pending
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

### P0-5：提交权分散，多次物理submit和同步readback绕过统一frame batch

compiled scene虽把graph buffers和IBL writeback合为一次submit，present仍另建bind group/encoder并第二次submit；output target writeback、resource upload、scene clear、UI external copy、retained cache、readback等路径有额外直接submit。normal frame不存在一个可以枚举“本帧全部packet、wait、present、readback、retire”的owner。静态最小submit数已经是2，动态功能会增加，但统计没有统一submission id和cause。

目标只有 `RhiSubmissionCoordinator` 可调用backend submit。所有upload、graph、surface blit、UI composition、readback copy和writeback先形成packet并进入同一frame batch；无法合并必须给typed reason和独立ticket。surface target优先直接成为graph external output，避免无必要offscreen-to-surface blit；确需blit时也并入同batch。每帧报告logical packet、physical backend submit、queue、wait、bytes和reason，验收普通单viewport无readback路径在backend允许时为一次physical submit。

### P0-6：产品readback可无限阻塞，完成轮询没有唯一owner或终态合同

多个buffer/texture/IBL helper逐调用创建staging/encoder、submit、`wait_indefinitely`、阻塞receiver并复制。texture helper还以u32直接乘宽度和bytes-per-pixel，未先拒绝0 extent或checked overflow。三槽queue虽不无限等待，但request数量和总字节无上限，ticket只有私有u64且wrap，queue持有clone device却允许外部传任意device poll，begin_map不绑定submission ticket，abort不能证明copy是否已submit，callback在poll collect中同步执行，poll error被忽略。

目标所有readback都经 `ReadbackAdmission` 返回generation-scoped ticket；copy由所属frame batch编码，ticket记录submission、range、byte budget、deadline、consumer和terminal state。一个completion owner每帧poll一次或消费backend callback，把ready result移入bounded completion queue；用户callback在锁外/指定executor运行。限制每帧entries、bytes、单request bytes、in-flight bytes和age；cancel/timeout/device loss/shutdown均exactly once。生产代码禁止 `wait_indefinitely`，仅显式commandlet/test在独立policy下允许有deadline的blocking wrapper。

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

- 一个真实产品frame从graph schema/instance经过versioned compile、RHI packet、统一submit、surface present、completion和resource retire形成可追踪闭环。
- 所有GPU handle、ticket、surface和cache都受device/execution generation保护，device loss和shutdown对在途工作有exactly-one终态。
- RHI command/capability/limit/fallback一致，产品不直接依赖WGPU对象；deterministic backend只作为同contract的测试实现。
- Graph culling、barrier、lifetime和alias基于资源version/subresource，不依赖总序或人工writer chain；执行热路径使用dense slot，不按String重建关系。
- readback/upload/diagnostics/present/UI进入统一submission和completion owner，生产代码没有无限GPU等待或未经预算的队列。
- 静态、行为、fault、scale、soak、GPU capture和跨平台证据全部落盘；性能比较满足同质量可复现条件。只有到此才能把09A从 `implementation_status: pending`改为完成。
