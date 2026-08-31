# Neutral Product Frame Owner Hard-Cut Design

## 状态

- 日期：2026-08-27
- 范围：Runtime90 M8 产品帧 device / submission / completion / surface owner 硬切前置设计
- 当前状态：`pfo_0_through_pfo_4c_pfo_4d0_and_pfo_4d4b_source_implemented_static_checks_passed_raw_device_queue_data_plane_and_dynamic_validation_pending`
- 验收限制：本记录只包含当前源码重审、结构决策与实施顺序，不代表 Cargo、真实 WGPU、RenderDoc、截图、性能、功耗、device-loss 或 surface 验收通过。

## 当前源码事实

当前产品帧的控制面已接入 production neutral RHI owner，但原生资源数据面尚未硬切：

1. `graphics::backend::RenderBackend`现在持有一个`WgpuRenderDevice`，不再持有第二份submission coordinator、fault supervisor、profile或UI registry。upload、command submit、poll、status、failure settlement、metrics、fault gate与UI context都委托给该owner。
2. 为兼容尚未neutral lowering的`SceneRenderer`、`ResourceStreamer`和旧Pass，`RenderBackend`仍保留同一generation的typed `Adapter/Device/Queue` clone。PFO-4d0已删除无consumer的raw `Instance` clone并把caps统一到`WgpuRenderDevice`；其余对象不拥有第二份submission history/fault callback，但queue write与原生资源数据面仍未中立化。
3. compiled scene 与 direct fallback 在帧首只调用一次`RenderBackend::poll_submission_completions`，并保留真实`SubmissionPollReceipt`；command buffer通过generation-qualified native packet进入`WgpuRenderDevice`唯一submission service。
4. scene、direct viewport product copy与surface blit的真实`SubmissionTicket`均可保留在frame receipt中，并约束为同device-generation序列。direct/compiled product copy与viewport surface blit已并入scene serial tail并复用scene ticket；direct/compiled neutral viewport present的源码拓扑为一个scene packet。retained UI和multi-viewport独立present仍保留显式packet。
5. 产品同步readback helper均已被`#[cfg(test)]`隔离，产品编译面为0个helper owner、0个direct-submit和0个blocking wait。graphics产品裸`wait_indefinitely`只剩1个批准的RenderDoc stop；UI retained-cache同名调用位于测试模块。共享UI context不再创建旧readback queue或推进第二条completion timeline，共享present和产品external-image copy也不再持有raw queue authority。该计数是静态调用面，不是profiler或性能数据。

问题不在单个 lock、allocation 或 draw call。关键断点是 generation-qualified identity 在产品帧边界被丢弃，而 device、resource、graph、surface 和 readback 仍以原生对象互相连接。只优化某个 helper 会保留双设备世界，也无法让资产 residency 的 completion/recovery 协议接入真实产品帧。

## 参考边界

Unreal 的适用部分是 owner 顺序，不是 D3D12 线程模型：

- `DynamicRHI.h`要求 command context 先 finalize，再由`RHISubmitCommandLists`统一提交；产品 renderer 不直接持有 native queue authority。
- `D3D12Submission.h/.cpp`把 command lists、query ranges、wait/signal 与 sync point 组织为 queue payload，并在 submission owner 分配单调 fence identity；query resolve 与 CPU-visible completion服从同一 payload。
- `DynamicRHI.cpp::RHIEndFrame_RenderThread`在 end-frame 边界 dispatch，而不是由每个 feature 自行 flush。
- Lumen 参考工程的`PreZ -> HZB -> lighting -> temporal -> tone map -> swapchain`只约束上层 graph 依赖。其多次`SubmitCommandList`和手写 D3D12 barrier 不迁入 Zircon；WGPU 当前 profile 仍只有一个物理串行 queue。

Zircon 应复刻“RHI owner + finalized packet + completion identity”的系统行为，同时保留 neutral RenderGraph compiler。不得复制 Unreal 的多 queue、submission thread 或 D3D12 resource-state 细节来伪造 WGPU 能力。

## 目标所有权

```text
WgpuRenderFrameworkCore
  -> ProductRenderDeviceOwner
     -> WgpuRenderDevice(DeviceId, DeviceGeneration)
        -> one ResourceRegistry
        -> one SubmissionService
        -> one CompletionService
        -> one SurfaceService
        -> one DiagnosticReadbackService
  -> RenderGraphFrameTransaction
     -> acquire/import surface or offscreen output
     -> materialize generation-qualified resources
     -> record immutable queue batches
     -> flush once per physical ordering boundary
     -> ProductFrameSubmissionReceipt
        { frame_generation, poll_receipt, scene_ticket, present_ticket }
```

