# Product diagnostic readback owner hard cut and performance plan

## 状态

- 日期：2026-08-27
- 范围：Runtime90 M8 / PFO-1 diagnostic integration / PFO-2 device owner replacement
- 当前状态：`d0_metrics_complete_d1_control_owner_complete_d2_diagnostics_complete_d2q_query_and_ibl_complete_d3_sync_capture_complete_pfo_4a_shared_present_complete_pfo_4b_external_copy_central_owner_and_failure_identity_static_checks_passed_submission_fusion_native_data_plane_dynamic_profile_pending`
- 证据边界：本记录来自源码与仓库内 Unreal Engine 参考实现重审。未运行 Cargo、真实 WGPU、profiler、RenderDoc、截图、功耗或 benchmark；不得把本记录当作性能验收结果。

## 结论先行

产品诊断读回不得继续扩展 `zr_rhi_wgpu::GpuReadbackQueue`。唯一目标 owner 是已经存在的：

```text
zr_rhi::DiagnosticReadbackTracker
  -> zr_rhi_wgpu::production::WgpuDiagnosticReadbackService
     -> WgpuRenderDevice::poll_submissions
     -> one submission-qualified terminal/delivery stream
```

旧队列同时拥有 cloned `wgpu::Device`、三槽 staging ring、独立 request ticket、预算、map completion、callback 与主动 `device.poll`。它与 production service 形成第二套 device-local completion world；把二者桥接或互相轮询只会固化重复所有权，不能作为迁移方案。

PFO-2 的硬切方向是产品渲染器接入 `WgpuRenderDevice`，由同一 submission/completion owner 同时推进 scene、query、readback、surface 与 resource retirement。旧 raw-WGPU pass 在迁移期只能使用 transaction-scoped native recorder lease：可录制但不可取得、克隆或保存 queue，且 command buffer 必须归还同 generation 的 submission service。

## 当前源码事实

### 已有正确基础

1. `DiagnosticReadbackTracker`以`DeviceId + DeviceGeneration + SubmissionTicket`形成`DiagnosticFrameKey`，并在 native work 录制前执行 request/frame/pending count 与 byte budget。
2. `WgpuDiagnosticReadbackService`持有 active batch、submission-keyed in-flight batch、ticket-ordered completion、bounded delivery ring 与 query service；native WGPU 对象不会穿过 delivery 边界。
3. `WgpuRenderDevice::poll_submissions`在唯一 device poll 后调用`collect_completed_maps`；fault/cancel/shutdown通过同一 diagnostics owner terminalize。
4. production query resolve 与产品场景 texture/buffer readback 已进入普通 scene submission packet；独立`submit_diagnostic_readback_frame`只保留为尚未硬切的显式/兼容入口，不再是 compiled/direct scene 的诊断提交路径。
5. 当前 request-qualified delivery API 只在目标 request 位于 delivery 队首时消费。该行为能防止误消费，却不能支撑多个产品消费者独立拉取；产品接入时必须有一个中央 delivery router 顺序 drain，再按 request owner 投递，不能让 feature owner 直接争抢队列。

### 遗留产品 owner

2026-08-27 D1 更新：产品 offscreen 构造现在创建一个`WgpuRenderDevice`，`RenderBackend`不再另建`WgpuSubmissionCoordinator`、`WgpuDeviceErrorSupervisor`、immutable profile或UI registry。现有上传、原生命令提交、poll、status、失败批量结算、metrics、fault gate和UI surface context均委托给该device owner。控制面因此只剩一份submission history/fault callback/profile/diagnostics owner。

这还不是完整device hard cut。`RenderBackend`仍保留同一generation的typed `Instance/Adapter/Device/Queue` clone，供尚未neutral lowering的旧Pass、原生资源创建、queue write和surface路径使用；这些clone没有第二份submission history或fault callback，但仍是PFO-4必须删除的数据面逃逸口。产品同步截图helper已经退出产品编译面；共享WGPU UI surface不创建旧readback queue，standalone surface虽暂时保留旧timer/readback staging数据结构，但已删除其独立submit/poll authority。

`SceneRendererCore`已经删除`GpuReadbackQueue/readback_frame_index`。compiled/direct scene 的GPU timer、pipeline statistics、viewport、HZB/indirect、advanced plugin、realtime IBL timestamp与IBL artifact都进入production diagnostic/query service，并通过共享`poll_frame_submission_completions`执行`唯一backend poll -> IBL callback/cache -> typed query route -> timer/statistics collect`。共享WGPU UI context由runtime推进completion；standalone context拥有自己的`WgpuRenderDevice`并在frame entry唯一poll。旧UI timer/readback staging只在owner poll后collect，不再拥有第二条completion timeline。

同步 helper 的库存必须区分源码文件与产品编译面：

