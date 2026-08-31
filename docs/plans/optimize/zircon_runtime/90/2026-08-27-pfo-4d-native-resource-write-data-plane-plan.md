# PFO-4d Native Resource / Write Data Plane Hard-Cut Plan

## 状态

- 日期：2026-08-27
- 范围：Runtime90 PFO-4d，`RenderBackend` raw WGPU resource/write data plane 收敛
- 当前状态：`pfo_4d4a_raw_adapter_pfo_4d1t_ui_buffer_pfo_4d4b_queue_authority_pfo_4d4c_bootstrap_queue_pfo_4d4d_graph_pass_queue_pfo_4d4e_runtime_prepare_particle_hgi_vg_queue_free_frame_transaction_pfo_4d4f_frame_queue_fact_pfo_4d4g_outer_backend_queue_owner_pfo_4d4h_scoped_runtime_prepare_gpu_capability_pfo_4d4i_compiled_submission_test_only_inputs_pfo_4d4j_source_cubemap_direct_mip_chain_readback_pfo_4d4k_standalone_diagnostic_submission_owner_pfo_4d4l_pipeline_diagnostic_timeline_owner_pfo_4d4m_realtime_ibl_timing_report_device_input_pfo_4d4n_advanced_plugin_cpu_owner_device_input_pfo_4d4o_runtime_prepare_external_binding_device_epoch_packet_pfo_4d4p_runtime_prepare_persistent_plugin_device_epoch_lifecycle_pfo_4d4q_contact_shadow_graph_executor_pipeline_device_epoch_cache_pfo_4d4r_core_graph_executor_native_pipeline_device_epoch_owner_pfo_4d4s_scene_renderer_core_device_epoch_guard_pfo_4d4t_runtime_prepare_gpu_readback_epoch_guard_pfo_4d4u_graph_execution_typed_device_epoch_owner_pfo_4d4v_generic_compute_pipeline_typed_device_epoch_owner_pfo_4d4w_ambient_occlusion_typed_pipeline_report_epoch_pfo_4d4x_transient_resource_pool_shared_device_epoch_owner_pfo_4d4y_scene_renderer_raw_resource_mutator_scope_pfo_4d4z_render_pass_native_capability_scope_pfo_4d5a_render_pass_native_resource_create_measurement_gate_pfo_4d5b_generic_compute_resource_create_measurement_source_implemented_static_checks_passed_dynamic_validation_pending_pfo_4d5c_pass_factory_coverage_inventory_recorded_pfo_4d5d_planar_filter_resource_create_measurement_source_implemented_static_checks_passed_dynamic_validation_pending_pfo_4d5e_froxel_resource_create_measurement_source_implemented_static_checks_passed_dynamic_validation_pending_pfo_4d5f_sss_resource_create_measurement_source_implemented_static_checks_passed_dynamic_validation_pending_pfo_4d5g_remaining_advanced_owner_dependency_audit_recorded`
- 前置：PFO-0 至 PFO-4c 源码实现与静态检查已完成；动态 WGPU、窗口、PNG、RenderDoc、profile、功耗与 benchmark 仍 pending。
- 证据边界：本文只记录当前工作树源码和仓库内 Unreal/Lumen 参考的结构审计，不把静态调用计数解释为运行时性能数据。

## 结论

当前主要瓶颈不是单个 `write_buffer`、bind group 或 command encoder 的微观成本，而是产品层同时保留两条 GPU 数据面：

1. `Arc<WgpuRenderDevice>` 已经拥有唯一 device generation、fault gate、resource registry、submission service、surface service、diagnostic service、物理显存预算和 neutral batch upload。
2. PFO-4d0 前，`RenderBackend` 又长期保存同一 generation 的 raw `Instance/Adapter/Device/Queue` clone，并把 `Device/Queue` 传入 SceneRenderer、ResourceStreamer、GPU Scene、RDG materializer、IBL、history、UI 与 pipeline cache。PFO-4d0 已删除无 consumer 的 `Instance` clone，但其余 raw 数据面仍在。
3. 因而资源创建、动态更新、瞬态复用和命令提交不在同一个 admission / generation / budget / retirement authority 下。即使 surface 与 scene 已融合为一次 submission，raw resource/write path 仍能绕过该 owner。

正确方向是把产品层改成 Unreal 的 RHI/RDG 分层：持久资源由 generation-qualified RHI resource owner 生命周期管理，图内资源由 RDG descriptor/materializer 管理，CPU 上传以批次附着到图或 submission transaction；产品 pass 只消费 typed resource/view/binding 和 recorder，不取得平台 queue。

## 当前源码量化

扫描范围：`zircon_runtime/src/graphics/**/*.rs`，包含测试源码；生产候选计数按路径过滤，因此只作迁移规模下界，不作验收数字。

| 入口 | 总命中 / 文件 | 生产候选命中 / 文件 | 主要 owner |
|---|---:|---:|---|
| `backend.device` | 419 / 86 | 230 / 63 | transient pool、compiled graph binding、GPU Scene、IBL、pipeline/material、history、UI |
| `backend.queue` | 168 / 43 | 99 / 29 | dynamic uniform/storage writes、texture upload、旧测试/兼容 submit |
| `backend.adapter` | 2 / 2 | 2 / 2 | PFO-4d0 前为 backend caps、pipeline cache identity；当前只剩 pipeline cache identity |
| `backend.instance` | 0 / 0 | 0 / 0 | PFO-4d0 已删除字段；无外部 consumer、无 clone |
| `write_buffer(` | 69 / 49 | 未逐项判定 | GPU Scene、lighting、shadow/probe、post process、history |
| `write_texture(` | 134 / 59 | 未逐项判定 | ResourceStreamer、materialization、environment/UI atlas |
| `create_buffer(` | 122 / 84 | 未逐项判定 | persistent resource、graph materialization、workspace/readback |
| `create_texture(` | 117 / 62 | 未逐项判定 | transient graph、history、asset/environment/UI |

产品 raw `backend.queue.submit` 仍有 12 处源码命中，其中 generic compute / per-pixel product / parallel encoder 属于生产候选；scene-clear 和 UI framebuffer 命中位于测试模块。该计数不能与 PFO-4c 稳定 viewport 一次 submission 目标混为一谈。

## 调用与生命周期分类

### A. 冷路径身份与能力

- `WgpuRenderDeviceContext` 明确是 one-shot ownership handoff，且不提供 native accessor；`WgpuRenderDevice::new` 已在交接时验证 adapter、device limits、requested features 与 queue topology，并生成唯一 `RenderBackendCaps`。
- `RenderBackend::caps` 当前又读取 raw adapter/device 并第二次映射能力，可能与 production neutral ABI 的 fail-closed capability mapper产生分歧。
- raw `Instance` 没有产品 consumer；raw `Adapter` 仅剩 pipeline cache identity。该类可先收敛，不依赖热路径资源迁移。

### B. 持久资源

- ResourceStreamer、mesh/material、history、IBL、UI atlas、GPU Scene 容器直接保存 `wgpu::Buffer/Texture/View/Sampler/BindGroup/Pipeline`。
- 它们需要类似 Unreal `FRenderResource::InitRHI/ReleaseRHI` 的显式 generation 生命周期、last-use ticket 与 deferred retirement；不能仅把 `&wgpu::Device` 改成一个通用 native callback，否则 registry、预算和 device-loss 重建仍被绕过。

### C. RDG 瞬态资源

- `TransientResourcePool` 以 descriptor key 做复用并有 frame/device epoch 基础，但直接创建和保存 raw WGPU resource。
- 目标不是删除池，而是让 pool 保存 generation-qualified physical allocation handle，materializer 在图执行期解析 native view；external/persistent extraction 继续显式注册，禁止把 raw texture 生命周期泄露到图外。

### D. 动态写入

- 当前多个 feature 每帧直接调用 `Queue::write_buffer/write_texture`，提交身份、字节预算、合并策略和资源 last-use 无法统一量化。
- `zr_rhi::RenderDevice` 已提供 `BufferUploadBatch/TextureUploadBatch`，WGPU owner 已校验范围/usage并经唯一 submission service 分配 ticket。这是 PFO-4d1 的基础，不再设计第二套 uploader。
- 高频小写入应先在 feature owner 中按目标 handle + 连续范围合并，整帧交给一个 bounded upload batch；大 payload / 稀疏范围继续使用 staging ring 和 scene encoder copy，避免把所有路径强制成 queue write。

### E. 命令录制

- `WgpuNativeRecorderLease` 只应用于尚未 neutralize 的 pass 编码，允许借用同 owner `Device + Encoder`，不允许 queue/poll/flush。
- resource create、persistent cache、upload 和 retirement 不得通过扩大 recorder lease 权限解决；它们分别归 registry/materializer/upload transaction/completion owner。

## Unreal 与 Lumen 参考约束

1. `dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/DynamicRHI.h` 的 buffer create/lock/unlock 都接收 `FRHICommandListBase`，平台实现位于 Dynamic RHI 下，而非散发 device/queue 给 RenderCore。
2. `dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderResource.h` 用 `InitResource/InitRHI/ReleaseRHI/ReleaseResource` 明确持久资源与 RHI 生命周期，并把跨线程初始化排入 render command owner。
3. `dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h` 用 descriptor 创建 graph-tracked texture/buffer，用 `RegisterExternal*` 区分外部资源，用 `QueueBufferUpload` 把上传数据生命周期延长到 graph execution。
4. `dev/LumenInUE5.5.4WithComputeShader` 是独立 D3D12 compute 复刻，适合核对 Lumen pass/resource/barrier 语义，不作为绕过 Zircon RHI owner 的接口模板；其显式 D3D12 resource/upload/command-list 代码必须在 Zircon 中映射到 neutral resource、RDG 和 submission owner。

## 分阶段硬切

### PFO-4d0 冷 owner 与 capability 单一事实源

- 删除 `RenderBackend` 无 consumer 的 raw `Instance` clone；one-shot context 直接消费 instance。
- `RenderBackend::caps` 只委托 `WgpuRenderDevice` 已验证的 neutral caps，删除第二套 native capability mapper和相关测试分支。
- 更新 Runtime15 F12 guard：保护“唯一 device owner + 不保留重复 instance + caps 单一事实源”，不再保护已被架构替代的旧三字段保活形态。
- 不迁移 raw `Adapter/Device/Queue`，它们仍由后续阶段真实 consumer 阻挡。

#### 源码实施与静态证据（2026-08-27）

1. `RenderBackend` 已删除 raw `wgpu::Instance` 字段；offscreen bootstrap把 instance直接 move进 one-shot `WgpuRenderDeviceContext`，产品 graphics 源码中 `backend.instance` 和 `instance.clone()` 均为0。
2. `RenderBackend::caps` 已删除第二套 adapter/device capability projection，只返回 `WgpuRenderDevice` 在native context校验后生成的 neutral caps clone。外层不会再把 adapter availability误报为已接线的 neutral ABI。
3. Runtime15 F12 source guard 已改为锁定 `Arc<WgpuRenderDevice>`、device profile、neutral caps delegation与shared UI context，并显式拒绝 raw instance字段；不再用人工 owner count保活已被架构替代的字段。
4. 三个精确Rust文件通过`rustfmt --edition 2021 --config skip_children=true --check`；scoped `git diff --check`通过，仅有仓库既有LF/CRLF提示。未运行Cargo或动态GPU验收。

### PFO-4d1 动态写入批次

- 按 feature owner 统计每帧 write count/bytes/ranges，先迁移 GPU Scene、scene uniform、lighting/shadow/probe/post-process 参数。
- 对连续 range 做排序后线性合并，算法上界 `O(r log r)`；已按 offset 产生的 owner 可保持 `O(r)`。禁止逐字节、逐 instance 注册或全资源扫描。
- 一个 frame upload transaction 只生成 bounded batch 和真实 submission ticket；失败必须保持 source frame / generation / handle / byte range 身份。

#### PFO-4d1a-k frame preparation / RDG upload transaction 源码实施与静态证据（2026-08-27）

