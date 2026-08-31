---
title: Runtime RHI / WGPU Adapter、Device、Capability、Resource、Command、Queue、Submission、Completion、Readback、Surface、Device Loss 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime90
review_date: 2026-08-21
baseline_head: be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1
baseline_epoch: 336
related_code:
  - zircon_runtime/crates/zr_rhi
  - zircon_runtime/crates/zr_rhi_wgpu
  - zircon_runtime/src/rhi.rs
  - zircon_runtime/src/graphics/backend
  - zircon_runtime/src/graphics/runtime/render_framework
  - zircon_runtime/src/graphics/scene/resources
  - zircon_runtime/src/graphics/scene/scene_renderer
tests:
  - zircon_runtime/crates/zr_rhi/src/tests
  - zircon_runtime/crates/zr_rhi/src/ui_surface/tests.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_readback_queue/tests.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/tests.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/tests/native_submission.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/79-runtime-ui-renderer-display-list-paint-order-clip-transform-opacity-atlas-text-glyph-batch-wgpu-submit-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/DynamicRHI.h
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/RHIResources.h
  - dev/UnrealEngine/Engine/Source/Runtime/D3D12RHI/Private/D3D12Submission.h
  - dev/UnrealEngine/Engine/Source/Runtime/D3D12RHI/Private/D3D12Submission.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/D3D12RHI/Private/D3D12Allocation.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/D3D12RHI/Private/D3D12Adapter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/D3D12RHI/Private/D3D12Util.cpp
  - dev/godot/servers/rendering/rendering_device_driver.h
  - dev/godot/drivers/vulkan/rendering_device_driver_vulkan.h
  - dev/godot/drivers/vulkan/rendering_device_driver_vulkan.cpp
  - dev/bevy/crates/bevy_render/src/gpu_readback.rs
  - dev/bevy/crates/bevy_render/src/error_handler.rs
  - dev/bevy/crates/bevy_render/src/renderer/render_device.rs
  - dev/Fyrox/fyrox-graphics/src/read_buffer.rs
  - dev/Fyrox/fyrox-graphics/src/server.rs
  - dev/Fyrox/fyrox-graphics-gl/src/read_buffer.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourcePool.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceRegistry.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/CommandBuffers/IUnsafeCommandBuffer.cs
doc_type: current_source_review
review_status: complete
implementation_status: m4_source_implemented_m5_foundation_in_progress_pfo1b_terminal_graph_and_direct_fallback_source_validation_blocked
implementation_progress_updated: 2026-08-30
source_recheck_required: true
---

# Runtime RHI / WGPU Adapter、Device、Capability、Resource、Command、Queue、Submission、Completion、Readback、Surface、Device Loss 与 Product Integration 当前源码工程化差距

## 1. 结论

当前`zr_rhi`不是完全空壳。它已有中立descriptor、typed enum handle、buffer/texture/sampler/bind group/shader/pipeline vocabulary、render-pass view与attachment validation、Graphics/Compute/Copy分类、debug marker以及一批边界测试。`zr_rhi_wgpu`也有真实WGPU capability映射、三槽异步readback staging ring、256-byte alignment、按2次幂扩缩容、GPU timestamp/pipeline statistics、共享device的UI composition、retained cache和typed retryable surface outcome。这些底座应保留。

但产品RHI仍是倒置的。`zr_rhi::RenderDevice`没有任何production实现；`zr_rhi_wgpu::device`在crate root被`#[cfg(test)]`隔离，唯一实现是CPU内存中的`DeterministicRhiContractDevice`。真实产品绕过中立RHI，直接由`graphics::backend::RenderBackend`及大量scene/UI模块持有`wgpu::Device`、`wgpu::Queue`与resource。测试甚至明确要求这种倒置继续存在。与此同时，capability mapper声明Graphics/Compute/Copy、async copy、indirect/multi-draw和graphics debugger capture，中立command/device接口却不能提交对应命令、表达真实queue topology、surface、device generation或GPU completion。

更关键的是，当前系统只有“已调用`queue.submit`”和“GPU已经完成”之间没有引擎级身份。`FenceValue`只是裸`u64`；resource handle没有device/generation/kind/slot所有权；产品没有submission index、targeted completion、deferred retirement或device-loss supervisor。readback queue虽有三槽和回调，却允许传入任意Device进行prepare/poll，ticket不带device/frame/submission/status，pending request与总bytes无上限，cancel和Drop不保证一次终态；多个产品路径仍直接submit后`wait_indefinitely()`整机等待。Surface/UI又有可创建第二套device的owned路径，并在submit后立即把cache记为ready，没有统一submission receipt与恢复代际。

本报告登记 **3项P0、48项P1、12项P2与48个资格门**。Runtime89继续拥有Render Graph builder/compiler/version/culling/alias和compiled packet消费；Runtime79继续拥有UI layout/batching/clipping/text算法；本篇接替Runtime09A中已经陈旧的RHI currentness，唯一拥有adapter/device/capability/resource identity、command/queue、submission/completion、readback、surface/device-loss以及产品WGPU hard cut。这里的目标不是再包装一层`wgpu`，而是建立能支撑多backend、故障恢复、资源寿命和可验证性能的production RHI owner。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test markers | 证据等级 | fingerprint |
|---|---:|---|---|
| `zr_rhi` production | **10 / 3,545 / 3,183 / 105,878 / 4** | E3逐文件读取capability、descriptor、device、handle、surface与UI contract | `b4008a1546a4bbc320fa9a37e3aef060c23d0315a2c69c3d9d967bdec968b53a` |
| `zr_rhi` focused tests | **5 / 1,255 / 1,167 / 43,529 / 31** | E3读取boundary、capability、descriptor与UI contract tests | `1df874b23df91a668ed32833ffeb4024a4c2e6af25eb35b57514f9ea5e785d5f` |
| `zr_rhi_wgpu` production-path files | **31 / 10,544 / 9,853 / 385,818 / 36** | E3读取capability、readback、timer/statistics与完整UI surface子树；含crate root隔离的test-only device文件 | `0f258e4dab3f5768b399f513b5968f5d0960486251d538f368044c0fb2b10ad8` |
| `zr_rhi_wgpu` focused tests | **30 / 8,700 / 8,067 / 292,637 / 180** | E3核对resource/command/pass/copy/readback/UI/native submission与framework boundary | `8fc67cf4a11aa57178e5a023b952890b60b084d9ac2d2ef55b71fb872f9bcc22` |
| 产品WGPU集成切片 | **34 / 12,500 / 471,829 bytes** | E3机械冻结直接WGPU调用文件，并逐段读取device request、blocking readback、compiled/direct submit、surface、texture streaming、IBL与history链 | `e114e585de8c43861cee54745d3c53f69dfe40fafcbc74aa5b44812bac12eb95` |
| 五引擎参考切片 | **19 / 28,985 / 24,675 / 1,228,675 / 29** | E2/E3读取Unreal submission/retirement/crash、Godot queue/swapchain/device fault、Bevy WGPU recovery/readback、Unity Graphics pool/readback与Fyrox最低接口 | `4a284efbef6a415a6e038a411e74b6d5d0778506392787515fb691f191ebd857` |

冻结集合代表2026-08-21共享working tree，不是只读HEAD、ABI freeze或验收receipt。Git基线为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator baseline epoch为336。Godot、Bevy、Fyrox与Unity Graphics参考revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal镜像没有独立`.git`，由参考aggregate fingerprint冻结。

`zircon_runtime/crates/zr_rhi_wgpu/src/gpu_pass_timer.rs`在本轮开始前已有共享会话改动，冻结、阅读和复核均包含该版本；本轮不拥有也未修改它。写报告前四个核心语料的文件数、行数和bytes未漂移。

### 2.2 证据限制

- 本轮只做current-source review，没有修改Rust、Cargo、asset或tooling，也没有运行Cargo、真实GPU、RenderDoc、device-loss injection、multi-queue、soak和benchmark。
- 生产集成切片是直接WGPU模式的机械集合；34个文件中有6个仅在同文件`#[cfg(test)]`路径生效。结论使用逐段确认过的production call chain，不把纯测试submit算成产品缺陷。
- `dev/Graphics`只是Unity Graphics package，不含Unity native backend；本篇只使用其command-buffer async readback和resource-pool产品形态，不宣称覆盖Unity完整RHI。
- Fyrox的graphics server/read buffer低于目标工程等级，只作为“object-safe backend + 显式异步状态”的最低线，不作为目标架构主导者。
- 用户要求未来tooling迁移到Rust，本篇不审查或新增tooling方案；Editor中的RHI诊断consumer只登记handoff contract。

### 2.3 当前产品链

```text
graphics::backend::RenderBackend
  -> wgpu Instance -> Adapter -> request_device
  -> owns wgpu Device + Queue directly
  -> SceneRenderer / resources / surface receive raw WGPU references
  -> many queue.write_buffer / queue.write_texture implicit submissions
  -> compiled graph central queue.submit(command_buffers)
  -> additional direct submits for readback, viewport, streaming, history, IBL and UI
  -> GpuReadbackQueue begins map without engine submission identity
  -> selected capture paths device.poll(wait_indefinitely)

zr_rhi neutral contracts
  -> descriptors + raw-u64 handles + CommandList + RenderDevice
  -> no production implementation
  -> DeterministicRhiContractDevice under #[cfg(test)] only

zr_rhi_wgpu UI surface
  -> shared-device path OR owned second Instance/Adapter/Device/Queue
  -> direct submit -> mark cache ready -> begin map -> present
```

产品compiled scene已经把graph command buffers集中为一次submit，这是应保留的局部收敛；但它不是全局submission owner。机械扫描在排除显式测试目录后仍命中21个文件中的31处`queue.submit`文本，剥离同文件测试与非WGPU queue abstraction后，production仍至少有18个独立submit点。全`zircon_runtime`搜索没有`set_device_lost_callback`、`on_uncaptured_error`、`on_submitted_work_done`或`SubmissionIndex`。

## 3. Owner边界与旧报告修正

| owner | 本篇处理 |
|---|---|
| Runtime09A | 保留历史总览价值；其“通用RHI/GPU lifetime待补”方向仍成立，但当前源码事实、优先级和实施边界由Runtime90接替 |
| Runtime89 | 唯一拥有graph handle/version、pass culling、transient alias plan、barrier plan与device-qualified execution packet；Runtime90只提供backend capability、queue和completion执行能力 |
| Runtime79 | 唯一拥有UI primitive、batch、clip、text/atlas、damage算法与视觉资格；Runtime90只拥有UI surface的device/submission/resource lifetime边界 |
| Runtime77及shader专题 | 拥有shader语言、shader compiler和材质算法；Runtime90拥有backend shader artifact、reflection ABI、pipeline cache/device identity |
| Runtime lifecycle / app host | 拥有进程与runtime shutdown；Runtime90提供DeviceOwner的quiesce、drain、lost/recreate和terminal receipt |
| Editor diagnostics | 后续消费RHI snapshot、submission timeline、memory budget和device fault artifact；不在本篇实现Editor UI |

## 4. 当前实现中应保留的底座

### 4.1 中立descriptor和validation已有真实内容

