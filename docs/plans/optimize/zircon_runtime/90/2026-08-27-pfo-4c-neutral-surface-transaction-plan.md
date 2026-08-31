# PFO-4c Neutral Surface Transaction Plan

## 状态

- 日期：2026-08-27
- 范围：Runtime90 PFO-4c，产品viewport surface acquire/target/submission/present owner硬切
- 当前状态：`pfo_4c1_pfo_4c2_source_implemented_static_checks_passed_dynamic_validation_pending`
- 证据边界：本记录来自当前源码、仓库内Unreal RHI/D3D12 viewport实现与既有Runtime90计划重审；不是Cargo、真实WGPU、窗口、PNG、RenderDoc、profile、功耗或benchmark结果。

## 重审时的结构性瓶颈（已完成源码硬切）

产品viewport仍有两套真实surface owner：

1. `graphics::backend::ViewportSurface`约469行，直接持有`wgpu::Surface`、configuration和blit pipeline，自行调用`get_current_texture`、reconfigure与`SurfaceTexture::present`。
2. `zr_rhi_wgpu::production::WgpuSurfaceService`约550行，已经持有真实`wgpu::Surface`/`SurfaceTexture`，并实现generation-qualified `SurfaceSession`、`SurfaceFrameLease`、registry target/view、submission-to-target验证、present/discard终态与有界terminal history。
3. 产品scene经唯一`WgpuRenderDevice`提交，但旧`ViewportSurface`在scene成功后才取得backbuffer、另录一条blit command buffer、另发一个ticket并直接present。重审基线中的稳定直接surface composition因此是scene + surface blit两个packet。

瓶颈不在一次bind-group创建或一次函数调用，而是产品viewport绕过已经存在的neutral surface transaction。继续优化旧`ViewportSurface`会复制format/present-mode协商、frame lease、失败终态与device-generation身份，且无法安全把backbuffer目标并入scene packet。

## Unreal约束与取舍

仓库内`DynamicRHI.h`把command context的finalize与`RHISubmitCommandLists`定义为统一提交边界；viewport backbuffer通过RHI资源返回，`RHIEndDrawingViewport`负责推进present。`D3D12Viewport.cpp`进一步注明平台RHI不应在translation期间隐式发送GPU命令，所有GPU工作应经`RHISubmitCommandLists`；当前present前的flush只是待消除的过渡约束。

Zircon采用同一原则，但不复制D3D12 swapchain或barrier细节：surface session和backbuffer身份归neutral RHI；raw WGPU scene/blit只通过generation-qualified migration recorder录制；queue、flush、ticket、target use与present终态仍由`WgpuRenderDevice`唯一拥有。

## PFO-4c1 最小硬切

1. `ViewportSurface`删除原生`wgpu::Surface`和configuration所有权，只保留完整`Arc<WgpuRenderDevice>`、negotiated `SurfaceSessionReceipt`与过渡blit资源。Drop经neutral owner销毁session。
2. surface创建使用`RenderDevice::create_surface_session`；Outdated/Lost/Suboptimal经`reconfigure_surface_session`替换session receipt；Timeout/Occluded保持无提交的typed outcome。
3. `WgpuNativeSurfaceFrameTarget`以RAII持有精确frame lease和一次轻量native view clone。录制回调只获得已验证owner的借用`wgpu::Device`、当前frame target view与调用方encoder，不获得queue、poll、flush或surface service。
4. scene submit owner在enqueue前验证target属于同一个`WgpuRenderDevice`，在flush前再次验证active `SurfaceFrameId + TextureHandle + TextureViewHandle`，并把scene ticket登记到surface target/view的last-use与submission集合；失败取消Accepted ticket。
5. 产品present顺序固定为`frame-begin poll -> neutral acquire -> scene-tail blit record -> central submit/flush -> neutral present`。prepare、录制或提交失败显式discard lease并保留cleanup错误；提交后present失败保留scene/present同一ticket并尝试discard，RAII Drop只作最后兜底。