| 类别 | owner / 路径 | 产品 native submit | 产品 blocking wait | 备注 |
| --- | --- | ---: | ---: | --- |
| 测试隔离 | `read_buffer_bytes.rs` | 0 | 0 | buffer/f32x4 array/SH9同步诊断只供`#[cfg(test)]` |
| 测试隔离 | `read_texture_rgba.rs` | 0 | 0 | 旧RGBA8 copy-only helper整模块仅供`#[cfg(test)]` |
| 测试隔离 | `read_texture_rgba16float_region.rs` | 0 | 0 | 旧cube mip/region copy-only helper整模块仅供`#[cfg(test)]` |
| 测试隔离 | `read_ibl_bake_artifact_sections/batch.rs` | 0 | 0 | 产品路径改用中央router的CPU-only pending section聚合；同步batch仅供`#[cfg(test)]` |
| 测试隔离 | `read_buffer_f32x4.rs` | 0 | 0 | 模块级`#[cfg(test)]` |
| 测试隔离 | `read_texture_rgba16float_3d.rs` | 0 | 0 | 模块级`#[cfg(test)]` |

因此当前产品编译面的同步helper基线是0个owner文件、0个direct native submit、0个blocking wait；源码仍保留测试工具，但不计入产品编译面。同步RGBA8/HDR外观改为向production diagnostic service登记有界请求，由唯一submission service提交显式diagnostic packet，再在30秒上限内用唯一completion pump等待自己的callback；没有raw `queue.submit`、`device.poll`或无限`recv`。graphics产品树中的裸`wait_indefinitely`只剩1个明确批准的RenderDoc stop工具边界；`zr_rhi_wgpu::ui_surface::retained_cache`中的同名调用位于`#[cfg(test)]`模块，不是生产等待。以上都是静态调用面，不是profiler结论。

同步 framework capture 先`finish_submission`，再等待显式capture边界，随后优先消费production router已经完成的async capture，否则调用同步RGBA/HDR fallback。非阻塞`poll_captured_frame_if_newer`及其callback mailbox已使用production diagnostic budget；旧队列仅留在兼容同步外观，不能重新成为普通产品帧owner。

## Unreal 参考边界

仓库内 Unreal Engine 源码给出的适用行为是 owner 顺序，而不是照搬 D3D12 实现：

- `RHIGPUReadback.h/.cpp`的`FRHIGPUMemoryReadback`把 staging resource 与 GPU fence 绑定；`EnqueueCopy`把 copy 与 fence写入传入的`FRHICommandList`，不会由 feature helper自行创建并提交第二条 queue timeline。`IsReady`轮询 fence；显式`Wait`是调用者选择的同步边界。
- `DynamicRHI.h`要求 command context先 finalize，再由`RHISubmitCommandLists`有序批量提交；product feature不拥有 queue submission authority。
- `DynamicRHI.cpp::RHIEndFrame_RenderThread`在 end-frame owner dispatch，而不是每个 feature自行 flush。
- Lumen 参考工程约束 scene graph 的 PreZ/HZB/lighting/temporal/tone-map 顺序，不赋予读回、query或 pass 独立 queue owner。

Zircon 应复刻“copy intent进入 frame command stream、completion由 submission-qualified identity观察”的行为。不得复制 Unreal 的 multi-GPU array、RHI thread或D3D12 fence细节来伪造 WGPU 能力。

## 目标所有权

```text
WgpuRenderFrameworkCore
  -> ProductRenderDeviceOwner(WgpuRenderDevice)
     -> frame-begin poll exactly once
     -> RenderFrameSubmissionTransaction
        -> scene/query/readback intents
        -> finalized packet(s)
        -> SubmissionTicket(s)
     -> WgpuDiagnosticReadbackService
        -> request admission + bounded staging
        -> map start after submitted identity exists
        -> terminal receipt + optional bytes
     -> ProductDiagnosticDeliveryRouter
        -> viewport capture mailbox
        -> timing/statistics collector
        -> HZB/indirect diagnostics
        -> IBL artifact writeback
```

`ProductDiagnosticDeliveryRouter`是 CPU delivery 的唯一消费者。它按 service 顺序 drain，并用受profile上限约束的`HashMap + VecDeque` request-owner索引路由；feature owner只持有 request ID/typed destination，不持有 callback queue、native buffer、device或poll authority。慢 callback 不在 render submit/poll锁内执行。

## 分层实施

### D0 口径与合同

1. 固化产品/测试 readback owner库存与 direct-submit/blocking-wait基线。
2. 为`RenderFrameSubmissionTransaction`定义 diagnostic intent/binding 合同：request在录制前 admission，只有真实 submission ticket出现后才发布 frame key；失败帧必须得到 Cancelled/MapFailed/DeviceLost/Shutdown之一。
3. 禁止新增普通产品`GpuReadbackQueue` owner、直接`device.poll`、readback helper `queue.submit`与无归属 callback。

### D1 WgpuRenderDevice 过渡录制边界

1. 产品构造只产生一个`WgpuRenderDevice`及其 fault/profile/submission/diagnostic/surface owner。
2. 在`zr_rhi_wgpu`内实现 transaction-scoped native recorder lease。lease借用 device录制旧 pass命令，但不暴露 queue；drop时未归还的 command buffer/intents必须 fail closed。
3. lease归还 finalized native command buffers、resource use与diagnostic intents；submission service分配唯一 ticket并提交。
4. raw queue write路径按 upload batch迁移，不以公开 queue clone保留兼容层。

### D2 产品 diagnostic 接入