`BufferDesc`、`TextureDesc`、sampler、bind group、shader/pipeline、render-pass attachment/view、viewport/scissor、copy region和draw/dispatch不是占位类型。WGPU test device的validation覆盖usage、范围、attachment format/sample/resolve、bind group layout与pipeline kind。重构应迁移这些约束到production backend和shared validator，不应删除后回到散落的`wgpu`错误。

### 4.2 Capability映射已开始记录negotiated事实

`RenderBackendCaps`已有adapter info、device limits、queue class和timestamp/subgroup/statistics/bindless相关字段；`wgpu_backend_caps`也区分`MULTI_DRAW_INDIRECT_COUNT`与fixed-count multi draw，并根据features映射timestamp、subgroup和pipeline statistics。这一层需要从flat booleans升级为有provenance的device profile，而不是废弃。

### 4.3 Readback staging不是同步临时实现

`GpuReadbackQueue`用3个frame slot、256-byte offset alignment、单slot聚合copy、power-of-two grow、240个低利用frame后shrink，并把callback panic隔离。timer和pipeline statistics复用同一readback queue。这比每次临时创建staging后整机等待更接近工程底座。

### 4.4 Shared-device UI composition方向正确

`WgpuUiSurfaceContext`显式要求Instance/Adapter/Device/Queue来自同一negotiated backend，viewport product registry也维持generation-stable外部图像。共享路径能避免CPU readback和跨device复制；`RetryableNoSubmit`也比把surface错误伪装为成功更可靠。必须硬切到该方向并移除owned second-device fallback。

### 4.5 产品已有局部bounded owner

viewport product generation最多保留3代，IBL pending writeback有4项上限，共享/本地UI image cache各有entry和byte budget。compiled scene统一收集command buffers后单次submit。这些局部纪律应并入全局ResourceRegistry、SubmissionService和ReadbackService，而不是另起一批无界容器。

## 5. 参考实现差异与适用边界

| 引擎 | 本地源码证据 | 对Zircon的最低要求 | 不应照搬 |
|---|---|---|---|
| Unreal RHI / D3D12RHI | `FD3D12SyncPoint`是一次性queue timeline point；submission thread解析跨queue wait并给每个queue递增completion fence；interrupt queue只在fence完成后释放payload持有的GPU资源；allocator deferred deletion和page reuse都比较completed fence；DRED、breadcrumbs、page fault与GPU timeout进入device-removed路径 | SubmissionTicket必须绑定device generation、queue timeline和completion；resource retirement、allocator reuse、query/readback完成必须消费同一completion证据；GPU hang/loss必须有fault artifact | 不复制Unreal线程、宏、COM层次或全部D3D12专有策略；借鉴所有权和不变量 |
| Godot RenderingDeviceDriver / Vulkan | 中立driver区分typed resource ID、Graphics/Compute/Transfer queue family、command pool/buffer、fence/semaphore、swapchain/HDR/color space、memory totals、limits和API traits；Vulkan queue有submit mutex，Fence记录signal来源queue，提交处理wait/signal/present并捕获`VK_ERROR_DEVICE_LOST`与device fault | Neutral RHI必须能表达真实queue/surface/memory/device facts，backend必须把这些facts兑现成命令和失败语义 | Godot Vulkan当前device loss多为终止/报告，不等于Zircon的recovery产品策略 |
| Bevy WGPU | `DeviceErrorHandler`注册device-lost与uncaptured-error callback，`RenderState`在Ready/Errored/Reinitializing间切换并允许Recover；GPU readback检查buffer范围、集中编码copy、异步map、池化并按unused frame清理 | 即使仅WGPU，也必须有device error owner、可观察状态机、重建入口和非阻塞readback pool | Bevy readback也缺少本篇要求的全局quota/exact terminal，不作为上限 |
| Unity Graphics | unsafe command buffer暴露buffer/texture/mip/region/format/native-array async readback；RenderGraph pool记录frame/execution provenance、异常回收、stale purge和memory stats | Readback request必须表达完整subresource与目标格式；pool必须有provenance、异常清理、预算和诊断 | package源码不能证明native fence/device-loss实现，不能替代Unreal/Godot主证据 |
| Fyrox | `GraphicsServer`是object-safe backend contract并暴露capability/memory/flush/finish；GL async read buffer用fence和nonblocking zero-timeout poll | 即使较小引擎也没有把production backend只留给test fixture；异步状态必须显式 | Fyrox capability、queue和lifetime模型远低于目标，不作为最终设计 |

主导参考是Unreal的submission/completion/retirement/fault闭环和Godot的中立RenderingDeviceDriver边界。Bevy用于WGPU error/recovery与readback交叉检查；Unity Graphics用于readback subresource与pool provenance；Fyrox只证明Zircon当前“neutral trait无production implementor”低于最低抽象线。

## 6. P0架构与正确性阻断

### `RHI90-P0-001`：中立RHI没有production backend，产品与contract test形成两个互不相交的设备世界

**确定性证据链：**

1. `zr_rhi_wgpu/src/lib.rs:15`把`device`模块置于`#[cfg(test)]`；production exports只有capabilities、timer/statistics、readback和UI surface。
2. `zr_rhi_wgpu/src/device.rs:204`的唯一`impl RenderDevice`属于`DeterministicRhiContractDevice`，资源只是HashMap状态，submit在同一临界区立刻把`completed_fence = fence`。
3. `zr_rhi::RenderDevice::create_command_list`包含`label: impl Into<String>`，trait不是object-safe；即使产品想以`Arc<dyn RenderDevice>`拥有backend也做不到。
4. `framework_boundary.rs:24-91`明确断言deterministic device必须test-only，产品必须直接使用`graphics/backend`的真实WGPU owner。测试把架构倒置固化为成功条件。
5. 产品的buffer、texture、pipeline、command encoder和queue因此无法由中立handle、validation、capability或lifetime owner约束；任何未来Vulkan/D3D12/Metal backend都只能再次复制整套产品路径。

**必须修复：** `zr_rhi_wgpu`提供真实production `WgpuRenderDevice`；`RenderDevice`改为object-safe或由稳定service handle间接调用；`graphics::backend`只拥有`RhiDeviceOwner`，scene/framework/UI不得直接创建第二设备。deterministic实现降级为tests fixture，测试改为证明production call graph经过neutral contract，不得继续证明绕过。

### `RHI90-P0-002`：capability可声明无法从中立接口执行的能力，device admission会产生false positive

**确定性证据链：**

1. `wgpu_backend_caps`无条件登记Graphics/Compute/Copy、async copy、indirect draw、fixed-count multi draw、debug marker/group和graphics debugger capture。
2. `RenderDevice`只接收单个boxed command list并返回裸`FenceValue`，没有queue object、wait/signal、surface、submission batch、indirect/multi-draw或capture operation。
3. `CommandListCommand`只有direct draw/indexed draw/dispatch和三类copy，连产品实际使用的texture-to-texture copy都不存在；Graphics/Compute/Copy只是validation label，不是可选择的物理queue事实。
4. WGPU backend当前只有一对真实Device/Queue；没有证据证明async copy或独立compute queue。`supports_graphics_debugger_capture`也无条件为true，而产品capture只是阻塞poll，没有capture provider negotiation。
5. 上层若按capability启用async copy、multi draw或capture，会通过admission却找不到可执行operation；这违反capability必须由同一backend实现与测试证明的基本合同。

**必须修复：** capability改为`DeviceProfile`中的typed feature/limit/format/queue facts，每项带`Native/Emulated/Unsupported`、来源与fallback；任何advertised operation必须在object-safe command/queue接口中可调用，并有backend conformance test。单队列WGPU不得冒充独立async queue。

### `RHI90-P0-003`：没有device generation与GPU completion owner，resource/readback/surface在失效或销毁时无法保证一次终态

**确定性证据链：**

1. 所有resource handle都是可公开`new(raw: u64)`和`raw()`的单字段类型，没有device id、generation、slot generation或resource kind owner；`FenceValue`也只是`pub u64`。
2. 产品没有`SubmissionIndex`、`on_submitted_work_done`、device-lost callback或uncaptured-error callback。compiled submit、direct scene和UI都只能观察API接受submit，不能观察目标GPU work完成。
3. compiled scene在`queue.submit`后调用writeback commit/readback begin-map，并把transient backing放回pool；direct scene调用`complete_submission(..., true)`；UI submit后立即`mark_*_ready`。这些名字把queue acceptance与GPU completion混为一谈。
4. `GpuReadbackQueue`保存创建时device clone，却允许`prepare_frame`和`poll_completed`传入任意`&wgpu::Device`；request也接收任意buffer/texture。ticket只有递增`u64`，不含device/frame/submission/status。
5. pending request和总staging bytes无admission上限；`cancel`静默删除callback，queue没有Drop terminalization。device loss、owner shutdown或poll永久失败时，调用方无法区分Cancelled/Lost/Abandoned，callback也不保证exactly once。

**必须修复：** 建立`DeviceId + DeviceGeneration + QueueTimeline + SubmissionTicket`；resource registry以generation handle验证同device所有权；所有destroy进入completion-qualified retirement；readback/surface/capture挂在ticket上并在Completed/Failed/Cancelled/DeviceLost/Shutdown中exactly-once终结；DeviceErrorSupervisor统一注册WGPU callbacks并驱动Stop/Drain/Recreate或fatal artifact。

## 7. P1工程化差距（48项）

### 7.1 Instance、Adapter、Device 与 Capability（P1-001至P1-008）

| ID | 当前差距 | 目标 |
|---|---|---|
| P1-001 | adapter选择只有单次high-performance request，没有enumeration、stable id、vendor/device/driver/backend denylist或用户override | `AdapterCatalog + AdapterSelectionPolicy + SelectionReceipt` |
| P1-002 | capability主要是flat bool，缺feature provenance、native/emulated、required/optional与fallback disposition | immutable `DeviceProfile`和可序列化negotiation receipt |
| P1-003 | queue classes是声明列表，不含真实family/count/timestamp/present支持和ownership | `QueueTopology`保存physical/virtual queue facts |
| P1-004 | 没有per-format sampled/storage/render/copy/filter/blend/MSAA/HDR capability table | `FormatCapabilityTable`参与resource/pipeline admission |
| P1-005 | `RenderDeviceLimits`只投影少量limits，没有alignment、texture dimensions、attachments、workgroups、push constants和binding arrays全量约束 | backend-normalized limit catalog与最小tier |
| P1-006 | `request_device`无条件要求`RG11B10UFLOAT_RENDERABLE`，即使adapter不支持也请求失败 | required tier、optional feature和typed fallback分离 |
| P1-007 | `MemoryHints::Performance`固定，缺memory class、UMA/discrete、budget/pressure与低内存策略 | adapter memory model和runtime budget policy |
| P1-008 | 没有device creation attempt history、enabled/disabled feature reason与driver workaround receipt | 可诊断、可复现的`DeviceCreationReceipt` |

### 7.2 Resource identity、descriptor 与 lifetime（P1-009至P1-016）