最终状态没有平行 raw-WGPU `RenderBackend`。Runtime89 继续拥有 device-agnostic graph truth，并在 lowering 后交付 generation-qualified packet；Runtime90/WGPU owner 唯一拥有 native resource materialization、queue submit、poll、surface 和故障终态。

## 分层硬切

### PFO-0 身份贯通

先让现有过渡产品链停止丢弃身份，不改变 native submit 数量：

1. `WgpuSubmissionCoordinator::poll`和`RenderBackend::poll_submission_completions`返回原有`SubmissionPollReceipt`。
2. scene submit 返回真实`SubmissionTicket`；surface blit返回自己的 ticket。
3. renderer 内部结果以一个小型、纯值的 frame receipt 关联 CPU frame generation 与 device/generation-qualified tickets。
4. 任一 foreign generation、回退 receipt 或缺失 scene ticket 在发布 frame product 前 fail closed。

该切片是后续 residency、readback、surface readiness 和 profiling 的身份基础。它不是 neutral product hard cut，也不宣称减少提交次数。

### PFO-1 单一帧事务

把 frame-begin poll、upload admission、scene packet、diagnostic query、surface acquire/present 和 retirement 放入一个`RenderGraphFrameTransaction`：

- 帧首只允许一个 completion pump；poll receipt 只由事务向 completion consumers 分发。
- upload/readback/present 是有类别的 batch producer，不持有 queue。
- 事务提交成功后一次发布全部 ticket；中途错误按 Accepted/Submitted 状态 cancel 或 terminalize，不能留下半发布 frame。
- 热路径只追加 dense ticket/producer records，不按资源名或全表扫描。

### PFO-2 Device owner 替换

产品构造改为只创建`WgpuRenderDevice`。旧 pass 迁移期间如必须录制尚未 neutral lowering 的 WGPU 命令，只能使用`zr_rhi_wgpu`内部、事务作用域内的 native recorder lease：

- lease 可借用`Device`完成录制，但不暴露、克隆或保存`Queue`；command buffer 只能交还同一 generation 的 submission service。
- lease 不能创建第二套 fault supervisor、submission history、surface service 或 profile。
- 每迁移一个 owner 就删除对应 lease API；Runtime89 M6 packet hard cut 后删除 lease 本身，不形成长期 facade。

### PFO-3 Graph 与 present 融合

基础 MVP graph 固定为 output acquire/import、scene clear/opaque、compose/tone-map、present/copy。surface blit成为 terminal graph pass并与 scene command buffers进入同一 packet；`present_surface_frame`只消费已到 Submitted/Completed 的同 generation ticket，不再自行 submit。稳定 present frame 的目标是一个 native submission，而不是当前 scene + blit 两次提交。

### PFO-4 删除旧链

- 删除`RenderBackend`中的 raw `Device/Queue`、独立 coordinator、重复 fault supervisor和重复 profile。
- 产品同步direct-submit readback owner已退出产品编译面；测试工具保留`#[cfg(test)]`。同步外观只可等待自己的bounded diagnostic callback/ticket。
- 删除普通产品`wait_indefinitely`；shutdown/fatal/debugger 的批准例外必须有清单和 receipt。
- `ResourceStreamer`、history、IBL、UI、GPU scene 和 surface 全部消费 generation-qualified resource/upload/packet contract。
- source guard 最终只允许`zr_rhi_wgpu`的 submission service调用 native submit/request-device。

## 性能与复杂度约束

本轮没有运行 profiler，不报告耗时或功耗。实现后必须在同一硬件、同一 adapter/profile、同一场景与同一画质下采集：

- `native_submission_count / frame`：direct/compiled neutral viewport present源码目标已收敛为scene与surface blit一个packet；retained UI独立present仍是显式第二packet。真实计数必须由同generation动态profile证明。
- CPU `record / enqueue / flush` p50、p95，GPU completion latency p50、p95/max，pending upload bytes/peak，以及 command-buffer/ticket 数。
- 60帧预热后至少300帧，至少3次独立 steady run；diagnostics off/on分开采样。
- PNG、RDC、graph dump、frame receipt 与 profile 必须属于同一`DeviceGeneration`和 frame generation。
- 正确性、故障、资源寿命和 surface gates未通过前，不与 Unreal 比较性能或功耗。

稳态算法预算：frame transaction为`O(B + T)`，其中`B`是实际 batch 数、`T`是本帧 ticket 数；不得新增资源全表扫描、按字符串查 ticket、每帧重建 owner map或第二份 submission history。恢复仍是允许`O(N log N)`的冷路径。

