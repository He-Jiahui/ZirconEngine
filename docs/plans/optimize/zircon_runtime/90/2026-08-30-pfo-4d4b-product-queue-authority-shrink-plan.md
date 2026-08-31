# PFO-4d4b Product Queue Authority Shrink

## 状态

- 日期：2026-08-30
- 范围：Runtime90 PFO-4d4，scene resource preparation与product GPU timer构造的raw WGPU queue权限收敛
- 当前状态：`runtime90_pfo_4d4b_source_implemented_static_checks_passed_dynamic_validation_pending`
- 证据边界：本文是current-source与仓库内Unreal RHI owner顺序复审，不是Cargo、WGPU、PNG、RDC、profile、功耗或性能验收。

## 当前源码事实

1. `ResourceStreamer::ensure_scene_resources`接收`&wgpu::Queue`，但只把它传给`ensure_material_for_frame`；后者继续传给`ensure_material_internal`，最终参数名已是`_queue`且没有任何读取。产品上传已由`RenderFrameSubmissionTransaction`与typed buffer/texture upload owner承担，因此这是无行为的历史权限透传。
2. `GpuPassTimer::try_new_product`接收raw queue只为读取一次`get_timestamp_period`。同一事实已由唯一`WgpuRenderDevice`在构造时捕获，并随production diagnostic query delivery发布；产品renderer没有理由持有queue authority来读取它。
3. Standalone UI仍使用legacy `GpuPassTimer::try_new(device, queue, ...)`创建自有query set/readback staging；该迁移受device-level query consumer routing依赖，本切片不把shared UI结果从scene中央router中抢走，也不恢复第二个poll owner。

## 参考与决策

Unreal的适用约束是RenderCore通过RHI command/query owner消费设备事实和提交结果，feature preparation不取得native queue。Zircon继续复刻“immutable device fact + frame transaction + finalized packet”的owner顺序，不复制D3D12 queue或线程模型。

本切片只执行两个hard cut：

1. scene resource preparation及其frame material入口删除`&wgpu::Queue`参数；测试专用`ensure_material`可暂时保留旧fixture形态，但不能把queue传入production internal owner。
2. `WgpuRenderDevice`公开只读timestamp period事实；`GpuPassTimer::try_new_product`改接收该值，不再接收queue。Legacy standalone构造器不变。

禁止增加通用native queue accessor、callback facade或第二套timestamp事实。该切片不会减少native submission，也不声称性能改善。

## 复杂度与验证

- scene preparation仍为现有资源/材质访问规模，不增加扫描、分配或锁；删除参数透传是`O(1)`接口收敛。
- timestamp period在device bootstrap读取一次，product timer构造只复制一个`f32`。
- failing-first guards要求product scene preparation和product timer constructor都不出现`wgpu::Queue`；同时保留legacy UI构造器。
- 精确rustfmt、scoped diff、owner行数和locked metadata属于静态证据。受管Cargo仍受已知`cargo_reuse_target_mismatch`门阻断，动态验收继续开放。

## 源码实施结果

1. `ensure_scene_resources -> ensure_material_for_frame -> ensure_material_internal`产品链已删除`&wgpu::Queue`。测试专用`ensure_material`仍接收fixture queue以保持现有测试调用面，但不再把它传入内部产品owner。
2. `WgpuRenderDevice`发布构造时固化的只读`timestamp_period_ns`；两个product timer构造点消费该事实，`GpuPassTimer::try_new_product`不再接收queue或自行调用`get_timestamp_period`。Legacy `try_new(device, queue, ...)`保持不变。
3. failing-first五项在实现前均为false；实现后扩展合同7/7通过。精确rustfmt、scoped diff与locked metadata通过；owner物理行数为device 798、timer 615、scene resource prepare 726、material 945、direct frame 459、compiled frame owner 392，未放宽既有800行review门，material大owner本次净减行。
4. 本切片没有改变上传批次、native submission数量、query result routing或资源创建。renderer core/bootstrap构造链随后由主PFO-4d计划中的PFO-4d4c负责；复审确认`gpu_upload_diagnostics(&device, &queue)`只存在于`cfg(test)`，不是产品raw queue consumer。没有Cargo、真实WGPU、PNG/RDC、profile或功耗证据，不标accepted milestone。