1. direct 与 compiled 产品帧的 scene uniform 路径不再直接调用 `Queue::write_buffer`。固定 scene uniform 与可选环境 SH9 先写入一个精确预分配的 `Vec<u8>`，再转换成一个共享 `Arc<[u8]>`；两个目标 range 复用同一 payload owner。该 owner 现在只返回待合并的 `WgpuBufferUploadBatch`，不再自行 enqueue 或制造独立 ticket。
2. GPU Scene 已拆为 `prepare -> append -> backend accept -> commit`。mesh build 只产出拥有 payload/range/report/commit metadata 的 `GpuScenePreparedUpload`；direct/compiled frame owner 把 direct ranges 或 staging-ring upload 追加到帧批次。只有批次被唯一 backend 接受且 ticket 已登记后，才清除 dirty primitive/instance/light range、full-upload flag 与 pending free。受理失败或准备对象被丢弃不会丢失下一帧所需更新。
3. shadow atlas、compiled irradiance-volume 参数与 reflection-probe buffer 参数已加入同一帧批次，三个生产段 `queue.write_buffer` 均为 0。shadow active slots 与 stale disabled tail 在 CPU 侧打包为一个连续 slot-buffer range，另加一个 globals-buffer range，因此原先最多 3 个 native shadow writes 的上界降为 2；`last_uploaded_slot_count` 同样延迟到 backend accept 与 ledger 成功之后提交。probe planar/probe-array/header 三个目标共享一个 exact-capacity payload，且在 environment-only provider upgrade 完成后才捕获最终 buffer handle；cubemap texture upload 保留专用 texture 路径。
4. direct/compiled 每帧只调用一次 `enqueue_copy_buffer_upload_batch(frame_buffer_uploads)`，并只以 `RenderFrameSubmissionProducer::FrameBufferUpload` 保存一个真实 Copy ticket。compiled 路径把该接受点延迟到所有 graph stage 成功之后，先合并 graph pass 初始化写入，再登记 ticket 和提交 GPU Scene/shadow CPU 镜像；任一 graph pass 失败都会丢弃尚未接受的 preparation/pass-local 批次。scene packet 仍是唯一 graphics flush owner，不增加 native submit 或 completion poll。一个逻辑批次不等于一个驱动写：不同目标 buffer/range 仍由 WGPU owner 执行各自写入。
5. scene/shadow payload packing 为 `O(payload bytes)` 时间与内存；GPU Scene 保留 dirty-range 排序合并的 `O(r log r)` 上界和大 payload staging-copy 分支，批次拼接只是 `Vec::append` 的所有权移动，不 clone payload 或 native buffer handle。稳定帧的逻辑 upload packet/ticket 从 scene constants 与 GPU Scene 两个收敛为一个；真实 p50/p95/p99、功耗与驱动写数量仍必须由动态 profile 证明。
6. compiled irradiance-volume 的 frame selection/prepare 只保留 outer frame owner 一处；原 graph executor 不再重复扫描 visible meshes/volumes、查询 streamer 或覆盖 pipeline state，执行复杂度收敛为只验证 pass identity/context 的 `O(1)` publication/order token。
7. RDG pass context 新增独占的 pre-submit upload batch。每个 executor 只追加 feature-owned batch；成功的 `RecordedGraphPass` 随 profiling 结果返回，serial/parallel stage owner 按 compiled topology/bucket order 用 `Vec::append` 合并。并行 worker 之间不共享 `Mutex`，合并为 `O(total uploads)`，并且该 API 明确限制为“一帧一个 CPU producer、所有 graph command buffer 执行前可见”的初始化写入。
8. light-grid 是第一个 graph-time consumer：params/z-bin/tile-mask 从每帧 3 次直接 `queue.write_buffer` 收敛为 1 个 exact-capacity `Arc<[u8]>` payload 和最多 3 个目标 range，生产段 raw buffer write 为 0。该切片把 scene-renderer 生产候选 `queue.write_buffer` 源码计数由 35/29 文件降为 32/28 文件。
9. TAA resolve、camera velocity 与 bloom 是下一组经调用图确认的单 CPU producer 参数。三者只由 compiled RDG context 调用；enabled 路径各返回一个整段 immutable upload，disabled/camera-cut 路径返回空批次，context 在 pass 成功结果内追加。`WgpuBufferUpload::from_bytes` 统一整段 payload 的 exact-length `Arc<[u8]>` 构造，未扩大 queue/poll/submit 权限。三处生产 raw buffer write 均由 1 降为 0，scene-renderer 生产候选计数进一步降为 29/25 文件。
10. exposure histogram/resolve 原先各自构造并写入同一个 `exposure_params_buffer`。现在 outer compiled frame owner 每帧只构造一次参数并追加一个 upload，两个 graph pass 只消费已绑定的持久 buffer；重复 CPU producer 和两次直接写均已删除。`EXPOSURE_ADAPTATION_DELTA_SECONDS = 1/60` 仍是 P0 正确性欠账：authoritative `FrameTimeSnapshot::raw_real_delta()` 尚未进入 `RenderFrameExtract/ViewportRenderFrame`，本切片没有用局部 wall clock 或另一个固定值掩盖该问题。
11. HZB stats reset 已从一次 queue write 改成同一 command encoder 内、dispatch 前的 `clear_buffer`，保持图命令顺序并删除独立 CPU upload。clustered lighting 的 directional-light rows 与 cluster params 共用一个 exact-capacity payload、最多两个目标 range；color-LUT、DOF prepare 和 half-resolution composite 参数均返回单 producer upload，disabled/invalid 分支返回空批次。上述切片将当时的 scene-renderer 候选计数依次收敛为 27/24、26/23、24/22 和 21/19。
12. 九个 full-screen/post-process/SSR pass 原先在录制热路径分别创建一个 432-byte `PostProcessParams` buffer、创建 bind group 并直接 queue 写。现在 `FullScenePostProcessResources` 初始化期拥有九个独立持久参数槽，总 payload capacity 3,888 bytes；执行阶段只创建 immutable upload。单个 coarse-reflection graph pass 即使录制多个 mip render pass，也只准备一次参数 payload。该切片消除完整活跃图每帧最多九次 buffer allocation 和九次直接驱动写，同时保持每个目标一帧一个 CPU producer。
13. final post-process pass 的 reflection probe、hybrid-GI probe 与 hybrid-GI trace-region 数据先按有效 count 截断，再写入一个精确容量 `Arc<[u8]>`；最多三个目标 range 共享该 payload，三类计数都为零时不分配 payload、不产生空写。随后 432-byte pass params 追加到同一个 `WgpuBufferUploadBatch`。旧 `write_*` 模块已硬删除，不保留 raw queue 双轨。
14. SSAO 参数仍在 graph resource binding 阶段准备，因为该阶段决定资源是否实际存在；但 owner 只向 `frame_buffer_uploads` 追加完整 buffer upload，不再接收 queue。后续 materialization/binding/graph validation 失败会丢弃尚未受理的整个帧批次，不留下 GPU side effect。当前后处理生产代码的直接 `queue.write_buffer` 为 0。
15. PFO-4d1h 新增/修改的精确 Rust 文件已执行 `rustfmt --edition 2021 --config skip_children=true`；scoped `git diff --check` 通过，仅有仓库既有 LF/CRLF 提示。该阶段的新鲜工作树扫描在排除 `tests.rs`/测试目录后记录 scene-renderer 生产候选为 13 次直接写、11 个文件；该数字是源码迁移规模证据，不是驱动调用数。
16. PFO-4d1i 把 HZB occlusion params workspace 拆成 `prepare -> append -> backend/ledger accept -> commit`。prepare 只在 committed `args_count` 不匹配时返回一个 32-byte upload 和 revision-qualified token，不再接收 queue，也不提前发布 `initialized/args_count`；unchanged 路径返回空 upload/token。token 随 `RecordedGraphPass` 按 topology/bucket order 线性合并，帧 owner 在唯一 upload ticket 登记后回交 culler。buffer create 统计现在只在真实 workspace map 插入时增加。静态顺序检查和 scoped diff check 通过，scene-renderer 生产候选进一步降为 12 次/10 文件；未运行 Cargo、真实 WGPU、PNG、RenderDoc、profile 或功耗验收。
17. PFO-4d1j 删除 mesh indirect prepare 的直接 queue side effect。九个 phase 的 args 与 compaction metadata 各自保留 committed/staged shadow、staged revision 与 committed buffer revision；prepare 生成共享 immutable payload ranges 和 token，backend/ledger 受理后才交换 shadow。前一失败帧创建但未受理的新 buffer 会因 revision 不匹配在下一帧重新 full upload。精确 dirty-range 扫描仍为 `O(n)`，所有 range 归入已有的一个 frame ticket；未经真实 profile 不添加 gap/byte 魔数。三个相关生产模块的 queue/direct write 均为 0，新鲜扫描把 scene-renderer 生产候选降为 11 次/9 文件。静态格式、顺序与 diff check 通过；Cargo、WGPU、PNG、RenderDoc、规模 profile 和功耗仍 pending。
18. PFO-4d1k 按 Unreal Skinning Scene Extension 的 scene-global growable storage 方向硬切 skinned palette 数据面。原 `2 * live skinned instances` 个固定 16,400-byte buffer、每实例两次 queue write 和每 draw 专用 GPU Scene bind group 已删除；GPU Scene 现在拥有两个 grow-only arena buffer，按 active joint 矩阵紧凑打包，并由 192-byte `GpuInstanceData` 的 current/previous base+count 间接寻址。非空帧只产生一个连续 palette payload，slot/span 仅在 scene success 后与 previous history 一起提交。skinning 生产模块的直接 queue write 从 2 降为 0，draw-owned palette buffer/bind-group override 为 0。当前共享工作树的新鲜 scene-renderer 扫描为 12 次直接写/11 个非测试生产文件；该绝对数包含本切片之外同时出现的 UI 写入点，因此本切片只声明自身 2->0，不伪造全局单调降幅。源码格式、ABI offset、旧符号与 direct-write 静态检查通过；动态 WGPU、PNG、RenderDoc、1K/10K profile、内存和功耗仍 pending。
19. PFO-4d1l 保留 Unreal RDG custom-viewport ClearQuad 对齐的 region triangle策略，但删除 clear feature 的 raw queue权限。color clear现在准备一个16-byte immutable buffer upload，depth-only/no-clear/empty-region返回空 batch；prepared payload随 `RenderGraphStageExecution` 在所有graph stage成功后才进入唯一 `FrameBufferUpload` ticket。因而后续stage失败会同时丢弃command encoder与未受理参数，不再留下queue side effect。源码顺序、queue权限、rustfmt与diff静态检查通过，新鲜scene-renderer非测试扫描由本切片前12次/11文件降为11次/10文件；Cargo、WGPU、PNG、RenderDoc、profile和功耗仍pending，Render01 `PERF-MVP-394` full-target clear融合不在本切片内。
20. PFO-4d1m 将 retained exposure history invalidation 改为 `request -> prepare -> commit`。new/resize rebuild 继续由 mapped creation 写默认值，不再重复提交两个 range；camera-cut reset 复用一个 payload、向同一 frame batch 追加 read/write 两个 range，并只在唯一 `FrameBufferUpload` ticket 受理且写入 ledger 后清除 pending。graph/admission 失败会保留 intent。history 生产代码的 queue write 从2降为0，新鲜 scene-renderer 非测试扫描为9次/9文件；静态格式、顺序与diff检查通过，Cargo、WGPU、产品readback、PNG、RenderDoc、300帧profile和功耗仍pending。
21. PFO-4d1n 将static environment cubemap的dynamic upload收敛到immutable artifact、一个grow-only staging buffer和现有`FrameBufferUpload`。artifact现在始终包含source/PMREM/irradiance，no-IEM预编码1x1 black cube；renderer先验证完整布局再rebind，删除render-thread float encode与逐face/mip texture-write回退。changed key只追加一个共享staging range并保留每mip encoder copy，stable key为0 payload/0 copy，upload key只在scene submit成功后commit。hot staging raw buffer write从1降为0，新鲜scene-renderer直接buffer write为8次/8文件；cold fallback texture初始化归PFO-4d2。静态格式、顺序与diff检查通过，Cargo、WGPU、PNG、RenderDoc、profile和功耗仍pending。
22. PFO-4d1o 将scene SH9默认值绑定到buffer mapped creation，保留`UNIFORM | COPY_DST`供动态frame batch更新，并删除构造器唯一raw buffer write。新鲜scene-renderer直接buffer write从8次/8文件降为7次/7文件。该切片只收敛cold queue权限，不把per-renderer neutral cubemap/BRDF LUT/sampler重复创建误报为已修复；generation-local shared system-texture owner明确交给PFO-4d2a。静态格式/source/diff检查通过，Cargo与WGPU仍pending。
23. PFO-4d1p 将SSS参数准备按Unreal RDG边界收敛为setup唯一producer。80-byte params与512-byte profile table成为transient graph uniform，setup/scatter通过显式write-read版本边消费；每个启用camera只resolve profile table一次、inverse matrix一次并向现有frame upload追加一个592-byte共享payload的两个range。indirect args改为encoder ordered clear，首个setup workgroup恢复Y/Z=1且active tile只原子累加X。SSS raw queue write、每帧3次`create_buffer_init`、profile clone、第二次resolver/inverse均删除；scene-renderer非测试raw buffer write从7次/7文件降为6次/6文件，剩余全部在UI。静态格式、source、图契约与diff检查通过，Cargo、WGPU、PNG、RenderDoc、profile与功耗仍pending。
24. PFO-4d1q 按 Unreal `FSceneViewFamily::Time` / eye-adaptation `DeltaWorldTime` 的单一帧时间权威补齐 Zircon 帧契约。`RuntimeDynamicSession` 在唯一 `tick_time` 后保存 outer frame index/raw-real delta，缓存命中后覆盖到 `RenderFrameExtract::timing`，不把逐帧时间塞入 scene cache key；compiled exposure params 使用该 delta，固定 `1/60` 常量已删除。新鲜源码计数为 fixed delta 0、tick capture 1、cache overlay 1、exposure consumer 1、cache timing mention 0、pass-local `Instant` 0；8个显式 extract 初始化器均有 timing/default或struct update。rustfmt与scoped diff通过；动态曝光序列PNG、RenderDoc、profile与功耗仍pending，不从静态计数推断性能收益。
25. PFO-4d1r 将唯一 `FullscreenPassParameterBindings` 实例按实际 immutable built-in plan 硬切为mapped initialization。motion-vector tile-max固定参数只在构造时序列化一次，buffer仅保留`UNIFORM` usage；raw Queue参数、`COPY_DST`、mutable upload bytes、String/discriminant layout副本、layout-match分支与未使用动态`write`入口全部删除。新鲜源码计数为mapped init 1、Queue/write_buffer/COPY_DST/dynamic write/旧queue构造参数/external writes均0，owner从225行降至154行。静态格式、源码契约与diff通过；这是权限/生命周期收敛，不宣称cold constructor或frame-time收益，动态WGPU与产品验收仍pending。
26. PFO-4d1s 将 GPU Scene morph/VG sideband 从独立 raw Queue 写入硬切到现有 scene upload transaction。两个 owner 返回 immutable batch + report + move-only commit token，mesh build 以本帧 prospective VG counts 准备 remap 参数，再把 sideband 附着到唯一 `GpuScenePreparedUpload`；shadow、scene counts 与 full-upload intent 只在 scene success 后提交。grow 后失败会保留每个新物理 buffer 的 full-upload intent，下一帧即使回到旧 shadow 也会重新初始化。两轮代码审查发现旧 frame token 越序和 caller-supplied scene 可伪造跨场景 attachment 两个缺口；现以每类 sideband 单未决 reservation 持有到 commit/drop，并让 core/sideband preparation 保留同一个不可伪造 scene identity，attachment API 不再接收 scene 参数，batch 离开本地所有权前与 commit 时均校验目标。连续 changed-run 仍为单次 `O(n)` 扫描，不添加未经 profile 证明的阈值。聚焦源码计数为目标生产 raw buffer write 0、sideband Queue 参数 0、旧即时 API/helper 0、typed product attachment 2、combined-frame abort/retry test 2、overlap rejection test 2、foreign-scene rejection test 2；动态 WGPU、PNG、RenderDoc、1K/10K profile、VRAM与功耗仍 pending。
27. PFO-4d1t 将 graphics 最后 6 个生产 `Queue::write_buffer`（UI generic/image、bitmap atlas instance/viewport、SDF vertex/material）收敛到同一帧 `WgpuBufferUploadBatch`。renderer 级单未决 generation fence 让候选 hash/retained plan 只有在 backend 与 ledger 接受后才能参与复用；drop/错误会让下一帧对 6 个 owner 强制全量重传。direct 在 UI record 后再接受唯一 frame batch，compiled 将 move-only commit token 从 pass 移交到 stage/outer frame 后提交。算法保持 `O(payload bytes + invalidated segments)`，不添加无 profile 依据的排序或逐顶点注册。独立复审发现 direct 的 UI attach/upload admission/ledger 三个失败出口未 defer timer 或释放 realtime-IBL scheduler token；现已与其他 direct/compiled 失败路径统一清理。复核确认三个失败出口都会 defer timer 并 abort realtime-IBL，prepared token drop 保留 full retry，三个 product framebuffer 也会在 draw 前 flush neutral upload，未发现新 correctness issue。聚焦源码计数为 UI production raw buffer write 0、neutral upload site 6、direct/compiled attach+commit 各1、abort/overlap/append/foreign-transaction guard source test 各1；精确 rustfmt 与 scoped diff 静态检查通过，Cargo、WGPU、PNG、RenderDoc、1K/10K/100K profile、VRAM与功耗仍 pending。UI atlas texture Queue 权限归 PFO-4d2 后续切片。
28. PFO-4d4b 删除两条无行为的产品raw queue权限链。`ensure_scene_resources -> ensure_material_for_frame -> ensure_material_internal`不再接收queue，材质/纹理上传仍服从既有frame transaction；`GpuPassTimer::try_new_product`改为消费唯一`WgpuRenderDevice`在启动时固化的timestamp period，不再读取raw queue。Legacy standalone UI timer不在本切片迁移，以免与scene中央query delivery router争抢结果。failing-first 0/5转为扩展合同7/7；device/timer/resource-prepare/material owner为798/615/726/945行，material本次净减行。其余core构造和upload diagnostics raw queue consumer、Cargo/WGPU/PNG/RDC/profile/功耗仍pending。

### PFO-4d2 持久资源生命周期

- 为 buffer/texture/view/sampler/layout/bind-group/pipeline 建立 generation-qualified materialization owner与显式 release/retire。
- ResourceStreamer、history、IBL、UI atlas、GPU Scene 先迁移；旧 raw resource 字段和 facade 同批删除，不保留双轨缓存。
- device loss 后只允许从 semantic/cooked descriptor 重建，不把 stale native object 重新发布为 ready。

#### PFO-4d2a0 环境回退物理身份源码实施与静态证据（2026-08-27）

1. 完整结构重审确认 system texture 必须是 WGPU device-generation owner，而不能是进程全局 raw WGPU singleton，也不能通过开放 registry native getter 形成新旁路。目标 owner 在完整候选 bundle 上传受理后原子发布 immutable lease，scene/RDG consumer 不取得 queue/poll/flush 权限。
2. 作为 generation owner 前置切片，`SceneEnvironmentCubemap::fallback` 的 source/specular/irradiance 三个槽已共享同一张 1x1x6 black cube 与同一个 cube view。冷回退 texture create `3 -> 1`、view create `3 -> 1`、48-byte direct texture upload `3 -> 1`，不改变绑定布局。
3. 环境 filtering sampler 的 descriptor 与 cubemap 尺寸/mip/content 无关，现只在 owner 冷构造时创建；真实环境 rebind 的 sampler create/publish 均从每次1降为0。不同语义的 source/specular/irradiance 动态纹理仍分别创建并走已有 staging/frame transaction，没有被错误合并。
4. 局部格式、fallback/dynamic 源码计数、handle clone 既有用法和 scoped diff 静态检查通过。Cargo、真实 WGPU、PNG、RenderDoc、profile与功耗未运行；BRDF/black cube/sampler 跨 core 的 generation-local owner 和 shadow fallback 迁移仍 pending。
5. PFO-4d2a1 将同一device generation中真实存在的shadow重复consumer迁移到只读binding lease。scene bundle从已初始化black cube/BRDF LUT、稳定sampler和SH9 buffer发布拥有handle clone的lease；`ShadowMapRenderer`不再接收device或创建环境资源，三个cube槽共享同一black-cube view。full-scene shadow cold增量由4 texture + 4 view + 1 sampler + 1 buffer降为0。atlas slot camera uniform/bind-group仍按slot创建，归后续独立hot-path优化。静态格式、constructor/binding计数与diff检查通过，动态证据仍pending。
6. PFO-4d2b 将上述atlas slot热路径收敛为grow-only持久workspace。一个uniform buffer按device alignment分槽，capacity按2倍增长且只在增长时重建固定offset bind groups；outer frame把全部active `SceneUniform`打包为一个aligned payload/一个upload range后进入现有frame transaction。graph recorder不再接收queue或逐slot创建buffer/bind-group，workspace缺失显式报错。稳态每帧native buffer/bind-group create由N/N降为0/0，逻辑upload range由N个mapped initialization收敛为1；真实driver、1/4/16/64 slot profile与功耗仍待动态验证。
7. PFO-4d2c 修复reflection provider generation替换后的跨模块发布缺口。environment-only placeholder升级完成后，outer scene frame立即把最终`ReflectionProbeGpuBindings`发布给deferred owner，再执行generic environment variant切换与mesh command cache清理；deferred lighting不再因旧`Arc` lease仍合法而静默采样1x1/单slot占位资源。该切片未新增queue/native create；动态provider注入PNG、WGPU validation与RenderDoc identity仍待验收。forward receiver bind-group缓存没有在缺少真实profile时实施。
8. PFO-4d2d 将lightmap atlas current判定从semantic asset id提升为prepared revision + physical lease identity。streamer从同一prepared row返回revision与resource，consumer只有在asset/revision/`Arc::ptr_eq`全部相同时才早退；同ID热重载或同revision residency替换都会重新发布到forward/deferred。ID-only early return由1降为0，missing prepared row不再借fallback掩盖。动态hot reload/residency PNG、WGPU与RenderDoc identity仍待验收。
9. PFO-4d2e 为forward receiver cache候选补齐先测量门槛。`MeshPipelineCache`按成功帧分别记录standard/full bind-group native create次数，并为standard create与full params+binding prepare提供独立CPU scope；direct/compiled入口清零、唯一成功出口上报。该切片不改变创建、缓存或resource owner。300帧Windows产品profile、RenderDoc/API核对与缓存立项判定仍pending，未取得数据前禁止宣称它是瓶颈。

