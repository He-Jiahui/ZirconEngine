# Standalone UI Local Submission and Completion Hard-Cut Design

## 状态

- 日期：2026-08-30
- 范围：Runtime90 M6/M8，standalone native UI 自有WGPU device的submission、completion、diagnostic readback与surface retirement
- 当前状态：`runtime90_standalone_ui_sui_0_through_sui_3_source_implemented_static_checks_passed_dynamic_validation_pending`
- 已完成：当前源码调用图、共享/独占profile差异、旧readback completion owner、Unreal SlateRHI/RHI submission顺序重审，以及SUI-0 owned device bundle、SUI-1 central submit、SUI-2 single local completion owner、SUI-3 ticket-qualified image pin retirement与fault/surface terminal源码hard cut
- 未完成：SUI-4受管Cargo、真实WGPU窗口、device-loss/resize/timeout行为验收、PNG/RDC、300帧profile、显存、功耗与跨引擎量化

本记录是设计与实施状态记录，不是accepted milestone，也不证明submit次数、帧耗时或功耗已经改善。

## 切入前源码事实

共享device的retained UI路径已经接入正确的中央owner：

1. `WgpuRenderDevice::ui_surface_context`把同一`Arc<WgpuRenderDevice>`交给UI context；context明确设置`owns_completion_timeline = false`。
2. `WgpuUiSurfaceRenderer::submit_present_command_buffer`把完成录制的command buffer归还native recorder，并经`submit_native_recording_packet`取得真实`SubmissionTicket`。
3. `presentation.rs`在中央submit成功后才提交retained-cache ready状态、开始map、调用`SurfaceTexture::present`并发布成功统计。

standalone路径仍是第二个device-local世界：

1. `WgpuUiSurfaceRenderer::new_owned`自行创建instance、adapter、device和queue；`new_with_owned_completion_timeline`只保存raw handles，`shared_render_device`为`None`。
2. `submit_present_command_buffer`在该分支直接调用`self.queue.submit`并返回`None`，因此没有device/generation-qualified ticket、fault admission、submission history或统一terminal settlement。
3. standalone GPU timer创建`GpuReadbackQueue`。该queue克隆`wgpu::Device`，`poll_completed`会直接执行`device.poll`；它与present raw submit共同形成独立但未建模的completion timeline。
4. 只把raw submit换成临时`WgpuRenderDevice`仍然错误：旧readback queue会继续主动poll同一个native device，造成两个completion owner，并让query/map terminal与submission ticket无法关联。
5. 共享路径支持同device external image；standalone device不能导入该资源。该差异必须是profile contract，不能在运行时静默尝试cross-device sampling。

## Unreal参考与适用边界

仓库内UE 5.5.4源码给出的可复刻行为是owner顺序：

- `SlateRHIRenderer.cpp`把窗口绘制工作排入render command，并在`DrawWindows_RenderThread`完成窗口绘制后统一进入`PresentWindow_RenderThread`。
- `SlateRHIRenderingPolicy.cpp`通过传入的`FRHICommandListImmediate`记录Slate资源和draw，不给widget或atlas helper独立queue authority。
- present前资源进入`ERHIAccess::Present`，随后由`EndDrawingViewport`结束同一viewport transaction；资源删除通过render command与`FlushPendingDeletes`延迟到RHI owner。
- `RHICommandList.cpp`先`FinishRecording`，再由RHI task/submit owner翻译和提交command lists；显式flush属于集中policy点。

Zircon复刻“finalized packet + single submission/completion owner + present/retirement identity”，不复制UE的RHI线程、多queue、D3D12 fence或Slate批处理细节。Standalone UI可以保留独立device profile，但该profile内部仍只能有一个本地RHI owner。

## 目标所有权

```text
StandaloneUiDeviceBundle
  -> immutable RenderDeviceProfile(DeviceId, DeviceGeneration, limits, budgets)
  -> Arc<WgpuRenderDevice>
     -> one SubmissionService
     -> one CompletionService
     -> one DiagnosticReadbackService
     -> one SurfaceService
  -> WgpuUiSurfaceContext
     -> recorder handles only
     -> same Arc<WgpuRenderDevice>

standalone present transaction
  -> poll local owner once
  -> acquire surface
  -> record UI + optional query/readback intents
  -> submit finalized packet -> required SubmissionTicket
  -> publish diagnostic binding and resource pins
  -> present
  -> later completion receipt retires pins/results
```