| ID | 当前差距 | 目标 |
|---|---|---|
| P1-009 | 八类handle均是公开可伪造裸`u64` | private packed generational handle，至少含device/generation/slot |
| P1-010 | 不同resource kind和不同device可碰撞，错误只按raw number报告 | registry验证kind、owner、generation并输出typed provenance |
| P1-011 | production没有统一resource registry，descriptor query与destroy contract只存在test simulator | production registry保存descriptor、native object、name、bytes、state与last use |
| P1-012 | destroy没有deferred retirement、last submission或in-flight lease | `RetirementQueue`按queue completion回收 |
| P1-013 | 没有allocator/heap/page/residency/defrag/eviction层，memory stats只是请求量或局部cache bytes | `GpuMemoryAllocator + ResidencyManager + BudgetSnapshot` |
| P1-014 | texture format/dimension/aspect/view vocabulary窄，缺compressed/depth-stencil plane、array/cube/3D view、mutable format和sample capability | 完整resource/view descriptor与format admission |
| P1-015 | shader module只有source String、single stage与entry point，缺artifact hash、reflection、specialization、diagnostic和backend binary | immutable shader artifact + reflected interface ABI |
| P1-016 | bind group缺buffer range、dynamic offset、texture view、storage access、arrays与sampler/texture compatibility | typed binding descriptor并由reflection验证 |

### 7.3 Command、Queue、Submission 与 Completion（P1-017至P1-024）

| ID | 当前差距 | 目标 |
|---|---|---|
| P1-017 | `RenderDevice`因generic label非object-safe，且resource calls与queue calls混在单trait | object-safe device/resource/queue接口或稳定service handle |
| P1-018 | command list无begin/end state、render/compute/copy context类型、thread-affinity与allocator/pool lease | typed command context和reusable pool |
| P1-019 | neutral command缺texture-to-texture、clear/copy variants、compute pass、resolve/query、indirect/multi-draw | advertised operation全集与capability gate |
| P1-020 | queue没有batch submit、wait/signal、present、priority、label和native interop边界 | `RhiQueue::submit(SubmissionPacket)`返回ticket |
| P1-021 | fence没有queue/device/generation，一次性sync point与timeline semaphore也未区分 | typed timeline point和CPU-visible completion handle |
| P1-022 | 没有central submission admission、batching、backpressure、ordering receipt或submission worker | `SubmissionService`唯一分配序号并记录依赖 |
| P1-023 | command allocator/encoder/buffer回收没有completion资格，产品各自创建encoder | completion-aware `CommandContextPool` |
| P1-024 | blocking等待只能整设备`poll(wait_indefinitely)`，没有等待目标ticket、timeout/cancel或继续有用工作的API | targeted await/poll/future和bounded timeout policy |

### 7.4 Readback、Query 与 GPU diagnostics（P1-025至P1-032）

| ID | 当前差距 | 目标 |
|---|---|---|
| P1-025 | queue保存device却让prepare/poll传外部device，source也无same-device证明 | readback service只接受registry handle并绑定DeviceGeneration |
| P1-026 | buffer request只检查非空和copy alignment，不在admission时校验`range.end <= buffer.size()` | descriptor-aware source range验证 |
| P1-027 | 三槽限制frame数量但不限制每帧request数、pending bytes、单请求bytes或总staging residency | requests/bytes/staging/callback四维quota与拒绝原因 |
| P1-028 | ticket只含`u64`且无查询状态、frame/submission/source/bytes | structured ticket和observable state machine |
| P1-029 | cancel不回调，Drop无terminalization，owner shutdown/device loss无一次终态 | exactly-once terminal callback/future |
| P1-030 | callback在poll owner线程内联执行，虽catch panic但可阻塞render线程或重入 | completion dispatch queue与执行预算 |
| P1-031 | texture readback硬编码RGBA、mip0、origin0、单layer，无法表达aspect/format conversion/3D/cube/region | typed texture subresource readback request |
| P1-032 | timer/statistics各自poll、按pass分配String、completed VecDeque无界，statistics聚合为线性/重复扫描且忽略部分admission error | unified query/readback scheduler、interned pass id、bounded result ring与loss counters |

### 7.5 Surface、Present、Device Loss 与产品提交（P1-033至P1-040）

| ID | 当前差距 | 目标 |
|---|---|---|
| P1-033 | `new_owned`可为UI再创建Instance/Adapter/Device/Queue，形成第二资源域 | product只允许共享`RhiDeviceOwner`；standalone sample另立binary边界 |
| P1-034 | `copy_texture_for_external_image`直接submit、零尺寸用`max(1)`掩盖且不返回ticket/error | validated copy request加入统一submission packet |
| P1-035 | UI present在submit后立即标cache ready和present，没有submission receipt、generation pin或failure rollback | cache state绑定Submitted/Completed/Invalidated timeline |
| P1-036 | UI和viewport各自选择format/present mode/latency；UI只偏好两个non-sRGB 8-bit格式且固定latency 2 | 统一SurfacePolicy，显式color space/HDR/alpha/latency/VRR |
| P1-037 | UI把Suboptimal当成功；viewport把Lost/Outdated/Timeout/Occluded压成`Ok(())`，缺reconfigure generation与telemetry | typed acquire/present outcome和surface state machine |
| P1-038 | 全仓无device-lost/uncaptured-error callback、OOM分类、quiesce/recreate或resource resurrection contract | `DeviceErrorSupervisor + RecoveryPlan + GenerationSwap` |
| P1-039 | compiled graph之外仍有readback、surface、texture mip/streaming、writeback、cubemap、history、direct scene等ad-hoc submit | 所有GPU work进入一个submission owner，例外必须显式native interop lease |
| P1-040 | backend readback helpers重复创建staging、直接submit并整设备等待；capture也等待所有readback而非目标ticket | 一个async readback service，sync API只是有timeout的上层适配 |

### 7.6 UI边界、产品lifetime、tests 与资格证据（P1-041至P1-048）

| ID | 当前差距 | 目标 |
|---|---|---|
| P1-041 | `zr_rhi`低层crate承载UI draw list/style/image table等高层contract | UI contract迁往framework/render/ui，RHI只保留surface、texture、command primitive |
| P1-042 | shared registry与每presenter local cache各自64MiB/256项，同一external texture在local cache再次按全bytes计量 | 全局residency owner按physical allocation去重，CPU/GPU/shared分别计账 |
| P1-043 | `queue.write_buffer/write_texture`是隐式submission但不分配engine ticket，无法进入lifetime/telemetry/backpressure | upload scheduler统一batch并生成receipt |
| P1-044 | compiled scene在submit后即把transient backing归还pool，只依赖WGPU内部持有和单queue顺序，没有引擎completion资格 | physical reuse由compiled lifetime与queue completion共同许可 |
| P1-045 | direct scene的`complete_submission(..., true)`和state rollover表达“已成功提交”，名称与后续消费容易误读为GPU完成 | 区分Accepted/Submitted/Completed并重命名状态接口 |
| P1-046 | framework boundary test固化“neutral device不得进product”，测试目标与架构方向相反 | conformance test要求产品backend实现neutral contract且禁止raw WGPU越界 |
| P1-047 | 缺cross-device handle、stale generation、Drop/cancel exactly-once、quota、device loss、poll failure、source bounds和targeted completion tests | deterministic fault injection + backend contract matrix |
| P1-048 | 没有真实adapter matrix、surface resize/minimize、device loss、OOM、long-run readback、submission latency或resource churn benchmark | Windows-native产品资格矩阵与可比较基线 |

## 8. P2长期能力（12项）

| ID | 长期能力 | 前置条件 |
|---|---|---|
| P2-001 | D3D12、Vulkan、Metal独立production backend | neutral contract、conformance suite与DeviceProfile稳定 |
| P2-002 | multi-adapter、linked/unlinked multi-GPU与显式node mask | DeviceId、queue timeline与resource visibility先完成 |
| P2-003 | sparse/tiled resource、virtual texture residency与feedback | memory allocator、residency budget和completion-qualified mapping |
| P2-004 | ray tracing BLAS/TLAS、ray pipeline和shader table | typed resource/pipeline/queue contract |
| P2-005 | mesh/task shader、work graph与GPU-driven command generation | indirect/bindless/feature tier先闭环 |
| P2-006 | bindless descriptor arena、descriptor compaction与generation validation | reflection ABI和resource registry |
| P2-007 | persistent pipeline library、driver-keyed cache和offline PSO database | adapter/driver/artifact identity稳定 |
| P2-008 | HDR10/scRGB、display calibration、VRR与present timing | unified SurfacePolicy与color management |
| P2-009 | low-latency frame pacing、present wait和input-to-photon telemetry | submission/present timeline统一 |
| P2-010 | GPU breadcrumbs、crash dump、page-fault allocation correlation | stable resource/submission ids和fault artifact store |
| P2-011 | heap defrag、eviction、residency priority与memory pressure feedback | allocator/budget/retirement完成 |
| P2-012 | 对Unreal/参考场景的可重复性能与画质超越计划 | correctness gates、capture、benchmark和统计方法先可信 |

## 9. 目标架构与hard cut

### 9.1 目标组件

```text
zircon_runtime framework/render
  RenderService
    -> AdapterCatalog / AdapterSelectionPolicy
    -> RhiDeviceOwner(DeviceId, DeviceGeneration)
       -> immutable DeviceProfile + CreationReceipt
       -> DeviceErrorSupervisor
       -> ResourceRegistry
          -> GpuMemoryAllocator / ResidencyManager
          -> RetirementQueue
       -> CommandContextPool
       -> SubmissionService
          -> QueueTopology
          -> SubmissionPacket -> SubmissionTicket
          -> CompletionService / UploadScheduler
       -> ReadbackService / QueryService
       -> SurfaceService / SurfaceSession / PresentTicket
       -> RhiDiagnosticsSnapshot / FaultArtifact

zr_rhi
  object-safe contracts + typed descriptors/handles/tickets/errors

zr_rhi_wgpu
  production WgpuRenderDevice implementing zr_rhi
  WGPU capability/format/limit mapper
  WGPU error callback bridge and surface backend

RenderGraph (Runtime89)
  consumes DeviceProfile
  emits DeviceQualifiedRenderGraphPacket
  submits only through SubmissionService
```

`RhiDeviceOwner`是唯一真实device authority。Resource、command、readback、surface和diagnostic都携带`DeviceGeneration`；recreate发布新generation后，旧handle fail closed。`SubmissionService`区分Accepted、Submitted和Completed，resource retirement、query/readback、cache readiness、surface present和allocator reuse都消费同一timeline，不再各自发明frame number。

### 9.2 必须维持的不变量

1. capability只有在同backend operation、validation与conformance test同时存在时才能advertise。
2. resource handle无法由用户构造，且任何操作都验证device、generation、kind和slot generation。
3. `destroy`只撤销逻辑可见性；native memory只能在所有相关queue completion后回收。
4. 每个GPU submission都有全局可关联ticket；queue-local timeline与跨queue wait明确记录。
5. readback/query/surface request在成功、失败、取消、device lost和shutdown下exactly once terminal。
6. device lost先stop admission，再terminalize/retain可恢复状态，最后按policy fatal或generation swap。
7. product module不得直接调用`wgpu::Queue::submit`或创建device；native escape hatch必须有scoped lease和审计。
8. 同一physical allocation只计一次GPU residency，view/cache/reference不得重复冒充allocation bytes。
9. blocking API只等待指定ticket并有timeout/cancel；禁止把整设备idle当普通读回流程。
10. benchmark只能在correctness、error和lifetime gates通过后用于“优于Unreal”等结论。