### PFO-4d3 RDG 瞬态池

- `TransientResourcePool` 改存 physical allocation handle + descriptor key + generation + last-use ticket；materialization 和 alias validation 保持图 owner。
- acquire/release 均摊 `O(1)`，frame-end retirement 只遍历本帧 touched / pending 集合，不扫描完整历史。
- external import、persistent extraction、history copy 分开建模，禁止用 lifetime extension 隐式逃逸 graph。

#### PFO-4d3a generation-qualified allocation lease 源码实施与静态证据（2026-08-27）

1. 完整结构重审确认 production `WgpuResourceRegistry` 已是持久资源的唯一 generation owner，但 transitional native scene recorder尚不能解析neutral handle；本切片没有建立第二个全局registry，而是把pool与graph execution之间的裸对象交接收敛为move-only graph-local allocation lease。
2. texture/buffer lease现在携带device epoch、descriptor key、完整descriptor、native allocation、byte size、last-used frame和optional last-use ticket；texture额外携带sampled identity。abort只回收无ticket lease，成功submit先校验ticket epoch再stamp exact scene ticket，Completed才清ticket进入free bucket，异常terminal/query error继续fail-closed丢弃。
3. `RenderGraphExecutionResources`的owned transient maps改存allocation lease，旧owned descriptor/identity旁表删除；external texture/buffer imports继续使用独立map，persistent extraction继续是独立acquire class并在提交前完成history copy，没有通过延长lifetime隐式逃逸graph。
4. 同期修复descriptor compatibility缺口：pool key、compiler alias validation和native WGPU create现在都包含`TextureDesc.view_formats`集合，禁止复用创建时未声明所需view format的物理纹理。
5. pool entry裸WGPU字段计数为0，owned texture/buffer lease map各1，external raw texture map保留1；主pool文件从1009行拆分到707行，allocation/test owner分别204/306行。rustfmt、source contract与scoped diff静态检查通过。
6. 本切片不改变`BTreeMap`、全历史stale/budget扫描或逐资源pending status查询，不宣称O(1)或性能收益。Windows WGPU、PNG、RenderDoc、300帧profile、VRAM与功耗仍pending；只有真实profile证明成本后才允许实施hash buckets、ticket buckets或age wheel。

#### PFO-4d3b transient pool measurement foundation 源码实施与静态证据（2026-08-27）

1. 在任何容器或淘汰算法改动前，`collect_completed_submissions`与`end_frame`分别加入CPU scope；报告新增纹理/缓冲completion status query、stale scan、budget accounting和over-budget sort candidate共8个work counter。
2. 所有计数复用已有执行点：status lookup前累加、`retain`访问桶时累加原长度、预算fold复用retained count、排序复用candidate vector长度；没有为观测再扫描pool，也没有改变resource identity、submission ticket、淘汰顺序或预算语义。
3. 8个counter全部进入既有render diagnostics store，并以非零fixture验证字段到series/tag的映射。旧构造调用通过zero default保持兼容，未建立第二条诊断数据面。
4. 定向rustfmt与scoped diff check通过；相关production/test owner分别为618/759行，report/pool owner分别为660/753行，均低于现有800行source contract。
5. 当前只具备采样基础，没有Cargo、WGPU、PNG、RenderDoc、300帧profile、allocator/RSS、GPU timing、VRAM或功耗数据。`BTreeMap`、全历史扫描、逐资源status query继续保持原实现；只有产品采样证明成本后才选择ticket coalescing、age bucket、retained-byte ledger或bounded eviction queue。

### PFO-4d4 raw field 删除

- pipeline cache identity迁入 production WGPU resource/pipeline owner后删除 raw `Adapter`。
- 所有产品资源与写入 consumer迁移后删除 `RenderBackend.device/queue`。
- 产品源码 `backend.instance/adapter/device/queue` 均为0；raw `queue.submit/poll` 只允许 production submission service 和明确隔离的独占测试 backend。

#### PFO-4d4a outer raw Adapter 硬切源码实施与静态证据（2026-08-27）

1. production `WgpuRenderDevice`继续唯一拥有device generation的native adapter；bootstrap不再clone adapter，而是把local adapter直接move进`WgpuRenderDeviceContext`。outer `RenderBackend`的raw adapter字段和`backend.adapter` consumer均降为0。
2. 唯一生产consumer原为mesh driver pipeline-cache identity。`RuntimePipelineCache`、`MeshPipelineCache`与scene-core构造链现改吃`RenderDeviceProfile::adapter()`的neutral `RenderAdapterFacts`，没有新增native accessor或第二套identity DTO。
3. 已核对WGPU 29.0.3源码：Vulkan key只使用vendor/device并格式化为`wgpu_pipeline_cache_vulkan_<vendor>_<device>`。本实现保持文件名逐字兼容，非Vulkan仍返回None；focused source test覆盖Vulkan exact key与DX12 disabled branch。
4. 定向rustfmt和scoped diff check通过；graphics backend/scene/pipeline范围`AdapterInfo`构造参数、`adapter.clone()`与outer raw adapter字段均为0。Cargo、真实Vulkan cache warm reuse、WGPU、PNG、RenderDoc与profile仍pending，不宣称性能收益。
5. outer raw `Device/Queue`仍有真实资源/上传consumer，按PFO-4d1/2后续阶段继续迁移；本切片不把PFO-4d4或完整raw data-plane标为完成。

#### PFO-4d4c renderer bootstrap Queue authority 结构复审与实施计划（2026-08-31）

1. current-source调用图确认，`SceneRendererCore::new_with_icon_source`的raw queue只有两条下游：screen-space UI text构造只是未读取的历史透传；mesh cache构造只把queue交给`LightCookieAtlasResources::new`。后者在renderer启动时分配4,194,304-byte全白临时块并执行一次`Queue::write_texture`，是这条bootstrap权限链唯一真实consumer。
2. `rendering.light_cookies` feature只有在extract存在cookie时才编译`cookie.atlas_build`，该pass声明写`advanced_lighting.cookie_atlas`并先于opaque/alpha-mask/transparent/deferred-lighting读取者；WGSL在`cookie_misc.x == 0`时直接返回1而不采样atlas。atlas build自身以白色render-attachment clear开始，因此compiled首次cookie采样已有同packet的write-before-read，不依赖generation-start CPU upload。复审同时发现direct renderer复用cookie-aware light packing却不执行plugin executor；旧白图只能把cookie错误退化为1。direct路径必须在同一scene encoder中显式调用同一rebuild，并位于scene/mesh consumer之前，不能靠未初始化纹理或继续保留bootstrap上传。
3. Unreal `LightFunctionAtlas.cpp`同样只在运行时系统请求atlas时生成资源；`AllocateTexture2DAtlas`在RDG内创建texture，`RenderAtlasSlots`以graph pass写入，并用slot index 0禁用采样。适用约束是“功能启用后由图内producer初始化、禁用态不采样”，不是在renderer bootstrap取得native queue并上传整张默认纹理。
4. 本切片将删除启动全白payload、`COPY_DST`用途和initial-upload profile counter，并从`LightCookieAtlasResources -> MeshPipelineCache -> SceneRendererCore -> SceneRenderer`以及screen-space UI构造链删除raw queue参数。compiled继续由RDG executor构建atlas；direct只在cookie非空时把同一个rebuild录入现有scene encoder，并在成功提交后发布同一profile。保留atlas build的白色clear，暂不改变每次rebuild的full clear算法。
5. 复杂度目标：bootstrap CPU临时分配与上传payload由4 MiB降为0，raw queue参数透传由该生产构造链降为0；这只是可由源码证明的工作量删除，不是实测startup、GPU、功耗或帧时间收益。动态验收仍须使用受管Windows WGPU、RenderDoc与profile证据。
6. 实施门：source contract必须证明生产构造链不含`wgpu::Queue`、light-cookie资源不含`queue.write_texture`/initial upload常量、graph producer仍保留white clear且feature仍声明external texture write-before-read；direct producer必须位于scene preparation/consumer与submit之前。随后执行精确rustfmt、scoped diff check与locked metadata；受管Cargo仍按milestone testing stage处理。

实施结果：

1. `LightCookieAtlasResources::new`已删除4,194,304-byte全白CPU payload、启动期`Queue::write_texture`、`COPY_DST`用途和`initial_white_upload` profile counter。atlas构造仍保留资源创建计时，实际build继续记录frame-plan、encode和written bytes。
2. `SceneRenderer -> SceneRendererCore -> MeshPipelineCache -> LightCookieAtlasResources`产品构造链及`ScreenSpaceUiRenderer -> ScreenSpaceUiTextSystem`产品构造链已不再接收raw queue；测试fixture可暂时保留未读取的`_queue`参数，但不能把它传入产品owner。
3. compiled路径继续由`cookie.atlas_build`的external texture write生产atlas。direct路径仅在本帧存在cookie时调用同一个`rebuild`，录入既有scene encoder并位于mesh consumer与唯一scene submit之前；没有增加native submission。该pass有独立`direct_light_cookie_atlas` GPU timer，成功提交后才发布light-cookie profile。
4. source contract 14/14通过，覆盖启动上传删除、五段产品构造链、RDG external write、无cookie不采样、direct write-before-read/submit和accepted-profile顺序。精确rustfmt、scoped diff check、locked metadata与结构行数检查通过；主direct frame owner拆出39行测试后为799行，未跨越1000行强制拆分门。
5. 本结果只证明源码结构：没有受管Cargo/真实WGPU、PNG、RDC、300帧profile、VRAM、功耗或startup/frame-time数据；因此PFO-4d4c仍是`source_implemented_static_checks_passed_dynamic_validation_pending`，不标accepted milestone，也不声称4 MiB payload删除等于已测性能收益。

#### PFO-4d4d compiled graph pass Queue authority 结构复审与实施计划（2026-08-31）

1. current-source从`CompiledSceneGraphStageContext -> RenderGraphPassFrameServices -> execute_graph_pass -> RenderPassGpuExecutionContext`追踪raw queue。graph executor中的实际读取只有两处：screen-space UI `record`接收后完全未读取；particle transparent把32-byte camera/intensity参数直接`queue.write_buffer`到持久uniform。其余post-process、HZB、subsurface、half-resolution transparency等pass已经使用`append_pre_submit_buffer_uploads`，证明frame-local neutral upload packet是现有主路径而非新旁路。
2. 当前`RecordedGraphPass`已经拥有`WgpuBufferUploadBatch`。executor成功才把batch合并到`RenderGraphStageExecution`，graph全部成功后再合并到唯一frame resource upload packet，并在scene command buffers前取得producer ticket；executor错误会在形成`RecordedGraphPass`前丢弃batch。这一现有事务边界足以承载粒子参数，不需要插件持有native queue。
3. Unreal `RenderGraphBuilder.h`的`FRDGBuilder::QueueBufferUpload`明确把resource lifetime延长并在pass执行前上传；Unity Graphics的`ComputeCommandBuffer`/`IComputeCommandBuffer`也把`SetBufferData`暴露为command-buffer能力，而不是把native queue交给feature。Zircon采用“pass upload recorder -> frame packet -> generation-qualified submission owner”，与二者共同的录制边界一致；由于WGPU queue write发生在packet flush而非encoder命令内部，必须保持每个目标buffer每帧单一CPU producer，不能把它误用为多pass版本化uniform。
4. 本切片新增可复用的`RenderPassBufferUploadRecorder` facade：只允许把一个buffer、offset和不可变bytes登记到pass-owned batch，不公开queue、submit或poll。particle transparent通过该capability登记32-byte参数；UI删除无行为queue参数。随后从compiled stage context、frame services、graph-pass executor和product `RenderPassGpuExecutionContext`构造链删除raw queue。
5. API深度：recorder属于`zircon_runtime::graphics`的render-pass execution contract，可被后续外部render plugin复用；`zr_rhi_wgpu::WgpuBufferUploadBatch`仍留在runtime内部，particles plugin无需新增底层RHI crate依赖。没有旧queue兼容分支；`cfg(test)` fixture构造器可保留未读取queue参数以避免本切片机械改写大量GPU测试，但不得传入product context。
6. 正确性门：failing-first contract必须覆盖product graph context/service/constructor无`wgpu::Queue`、UI record无queue、particle draw context无queue、transparent renderer无`queue.write_buffer`并通过recorder登记、frame upload仍先于scene submit。精确rustfmt、scoped diff、locked metadata与结构预算属于静态证据；受管Cargo/WGPU、PNG/RDC和性能功耗继续按milestone testing stage执行。

实施结果：

1. `RenderPassBufferUploadRecorder`已作为`zircon_runtime::graphics`公开capability落地；它只把buffer/offset/bytes复制进pass-owned `WgpuBufferUploadBatch`，不公开queue、submit、poll或底层RHI batch类型。particles runtime继续只依赖`zircon_runtime`，未新增`zr_rhi_wgpu`依赖。
2. particle transparent的32-byte camera/intensity uniform已从`queue.write_buffer`切到recorder，并沿`ParticleGpuTransparentDrawContext -> executor -> runtime owner -> backend -> transparent renderer`传递。成功pass按拓扑合并upload；任一executor/graph错误会在frame packet enqueue前丢弃该写入，修复了旧raw queue write无法随失败帧回滚的事务缺口。
3. screen-space UI record删除未读取queue；`CompiledSceneGraphStageContext`、`RenderGraphPassFrameServices`、`execute_graph_stage/execute_graph_pass`和product `RenderPassGpuExecutionContext::new`均删除native queue。scene-pass helper同步删除queue透传，compiled graph pass产品执行面raw queue降为0；`new_for_test`只保留未读取`_queue` fixture参数。
4. failing-first静态合同由1/9转为10/10；随后审计发现Hybrid GI `scene_depth_handoff`仍从`gpu.queue`写两个经过window校验的scene buffer packet，已补为同一`RenderPassBufferUploadRecorder`，并新增executor级合同防止该旁路回归。精确rustfmt、scoped diff、locked metadata、public re-export和结构行数检查通过。
5. particles simulation `execute_frame`仍用raw queue写emitter/counter buffer，compiled submit/readback owner也仍需queue；它们不属于本次graph-pass录制权限结论，继续由后续PFO-4d切片按完整producer/rollback语义迁移。没有受管Cargo、真实WGPU、PNG/RDC、profile、VRAM或功耗证据，本切片状态为`source_implemented_static_checks_passed_dynamic_validation_pending`。

#### PFO-4d4e RuntimePrepare write owner 审计边界（2026-08-31）

1. RuntimePrepare 的 raw queue 写入不是一个同质路径：particles `execute_frame` 在 compute dispatch 前写 emitter/counter；Hybrid GI `execute_prepare` 还写 completion params、radiance-cache stage params、global-SDF page table；Virtual Geometry 保留独立 uploader params owner。它们都发生在 graph pass 录制前，但 producer 生命周期、目标 buffer 和失败回滚责任不同。
2. `RuntimePrepareCollectorContext` 当前只把 encoder/readback/external binding 传给 collector，没有 frame-local upload packet 回传通道；直接删除 queue 会使这些 compute 参数在 dispatch 前没有可排序 producer。正确下一步是把 `WgpuBufferUploadBatch` 作为 RuntimePrepare 的事务输出，和 compiled graph 一样在唯一 scene command submission 前取得 Copy ticket，并在 collector/graph/materialization 失败时整包丢弃。
3. 粒子、Hybrid GI、Virtual Geometry 必须按独立 producer 迁移并保持每个目标 buffer 每帧单一 CPU producer；不能把 runtime-prepare 参数写入与 graph-pass uniform 或 compiled readback queue 混成一个 API。compiled submit/readback 的 queue 仍是 readback/surface completion owner，暂不改写。
4. 本审计未改变 product 行为，仅明确下一切片的接口与回滚约束；未执行受管 Cargo、真实 WGPU、PNG/RDC、profile、VRAM 或功耗验证。状态为`runtime90_pfo_4d4e_audit_complete_next_slice_scoped_by_transaction_owner`。