## 第一生产切片

当前切片只实施 PFO-0 的最小身份贯通：

- coordinator/backend poll 返回 receipt；
- compiled/direct scene submit ticket向 renderer 返回；
- surface present ticket向 renderer返回；
- frame内部 receipt保持 scene/present ticket 与 frame generation。

切片完成后仍保留旧 raw WGPU resources、两次 present submission 和同步 readback helper，状态必须写为 source-only。后续动态验收需要 managed Cargo、真实 offscreen/present frame、fault injection、RenderDoc 与非纯文本 PNG；在这些证据齐全前不得标记 M8 accepted。

## 第一生产切片实施结果（2026-08-27）

状态：`source_implemented_static_checks_passed_dynamic_validation_pending`，不是 Runtime90 M8 accepted。

已完成：

1. `WgpuSubmissionCoordinator::poll`与`RenderBackend::poll_submission_completions`不再把成功的`SubmissionPollReceipt`降成`()`。
2. compiled scene 与 direct scene 的成功提交均返回真实`SubmissionTicket`；`RenderFrameSubmissionReceipt`把 CPU frame generation、frame-begin poll、scene ticket 与可选 present ticket绑定为纯值合同，并拒绝 poll/present 的 foreign device generation 以及不前进的 present sequence。
3. `ViewportSurface::present_texture`返回`Presented(ticket)`、`Reconfigured`、`DeferredTimeout`或`DeferredOccluded`。只有实际提交 blit 并调用 native present 的路径才发布 present ticket；重配、超时和遮挡不再伪装为已呈现提交。
4. `SceneRenderer`保留最近成功 frame receipt；compiled/direct offscreen、compiled/direct present 均更新它。framework 的`RenderStats::last_frame_submission_receipt`在 CPU generation 后发布同一 receipt，供后续 profile、residency、surface 与 capture 对齐。
5. 新增 8 个源内合同测试条目：4 个 frame receipt identity/ordering、1 个 surface outcome、1 个 direct frame retention、1 个 compiled frame retention、1 个 framework stats publication。`rustfmt --check`与 scoped `git diff --check`通过；未运行 Cargo、GPU、RenderDoc 或 benchmark。

当前源级库存与剩余风险：

- 稳定 present 成功路径的结构仍是 scene submit + surface blit submit，即 2 次 native submission/frame；本切片只保留两张 ticket，没有宣称 PFO-3 的 1 次提交目标完成。
- 同步readback helper源码仍保留测试工具，但产品编译面已为0个helper owner/0个direct submit/0个blocking wait。显式RGBA8/HDR capture改用唯一production diagnostic owner和30秒callback超时；graphics产品裸`wait_indefinitely`只剩RenderDoc stop批准例外。`zr_rhi_wgpu::ui_surface::retained_cache`中的等待只在测试模块。它们是PFO-4静态库存，不是实测瓶颈数值。
- texture pre-upload 与 history backing construction 等帧外准备路径仍可能通过 coordinator 独立提交并丢弃局部 ticket；它们必须在 PFO-1 变成有类别的 frame producer，不能把本切片误报为单一帧事务。
- compiled submit 后的 readback map 或 IBL writeback prepare/poll 仍可能在 native submission 已发生后返回错误。PFO-1 必须用 frame transaction terminal result同时携带 submitted ticket和失败 disposition，不能吞错，也不能让已提交 frame 变成无身份错误。
- `RenderBackend`仍拥有 raw WGPU device/queue；production`WgpuRenderDevice`尚未接管产品 renderer。动态证据、非纯文本截图与量化数据继续待 managed validation 和真实 GPU 环境。

## PFO-1a 帧边界 completion owner（2026-08-27）

状态：`source_implemented_static_checks_passed_dynamic_validation_pending`。

在 PFO-0 身份贯通之后，frame-begin completion pump 与 receipt finalize 已从两个 core scene实现上移到`SceneRenderer`帧入口：

1. direct/compiled frame owner在 reset 后先取得唯一`SubmissionPollReceipt`，再 drain readback completion、准备scene resources与history，最后调用core录制和提交。
2. direct core和compiled core不再调用`poll_submission_completions`，也不构造frame receipt；它们只返回scene`SubmissionTicket`。outer frame owner把先前poll receipt、CPU frame generation与scene ticket统一finalize为`RenderFrameSubmissionReceipt`。
3. 新增4个源内合同测试条目，分别锁定direct/compiled帧入口的`poll -> resource -> history/core`顺序，以及两个core不得重新取得completion pump。PFO-0与PFO-1a当前合计新增12个合同测试条目。