### 9.3 Hard cut清单

- 把`DeterministicRhiContractDevice`移入纯tests fixture，删除“产品必须绕过neutral RHI”的架构断言。
- `zr_rhi_wgpu`新增production implementation后，一次性把`graphics::backend`、scene resource、UI和surface切到`RhiDeviceOwner`；不保留平行raw-WGPU facade。
- 私有化全部handle raw constructor；调试序列化使用opaque diagnostic id，不暴露可回灌native identity。
- 删除flat capability与无法执行operation的组合；不以shim把单queue命名为三个async queue。
- 删除owned UI second-device product路径；共享device失败必须返回typed unsupported/failure，不静默另创device。
- 删除重复blocking readback helper；同步调用经统一ReadbackService适配并等待具体ticket。
- 收口所有direct submit与queue write；Runtime89 compiled submit、upload、surface、readback和UI进入同一SubmissionService。
- 不保留`complete_submission(..., true)`这类把accepted误称completed的兼容命名。

## 10. 依赖有序重构里程碑

| 里程碑 | 内容 | 完成条件 |
|---|---|---|
| M0 合同冻结 | 写P0 regression tests、raw WGPU边界inventory、capability/operation矩阵与failure taxonomy | 测试先红，所有例外有owner和删除条件 |
| M1 Device foundation | AdapterCatalog、selection receipt、DeviceProfile、DeviceId/Generation、WGPU error callbacks | adapter/device facts可序列化，lost/OOM/validation进入状态机 |
| M2 Production neutral backend | object-safe `zr_rhi`、真实`WgpuRenderDevice`、generational handles、production registry | 产品最小offscreen triangle/compute/copy经neutral contract运行 |
| M3 Submission/completion | QueueTopology、CommandContextPool、SubmissionPacket/Ticket、CompletionService | Accepted/Submitted/Completed可区分，targeted wait与fault injection通过 |
| M4 Resource/memory | allocator、budget、residency snapshot、deferred retirement与upload scheduler | destroy/reuse只在completion后发生，memory统计按physical allocation准确 |
| M5 Readback/query | 合并readback/timer/statistics，subresource request、quota、exact terminal、dispatch queue | cancel/Drop/lost/shutdown全部一次终态，无普通路径device-wide wait |
| M6 Surface/present | 统一UI/viewport SurfacePolicy、SurfaceSession、PresentTicket、HDR/color space与reconfigure generation | resize/minimize/lost/suboptimal/timeout矩阵通过，产品仅共享device |
| M7 Shader/pipeline ABI | shader artifact、reflection、typed binding、format table与driver-keyed pipeline cache | capability、resource、binding和pipeline在creation前fail closed |
| M8 Product hard cut | 移除raw product device/queue/direct submit/blocking helpers和反向framework test | source guard无未授权`wgpu::Queue::submit/request_device` |
| M9 资格与基线 | Windows真实GPU、device loss/OOM、surface、soak、memory、latency与capture矩阵 | 48门全绿并生成可审计receipt，才进入性能超越阶段 |

顺序不可颠倒。尤其不能先批量把`wgpu`调用包进新类型再补generation/completion；那只会把当前问题隐藏到另一层。M1至M3先定义身份和timeline，M4至M7才有可靠的资源、读回、surface与pipeline语义，最后做产品hard cut。

## 11. 资格门（48项）

### 11.1 Device、Capability 与 backend conformance（G01至G08）

| Gate | 验收要求 |
|---|---|
| G01 | 至少两个mock adapter的选择排序、override、denylist和receipt确定性测试 |
| G02 | unsupported required feature返回typed failure，optional feature产生明确fallback |
| G03 | format capability覆盖render/storage/sample/filter/blend/copy/MSAA正反例 |
| G04 | queue topology不把单WGPU queue报告成独立async compute/copy |
| G05 | 每个advertised command都有WGPU conformance test；缺operation时capability必须false |
| G06 | `RenderDevice`可通过稳定object-safe owner使用，production不依赖test-only实现 |
| G07 | driver/backend/device identity进入pipeline/resource/fault receipt |
| G08 | product device创建失败包含adapter、feature、limit与fallback原因，不只透传字符串 |

### 11.2 Handle、Resource 与 Memory lifetime（G09至G16）

| Gate | 验收要求 |
|---|---|
| G09 | forged raw handle在public API不可构造，stale slot generation fail closed |
| G10 | cross-device、cross-generation和wrong-kind handle均返回typed error |
| G11 | logical destroy后新操作失败，但native object保留到last-use completion |
| G12 | graphics/compute/copy多timeline资源只在所有相关completion后回收 |
| G13 | allocator page reuse、deferred delete和shutdown drain有deterministic tests |
| G14 | physical allocation、view、cache alias与CPU copy的memory accounting不重复 |
| G15 | budget超限、pressure与OOM产生有界eviction/failure，不出现无界增长 |
| G16 | 10万次create/destroy churn后handle、registry、retirement和bytes回到基线 |

### 11.3 Command、Submission 与 Completion（G17至G24）

| Gate | 验收要求 |
|---|---|
| G17 | command context begin/end、thread lease、invalid state和pool reuse矩阵通过 |
| G18 | direct/indirect/multi-draw、dispatch与所有copy仅在capability支持时可录制 |
| G19 | batch submit保持packet顺序并返回唯一device/queue/generation ticket |
| G20 | cross-queue wait/signal无未解析timeline、逆序deadlock或静默fallback |
| G21 | Accepted、Submitted、Completed状态在测试中可独立延迟与故障 |
| G22 | targeted poll/wait只等待指定ticket，timeout/cancel不idle整设备 |
| G23 | command allocator/context只在completion后回池，GPU仍使用时不得复用 |
| G24 | source guard只允许SubmissionService和批准的backend文件调用native submit |

### 11.4 Readback、Query 与 terminal semantics（G25至G32）

| Gate | 验收要求 |
|---|---|
| G25 | buffer range在admission时验证size、usage、alignment、device和generation |
| G26 | texture mip/layer/aspect/region/format conversion正反例通过 |
| G27 | request count、per-request bytes、frame bytes和total staging quota均可触发typed拒绝 |
| G28 | success/map failure/cancel/Drop/device lost/shutdown每个request只终结一次 |
| G29 | callback/future在指定completion executor运行，慢callback不阻塞render submit |
| G30 | staging grow/shrink、slot pressure与memory accounting在长跑后稳定 |
| G31 | timer/statistics/readback共享一次poll owner，result ring有上限和drop counter |
| G32 | capture/readback同步适配只等待自身ticket并有明确timeout receipt |

### 11.5 Surface、Device loss 与 recovery（G33至G40）

| Gate | 验收要求 |
|---|---|
| G33 | UI与viewport共享同一DeviceGeneration，产品代码不能创建第二device |
| G34 | format/color space/HDR/alpha/present mode/latency选择生成统一policy receipt |
| G35 | resize、zero-size、minimize、occluded、timeout、outdated、lost、suboptimal状态矩阵通过 |
| G36 | cache readiness和external image generation绑定submission/present ticket |
| G37 | device-lost与uncaptured-error callback只登记首要错误并停止新admission |
| G38 | in-flight readback、query、surface和resource lease在device loss后全部终态 |
| G39 | recovery创建新generation，旧handle/surface/ticket一律fail closed |
| G40 | fatal路径输出adapter、driver、last submissions、resource ids、breadcrumbs/fault信息 |

### 11.6 产品、压力与性能证据（G41至G48）

| Gate | 验收要求 |
|---|---|
| G41 | offscreen、scene、compiled graph、direct fallback、UI和viewport全走production neutral backend |
| G42 | texture streaming、history、IBL、writeback、upload和surface无ad-hoc direct submit |
| G43 | 全仓无普通产品`wait_indefinitely`，批准的shutdown/fatal路径有清单 |
| G44 | 真实Windows adapter至少覆盖一套discrete与一套UMA/fallback profile |
| G45 | 30分钟readback/query/surface resize soak无request、staging、cache或resource泄漏 |
| G46 | device loss/OOM injection不会hang，终态和恢复/fatal receipt可复现 |
| G47 | submission CPU cost、GPU queue latency、allocator churn、readback latency与memory peak有基线 |
| G48 | 只有同场景、同画质、同硬件、同统计方法通过correctness后，才允许声明性能或表现优于Unreal |

## 12. 禁止的临时实现

- 禁止新增只在`#[cfg(test)]`实现的“production-shaped backend”来宣称RHI完成。
- 禁止用`Arc<wgpu::Device>`或`Arc<wgpu::Queue>`再包一层就称为device owner。
- 禁止继续公开`Handle::new(raw)`并依赖HashMap查不到来模拟generation safety。
- 禁止把单物理queue复制命名为Graphics/Compute/Copy后宣称async queue完成。
- 禁止在capability中先填true、等待未来补command实现。
- 禁止用frame index固定延迟代替GPU completion，或用queue acceptance代替Completed。
- 禁止在readback cancel/Drop/device loss时静默丢callback。
- 禁止用`device.poll(wait_indefinitely)`作为正常截图、烘焙或inspection流程。
- 禁止UI/viewport在共享device失败时静默创建第二device。
- 禁止Surface Lost/Timeout/Occluded/Suboptimal统一压成成功且不记录状态。
- 禁止每个产品子系统继续直接submit或queue-write，再用source test保证“本文件只有一次submit”。
- 禁止把UI layout/style/text算法继续下沉到RHI来规避framework owner。
- 禁止在没有真实GPU、fault和benchmark receipt时声称已达到或超过Unreal。

## 13. 完成边界

本篇是review和重构输入，不是implementation acceptance record。`review_status: complete`只表示上述冻结语料、差异、owner、里程碑和门禁已形成。完成Runtime90实现至少要求M0至M9按顺序落地、48个资格门全部有可复核证据、产品raw WGPU路径完成hard cut，并由Runtime89的device-qualified graph packet接入同一SubmissionService。

2026-08-21 的 review 轮次没有修改 Rust/Cargo，也没有运行 Cargo、真实 GPU、RenderDoc、device loss、multi-queue、fault、soak 或 benchmark。2026-08-24 的源码实施增量与验证限制记录在下方；后续实现仍必须在共享 working tree 上重查本篇 P0 证据，尤其是当前已有其他会话改动的`gpu_pass_timer.rs`。tooling继续按用户要求排除。

### 13.1 当前实施状态（2026-08-23）

`M0`已完成源级合同冻结，但尚未被 managed Cargo 验收，故整体状态为`m0_implemented_validation_blocked`，不是 accepted。