1. 先迁移 viewport async capture，因为它已有 request/admission/mailbox，能形成最小闭环。
2. 再迁移 GPU timer/pipeline statistics，优先复用 production query service，不把 timestamp payload转成通用 byte callback。
3. 迁移 HZB、indirect draw、advanced plugin 与 realtime IBL readback；按用途区分控制反馈和纯诊断。影响下一帧决策的反馈必须有最大陈旧帧数与 unavailable fallback。
4. IBL artifact采用现有 pending section聚合语义，但 staging/map/completion归入 production service；不得恢复同步 batch作为常规路径。
5. 中央 router投递后再运行用户 callback或写 mailbox，render poll只移动有界结果。

#### D2-Q typed query 结构重审（2026-08-27）

当前瓶颈是 owner 时序不兼容，不是时间戳差值或五计数器解码算法：

1. 旧`GpuPassTimer/GpuPipelineStatisticsTimer`在renderer构造时长期持有query set与resolve buffer，以旧三槽readback frame准入录制；frame tail再把resolve buffer作为通用bytes请求。它没有submission-qualified typed terminal，且旧queue主动poll。
2. production `DiagnosticQueryPlan/WgpuDiagnosticQueryService`已经用dense pass ID、bounded scope、`O(N)`聚合和typed `Unavailable/OverBudget/MapFailed/DeviceLost` delivery实现目标结果模型；但当前只在neutral packet取得ticket后创建query set，无法供尚未neutral lowering的raw WGPU Pass在录制前写query。
3. copy readback和query目前共用一个`DiagnosticReadbackTracker.active_frame`。产品viewport/plugin/HZB/IBL scope会跨runtime-prepare与graph recording保持active，因此在同帧再调用现有query `prepare_frame`必然产生`FrameAlreadyActive`。这不是锁优化可以解决的问题。
4. Unreal `FRDGTimingFrame/FRDGTimingPool`提供适用参照：作用域从有界query pool分配，begin/end写入原command list，帧终保留RHI fence并在后续帧非阻塞gather；`RHIGetRenderQueryResult(..., bWait)`只有调用者显式选择等待。Zircon复刻pool/frame/fence owner顺序，不复制RHI thread或平台query实现。

实施顺序：

1. 在同一个`WgpuDiagnosticReadbackService`锁与同一个device poll owner内，为typed query建立独立的内部lifecycle tracker，使query reservation能与buffer/texture active batch并存；它不是第二个queue、poll、submission或对外service。
2. 新增generation-qualified native query recorder reservation。reservation在任何query write之前按profile最大timestamp/statistics scope预占有界bytes；失败立即形成typed terminal并跳过本帧query录制。recorder只暴露query set引用，不暴露queue、poll或flush。
3. frame tail用实际`DiagnosticQueryPlan`校验已录制scope，resolve/copy并入scene serial tail，生成opaque native query frame；scene ticket分配后由query service绑定并进入现有typed map/aggregate/delivery状态机。
4. 产品query router按frame generation登记一次typed sink，把dense pass result映射回compiler-owned label，并分别发布GPU timing与pipeline statistics DTO；callback/label不进入WGPU service。
5. 完成compiled/direct接入后删除旧timer resolve buffer、`request_readback_external`和readback-ready耦合。旧`GpuReadbackQueue`的主动poll只可在无剩余consumer时整体删除。

2026-08-27 D2-Q implementation update：production query service now owns a private query lifecycle tracker so it can coexist with the cross-runtime-prepare product copy tracker while sharing the same diagnostics mutex, submission status source, device poll, and terminal pump. `WgpuNativeDiagnosticQueryRecorder` reserves generation-qualified bounded query sets before raw native passes record; `prepare_native_diagnostic_query_frame` validates the actual neutral `DiagnosticQueryPlan`, appends resolve/copy commands to the scene encoder tail, and returns an opaque frame. `enqueue_native_recording_packet_with_frame_diagnostics` binds that frame, plus any native copy frame, to the one scene `SubmissionTicket` before commit. Unbound recorder/frame cancellation produces typed terminal delivery. This remains infrastructure-only: legacy timers/statistics still need lowering to the recorder and old external readback paths are not yet deleted.

复杂度/性能约束：diagnostics-off不创建reservation/query set；query-on frame的scope登记与聚合均为`O(S + P)`，其中`S`为实际physical scopes、`P`为logical pass数，且上限来自profile。迁移期可按profile上限预留最多`64 * 2 * 8 = 1024` timestamp bytes与`64 * 5 * 8 = 2560` statistics bytes；不得为每个scope单独map、submit或poll。当前没有profiler数据，因此本切片只修复owner与有界算法，不做query-set复用、staging池或索引微优化；这些只能在300帧三次采样后决定。

### D3 同步外观收敛

1. `capture_frame`和HDR capture先请求自己的 bounded diagnostic request，再只等待该 request的终态；不得执行全设备`wait_indefinitely`。
2. 同步外观必须有 timeout、device-loss、over-budget与shutdown terminal result，并保留 frame/submission identity。
3. screenshot/pixel-test可使用同步外观，但不得绕过 product diagnostics另建 queue submit。
4. debugger stop作为显式工具边界单独登记；是否保留 blocking wait由RenderDoc真实验证决定，不计入普通产品帧热路径。

### D4 删除旧链