基础设施实施结果：

1. `RuntimePrepareCollectorContext`新增受限`RuntimePrepareBufferUploadRecorder`和`RuntimePrepareFrameTransactionRecorder`；插件只能登记buffer/offset/immutable bytes与有界CPU commit/rollback action，不能取得底层`WgpuBufferUploadBatch`、raw Queue、submit或poll权限。particle、Hybrid GI、Virtual Geometry迁移后，raw `wgpu::Queue`已从collector context、collector闭包ABI、advanced-plugin dispatcher与compiled wrapper整条调用链硬切删除。
2. 每个collector拥有独立frame-local upload/state packet，wrapper在collector返回时把它move进RuntimePrepare总packet；任一collector错误会让总packet随函数栈一起丢弃。成功结果由`SceneRendererAdvancedPluginReadbacks`持有，graph prepare/materialization/scene validation失败时自动rollback，只有accepted scene才提交CPU shadow state。
3. 只有compiled prepare成功返回后，runtime-prepare upload才并入既有`frame_buffer_uploads`；graph执行成功后它与graph pass writes一同进入唯一`FrameResourceUpload` packet，并先于scene command buffers取得同一frame transaction的producer ticket。state transaction在scene submission ticket validation之后、renderer outputs提取之前提交；没有新增native submission。
4. recorder/collector/result/compiled handoff、raw Queue硬切、transaction exactly-once与validation-before-commit静态合同通过；精确rustfmt、scoped diff check与locked metadata通过。managed Cargo仍受`cargo_reuse_target_mismatch`外部状态阻断，真实WGPU、PNG/RDC、profile、VRAM和功耗证据为空。状态为`runtime90_pfo_4d4e_runtime_prepare_foundation_and_queue_authority_hard_cut_source_implemented_static_checks_passed_dynamic_validation_pending`。

#### PFO-4d4e particle producer transaction 源码实施与静态证据（2026-08-31）

1. 粒子 producer 现在通过 `RuntimePrepareGpuRecordingContext` 取得 device、encoder 和 queue-free upload recorder；emitter 参数与 counters 的两次 `queue.write_buffer` 已转为同一个 RuntimePrepare frame upload packet，生产 backend 不再取得 Queue。
2. `ParticleGpuFramePlanner` 增加 prepare/commit 分界。prepared frame 只在候选副本上推进 age、continuous spawn accumulator 和 burst cursor；失败或丢弃时不消耗 burst，成功时一次性提交小型 planner state。
3. `ParticleGpuBackend::execute_frame` 返回显式 ping-pong commit token，dispatch 使用候选 output index，`active_buffer_index` 只在成功 commit 更新。透明 draw 在 pending frame 存在时绑定候选 output，确保同一 scene submission 读取刚编码的结果。
4. `ParticleGpuRuntimeOwner` 保存单一 pending frame transaction；RuntimePrepare 成功钩子在 scene submission ticket validation 之后调用 owner commit，任何 collector、graph、materialization 或 submission 前错误通过事务 Drop 清除 pending state。资产切换仍按新 aggregate backend 建立，不伪造旧 GPU 内容回滚。
5. 每个 producer 事务收敛为一个 erased action allocation，避免为 commit/rollback 分配两个闭包对象；planner 候选状态复制范围仅为每个 emitter 的两个小型数组，未引入全状态 map clone。
6. 新增 failing-first contracts：RuntimePrepare transaction commit/drop exactly-once、compiled scene validation-before-commit、planner abandoned burst retry、particle queue-free producer ordering；静态契约全部通过。精确 rustfmt、scoped diff check、locked metadata通过；当前backend/planner/owner/runtime-prepare分别为556/417/488/594行，未触发结构拆分门。
7. 粒子GPU readback future不再在编码阶段直接进入跨帧队列；它在创建后注册第二个frame transaction，accepted scene才发布，失败帧则连同未注册readback request一起丢弃。这样下一帧不会消费由`SceneRendererAdvancedPluginReadbacks::Drop`标错的失败帧future。
8. 本项仍只有源码证据：受管 Cargo 仍受 `cargo_reuse_target_mismatch` 外部状态阻断，真实 WGPU、PNG、RenderDoc、1K/10K/100K profile、VRAM 和功耗数据为空，不宣称运行时性能收益或完整 milestone acceptance。

#### PFO-4d4e Hybrid GI / Virtual Geometry producer与提交状态复审（2026-08-31）

1. Hybrid GI completion params、radiance-cache各stage params、consume params与Global SDF page table已全部改用RuntimePrepare upload recorder；相关生产源码`queue.write_buffer`为0。Virtual Geometry的prepare params接口同步移除Queue并接受同一upload sink；当前VG execute-prepare没有产品caller，但旧Queue能力已在接线前删除，不保留兼容分支。
2. Global SDF `trace_page_signature`从即时CPU写改为`Arc<AtomicU64>`提交动作：page-table payload进入本帧upload packet后，只在accepted scene transaction提交signature；collector、graph、materialization或scene validation失败会保留旧signature，下一帧重新生成必要上传。
3. HGI结构复审区分了权威输入状态与提交状态。mesh projection cache、mesh-SDF scene、Global SDF clipmap residency/influence、dirty pages、terminal fallback和deferred cursor都由当前camera/scene输入决定；失败帧后保留它们不会把未执行GPU结果标成sampleable，dirty build request仍可重试。Radiance-cache revision也只在GPU readback成功后confirm，失败帧继续走bootstrap snapshot。
4. 两类HGI readback future现在都不在编码阶段直接进入instance pending队列。Global SDF build和radiance-cache readback各自注册frame transaction，accepted scene后才发布；失败帧drop future，避免下一帧把“request dropped before registration”升级为新的BufferMap错误。本帧已staged但未提交的Global SDF future仍计入三帧共享readback上限，避免同帧超额登记radiance readback。
5. RuntimePrepare完整生产调用链已删除raw Queue：context、collector closure ABI、advanced-plugin dispatcher、compiled wrapper均无`wgpu::Queue`。粒子/HGI/VG目标生产源码的`queue.write_buffer`扫描仅剩测试断言或Queue-backed WGPU test sink。
6. 当前静态证据为queue authority/readback transaction合同通过、scoped `git diff --check`通过（仅仓库LF/CRLF提示）、locked `cargo metadata`通过；RuntimePrepare core/advanced dispatcher/particle collector/HGI collector分别为867/545/594/863行，均低于1000行拆分门。
7. 本切片状态为`source_implemented_static_checks_passed_dynamic_validation_pending`。没有受管Cargo编译、真实WGPU、窗口PNG、RenderDoc RDC、300帧profile、VRAM或功耗数据，因此不标accepted milestone，也不提交性能收益结论。

#### PFO-4d4f direct/compiled frame Queue fact与test-only readback边界（2026-08-31）

结构复审与实施计划：

1. current-source确认direct `render_scene`与compiled `render_compiled_scene`都从`RenderBackend.queue`取得raw Queue，但产品帧内唯一读取只是`get_timestamp_period()`后传给realtime IBL diagnostic request。这个immutable device fact已由generation-qualified `WgpuRenderDevice::timestamp_period_ns()`在device bootstrap固化，继续从frame owner读取Queue是重复authority而非数据依赖。
2. compiled `CompiledSceneFrameSubmissionContext.queue`只被同文件`cfg(test)`的scene-velocity/exposure/color-LUT同步读回使用；production分支通过`let _ = (device, queue, streamer, frame)`明确丢弃它。测试readback仍有独立native submit/wait，属于验证代码与product diagnostic后续迁移边界，不应迫使产品submission context保留Queue。
3. 本切片将让direct/compiled realtime IBL读取`backend.render_device.timestamp_period_ns()`，删除两个产品frame owner的unconditional Queue local；compiled submission context的Queue field/destructure/literal改为`#[cfg(test)]`，测试语义保持不变。不会改动backend诊断helper、submission数量、query算法或readback同步外观。
4. 复杂度与性能影响均为`O(1)`事实读取替换，没有新增分配、锁、扫描、packet或submission。静态门要求两个产品frame函数无`let queue = &backend.queue`和`queue.get_timestamp_period()`，compiled production submission context无unconditional`wgpu::Queue` field，同时保留`cfg(test)`像素统计。
5. 实施结果：direct/compiled的realtime IBL consumer均改读`backend.render_device.timestamp_period_ns()`；两个frame函数的`let queue = &backend.queue`与`queue.get_timestamp_period()`均为0。compiled graph prepare因最后一个Queue consumer随RuntimePrepare硬切消失，也删除了未使用的Queue local。
6. `CompiledSceneFrameSubmissionContext.queue`字段、destructure和caller literal均限定为`#[cfg(test)]`；生产finalization不再以无行为tuple消费Queue。scene velocity、exposure和color-LUT同步读回测试保持原行为，未扩大到产品路径。
7. failing-first源码合同、精确rustfmt、scoped diff check与locked metadata通过；compiled frame/compiled submission/direct frame分别为483/629/796行，未触发1000行拆分门。本切片状态为`source_implemented_static_checks_passed_dynamic_validation_pending`，受管Cargo、WGPU、截图、RenderDoc、profile、VRAM和功耗门仍开放。

#### PFO-4d4g outer RenderBackend Queue owner硬切（2026-08-31）

结构复审与实施计划：

1. current-source确认`RenderBackend.queue`在backend生产实现内的`self.queue`读取为0；全仓`backend.queue`消费者均位于单元测试、test-only framebuffer/readback或刚限定为`cfg(test)`的compiled submission field。产品submission/upload/poll/status/caps/UI surface都已经委托generation-qualified `WgpuRenderDevice`。
2. `new_offscreen_with_policy`当前把`requested_device.queue.clone()`交给`WgpuRenderDeviceContext`，再把原Queue保存在outer backend；这使生产device generation同时保留两个Queue handle，却没有第二个产品consumer。Unreal RHI的适用owner约束是native command queue留在RHI device/submission owner，renderer feature不保留平行queue authority；Zircon应让one-shot WGPU context成为唯一产品owner。
3. 本切片将把`RenderBackend.queue`字段限定为`#[cfg(test)]`，production直接move `requested_device.queue`进入`WgpuRenderDeviceContext`；只有test构建在handoff前创建一个compatibility clone，维持现有GPU单元测试，不把它编入产品binary。不会修改device字段、测试helper API、submission service或测试执行语义。
4. 复杂度为cold bootstrap `O(1)`。production减少一个Queue handle clone/retention；test构建维持一次clone。静态门要求backend production owner无unconditional Queue field、constructor的production handoff不调用`queue.clone()`、backend production方法`self.queue`为0，并保留明确test-only guard。
5. 实施结果：`RenderBackend.queue`字段已限定为`#[cfg(test)]`；offscreen production构造把`requested_device.queue`直接move给`WgpuRenderDeviceContext`，不再执行production `queue.clone()`。test构建在handoff前保留一次`test_queue` clone，因此现有WGPU单元测试调用面不变。
6. backend production实现`self.queue`读取为0；source contract、精确rustfmt、scoped diff check与locked metadata通过，outer owner/constructor分别为158/72行。状态为`source_implemented_static_checks_passed_dynamic_validation_pending`；这属于owner/权限收敛，不把cold handle删除解释为帧时、功耗或瓶颈收益，动态验收门保持开放。

#### PFO-4d4h RuntimePrepare scoped GPU capability硬切（2026-08-31）

结构复审与实施计划：

1. PFO-4d4e虽然从collector ABI删除了raw Queue，但`RuntimePrepareCollectorContext`仍公开`device`与`encoder`字段。任意插件collector因此可以绕过`gpu_recording_context()`，直接录制命令，并与frame-local upload/state transaction能力脱节；公开`frame_extract`字段也保留了与只读accessor并行的第二条接口。
2. Unreal RenderGraph pass只在execute lambda内取得`FRHICommandList`，不会把command list/device作为pass根上下文的可随意字段；Zircon对应边界应是短生命周期`RuntimePrepareGpuRecordingContext`。该capability仍暂时借出同generation的Device与Encoder以承载尚未neutralize的插件资源/compute录制，但同时携带upload recorder与frame transaction recorder，且不能取得Queue、submit、poll或flush。
3. 本切片将把collector根上下文的Device、Encoder与frame extract字段全部私有化；只读frame/scene访问统一走显式accessor，并让返回引用绑定到底层frame生命周期而非对context本身的短借用。particle neutral owner、Hybrid GI instance/resource初始化与Global SDF dispatch全部迁移到scoped GPU recording context，不保留getter或旧公开字段兼容层。
4. 复杂度保持不变：不新增payload clone、锁、native packet或submission。HGI已有的scene mesh、lighting与prepared frame引用继续来自同一immutable frame；scoped capability仅收窄Rust权限面。静态门要求collector owner无public Device/Encoder/frame-extract字段，particle/HGI production collector无`context.device/context.encoder/context.frame_extract`直接访问，同时保留现有upload、readback和accepted-frame transaction顺序。
5. 实施结果：`RuntimePrepareCollectorContext`的Device、Encoder与frame extract字段已全部私有化；唯一公开native录制面是短生命周期`RuntimePrepareGpuRecordingContext`，并继续同时携带buffer upload与frame transaction recorder。只读frame/scene/sideband accessor显式返回底层frame生命周期引用，collector可在不clone mesh/extract的前提下交替取得scoped mutable recording capability。
6. particle neutral owner通过scoped capability取得Device后再注册static external bindings；real producer保持同一capability内的Device/Encoder/upload recorder。Hybrid GI的instance/resource初始化、Global SDF dispatch和radiance-cache prepare全部迁入scoped capability，production collector中`context.device`与`context.encoder`均为0；资源初始化异常保持typed `GraphicsError`，未引入panic或兼容getter。
7. failing-first源码合同先记录核心3个公开字段、particle direct Device、HGI direct Device/Encoder为失败，实施后17项scoped capability/负向合同全部通过；4个精确Rust文件通过rustfmt解析与格式检查。该结果只证明源码权限和借用边界，不证明受管Cargo、WGPU、PNG/RDC、profile、VRAM或功耗；状态为`source_implemented_static_checks_passed_dynamic_validation_pending`。

#### PFO-4d4i compiled submission test-only diagnostic inputs硬切（2026-08-31）

结构复审与实施计划：

1. current-source确认`CompiledSceneFrameSubmissionContext`中的`device`、`streamer`与`frame`只被`#[cfg(test)]`同步scene velocity、exposure与color-LUT读回使用；生产提交路径不消费这三个输入，只用`#[cfg(not(test))] let _ = (...)`压制未使用警告。submission ticket、history transaction commit、pipeline/probe/IBL提交后commit、HZB异步诊断与transient retirement均独立于这三个测试输入。
2. Unreal RHI/RDG对应边界是产品submission/finalization packet只携带提交、状态提交和资源退役所需能力；同步GPU readback属于测试诊断层，不应扩大shipping ABI或让生产finalizer取得额外Device/asset-frame authority。
3. 本切片将把context定义、解构和caller literal中的`device`、`streamer`、`frame`全部限定为`#[cfg(test)]`，并把仅因此存在的import一并测试化；删除production no-op tuple，不新增getter、clone、同步等待、submission或资源写入路径。
4. 复杂度与运行时行为保持不变：production结构减少3个borrowed pointer-sized字段的传递，test构建保留原同步读回行为。静态门要求定义、解构与caller三处测试条件一致，并确保生产源码不再保留no-op tuple；动态WGPU、截图、RenderDoc、profile、VRAM与功耗门继续开放，不把ABI收窄解释为性能收益。
5. 实施结果：context定义、函数解构与caller literal中的`device`、`streamer`、`frame`已统一限定为`#[cfg(test)]`，production no-op tuple已删除；纯测试`ViewportRenderFrame` import同步隔离。`ResourceStreamer` import继续保留在production，因为同文件的IBL artifact writeback prepare合法消费该类型，没有把独立生产职责误判为测试依赖。
6. failing-first源码合同先得到7项失败，实施后定义3项、caller 3项与no-op tuple负向合同共7/7通过。该结果只证明shipping ABI和authority surface收窄；没有运行受管Cargo/WGPU、PNG、RenderDoc、300帧profile、VRAM或功耗验收，状态为`source_implemented_static_checks_passed_dynamic_validation_pending`。

#### PFO-4d4j source cubemap direct mip-chain readback（2026-08-31）

