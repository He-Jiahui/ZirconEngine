# MD-M3 MeshDraw 重放状态缓存正确性复审

日期: 2026-08-26  
状态: 代码修正已落地，等待 managed compile、WGPU 产品对拍、RenderDoc 与 WPR/xperf 验证；不得标记 MD-M3 完成

## 1. 复审范围

本轮先复审状态去重与 indirect replay 的整体契约，不把局部计数微调当作性能优化。范围包括:

- `mesh/mesh_pass/replay.rs`: pipeline、bind group、geometry 状态缓存与 direct/indirect replay。
- `mesh/mesh_pass/mesh_draw_command.rs`: replay 持有的 WGPU bind/geometry 身份与生命周期。
- `mesh/mesh_draw/mesh_pass_batch.rs`: 当前帧 `MeshDraw` 到命令 payload 的资源投影。
- deferred、prepass、OIT、overlay、shadow、velocity、TAA reactive mask 的全部 `replay_command_stream` 调用点。
- Unreal `FMeshDrawCommandStateCache`、`SubmitDrawBegin`/`SubmitDrawEnd` 和 visible mesh command 提交流程。

## 2. Unreal 对齐结论

主要参考:

- `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Public/MeshPassProcessor.h:903`: `FMeshDrawCommandStateCache` 用不可达初值防止首 draw 被错误过滤；pipeline 切换会清空 vertex stream 与 shader binding 状态。
- `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshPassProcessor.cpp:1391`: indirect begin 与 direct draw 共用同一套状态准备契约。
- `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshPassProcessor.cpp:1481`: `SubmitDraw` 只在状态准备成功后提交实际 draw。
- `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshPassProcessor.cpp:1663`: 一个提交区间拥有一个 state cache，并逐 visible command 投影 view-local 参数。

Zircon 的固定方向:

1. 状态缓存只能跳过已经在当前 render pass 中真实建立、且身份与生命周期可证明的状态。
2. pipeline 改变或外部 pipeline/bind/geometry 录制后，相关追踪状态必须失效。
3. static cache 保存 generation-owned 不可变 submission payload；current command 保存 source、sort、GPUScene span 等 view-local 投影。
4. global indirect batch 不能直接承载 per-view entity predicate。shadow 最终应构建 view-local sealed range/compaction plan，而不是让首命令的 predicate 代表整个 batch。

## 3. 已确认问题与修正

| 问题 | 根因 | 当前修正 | 状态 |
|---|---|---|---|
| unbatched indexed-indirect 被统计为 direct | `draw_indexed` 无条件增加 `direct_draw_call_count` | 按 `MeshDrawArgs::is_indirect()` 分类 direct 与 per-draw indirect | 已落地，待 managed test |
| static payload 可保留短命地址身份 | `MeshDraw` 字段地址被写入 `MeshBindHandle.id`，跨帧 allocator 地址可复用 | `MeshBindHandle::new` 改为进程内非零、单调、溢出即失败的稳定 ID；clone 保持 ID | 已落地，待 managed test |
| owned 与 borrowed bind 身份可数值碰撞 | 两种来源共用裸 `u64` namespace | replay 追踪改为 `Owned(u64)` / `Borrowed(u64)` typed identity | 已落地，待 managed test |
| OIT pipeline 解析失败污染 state cache | `should_set_pipeline` 先记录 variant，失败分支未失效 | unsupported shader 返回前调用 `invalidate_state_after_external_pipeline` | 已落地，待 managed test |
| shadow view filter 对 indirect batch 只检查首命令 | replay 以 `args_count` 跳过整个 batch，predicate 只执行一次 | 有 view-local filter 时使用 `MeshDrawCommandStream::without_indirect()`，逐命令过滤；无 filter 仍保留 indirect execution | 已落地，待 managed test |

## 4. 当前算法与复杂度

无 view-local filter 的普通路径保持:

- command replay: `O(N + B)`，其中 `N` 为命令数，`B` 为 indirect batch 数。
- state lookup: pipeline、4 个 tracked bind slot、geometry 均为常数时间。
- command stream 降级: 只复制两个 slice/reference 字段，不克隆命令、不分配。

shadow 有 view-local filter 时当前为正确性优先的 MVP 路径:

- predicate 对每条 command 执行一次，复杂度 `O(N log V)`；`V` 为 `BTreeSet<EntityId>` 中的 view-visible entity 数。
- direct command 直接重放；原生 `IndexedIndirect` command 仍逐条调用其 indirect args，不会被误算为 direct。
- global generated multi-draw 暂不使用，避免首命令可见性错误扩散到整批。

这不是最终性能结构。目标结构是 `view visibility -> view-local command/range projection -> view-local indirect compaction -> sealed replay ranges`，使过滤在 batch 形成前完成，并把提交复杂度恢复为 `O(N + B_view)`。在该 owner 建立前，不允许用全局 batch 猜测 shadow view 可见性。

## 5. 回归约束

已新增或加强:

- `mesh_draw_command_replayer_classifies_unbatched_direct_and_indirect_draws`
- `mesh_draw_command_replayer_does_not_alias_owned_and_borrowed_bind_ids`
- `mesh_bind_handle_ids_are_nonzero_unique_and_stable_across_clone`
- `unsupported_mesh_pipeline_invalidates_replay_state_before_skipping_draw`
- `shadow_view_filter_disables_global_indirect_batches_before_replay`

静态检查还必须确认:

- production `MeshBindHandle::new` 不再接收 raw address ID。
- `ref_id` 不再存在于 mesh command 投影路径。
- 所有可失败的 pipeline resolve 分支在跳过 draw 前使 replay state 失效。
- 只有存在 view-local filter 的 shadow stream 关闭 global indirect execution。

## 6. 性能验证协议

本轮不声明性能提升。下一次继续优化前先采集旧/新路径的同场景基线，产物只写入 `E:\zircon-profiles\render\md-m3\` 或 `docs/tests/runtime/render/`:

1. WPR/xperf: render thread 的 mesh replay inclusive/exclusive CPU、context switch、CPU frequency 与 package power 可用计数。
2. RenderDoc: direct、per-draw indirect、fixed multi-draw、indirect-count 调用数；pipeline/bind/vertex/index 设置序列。
3. 引擎 stats: `draw_call_count`、各 indirect 分类、`state_change_count`、bind set/skip、material set/skip。
4. 场景矩阵: 重复材质 opaque、混合 pipeline、多个 shadow atlas slot 且各 view 可见集不同、OIT unsupported shader 后跟同 variant command。
5. 对比条件: 固定分辨率、adapter、warmup 帧、采样帧数、present mode 与 shader cache 状态；报告中记录中位数、P95 和变异系数。

只有当 managed compile/test、真实 WGPU 产品、RenderDoc 绑定序列和性能基线全部闭合，才允许判断瓶颈是否消失或功耗是否接近参考引擎经验值。

## 7. 待办与验收门

- 待 managed `cargo test -p zircon_runtime mesh --locked` 与相关 render-product 回归。
- 待不同 shadow view 可见集的真实像素产品与 PNG 证据。
- 待 RenderDoc 抓帧证明 filtered shadow 不再提交其他 view 的 entity，并确认普通路径 multi-draw 未退化。
- 待 WPR/xperf 前后数据；当前不得写入任何虚构耗时、功耗或提升百分比。
- 待 view-local sealed range/compaction owner 的架构与基线完成后，再实施最终 shadow indirect 优化。

