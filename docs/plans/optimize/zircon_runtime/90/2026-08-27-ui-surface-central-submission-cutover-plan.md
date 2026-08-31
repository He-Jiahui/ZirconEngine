# UI Surface Central Submission Cutover Plan

## 状态

- 日期：2026-08-27
- 范围：Runtime90 PFO-4a/PFO-4b/PFO-4c，共享WGPU UI与viewport surface提交数据面
- 当前状态：`pfo_4a_pfo_4b_pfo_4c_source_implemented_static_checks_passed_dynamic_validation_pending`
- 证据边界：只完成源码、计划、现有neutral surface/native packet合同与仓库内Unreal RHI owner顺序重审；未运行Cargo、真实窗口、WGPU、RenderDoc、PNG、profile、功耗或benchmark。

## 切入前瓶颈不是单次API开销

切入前共享UI路径有2个产品raw submit入口：

1. `WgpuUiSurfaceRenderer::render_draw_list_to_surface`每个成功native present直接`queue.submit`一次，提交身份不进入`SubmissionHistory`，`UiSurfacePresentStats`也没有ticket。
2. `WgpuUiSurfaceContext::copy_texture_for_external_image`每次发布新的retained viewport generation直接`queue.submit`一次，调用方返回普通`RenderViewportProduct`，无法表达admission、submit或flush失败。

稳定scene与UI present因此至少是scene packet加surface packet两次native submission；若同帧发布新的direct viewport product，还会增加一次copy submission。问题不是函数调用本身，而是UI持有长期raw queue authority：提交绕过generation、fault gate、backpressure、ticket history、metrics和统一terminal settlement，无法与资源retirement和性能证据关联。

## 参考边界与选择

Unreal的适用约束是RHI command list先finalize，再由统一RHI submit owner发布；present资源和提交身份不能脱离同一device timeline。Zircon已有等价基础：

- `WgpuNativeRecorderLease`只借出device录制，不暴露queue/poll/flush；
- `WgpuNativeSubmissionPacket`携带`DeviceId + DeviceGeneration + RenderQueueClass`；
- `WgpuRenderDevice::submit_native_recording_packet`执行fault admission、ticket reserve/commit、唯一flush和失败cancel；
- neutral `SurfaceFrameLease/SurfacePresentReceipt`已经证明acquire/target/ticket/present的正确终态形态。

不新增可克隆的半功能submission service。`RenderBackend`改为`Arc<WgpuRenderDevice>`持有唯一owner，长期UI presenter的私有context克隆同一个Arc。这样owner的fault、diagnostics、surface、registry和submission状态仍是一体，且device可安全活到最后一个presenter释放。

## 分阶段实施

### PFO-4a 共享UI present

1. `RenderBackend::render_device`改为`Arc<WgpuRenderDevice>`；`ui_surface_context(self: &Arc<Self>)`只向crate内部context写入同一owner。
2. UI recorder仍用现有raw render-pass实现，但完成后把command buffer归还`WgpuNativeRecorderLease`并调用唯一device owner提交；独占fallback surface保留自己的raw queue。
3. `UiSurfacePresentStats`增加`Option<SubmissionTicket>`。共享native成功提交必须为`Some`；headless、独占兼容与retryable-no-submit为`None`，不得伪造ticket。
4. submit失败时不提交retained-cache ready状态、不调用native present，并通过现有`Result<RhiError>`上抛。

### PFO-4b external-image copy

1. `copy_texture_for_external_image`改为返回带ticket的typed result；`ViewportProductRegistry::publish`改为`Result`。
2. direct/compiled framework发布失败必须保留已提交scene receipt，并形成post-submit failure identity，不能把复制失败降级成成功产品。
3. 把copy命令并入scene serial tail，使新generation发布不新增第三次submit；目标纹理必须在scene录制前按最终输出尺寸预分配，提交成功后才允许registry发布。

### PFO-4c surface transaction融合

将UI surface acquire/record/submit/present迁入neutral surface session；当scene/UI直接渲染同一acquired target时，稳定路径目标为1个native submission/frame。保留offscreen composition的明确双packet模式，不为追求数字破坏多viewport/retained UI语义。

## 性能与验收指标