结构复审与实施计划：

1. current-source确认source cubemap每个face先由renderer通过raw `backend.device`创建完整mip-chain大小的`COPY_DST | COPY_SRC`中间buffer，逐mip执行texture-to-buffer；随后product diagnostic service再把该buffer复制到自己拥有的MAP_READ staging。每个face因此有2段GPU copy、1个额外native buffer及renderer侧retention slot，尽管最终ticket、budget和map已由RHI diagnostic owner统一管理。
2. Unreal `FRHIGPUMemoryReadback`/`FRHIGPUTextureReadback`把staging resource、fence、copy与map生命周期统一放在RHI readback owner，feature只通过command list enqueue copy；Lumen compute参考同样把subresource copy录制在统一command context，而不让上层另建第二套readback staging owner。Zircon应扩展现有submission-bound diagnostic request，而不是给renderer新增Device getter。
3. 本切片将增加一个native RGBA16F mip-chain request：一个face仍计为一个budget request，但内部保存同一texture的多个validated mip subresource layout；RHI owner直接把所有mip复制到同一batch staging的连续区间，并在一次delivery中去除逐行padding、输出canonical mip-major bytes。
4. source-cubemap pending owner继续保留6 faces、默认1024链每批2 faces/共3批、上一批terminal后才允许下一批的有界语义；删除中间buffer创建、buffer diagnostic enqueue、buffer retention与renderer侧padding unpack。预期每face从`mip_count + 1`次GPU copy降为`mip_count`次，并删除1个临时GPU buffer；这是静态算法计数，实际帧时、带宽、功耗与显存收益必须由后续WGPU/RenderDoc/profile验收确认。
5. 实施结果：diagnostic request新增`NativeTextureMipChain` source与相对staging-offset layout；一个request只clone一次source texture，admission按整条链的padded bytes计费。scene-tail prepare与standalone diagnostic packet都遍历同一validated layout并复用同一个texture-copy helper；delivery一次性按mip顺序去除row padding，没有每mip临时payload allocation。
6. source-cubemap consumer已改为每face调用一次`enqueue_product_diagnostic_texture_rgba16float_mip_chain`；production owner中的raw `backend.device.create_buffer`、`enqueue_product_diagnostic_buffer`、`record_texture_copies`、6-slot `retained_buffers`与callback二次row-unpack均删除。6 faces、默认2 faces/batch、3 batches及上一批terminal后才推进的状态机保持不变。
7. 对默认1024、11 mip RGBA16F链的静态descriptor/layout计算：每face padded staging为11,190,016 bytes（10.672 MiB），canonical payload为11,184,808 bytes；旧两-face批次同时持有2个中间buffer和owner staging，共44,760,064 bytes（42.687 MiB），新路径只保留22,380,032 bytes（21.343 MiB）owner staging，删除22,380,032 bytes（21.343 MiB）中间描述容量。每face GPU copy从12降为11，批次中间GPU buffer从2降为0；allocator rounding、真实峰值、带宽、帧时与功耗仍需动态工具验证。
8. failing-first契约先观察request/service/device与cubemap中间资源缺口；实施后13/13 mip-chain/无Queue/无submit/poll/无consumer中间buffer源码合同通过，并加入纯CPU layout测试覆盖跨mip offset、row padding去除和mip-major输出。11个精确Rust文件通过rustfmt解析/格式检查，scoped diff check与locked metadata通过；主device diagnostics由938行拆回869行，80行helper无Queue/submit/poll。受管Cargo测试、WGPU、PNG、RenderDoc、300帧profile、VRAM与功耗仍pending。

#### PFO-4d4k standalone diagnostic submission owner硬切（2026-08-31）

结构复审与实施计划：

1. current-source确认环境捕获source-cubemap persistence已经通过RHI diagnostic service直接admit整条mip chain，但SceneRenderer仍只为该diagnostic batch通过raw `backend.device`创建空encoder，再调用`scope.prepare`把copy录入该encoder并经graphics packet提交。blocking product texture capture也保留同样的第二套encoder/submit编排；两条路径均把RHI已经拥有的copy packet、ticket、flush与terminalization重新暴露给上层。
2. Unreal `FRHIGPUMemoryReadback`/`FRHIGPUTextureReadback`由RHI readback owner持有staging、fence、copy与map生命周期，feature只enqueue command-list copy；Unity Graphics风格的render context同样由pipeline/context边界统一执行与submit。Zircon的standalone diagnostic应由`WgpuRenderDevice`完成encode、ticket bind、commit、flush与失败取消，`RenderBackend`只公开typed scope facade，SceneRenderer不取得native encoder。
3. 目标API分两层：`WgpuRenderDevice::submit_and_flush_diagnostic_readback_frame`复用现有`submit_diagnostic_readback_frame`的batch/ticket绑定，并在flush失败时通过同一`RenderDevice::cancel_submission`终态化Accepted ticket及diagnostic delivery；`ProductDiagnosticReadbackFrameScope::submit`消费scope并只返回可观察`SubmissionTicket`。scene-tail readback继续保留`scope.prepare`，因为它必须与已有scene command buffer共享同一ticket，不能错误拆成standalone packet。
4. 环境捕获persistence与blocking capture都硬切到`scope.submit`，删除其diagnostic-only raw command encoder和graphics packet编排。每个batch仍为一个logical diagnostic request batch和一个copy-class packet；6 faces、2 faces/batch、3 batches、上一批terminal后推进、callback routing与30秒显式capture timeout均保持不变。
5. 静态结构预期：环境捕获persistence每批native encoder创建从1降为0、上层command-buffer Vec构造从1降为0，RHI owner仍创建一个必要copy encoder且native submission数量不增加。真实CPU时间、GPU copy时间、queue latency、功耗与显存必须由后续Windows WGPU/RenderDoc/profile验收确认，不用源码计数宣称运行时收益。
6. 实施结果：`WgpuRenderDevice::submit_and_flush_diagnostic_readback_frame`成为standalone diagnostic唯一提交门面，复用既有copy packet的encode/bind/commit，并在同一device timeline上flush；flush失败沿`RenderDevice::cancel_submission`请求Accepted ticket终态化。`ProductDiagnosticReadbackFrameScope::submit`消费scope并隐藏native frame/encoder，原`prepare`继续仅服务必须附着scene packet的路径。
7. 环境捕获persistence与blocking texture capture的上层native encoder创建均由每批1降为0，上层`Vec<CommandBuffer>`构造均由1降为0，旧graphics diagnostic submit调用均由1降为0；环境捕获persistence owner中的raw `.device`访问降为0。RHI owner仍为每个standalone batch创建1个必要copy encoder并提交1个copy-class packet，因此源码结构没有增加native submission数量，也没有改变6 faces、2 faces/batch、3 batches或30秒timeout。
8. failing-first静态探针先观察到10/10目标条件不满足；实施后扩展契约16/16通过，覆盖RHI delegate/flush/cancel、无direct queue submit/poll、scope facade以及两个consumer无raw device/encoder/prepare/graphics submit。4个精确Rust文件通过rustfmt解析/格式检查，scoped diff check与locked metadata通过；文件规模分别为884、204、637、719行，未触发1000行模块拆分门槛。受管Cargo编译/测试、Windows WGPU、真实PNG、RenderDoc、300帧profile、VRAM与功耗仍pending。

#### PFO-4d4l pipeline diagnostic timeline owner硬切（2026-08-31）

结构复审与实施计划：

1. current-source调用图确认compiled frame先通过`poll_frame_submission_completions`完成唯一backend/RHI timeline poll，随后`MeshPipelineCache::drain_pipeline_creation_diagnostics`又取得raw `backend.device`并执行`Device::poll(Poll)`。同步prewarm/fallback的`finish_pipeline_creation_diagnostics_for_variant`也执行同一额外poll；validation failure分支再次调用drain，因此错误路径还可能重复触达native device timeline。
2. `track_pipeline_creation_error_scope`在登记diagnostic之前已经`pollster::block_on(error_scope.pop())`并把结果降为`Option<String>`。下游pending队列只保存已解析CPU事实，后置`Device::poll`不可能再参与这个已完成future的解析，也不拥有submission、resource retirement或map callback职责。正确算法是`O(p)`消费pending diagnostics，不在消费前附加一次无关driver poll。
3. Unreal `PipelineStateCache`以`FGraphEventRef`/task graph表达PSO async completion，并通过`WaitCompletion`或`WaitForAllTasks`等待明确的compile task；RHI command list/device timeline仍由RHI owner管理，renderer cache不会为消费已完成diagnostic另行poll native device。Zircon应保留frame begin的一次RHI completion poll，pipeline cache只消费自身resolved receipts。
4. 本切片将从`drain_pipeline_creation_diagnostics`与`finish_pipeline_creation_diagnostics_for_variant`删除`wgpu::Device`参数和两处`Device::poll`，并硬切全部caller；不改变error-scope pop、pending容量、失败pipeline eviction、shader-module引用释放、async compile worker或submission状态机。
5. 静态预期：正常compiled frame的显式native completion poll上界从RHI owner 1次 + pipeline diagnostic 1次降为仅RHI owner 1次；mesh pipeline diagnostic生产源码中的`Device::poll`从2降为0，两个消费API的raw Device参数从2降为0。真实driver CPU time、帧时、功耗和pipeline hitch必须由后续Windows profile/RenderDoc/ETW验收，不用源码调用计数宣称运行时收益。
6. `ensure_pipeline/tests.rs`当前1879行，但本切片只做一个既有调用的机械签名修正，不增加fixture、helper或新职责；新的ownership contract放入168行的diagnostic owner文件。为避免把独立timeline修复扩大成高风险测试重排，本切片按large single-purpose test-suite例外不做无关拆分，后续首次增加该suite新职责时按语义域拆分。
7. 实施结果：两个diagnostic消费API已删除raw Device参数及全部后置poll；frame-begin owner和8个validation-failure drain caller统一调用device-free drain，两个生产finish caller及一个既有测试caller统一调用device-free finish。正常compiled frame的显式completion poll源码上界由2降为1，mesh diagnostic direct `Device::poll`由2降为0；error-scope同步pop、pending队列、容量rollover和失败资源淘汰保持原顺序。
8. failing-first静态契约先得到1/6通过，五个失败项精确覆盖raw Device、direct poll、两个签名和frame caller；实施后6/6通过。12个精确Rust文件通过rustfmt解析/格式检查，scoped diff check与locked metadata通过；diagnostic owner由168增至203行（新增ownership contract），frame owner393行，主ensure owner972行，未向接近1000行的主owner增加逻辑。受管Cargo编译/测试、Windows WGPU、PNG、RenderDoc、ETW/300帧profile、VRAM与功耗仍pending。

#### PFO-4d4m realtime IBL timing report Device输入硬切（2026-08-31）

结构复审与实施计划：

1. current-source确认realtime IBL timestamp readback通过product diagnostic callback在backend/RHI completion dispatch后把`RealtimeIblGpuTimingReport`写入collector的共享队列；`take_completed`只锁定并drain已完成CPU report，不poll、不submit、不map，也不读取device generation。
2. `RealtimeIblRuntime::take_gpu_timing_reports`仍接收命名为`_device`的`&wgpu::Device`，唯一caller因此读取raw `self.backend.device`，但该输入对结果和生命周期均无行为。与Unreal RHI query/profiler owner一致，GPU query完成由submission/query owner推进，report consumer只消费resolved result，不应重新取得native device。
3. 本切片将从runtime report drain删除Device参数，并硬切SceneRenderer caller；不改变timestamp query支持探测、diagnostic request、callback routing、collector锁、report顺序或framework同步边界。静态指标为该API raw Device参数1降为0、caller `backend.device`读取1降为0；运行时算法、分配、锁和submission数量不变，因此不宣称帧时或功耗收益。
4. 实施结果：runtime report drain与唯一SceneRenderer caller已统一为device-free API，raw Device参数和`backend.device`读取均由1降为0；collector仍只drain由product diagnostic callback发布的completed reports。failing-first契约由1/4转为4/4，两个精确Rust文件通过rustfmt解析/格式检查、scoped diff check与locked metadata；runtime owner612行、facade含新增contract为59行。受管Cargo、WGPU、PNG/RDC、profile与功耗验收仍pending。

#### PFO-4d4n advanced-plugin CPU owner Device输入硬切（2026-08-31）

结构复审与实施计划：

1. `SceneRendererAdvancedPluginResources::new`虽然接收`_device: &wgpu::Device`，实现只把render feature描述映射为CPU capability并收集`RuntimePrepareCollectorRegistration`；真正需要Device/Encoder的插件录制继续由后续短生命周期`RuntimePrepareGpuRecordingContext`提供。构造函数自身不创建资源、不编码命令，也不读取device facts。
2. `SceneRendererAdvancedPluginReadbacks::collect_into_outputs`同样接收`_device`，但只把neutral `RenderPluginRendererOutputs`移动到renderer output owner；GPU readback completion已在RuntimePrepare transaction和product diagnostic owner中终态化。Unreal对应的feature registration/output publication也只传semantic state，RHI command context只进入实际录制边界。
3. 本切片将从上述两个CPU-only API删除Device参数并硬切一个生产构造caller、一个生产publication caller及三个既有测试构造caller；不改变collector ABI、scoped Device/Encoder recording capability、readback transaction、feature enablement或output publication顺序。静态指标为生产CPU-only API raw Device参数2降为0、生产caller raw Device借用2降为0；算法、分配、锁、submission和GPU工作量不变，不宣称运行时性能收益。
4. 实施结果：两个CPU-only API及全部五个caller已完成device-free硬切，生产caller raw Device借用由2降为0；`RuntimePrepareCollectorContext`根结构仍隐藏Device/Encoder，合法`RuntimePrepareGpuRecordingContext`继续提供scoped GPU录制能力。failing-first契约由1/6转为扩展后的7/7，五个精确Rust文件通过rustfmt解析/格式检查、scoped diff check与locked metadata；文件规模124、235、28、320、543行。受管Cargo、WGPU、PNG/RDC、profile与功耗验收仍pending。

#### PFO-4d4o RuntimePrepare external binding packet设备代际合同（2026-08-31）

结构复审与实施计划：

1. current-source确认`RuntimePrepareExternalBufferBinding`只携带logical/backing name、raw `wgpu::Buffer`与可选physical descriptor；`SceneRendererAdvancedPluginReadbacks`把绑定向量跨过runtime-prepare、transient materialization和plugin binder，但packet没有产生时的device/generation身份。graph execution resources虽然在transient materialization时记录当前`RenderDeviceProfile`，plugin binder却没有比较两边代际。
2. 当前生产源码有两类external-buffer registration producer：Particles把同一runtime-prepare scoped Device上建立的持久backend buffers通过static-name API登记，Virtual Geometry feedback则在该scoped Device上当帧创建并登记buffer；两者都随同一次dispatcher调用离开collector。无代际packet会让未来device recovery、错误缓存或脱离renderer-generation生命周期的持久producer把旧帧绑定交给新generation，最终把失败推迟到WGPU validation。注册时device epoch只证明packet admission身份，不证明任意raw handle的真实创建来源；本计划不会夸大该保证。
3. Unreal `FRDGBuilder::RegisterExternalBuffer`只接收引用计数的`FRDGPooledBuffer`，按底层RHI identity去重，并在RDG registration/validation owner中登记外部资源；它不接收无owner的裸buffer。Zircon现阶段还没有可替代全部plugin native录制的neutral pooled resource，因此先让整批runtime-prepare external bindings携带一个generation-qualified packet身份，并在graph execution resources已绑定当前device epoch后一次fail closed校验，避免为每个binding重复存储和比较epoch。
4. 实施边界：新增graphics-internal `RuntimePrepareExternalBufferBindingPacket`，由advanced-plugin dispatcher在collector全部成功后用当前`RenderDeviceProfile`封装；`SceneRendererAdvancedPluginReadbacks`持有可选packet，compiled binder只接收该packet。binder在遍历binding前用`RenderGraphExecutionResources.device_epoch()`做一次O(1)检查；缺失epoch或device/generation不一致立即返回typed asset error，不绑定任何plugin buffer，也不创建fallback。空binding不构造packet，既有fallback路径不受影响。
5. failing-first门包括packet qualification、dispatcher profile handoff、compiled caller不再借裸slice、binder before-loop epoch validation，以及两个独立WGPU backend之间的mismatch拒绝测试。不会新增native submission、poll、锁、per-binding allocation或句柄clone；packet固定增加两个typed scalar，比较成本O(1)。Virtual Geometry每帧feedback buffer分配已登记为后续profile对象，在Windows WGPU/ETW/RenderDoc数据可用前不实施池化或声称性能收益。
6. 实施结果：非空binding Vec现在只move一次进入`RuntimePrepareExternalBufferBindingPacket`，packet保存一个`DeviceId`和一个`DeviceGeneration`；空Vec保持`None`。dispatcher与compiled wrapper显式传递当前`RenderDeviceProfile`，`SceneRendererAdvancedPluginReadbacks`不再公开裸binding slice，compiled caller只借用qualified packet。
7. plugin binder在任何binding迭代或fallback资源创建之前执行一次packet/execution epoch比较；graph execution尚未建立epoch或device/generation不一致时立即返回`GraphicsError::Asset`链路的明确诊断。成功路径没有新增分配、锁、native command、submission、poll或per-binding比较，复杂度为一次O(1) packet admission加原有O(n) binding遍历。
8. 新增两个WGPU负向测试源码：未建立graph epoch时拒绝packet，以及source/destination两个独立backend device identity不同时在native buffer绑定前拒绝；两者均确认失败后execution resources没有安装目标buffer。受管Cargo lane尚未解阻，因此这些测试已写但未执行，不能作为动态WGPU acceptance。
9. failing-first源码合同从0/8转为扩展后的10/10；8个精确Rust文件通过Rust 2024 rustfmt解析/格式检查，scoped diff check与locked metadata通过。核心owner为909/260/626/32/265/155/654行，均低于1000行拆分门。`plugin_graph_resources.md`同步了generation-qualified packet和“registration epoch不证明任意raw handle来源”的边界；Windows WGPU、PNG、RenderDoc、ETW/300帧profile、VRAM与功耗仍pending。