PFO-1a仍未把texture upload、history initialization、diagnostic或surface acquire变成可取消producer。PFO-1b必须为每个accepted producer保留ticket，在scene flush前出错时取消仍为`Accepted`的ticket；已到`Submitted`的ticket则进入带失败disposition的终态receipt。没有这层ledger前，不允许仅把pre-upload从immediate submit改成enqueue，否则失败帧会把未归属的pending work泄漏到下一次flush。

## PFO-1b pre-scene producer ledger 第一部分（2026-08-27）

状态：`history_producer_identity_source_implemented_static_checks_passed_cancel_and_texture_producers_pending`。

已完成：

1. 新增`RenderFrameSubmissionTransaction`。它在frame-begin持有poll receipt，只在真实pre-scene producer出现时增长dense `Vec`；成功finalize时把非空记录冻结为`Arc<[RenderFrameSubmissionProducerRecord]>`，使`RenderStats`克隆保持`O(1)`共享，steady无producer帧不创建记录数组。
2. producer ticket必须与poll属于同一device generation，记录顺序必须严格递增，且全部producer sequence必须早于scene ticket；任一违反均fail closed。
3. history texture初始化现在返回真实提交ticket；`prepare_history_textures`在新建或重建history时把它以`HistoryInitialization`记录到frame ledger。现有兼容构造入口仍返回history本身，只有产品frame路径消费精确ticket。
4. 新增7个源内合同测试条目：5个transaction identity/ordering/empty-steady合同、1个history构造ticket合同和1个history-to-frame-ledger合同。
5. coordinator的ticket status观察不再先要求健康admission。观察会先把fault gate同步到submission history终态，再返回原ticket的`DeviceLost`或`Failed`；submit、enqueue与poll仍fail closed。故障到提交终态的映射已收敛为`zr_rhi_wgpu::production`唯一策略，旧产品coordinator与neutral device不再各自维护一份映射。
6. `WgpuSubmissionService`新增abandoned-frame批量settlement：一次取得queue/state锁，先完整验证ticket，再以HashSet和一次pending保序过滤将Accepted转为Cancelled；Submitted、Completed、Failed、Cancelled与DeviceLost保持原状态。冷错误路径复杂度为`O(P + T)`，其中`P`是当前pending packet数、`T`是本次transaction ticket数；稳态成功帧不调用、不分配。
7. `RenderFrameSubmissionFailureReceipt`按producer/ticket/status保留settled pre-scene身份，拒绝status数量不完整或仍为Accepted的伪终态。compiled frame的history/target/core prepare错误域已接入：有producer时批量settle并通过`GraphicsError::FrameSubmissionFailed`携带原错误与receipt；无producer时直接返回原错误。
8. 本部分再新增6个源内合同测试条目：3个failure receipt完整性合同、1个transaction abort顺序合同、1个coordinator settlement路由合同和1个compiled failure owner合同。PFO-0、PFO-1a与当前PFO-1b部分合计新增27个合同测试条目。

本第一部分结束时尚未完成：texture pre/copy/post upload tickets、direct frame producer与scene-ticket failure receipt。上述缺口已由下列第二部分继续收敛；本段仅保留实施顺序事实。

## PFO-1b texture 与 submitted-scene failure ledger 第二部分（2026-08-27）

状态：`source_implemented_static_checks_passed_surface_and_diagnostic_transaction_integration_pending_dynamic_validation_pending`。

已完成：

1. 纹理首次上传和mip residency重建的pre/copy/post真实ticket均进入同一个frame transaction；每条dense producer record携带可选`ResourceId`，不建立第二份ticket表。材质、lightmap、light cookie、sprite、UI texture、2D LUT与mip apply的产品帧入口统一传递该transaction。
2. 失败结算只在冷路径复制ticket并调用一次批量settlement。`Accepted`转`Cancelled`，`Submitted/Completed`保持有效，`Failed/Cancelled/DeviceLost`纹理撤销缓存发布；回滚同时移除对应mip状态和按纹理依赖精确命中的材质代际，不执行全局`materials.clear()`。因此稳定帧不新增资源表扫描，失败路径为`O(P + T + M * D)`，其中`M`为缓存材质数、`D`为每材质纹理依赖数。
3. direct与compiled frame的resource/history/core错误出口统一进入`settle_failed_frame_submissions`。失败回执可额外携带已真实提交的scene ticket，并拒绝foreign device generation或producer不早于scene的伪回执。
4. direct readback map失败、compiled readback/IBL finalization失败以及compiled `store_last_runtime_outputs`失败不再丢弃scene ticket。core用`FrameFailedAfterSceneSubmission`显式返回submitted-error；compiled readback与IBL同时失败时聚合两项错误，外层先验证scene identity，再生成带scene ticket的失败回执。
5. 大文件按owner拆分：compiled frame主文件从1158行降至951行，submission owner为228行；material owner从约1008行降至949行，texture binding/cache identity子owner为111/53行；mip streaming主文件从1000行降至921行，frame apply子owner为115行。
6. PFO-0/PFO-1a/PFO-1b当前源码合同库存合计42项。新增覆盖包括纹理三阶段顺序与资源身份、scene resource/mip路由、失败纹理精确回滚、direct失败结算、submitted-scene回执与foreign owner拒绝、direct post-submit ticket保留；既有compiled post-submit顺序合同已更新为双错误聚合与scene identity合同。
7. direct/compiled surface presentation统一经过`finalize_surface_presentation`。重配、timeout与occluded继续返回scene-only receipt；surface validation或blit submit错误通过`FramePresentationFailed`携带scene receipt；present ticket若违反device generation/sequence合同，错误同时保留scene receipt和原present ticket。该切片不改变scene + blit两次native submission结构。