- diagnostics/timing关闭的共享UI present：0次额外device poll、0个legacy readback owner、1个中央surface ticket；
- PFO-4a后native submit次数暂不下降，但100%进入同一submission metrics/history；
- PFO-4b源码目标已达到：direct/compiled product copy并入scene tail，新generation不增加独立submit；动态submission计数仍待真实WGPU验证；
- PFO-4c目标：可直接surface composition的稳定帧为1次native submit，其他模式按显式receipt计数；
- 所有算法为每packet `O(command_buffer_count)`，admission/history为均摊`O(1)`且受既有submission limits约束；
- 动态阶段必须记录300帧CPU frame、GPU UI timestamp、submission count、queue latency、present latency、显存、功耗、PNG与RDC，并与同generation ticket关联。没有这些数据不得声称瓶颈消失或达到其它引擎经验值。

## PFO-4a 实施结果（2026-08-27）

状态：`source_implemented_static_checks_passed_cargo_wgpu_surface_profile_pending`，不是PFO-4或Runtime90 M8 accepted。

1. `RenderBackend`和长期共享UI context现在持有同一个`Arc<WgpuRenderDevice>`；context不克隆第二个submission/fault/diagnostic/surface owner，也不形成owner环。`WgpuRenderDevice::ui_surface_context(self: &Arc<Self>)`只在crate内部写入该owner，公共raw handle构造仍明确属于兼容入口。
2. 共享native present完成录制后把command buffer归还`WgpuNativeRecorderLease`，再调用`submit_native_recording_packet`。fault admission、generation校验、ticket reserve/commit、唯一flush和失败cancel因此全部由同一device owner执行；present函数不再直接调用`queue.submit`。
3. `UiSurfacePresentStats::submission`、`WgpuUiSurfacePresentation::submission`和renderer stats保留真实`Option<SubmissionTicket>`。共享成功present为`Some`；headless、retryable-no-submit和独占兼容路径为`None`，不伪造中央ticket。
4. 提交发生在retained-cache ready commit和`SurfaceTexture::present()`之前。中央提交失败会直接返回`RhiError`，不发布cache-ready状态，也不调用native present。共享context继续不创建或poll旧`GpuReadbackQueue`；独占自有device surface保留本地兼容计时和raw submit。

静态证据：12个精确Rust文件通过`rustfmt --edition 2021 --check`；共享present源码中raw `self.queue.submit`为0，中央helper包含`begin_native_recording -> adopt command buffer -> finish packet -> submit_native_recording_packet`；顺序索引为`central submit 8842 < retained-cache commit 9066 < native present 9671 < success publication 9706`。相关owner行数为neutral UI contract 759、production device 797、WGPU UI parent 755、presentation 448、native-submission tests 759、timeline tests 51、backend parent 174，均未超过当前800行预算。scoped `git diff --check`通过，仅有LF/CRLF提示。

## PFO-4b 实施结果与未完成

### PFO-4b 中央owner与失败身份（2026-08-27）

状态：`scene_tail_fusion_source_implemented_static_checks_passed_dynamic_validation_pending`。

1. external-image copy从712行UI parent拆到74行`external_image_copy.rs`。raw context不再被允许静默提交无身份产品；只有持有共享`Arc<WgpuRenderDevice>`的context可以开始generation-qualified Graphics recorder、录制copy packet并由唯一device owner提交。
2. `WgpuUiExternalImageCopyReceipt`绑定generation-stable image与真实`SubmissionTicket`。`ViewportProductRegistry::publish`改为`Result`，先提交copy、再用scene receipt校验同device generation和严格递增sequence，只有校验成功才更新有界generation registry。
3. `RenderFrameSubmissionReceipt`新增可选`viewport_product_submission`，并约束`scene <= viewport product < present`。direct与compiled路径现在都复用scene ticket；`ViewportAsyncCaptureSubmission`保留完整scene receipt，registry校验copy generation和同一product ticket后才移动image并提交状态。
4. copy admission/submit失败携带scene receipt且`product_submission=None`；copy已经提交后receipt校验失败同时携带scene receipt和`Some(copy ticket)`。该身份通过新增公共`RenderFrameworkError::FrameProductPublicationFailed`越过framework边界，不再降级为只有字符串的`Backend`错误。
5. direct extract与runtime frame入口都固定执行`generation validation -> product publication -> capture success`。publication失败会先终结active debugger capture并返回typed错误，不会先发布capture成功。

