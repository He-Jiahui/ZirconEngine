# PFO-4d1j Mesh Indirect Two-Phase Upload Plan

## 状态

- 日期：2026-08-27
- 范围：Runtime90 PFO-4d1j，mesh pass indirect args / compaction metadata 上传事务
- 当前状态：`source_implemented_static_checks_passed_dynamic_validation_pending`
- 前置：PFO-4d1a-i 已把 frame/RDG 参数写入收敛到唯一 `FrameBufferUpload` ticket；动态 WGPU、PNG、RenderDoc、profile、功耗与 benchmark 仍 pending。
- 证据边界：本文记录源码调用图、失败语义、复杂度与待验证指标，不把源码计数解释为驱动耗时或功耗结果。

## 当前问题

`MeshIndirectDrawWorkspace::prepare` 在 compiled scene graph 录制前，针对九个 mesh phase 直接执行两类 CPU 到 GPU 写入：

1. indirect args buffer；
2. indirect compaction metadata buffer。

两类写入共用 `write_changed_pod_ranges`。它线性扫描 committed CPU shadow，按每段连续脏元素调用一次 `Queue::write_buffer`，然后立即修改 shadow。该结构有三个系统性问题：

1. 写入绕过 frame-owned `WgpuBufferUploadBatch` 与唯一 `FrameBufferUpload` ticket；graph validation、pass materialization 或后续录制失败时，GPU side effect 已经发生。
2. CPU shadow 在 backend admission 前已经发布。失败重试会把尚未被 frame transaction 接受的数据误判为 committed；buffer 扩容后若本帧失败，下一帧甚至可能跳过新 buffer 的完整初始化。
3. 交错脏元素最坏产生 `O(n)` 次 native write。当前没有真实 profile 数据支持任意 gap/byte 阈值，因此本阶段不得凭经验加入魔数式合并策略。

## Unreal 对照

1. `dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h` 的 `QueueBufferUpload` 把 payload 生命周期延长到 graph execution，并保证 pass 执行前完成上传；feature 不直接拥有平台 queue。
2. `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Skinning/SkinningSceneExtension.cpp` 使用 persistent growable buffers 与 `ResizeAndUploadTo`，并通过一次 graph builder blackboard publication 避免同一 builder 重复完成上传。
3. `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Skinning/AnimRuntimeTransformProvider.cpp` 把 dirty current/previous transform 写入 scatter uploader，由 graph pass 统一执行，而不是逐实体直接提交平台写入。
4. Lumen 的 indirect args 由 RDG buffer 与 compute pass 生成/消费；`dev/LumenInUE5.5.4WithComputeShader` 只用于核对显式 resource/command 顺序，不能作为绕过 Zircon RHI owner 的接口模板。

## 目标状态机

每个 args / metadata target 使用独立的 `PodRangeUploadShadow<T>`：

- `committed`：仅代表已被 backend 接受并登记到 frame ledger 的 CPU 镜像；
- `staged`：当前 preparation 的可覆盖 scratch，跨失败帧复用 capacity；
- `staged_revision`：区分被后续 preparation 覆盖的旧 token；
- `committed_buffer_revision`：保证新建/扩容 buffer 在首次成功帧执行完整初始化。

`prepare(current, buffer_revision)` 只执行以下操作：

1. 对 committed shadow 做 `O(n)` 顺序比较；
2. 把 current 写入可复用 staged shadow；
3. 为每段精确连续脏区创建共享 immutable payload range；
4. 返回 batch 与 revision-qualified commit token，不修改 committed shadow。

帧 owner 按如下顺序处理：

```text
mesh indirect prepare
  -> append frame upload batch
  -> graph recording succeeds
  -> backend accepts one FrameBufferUpload ticket
  -> submission ledger accepts ticket
  -> commit indirect shadows
  -> submit scene packet
```

任一 accept 前失败只丢弃 preparation；下一帧仍以旧 committed shadow 比较。buffer revision 不匹配时必须 full upload，即使该 buffer 是前一失败帧创建的。

## 算法与规模