静态证据：本切片精确文件集`rustfmt --edition 2021 --check`通过；frame/texture部分17项、surface部分8项源码结构断言最终通过。检查器调试过程中曾把测试反向断言当作生产代码、错误计数函数定义、错误截断顶部`#[cfg(test)]`导入，以及使用被PowerShell展平的嵌套数组；这些失败均不计通过，修正生产区边界后的结果才作为证据。未运行Cargo、真实WGPU、RenderDoc、截图、profiler或功耗测试，不能标记Runtime90 M8 accepted。

仍未完成：surface acquire/present仍发生在scene transaction成功finalize之后，稳定present仍是scene与blit两次native submission；共享UI present与external-image copy仍直接调用raw queue submit。同步readback owner与共享UI第二completion推进者已在后续切片清零，但`RenderBackend` raw native资源/write/surface数据面仍存在。下一步把surface submission和typed query计时接入同一transaction，不在缺少动态通道时把验收队列当作唯一工作项。

2026-08-29 current-source reconciliation：上述第二部分的 texture producer、submitted-scene failure、diagnostic tail 与 surface transaction 缺口已由后续 PFO-3/PFO-4b/PFO-4c 源实现收敛，不能再作为当前源码状态使用。当前 direct/compiled 路径把纹理 pre/copy/post ticket、scene-tail product copy、surface blit、typed query 与 IBL artifact 绑定到唯一 scene packet；`WgpuNativeSurfaceFrameTarget` 在同一 `submit_graphics_command_buffers_with_frame_diagnostics_and_surface` 调用中录制并提交，present 只消费该 scene ticket。compiled-scene preparation 另外对缺失 live physical backing 与 GPU Scene upload 返回 typed error，后者先 abort pending realtime-IBL submission；framework boundary 对直接错误保留 typed identity。静态源证据与 metadata 已通过，受管 Cargo request `6d6bb68074fd4e3e8d6abaa787698b96` 仍在 `cargo.acquire` 阶段超时，故 Cargo、真实 WGPU、窗口、RenderDoc、PNG、profile、显存和功耗仍未验收。

## PFO-1 diagnostic / PFO-2 D0 量化基础（2026-08-27）

状态：`metrics_and_batch_delivery_source_implemented_static_checks_passed_product_owner_cutover_pending_dynamic_validation_pending`。

1. 重审确认`zr_rhi::DiagnosticReadbackTracker + WgpuDiagnosticReadbackService + WgpuRenderDevice::poll_submissions`已经是正确的唯一目标owner；旧`GpuReadbackQueue`拥有第二套device clone、ticket、budget、map callback与主动poll，不允许继续扩展或增加桥接层。
2. 精确区分源码库存和产品编译面：6 owner/7 submit/7 wait源码库存包含两个模块级test-only helper；产品编译面是4 owner/5 submit/5 wait，连同debugger stop和renderer drain共7个graphics产品blocking wait。该事实已写入独立迁移与性能计划。
3. production diagnostic service新增generation-qualified monotonic metrics snapshot/delta，覆盖request/bytes、batch/map/terminal/delivery、当前gauge与双维高水位；更新路径`O(1)`且不改变poll/submit算法。
4. device owner新增一次锁、顺序、move-only的batch delivery drain，为中央product router提供接入边界，避免feature owner争抢队列或逐delivery重复加锁。
5. 3项diagnostic合同、8文件rustfmt check、owner/source/export合同、5/5产品与2/2 test-only基线计数以及scoped diff check通过。Cargo、WGPU、profile、RenderDoc、PNG与功耗仍未执行，不得声称PFO-2或M8完成。

