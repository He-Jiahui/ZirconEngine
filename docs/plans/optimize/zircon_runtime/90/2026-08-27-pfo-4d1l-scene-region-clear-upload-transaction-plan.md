# PFO-4d1l Scene Region Clear Upload Transaction Plan

## 状态

- 日期：2026-08-27
- 当前状态：`source_implemented_static_checks_passed_dynamic_validation_pending`
- 范围：Runtime90 PFO-4d1l，compiled scene 的 region-scoped color/depth clear 参数发布
- 前置：PFO-4d1a-k 已建立每帧唯一 `FrameBufferUpload` 事务；动态 WGPU、PNG、RenderDoc、profile 与功耗仍 pending。
- 证据边界：本切片修正 native queue 权限和失败帧 side effect，不把一个 16-byte 源码写入点的消除解释为运行时瓶颈或性能收益。

## 当前代码审计

1. `SceneRegionClearResources::record_frame_clear(...)` 在 compiled graph stage 录制开始时解析 `ViewportSceneClearPlan`，region clear 采用一次 fullscreen-triangle draw 和 viewport/scissor，保持 split-view 正确性。
2. color clear 在录制 pass 前直接调用 `queue.write_buffer(&self.color_buffer, ...)`。若后续任一 graph stage 失败，command encoder 会被丢弃，但该 queue side effect 已经发布；它绕过同帧 `FrameBufferUpload` 的 admission、ticket、ledger 和统一统计。
3. depth-only clear 不需要参数上传；no-clear/empty-region 也不应制造空 batch。color-only 与 color+depth 都只需要一个 16-byte `SceneRegionClearColorUniform`。
4. `RenderGraphStageExecution` 已拥有 pass-local `WgpuBufferUploadBatch`，并只在所有 graph stage 成功后由 outer frame owner取出，和 frame uploads 合并后调用一次 `enqueue_copy_buffer_upload_batch(...)`。失败分支不会受理该批次。

## Unreal 对齐

- `dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphUtils.cpp:672` 的 `AddClearRenderTargetPass(...)` 对完整匹配 clear value 使用 render-pass clear action；对 custom viewport 在 RDG 中登记 raster pass，并在 pass lambda 内通过 `DrawClearQuad(...)` 执行。
- Zircon 继续保留 region triangle，因为 camera-stack/split-view 不能用 whole-view attachment clear。对齐点是 clear work 和参数生命周期必须属于 graph/frame owner，而不是把平台 queue 暴露给 clear feature。
- 本切片不重写 camera clear policy，也不尝试把所有 full-target clear 融合问题混入该 owner hard cut；Render01 `PERF-MVP-394` 仍负责 full-target attachment-clear 融合验收。

## 实施方案

1. `record_frame_clear(...)` 删除 `&wgpu::Queue` 参数并返回 `WgpuBufferUploadBatch`。
2. color 存在且 region 非空时，以 `WgpuBufferUpload::from_bytes(...)` 准备唯一 16-byte immutable payload；depth-only、empty/no-clear 返回空 batch。
3. `RenderGraphStageExecution` 增加窄的 `append_buffer_uploads(...)` 所有权移动入口；scene clear 录制完成后立即把 prepared batch附着到当前 graph execution。
4. outer frame owner保持既有顺序：graph全部成功 -> take graph uploads -> append frame uploads -> backend受理一个 batch/ticket -> submit graph command buffers。graph失败时 upload随 execution丢弃。
5. 测试更新为显式受理 returned batch 后再提交 command buffer，并增加 depth-only 不产生上传与生产段不再包含 `queue.write_buffer` 的 source guard。

## 复杂度与预期数据

- CPU packing：固定 16 bytes，`O(1)` 时间/空间。
- upload count：color clear 仍为一个逻辑 range；本切片不伪造 native driver call 减少，因为它会并入同帧批次，实际驱动行为需动态 trace。
- submission：不新增 queue submit、poll 或 completion owner；沿用一个 `RenderFrameSubmissionProducer::FrameBufferUpload` ticket。
- 正确性收益：graph录制失败不再留下 clear-color buffer side effect；参数和消费 command buffer由同一成功帧事务发布。

## 验证门槛

- 本阶段：focused rustfmt、旧 raw queue调用/新 batch顺序/source guard、scoped diff check。
- 测试阶段：focused Cargo/WGPU scene-region-clear product test，颜色/深度/region像素覆盖。
- 里程碑验收：真实窗口/产品 PNG、RenderDoc 中 region clear pass与上传顺序、300帧 upload/profile/power 数据。缺少动态证据时状态必须保持 source implemented / dynamic pending。

## 当前源码结果

1. `SceneRegionClearResources::record_frame_clear(...)` 已删除 queue 参数，color clear 返回一个 16-byte `WgpuBufferUpload`；depth-only、no-clear 与 empty-region 路径返回空 batch。
2. clear draw录制后，prepared batch通过 `RenderGraphStageExecution::append_buffer_uploads(...)` 所有权移动到 graph execution。outer frame owner仅在所有 stage 成功后取出并合入唯一 `FrameBufferUpload`；失败路径不会提前发布 clear-color side effect。
3. focused source guard锁定 `record -> append` 与 `graph success -> take -> backend accept` 顺序，生产段 `queue.write_buffer` 和 queue参数均为0。
4. 三个精确Rust文件通过`rustfmt --edition 2021 --config skip_children=true --check`；scoped `git diff --check`通过。新鲜 scene-renderer 非测试扫描由本切片前12次直接写/11文件降为11次/10文件。
5. 未运行Cargo、真实WGPU、PNG、RenderDoc、profile或功耗验收，因此不声明运行时性能收益，也不关闭Render01 `PERF-MVP-394` full-target clear融合任务。