1. 删除`SceneRendererCore::readback_queue/readback_frame_index`与`GpuReadbackQueue`公开导出。
2. 删除旧`ReadbackTicket/ReadbackCallback/ReadbackPollStats`及 framework 对`FRAME_SLOTS`的耦合。
3. 产品同步helper的direct native submit/blocking wait已清零；测试专用helper保持模块级`#[cfg(test)]`。
4. source guard要求普通产品只有 production submission service拥有 native submit/device poll；批准的test/debugger例外必须显式列出。

## 算法与复杂度目标

- admission：每个 request `O(1)`；active batch只追加 dense request与对齐后的 staging offset。
- record/encode：`O(R)`，其中`R`为本帧 admitted requests；同一诊断帧只分配一个 staging buffer和一个 copy command stream。
- completion poll：`O(B + C)`，其中`B`为有界 in-flight batches，`C`为本次完成 requests；不得扫描 renderer resource/material全表。
- delivery route：`O(C)`；request ID到owner使用有界 dense/slab索引或哈希表，不能为每个feature线性扫描全部delivery。
- retained memory：`O(max_pending_bytes + max_completed_bytes)`，且两者都来自 profile budget；不得由 callback消费速度形成无界增长。
- steady diagnostics-off frame：零 staging allocation、零 callback allocation、零额外 native submission、零额外 device poll。
- capture-on frame：在PFO-3融合前允许一个可观测copy packet；最终目标是copy intent并入拥有source最后写入的frame packet，不固定增加submission/frame。

当前 production service每次`collect_completed_maps`会分别扫描 in-flight batch以启动map和收割callback，复杂度已经有界但常数需实测。没有 profiler 证据前不改成事件索引、bitset或额外队列；这些结构可能增加维护成本而不减少真实瓶颈。

## 性能测量计划

所有数据必须在同一机器、adapter、driver、device profile、分辨率、场景、画质、present模式和电源策略下采集。每个场景预热60帧，采集300帧，重复3次，并记录中位数与最差一次。

测试矩阵：

1. diagnostics off。
2. viewport capture 1 Hz。
3. viewport capture every frame。
4. timestamps/statistics enabled，capture off。
5. HZB/indirect/IBL各自单开与组合开启。

必须采集：

- CPU frame、record、admission、encode、submission、poll、map collection、delivery route的p50/p95/p99与最大值。
- GPU frame与diagnostic copy/resolve时长；request submission到terminal delivery的帧延迟分布。
- native submission count/frame、device poll count/frame、render-thread blocked ms/frame。
- admitted/rejected/completed/dropped request count与bytes；active/in-flight/delivery高水位；staging分配/复用/增长次数。
- process CPU、GPU utilization、显存/系统内存与整机/GPU功耗；空闲、diagnostics-off和capture-on同时对比。
- RenderDoc capture、PNG截图、profile JSON必须记录同一`DeviceGeneration`、CPU frame generation与scene/readback/present ticket，防止跨帧拼接证据。

性能验收门槛只能在采集基线后落数值。结构门槛先固定为：diagnostics-off不增加submission或poll；普通产品帧无blocking wait；稳定present最终一个native submission；所有队列和bytes有profile上限；复杂度达到上述规模。

## 风险与失败策略

- 若 native resource仍未迁入 neutral registry，readback source无法安全交给 production service。此时继续迁移其他neutral owner，不增加旧队列能力；缺口记录为PFO-2 recorder/resource hard-cut依赖。
- 若某控制反馈必须下一帧可用，over-budget或延迟时使用显式 conservative fallback，不能阻塞整个device等待。
- 若多个消费者导致队首阻塞，修复中央router，不放宽`take_delivery_for`去越序偷取别人的delivery。
- 若动态验证通道不可用，继续完成 owner、contract、source guard与迁移代码，但状态保持source-only，不生成伪截图、伪RDC或伪性能数据。

## 完成定义

本计划只有在以下全部满足后才可标记完成：

1. 产品构造与frame poll只存在一个`WgpuRenderDevice` owner。
2. 产品scene旧`GpuReadbackQueue`与产品direct-submit同步读回路径已删除；全crate旧queue只有在WGPU UI surface/兼容测试也完成迁移后才算删除。
3. async/sync viewport capture、query、HZB/indirect、IBL全部通过submission-qualified diagnostic service与中央router。
4. diagnostics-off/on性能矩阵、RenderDoc、真实PNG与功耗数据齐全，且证据身份一致。
5. 状态写回render计划；managed验证通过；里程碑accepted后才提交Git、同步协调器并发送企微量化结果。

## D0 实施结果（2026-08-27）

状态：`source_implemented_static_checks_passed_cargo_wgpu_profile_pending`，不是PFO-2完成或Runtime90 M8 accepted。

已完成：

1. production diagnostic service新增generation-local `WgpuDiagnosticReadbackMetricsSnapshot/Delta`。快照包含admitted/rejected request与bytes、submitted/map/terminal/succeeded/drained/dropped单调计数，active/in-flight/retained当前gauge，以及in-flight batch/request/bytes和retained delivery count/bytes生命周期高水位。
2. `delta_since`只接受同一`DeviceId + DeviceGeneration`且所有单调字段未回退的样本；当前gauge取后一个样本，高水位保持lifetime语义。读者不能reset共享窗口。
3. metrics更新均附着于已有admission、bind、map callback、terminal、delivery drain与eviction路径，单次为`O(1)`，没有新增native submit、device poll、callback、资源表扫描或diagnostics-off allocation。
4. `WgpuRenderDevice::diagnostic_readback_metrics`公开只读快照。`append_diagnostic_readback_deliveries`在一次diagnostics锁内按ticket顺序把所有delivery移动到调用方复用的`Vec`，payload不复制，为后续`ProductDiagnosticDeliveryRouter`提供唯一顺序drain边界；既有单项与request-qualified接口暂留兼容。
5. 新增3项源码内合同：foreign generation拒绝区间差分、单调计数与当前gauge/高水位分离、批量delivery保持顺序并释放retained bytes且更新drained metrics。