## PFO-2 控制面单 owner 切换（2026-08-27）

状态：`source_implemented_static_checks_passed_native_data_plane_surface_diagnostics_dynamic_validation_pending`。

1. 新增transaction-scoped `WgpuNativeRecorderLease/WgpuNativeSubmissionPacket`。旧Pass只归还同generation的opaque command packet，不能取得queue、poll或独立flush权限；空packet和foreign generation在submission ticket分配前拒绝。
2. native buffer/texture upload bridge直接复用`WgpuRenderDevice`唯一submission service与预算；commit失败立即取消已接纳ticket。abandoned transaction继续通过一次submission state锁批量settle，诊断终态与resource retirement同步更新。
3. 产品offscreen构造现在只安装一次device fault callback、只创建一份submission history/profile/UI registry。`RenderBackend`删除重复coordinator、supervisor、profile与UI registry字段，原公开调用面内部统一委托给`WgpuRenderDevice`；无调用方的`WgpuSubmissionCoordinator`类型与crate导出同时硬删除。
4. Runtime15 retained owner静态合同恢复为`Instance + Adapter + Config = 3`，不再把临时coordinator/supervisor计入长期backend结构合同。
5. 精确文件rustfmt check、scoped diff check、全crate旧coordinator符号清除、单一构造入口、无queue submit/poll逃逸与86行模块预算检查通过。未运行Cargo、真实WGPU、截图、RenderDoc、profiler、功耗或benchmark。

剩余边界：`RenderBackend`仍保留同generation的raw WGPU handles供旧Pass和surface数据面使用；`GpuReadbackQueue`仍是第二个diagnostic completion owner；surface blit仍独立提交。因此PFO-2与Runtime90 M8都不能标记完成。下一生产切片是中央diagnostic delivery router与viewport async capture迁移。

## PFO-3 viewport diagnostic tail 第一切片（2026-08-27）

状态：`viewport_async_capture_source_implemented_static_checks_passed_remaining_diagnostic_consumers_surface_dynamic_validation_pending`。

1. viewport async capture已退出旧三槽`GpuReadbackQueue`。产品只向`WgpuRenderDevice`登记受profile预算约束的RGBA8 native source lease；不兼容format/usage/sample/extent在录制前fail closed。
2. diagnostic copy位于compiled scene最后serial suffix并在output-target writeback之后，opaque staging frame与scene packet绑定同一真实ticket；capture不会再创建独立native submit或主动device poll。
3. 新增中央有界delivery router。device delivery一次锁批量move到router，request ID由`HashMap + VecDeque`均摊`O(1)`查找/删除并受profile上限约束，callback在所有owner锁之外执行；diagnostics-off不分配staging/callback/dispatch Vec。
4. viewport mailbox与capture profile retention改用production diagnostic pending budget，不再引用旧`GpuReadbackQueue::FRAME_SLOTS`。
5. 精确rustfmt、scoped diff、旧viewport request清除、copy/submit顺序、同ticket bind、router上限与无queue authority静态合同通过。未运行Cargo、真实WGPU、截图、RenderDoc、profiler、功耗或benchmark。

后续buffer consumer切片已将compiled-scene HZB stats、indirect args/draw-count和realtime IBL timestamp迁入同一production batch：native buffer lease在admission时校验`COPY_SRC`、范围与copy alignment，所有buffer/texture copy在最终serial tail编码并共享scene ticket。HZB保留4帧有界FIFO、source frame身份和drop/fail回退，不同步等待诊断结果。direct-scene realtime IBL随后也改用同一production source/router，把diagnostic tail附着到该路径唯一scene ticket；纯诊断begin/admission/prepare失败fail closed，且7个已有提交前失败出口都会释放IBL scheduler batch。

advanced plugin runtime-prepare readback也已迁入该batch。只有显式声明`requests_gpu_readback()`的collector才会在runtime-prepare前打开RAII diagnostic scope；提前返回自动取消active batch。viewport、plugin、HZB与IBL使用固定优先级登记并共享scene ticket。Hybrid GI保留语义化三帧反馈上限，粒子新增三帧逻辑批次上限且在compute前检查，避免production全局192-request容量把feature陈旧度从旧三槽无意放大；payload由router直接move到future completion，不新增字节克隆。

GPU timer/pipeline statistics随后已lower到typed native query reservation和中央typed delivery；IBL artifact也已拆成PMREM face/mip、SH9和IEM face请求并进入同一scene diagnostic tail。artifact feature owner只保留CPU section slots，成功scene submit后才提交reservation，下一帧唯一backend poll分发callback后写缓存；没有artifact专属command buffer、submit、map queue或poll。router满载改为显式拒绝新注册，不再静默遗失旧callback。