最终生产状态不再用`owns_completion_timeline: bool`表达互斥但不完整的所有权。共享和standalone context都必须持有一个明确的render-device owner；差异由typed profile表达：

- `SharedRuntime`：复用runtime generation，允许同generation external image，不主动poll。
- `StandaloneTool`：拥有独立generation，不允许runtime external image，由本地帧入口唯一poll。

## 必须维持的不变量

1. 每个native device恰好一个submit owner和一个poll owner；query、readback、surface、fault与retirement消费同一ticket timeline。
2. 每个成功standalone native present必须发布真实`SubmissionTicket`；headless/retryable-no-submit可无ticket，不能以`None`表示raw submit成功。
3. 任何diagnostic intent都在encode前有界admission，只有terminal packet取得ticket后才发布submission binding。
4. map callback只登记完成状态；结果收集发生在owner完成一次device poll之后，不得再次poll。
5. retained image allocation pins按submission terminal retirement，不使用无身份的`queue.on_submitted_work_done`作为最终寿命协议。
6. device lost先停止admission，再一次性terminalize submission、diagnostic和surface；旧generation不得被新surface重配复用。
7. standalone profile fail closed拒绝shared external image，不能复制到第二device或静默退回CPU readback。
8. 迁移不得增加泛型queue facade、可克隆submission service或第二套device-profile builder。

## 依赖有序实施

### SUI-0 Owned device bundle

1. 把standalone的adapter facts、feature negotiation、limits、budgets和新`DeviceId`固化为一个immutable `RenderDeviceProfile`。
2. 由唯一factory同时构造`Arc<WgpuRenderDevice>`和UI surface context；删除“raw owned handles但没有render owner”的可构造状态。
3. 纯合同测试锁定共享/standalone profile、external-image policy和generation identity，不改变present调用面。

### SUI-1 Submission hard cut

1. 删除standalone `self.queue.submit` fallback；共享和standalone command buffer都必须进入各自`WgpuRenderDevice::submit_native_recording_packet`。
2. native成功present要求`WgpuUiSurfacePresentation::submission = Some(ticket)`，并验证ticket属于context profile。
3. submit失败时不得commit retained cache、开始map、present或发布成功统计。

SUI-1不能脱离SUI-2单独合入，否则会制造中央submit加旧readback主动poll的双owner状态。

### SUI-2 Completion and diagnostic hard cut

1. standalone帧入口在acquire/record前调用本地`poll_submissions`一次，并把receipt路由给UI diagnostic consumer。
2. UI timing/statistics迁入production diagnostic/query service；若迁移期间复用旧staging ring，只允许新增不poll的`collect_completed_after_owner_poll`内部接口，并立即登记删除条件。
3. 删除standalone产品`GpuReadbackQueue` owner及其`device.poll`调用面；callback/result ring继续有profile预算和typed terminal。

### SUI-3 Fault and retirement

1. image/text/retained-cache pinning绑定ticket terminal；device loss、shutdown和surface loss一次性释放或标记stale。
2. resize/reconfigure发布新的surface generation，但不改变device generation；旧surface frame不能在新配置上present。
3. 将standalone fault/terminal receipt接入UI surface stats和上层错误边界，不擦除typed原因。

### SUI-4 Dynamic qualification

1. 通过受管validator执行focused Rust和workspace闭包；已知`cargo_reuse_target_mismatch`消失前不重复声称Cargo结果。
2. 在Windows真实窗口验证创建、普通绘制、resize、minimize/restore、surface lost/outdated、device lost与shutdown；shared profile另验external image，standalone明确拒绝。
3. 在`docs/tests/runtime/render`落盘真实PNG和同generation receipt；使用`D:\Tools\renderdoc`生成RDC并核对present前资源状态与packet边界。
4. 固定adapter/profile/分辨率/场景，60帧预热、至少300帧和3次steady run，记录CPU encode/submit、GPU UI timestamp、logical packet、physical submit、queue completion latency、present latency、VRAM/RSS与功耗。

## Source guards与复杂度门