- 已在`zr_rhi`建立固定大小、默认 fail-closed 的`RenderOperationMatrix`，覆盖 direct draw、indexed draw、compute、三类 copy、debug marker/group、indirect/multi-draw、async queue 与 capture 共14项中立操作；查询和 admission 不分配堆内存。
- `RenderDevice::require_operation`与`RhiError::UnsupportedOperation`提供结构化拒绝路径；M0 tests 锁定 native、emulated、unsupported 与 enum/table 连续性。
- `DeterministicRhiContractDevice`测试双只对其实际可执行的八项中立 command（direct/indexed draw、compute、三类 copy、debug marker/group）报告`Native`；间接绘制、异步拓扑、capture 与 production WGPU operation 继续 fail-closed，避免测试 owner 产生 capability 与实现不一致。
- `CommandListCommand::required_operation`以穷尽匹配把这八类可执行 command 映射回中立 operation；`RenderDevice::require_recorded_command_operations`在不分配堆内存的情况下逐项 admission，deterministic submit 在资源验证和执行前调用它。新的 contract tests 固定映射，并证明未声明`DebugMarker` capability 的已录制命令会在 submit 时结构化拒绝，调用方不能把 capability 表仅当作可绕过的提示。此保证只覆盖现有 test-only backend；production WGPU 的 command execution 仍留待 M2。
- `zr_rhi_wgpu::wgpu_backend_caps`保留原始 WGPU capability diagnostics，并将 graphics/compute/copy 报为可经同一物理 queue 串行执行的逻辑 command lane；`supports_async_compute`和`supports_async_copy`保持 false，不把该降级路径误报为独立 async queue。所有中立 operation 仍为`Unsupported`，直到 M2 的真实 production `WgpuRenderDevice`能通过 object-safe command/queue 表面执行它们；因此也不会预先宣称 multi-draw 或 capture 已可经中立 RHI 调用。
- `rustfmt --check`和`git diff --check`已通过。managed validation ticket `bc62da97711c43db9de5b080282644cf`在 Cargo 启动前因`validation_copy_baseline_drift`失败：副本闭包规划器在 pin `HEAD`后仍读取 live workspace members，混入其他会话的 UI 输入。复审还确认其只解析第一个`-p`，而该 ticket 显式请求`zr_rhi`与`zr_rhi_wgpu`两个包；正确修复必须解析全部显式包并只取其传递依赖闭包，无`-p`时才覆盖整个工作区。该失败不证明 RHI 测试失败，也不能作为 M0 acceptance evidence。
- 为修复上述验证闭包而提交的 coordinator 子会话注册请求`a178e07d5ee148d19727e1a5ac74c8bb`在服务端 accepted 后终态为`failure_snapshot_stale`：其他会话更新了 Editor failure artifact，导致全局 failure import snapshot 失效。2026-08-22 随后的`failure import`请求`a27a64f288164345afe67e037f467029`已完成刷新；最小修复会话重试`d77f1cae132a4a33939b4275954c81dd`则被`plan_wip_limit_reached`拒绝，因为`coordinator01-prestart-rollover-r4-20260822`仍是该计划族的`resolving_failure` primary，且其 source scope不含`validation_copies.py`。子会话没有落库，工具代码和 Coordinator01 failure artifact 均没有由本会话修改；待该 primary 结束或显式让出计划族后，才能取得最小工具 scope并重试。
- 2026-08-23 已修复跨计划的`gpu_readback_queue::tests::readback_layout_failure_preserves_callbacks_for_abort`夹具生命周期缺陷：保留`offscreen_test_device()`返回的`wgpu::Queue`直至该测试完成，令测试实际抵达既有的`CapacityOverflow`与一次性 abort callback 契约。该 focused managed replay 已通过；完整`zr_rhi_wgpu --lib` current-source 复跑尚未形成 receipt，因此相关 failure 保持`open`，不改变 M0 的`m0_implemented_validation_blocked`状态。
- `M1`当前状态为`m1_source_in_progress_validation_pending`，不是 accepted，也不解除 M0 的`m0_implemented_validation_blocked`状态。为避免 MVP 基础帧继续被非 core feature 阻断，已在不触碰 M2 command/resource owner 的前提下完成以下 M1 基础代码：
  - `zr_rhi`新增可序列化的`DeviceId`、`DeviceGeneration`、`RenderAdapterFacts`、`RenderAdapterCatalog`、`AdapterSelectionPolicy`与`RenderDeviceProfile`。选择 receipt 记录选中 adapter 和 override mismatch、denylist、software rejection、lower priority 的候选原因；合成 facts contract tests 覆盖枚举顺序稳定、override、denylist、hard requirement 与 optional fallback。
  - MVP `RenderDeviceRequestPolicy`默认不请求 HDR、bindless、timestamp、statistics、indirect 或 subgroup optional features。`zr_rhi_wgpu`是唯一把中立 feature 映射到 WGPU bits 的位置；`graphics::backend`只消费协商结果，且仅在 bindless profile 实际获准时才请求 binding-array limits。
  - 主 device 创建失败不再把`wgpu::RequestDeviceError`作为`GraphicsError` API 向上传递；中立`RenderDeviceRequestFailure`保留 adapter facts、feature negotiation/fallback、requested limits 与 backend diagnostic detail，供后续 profile fallback 和故障诊断消费。
  - offscreen backend 在初始化冷路径枚举 WGPU adapters，依中立 policy 选择后创建设备，并将 adapter facts、DeviceId、initial generation、feature negotiation receipt、实际 negotiated limits 与单物理串行 queue topology 保存在不可变 profile。该路径不再把单一`HighPerformance`请求或 WGPU 枚举顺序作为选择策略。
  - `DeviceFaultGate`以 healthy acquire-load、recording/faulted fail-closed 和首个 fault mutex record 建立 M1 admission 基础；`WgpuDeviceErrorSupervisor`是当前主 device 上注册`on_uncaptured_error`和`set_device_lost_callback`的 owner，回调只记录 OOM/validation/internal/unknown-loss/destroyed，不执行 poll、destroy 或 recovery。
- M1尚缺 surface format provenance、用于产品 raw-WGPU owners 的全面 admission、fault injection receipt 及 managed Cargo/真实 device 验证；因此没有生成 PNG、RDC、GPU timing、功耗、device-loss、surface 或 soak 证据，也不得作性能、画质或 Unreal 对比声明。
- 因尚无 production neutral backend，当前没有生成`docs/tests/runtime/render/` PNG、RenderDoc RDC、GPU timing、功耗、device-loss、surface 或 soak 证据；M9 前不得作性能、画质或 Unreal 对比声明。

#### 13.1.1 当前源码实施增量（2026-08-24，未验收）

- `M2-M4`已完成当前 working tree 的源码闭合，但均未获得 managed Cargo receipt，不能标记 accepted：`WgpuRenderDevice`拥有 generation-qualified registry 和唯一 native queue service；`SubmissionHistory`将 caller-visible terminal status 与压缩 terminal range 分离，限制 unresolved submission；physical buffer/texture backing、retired backing 与 upload staging 都在 profile budget 下 admission；resource retirement 保留所有未终态 use ticket，避免“较晚取消 ticket 覆盖较早已提交 use”时提前释放；新增 10 万次 create/destroy churn 合同。2026-08-24 的源码增量还让同一 neutral command path 实际编码 color-texture `BufferToTextureCopy`与`TextureToBufferCopy`，并将两项 capability 报为`Native`；usage、region、buffer range、WGPU padded-row 规则均在 encode 前复验。它们不是 CPU readback，`supports_buffer_readback`仍为 false；由于中立 DTO 尚无 texture aspect，depth/stencil copy 显式拒绝，不能将该增量误报为完整格式支持。
- `P1-016`当前源码增量：bind group 的 buffer entry 已硬切为`BindGroupBufferBinding { buffer, offset, size }`，layout 具有`has_dynamic_offset`与`min_binding_size`，command list 以`set_bind_group_with_dynamic_offsets`记录动态 offset。deterministic 和 production WGPU owner 都在 admission/encoding 前复验范围、usage、最小大小、实际 negotiated uniform/storage 对齐，并按 binding 编号升序解释动态 offset；graph compute schema 同步改为静态`RenderGraphBufferBindingRange { offset, size }`，对已知 transient descriptor 在 compile 期拒绝越界、空 range 与缺失`UNIFORM`/`STORAGE` usage，外部资源在执行期复验。texture view、storage texture 和 sampler 的既有 typed binding 路径继续适用。此项仅为 source implementation，managed Cargo/真实 GPU 验证尚未完成；binding arrays 与 shader reflection 仍是 P1-016 未完成范围，不能将该条目改为 accepted。
- 资源预算、submission limits 和 diagnostic readback budget 已统一进入 immutable `RenderDeviceProfile`，offscreen backend 与 production-device fixture 都只能从该 profile 获得配置，不再维护彼此漂移的临时常量。
- `M5`当前继续处于 source foundation 阶段，尚未替换产品 raw-WGPU readback：中立 tracker 以`DiagnosticFrameKey { DeviceId, DeviceGeneration, SubmissionTicket }`绑定 admission，按单请求、单帧、in-flight、staging 与 result-ring quota 生成一次性 terminal receipt；`zr_rhi_wgpu::production::diagnostics`保留私有 query-plan，使用 dense `PassDiagnosticId(u32)`、encode 前 query-scope 限额与线性聚合，拒绝字符串查找。`WgpuRenderDevice`现拥有 generation-local 的混合 buffer/color-texture diagnostic batch：buffer handle/usage/range/alignment与texture handle/`COPY_SRC`/mip-layer-slice region均在 native encode 前复验；每个请求在 admission 时计算实际 staging offset，纹理行按 WGPU 256-byte row alignment 填充并在 delivery 前去填充，故混合请求的对齐空洞也计入 quota。全部 admitted request 编为一个 staging copy packet，只经`WgpuSubmissionService`提交；其 map 仅在该 ticket 已提交后安排，完成只在 device 的单次 submission poll 后收集为有界 delivery ring。该 ring 同时受 receipt 数量和`max_pending_bytes` payload 上限约束，并报告淘汰计数；取消、device loss、map failure 和 source encode failure均以一次 terminal receipt 结束。该路径仍不开放中立同步 readback capability；深度/stencil转换、native timestamp/statistics resolve、completion executor与产品 raw-WGPU hard cut尚未实现，故不能宣称 M5 completed。已有`GpuReadbackQueue`现在只使用构造时保存的 device，默认采用`DiagnosticReadbackBudget`，也可用显式预算构造；它在 request 前检查单请求、单帧及物理 in-flight 额度，并在未编码取消、abort 与映射终结时归还同一物理计数。scene/UI 在各自帧入口只 poll 一次，timer/statistics 只 drain completed ring；encoded frame 不再接纳新 request，cancel/Drop 以`Cancelled`/`Shutdown` callback 一次性终态化。legacy queue和新的 production batch都尚未完成产品 single-poll hard cut，仍属于后续 M5/M8 工作。
- 本轮 managed validation 只尝试仓库规定的`validate-matrix.ps1 -Package zr_rhi_wgpu -LibTests`。命令在启动 Cargo 前被离线 coordinator 拦截，未产生编译或测试结果；因此只记录 rustfmt/static-source 审计，不宣称 build/test 通过，也不生成 PNG、RDC 或性能数据。

### 13.2 M1 实现前架构与热路径决策（2026-08-22）

以下设计以 Unreal 的“RHI 资源生命周期不能超出声明的 command/graph 作用域”及 RDG 的“pass 参数决定依赖和生命周期”为约束。Lumen 参考工程中 PreZ、HZB、clear、lighting 与 final compose 的明确顺序只用于约束上层 graph pass 编排，不会下沉到`zr_rhi`。