PFO-4c1不宣称降低native submission count。它删除双surface owner并让现有第二个packet拥有neutral target/terminal身份，是后续融合的必要基础。

## PFO-4c2 一次提交路径

产品帧在scene录制前acquire neutral surface frame，把surface blit作为terminal product pass追加到direct/compiled scene packet；scene ticket同时登记offscreen资源、product copy、diagnostic tail和surface target。提交成功后neutral present只消费同一个scene ticket，不再自行录制或提交。旧`present_texture`、独立surface packet和对应公共WGPU packet类型已硬删除，防止产品路径退回两次提交。

该源码路径只覆盖`ViewportSurface`直接/编译场景present。retained UI独立绘制、multi-viewport和独立共享UI present仍保留显式packet，不为提交数指标破坏产品语义。

## 源码实施与静态证据（2026-08-27）

1. direct和compiled present入口都严格执行一次`poll -> acquire`；poll receipt注入原frame owner，owner不会二次poll。retry/reconfigure不分配surface ticket，但场景帧仍按既有语义提交。
2. direct尾部顺序索引为`writeback 12846 < product copy 13585 < surface blit 13770 < diagnostic tail 14468 < scene submit 15206`；compiled尾部为`product copy 2905 < surface blit 3164 < diagnostic tail 3563 < encoder finish 4495`。
3. fused native owner顺序为`owner validate 13649 < scene enqueue 13855 < surface target register 14106 < sole flush 14337`；注册或flush失败取消已接纳ticket。
4. 产品direct/compiled旧`present_texture`调用为0，`ViewportSurface`旧方法定义为0，产品surface raw `queue.submit/get_current_texture/configure/native present`为0。`RenderFrameSubmissionReceipt`现在接受`scene == viewport product == present`，仍拒绝foreign owner/generation或任何早于scene/product的ticket。
5. `render_frame_with_pipeline.rs`从995行拆为773行主owner、139行surface presentation owner和92行test owner；`render.rs`786行、compiled submit owner725行、direct scene613行、viewport surface505行、native surface target159行，均低于当前800行门槛。
6. 精确Rust文件已通过`rustfmt --edition 2021 --config skip_children=true`解析格式化，scoped `git diff --check`无空白错误；仅有仓库既有LF/CRLF提示。未运行Cargo或动态GPU验收。

## 算法与性能门槛

- acquire、frame lookup、ticket attach与terminalization保持均摊`O(1)`；不得每帧扫描session或submission history。
- native target view只clone一次轻量WGPU handle；pixel payload不clone、不map、不wait。
- PFO-4c1/PFO-4c2源码结构已经把产品原生surface owner从2套降为1套，raw product `get_current_texture/configure/present`调用降为0，并让direct/compiled surface blit进入scene packet。
- 可融合稳定路径的源码目标为1 submission/frame，但真实次数必须用300帧profile量化CPU frame、GPU pass、submission count、queue latency、present latency、显存与功耗，并与同generation ticket关联；当前不得把源码结构计数写成运行时性能结果。
- 动态验收必须生成真实PNG与RenderDoc capture到`docs/tests/runtime/render`。没有这些数据不得声称瓶颈消失、功耗接近其它引擎经验值或算法达到最优规模。

## 失败与回退边界

- session/frame handle属于错误device、generation、allocator或已terminal时fail closed。
- acquire retry/reconfigure不分配ticket；已acquire但未submit的任何错误都必须discard。
- submit成功后present失败必须携带ticket；surface target retirement继续等待该ticket终态。
- device loss由唯一fault gate terminalizesubmission与surface frame；不得恢复旧`ViewportSurface` raw owner作为回退。
- 若PFO-4c2动态数据证明直接surface融合破坏retained/multi-viewport语义，只回退到PFO-4c1的明确双packet transaction，不恢复第二套surface service。