- `ui_surface.rs`生产分支的native `queue.submit`为0；只有`production/submission.rs`拥有native submit。
- `ui_surface`生产树的直接`device.poll`为0；每个owned device帧只有一次`poll_submissions`。
- shared与standalone context都恰好持有一个`Arc<WgpuRenderDevice>`，不存在“有device/queue但没有owner”的生产构造器。
- native成功present无`None` submission，foreign generation ticket fail closed。
- admission、record、submit和结果收集均为每packet/request `O(N)`；terminal lookup与history保持现有有界/均摊`O(1)`合同。
- 不在profile前改动batch合并、submit数量或staging分配算法。先用上述ticket metrics定位瓶颈，再写量化优化计划并实施。

## SUI-0至SUI-3实施结果

状态：`runtime90_standalone_ui_sui_0_through_sui_3_source_implemented_static_checks_passed_dynamic_validation_pending`。

1. `device_profile.rs`新增唯一`initial_wgpu_render_device_profile` factory；offscreen runtime与standalone UI都复用同一`DeviceId`、initial generation、feature negotiation、limits、single-queue topology、memory/submission/diagnostic budget构造，不再分别复制profile默认值。
2. `WgpuUiSurfaceContext`删除无render owner的raw public构造状态，改为必有`Arc<WgpuRenderDevice>`与typed `External/Local` completion owner。`new_owned`请求surface-compatible adapter后直接构造标准owner，再从该owner取得本地context；standalone external image仍按profile fail closed。
3. `submit_present_command_buffer`删除raw `self.queue.submit` fallback，所有native成功present都经native recorder/packet owner提交并发布`Some(SubmissionTicket)`。submit仍位于retained-cache commit、map begin和native present之前。
4. local present在resize/acquire之前调用一次`render_device.poll_submissions`；共享context的`External`分支不poll。旧`GpuReadbackQueue`产品调用面只保留`collect_completed_after_device_poll`，直接`device.poll`方法已限制为`#[cfg(test)]`。
5. SUI-2当前采用计划允许的迁移桥：timer/query staging仍使用旧queue的数据结构，但它不再拥有poll authority。后续若迁入production diagnostic query service，必须保持同一frame-entry poll与ticket终态，不得恢复第二个completion owner。
6. native UI packet把`WgpuUiImageInFlightPins`作为不可克隆payload随command buffers提交；submission service在native submit成功后以同一`SubmissionTicket`登记到有界`HashMap`，不再为pin另建无身份的`queue.on_submitted_work_done`回调。
7. 现有唯一submission completion callback先把同批ticket转为`Completed`，释放提交状态锁后再按ticket移除pin；device fault、poll error与shutdown terminalization清空同一退休owner。accepted/pending取消也在提交状态锁外析构packet与pin，避免资源析构扩大临界区。
8. surface reconfigure继续先取消旧session在途工作并discard旧frame，再分配replacement session identity；device fault/poll error同时terminalize submission、diagnostic、surface frame并prune资源，不把旧frame复用于新配置。

源码证据：SUI-0至SUI-2 failing source contract落地前，typed owner/required owner/no optional owner/no raw submit/local poll/after-poll collect为`false/true/false/false/false/false`；profile factory五项为全false，实施后两组分别为6/6与5/5通过。SUI-3 ticket retirement合同在实现前为0/8，实施后为8/8。精确rustfmt、scoped `git diff --check`、poll-before-acquire与submit-before-commit/present顺序、产品UI direct submit/poll/extra completion callback负向扫描及locked metadata通过。顺序源码索引为静态位置而非耗时。当前owner物理行数为submission root 763、queued work 58、UI retirement 53、native recording 475、device root 798、fault terminal 51、UI root 715、surface setup 206、external-image copy 129、presentation 477、readback queue 749、timeline tests 113，均低于现有800行结构门。

受管Cargo未重试，仍受已知`cargo_reuse_target_mismatch`前置门阻断；没有真实WGPU、窗口、PNG、RDC、profile或功耗证据。因此SUI-4、Runtime90 M8/M9与性能结论保持开放。

## 完成定义

SUI-0至SUI-3源码、合同、fault和结构门已通过；SUI-4生成真实窗口、PNG、RDC、profile与功耗证据后，standalone兼容边界才可关闭。当前状态为`runtime90_standalone_ui_sui_0_through_sui_3_source_implemented_static_checks_passed_dynamic_validation_pending`，不提交accepted milestone、不发送企微完成通知。