D0当时的静态证据：8个精确Rust文件`rustfmt --edition 2021 --check`通过；source contract确认production service不包含`device.poll`或`queue.submit`，metrics/API/export链完整；当时产品同步readback基线为5 submit/5 wait、test-only基线为2 submit/2 wait；scoped `git diff --check`通过。当前基线见上方库存表与D3结果。未运行Cargo、真实WGPU、profiler、RenderDoc、PNG或功耗验证。

下一步：D1先定义并实现`WgpuRenderDevice` transaction-scoped native recorder/packet归还合同，再让产品构造从重复fault/submission owner切到唯一device owner。不得为了快速接入把raw queue clone暴露给产品renderer。

## D1 控制面 owner 切换结果（2026-08-27）

状态：`source_implemented_static_checks_passed_native_data_plane_and_product_diagnostics_pending`，不是PFO-2完成或Runtime90 M8 accepted。

已完成：

1. `WgpuNativeRecorderLease`借用owner device录制或接收现有并行recorder产生的command buffer；finished packet携带`DeviceId + DeviceGeneration + RenderQueueClass`，空packet与foreign generation均fail closed。lease与packet不公开queue、poll或flush。
2. `WgpuRenderDevice`新增迁移期native buffer/texture upload batch入口。它们复用唯一`WgpuSubmissionService`的admission、staging预算、ticket、commit和cancel路径，不创建第二个coordinator；产品producer仍没有flush authority。
3. `RenderBackend::new_offscreen_with_policy`把同一native context与profile交给一个`WgpuRenderDevice`。重复的`WgpuSubmissionCoordinator`、`WgpuDeviceErrorSupervisor`、profile字段和UI registry字段已删除；已无调用方的coordinator类型及crate公开导出也已硬删除。UI context、fault gate、submission/poll/status/settlement/metrics均由device owner提供。
4. 现有`RenderBackend`提交方法保持调用面稳定，内部将command buffer封装为generation-qualified native packet；immediate submit仍由该device owner执行原有flush，enqueue路径仍等待frame owner flush。失败结算继续在一次submission state锁下保持`O(P + T)`。
5. 静态证据：精确Rust文件`rustfmt --edition 2021 --check`通过，scoped `git diff --check`通过；全`zr_rhi_wgpu`源码已无`WgpuSubmissionCoordinator`符号，产品backend目录已无`submission_coordinator`、`device_error_supervisor`或独立`ui_image_registry`字段/构造；迁移桥生产段无`wgpu::Queue`、`device.poll`或`queue.submit`；新增native submission模块86行，未形成大文件。

未完成与下一步：

- 旧Pass仍直接使用`RenderBackend.device/queue`创建资源与执行queue write，surface也未切入`WgpuSurfaceService`；所以这里只完成控制面单owner，不是native数据面硬切。
- `SceneRendererCore::GpuReadbackQueue`仍有独立device poll/map/callback世界。D2先实现中央`ProductDiagnosticDeliveryRouter`并迁移viewport async capture，再迁移query/HZB/indirect/IBL。
- 未运行Cargo、真实WGPU、profiler、RenderDoc、PNG、功耗或benchmark；性能瓶颈是否消失尚无动态证据。

## D2 viewport async capture 与 compiled buffer consumer 闭环（2026-08-27）

状态：`source_implemented_static_checks_passed_typed_query_ibl_artifact_complete_sync_helpers_dynamic_validation_pending`，不是D2整体完成或Runtime90 M8 accepted。

已完成：

