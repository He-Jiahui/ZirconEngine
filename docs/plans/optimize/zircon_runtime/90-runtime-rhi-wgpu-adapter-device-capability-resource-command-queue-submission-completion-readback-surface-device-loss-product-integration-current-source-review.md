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
implementation_status: not_started
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

本篇是review和重构输入，不是implementation acceptance record。`review_status: complete`只表示上述冻结语料、差异、owner、里程碑和门禁已形成；`implementation_status`保持`not_started`。完成Runtime90实现至少要求M0至M9按顺序落地、48个资格门全部有可复核证据、产品raw WGPU路径完成hard cut，并由Runtime89的device-qualified graph packet接入同一SubmissionService。

本轮没有修改Rust/Cargo，没有运行Cargo、真实GPU、RenderDoc、device loss、multi-queue、fault、soak或benchmark。后续实现前必须再次冻结共享working tree并重查本篇P0证据，尤其是当前已有其他会话改动的`gpu_pass_timer.rs`。tooling继续按用户要求排除。