direct与compiled-pipeline帧入口现在共用`SceneRenderer::poll_frame_submission_completions`，固定执行`backend poll -> IBL callback/cache -> typed query route -> timer/statistics collect`并返回同一个`SubmissionPollReceipt`给frame transaction。`SceneRendererCore`已无`GpuReadbackQueue/readback_frame_index`；共享WGPU UI context也不再创建或poll旧queue，只有独占device兼容surface与测试保留本地owner。

该切片仍不代表PFO-3完成：产品同步helper已经退出产品编译面，共享UI旧completion owner也已隔离，但surface仍是第二次submit，raw resource/write数据面仍存在。静态顺序和owner合同通过，但未运行Cargo、真实WGPU、截图、RenderDoc、profiler、功耗或benchmark。后续迁移继续遵循独立diagnostic计划，不把当前consumer接入扩大解释为全管线验收。

## PFO-3 D3 同步capture owner收敛（2026-08-27）

RGBA8 viewport/output-target与RGBA16Float scene-color同步capture已统一到production diagnostic service。显式capture分配受profile约束的request，copy packet由唯一submission service提交，中央router把终态payload move到容量1 callback mailbox；等待循环只调用共享completion pump并有30秒上限。旧buffer/RGBA8/RGBA16Float同步helper均只在测试构建导出，产品面不再存在helper自有native submit或blocking wait。普通frame-begin行为不变；显式capture仍会产生一条独立copy submission，后续是否改为next-frame async request由产品API语义单独决定。

当前证据仅为源码格式、调用面和owner静态合同。Cargo、真实WGPU、PNG、RenderDoc、profile、功耗与device-loss/timeout动态验证均未运行，因此PFO-3与Runtime90 M8仍未accepted。

## PFO-3 UI Surface共享completion owner隔离（2026-08-27）

共享`WgpuUiSurfaceContext`现在显式标记不拥有completion timeline，`WgpuUiSurfaceRenderer`只为自己申请独立device的surface创建旧GPU timer与`GpuReadbackQueue`。因此运行时共享surface不会再执行第二次device poll；请求GPU timing时暂时报告unsupported，直到surface command packet接入production typed query router。独占surface继续在自己的device上保留兼容计时。

该隔离本身不等于surface submission硬切；其后的PFO-4a已收敛共享UI present owner，PFO-4b已把external-image copy融合进scene packet，neutral surface transaction仍待后续。稳定scene+surface当前仍是两次提交。真实窗口、RenderDoc、profile与功耗数据仍待动态阶段。

## PFO-4a 共享UI present中央owner（2026-08-27）

`RenderBackend`和产品UI context现在共享同一个`Arc<WgpuRenderDevice>`。共享surface录制完成后把command buffer归还generation-qualified native recorder packet，并调用唯一device owner提交；`render_draw_list_to_surface`不再直接`queue.submit`。真实ticket通过`UiSurfacePresentStats::submission`保留，共享成功路径为`Some`，headless、retryable和独占兼容路径为`None`。

提交顺序固定为`central submit -> retained-cache ready commit -> native present -> success publication`。因此admission或flush失败不会发布ready cache，也不会调用native present。该切片没有减少提交次数：scene packet与UI present packet仍各提交一次；其后的PFO-4b已把新direct/compiled viewport generation copy编码并入scene serial tail并复用scene ticket。PFO-4c才负责neutral acquire/target/submit/present transaction与可行路径的一次提交融合。

当前只有12个精确Rust文件的rustfmt、source-contract、顺序、800行预算和scoped diff静态证据。Cargo、真实WGPU窗口、PNG、RenderDoc、300帧profile、present latency和功耗均未运行，PFO-4与Runtime90 M8仍未accepted。

## PFO-4b external-image copy身份与scene-tail融合（2026-08-27）

共享context在scene录制前按最终输出尺寸预分配generation-stable目标；direct/compiled渲染都在output writeback之后、diagnostic tail之前把copy命令追加到scene encoder。typed receipt把image绑定到同一个scene ticket；frame receipt校验同generation及`scene <= product < present`，registry进一步要求publication generation和记录的product ticket完全匹配后才移动image并写入状态。raw context仍会显式拒绝无共享owner的产品发布。