#### PFO-4d4p RuntimePrepare持久插件资源设备代际生命周期（2026-08-31）

结构复审与实施计划：

1. current-source调用图确认PFO-4d4o只给frame-local external-binding packet增加了admission身份，仍不能证明Particles和Hybrid GI持久raw WGPU资源来自当前device generation。Particles的neutral owner通过长期保存`wgpu::Device` clone比较设备并重建40-byte identity buffers，但真实simulation backend只在aggregate asset变化时重建；GPU工作被readback/backpressure拒绝时还会直接登记旧backend buffers。Hybrid GI则长期保存shared pipelines/buffers、最多32个instance GPU state和两类pending readback future，功能disabled或prepared frame为空时不会触达任何释放路径。
2. 帧事务复审确认runtime-prepare CPU transaction只在同一compiled scene成功提交后同步commit，错误/丢弃则在该调用栈rollback；下一帧不会与未终态化的本地packet并行修改同一个plugin owner。因此device epoch切换可在collector入口、任何feature/admission早退之前同步drop旧代资源，不需要新增锁、等待、native poll或跨帧generation transaction队列。Particles独立pending readback队列必须与owner同时清空；Hybrid GI instance map内的radiance/global-SDF futures随instance统一drop，旧callback只剩无consumer completion owner。
3. Unreal `FRenderResource`通过`ReleaseRHIForAllResources`逆序释放全部已初始化RHI资源，并在新RHI有效后统一`InitRHI`；`UpdateRHI`也严格执行`ReleaseRHI -> InitRHI`，而不是让持久resource cache自行保留native device句柄并猜测兼容性。Zircon本切片采用等价的generation-qualified release/reinitialize边界，但保持RuntimePrepare collector局部所有权，不建立全局插件锁或复制Unreal全局resource list。
4. 实施边界：新增公开只读、`Copy + Eq`的`RuntimePrepareDeviceEpoch`，dispatcher从当前`RenderDeviceProfile`每帧构造一次并传入所有collector context；根context可在CPU early-return之前读取，scoped `RuntimePrepareGpuRecordingContext`也携带同一epoch，使raw Device/Encoder录制能力与代际事实不可分离。closure ABI只传一个typed value，不暴露完整profile或新增native handle clone。
5. Particles owner保存一个active epoch。首次激活也fail closed释放任何未限定存量；后续不一致时清除pending frame、planner states、aggregate asset/backend/executed state和neutral buffers。collector在读取completed readback、判断GPU admission或登记active bindings之前完成激活，并在切换时清空独立pending readback queue；neutral buffers删除长期`wgpu::Device` clone，只在owner已校验epoch且bundle缺失时创建。重置planner意味着下一次admitted frame从当前runtime instance age/transform权威状态重建，不把旧设备GPU粒子状态伪装成可迁移数据。
6. Hybrid GI state同样保存active epoch，并在prepared/extract/feature early return之前激活；不一致时drop shared GPU resources和整个instance map，保留仅用于诊断排序的单调collector frame index。该策略会丢弃旧代radiance/global-SDF GPU结果与CPU投影cache，换取明确正确的全bootstrap恢复；是否跨generation保留纯CPU cache必须先通过恢复profile证明重建成本，当前不增加复杂的拆分owner。
7. 稳定帧成本为dispatcher构造一个two-scalar value、每个持久collector一次O(1) equality compare，无分配、无native command、无submission、无poll；设备切换成本与被释放的Particles资源数及Hybrid GI resident instance/cache规模线性相关，只在generation change执行。failing-first源码合同得到0/10，精确覆盖typed epoch、context/closure handoff、Particles owner/collector/neutral clone删除和Hybrid GI owner reset；动态device-loss恢复、Windows WGPU、PNG、RenderDoc、ETW恢复尖峰、VRAM与功耗仍必须在验收lane解阻后验证。
8. 实施结果：独立29行`RuntimePrepareDeviceEpoch`只保存typed `DeviceId + DeviceGeneration`，advanced-plugin dispatcher每帧构造一次并以Copy value传给全部collector closure；根context和scoped GPU recording context观察同一值。没有新增Queue、submit、poll、allocator、锁、native handle clone或完整profile权限。
9. Particles owner首次激活也fail closed释放未限定存量，后续epoch mismatch统一drop pending frame、planner states、aggregate backend和neutral bundle；独立pending readback queue在任何completed-readback消费或admission fallback之前同步清空。neutral bundle长期`wgpu::Device` clone由1降为0，稳定epoch继续复用已有40-byte identity buffers，transaction id保持单调而不因重建回绕。
10. Hybrid GI在prepared-frame/feature-enable早退之前激活epoch，首次或mismatch时drop shared resources与最多32个resident instance的GPU/cache/readback state；collector frame index保持单调。无shared GPU-work admission时只消费当前epoch既有instance的ready completion，不再创建新instance；按现有descriptor常量，单个新instance被推迟的创建为13个buffer、279,160 bytes（272.617 KiB）描述容量，32-instance上界8,933,120 bytes（8.519 MiB）。该数字不含allocator rounding、pipeline/cache元数据，也不是实测VRAM收益。
11. Unreal `FRenderResource::ReleaseRHIForAllResources`/`InitPreRHIResources`与`UpdateRHI`的release-then-init生命周期已映射为plugin-owner局部generation activation，而不是复制全局resource list。稳定帧算法为每个持久collector一次O(1)比较；切换路径随真实resident state线性释放，不扫描历史资源、不新增全局锁。
12. failing-first源码合同由0/10转为扩展后的14/14；11个精确Rust文件通过Rust 2024 rustfmt解析/格式检查，scoped diff check和locked metadata通过。核心文件规模为29/925/559/155/239/652/539/200/652/909/53行，均低于1000行门；`plugin_graph_resources.md`已同步代际能力、Particles/HGI重建语义和非admitted allocation边界。受管Cargo编译/测试、真实device-loss、Windows WGPU、PNG/RDC、ETW/300帧profile、VRAM与功耗仍pending。

#### PFO-4d4q Contact Shadow graph executor pipeline设备代际缓存合同（2026-08-31）

结构复审与实施计划：

1. current-source确认`zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs`的`ContactShadowRenderPassExecutor`把`BindGroupLayout + ComputePipeline`长期存入`Mutex<Option<ContactShadowPipeline>>`，命中路径只判`Option::is_none()`；它不比较当前graph execution device/generation。这个executor由plugin registration跨帧持有，不能假设SceneRenderer或plugin registration一定随RHI recovery整体重建。
2. `RenderPassGpuExecutionContext`已经持有materialized `RenderGraphExecutionResources`，而后者在transient materialization阶段记录当前`(DeviceId, DeviceGeneration)`。本切片只在GPU context提供一个只读`device_epoch()` facade，不暴露resources registry或native device clone；epoch缺失时Contact Shadow拒绝录制，避免无代际证明的pipeline命令。
3. 缓存值改为`Option<ContactShadowPipelineCache>`，其中保存不可拆解、`Copy + Eq`的`RenderPassDeviceEpoch`和pipeline。稳定帧做一次O(1) epoch比较，相同则继续复用bind-group layout/pipeline；epoch变化先丢弃旧值，再用当前scoped GPU device创建新值。不会新增Queue、submit、poll、锁、每帧pipeline创建或bind-group分配；只在代际切换重建一次。
4. Unreal RenderCore的PSO/resource owner要求RHI变化时`ReleaseRHI -> InitRHI`，不能把旧RHI对象当作跨设备兼容资源。本切片把该规则限制在Contact Shadow owner，核心OIT/Froxel/Planar缓存另行确认SceneRenderer recovery ownership后再扩展，不在此处批量改动。
5. failing-first源码合同覆盖GPU context epoch facade、Contact Shadow cache epoch storage/rebuild、missing-epoch fail-closed和旧`Option<ContactShadowPipeline>`形态删除；动态device-loss、Windows WGPU、PNG/RDC、RenderDoc marker、profile与功耗仍pending。
6. 实施结果：公开`RenderPassDeviceEpoch`只允许`Copy + Eq`，不公开`device_id`/`generation`字段；`RenderPassGpuExecutionContext::device_epoch()`只从已materialize的`RenderGraphExecutionResources`构造该值，并经scene/graphics facade提供给外部graph executor。插件能保存和比较代际事实，但不能拆解标量、访问resource registry或克隆native device owner。
7. Contact Shadow持久缓存保存typed epoch和`BindGroupLayout + ComputePipeline`。稳定帧在已有Mutex临界区内增加一次O(1)比较，无新增分配、native command、submit或poll；mismatch路径显式`take/drop`旧缓存后创建一个新layout和一个新pipeline，严格保持release-then-init。missing-epoch路径在4次graph texture lookup和cache lock之前返回错误；每帧bind group创建行为未在无profile证据时扩大或重写。
8. failing-first源码合同由0/5转为扩展后的20/20，覆盖typed/private epoch、四级公开导出、materialized-resource来源、缓存键、缺失epoch早退、旧形态删除以及`lock -> release -> create -> store`顺序；另已编写一个typed epoch语义单测，覆盖相同身份相等和device/generation任一变化不等。7个精确Rust文件通过Rust 2024 rustfmt解析/格式检查，scoped diff check、root/plugin locked metadata通过。核心文件为545/367/521/42/100/155/831行，均低于1000行拆分门；`rendering-plugin-options.md`已同步插件设备恢复边界。受管Cargo编译/测试仍被共享target mismatch阻塞，因此语义单测尚未执行；真实device-loss、Windows WGPU、PNG/RDC、RenderDoc marker、ETW/300帧profile、VRAM与功耗仍pending。

#### PFO-4d4r core graph executor native pipeline代际owner（2026-08-31）

结构复审与实施计划：

1. current-source从`WgpuRenderFramework -> RenderFrameworkState -> SceneRenderer -> RenderBackend/WgpuRenderDevice`复核恢复边界：当前产品没有在存活SceneRenderer内原地替换native device的完成入口，device-loss主要被fault gate和submission completion终态化为失败；但RenderGraph materializer、transient pool、统一compute pipeline cache和render-asset residency已经显式支持device epoch变化。因而核心executor不能继续依赖“当前尚未完成live recovery”来证明长期native cache永远兼容。
2. advanced-lighting共有7个持久cache owner：Froxel 3个、OIT 2个、Planar filter 1个、SSS shared bundle 1个；它们服务9个pass执行入口，并至少长期保存20个显式WGPU native handle（bind-group layout、compute/render pipeline和一个fallback buffer）。当前命中算法只检查`Option`，或只检查depth/target format，不检查materialized graph epoch；OIT fragment甚至会在缓存代际验证前录制`clear_buffer`。
3. 新增SceneRenderer私有`RenderPassDeviceEpochCache<K, V>`，cache identity固定为`(RenderPassDeviceEpoch, local descriptor key)`：无额外descriptor的Froxel/Planar使用`()`，OIT fragment/resolve和SSS使用depth/target format。稳定命中只做两个`Eq`比较并复用值；mismatch先`take/drop`完整旧entry，再执行fallible constructor，成功后才发布新entry。constructor失败保持cache为空，不回退旧代资源，不新增全局registry、Queue、submit、poll或native device clone。
4. 所有9个pass入口都必须在graph resource lookup、buffer clear、upload append或command encoding之前要求materialized epoch；epoch缺失立即返回owner-specific错误。现有每executor Mutex继续作为短临界区，helper不增加锁层级或堆分配。按所有相关feature同时启用的源码上界，稳定帧最多增加9次typed epoch/key常数比较；真实CPU/GPU耗时、锁竞争和恢复尖峰必须由ETW/RenderDoc/profile验证，不能由该上界推断。
5. Unreal `FRenderResource::ReleaseRHIForAllResources`逆序释放、`InitPreRHIResources`重新初始化以及`UpdateRHI`固定`ReleaseRHI -> InitRHI`；本切片映射为每个executor局部entry的release-then-create，而不是复制Unreal全局resource list。泛型owner必须用纯Rust drop-order测试证明稳定命中不创建、epoch/key变化先drop、创建失败不保留旧entry。
6. failing-first合同覆盖generic typed entry、7个owner迁移、9个epoch gate、OIT clear顺序、descriptor key保留和旧裸`Mutex<Option<Pipeline>>`形态删除；动态device-loss、Windows WGPU、PNG/RDC、RenderDoc marker、300帧profile、VRAM和功耗仍pending。
7. 实施结果：新增116行core-private `RenderPassDeviceEpochCache<K,V>`，entry只保存typed epoch、local descriptor key和native value。稳定identity直接返回引用；mismatch路径固定执行`take/drop -> fallible create -> insert/publish`。两个纯Rust语义测试已编写，覆盖稳定命中零重建、device epoch变化、descriptor key变化、旧值在constructor前已drop，以及constructor失败后cache为空；它们等待受管Cargo lane解阻后执行。
8. Froxel media/scatter/integrate、OIT fragment/resolve、Planar filter和SSS shared bundle共7个owner已全部迁移，目标范围旧`Mutex<Option<...Pipeline>>`由7降为0。9个pass执行入口都在资源查找或编码前取得materialized epoch；OIT fragment把`clear_buffer`从pipeline admission之前移动到之后。OIT/SSS原来只用于旧cache判定的3个重复descriptor字段/getter已删除，depth/target format继续作为构造参数和typed cache key。
9. 稳定满功能帧源码上界为9次epoch/key常数比较；仍沿用7个原有Mutex，不增加锁层、分配、Queue、submit、poll或native handle clone。设备/key变化只重建对应owner；SSS三pass共享一个bundle，同epoch/format下setup创建后scatter/recombine命中。该上界不是耗时、功耗或锁竞争实测。
10. failing-first源码合同由0/14转为14/14，覆盖generic entry、epoch/key比较、release-create-publish顺序、2个drop测试、7个owner、9个gate、7个admission、旧形态删除、OIT clear顺序和重复getter删除；13个精确Rust文件通过Rust 2024 rustfmt解析/格式检查，scoped diff check与root/plugin locked metadata通过。文件规模为116/44/548/473/87/158/87/208/158/166/113/248/374行，均低于1000行拆分门；`render_graph_execution_resources.md`已同步统一owner合同和证据边界。受管Cargo编译/测试、live recovery、Windows WGPU、fresh PNG/RDC、RenderDoc、ETW/300帧profile、VRAM与功耗仍pending。

#### PFO-4d4s SceneRenderer core资源包设备代际护栏（2026-08-31）

结构复审与实施计划：

1. Deferred、HZB、post-process 以及 SceneRendererCore 的其它持久 WGPU 对象都在 `SceneRendererCore::new_with_icon_source` 中一次性构造，并与 `SceneRenderer`/`RenderBackend` 同寿命。当前产品没有在存活 renderer 内替换 native `Device` 的完成入口；只给这些局部缓存逐一增加 epoch 会产生局部安全假象，不能完成 Unreal `ReleaseRHI -> InitRHI` 语义。
2. 在真正的 core 重建 owner 接入前，direct 与 compiled 两条渲染入口必须先证明当前 backend 仍是构造资源时的同一 `(DeviceId, DeviceGeneration)`。失配时在创建 encoder、读取 history、访问任何 core resource 之前 fail closed，禁止旧代资源继续录制。
3. 护栏只保存两个 typed RHI 标量并做一次 O(1) 比较；不克隆 native device、queue、adapter，不增加锁、提交、poll、资源扫描或每 pass 比较。它是未来 recovery 的 admission gate，不是 recovery 实现，也不把 fault gate 的终态化误报为可恢复。
4. 后续 live recovery 必须由统一 SceneRenderer owner 执行逆序 release、重新构造整个 core/resource-streamer/history/UI 资源包，并在 replacement profile 通过后再重新开放帧录制；本切片不拆出局部重建 API。
5. failing-first 合同覆盖 core 保存构造 epoch、direct/compiled 在首次 native work 前调用 guard、失配返回显式 GraphicsError，以及稳定路径不引入 native/queue 权限。静态验证不代表真实 device-loss；Windows WGPU、PNG/RDC、RenderDoc、ETW/300帧 profile、VRAM 和功耗继续 pending。