1. production diagnostic source新增迁移期native RGBA8 texture lease，仅接受`Rgba8Unorm/Rgba8UnormSrgb + COPY_SRC + single-sample D2`且extent在source范围内。texture clone随bounded batch保留到submission-qualified map终态，不把native texture交给callback/router。
2. viewport capture在旧Pass全部写完后，将copy编码到`FrameCommandEncoderSet`最后一个serial suffix；output-target writeback先完成，diagnostic tail随后编码。opaque `WgpuNativeDiagnosticReadbackFrame`与scene command buffers一起进入`submit_native_recording_packet_with_diagnostics`，在一个scene ticket、一次flush和同一native submit中完成，不增加copy-only submission。
3. 新增有界`ProductDiagnosticDeliveryRouter`。request ID保存在`HashMap + VecDeque`，上限为profile的`max_pending_requests + max_completed_receipts`；查找/删除均摊`O(1)`，顺序索引超过两倍预算时压缩。frame-begin唯一device poll后一次锁移动全部delivery，释放router/device锁后再运行callback，payload ownership移动不复制，callback panic被隔离并计数。
4. viewport capture拒绝不再向mailbox发布admitted状态；成功request的callback只由中央router投递。viewport mailbox/profile retention不再依赖旧`GpuReadbackQueue::FRAME_SLOTS`，改用production diagnostic budget的pending-request边界。
5. 所有begin/request/writeback/prepare错误出口都会取消production diagnostic active frame并继续清理旧readback/realtime-IBL/transient frame；packet validation、ticket admission、diagnostic bind或commit失败同样terminalize/cancel，不遗留到下一次flush。
6. production source新增native buffer range lease，只接受`COPY_SRC`、非零且WGPU copy-aligned并位于source size内的范围。compiled-scene HZB stats、最多4个phase的indirect args/draw-count与realtime IBL timestamp resolve现在向同一active batch登记；一个staging buffer、一个serial-tail copy stream和scene ticket覆盖这些buffer与viewport texture请求。
7. HZB仍保留4帧本地诊断上限和`source_frame_index` FIFO。production frame不可用、request over-budget或callback失败时走显式drop/fail路径，不阻塞渲染等待GPU；indirect部分失败使该帧summary不可用，而不把不完整参数解释为有效控制反馈。realtime IBL只在已有pending submission/timestamp source时登记。
8. direct-scene realtime IBL timestamp也在旧Pass录制完成后打开production diagnostic frame，先登记timestamp resolve source，再编码旧typed timer copy、output writeback和production diagnostic tail，最后通过`submit_graphics_command_buffers_with_diagnostics`与唯一scene ticket一起提交。begin、admission或prepare失败只关闭纯诊断；scene提交前的shadow、mesh、overlay、UI、copy、writeback及submit失败均释放realtime IBL scheduler batch，避免pending状态泄漏。
9. advanced plugin runtime-prepare readback不再向旧`GpuReadbackQueue::request_readback_external`登记。collector通过显式`requests_gpu_readback()`元数据声明需求；compiled frame仅在存在该类collector时于runtime-prepare前打开RAII product diagnostic scope，任何提前返回都会自动取消active batch。viewport、plugin、HZB、IBL按固定优先级登记，同一serial tail和scene ticket完成复制与终态投递。
10. Hybrid GI继续用语义化的三帧本地在途上限约束主反馈和Global SDF反馈；粒子新增同样的三帧逻辑批次上限，并在任何新compute dispatch前检查。production全局预算仍负责跨consumer request/byte admission，两层上限分别约束feature陈旧度和device总内存，不再把feature语义耦合到旧staging ring常量。native callback直接移动`Vec<u8>`到future completion，不再执行一次payload克隆。

算法与热路径：diagnostics-off只有空router drain检查，无staging、callback、native copy或submission；没有readback能力声明的runtime-prepare collector不会触发scope reservation。capture/plugin-on admission/route为`O(1)`查找加`O(C)`完成投递，pending map和delivery bytes均有profile上限。当前每个非空diagnostic batch仍分配一个production staging buffer，是否需要复用必须由profile证明，不能在无数据时引入另一套slot owner。

静态证据：精确Rust文件`rustfmt --edition 2021 --check`通过；旧viewport `request_texture_rgba`调用为零；viewport mailbox旧`FRAME_SLOTS`耦合为零；compiled HZB/indirect和advanced plugin生产段旧`request_readback_external`调用为零；realtime IBL旧`request_gpu_timestamp_readback`生产调用为零。compiled顺序索引为`scope 19955 -> runtime prepare 20728 -> viewport 29740 -> plugin 30979 -> HZB 31731 -> IBL 32790 -> legacy query copies 33779 -> writeback 34888 -> diagnostic prepare 36017 -> scene submit 36932`；direct顺序仍为`begin -> IBL -> legacy timer copies -> output writeback -> diagnostic prepare -> scene submit`。scene-qualified`bind_batch(ticket, frame.batch, frame.staging)`、native buffer/texture prepare无ticket/queue/poll、router bounded/move-only合同通过；相关Hybrid GI/particle/VG readback源均静态确认包含`COPY_SRC`。未运行Cargo、真实WGPU、PNG、RenderDoc、profiler、功耗或benchmark。

下一步：共享WGPU UI present和external-image copy都已接入唯一production submission owner；copy typed result、frame ticket和已提交scene失败身份已闭合。继续把copy目标预分配并合并到scene serial tail，随后为共享surface计时接入typed query router、完成raw resource/write数据面硬切，再执行同generation动态、性能、截图、RenderDoc与功耗验收。

## D2-Q typed timer/statistics 与 IBL artifact 闭环（2026-08-27）

状态：`source_implemented_static_checks_passed_cargo_wgpu_profile_pending`，不是PFO-3完成或Runtime90 M8 accepted。

已完成：