- `DeviceId`和`DeviceGeneration`是不可变、可序列化的 owner identity。M1 只定义 identity 和 profile；M2 才以它们私有化 resource handle 构造，M3 才把它们附到 submission ticket。这样不会以裸`u64`或 frame index 伪造 completion safety。
- `AdapterCatalog`保存可序列化 adapter facts，`AdapterSelectionPolicy`先应用显式 override 与 denylist，再按已声明偏好生成稳定的排序 receipt。选择过程是 device creation 前的冷路径，可分配；稳定排序键必须包含 backend、vendor、device、driver 与 adapter type，不能依赖枚举顺序或`HashMap`迭代顺序。
- `DeviceProfile`是创建完成后的 immutable snapshot。feature、limit、format 与 logical queue facts均携带`Native`、`Emulated`或`Unsupported`以及来源/回退 disposition。单物理 WGPU queue 只能产生 graphics queue fact；compute/copy 只能作为同队列 command class，不能在 M1 声称为 async topology。
- `DeviceFaultGate`采用两级状态：正常 admission 只做一次 acquire atomic load；首个 callback 以 CAS 进入 recording，再在低频 mutex 中写入可序列化 first-fault，最后 release store 到 faulted。任何 observer 在 recording/faulted 均 fail-closed。该设计把错误字符串分配和锁竞争限定到故障路径，不向 draw、resource lookup 或 command recording 热路径扩散。
- `DeviceErrorSupervisor`是`zr_rhi_wgpu`中唯一登记`wgpu::Device::on_uncaptured_error`和`set_device_lost_callback`的 bridge。它把 WGPU `OutOfMemory`、`Validation`、`Internal`以及`Unknown/Destroyed` loss 映射为中立 fault record；M1 不在 callback 中销毁资源、poll device 或重建设备。M3-M6 的 completion、retirement、surface 和 recovery policy 消费该记录。
- 当前 Runtime89 只有`CompiledRenderGraph`和`RenderGraphExecutionRecord`，尚无 device-qualified packet 或 submission ticket。因此 M1 不添加假 packet 或平行 submit API；M3 应让 Runtime89 的 compiled graph 以`DeviceProfile`编译并由唯一 SubmissionService 产生 timeline-qualified ticket。

实现前参考证据（非 M1 实现或性能结论）：Unreal `dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/RHIGlobals.h`的`FGpuInfo`只在 RHI 初始化后写入并保持不变，包含 adapter name、driver version、vendor/device id、adapter generation 与 dedicated memory；M1 应同样把这些可观察事实固化在 immutable profile，而不是散落在资源或 submit owner。Bevy `dev/bevy/crates/bevy_render/src/renderer/render_device.rs`将`wgpu::Device`封装在 render crate，公开 features、limits 和显式 poll，且其局部测试保持在该 owner 内；Zircon 不复制该薄 wrapper，而是补齐 Bevy 未表达的 generation、selection receipt 与 first-fault state。WGPU 29 的`src/api/device.rs`要求`on_uncaptured_error` handler 为`Send + Sync + 'static`，`set_device_lost_callback`为`Send + 'static`，`wgpu-types`的`DeviceLostReason`仅有`Unknown`和`Destroyed`；因此 supervisor 注册只能在 device construction 后一次完成，并将 callback 输入收敛为中立 first-fault record，不能在 callback 中进行 destroy、poll 或 recovery。

性能验证约束：M1 的 atomic admission、profile snapshot 和 selection receipt 先通过纯 source/contract tests；实际纳秒、CPU submit、GPU latency、memory peak 和功耗数据只在 M9 依据 Render17 的同场景 60-frame warm-up 加至少 300-frame cold/warm 样本采集。PNG、RDC、graph dump 与 profile 必须共享同一 generation，未完成这些条件前不得做任何性能结论。

### 13.3 实施前结构性热路径审计（2026-08-22）

本轮只做源码计数，不能替代 profiler。`zircon_runtime/src/graphics`、`zr_rhi`与`zr_rhi_wgpu`当前分别合计有21处`request_device`、10处`request_adapter`、36处`wait_indefinitely`、11处`complete_submission`和167处`.submit(`调用。它说明瓶颈首先是设备/提交所有权分散，而不是单个 draw 或 allocator 的微优化对象。

因此优化顺序固定为：M1 先建立不可变 device profile 与 fail-closed fault gate；M2 把资源和 command 编码绑定同一 device generation；M3 由唯一 SubmissionService 产生可完成 ticket；M4-M8 才逐项删除这些 raw owner。M9 使用 Render17 的60帧预热和至少300帧样本，结合同 generation 的 PNG、RDC、graph dump 与 profile 验证该结构收敛是否消除阻塞等待。没有这些实测资料时，不报告耗时、功耗或对 Unreal 的比较结论。

### 13.4 MVP device-request compatibility recheck（2026-08-23）

本轮重审确认，在 M1 之前还有一个会直接阻断 MVP 基础帧的结构性问题，不能以“高性能优先”名义保留。`render_backend_new_offscreen.rs`以`HighPerformance`请求单一 adapter 后立即调用`request_device`；后者的`required_render_features`无条件加入`RG11B10UFLOAT_RENDERABLE`。同文件的单元测试还明确锁定了“adapter feature 集为空时该 feature 仍为 required”的行为。WGPU 29 本地类型定义把该项标记为受支持平台集合约束的 feature，而非全部 backend 的 core requirement。因此 adapter 选择、MVP 是否能生成非空帧、HDR format 是否可用三件事被错误地折叠为一次 opaque device request。

2026-08-23 对`zircon_runtime/src/graphics`的机械源码计数（包含测试代码，不能替代 profiler）为：6处`request_adapter`、11处`request_device`、56处`.submit(`和31处`wait_indefinitely`。`RG11B10UFLOAT_RENDERABLE`的4处引用均在同一 device-request 文件；生产创建路径没有记录候选 adapter、所选 profile、被拒的 feature 或 fallback disposition。失败最终只会以`GraphicsError::RequestDevice(wgpu::RequestDeviceError)`向上传递，既不能区分“没有 adapter”“MVP baseline 无法创建”与“可选 HDR profile 被拒”，也无法为后续 profile、故障诊断或产品验收提供 selection receipt。

M1 实现必须把这一处收敛为明确的冷路径协商，不允许在 `RenderBackend`、scene pass 或 UI surface 各自补条件：

- `AdapterCatalog`先采集 immutable adapter facts，`AdapterSelectionPolicy`按稳定键和显式 override 选出候选；不得让 WGPU 枚举顺序或单个`HighPerformance`请求成为选择策略。
- `RenderDeviceRequestPolicy`将特性和 limits 分为 MVP baseline、已声明的 optional profile（HDR、bindless、timestamp、statistics、indirect 等）和 hard requirement。MVP baseline 只能请求生成基础非空 frame 所必需的能力；optional profile 只有在候选 adapter 支持且上层 graph 明确选择时才进入 `required_features`，其结果必须在`DeviceProfile`中以`Native`、`Emulated`或`Unsupported`及原因固化。
- profile 协商失败必须映射为中立、可序列化的 typed negotiation error 与 selection receipt；WGPU 错误字符串只能保留在`zr_rhi_wgpu`诊断详情中，不能继续向 Runtime89、scene 或 editor 泄漏 WGPU 类型。
- 纯 contract test 要用合成 adapter facts 验证：无 HDR 特性的候选仍能选出 baseline；HDR graph 只在 HDR profile admitted 时可编译；请求 feature 始终是候选 feature 的子集；同一 facts 集合得到相同的排序和拒绝 receipt。该测试不需要 GPU、Cargo 或 RenderDoc，真实 device creation 和 frame evidence 仍留给 M9。

这不是 M1 代码已经完成的声明。它是 M1 开工的设计前置条件；当前 M0 仍必须先取得修复 validation-copy 闭包后的精确 managed Cargo receipt，MVP00 未完成前不得以此为由绕过基础闭环或提前交付 Lumen、multi-queue、HDR 或性能结论。

### 13.5 GPU diagnostic query/readback pre-optimization architecture review（2026-08-23）

本轮对`gpu_pass_timer.rs`、`gpu_pipeline_statistics.rs`和`gpu_readback_queue/queue.rs`做了源码级结构审计。结论不是先压缩单个函数，而是先让诊断 query 与 readback 归属已经完成的 submission：当前两个 collector 分别持有无上限的`VecDeque`，分别在`try_collect`中对原始`wgpu::Device`执行 poll，callback 还携带每 pass 的`String`。完成结果按 frame generation 线性定位再插入，单次为`O(F)`；pipeline statistics 对同一 graph pass 的物理 scope 以字符串线性查找聚合，最坏为`O(N^2)`。`GpuReadbackQueue`虽拥有 device，却仍接受任意外部 device 进行 prepare/poll，pending request 也没有预算；timer/statistics callback 对 readback 或 decode 的`Err`直接返回，因而不能形成可审计的失败终态。这些是 owner、completion 和预算缺失，不是适合直接微调 allocator 的问题。

Unreal 的 D3D12 submission 将 timestamp、pipeline statistics 和 query range 放入同一个 queue payload/batch，以 completion fence 标识可见性；其 query range 按 heap 分组、排序并合并相邻范围后再 resolve，避免每个 pass 独立回读。Zircon 应采纳“submission-qualified batch + completion receipt”的所有权，而不复制 Unreal 的 submission thread 或多物理 queue：当前 WGPU profile 仅允许一个物理 serial queue。Bevy 的`bevy_render/src/gpu_readback.rs`提供 render-owned readback 生命周期和按 buffer size 的复用池，并以`max_unused_frames`限制闲置缓存；但该实现自身假设 readback 数量较少，且注释保留了每帧大 buffer 的后续优化方向，不能代替 Zircon 的显式 ticket、硬预算和故障终态协议。

M3 建立 `SubmissionTicket` 后，诊断 owner 应收敛在私有的`zr_rhi_wgpu` query-batch 模块，而不是添加 root facade 或让 timer/statistics 继续独立 poll。每批使用`DiagnosticFrameKey { DeviceId, DeviceGeneration, SubmissionTicket }`、`QueryPlan`和编译 graph 生成的`PassDiagnosticId(u32)`；热路径只传递整数 ID，只有导出诊断时才 materialize pass label。一个 submission-qualified readback batch 同时承载 timestamp 和 statistics，并且恰好产生一个`Completed`、`Failed`、`Cancelled`、`DeviceLost`或`OverBudget`终态 receipt。`SubmissionService`每 device frame 只能有一个 poll owner；`GpuReadbackQueue`不得再接受与其所有权不同的 device。

容量与失败行为必须先配置化，而不是引入新的裸常量：`DiagnosticReadbackBudget`从 profile/capability 得到 query、staging bytes、in-flight batch 与 completed receipt 上限；超过预算在 encode 前返回`OverBudget` receipt。完成结果以有界、ticket 有序的 ring 保存并记录 drop/overflow counter。decode 直接从映射 bytes 写入预分配的 pass-ID 索引聚合表，避免`Vec<[u64; 2]>`中间分配和按字符串重复查找；其复杂度应为物理 query scope 数量的`O(N)`。device loss、取消、map failure、截断结果和 decode failure 均必须终态化，不允许 callback 静默丢弃。