实施结果：

1. `SceneRendererCore` 现在保存构造时的 `DeviceId + DeviceGeneration`，并提供私有 fail-closed guard；direct/compiled render 在借用 backend device、创建 encoder 或准备 graph 前调用该 guard。
2. 稳定设备代际只增加两次标量比较；代际失配返回 `GraphicsError::SceneRendererDeviceEpochMismatch`，不会触碰任何持久 native resource。没有新增 queue、submit、poll、锁、分配或资源遍历。
3. 新增源码合同覆盖 core 字段、两条入口的早期 guard 和错误变体；精确 rustfmt、scoped diff check、locked metadata 通过后，本切片状态为 `source_implemented_static_checks_passed_dynamic_validation_pending`。未运行受管 Cargo、live device-loss、Windows WGPU、PNG/RDC、RenderDoc、profile、VRAM 或功耗验证。

#### PFO-4d4t RuntimePrepare GPU readback请求设备代际护栏（2026-08-31）

结构复审与实施计划：

1. current-source确认`RuntimePrepareCollectorContext::request_gpu_readback`把collector作用域内创建的raw `wgpu::Buffer`移动到跨阶段请求，但请求只保存名称、范围与completion；`SceneRendererAdvancedPluginReadbacks::register_product_gpu_readbacks`稍后才把它交给当前`RenderBackend`的product diagnostic owner。请求创建与注册之间没有代际证明，未来RHI replacement可能把旧代buffer带入新代diagnostic admission。
2. Unreal readback资源由RHI owner和command-list fence共同限定，feature只提交明确的RHI资源引用；它不会让异代native资源靠WGPU validation才失败。Zircon当前仍保留raw buffer作为过渡输入，因此本切片只增加一次request-level typed epoch admission，不伪装成neutral buffer handle迁移。
3. 实施边界：请求创建时复制当前scoped `RuntimePrepareDeviceEpoch`；`register`在计算范围、构造callback或调用`enqueue_product_diagnostic_buffer`之前比较backend profile。失配必须先写入completion错误并返回`GraphicsError::RuntimePrepareDeviceEpochMismatch`，不得消耗diagnostic budget、创建staging、录制copy、map、submit或poll；相同代际保持现有O(1)请求拆包和diagnostic routing。
4. 稳定帧增加一个typed two-scalar equality compare和一个request字段，设备切换路径只终态化该请求，不遍历历史buffer、不增加锁层级、不复制字节、不新建队列。这个护栏只证明buffer request的admission代际，不证明任意插件长期buffer的真实创建来源；持久插件资源仍由PFO-4d4p whole-owner release/reinitialize负责。
5. failing-first源码合同覆盖request保存epoch、epoch guard早于diagnostic admission、typed error与现有无`GpuReadbackQueue`/无byte clone路径；动态异代backend、device-loss、Windows WGPU、PNG/RDC、RenderDoc、profile、VRAM与功耗仍必须在受管验证lane解阻后执行。
6. 实施结果：`RuntimePrepareGpuReadbackRequest`保存创建时`RuntimePrepareDeviceEpoch`，注册前比较当前`RenderDeviceProfile`；失配立即把completion置为错误并返回typed `GraphicsError`，旧代buffer不会进入diagnostic owner。请求owner已拆入`runtime_prepare_collector/gpu_readback.rs`，根collector只负责构造和编排，避免945行文件继续堆协议逻辑。静态合同扩展为7/7，精确rustfmt、scoped diff check与locked metadata通过；本切片没有运行受管Cargo或真实WGPU验证，状态为`source_implemented_static_checks_passed_dynamic_validation_pending`。

#### PFO-4d4u graph execution typed device epoch owner（2026-08-31）

结构复审与实施计划：

1. current-source确认`RenderGraphExecutionResources`在materialization时把设备代际保存为`Option<(u64,u64)>`，`RenderPassGpuExecutionContext::device_epoch`每次再把tuple包装成`RenderPassDeviceEpoch`；external binding packet也按tuple拆解比较。这个重复表示允许不同owner误传单个scalar，并让graph resource packet的核心身份没有一个共享类型。
2. Unreal RDG/RHI把资源所属RHI代际作为资源owner事实，feature只消费被owner验证的资源；序列化诊断报告才投影稳定的数值字段。Zircon保留neutral `RenderGraphComputePipelineResolution`的u64报告ABI，但图执行资源和native binding admission应共享不透明的`RenderPassDeviceEpoch`。
3. 实施边界：新增graph-execution-private epoch owner，字段继续私有，提供仅crate可用的raw projection；resource packet字段和getter改为`Option<RenderPassDeviceEpoch>`，materialization单次构造，GPU context直接转发，external binding packet接受typed epoch并只在边界转换为诊断字符串。不得新增native clone、Queue、submit、poll、锁或每pass扫描。
4. 稳定路径仅保存/复制一个`Copy + Eq` typed value，移除resource packet和GPU facade之间的tuple重包装；设备切换仍由既有transient/resource owner处理。本切片不改compute pipeline report的序列化字段，也不声称运行时性能收益。
5. failing-first源码合同覆盖共同epoch owner、typed resource storage/materialization、GPU facade直传、typed external packet和目标范围内裸`Option<(u64,u64)>`清零；动态device-loss、Windows WGPU、PNG/RDC、RenderDoc、profile、VRAM与功耗继续由受管验证lane负责。
6. 实施结果：`RenderPassDeviceEpoch`移入graph execution独立owner；resource packet直接存储typed epoch，materialization构造一次，GPU context直接返回，external binding packet改为typed admission并通过crate内raw projection生成诊断。目标范围裸epoch tuple由1处降为0，静态合同7/7、rustfmt、locked metadata与scoped diff check通过；受管Cargo和真实WGPU验证仍pending，状态为`source_implemented_static_checks_passed_dynamic_validation_pending`。

#### PFO-4d4v generic compute pipeline typed device epoch owner（2026-08-31）

结构复审与实施计划：

1. current-source确认generic compute pipeline cache是graph executor内另一个长期保存`ComputePipeline + BindGroupLayout`的native owner。虽然graph resources已经提供typed `RenderPassDeviceEpoch`，cache仍用`Option<(u64,u64)>`保存active epoch，generic executor因此必须在调用边界降级tuple，削弱代际清除和last-good fallback的统一身份。
2. neutral `RenderGraphComputePipelineResolution`是跨模块诊断/序列化数据，保留`device_id: Option<u64>`和`device_generation: Option<u64>`字段；native cache本身应像前述7个advanced owners一样保存完整typed identity，只在生成neutral resolution时一次投影。
3. 实施边界：`ComputePipelineCache::resolve`、`active_device_epoch`和`update_device_epoch`全部接收`Option<RenderPassDeviceEpoch>`；ready/last-good resolution构造点调用crate内`raw_parts`一次。缓存清除顺序、family publication、容量/LRU和fallback行为不变，不新增锁、Queue、submit、poll、分配或pipeline重建。
4. 稳定路径仍是一次`Copy + Eq` typed compare；device/key变化继续清除scene layout、pipeline records和published families。此切片只消除native cache内部tuple，不把neutral report字段误报为资源owner，也不宣称运行时性能收益。
5. failing-first源码合同覆盖typed cache field/argument、single raw report projection、generic executor直接传typed resources epoch和旧`Option<(u64,u64)>`删除；动态WGPU、pipeline hitch、PNG/RDC、RenderDoc、profile、VRAM与功耗继续由受管验证lane负责。
6. 实施结果：generic compute cache现保存`RenderPassDeviceEpoch`，generic executor直接传graph资源typed epoch，ready/last-good report只在边界投影raw parts；目标cache内部裸epoch tuple由3处降为0。新增源码合同通过，精确rustfmt、locked metadata与scoped diff check通过；受管Cargo、真实WGPU和性能证据仍pending，状态为`source_implemented_static_checks_passed_dynamic_validation_pending`。

#### PFO-4d4w ambient occlusion typed pipeline report epoch（2026-08-31）

结构复审与实施计划：

1. current-source确认AO执行记录从neutral `RenderGraphComputePipelineResolution`读取`device_id/device_generation`后，以`Option<(u64,u64)>`在evaluate/spatial/upsample三阶段之间比较。该记录是诊断报告owner，不应重新定义native epoch身份，也不应让元组成为graph执行层的第二种设备代际类型。
2. neutral resolution与`RenderAmbientOcclusionExecutionReport`继续保留稳定的两个`Option<u64>`字段，供跨模块报告、日志和序列化使用；AO内部一致性校验改用graph execution的opaque `RenderPassDeviceEpoch`，仅在写入neutral report字段时调用一次`raw_parts`。
3. 实施边界：`pipeline_device_epoch`返回`Option<RenderPassDeviceEpoch>`并在构造点校验两个neutral字段均存在；evaluate/spatial/upsample只做typed `Copy + Eq`比较，报告赋值处完成单次raw projection。不得改动pass计数、fallback状态、资源访问统计、Queue、submit、poll或报告ABI。
4. 稳定路径仍是三次typed option比较和一次报告投影；缺失任一阶段epoch继续触发既有`PIPELINE_DEVICE_EPOCH_MISMATCH`，不会以默认标量掩盖异代或缺失身份。该切片只消除AO执行记录内部裸tuple，不宣称动态性能收益。
5. failing-first源码合同覆盖typed helper、opaque constructor、单次raw projection和旧`Option<(u64,u64)>`/tuple field projection删除；动态AO WGPU、pipeline fallback、PNG/RDC、RenderDoc、300帧profile、VRAM与功耗仍由受管验证lane负责。
6. 实施结果：AO pipeline epoch比较现在使用`RenderPassDeviceEpoch`，neutral report只在赋值边界投影raw parts；目标AO记录裸epoch tuple由1处降为0。新增源码合同、精确rustfmt、locked metadata与scoped diff check通过；受管Cargo、真实WGPU和性能证据仍pending，状态为`source_implemented_static_checks_passed_dynamic_validation_pending`。

#### PFO-4d4x transient resource pool shared device epoch owner（2026-08-31）

结构复审与实施计划：

1. current-source确认`TransientResourcePool`及其texture/buffer allocation另有一份`TransientResourcePoolDeviceEpoch`，虽与graph packet、pipeline cache、advanced owners表达同一`(DeviceId, DeviceGeneration)`，却让free-list reuse、pending retirement和graph materialization存在第二种设备身份类型。
2. 该重复类型不是budget eviction的`(last_used_frame, byte_size)`元数据，也不是mesh geometry key；它直接参与allocation epoch、active pool、wrong-epoch错误和submission-ticket校验，属于同一graph execution owner的结构重复。
3. 实施边界：`RenderPassDeviceEpoch`增加从`RenderDeviceProfile`构造的crate-internal入口；transient pool与allocation字段、错误和helper统一使用该opaque type。`SubmissionTicket`的`DeviceId/DeviceGeneration`只在`accepts_ticket`边界投影raw parts；descriptor-key reuse、completion status、retirement和budget eviction算法保持不变。
4. 稳定路径仍是一次typed epoch equality，ticket admission仅做一次双标量边界比较；device切换继续清空free/pending backing并累计既有报告，不新增Queue、submit、poll、锁、分配或资源扫描。该切片不把ticket字段或neutral report改成graph-private类型。
5. failing-first源码合同覆盖shared epoch field/profile constructor/allocation ownership、ticket boundary projection及旧`TransientResourcePoolDeviceEpoch`删除；动态device-loss、Windows WGPU、PNG/RDC、RenderDoc、300帧profile、VRAM与功耗继续由受管验证lane负责。
6. 实施结果：瞬态纹理/缓冲池和allocation现在与graph packet、executor cache共享`RenderPassDeviceEpoch`；profile只在每帧激活时构造一次，ticket scalar只在admission边界读取。目标重复pool epoch类型由1降为0。新增源码合同、精确rustfmt、locked metadata与scoped diff check通过；受管Cargo、真实WGPU和性能证据仍pending，状态为`source_implemented_static_checks_passed_dynamic_validation_pending`。

#### PFO-4d4y scene-renderer raw resource mutator scope（2026-08-31）

结构复审与实施计划：

1. current-source确认`RenderGraphExecutionResources`的`import_texture_view`、`insert_buffer`和`import_texture_alias`仍是crate-wide `pub`，允许任何可见模块直接写入WGPU texture view/buffer表；这与已限定的resolver lookup、qualified plugin packet和scene-renderer graph binder形成并行的未审查注入面。
2. 调用图显示三个mutator的生产使用者均在`scene_renderer`内部，外部插件不需要直接写资源表；插件资源必须经过`RuntimePrepareExternalBufferBindingPacket`和`bind_plugin_graph_resources`的代际/声明校验。因而该问题是可见性边界错误，不需要新增运行时检查或复制资源。
3. 实施边界：将三个兼容mutator改为`pub(in crate::graphics::scene::scene_renderer)`，保留现有参数、返回值、WGPU clone行为及所有scene-renderer调用；不改动`RenderGraphExecutionResources`的读路径、external access table、transient materialization或neutral report ABI。
4. 稳定路径无算法变化，只收窄Rust权限面；scene renderer内仍是原有O(1) map insert/remove，其他模块无法绕过资源binder注入未经声明的native handle。不得新增Queue、submit、poll、锁或fallback。
5. failing-first源码合同覆盖三个scene-renderer-scoped签名和旧crate-wide `pub fn`删除；动态WGPU、device-loss、PNG/RDC、RenderDoc、300帧profile、VRAM与功耗仍由受管验证lane负责。
6. 实施结果：三个raw resource mutator已收窄到scene-renderer owner，scene内调用保持不变；新增source contract、精确rustfmt、locked metadata与scoped diff check通过。该切片不宣称运行时性能收益，状态为`source_implemented_static_checks_passed_dynamic_validation_pending`。

#### PFO-4d4z render-pass native capability scope（2026-08-31）

1. current-source确认`RenderPassGpuExecutionContext`仍以公开字段暴露`Device`、`CommandEncoder`、scene bind group、graph resource table和plugin output mailbox；Hybrid GI、Contact Shadow和particles跨crate消费者直接读取这些字段。该形态绕过了已建立的graph resource resolver与pass upload capability边界。
2. Unreal/Unity的pass执行面把命令录制能力作为短生命周期pass parameter或command-buffer capability，不把graph resource registry作为可持久化上下文字段。Zircon保留现有单encoder和graph resource生命周期，只把native recording handles显式分组为短借用capability；资源解析仍先通过typed GPU lookup，输出回执仍通过mailbox accessor。
3. 实施边界：新增公开`RenderPassGpuNativeContext`，包含`Device`、`CommandEncoder`和scene bind handles；主GPU context字段改为`pub(in crate::graphics::scene::scene_renderer)`，并新增`native_context()`、`plugin_outputs()`与`plugin_outputs_mut()`。Hybrid GI三处handoff、Contact Shadow pipeline/bind/dispatch和particles输出projection迁移到这些入口；不新增Queue、submit、poll、clone、锁或资源复制。
4. 稳定路径只增加一次短期结构借用，编码顺序、pass-local upload batch、device epoch gate和single submission保持不变。native capability不包含resource table或output storage本身，避免外部feature通过公开context字段绕过声明/lease检查。
5. failing-first源码合同覆盖scoped native fields、explicit capability owner、plugin output accessor及三类跨crate消费者无legacy field access；动态Cargo、Windows WGPU、device-loss、PNG/RDC、RenderDoc、300帧profile、VRAM与功耗仍由受管验证lane负责。
6. 实施结果：主GPU context的native/resource/output字段已收窄到scene-renderer owner；新增`RenderPassGpuNativeContext`短生命周期录制面，Hybrid GI、Contact Shadow与particles已完成调用迁移；source contract、精确rustfmt、scoped diff check和locked metadata通过。本切片不宣称运行时性能收益，状态为`source_implemented_static_checks_passed_dynamic_validation_pending`。

#### PFO-4d5a render-pass native resource create measurement gate（2026-08-31）

结构复审与实施计划：