`ViewportAsyncCaptureSubmission`携带完整scene receipt穿过framework，而不是只携带裸generation。预分配/scene submit失败走既有帧失败身份；scene提交后的product receipt或registry身份失败在公共`RenderFrameworkError::FrameProductPublicationFailed`中保留scene receipt和已记录product ticket。两个产品入口在graphics debugger capture成功发布前完成该事务，避免“截图成功、产品发布失败”的相互矛盾终态。

当前切片只有rustfmt、scoped whitespace、源码owner/顺序/错误身份和文件预算证据。external copy产品路径为0个raw submit、0个device poll、0个compatibility copy调用；direct与compiled的copy均位于scene terminal tail并复用scene ticket，新generation不再增加独立submission。compiled主owner由918行拆到783行，terminal/lifecycle/irradiance child为112/68/37行。真实WGPU、PNG、RenderDoc、300帧submission/profile和功耗验证待后续；PFO-4c与Runtime90 M8仍未accepted。

## PFO-4c neutral viewport surface transaction与scene packet融合（2026-08-27）

状态：`source_implemented_static_checks_passed_dynamic_validation_pending`。

`ViewportSurface`的raw surface/configuration/acquire/present owner已硬删除，真实surface和`SurfaceTexture`只由production `WgpuSurfaceService`持有。新RAII `WgpuNativeSurfaceFrameTarget`封装精确frame lease与短生命周期target view；record回调不暴露queue/poll/flush，且录制和提交都校验同一个`WgpuRenderDevice` owner。compiled PrimarySurface 图通过内部 `Present` stage 的 non-cullable `surface-present` pass读取最终逻辑输出，并在同一scene encoder上录制surface blit；纹理/headless图不含该pass。direct fallback仍在兼容serial tail录制同packet blit，待fallback删除或graph化。两条路径都由scene packet的唯一flush登记同一个ticket；neutral present只消费scene ticket，旧独立surface packet和`present_texture`入口已删除。

失败收尾覆盖prepare/record/submit/present：显式discard保留原错误与cleanup错误，已提交present失败保留scene/present同一ticket；frame receipt接受`scene == viewport product == present`而不放松device/generation或前后次序。源码静态审计为产品旧present调用/定义0、surface raw submit/acquire0，direct/compiled present各1次frame poll。compiled frame主owner从995行拆到773行，surface/test child为139/92行，其余触及生产owner均低于800行。

该结果不把retained UI或multi-viewport独立present强行融合，也不把direct fallback误报为compiled graph pass。compiled terminal tail blit静态计数为0、graph executor blit为1；Cargo、真实WGPU、窗口、PNG、RenderDoc、300帧submission/profile、显存和功耗仍待动态阶段，PFO-4与Runtime90 M8均未accepted。

## PFO-4d0 冷路径native owner与capability单一事实源（2026-08-27）

完整调用审计确认raw resource/write切面覆盖至少230处生产候选device访问和99处queue访问，不能通过扩大native recorder权限或一次全局替换解决。独立PFO-4d计划已按冷身份、动态上传、持久资源、RDG瞬态池与最终字段删除分阶段，并以Unreal Dynamic RHI、RenderResource和RDG upload/resource owner为约束。

第一切片已删除`RenderBackend`无consumer的raw `Instance` clone，offscreen bootstrap直接把instance move进one-shot `WgpuRenderDeviceContext`；`RenderBackend::caps`也不再第二次读取adapter/device映射能力，而只消费production `WgpuRenderDevice`已经验证并fail-close到neutral ABI的caps。Runtime15 F12 guard同步保护唯一generation owner、profile、caps与shared UI context，不再保护旧人工三字段保活计数。

源码扫描中产品`backend.instance`与`instance.clone()`均为0；三个精确Rust文件的rustfmt check和scoped diff check通过。该切片没有迁移raw `Adapter/Device/Queue`，没有运行Cargo、WGPU、PNG、RenderDoc、profile、显存或功耗，因此只标source implemented，下一步是PFO-4d1动态write batch收敛。

## PFO-4d4b product raw queue权限收敛（2026-08-30）

后续PFO-4d1上传事务已使scene resource preparation中的queue参数退化为无行为透传；product GPU timer也只用queue读取已由`WgpuRenderDevice`保存的timestamp period。PFO-4d4b删除这两条权限链：frame resource/material入口不再接收queue，timer改消费device-owner immutable fact。测试fixture与legacy standalone UI query构造保留具名例外，不创建通用queue accessor。

failing-first 0/5转为扩展合同7/7，精确rustfmt、scoped diff、结构预算与locked metadata通过。该切片不改变上传、提交、query delivery或缓存算法；其余raw Device/Queue consumer以及Cargo、WGPU、PNG/RDC、profile、显存与功耗仍开放，因此PFO-4d/M8不标accepted。