1. `GpuPassTimer`和`GpuPipelineStatisticsTimer`的产品构造改为预留`ProductDiagnosticQueryFrameScope`中的typed ranges。raw pass只接触generation-qualified recorder，scene尾部按实际plan统一resolve/copy；query frame与buffer/texture diagnostic frame一起绑定唯一scene `SubmissionTicket`。下一帧唯一backend poll先路由typed delivery，再由timer/statistics collector消费，不再从产品scene登记旧external byte readback。
2. IBL PMREM每个face/mip、SH9 buffer和IEM每个face在资源释放前向同一product diagnostic scope登记。新增native `Rgba16Float` texture source校验D2、single-sample、format、`COPY_SRC`、mip/layer、extent及8-byte texel布局；copy仍进入scene serial tail，不创建artifact专属command buffer、submit、map callback queue或device poll。
3. `IblBakeArtifactWgpuPendingReadback`只持有有界CPU slots和首个失败原因。中央router callback按face-major/mip-major顺序组装section；任何拒绝、终态错误、长度不符或重复delivery都使整个artifact失败，不发布部分缓存。成功scene submit后才把reservation转入pending queue；下一帧唯一completion poll已分发全部callback后再写`.zribl`缓存。
4. router满载不再静默驱逐最老callback。新注册显式失败并保留已有request owner，IBL调用方立即终结对应slot，从而消除`remaining > 0`永久悬挂风险。router查找/删除保持均摊`O(1)`，顺序索引仍按既有两倍预算阈值压缩。
5. direct与compiled-pipeline两条帧入口统一委托`SceneRenderer::poll_frame_submission_completions`。此前compiled入口仍引用已经删除的`self.core.readback_queue`且漏掉typed query route；共享owner消除该编译/行为漂移，并保证两条入口都只消费一次真实`SubmissionPollReceipt`。

算法与热路径：query-off不创建query set或resolve buffer，IBL artifact只在runtime bake reservation存在时登记；普通帧没有PMREM/SH9/IEM staging开销。开启artifact readback时request数为`6 * mip_count + SH9?1:0 + IEM?6:0`，由既有request/byte budget在录制前限制，完成组装为`O(total payload bytes)`且每个payload只move一次。是否池化query/staging资源必须由300帧profile证明，本切片不在无数据时引入新pool owner。

静态证据：本切片Rust owner完成`rustfmt --edition 2021`解析；scoped `git diff --check`通过（仅LF/CRLF提示）；compiled scene顺序合同确认`artifact registration -> output writeback -> diagnostic tail -> query tail -> scene submit`，提交合同确认`scene submit -> IBL ownership commit -> transient retirement -> post-submit error`，共享frame-begin生产段只有一次completion poll且先分发IBL callback、typed query再collect，两条调用入口均无直接poll或旧queue访问。CPU aggregator不含`wgpu::Buffer`、`map_async`、`device.poll`或`queue.submit`；native diagnostic prepare不含submit/poll。未运行Cargo、真实WGPU、PNG、RenderDoc、profiler、功耗或benchmark。

## D3 同步截图外观收敛（2026-08-27）

状态：`source_implemented_static_checks_passed_cargo_wgpu_png_profile_pending`，不是PFO-3完成或Runtime90 M8 accepted。

1. `finish_viewport_frame`、latest-frame fallback与HDR scene-color capture不再调用自有staging helper。RGBA8/RGBA16Float均先通过production source校验与readback budget，再把copy编码为opaque diagnostic frame并交给唯一`WgpuRenderDevice` submission timeline。
2. 同步外观只等待自己的容量1 callback mailbox；每次循环调用唯一`poll_submission_completions`，callback由中央router锁外执行，结果用`try_recv`提取。统一30秒上限返回typed `GraphicsError::DiagnosticReadbackTimedOut`，不使用无限`recv`或`wait_indefinitely`。
3. framework在消费已有async capture前使用同一有界drain；收敛条件是production metrics的active、in-flight和retained request gauge全部归零。普通frame-begin仍只有一次非阻塞completion poll，只有显式capture API进入有界等待循环。
4. 旧`read_buffer_bytes`、`read_texture_rgba`和`read_texture_rgba16float_region`整模块只在测试构建中导出。产品同步helper静态基线从3个文件/4次direct submit/4次blocking wait降为0；graphics产品裸`wait_indefinitely`只剩RenderDoc stop的1处批准例外。

算法与错误边界：callback路由和mailbox写入均为`O(1)`，payload只move一次；显式capture增加一条copy diagnostic submission，但它通过唯一submission owner并携带generation/ticket/terminal身份，不会成为普通产品帧的额外提交。admission拒绝、submit失败、device fault、callback断开和timeout均显式返回，router满载仍保留旧owner而拒绝新请求。

静态证据：相关Rust文件通过`rustfmt --edition 2021`解析；源码合同确认两种格式都走product diagnostic owner、旧helper整模块`#[cfg(test)]`、同步callback使用`try_recv`、等待循环只有唯一completion pump且无`wait_indefinitely`/raw device poll。未运行Cargo、真实WGPU、PNG、RenderDoc、profiler、功耗或benchmark。

## PFO-3 UI Surface共享completion owner隔离（2026-08-27）

状态：`source_implemented_static_checks_passed_cargo_wgpu_surface_profile_pending`，不是PFO-3完成或Runtime90 M8 accepted。

1. `WgpuUiSurfaceContext`现在显式记录是否拥有completion timeline。由`WgpuRenderDevice::ui_surface_context()`产生的公共共享context固定为false；只有`WgpuUiSurfaceRenderer::new_owned()`自己申请adapter/device后，才通过私有构造声明true。
2. UI renderer的旧`GpuReadbackQueue`改为可选owner。共享context即使`allow_gpu_timing=true`也不会创建timer/readback queue，不调用其`poll_completed`，并如实报告`gpu_timestamp_supported=false`；独占device surface继续保留兼容计时，不影响其本地device的completion所有权。
3. 该切片消除了共享产品device上的第二个非阻塞completion推进者。后续PFO-4a已迁移共享UI present，PFO-4b已把产品external-image copy融合进scene packet；共享UI GPU timing仍须在同一surface submission接入typed query reservation后恢复。