1. current-source复审确认4d4z虽然收窄了主GPU context，但短生命周期native capability仍公开raw `Device`。Hybrid GI的scene-depth、trace-schedule、resolve三处handoff在每次pass录制时创建layout、shader、pipeline和bind group；Contact Shadow在cache miss时创建pipeline族、稳定帧仍创建bind group。现有profile只记录CPU时间、draw/dispatch和upload，无法先量化创建压力再判断pipeline prewarm、bind-group cache或参数buffer owner。
2. Unreal的RDG/RHI pass参数与pipeline cache把命令录制、资源生命周期和PSO创建职责分层；Unity command-buffer profiling也将工作归属到录制scope。Zircon不复制其线程/多队列实现，而是保留单encoder和现有graph资源owner：raw `Device`在native context中改为private，跨crate pass只通过可计数的`RenderPassGpuResourceFactory`创建buffer、bind group、layout、shader module和pipeline。texture/sampler不进入该能力，避免graph texture或持久资源在pass内形成第二owner。
3. `RenderPassNativeResourceCreateMetrics`按buffer、bind group、bind-group layout、shader module、pipeline layout、compute pipeline和render pipeline分类，使用饱和计数；每个pass结束时通过`RecordedGraphPass -> RenderGraphPassProfileRecord -> RenderPassProfileEntry`传播，旧profile JSON通过`serde(default)`保持可读。固定诊断路径为`render.profile.native_resource_create.{total_count,buffer_count,bind_group_count,bind_group_layout_count,shader_module_count,pipeline_layout_count,compute_pipeline_count,render_pipeline_count}`，不生成动态pass-name路径。
4. 当前源码执行路径的创建规模上界为：Hybrid GI resolve每次6项（含1个参数buffer与1条render pipeline），scene-depth每次5项，trace-schedule每次5项；Contact Shadow稳定cache命中帧1项，cold/device-epoch cache miss帧5项。这些是从调用图得到的结构计数，不是运行时采样、耗时或功耗证据，也不证明应采用某一种缓存策略。
5. 本切片不改shader、dispatch、submission、pipeline cache命中规则或资源生命周期算法。后续优化必须先在代表性场景采集至少300帧的per-pass create counts、CPU encode p50/p95/p99、GPU timestamp/RenderDoc hitch、显存与功耗，再决定将哪类创建迁移到generation-qualified持久owner或frame-local cache，并验证创建计数和停顿是否消失。
6. failing-first源码合同覆盖raw device不可见、factory方法集合、pass计数归零/提取、graph/frame profile传播、Hybrid GI/Contact Shadow无`native.device`旁路及固定诊断聚合。实施后精确rustfmt、locked metadata、scoped diff check与WGPU 29.0.3本地API签名复核通过；受管Cargo、真实Windows WGPU、device-loss、PNG/RDC截图、RenderDoc、300帧profile、VRAM与功耗仍pending。状态为`source_implemented_static_checks_passed_dynamic_validation_pending`，不宣称性能或视觉验收完成。

#### PFO-4d5b generic compute resource create measurement（2026-08-31）

结构复审与实施计划：

1. 4d5a后的覆盖复审确认factory已覆盖跨crate Hybrid GI/Contact Shadow，但内建generic compute executor仍直接调用`gpu.device.create_bind_group`，其bounded `ComputePipelineCache`在cold miss时也直接创建bind-group layout、pipeline layout、shader module和compute pipeline。因此profile对通用compute稳定帧和首帧/variant miss均会漏计，不能据此选择prewarm或cache调整。
2. 本切片只扩展同一测量authority：`RenderPassGpuExecutionContext`实现`RenderPassGpuResourceFactory`，复用同一pass-local饱和计数；generic executor把该factory传入cache并用它创建dispatch bind group。cache继续单独接收raw device用于`limits()`和validation error scope，不把raw device保存、公开或用于native resource create。
3. `ComputePipelineCache::get_or_create`保留测试/独立调用签名并以raw device的neutral factory实现委托；产品`resolve`走`get_or_create_with_factory`。MRU hash bucket、16项容量、device-epoch清除、family publication、last-good fallback、workgroup validation、error-scope pop和锁范围均不变。
4. 成功cache hit的当前结构计数为每dispatch 1个bind group；成功cold/variant miss为1个bind group、1个bind-group layout、1个pipeline layout、1个shader module和1条compute pipeline，共5项。失败candidate是否完成资源创建取决于失败发生在WGSL/workgroup验证前还是WGPU error-scope内；失败pass当前不会发布成功profile，不能把源码上界解释为运行时失败计数。
5. failing-first源码合同覆盖GPU context factory实现、generic executor的factory注入和bind-group旁路删除、cache四类create调用改道；精确rustfmt、scoped diff check与结构预算通过，`gpu.rs`/`compute_pipeline_cache.rs`/`generic_compute_executor.rs`分别为636/807/874行，均低于1000行硬上限。受管Cargo、真实WGPU、PNG/RDC、RenderDoc、300帧profile、VRAM与功耗仍pending，状态为`source_implemented_static_checks_passed_dynamic_validation_pending`。
6. 剩余覆盖缺口包括OIT和多个built-in renderer helper继续把owner-scoped raw device传入pipeline/bind helper；后续应按owner逐批迁移并在每批后重新量化，不能在覆盖收敛前把零计数解释为“本帧无原生资源创建”。本切片不修改任何缓存/预热算法，也不宣称性能收益。

#### PFO-4d5c pass factory coverage inventory（2026-08-31）

1. 扫描`advanced_lighting`、`environment`、`graph_execution`与`temporal`的生产导向Pass范围，`self.device`/`gpu.device`当前共有89处引用、分布于26个文件。该数字包含limits、validation、编码helper与测试源码字符串等引用，只是raw-device authority迁移上界，不是每帧create次数或性能结果。
2. 高频owner按引用数排序为：GPU context root 8；post-process effects 6；subsurface executors 6；mesh recording 6；SSR 5；OIT executors 5；generic compute 5；post-process computed resources 5；其余分布在terminal/temporal/OIT context、velocity、IBL、surface、particle、froxel、planar与light-cookie。4d5b已经把generic compute的native create改道，但保留limits/error-scope所需raw device读取，因此不能用文本命中数直接判断测量覆盖。
3. Unreal本地参考的`FRDGBuilder::CreateTexture/CreateBuffer/AddPass`把图资源声明与Pass执行绑定，`PipelineStateCache::GetAndOrCreateComputePipelineState`/`GetAndOrCreateGraphicsPipelineState`统一进行cache lookup、miss统计与可选异步编译；`DynamicRHI.h`也明确提示graphics PSO应先经过PipelineStateCache。Zircon应保持RDG materialization、generation-qualified pipeline owner和pass recorder三层，不把所有创建塞入一个全局锁或让feature直接持有raw device。
4. `dev/LumenInUE5.5.4WithComputeShader/AllocateProbeTraces.cpp`的复刻样例在`InitAllocateProbeTracesPass`中调用`BuildComputePipeline`，`ExecuteAllocateProbeTracesPass`只更新SRV并执行；这支持“初始化/预热与稳定帧dispatch分离”的结构方向，但该standalone D3D12样例没有Zircon的WGPU device-loss、RDG lifetime和多feature cache约束，不能直接复制其global static pass owner。
5. 后续测量迁移顺序固定为：先advanced feature generation-qualified pipeline owners（froxel/OIT/subsurface/planar/IBL），再core mesh/post-process helpers，最后per-pass transient buffer/bind-group路径。每一批必须同时保留device epoch、fallback、validation、graph resource声明和单submission语义，并更新factory覆盖矩阵；只有测量覆盖收敛后，300帧profile才可用于决定PSO prewarm、bind-group cache或参数buffer复用算法。
6. 本节仅完成结构调研和迁移排序，没有修改缓存算法，也没有把89/26解释为性能瓶颈。当前可执行实现仍止于4d5b的source-only状态；动态Cargo/WGPU、截图、RenderDoc、profile、VRAM与功耗证据继续pending。

#### PFO-4d5d Planar filter resource create measurement（2026-08-31）

1. current-source复审确认Planar filter已经使用`RenderPassDeviceEpochCache<(), PlanarReflectionFilterPipeline>`，device epoch变化固定先释放旧pipeline owner再重建；本切片没有修改cache key、命中、fallback、mip计算、shader、dispatch、graph resource声明或single-submission语义。
2. 原构造和encode路径共有6个raw-device create调用点：bind-group layout、shader module、pipeline layout、compute pipeline、per-mip参数buffer和per-mip bind group。它们现在全部经`RenderPassGpuResourceFactory`进入当前pass的饱和计数；`Texture::create_view`继续由输出纹理owner负责，不伪装成factory已覆盖的类别。
3. graph-execution共享`RenderPassGpuRecordingContext`把同一短生命周期的resource factory与command encoder成对提供，associated factory保持单态化，不在每个mip引入trait-object虚调用。executor只在词法作用域内持有native capability和pipeline-cache MutexGuard，编码完成后先释放两者，再记录dispatch profile。
4. Planar分辨率上限1024产生11个mip。稳定pipeline-cache hit的源码创建上界为11个buffer加11个bind group，共22项；cold/device-epoch miss额外增加1个bind-group layout、1个shader module、1个pipeline layout和1条compute pipeline，共26项。这些数字不是运行时采样，也不证明应缓存bind group、参数buffer或预热pipeline。
5. failing-first源码合同已由缺少factory/capability边界转为通过，并锁定6个raw create旁路为0、executor使用native capability、associated factory和profile前作用域释放；Planar三个Rust文件通过精确rustfmt检查。受管Cargo/WGPU、fresh PNG/RDC、RenderDoc、300帧create/encode profile、VRAM与功耗仍pending，因此本切片只标记source implemented，不宣称性能或视觉验收完成。

#### PFO-4d5e Froxel resource create measurement（2026-08-31）

1. current-source复审确认volumetric media inject、light scatter与integrate各自使用`RenderPassDeviceEpochCache<(), Pipeline>`，device epoch变化仍按release-then-create-publish重建完整owner。本切片不修改cache identity、fog volume筛选、temporal reprojection、light/shadow binding window、shader、dispatch或single-submission语义。
2. 三个pipeline原有22个raw-device create调用点：integrate 6个，light scatter 8个，media inject 8个。构造期layout/shader/pipeline/fallback buffer和稳定帧params buffer/bind group现在全部经`RenderPassGpuResourceFactory`计入当前pass；三个executor的`gpu.device`构造/encode旁路归零。
3. Planar与Froxel共用graph-execution唯一`RenderPassGpuRecordingContext`，associated factory保持单态化；仅`cfg(test)`提供`(Device, CommandEncoder)`适配器，产品构建不能借它恢复raw-device录制。每个executor在词法作用域内同时持有native capability和既有MutexGuard，encode结束后再记录profile/history。
4. 三条pass全开时，稳定cache hit的源码上界为integrate 2项、light scatter 3项、media inject 2项，无本地fog volume共7项；有本地volume时media inject增加一个上传buffer，共8项。三个cache同时cold/device-epoch miss时分别为6、8、7/8项，总计21/22项。这些数字不是运行时采样，不证明应缓存bind group/params buffer、池化volume buffer或预热pipeline。
5. failing-first跨owner源码合同已由缺少shared recording owner且保留raw creates转为通过，锁定22个create全部经factory、三个pipeline generic recording、三个executor native capability和旧`gpu.device`旁路为0。核心native/pipeline/executor文件为202、170/331/323、90/160/91行，均低于1000行拆分门。受管Cargo/WGPU、fresh PNG/RDC、RenderDoc、300帧profile、VRAM与功耗仍pending，本切片只标source implemented。

#### PFO-4d5f SSS resource create measurement（2026-08-31）

1. current-source复审确认SSS setup、scatter与recombine共享一个`RenderPassDeviceEpochCache<wgpu::TextureFormat, SubsurfacePipelines>`；设备代际或输出格式变化仍按release-then-create-publish重建完整bundle。本切片不修改prepared-frame上传、profile table解析、tile-list生成、indirect dispatch、viewport、shader、graph resource声明或single-submission语义。
2. 共享bundle原有12个raw-device create调用点：三个bind-group layout、两个compute pipeline各自的pipeline layout/shader/compute pipeline、一个recombine pipeline的pipeline layout/shader/render pipeline。三条pass的稳定帧bind group现在与构造期资源一起全部经`RenderPassGpuResourceFactory`进入当前pass计数，目标pipeline文件的raw-device create旁路为0。
3. SSS与Planar/Froxel共用graph-execution唯一`RenderPassGpuRecordingContext`；setup仍在compute pass之前通过同一encoder清零indirect args，scatter仍使用`dispatch_workgroups_indirect`，recombine仍保留render-region viewport和load/store语义。graph buffer lookup直接返回显式`BufferBinding<'a>`，不克隆native buffer或扩大binding生命周期。
4. setup/scatter/recombine全开时，稳定cache hit的源码创建上界为三个bind group，共3项；cold/device-epoch或target-format miss额外创建共享bundle的12项，共15项。这些数字不是运行时采样，不证明应预热pipeline、缓存bind group或改变SSS分阶段算法。
5. failing-first源码合同已由12个raw creates、旧device/encoder参数和binding clone wrapper转为通过，锁定共享factory/recording owner、三个executor native capability、clear-before-pass与indirect dispatch不变。核心native/executor/pipeline文件为202/269/390行，均低于1000行拆分门。精确rustfmt与静态合同通过；受管Cargo/WGPU、fresh PNG/RDC、RenderDoc、300帧profile、VRAM与功耗仍pending，本切片只标source implemented。

#### PFO-4d5g remaining advanced owner dependency audit（2026-08-31）

1. OIT不是一个可按`advanced_lighting/oit_buffers`目录独立迁移的owner。fragment/resolve/GPU-context当前共有15个显式raw create调用点，但fragment store还调用core mesh的`create_forward_shading_bind_group`和`ensure_oit_pipeline_admission_for_base_variant`；前者稳定帧创建volumetric params buffer与forward bind group，后者可能在variant miss时创建mesh shader/PSO。只改15个显式点会留下不可见旁路并错误产生“零创建”结论。
2. light-cookie的blit pipeline有5个显式create调用点，但`LightCookieAtlasResources`连同atlas texture、view、sampler和pipeline由`MeshPipelineCache::light_cookies`持有，graph executor只调用其`rebuild`。它应与mesh owner的device-epoch、atlas重建和bind-group测量一起迁移，而不是在executor层复制第二个cache。
3. IBL bake的layout/shader/pipeline cache同时被RDG bake pass和standalone environment-capture recorder消费；后者已有source-mip、params buffer、bind group和creation-micros专用报告。正确边界是一个共享双录制能力：RDG路径把七类native create计入pass factory，standalone路径继续计入capture report；不能把pass-local metrics强加到独立capture transaction，也不能复制两份pipeline cache。
4. 迁移顺序修正为：先收敛core forward bind-group、mesh PSO admission与相关buffer helper的factory入口，再完成OIT；light-cookie随mesh-owned atlas生命周期迁移；IBL最后以共享cache加双metrics sink迁移。三个owner当前都有大规模外来未提交改动，本切片不叠加生产代码修改，不认领这些变更。
5. 本节只记录current-source依赖审计和结构顺序，没有修改算法、shader、cache、dispatch、submission或资源生命周期，也不把调用点计数解释为运行时性能数据。受管Cargo/WGPU、fresh PNG/RDC、RenderDoc、300帧profile、VRAM与功耗继续pending。

## 性能与验收门槛

- 结构指标：一个 device generation owner、一个 physical submission timeline、一个 memory budget、一个 resource registry；产品 raw native owner计数降为0。
- CPU：记录每帧 upload call count、batch count、range count、uploaded bytes、merge ratio、staging allocations、resource creates/reuses/retires；分别报告 p50/p95/p99。
- GPU：记录 submission count、queue latency、copy/upload pass耗时、transient peak bytes、alias reuse、pipeline cache hit/miss和首帧编译停顿。
- 规模：至少覆盖 1K/10K/100K instance，稳定场景和 streaming burst分开；复杂度必须随 dirty range / visible work 增长，不随全部历史资源增长。
- 验收：Windows WGPU 真实窗口与 headless均通过；PNG、RenderDoc capture、300帧 profile、显存和功耗证据写入 `docs/tests/runtime/render`。没有这些结果不得宣称瓶颈消失、功耗接近其它引擎或算法达到最优规模。

## 当前非目标

- 本阶段不重写 Lumen GI 算法，不把 D3D12 专用 resource/barrier API复制到 WGPU 产品层。
- 不为了减少源码命中数把所有资源都塞进一个全局锁；锁竞争必须通过分批 materialization、frame-local staging和短临界区控制。
- 不用 source-only guard替代 Cargo/WGPU/RenderDoc/performance验收；静态切片只标 source implemented。