实现前的纯合同测试须覆盖：零/单个/恰好预算/超预算 query，错误 query count 与截断 bytes，同一 pass ID 的重复物理 scope 在线性聚合后得到唯一结果，readback 早期失败的 receipt，乱序 callback 下按 ticket 的确定性消费，以及每个 device frame 只有一次 poll。M3 之后的集成测试再覆盖 generation mismatch、device loss 终态化、cancel/overflow 和 present 前单 collector。M9 真实 GPU 证据使用同一 1080p 固定场景、60 帧预热和至少 300 帧 cold/warm 样本，PNG、RDC、graph dump 与 profile 必须是同一 generation；对比 diagnostics off/on 的 CPU submit 与 GPU query 开销，功耗单独采样。在这些 receipt 产生前，本轮不报告任何性能、功耗或 Unreal 对比数值，也没有写入诊断生产代码。

### 13.6 Lumen reference versus Zircon MVP graph boundary（2026-08-23）

`dev/LumenInUE5.5.4WithComputeShader/App.cpp`的真实帧序列是有价值的依赖样本：`PreZ -> HZB -> surface cache capture/copy -> Base -> shadow/directional/direct lighting -> radiance cache/screen probe -> temporal -> FinalCompose -> ToneMapping -> SwapChain`，并显式维护 current/last ping-pong history。它同时在帧内至少三次直接`SubmitCommandList`，并在 pass 之间手写 D3D12 barrier、discard 和 resource state。这些后两项是示例自身的 D3D12 执行细节，不能迁入 Zircon 的 runtime 或成为 WGPU 多 queue 设计依据。

Zircon 的`render_graph/builder/compile.rs`已经是正确的声明层起点：它对显式和资源推导依赖执行拓扑排序，拒绝 read-before-producer、read-after-discard 与不匹配的 compute binding，区分执行 hazard 和 semantic provenance 以进行 culling，并把标为 readback 的资源生命周期延长到末尾 compiled pass。2026-08-23 对整个`src/render_graph`的精确搜索没有发现`wgpu::Device`、`wgpu::Queue`、`DeviceId`、`DeviceGeneration`或`SubmissionTicket`；该图应继续保持 device-agnostic，不能为了闭环把设备创建、poll、surface 或 completion 反向塞入 compiler。

MVP 的最小 graph contract 因而只要求可验证的一条基础分支：import/acquire output，明确写入的 scene/clear 或 opaque pass，读取该 scene 的 tone-map/compose pass，以及声明为 external output 的 present/copy pass。每一条读写边都由 graph 声明，实际物理资源和一次 submission 仅在 M2/M3 的 device-qualified materialization/SubmissionService 边界绑定。PreZ/HZB、surface cache、radiance cache、screen probe、indirect dispatch、temporal history、HDR 和 Lumen lighting 都是后续可选 graph branches；它们必须以显式资源版本、history import 与 capability/profile admission 接入，不得被伪装成 MVP 的必经 pass。

实施前的图级合同测试应固定：基础 branch 的输出 producer 未被 cull；资源读写令 pass 顺序可复现；discard 后的读取被拒绝；未声明的 compute binding 被拒绝；readback backing 生命周期覆盖 terminal compiled pass；在未 admission 的 HDR/Lumen profile 下 optional branch 不进入 compiled graph。M3 后的集成测试再要求同一`DeviceGeneration`、同一`SubmissionTicket`串起 materialization、单次 submit、completion、present 与 readback。PNG、RDC 和性能样本仍属于 M9；本节没有把 Lumen 示例、graph 编译结果或源码计数误作渲染效果或性能证据。

### 13.7 Raw submission owner census and migration protocol（2026-08-23）

本轮静态审计覆盖`zircon_runtime/src`的7,530个 Rust 文件。全树三次 PowerShell 递归文本扫描在30秒上限内超时；改用 Git 索引检索后稳定定位了候选。该方法只证明调用面和所有权分散，不能替代 profiler，也不报告 CPU/GPU 耗时。排除测试 fixture 后，当前 raw WGPU queue 调用仍横跨 backend readback、viewport surface、resource streamer、texture/environment/history upload、scene clear/UI helper、generic compute/parallel encoder 与 frame framework。它们不能再被视为彼此独立的“少量辅助提交”。

`scene_renderer/.../submit_compiled_scene_frame.rs`是现有最接近正确模型的 owner：它集中一个 graph frame 的`queue.submit(command_buffers)`，且同目录 source contract 锁定该文件只出现一次该调用并禁止正常帧的`wait_indefinitely`。但这不足以形成全局 pipeline，因为 backend 的`read_buffer_*`、`read_texture_*`、IBL artifact readback 和 debugger capture 仍自行 encode/submit 后无限 wait；`viewport_surface.rs`以及 upload/writeback/historical resource 路径也能绕过 compiled-scene owner。`render_backend_new_offscreen.rs`和`request_device.rs`则仍是唯一明确的生产 device creation 所在，进一步说明 device、submit、completion 目前不是同一 service 的同一 ownership chain。

迁移协议必须以 M3 的私有`SubmissionService`为唯一入口，而不是在每个现有 helper 外层添加新的 queue wrapper：

- graph materialization 交付不可直接提交的 encoded batch；service 为该 batch 分配`SubmissionTicket`并完成唯一物理 queue submit。
- readback、capture、resource upload、IBL/history writeback 和 surface present 分别成为有类别的 batch producer；它们只可携带同一 device generation 的 handle，并通过 ticket completion/cancel/device-loss receipt 收尾。正常产品路径禁止自行 poll indefinitely。
- runtime 以外不得保存可调用的`wgpu::Queue`。`zr_rhi_wgpu`内部的私有 queue owner 是唯一例外；test fixture 可以直接使用 WGPU，但不得被 source-count 误作 production 收敛证据。
- 每一批的录制数、command-buffer 数、CPU encode/submit 时间、queue completion latency、readback bytes、overflow/cancel 和 fault disposition 都归入 ticket telemetry。M9 才以固定场景采集分位数、峰值和能耗；本轮静态 census 不捏造这些数据。

落地前的检查顺序为：先以纯 source contract 列出所有 production raw-submit/readback owner；随后以一次一个 owner 的迁移测试证明其只能通过 batch producer 进入 service；最后由 M9 的 RenderDoc frame、同 generation PNG、graph dump 和 profiling trace 交叉验证。任何“文件只剩一次 submit”的文本断言只能是回归护栏，不能替代 completion、resource lifetime 或真实性能验收。

### 13.8 Neutral product frame owner hard-cut recheck（2026-08-27）

当前源码已经出现了新的结构性分界：production `WgpuRenderDevice`拥有完整 neutral generation、registry、submission/completion、diagnostic 与 surface service，但真实`WgpuRenderFrameworkCore -> SceneRenderer -> RenderBackend`仍创建并持有另一套原生 WGPU device/queue。compiled/direct scene 虽已通过`WgpuSubmissionCoordinator`提交，却在适配层丢弃`SubmissionPollReceipt`和 scene `SubmissionTicket`；surface blit又产生并丢弃第二张 ticket后立即 present。因而 asset residency、surface readiness、readback、retirement和frame profile仍无法关联到一条产品 device timeline。

本轮已完成产品调用链、Runtime89交界、UE D3D12 submission/RHI end-frame 与 Lumen示例帧序列的源码重审，并形成[`90/2026-08-27-neutral-product-frame-owner-hard-cut-design.md`](90/2026-08-27-neutral-product-frame-owner-hard-cut-design.md)。实施顺序固定为：先贯通 poll/scene/present identity，再建立单一 frame transaction，然后让`WgpuRenderDevice`接管产品 device generation，最后把 surface terminal pass、readback/upload与Runtime89 packet并入同一 submission service并删除旧`RenderBackend`。当前普通产品源码仍有6个同步 readback direct-submit点和8个`wait_indefinitely`点；这些是迁移调用面，不是 profiler或性能结论。

第一切片只允许补齐现有产品链丢失的 receipt/ticket，不改变原生提交次数，也不把 scoped native recorder误报为最终 neutral hard cut。真实性能验证仍要求固定场景60帧预热、至少300帧和3次独立steady run，并令PNG、RDC、graph dump、frame receipt与profile同generation；当前没有这些动态证据，M8/M9保持未验收。

2026-08-27 第一身份切片已完成源码实施并通过局部`rustfmt --check`和 scoped `git diff --check`，状态为`source_implemented_static_checks_passed_dynamic_validation_pending`。coordinator/backend poll、compiled/direct scene submit与surface present现在分别向上保留`SubmissionPollReceipt`和`SubmissionTicket`；renderer以`RenderFrameSubmissionReceipt`校验同device generation及present sequence，并把成功结果发布到`RenderStats`。surface acquire重配、超时与遮挡具有独立 typed outcome，不生成虚假的present ticket。新增8个源内合同测试条目，但未执行Cargo或真实GPU测试。

该切片没有完成single-frame transaction：正常present仍为2次native submission；graphics生产树仍有6个同步readback direct-submit点和8个阻塞等待，neutral WGPU retained UI cache另有1个生产阻塞等待；texture pre-upload/history backing也仍可独立提交。另一个必须在PFO-1解决的错误边界是“native submit成功后readback/IBL收尾报错”，其终态需要同时保留ticket和失败disposition。因而Runtime90 M8/M9继续未验收，PNG、RDC、device-loss、性能与功耗结论均为空。

同日 PFO-1a 已把frame-begin completion pump与receipt finalize从direct/compiled core上移到`SceneRenderer`帧入口。两个outer owner均在scene resource/history准备前取得poll receipt，两个core只返回scene ticket；源级调用面只剩两个outer poll点。新增4个顺序/owner合同测试，PFO-0与PFO-1a合计12个新增合同测试条目。该阶段仍没有producer ledger或失败帧cancel，因此pre-upload/history不能直接改为无条件enqueue；PFO-1b必须先记录每个accepted ticket并区分Accepted取消与Submitted失败终态。

PFO-1b 第一部分随后完成history producer identity：`RenderFrameSubmissionTransaction`按同device generation和严格递增sequence记录pre-scene producer，成功frame receipt以可共享的非空`Arc`保留记录，steady无producer路径不创建记录数组。history新建/重建的真实初始化ticket以`HistoryInitialization`进入ledger并被校验为早于scene ticket。新增7个合同测试，PFO-0至当前PFO-1b部分合计19个新增合同测试条目。texture producer、失败帧Accepted cancel、Submitted failure receipt与fault后status查询仍待完成，因此PFO-1整体继续未验收。

### 13.9 Terminal graph ownership and typed output-target contract（2026-08-29，未验收）