算法与热路径：共享surface关闭旧timer后，每帧减少一次旧queue poll检查，且不创建timer query/readback资源；普通UI绘制和present提交数不变。独占surface行为保持兼容。真实性能、功耗和present时延必须由真实窗口profile验证，本切片只有源码owner合同与格式证据。

## PFO-4a 共享UI present中央提交（2026-08-27）

状态：`source_implemented_static_checks_passed_cargo_wgpu_surface_profile_pending`，不是PFO-4完成或Runtime90 M8 accepted。

1. 产品`RenderBackend`改为持有`Arc<WgpuRenderDevice>`，`ui_surface_context(self: &Arc<Self>)`把同一个完整owner保留到长期UI presenter；没有新增可克隆的半功能submission service或第二份history/fault/diagnostic状态。
2. 共享surface present把录制完成的command buffer交回`WgpuNativeRecorderLease`并通过`submit_native_recording_packet`提交。共享present生产段raw `self.queue.submit`降为0；独占自有device surface仍保留兼容raw提交。
3. `UiSurfacePresentStats`新增可选真实`SubmissionTicket`。共享提交成功为`Some`，headless、retryable-no-submit和独占兼容为`None`。中央提交失败发生在retained-cache ready commit和native present之前，不会发布伪成功状态。
4. PFO-4a只统一owner、ticket、fault admission和metrics/history，没有降低native submission count，也没有恢复共享surface timestamp。后续PFO-4b负责的copy身份与scene-tail融合已完成源码实现；PFO-4c继续负责neutral surface transaction。

静态证据：12个精确Rust文件通过`rustfmt --edition 2021 --check`；中央调用链、可选ticket、提交失败顺序、无共享legacy completion owner与800行预算合同通过；scoped `git diff --check`通过。未运行Cargo、真实WGPU、窗口、PNG、RenderDoc、profile、功耗或benchmark。

## PFO-4b external-image copy scene-tail融合（2026-08-27）

状态：`scene_tail_fusion_source_implemented_static_checks_passed_dynamic_validation_pending`，不是PFO-4完成或Runtime90 M8 accepted。

1. 136行external-image copy child把目标预分配、copy编码和ticket完成拆成三个阶段。产品渲染在scene录制前按最终输出尺寸创建generation-stable目标；child中无raw queue submit或device poll，没有共享owner的raw context会显式拒绝产品发布。
2. direct与compiled路径都在output writeback之后、diagnostic tail之前调用`encode_copy`，随后以同一个scene `SubmissionTicket`完成typed receipt。frame receipt强制同device generation与`scene <= viewport product < present`；当前产品路径取等号，不新增独立copy submission。
3. registry先调用`validate_viewport_product_publication(copy.generation(), product_submission)`，再移动image并更新索引；frame-extract与runtime-frame入口均在active graphics capture成功前完成publication，失败保留完整scene receipt和已记录product ticket。
4. compatibility `copy_texture_for_external_image`仍通过中央recorder/packet owner保留给非融合调用方，但graphics产品调用数为0；独占surface的私有device兼容路径不冒充共享产品owner。

算法与性能边界：admission、ticket attach与registry索引保持均摊`O(1)`，retained generation ring仍为每viewport固定3项；没有CPU pixel clone、map或wait。源码结构把新generation从独立第三次copy submission收敛为scene packet内的一条texture copy。静态顺序索引为direct `12734 < 13473 < 14176 < 14357`、compiled terminal `2236 < 2749 < 3030 < 3561 < 3962`、registry `1968 < 2307 < 3039`；compiled主owner由918行降到783行。精确rustfmt与scoped diff合同通过；Cargo、真实WGPU、PNG、RenderDoc、profile、功耗或benchmark未运行。

## SUI-0至SUI-3 standalone local owner硬切（2026-08-30）

状态：`runtime90_standalone_ui_sui_0_through_sui_3_source_implemented_static_checks_passed_dynamic_validation_pending`，不是Runtime90 M8/M9 accepted。

PFO-3/PFO-4a所述“独占surface继续保留兼容raw submit与本地主动poll”已被本切片替代。Standalone和offscreen现在共用`initial_wgpu_render_device_profile`；所有native UI context必有`Arc<WgpuRenderDevice>`，以typed `External/Local`区分谁在frame entry推进completion。standalone native present删除raw submit fallback并发布真实ticket；旧`GpuReadbackQueue`只在owner `poll_submissions`之后collect，直接`device.poll`退到test-only。共享UI仍不poll，standalone仍拒绝shared external image。Image in-flight pin随native packet进入submission service，由exact ticket完成或fault terminalization释放；UI不再登记第二个raw queue completion callback。

该切片没有把旧timer staging完全迁入production diagnostic query service，但该迁移桥已经失去poll authority。基础owner/profile合同11/11、SUI-3 ticket retirement合同8/8、精确rustfmt、源码顺序/负向扫描、结构预算、scoped diff和locked metadata通过；受管Cargo、真实窗口故障注入、PNG/RDC、profile与功耗均未执行。