静态证据：PFO-4b owner文件已通过精确rustfmt，后续compiled owner拆分的6个Rust文件再次通过`rustfmt --edition 2021 --check`；scoped `git diff --check`通过，仅有LF/CRLF提示。external copy child为136行，产品graphics对兼容`copy_texture_for_external_image`调用为0，child中raw `queue.submit`/device poll均为0。direct顺序索引为`writeback 12734 < product copy 13473 < query tail 14176 < scene submit 14357`；compiled terminal顺序为`writeback 2236 < product copy 2749 < diagnostic copy 3030 < query tail 3561 < encoder finish 3962`。registry顺序为`ticket/generation validation 1968 < image move 2307 < mutation 3039`。compiled主owner由918行降到783行，拆出的terminal/lifecycle/irradiance owner分别为112/68/37行。

PFO-4b源码融合已经完成：generation-stable目标在scene录制前预分配，direct/compiled都在output writeback之后、diagnostic tail之前把copy编码进scene packet，copy receipt以同一个scene ticket完成，registry在验证帧generation与票据完全相等后才发布。新generation因此不再创建独立native submission；独立共享UI present仍保留自己的显式packet。

## PFO-4c neutral viewport surface transaction结果（2026-08-27）

状态：`source_implemented_static_checks_passed_dynamic_validation_pending`。

`ViewportSurface`不再拥有raw WGPU surface/configuration/acquire/present提交路径；它只保留共享`Arc<WgpuRenderDevice>`、neutral session receipt和blit resources。direct/compiled present在本帧唯一completion poll之后取得RAII `WgpuNativeSurfaceFrameTarget`，把blit追加到scene terminal encoder；submit owner在唯一flush前绑定同一个scene ticket，neutral present随后消费该ticket。旧`present_texture`、独立surface packet及对应WGPU packet类型已硬删除。

失败路径在prepare、record、submit、present阶段都保留原错误并显式discard；cleanup失败形成typed组合错误，Drop只作最后重试。frame receipt允许`scene == viewport product == present`，但仍拒绝foreign owner/generation和早于scene/product的票据。源码静态计数为direct/compiled产品旧present调用0、viewport旧present定义0、surface raw submit/acquire 0；direct与compiled present各只有1次frame completion poll。compiled frame主owner已从995行拆到773行，surface/test child为139/92行。

这只证明direct/compiled viewport surface的单packet源码拓扑；retained UI与multi-viewport独立present仍按显式packet语义执行。Cargo、真实WGPU、窗口present、PNG、RenderDoc、300帧profile和功耗均未验收；Runtime90 M8保持未accepted，不提交里程碑Git、不推送协调器、不发送企微完成通知。

## Standalone UI local owner收敛（2026-08-30）

状态：`runtime90_standalone_ui_sui_0_through_sui_3_source_implemented_static_checks_passed_dynamic_validation_pending`。

PFO-4a保留的独占兼容边界已完成SUI-0至SUI-3源码硬切。`device_profile.rs`集中唯一initial WGPU profile factory，offscreen与standalone不再复制generation/limits/budget默认值；UI context删除无render owner构造状态。共享与standalone present都通过各自`Arc<WgpuRenderDevice>`提交，成功native present必有ticket。只有typed `Local` owner在surface acquire前调用一次`poll_submissions`，旧readback staging只收集该poll之后的callback，产品UI不再直接submit或poll native device。Image pin附着到finalized native packet，以submission ticket进入有界退休表，并由唯一中央completion callback或fault terminalization释放。

本增量不声称retained UI packet已与scene融合，也未完成动态验收。Device admission fault与poll error均同步terminalize submission、diagnostic、surface frames和pin retirement；surface reconfigure继续discard旧frame并分配replacement session identity。当前submission/queued-work/UI-retirement/native-recording/device/fault-terminal owner为`763/58/53/475/798/51`行，UI root/surface-setup/external-copy/presentation/readback/timeline-test为`715/206/129/477/749/113`行；基础11项与SUI-3 8项合同、精确rustfmt、scoped diff、locked metadata通过。Cargo、窗口、PNG、RDC、profile、显存和功耗仍为空。