- Render01 的 PrimarySurface 与 Texture target 终端路径已在源码层完成一次性规划：`Present` 阶段的 surface pass 只消费最终输出；sRGB Texture target 使用不可裁剪、只读最终输出的 `output-target-direct-import` pass，不声明虚假的 self-copy/write hazard；只有线性 Texture target 才使用 `output-target-writeback` 并声明 `Sampled(FRAGMENT) -> ColorAttachment`，headless 不生成终端 pass。
- `FINAL_COLOR` 与 `VIEWPORT_OUTPUT` 现在由内建 feature 共享 View 尺寸、`Rgba8UnormSrgb`、`RENDER_ATTACHMENT|SAMPLED|COPY_SRC` 的外部纹理 schema。直接导入路径同时绑定真实 WGPU texture、view 和 `TextureDesc`；输出目标资源的 WGPU/RHI usage 始终保留 graph-readable 的 sampled/copy 能力，物化校验不再依赖 view-only 猜测。
- 输出目标 readiness、graph import、writeback、final selection 和 executor 消费同一份 immutable `OutputTargetFramePlan`；直接导入与 suppression 分支不创建物理 writeback 资源，terminal packet 不再编码旧式 writeback，也不新增独立 queue submit。该结构与 UE RDG 的“声明决定依赖/生命周期、执行器只消费编译结果”边界一致；本条是结构正确性收敛，不是性能验收。
- 已完成的证据为 exact camera-target fingerprint/cache key、静态 graph contract tests、精确 rustfmt 与 scoped `git diff --check`。正确的 managed validator 已到达 `E:\\cargo-targets\\runtime-render`，但在 Cargo 启动前因 `cargo_reuse_target_mismatch` 拒绝，未形成 build/test receipt；未伪造 PNG、RDC、RenderDoc、GPU timing、功耗或吞吐数据。
- 后续门禁仍是：协调器提供匹配的独立 target 池后执行受管 Cargo；连接真实 WGPU surface/texture 路径验证 direct import、linear conversion、surface present 与 device-generation receipt；再在 `docs/tests/runtime/render` 写入真实 PNG，并用 `D:\\Tools\\renderdoc` 生成同 generation RDC/graph 对拍。PFO-1b texture producer、失败帧终态与 Runtime90 M8/M9 仍未验收。
- 2026-08-30 EnvironmentOnly direct fallback terminal ownership：重审确认 `render_frame_direct_submission` 只由 `EnvironmentOnlyPbrPreview` 使用；FullScene/StandardPbrPreview 均走 compiled scene graph。`OutputTargetFramePlan` 在资源准备时一次解析并分别固化 `compiled_graph_writeback_plan` 与 `direct_submission_writeback_plan`：sRGB target 在 graph 分支 direct import/跳过 copy，在不执行 graph 的 direct fallback 分支必须 copy；线性 target 两条分支都执行 conversion。直录帧把该 immutable plan 传入 `SceneRendererCore::render_scene`，不再隐式从 frame 重新规划 output target。PrimarySurface 仍由唯一 surface lease 在该专用 fallback 中完成 blit/present，未被误报为 compiled graph pass。该切片仅有 rustfmt、scoped diff 与 source contract 证据；managed Cargo 因 target pool mismatch 未执行，PNG、RDC、GPU timing、性能与功耗仍为空。
- 同日资源描述 builder 按职责拆分：terminal schema 保持在独立 52 行模块，25 个 external-resource API 迁入 355 行的 `external_resources.rs`，`construct.rs` 从 1280 行降到 936 行；该变更不改变公共 builder 名称或资源语义。执行后的 graph-import observation 也从 `OutputTargetFramePlan` 分离到独立 report 字段，executor 不再反向修改本帧不可变计划。

### 13.10 Standalone UI local submission/completion hard cut（2026-08-30，SUI-0至SUI-3源码完成、未验收）

PFO-4a已经让共享UI复用runtime `Arc<WgpuRenderDevice>`，但切入前独占native surface仍自行request device、raw submit，并用本地`GpuReadbackQueue`主动poll。源码重审确认这三个owner必须整体迁移：只替换submit会留下第二个completion owner，不能形成generation-qualified diagnostic、fault和retirement闭环。

实施设计记录在[`90/2026-08-30-standalone-ui-local-submission-completion-hard-cut-design.md`](90/2026-08-30-standalone-ui-local-submission-completion-hard-cut-design.md)。Standalone继续允许独立device，但构造结果必须是immutable profile与唯一`Arc<WgpuRenderDevice>` bundle；present必须返回真实ticket，local timing/readback只能消费同一owner poll后的completion，资源pin与surface终态服从该ticket。SUI-0至SUI-3必须按依赖合入，SUI-1不得作为submit-only facade单独落地。

SUI-0至SUI-3已按依赖落地：standalone和offscreen device共用`initial_wgpu_render_device_profile`；UI context删除无owner可构造状态；native present不再有raw submit fallback；`External/Local` typed owner保证只有local standalone在acquire前推进`poll_submissions`，旧readback queue只能在该poll之后collect。所有native成功present都发布真实ticket，standalone external image继续fail closed。UI image pin随finalized native packet进入submission service，以ticket为键进入有界退休表，并由现有唯一completion callback或fault terminalization锁外释放；device poll错误同批终结submission、diagnostic和surface frame。

基础owner/profile failing-first source contract转为11/11通过，SUI-3 ticket retirement合同从0/8转为8/8；精确rustfmt、scoped diff、结构预算、产品UI raw submit/direct poll/extra completion callback负向扫描及locked metadata通过。当前没有受管Cargo、真实窗口、PNG/RDC、profile或功耗证据；SUI-4、M6/M8/M9和G31/G33-G40/G48保持开放。状态为`runtime90_standalone_ui_sui_0_through_sui_3_source_implemented_static_checks_passed_dynamic_validation_pending`。

### 13.11 Product raw queue authority shrink（2026-08-30，PFO-4d4b源码完成、未验收）

SUI迁移后的raw data-plane复审把直接`queue.submit`候选重新按module `cfg(test)`边界分类；generic compute per-pixel、parallel encoder和scene clear命中均属于测试，不是产品提交旁路。产品仍存在raw queue参数透传，但其中两条没有资源或提交行为：scene resource preparation只把queue传到最终未读取的`_queue`参数，product GPU timer只调用一次`get_timestamp_period`，而该事实已由唯一`WgpuRenderDevice`构造时固化。

[`90/2026-08-30-pfo-4d4b-product-queue-authority-shrink-plan.md`](90/2026-08-30-pfo-4d4b-product-queue-authority-shrink-plan.md)完成这两个hard cut：frame resource/material链不再接收queue；product timer消费device-owner timestamp period。Legacy standalone UI timer继续保留其局部query-set构造，后续迁移必须先建立device-level query consumer routing，不能让UI从scene中央router窃取delivery。failing-first五项全红，实施后扩展合同7/7、精确rustfmt、scoped diff、结构预算和locked metadata通过。其余raw Device/Queue consumer、动态WGPU与性能证据保持开放；状态为`runtime90_pfo_4d4b_source_implemented_static_checks_passed_dynamic_validation_pending`。

### 13.12 Renderer bootstrap queue authority hard cut（2026-08-31，PFO-4d4c源码完成、未验收）

renderer bootstrap调用图复审确认，scene core取得raw queue后只有UI未读取透传和light-cookie atlas启动初始化两个下游；后者每次renderer构造分配4,194,304-byte全白CPU块并执行一次`Queue::write_texture`。这不是持久资源必需初始化：compiled light-cookie feature已经声明`cookie.atlas_build`写external atlas并先于mesh/deferred读取，shader在cookie slot为0时不采样；Unreal `LightFunctionAtlas.cpp`也由RDG运行时producer创建和写atlas，而非renderer bootstrap上传默认纹理。

[`90/2026-08-27-pfo-4d-native-resource-write-data-plane-plan.md`](90/2026-08-27-pfo-4d-native-resource-write-data-plane-plan.md)中的PFO-4d4c已删除启动全白payload、raw write、`COPY_DST`和initial-upload counter，并从scene/core/mesh/light-cookie及UI/text产品构造链删除queue。复审同时修复了direct renderer旧有语义缺口：direct light packing会携带cookie slot，却不执行compiled plugin executor；现在它只在cookie非空时把同一atlas rebuild录入已有scene encoder，位于mesh consumer和scene submit之前，并使用独立GPU pass timer，成功提交后才发布profile。该结构没有新增native submission。

静态source contract 14/14、精确rustfmt、scoped diff、结构预算和locked metadata通过；4 MiB启动payload与该产品构造链raw queue权限在源码层降为0。尚无受管Cargo、真实WGPU、PNG/RDC、startup/frame profile、VRAM或功耗数据，不能把静态删除量报告成实测性能提升；状态为`runtime90_pfo_4d4c_source_implemented_static_checks_passed_dynamic_validation_pending`。

### 13.13 Compiled graph pass queue authority hard cut（2026-08-31，PFO-4d4d源码完成、未验收）

compiled graph的raw queue调用图此前从stage context贯穿frame services、pass executor与GPU execution context，但真正读取只有UI的空参数和particle transparent的32-byte参数写入。仓内post-process/HZB/subsurface等pass早已把CPU写入汇总到pass-owned upload batch；Unreal `FRDGBuilder::QueueBufferUpload`同样在RDG执行前统一上传，Unity Graphics也通过command-buffer `SetBufferData`能力录制写入，而不是让feature取得native queue。

PFO-4d4d新增公开但受限的`RenderPassBufferUploadRecorder`：外部render plugin只能登记buffer/offset/immutable bytes，底层batch与提交权仍由runtime持有。particle transparent五层调用链改用该recorder，UI删除无行为参数；compiled stage context、frame services、graph-pass函数和product GPU context已没有raw queue。upload只在executor成功形成`RecordedGraphPass`且全图成功后进入frame resource packet，失败pass不再像旧`queue.write_buffer`一样泄漏未提交帧状态；frame packet仍先于唯一scene submit，不新增native submission。

failing-first合同从1/9转为10/10；随后发现Hybrid GI scene-depth handoff仍有`gpu.queue`旁路，已改为同一pass upload recorder并补executor级source contract。精确rustfmt、scoped diff、public re-export、结构预算和locked metadata通过。particle simulation prepare、compiled submit/readback中的真实queue consumer仍开放；没有受管Cargo、真实WGPU、PNG/RDC、profile或功耗数据，状态为`runtime90_pfo_4d4d_source_implemented_static_checks_passed_dynamic_validation_pending`。

### 13.14 RuntimePrepare upload transaction foundation（2026-08-31，PFO-4d4e源码完成、未验收）

RuntimePrepare 的 queue 写入分成三类真实 producer：particles simulation 在 dispatch 前写 emitter/counter，Hybrid GI prepare 写 completion/radiance-cache/global-SDF page-table，Virtual Geometry 维护独立 uploader params。直接删除queue会破坏dispatch前写入先行关系，因此本切片先建立共享事务输出，不机械迁移producer。

`RuntimePrepareCollectorContext`现提供受限`RuntimePrepareBufferUploadRecorder`，collector只登记buffer/offset/immutable bytes；每个collector的局部batch汇入RuntimePrepare总batch，错误时整包随结果丢弃。成功batch由`SceneRendererAdvancedPluginReadbacks`跨越prepare边界持有，只有graph prepare/materialization成功返回后才并入唯一frame upload packet；随后继续服从现有`FrameResourceUpload` ticket早于scene submission的顺序，没有新增native submission或第二条queue owner。

静态合同11/11、精确rustfmt、scoped diff check和locked metadata通过。particles、Hybrid GI与Virtual Geometry producer仍使用原queue路径，compiled submit/readback queue仍归completion/readback owner；managed Cargo因既有`cargo_reuse_target_mismatch`未重试，真实WGPU、PNG/RDC、profile、VRAM与功耗证据为空。状态为`runtime90_pfo_4d4e_upload_transaction_foundation_source_implemented_static_checks_passed_producer_migration_and_dynamic_validation_pending`。