- dirty detection：每个 target `O(n)` 时间，稳定帧不分配 payload、不生成 range。
- staged shadow：每个 target 保留一份可复用 `Vec<T>`，空间 `O(n)`；失败重试覆盖 staged capacity，不逐帧累积。
- payload：每个发生实际写入的 target 最多一个 `Arc<[u8]>` owner，各 dirty range 只保存 source range 与 native buffer clone；payload 构造 `O(n)`，不会按 range 重复复制。
- merge：本阶段保留精确连续脏区，range count 最坏 `O(n)`，但所有 range 归入一个 frame ticket。待 RenderDoc/CPU profile 获得 write-call fixed cost、dirty span 分布与 uploaded-byte tradeoff 后，再决定 bounded-range 或 scatter-copy 阈值。
- phase lookup：使用固定九 phase enum 直接定位 workspace；commit 为 `O(active phases)`，不扫描历史资源、不引入锁。

## 实施步骤

1. 为纯 dirty-range 算法加入稳定、增长、缩短、交错与 force-full 测试。
2. 把 `write_changed_pod_ranges` 替换为无 queue 的 `PodRangeUploadShadow::prepare/commit`。
3. args 与 compaction metadata 各自维护 buffer revision；扩容失败后的重试必须 full upload。
4. `MeshIndirectDrawWorkspace::prepare` 返回 execution、stats 与 owned prepared upload；compiled frame 把 batch 合并到 `frame_buffer_uploads`。
5. 只在 backend enqueue 和 `FrameBufferUpload` ledger 都成功后 commit；所有早退路径自然丢弃 token。
6. 增加 source contract，禁止 mesh indirect production path 接收 queue 或调用 `write_buffer`。
7. 更新 PFO-4d 主计划、RDG transaction 计划和 render module 文档；重新统计剩余 production direct writes。

## 动态验收（pending）

- Windows WGPU：稳定/增长/缩短/graph failure retry 的 indirect draw 正确性。
- 规模：1K/10K/100K draws，记录 dirty detection CPU p50/p95/p99、payload bytes、range count、upload ticket count与 native write count。
- RenderDoc：确认 upload 先于所有 indirect consumer，失败重试的新 buffer 完整初始化，scene frame 仍只有一个 graphics flush owner。
- 功耗：固定相机稳定场景与高 churn 场景分别记录；没有实际工具数据不得宣称瓶颈消失或达到最优阈值。

## 当前源码结果

1. `write_changed_pod_ranges` 已删除；args 与 compaction metadata 统一使用无 queue 权限的 `PodRangeUploadShadow<T>`。稳定数据不创建 payload，变化数据的所有 dirty ranges 共享一个完整 current payload owner。
2. committed/staged shadow 已分离。每次有效 preparation 返回 staged revision 与 buffer revision token；commit 交换两个可复用 `Vec<T>`，因此不会逐帧累积 scratch。未受理的新建 buffer 因 committed buffer revision 不匹配，在下一帧仍强制完整上传。
3. 九个固定 mesh phase 使用 enum 直接定位 workspace。`MeshIndirectWorkspacePreparedUpload` 随 execution/stats 返回，在 graph 录制前追加到 `frame_buffer_uploads`；只有唯一 backend upload ticket 和 `FrameBufferUpload` ledger 都成功后才 commit。
4. 三个 mesh indirect 生产模块中的 `wgpu::Queue` 与 `queue.write_buffer` 均为 0。纯算法测试覆盖稳定、增长/连续变化、仅缩短、交错脏区与 force-full，workspace 测试新增未 commit 新 buffer 的完整重试契约。
5. 精确修改的 Rust 文件已执行 `rustfmt --edition 2021 --config skip_children=true`；scoped `git diff --check` 通过，仅报告仓库既有 LF/CRLF 提示。新鲜扫描把 scene-renderer 生产候选直接写从 12 次/10 文件降为 11 次/9 文件。
6. 按当前里程碑验证策略未运行 Cargo 或动态 GPU 验收；Windows WGPU、PNG、RenderDoc、1K/10K/100K profile、native write 分布和功耗仍 pending，因此尚未决定 dirty-gap 合并阈值。
